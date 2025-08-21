//! Minimal Sv39 identity mapping: map 128 MiB DRAM with 2 MiB pages (RWX),
//! and map UART (0x1000_0000) as RW (no exec) with a 4 KiB page.
#![allow(dead_code)]

use core::mem::MaybeUninit;

// ----- Sv39 constants -----

const PAGE_SIZE: usize = 4096;
const ENTRIES: usize = 512;

// PTE flag bits
const PTE_V: u64 = 1 << 0;
const PTE_R: u64 = 1 << 1;
const PTE_W: u64 = 1 << 2;
const PTE_X: u64 = 1 << 3;
const PTE_U: u64 = 1 << 4;
const PTE_G: u64 = 1 << 5;
const PTE_A: u64 = 1 << 6;
const PTE_D: u64 = 1 << 7;

// Convenience sets
const RWX: u64 = PTE_R | PTE_W | PTE_X | PTE_V | PTE_A | PTE_D;
const RW:  u64 = PTE_R | PTE_W | PTE_V | PTE_A | PTE_D;
const RX:  u64 = PTE_R | PTE_X | PTE_V | PTE_A; // no D (won’t be written)

// Page sizes (Sv39 levels): L0=4K, L1=2M, L2=1G
const SIZE_4K: usize = 1 << 12;
const SIZE_2M: usize = 1 << 21;
const SIZE_1G: usize = 1 << 30;

// QEMU virt memory we’ll map
const DRAM_BASE: usize = 0x8000_0000;
const DRAM_SIZE: usize = 128 * 1024 * 1024; // 128 MiB

const UART0: usize = 0x1000_0000;

// ----- Simple PT “allocator”: a tiny pool of zeroed page-table pages -----

#[derive(Copy, Clone)]
#[repr(align(4096))]
struct PtPage([u64; ENTRIES]);

static mut PT_POOL: [MaybeUninit<PtPage>; 32] = [MaybeUninit::uninit(); 32];
static mut PT_CUR: usize = 0;

#[allow(static_mut_refs)]
unsafe fn alloc_pt_page() -> *mut u64 {
    let idx = PT_CUR;
    assert!(idx < PT_POOL.len(), "Out of PT pages");
    PT_CUR += 1;
    let p = PT_POOL[idx].as_mut_ptr();
    // zero it
    core::ptr::write_bytes(p as *mut u8, 0, core::mem::size_of::<PtPage>());
    (*p).0.as_mut_ptr()
}

#[inline]
fn ppn(pa: usize) -> u64 { (pa as u64) >> 12 }

#[inline]
fn vpn_indices(va: usize) -> [usize; 3] {
    // Sv39: VPN[2]=bits 38..30, VPN[1]=29..21, VPN[0]=20..12
    [ (va >> 12) & 0x1ff, (va >> 21) & 0x1ff, (va >> 30) & 0x1ff ]
}

// ----- Mapping helpers -----

unsafe fn map_4k(root: *mut u64, va: usize, pa: usize, flags: u64) {
    let [i0, i1, i2] = vpn_indices(va);
    // Walk/create L2 -> L1
    let l2 = root;
    let pte2 = l2.add(i2);
    let mut next = *pte2;
    let l1 = if (next & PTE_V) == 0 {
        let new = alloc_pt_page();
        *pte2 = (ppn(new as usize) << 10) | PTE_V; // non-leaf pointer
        new
    } else {
        ((((next >> 10) & ((1 << 44) - 1)) as usize) << 12) as *mut u64
    };

    // Walk/create L1 -> L0
    let pte1 = l1.add(i1);
    next = *pte1;
    let l0 = if (next & PTE_V) == 0 {
        let new = alloc_pt_page();
        *pte1 = (ppn(new as usize) << 10) | PTE_V;
        new
    } else {
        ((((next >> 10) & ((1 << 44) - 1)) as usize) << 12) as *mut u64
    };

    // Leaf at L0 (4K)
    let pte0 = l0.add(i0);
    *pte0 = (ppn(pa) << 10) | flags;
}

unsafe fn map_2m(root: *mut u64, va: usize, pa: usize, flags: u64) {
    // Leaf at L1
    assert!(va % SIZE_2M == 0 && pa % SIZE_2M == 0);
    let [_, i1, i2] = vpn_indices(va);

    let l2 = root;
    let pte2 = l2.add(i2);
    let next = *pte2;
    let l1 = if (next & PTE_V) == 0 {
        let new = alloc_pt_page();
        *pte2 = (ppn(new as usize) << 10) | PTE_V;
        new
    } else {
        ((((next >> 10) & ((1 << 44) - 1)) as usize) << 12) as *mut u64
    };

    let pte1 = l1.add(i1);
    *pte1 = (ppn(pa) << 10) | flags; // set RWX/V (+A/D in flags)
}

// Map a VA..VA+len identity to same PA..PA+len using largest pages (2M) where possible.
unsafe fn id_map_region(root: *mut u64, base: usize, len: usize, flags_2m: u64, flags_4k: u64) {
    let mut va = base & !(SIZE_4K - 1);
    let end = (base + len + SIZE_4K - 1) & !(SIZE_4K - 1);

    // Align up to 2M boundary for the big pages, do 4K for the head/tail if needed
    while va < end && (va % SIZE_2M != 0) {
        map_4k(root, va, va, flags_4k);
        va += SIZE_4K;
    }
    while va + SIZE_2M <= end {
        map_2m(root, va, va, flags_2m);
        va += SIZE_2M;
    }
    while va < end {
        map_4k(root, va, va, flags_4k);
        va += SIZE_4K;
    }
}

// ----- Public init -----

pub unsafe fn enable_sv39() {
    // Root PT page
    let root = alloc_pt_page();

    // Identity-map 128 MiB DRAM (RWX for now to keep life simple)
    id_map_region(root, DRAM_BASE, DRAM_SIZE, RWX, RWX);

    // Map UART MMIO as RW (no exec). One 4K page is enough.
    map_4k(root, UART0, UART0, RW);

    // Flip SATP: MODE=Sv39 (8), ASID=0, PPN = root >> 12
    use riscv::register::satp::{self, Satp, Mode};
    let root_ppn = (root as usize) >> 12;

    // satp layout on RV64 (Sv39):
    // [63:60] MODE (8 = Sv39)
    // [59:44] ASID (16 bits) -> we'll use 0
    // [43:0]  PPN  (44 bits)
    let asid: usize = 0;
    let bits: usize =
        ((Mode::Sv39 as usize) << 60) |
        ((asid & 0xffff) << 44) |
        (root_ppn & ((1usize << 44) - 1));

    let new = Satp::from_bits(bits);
    unsafe { satp::write(new) };

    // Flush TLBs
    riscv::asm::sfence_vma_all();
}
