// kernel/src/timer.rs
use core::sync::atomic::{AtomicU64, Ordering};
use riscv::register::time;

use crate::sbi;

pub static TICKS: AtomicU64 = AtomicU64::new(0);

// 10ms at ~10_000_000 Hz?  NOTE: On QEMU/OpenSBI, the timebase is usually
// 10 MHz. We use 100_000 cycles ≈ 10ms. Adjust if you want faster/slower.
const TICK_INTERVAL: u64 = 100_000;

pub fn init() {
    // Arm first tick
    // let now: u64 = time::read().try_into().unwrap(); // allowed in S-mode on QEMU virt (OpenSBI delegates time)
    // sbi::set_timer(now + TICK_INTERVAL); // program next interrupt
}

pub fn on_timer() {
    // acknowledge and schedule next
    let now: u64 = time::read().try_into().unwrap();
    sbi::set_timer(now + TICK_INTERVAL);

    let t = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    // light logging every 50 ticks to avoid spamming
    /* if t.is_multiple_of(50) {
        use core::fmt::Write;
        let mut uart = crate::uart::Uart::new();
        let _ = writeln!(uart, "tick {t}");
    } */
}
