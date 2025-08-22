//! Minimal ELF64 (RISC-V) loader for user programs.
//! - Maps PT_LOAD segments at their p_vaddr with U=1 pages
//! - Copies file bytes and zeros BSS
//! - Returns entry VA and a ready user stack VA

use core::mem::{size_of};
use crate::sv39::{self, PTE_V, PTE_R, PTE_W, PTE_X, PTE_U, PTE_A, PTE_D};

const PT_LOAD: u32 = 1;
const EM_RISCV: u16 = 243;

use riscv::register::sstatus;

// allow S-mode writes to user pages while we populate the stack
#[inline(always)]
unsafe fn with_sum<F, R>(f: F) -> R
where F: FnOnce() -> R {
    sstatus::set_sum();
    let r = f();
    sstatus::clear_sum();
    r
}

unsafe fn write_user_bytes(va: usize, bytes: &[u8]) {
    with_sum(|| {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), va as *mut u8, bytes.len());
    });
}

unsafe fn write_user_usize(va: usize, val: usize) {
    let bytes = core::slice::from_raw_parts((&val as *const usize) as *const u8, core::mem::size_of::<usize>());
    write_user_bytes(va, bytes);
}

// --- ELF headers (packed, unaligned reads) ---
#[repr(C)]
#[derive(Copy, Clone)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type:  u16,
    e_machine: u16,
    e_version: u32,
    e_entry:  u64,
    e_phoff:  u64,
    e_shoff:  u64,
    e_flags:  u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum:  u16,
    e_shentsize: u16,
    e_shnum:  u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Elf64Phdr {
    p_type:   u32,
    p_flags:  u32, // PF_X=1, PF_W=2, PF_R=4
    p_offset: u64,
    p_vaddr:  u64,
    p_paddr:  u64,
    p_filesz: u64,
    p_memsz:  u64,
    p_align:  u64,
}

// Unaligned reads (ELF is little-endian; we are little-endian too on rv64)
unsafe fn read_ehdr(buf: &[u8]) -> Elf64Ehdr {
    let mut out = core::mem::MaybeUninit::<Elf64Ehdr>::uninit();
    core::ptr::copy_nonoverlapping(
        buf.as_ptr(),
        out.as_mut_ptr() as *mut u8,
        size_of::<Elf64Ehdr>(),
    );
    out.assume_init()
}
unsafe fn read_phdr(buf: &[u8], off: usize) -> Elf64Phdr {
    let mut out = core::mem::MaybeUninit::<Elf64Phdr>::uninit();
    core::ptr::copy_nonoverlapping(
        buf.as_ptr().add(off),
        out.as_mut_ptr() as *mut u8,
        size_of::<Elf64Phdr>(),
    );
    out.assume_init()
}

// Map PF flags to PTE flags (always U=1, V=1, set A/D for W)
fn pte_flags_from_pf(pf: u32) -> u64 {
    let mut f = PTE_V | PTE_U | PTE_A;
    if (pf & 0x4) != 0 { f |= PTE_R; }
    if (pf & 0x2) != 0 { f |= PTE_W | PTE_D; }
    if (pf & 0x1) != 0 { f |= PTE_X; }
    f
}

pub struct Loaded {
    pub entry_va: usize,
    pub user_sp: usize,
    pub argc: usize,
    pub argv_va: usize,
    pub envp_va: usize,
}

// Map one VA page to a fresh PA page with flags
unsafe fn map_user_page(root: *mut u64, va_page: usize, flags: u64) -> usize {
    let pa = sv39::alloc_user_page();
    sv39::map_4k(root, va_page, pa, flags);
    pa
}

// Copy bytes into PA (visible through the kernel's identity map)
unsafe fn memcpy_pa(dst_pa: usize, src: *const u8, len: usize) {
    core::ptr::copy_nonoverlapping(src, dst_pa as *mut u8, len);
}

// Zero bytes into PA
unsafe fn memzero_pa(dst_pa: usize, len: usize) {
    core::ptr::write_bytes(dst_pa as *mut u8, 0, len);
}

pub fn load_user_elf(    image: &[u8],
    user_stack_top_va: usize,
    user_stack_bytes: usize,
    argv: &[&str],
    envp: &[&str],) -> Result<Loaded, &'static str> {
    unsafe {
        if image.len() < size_of::<Elf64Ehdr>() { return Err("short ELF"); }
        let eh = read_ehdr(image);
        if &eh.e_ident[0..4] != b"\x7FELF" { return Err("bad magic"); }
        if eh.e_ident[4] != 2 { return Err("not ELF64"); }
        if eh.e_machine != EM_RISCV { return Err("not RISCV"); }
        if eh.e_phentsize as usize != size_of::<Elf64Phdr>() { return Err("phentsize"); }

        let root = sv39::root_pt();
        if root.is_null() { return Err("satp not set"); }

        // Map each PT_LOAD
        for i in 0..eh.e_phnum {
            let off = eh.e_phoff as usize + (i as usize) * size_of::<Elf64Phdr>();
            if off + size_of::<Elf64Phdr>() > image.len() { return Err("phdr oob"); }
            let ph = read_phdr(image, off);
            if ph.p_type != PT_LOAD { continue; }

            let va_start = ph.p_vaddr as usize;
            let filesz   = ph.p_filesz as usize;
            let memsz    = ph.p_memsz as usize;
            let fileoff  = ph.p_offset as usize;
            let flags    = pte_flags_from_pf(ph.p_flags);

            // Page-align
            let page = 4096usize;
            let va0  = va_start & !(page-1);
            let head = va_start - va0;
            let va_end = (va_start + memsz + page-1) & !(page-1);

            // For each page in the segment:
            let mut cur_va = va0;
            let mut copied = 0usize;

            while cur_va < va_end {
                let pa = map_user_page(root, cur_va, flags);

                // Determine how many bytes of FILE go into this page
                let page_off = if cur_va == va0 { head } else { 0 };
                let page_space = page - page_off;

                // File bytes remaining for this page
                let file_left = filesz.saturating_sub(copied);
                let file_chunk = core::cmp::min(file_left, page_space);

                // Copy file bytes (if any)
                if file_chunk > 0 {
                    if fileoff + copied + file_chunk > image.len() { return Err("file oob"); }
                    let src = image.as_ptr().add(fileoff + copied);
                    memcpy_pa(pa + page_off, src, file_chunk);
                    copied += file_chunk;
                }

                // Zero the rest of the page that belongs to memsz
                let mem_covered = if cur_va + page > (va_start + memsz) {
                    (va_start + memsz).saturating_sub(cur_va)
                } else { page };
                if mem_covered > page_off + file_chunk {
                    let zero_len = mem_covered - (page_off + file_chunk);
                    memzero_pa(pa + page_off + file_chunk, zero_len);
                }

                cur_va += page;
            }
        }

        let (sp, envp_va, argv_va, argc) = setup_user_stack(user_stack_top_va, user_stack_bytes, argv, envp, root)?;        

        Ok(Loaded {
            entry_va: eh.e_entry as usize,
            user_sp: sp,
            argc,
            argv_va,
            envp_va,
        })
    }
}

unsafe fn setup_user_stack(user_stack_top_va: usize, user_stack_bytes: usize, argv: &[&str], envp: &[&str], root: *mut u64) -> Result<(usize, usize, usize, usize), &'static str> {
    let stack_pages = (user_stack_bytes + 4095) / 4096;
    let mut va = (user_stack_top_va - stack_pages*4096) & !(4095);
    for _ in 0..stack_pages {
        map_user_page(root, va, PTE_V | PTE_U | PTE_R | PTE_W | PTE_A | PTE_D);
        va += 4096;
    }

    // --- Lay out strings at the top, then pointer vectors, then argc ---
    let mut sp = user_stack_top_va;
    // keep a tiny guard so debug reads can include the trailing NUL
    sp -= 16; 

    // 1) Copy env strings
    let mut env_ptrs: heapless::Vec<usize, 32> = heapless::Vec::new();
    for &s in envp {
        let bytes = s.as_bytes();
        sp -= bytes.len() + 1;
        write_user_bytes(sp, bytes);
        write_user_bytes(sp + bytes.len(), &[0]);
        env_ptrs.push(sp).map_err(|_| "too many env")?;
    }

    // 2) Copy argv strings (argv[0] first)
    let mut arg_ptrs: heapless::Vec<usize, 32> = heapless::Vec::new();
    for &s in argv {
        let bytes = s.as_bytes();
        sp -= bytes.len() + 1;
        write_user_bytes(sp, bytes);
        write_user_bytes(sp + bytes.len(), &[0]);
        arg_ptrs.push(sp).map_err(|_| "too many args")?;
    }
    
    // 3) Align sp to 16
    sp &= !15;

    // 4) Push NULL terminators and pointer arrays (stack grows down)
    // envp NULL
    sp -= core::mem::size_of::<usize>();
    write_user_usize(sp, 0);
    for &p in env_ptrs.iter().rev() {
        sp -= core::mem::size_of::<usize>();
        write_user_usize(sp, p);
    }
    let envp_va = sp;

    // argv NULL
    sp -= core::mem::size_of::<usize>();
    write_user_usize(sp, 0);
    for &p in arg_ptrs.iter().rev() {
        sp -= core::mem::size_of::<usize>();
        write_user_usize(sp, p);
    }
    let argv_va = sp;

    // argc
    let argc = arg_ptrs.len();
    sp -= core::mem::size_of::<usize>();
    write_user_usize(sp, argc);
    sp &= !15;
    Ok((sp, envp_va, argv_va, argc))
}
