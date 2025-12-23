//! Minimal ELF64 (RISC-V) loader for user programs, using `goblin`.
//!
//! Responsibilities:
//! - Parse ELF headers and PT_LOAD segments
//! - Map segments at their `p_vaddr` with Sv39 (U=1; R/W/X from ELF flags)
//! - Copy file bytes; zero BSS tail
//! - Build a user stack (argc/argv/envp) and return entry & SP
//!
//! Safety:
//! - All writes to user VAs happen with SSTATUS.SUM set via `with_sum`
//! - Page mapping/copying uses the kernel’s identity mapping to touch PAs

#![allow(clippy::too_many_arguments)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, rust_2018_idioms, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::missing_panics_doc, clippy::missing_errors_doc)]

use core::mem::size_of;

use goblin::elf::{header, program_header, Elf};
use riscv::register::sstatus;

use crate::sv39::{self, PTE_A, PTE_D, PTE_R, PTE_U, PTE_V, PTE_W, PTE_X};

/// Loader errors (compact, no strings in the happy path)
#[derive(Debug, Copy, Clone)]
pub enum ElfLoadError {
    Short,
    BadMagic,
    Not64LE,
    NotRiscv,
    PhOutOfBounds,
    SatpNotSet,
    SegmentOverflow,
}

pub struct Loaded {
    pub entry_va: usize,
    pub user_sp: usize,
    pub argc: usize,
    pub argv_va: usize,
    pub envp_va: usize,
    pub brk: usize,
}

/* ---------- SUM-guarded user writes ---------- */

#[inline(always)]
unsafe fn with_sum<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe { sstatus::set_sum(); }
    let r = f();
    unsafe { sstatus::clear_sum(); }
    r
}

unsafe fn write_user_bytes(va: usize, bytes: &[u8]) {
    unsafe { with_sum(|| {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), va as *mut u8, bytes.len());
    }) };
}

unsafe fn write_user_usize(va: usize, val: usize) {
    let bytes = unsafe { core::slice::from_raw_parts(
        (&val as *const usize) as *const u8,
        core::mem::size_of::<usize>(),
    ) };
    unsafe { write_user_bytes(va, bytes) };
}

/* ---------- mapping & copying helpers ---------- */

/// Map one VA page to a fresh PA page with flags, return PA.
unsafe fn map_user_page(root: *mut u64, va_page: usize, flags: u64) -> usize {
    let pa = unsafe { sv39::alloc_user_page() };
    unsafe { sv39::map_4k(root, va_page, pa, flags) };
    pa
}

/// Copy `len` bytes into a **physical** address (visible via kernel identity map).
unsafe fn memcpy_pa(dst_pa: usize, src: *const u8, len: usize) {
    unsafe { core::ptr::copy_nonoverlapping(src, dst_pa as *mut u8, len) };
}

/// Zero `len` bytes at a **physical** address.
unsafe fn memzero_pa(dst_pa: usize, len: usize) {
    unsafe { core::ptr::write_bytes(dst_pa as *mut u8, 0, len) };
}

/// Map ELF p_flags to PTE flags (always V|U|A; add D if W).
#[inline(always)]
pub fn pte_flags_from_pf(pf: u32) -> u64 {
    let mut f = PTE_V | PTE_U | PTE_A;
    if (pf & 0x4) != 0 { f |= PTE_R; }           // PF_R
    if (pf & 0x2) != 0 { f |= PTE_W | PTE_D; }   // PF_W
    if (pf & 0x1) != 0 { f |= PTE_X; }           // PF_X
    f
}

/* ---------- public API ---------- */

pub fn load_user_elf(
    image: &[u8],
    user_stack_top_va: usize,
    user_stack_bytes: usize,
    argv: &[&str],
    envp: &[&str],
) -> Result<Loaded, ElfLoadError> {
    unsafe {
        if image.len() < size_of::<goblin::elf::header::Header>() {
            return Err(ElfLoadError::Short);
        }

        // Parse ELF
        let elf = Elf::parse(image).map_err(|_| ElfLoadError::BadMagic)?;

        // Basic validation (class + endianness)
        let ident = &elf.header.e_ident;
        if &ident[..4] != b"\x7FELF" {
            return Err(ElfLoadError::BadMagic);
        }
        if ident[header::EI_CLASS] != header::ELFCLASS64
            || ident[header::EI_DATA] != header::ELFDATA2LSB
        {
            return Err(ElfLoadError::Not64LE);
        }
        if elf.header.e_machine != header::EM_RISCV {
            return Err(ElfLoadError::NotRiscv);
        }

        let root = sv39::root_pt();
        if root.is_null() {
            return Err(ElfLoadError::SatpNotSet);
        }

        // Map PT_LOAD segments
        let page = 4096usize;
        let mut max_brk = 0usize;

        for ph in &elf.program_headers {
            if ph.p_type != program_header::PT_LOAD {
                continue;
            }

            let p_vaddr  = ph.p_vaddr as usize;
            let p_offset = ph.p_offset as usize;
            let p_filesz = ph.p_filesz as usize;
            let p_memsz  = ph.p_memsz as usize;
            let p_flags  = ph.p_flags as u32;

            // Safety bounds: file bytes must lie within the ELF image
            if p_offset
                .checked_add(p_filesz)
                .map(|end| end <= image.len())
                .unwrap_or(false) == false
            {
                return Err(ElfLoadError::PhOutOfBounds);
            }

            if p_memsz == 0 {
                continue; // nothing to map
            }

            // Update max_brk
            let seg_end = p_vaddr + p_memsz;
            if seg_end > max_brk {
                max_brk = seg_end;
            }

            // Page-aligned mapping range
            let va0   = p_vaddr & !(page - 1);
            let head  = p_vaddr - va0;
            let vaend = (p_vaddr + p_memsz + page - 1) & !(page - 1);

            let flags = pte_flags_from_pf(p_flags);

            let mut cur_va = va0;
            let mut copied = 0usize;

            while cur_va < vaend {
                let pa = map_user_page(root, cur_va, flags);

                // Content for this VA page
                let page_off   = if cur_va == va0 { head } else { 0 };
                let page_space = page - page_off;

                let file_left  = p_filesz.saturating_sub(copied);
                let file_chunk = core::cmp::min(file_left, page_space);

                // Copy file bytes
                if file_chunk > 0 {
                    let src = image.as_ptr().wrapping_add(p_offset + copied);
                    memcpy_pa(pa + page_off, src, file_chunk);
                    copied += file_chunk;
                }

                // Zero the remaining BSS in this page
                let seg_end   = p_vaddr + p_memsz;
                let page_end  = cur_va + page;
                let mem_covered = if page_end > seg_end { seg_end.saturating_sub(cur_va) } else { page };
                if mem_covered > page_off + file_chunk {
                    let zero_len = mem_covered - (page_off + file_chunk);
                    memzero_pa(pa + page_off + file_chunk, zero_len);
                }

                cur_va += page;
            }
        }

        // Build user stack
        let (sp, envp_va, argv_va, argc) =
            setup_user_stack(user_stack_top_va, user_stack_bytes, argv, envp, root)?;

        // Align brk to page boundary for safety/simplicity, or keep it exact?
        // Usually brk starts page-aligned or just after data.
        // Let's keep it exact, sys_brk will handle page alignment.

        Ok(Loaded {
            entry_va: elf.header.e_entry as usize,
            user_sp: sp,
            argc,
            argv_va,
            envp_va,
            brk: max_brk,
        })
    }
}

/* ---------- user stack layout ---------- */

unsafe fn setup_user_stack(
    user_stack_top_va: usize,
    user_stack_bytes: usize,
    argv: &[&str],
    envp: &[&str],
    root: *mut u64,
) -> Result<(usize, usize, usize, usize), ElfLoadError> {
    // Map stack pages U=RW
    let stack_pages = (user_stack_bytes + 4095) / 4096;
    let mut va = (user_stack_top_va - stack_pages * 4096) & !4095;
    for _ in 0..stack_pages {
        unsafe { map_user_page(root, va, PTE_V | PTE_U | PTE_R | PTE_W | PTE_A | PTE_D) };
        va += 4096;
    }

    // Strings at top, then pointer vectors, then argc.
    let mut sp = user_stack_top_va;
    sp -= 16; // tiny guard so trailing NUL sits inside the last page

    // env strings
    let mut env_ptrs: heapless::Vec<usize, 32> = heapless::Vec::new();
    for &s in envp {
        let b = s.as_bytes();
        sp -= b.len() + 1;
        unsafe { write_user_bytes(sp, b);
        write_user_bytes(sp + b.len(), &[0]); }
        env_ptrs.push(sp).map_err(|_| ElfLoadError::SegmentOverflow)?;
    }

    // argv strings
    let mut arg_ptrs: heapless::Vec<usize, 32> = heapless::Vec::new();
    for &s in argv {
        let b = s.as_bytes();
        sp -= b.len() + 1;
        unsafe { write_user_bytes(sp, b);
        write_user_bytes(sp + b.len(), &[0]); }
        arg_ptrs.push(sp).map_err(|_| ElfLoadError::SegmentOverflow)?;
    }

    // 16-byte alignment before vectors
    sp &= !15;

    // envp NULL + vector (in reverse so argv[0] is lowest)
    sp -= size_of::<usize>();
    unsafe { write_user_usize(sp, 0) };
    for &p in env_ptrs.iter().rev() {
        sp -= size_of::<usize>();
        unsafe { write_user_usize(sp, p) };
    }
    let envp_va = sp;

    // argv NULL + vector
    sp -= size_of::<usize>();
    unsafe { write_user_usize(sp, 0) };
    for &p in arg_ptrs.iter().rev() {
        sp -= size_of::<usize>();
        unsafe { write_user_usize(sp, p) };
    }
    let argv_va = sp;

    // argc
    let argc = arg_ptrs.len();
    sp -= size_of::<usize>();
    unsafe { write_user_usize(sp, argc) };

    // final align (some ABIs like it; harmless otherwise)
    sp &= !15;

    Ok((sp, envp_va, argv_va, argc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pte_flags_from_pf_no_perms() {
        // No permissions (0)
        let flags = pte_flags_from_pf(0);
        assert_eq!(flags & PTE_V, PTE_V); // Valid
        assert_eq!(flags & PTE_U, PTE_U); // User
        assert_eq!(flags & PTE_A, PTE_A); // Accessed
        assert_eq!(flags & PTE_R, 0);     // Not readable
        assert_eq!(flags & PTE_W, 0);     // Not writable
        assert_eq!(flags & PTE_X, 0);     // Not executable
    }

    #[test]
    fn test_pte_flags_from_pf_read_only() {
        // Read-only (PF_R = 0x4)
        let flags = pte_flags_from_pf(0x4);
        assert_eq!(flags & PTE_R, PTE_R);
        assert_eq!(flags & PTE_W, 0);
        assert_eq!(flags & PTE_X, 0);
        assert_eq!(flags & PTE_D, 0); // No dirty bit for read-only
    }

    #[test]
    fn test_pte_flags_from_pf_read_write() {
        // Read-write (PF_R | PF_W = 0x6)
        let flags = pte_flags_from_pf(0x6);
        assert_eq!(flags & PTE_R, PTE_R);
        assert_eq!(flags & PTE_W, PTE_W);
        assert_eq!(flags & PTE_D, PTE_D); // Dirty bit set for writable
        assert_eq!(flags & PTE_X, 0);
    }

    #[test]
    fn test_pte_flags_from_pf_read_exec() {
        // Read-execute (PF_R | PF_X = 0x5)
        let flags = pte_flags_from_pf(0x5);
        assert_eq!(flags & PTE_R, PTE_R);
        assert_eq!(flags & PTE_X, PTE_X);
        assert_eq!(flags & PTE_W, 0);
        assert_eq!(flags & PTE_D, 0);
    }

    #[test]
    fn test_pte_flags_from_pf_all_perms() {
        // All permissions (PF_R | PF_W | PF_X = 0x7)
        let flags = pte_flags_from_pf(0x7);
        assert_eq!(flags & PTE_R, PTE_R);
        assert_eq!(flags & PTE_W, PTE_W);
        assert_eq!(flags & PTE_X, PTE_X);
        assert_eq!(flags & PTE_D, PTE_D);
        assert_eq!(flags & PTE_V, PTE_V);
        assert_eq!(flags & PTE_U, PTE_U);
        assert_eq!(flags & PTE_A, PTE_A);
    }

    #[test]
    fn test_pte_flags_from_pf_write_only() {
        // Write-only (PF_W = 0x2) - unusual but valid
        let flags = pte_flags_from_pf(0x2);
        assert_eq!(flags & PTE_W, PTE_W);
        assert_eq!(flags & PTE_D, PTE_D);
        assert_eq!(flags & PTE_R, 0);
        assert_eq!(flags & PTE_X, 0);
    }

    #[test]
    fn test_pte_flags_from_pf_exec_only() {
        // Execute-only (PF_X = 0x1)
        let flags = pte_flags_from_pf(0x1);
        assert_eq!(flags & PTE_X, PTE_X);
        assert_eq!(flags & PTE_R, 0);
        assert_eq!(flags & PTE_W, 0);
        assert_eq!(flags & PTE_D, 0);
    }

    #[test]
    fn test_elf_load_error_types() {
        // Verify error types are distinct
        use core::mem::discriminant;
        assert_ne!(discriminant(&ElfLoadError::Short), discriminant(&ElfLoadError::BadMagic));
        assert_ne!(discriminant(&ElfLoadError::Not64LE), discriminant(&ElfLoadError::NotRiscv));
        assert_ne!(discriminant(&ElfLoadError::PhOutOfBounds), discriminant(&ElfLoadError::SatpNotSet));
    }

    #[test]
    fn test_loaded_struct_fields() {
        // Test that Loaded struct can be created and accessed
        let loaded = Loaded {
            entry_va: 0x10000,
            user_sp: 0x20000,
            argc: 2,
            argv_va: 0x1F000,
            envp_va: 0x1E000,
            brk: 0x30000,
        };
        
        assert_eq!(loaded.entry_va, 0x10000);
        assert_eq!(loaded.user_sp, 0x20000);
        assert_eq!(loaded.argc, 2);
        assert_eq!(loaded.argv_va, 0x1F000);
        assert_eq!(loaded.envp_va, 0x1E000);
        assert_eq!(loaded.brk, 0x30000);
    }

    #[test]
    fn test_elf_constants() {
        // Verify ELF constants are correct
        use goblin::elf::header::*;
        assert_eq!(ELFCLASS64, 2);
        assert_eq!(ELFDATA2LSB, 1);
        assert_eq!(EM_RISCV, 243);
    }
}
