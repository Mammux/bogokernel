// kernel/src/sbi.rs

#[inline(always)]
fn sbi_call(ext: usize, fid: usize, a0: usize, a1: usize, a2: usize) -> isize {
    let mut error: isize;
    let mut _value: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") ext, in("a6") fid,
            inlateout("a0") a0 as isize => error,
            inlateout("a1") a1 as isize => _value,
            in("a2") a2,
            lateout("a3") _, lateout("a4") _, lateout("a5") _,
            options(nostack)
        );
    }
    error
}

/* SBI v0.2 TIME extension: EID = 0x54494D45 ('TIME') fid=0 -> set_timer */
const SBI_EID_TIME: usize = 0x54494D45;
pub fn set_timer(stime_value: u64) {
    let _ = sbi_call(
        SBI_EID_TIME,
        0,
        stime_value as usize,
        (stime_value >> 32) as usize,
        0,
    );
}

/*
/// Optional: legacy shutdown (works on QEMU OpenSBI)
pub fn shutdown() -> ! {
    let _ = sbi_call(0x08, 0, 0, 0, 0);
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}
*/
