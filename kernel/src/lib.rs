//! Kernel library - testable pure functions
//! 
//! This library exposes pure functions from kernel modules for testing.
//! Tests run in a std environment, while the kernel binary remains no_std.

#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
extern crate alloc;

/// SV39 paging helper functions
pub mod sv39 {
    /// Calculate PPN (Physical Page Number) from physical address
    #[inline]
    pub fn ppn(pa: usize) -> u64 {
        (pa as u64) >> 12
    }

    /// Extract VPN indices from virtual address
    /// Sv39: VPN[2]=bits 38..30, VPN[1]=29..21, VPN[0]=20..12
    #[inline]
    pub fn vpn_indices(va: usize) -> [usize; 3] {
        [(va >> 12) & 0x1ff, (va >> 21) & 0x1ff, (va >> 30) & 0x1ff]
    }

    // PTE flag constants
    pub const PTE_V: u64 = 1 << 0;
    pub const PTE_R: u64 = 1 << 1;
    pub const PTE_W: u64 = 1 << 2;
    pub const PTE_X: u64 = 1 << 3;
    pub const PTE_U: u64 = 1 << 4;
    pub const PTE_G: u64 = 1 << 5;
    pub const PTE_A: u64 = 1 << 6;
    pub const PTE_D: u64 = 1 << 7;

    // Memory layout constants
    pub const DRAM_BASE: usize = 0x8000_0000;
    pub const DRAM_SIZE: usize = 128 * 1024 * 1024;
    pub const USER_VA_BASE: usize = 0x4000_0000;
    pub const USER_PA_POOL_START: usize = 0x8800_0000 - 0x0100_000 - 0x10000;
    pub const USER_PA_POOL_END: usize = 0x8800_0000 - 0x10000;

    pub const USER_CODE_VA: usize = USER_VA_BASE + 0x0000_0000;
    pub const USER_STACK_VA: usize = USER_VA_BASE + 0x0000_1000;

    // Page sizes
    pub const SIZE_4K: usize = 1 << 12;
    pub const SIZE_2M: usize = 1 << 21;
    pub const SIZE_1G: usize = 1 << 30;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_ppn_calculation() {
            assert_eq!(ppn(0x0000), 0);
            assert_eq!(ppn(0x1000), 1);
            assert_eq!(ppn(0x2000), 2);
            assert_eq!(ppn(0x8000_0000), 0x80000);
            assert_eq!(ppn(0xFFFF_F000), 0xFFFFF);
        }

        #[test]
        fn test_vpn_indices_simple() {
            let indices = vpn_indices(0x0000_0000);
            assert_eq!(indices, [0, 0, 0]);
            
            let indices = vpn_indices(0x0000_1000);
            assert_eq!(indices, [1, 0, 0]);
            
            let indices = vpn_indices(0x0020_0000);
            assert_eq!(indices, [0, 1, 0]);
        }

        #[test]
        fn test_vpn_indices_complex() {
            let va = (0x6 << 30) | (0x45 << 21) | (0x123 << 12);
            let indices = vpn_indices(va);
            assert_eq!(indices, [0x123, 0x45, 0x6]);
        }

        #[test]
        fn test_vpn_indices_mask() {
            let va = (0x1FF << 30) | (0x1FF << 21) | (0x1FF << 12);
            let indices = vpn_indices(va);
            assert_eq!(indices, [0x1FF, 0x1FF, 0x1FF]);
            
            assert!(indices[0] <= 0x1FF);
            assert!(indices[1] <= 0x1FF);
            assert!(indices[2] <= 0x1FF);
        }

        #[test]
        fn test_vpn_indices_kernel_addresses() {
            let indices = vpn_indices(DRAM_BASE);
            assert_eq!(indices, [0, 0, 2]);
            
            let indices = vpn_indices(USER_VA_BASE);
            assert_eq!(indices, [0, 0, 1]);
        }

        #[test]
        fn test_pte_flags_constants() {
            assert_eq!(PTE_V, 1 << 0);
            assert_eq!(PTE_R, 1 << 1);
            assert_eq!(PTE_W, 1 << 2);
            assert_eq!(PTE_X, 1 << 3);
            assert_eq!(PTE_U, 1 << 4);
            assert_eq!(PTE_G, 1 << 5);
            assert_eq!(PTE_A, 1 << 6);
            assert_eq!(PTE_D, 1 << 7);
        }

        #[test]
        fn test_page_size_constants() {
            assert_eq!(SIZE_4K, 1 << 12);
            assert_eq!(SIZE_2M, 1 << 21);
            assert_eq!(SIZE_1G, 1 << 30);
        }

        #[test]
        fn test_memory_layout_constants() {
            assert!(DRAM_SIZE > 0);
            assert!(USER_PA_POOL_START < USER_PA_POOL_END);
            assert!(USER_PA_POOL_END <= DRAM_BASE + DRAM_SIZE);
        }

        #[test]
        fn test_user_address_space() {
            assert_eq!(USER_CODE_VA, USER_VA_BASE);
            assert_eq!(USER_STACK_VA, USER_VA_BASE + 0x1000);
            assert!(USER_STACK_VA > USER_CODE_VA);
        }
    }
}

/// ELF loader helper functions
pub mod elf {
    use super::sv39::{PTE_A, PTE_D, PTE_R, PTE_U, PTE_V, PTE_W, PTE_X};

    /// Map ELF p_flags to PTE flags (always V|U|A; add D if W).
    #[inline(always)]
    pub fn pte_flags_from_pf(pf: u32) -> u64 {
        let mut f = PTE_V | PTE_U | PTE_A;
        if (pf & 0x4) != 0 { f |= PTE_R; }
        if (pf & 0x2) != 0 { f |= PTE_W | PTE_D; }
        if (pf & 0x1) != 0 { f |= PTE_X; }
        f
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_pte_flags_from_pf_no_perms() {
            let flags = pte_flags_from_pf(0);
            assert_eq!(flags & PTE_V, PTE_V);
            assert_eq!(flags & PTE_U, PTE_U);
            assert_eq!(flags & PTE_A, PTE_A);
            assert_eq!(flags & PTE_R, 0);
            assert_eq!(flags & PTE_W, 0);
            assert_eq!(flags & PTE_X, 0);
        }

        #[test]
        fn test_pte_flags_from_pf_read_only() {
            let flags = pte_flags_from_pf(0x4);
            assert_eq!(flags & PTE_R, PTE_R);
            assert_eq!(flags & PTE_W, 0);
            assert_eq!(flags & PTE_X, 0);
            assert_eq!(flags & PTE_D, 0);
        }

        #[test]
        fn test_pte_flags_from_pf_read_write() {
            let flags = pte_flags_from_pf(0x6);
            assert_eq!(flags & PTE_R, PTE_R);
            assert_eq!(flags & PTE_W, PTE_W);
            assert_eq!(flags & PTE_D, PTE_D);
            assert_eq!(flags & PTE_X, 0);
        }

        #[test]
        fn test_pte_flags_from_pf_read_exec() {
            let flags = pte_flags_from_pf(0x5);
            assert_eq!(flags & PTE_R, PTE_R);
            assert_eq!(flags & PTE_X, PTE_X);
            assert_eq!(flags & PTE_W, 0);
            assert_eq!(flags & PTE_D, 0);
        }

        #[test]
        fn test_pte_flags_from_pf_all_perms() {
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
            let flags = pte_flags_from_pf(0x2);
            assert_eq!(flags & PTE_W, PTE_W);
            assert_eq!(flags & PTE_D, PTE_D);
            assert_eq!(flags & PTE_R, 0);
            assert_eq!(flags & PTE_X, 0);
        }

        #[test]
        fn test_pte_flags_from_pf_exec_only() {
            let flags = pte_flags_from_pf(0x1);
            assert_eq!(flags & PTE_X, PTE_X);
            assert_eq!(flags & PTE_R, 0);
            assert_eq!(flags & PTE_W, 0);
            assert_eq!(flags & PTE_D, 0);
        }
    }
}
