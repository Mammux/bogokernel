// kernel/src/timer.rs
use core::sync::atomic::{AtomicU64, Ordering};
use riscv::register::time;

use crate::sbi;

pub static TICKS: AtomicU64 = AtomicU64::new(0);

// 10ms at ~10_000_000 Hz?  NOTE: On QEMU/OpenSBI, the timebase is usually
// 10 MHz. We use 100_000 cycles ≈ 10ms. Adjust if you want faster/slower.
const TICK_INTERVAL: u64 = 100_000;

// Timebase frequency in Hz (10 MHz on QEMU virt with OpenSBI)
const TIMEBASE_HZ: u64 = 10_000_000;

pub fn init() {
    // Arm first tick
    let now: u64 = time::read().try_into().unwrap(); // allowed in S-mode on QEMU virt (OpenSBI delegates time)
    sbi::set_timer(now + TICK_INTERVAL); // program next interrupt
}

// Cursor blink rate in timer ticks (50 ticks * 10ms = 500ms)
const CURSOR_BLINK_TICKS: u64 = 50;

pub fn on_timer() {
    // acknowledge and schedule next
    let now: u64 = time::read().try_into().unwrap();
    sbi::set_timer(now + TICK_INTERVAL);

    let t = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    // Update cursor blink every CURSOR_BLINK_TICKS (~500ms with 10ms ticks)
    if t % CURSOR_BLINK_TICKS == 0 {
        crate::console::update_cursor_blink();
    }

    // light logging every 50 ticks to avoid spamming
    /* if t.is_multiple_of(50) {
        use core::fmt::Write;
        let mut uart = crate::uart::Uart::new();
        let _ = writeln!(uart, "tick {t}");
    } */
}

/// Calibrate bogomips by running a delay loop for a fixed duration
/// Returns the calculated bogomips value scaled by 100 (for XX.YY format)
pub fn calibrate_bogomips() -> u64 {
    // Calibration duration in seconds
    const CALIBRATION_SECONDS: u64 = 1;
    const CALIBRATION_CYCLES: u64 = TIMEBASE_HZ * CALIBRATION_SECONDS;
    
    // Measure time before
    let start_time: u64 = time::read().try_into().unwrap();
    let target_time = start_time + CALIBRATION_CYCLES;
    
    // Run the delay loop and count iterations
    let mut loops: u64 = 0;
    while time::read().try_into().unwrap_or(u64::MAX) < target_time {
        // Simple delay loop - similar to Linux's delay_loop
        for _ in 0..1000 {
            core::hint::black_box(());
        }
        loops += 1;
    }
    
    let end_time: u64 = time::read().try_into().unwrap();
    let elapsed_cycles = end_time - start_time;
    
    // Calculate loops per second
    let total_loops = loops * 1000; // multiply by inner loop count
    let loops_per_second = (total_loops * TIMEBASE_HZ) / elapsed_cycles;
    
    // BogoMIPS = (loops per second * 100) / 1,000,000
    // This gives us a value scaled by 100 for XX.YY format
    (loops_per_second * 100) / 1_000_000
}
