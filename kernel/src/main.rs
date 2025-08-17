#![no_std]
#![no_main]

mod entry;
mod uart;
mod sbi;
mod trap;
mod timer;
mod trap_entry;

use core::fmt::Write;
use uart::Uart;

#[no_mangle]
extern "C" fn rust_start() -> ! {
    let mut uart = Uart::new();

    // Hello banner
    let _ = writeln!(uart, "\r\nriscv-os: hello from S-mode at 0x8020_0000!");

    trap::init();   // set stvec + enable SIE/STIE
    timer::init();  // arm first tick

    loop {
        // Safe low-power wait (no interrupts yet, but fine)
        unsafe { riscv::asm::wfi(); }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut uart = Uart::new();
    let _ = writeln!(uart, "\r\n*** KERNEL PANIC ***");
    if let Some(loc) = info.location() {
        let _ = writeln!(
            uart,
            "at {}:{}:{}",
            loc.file(),
            loc.line(),
            loc.column()
        );
    }

    let _ = writeln!(uart, "{}", info.message());

    loop {
        unsafe { riscv::asm::wfi(); }
    }
}

