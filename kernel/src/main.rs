#![no_std]
#![no_main]

mod entry;
mod sbi;
mod timer;
mod trap;
mod trap_entry;
mod uart;
mod user;

use core::fmt::Write;
use uart::Uart;

#[repr(align(16))]
struct Stack([u8; 4096]);
#[no_mangle]
static mut USER_STACK: Stack = Stack([0; 4096]);

#[inline(always)]
fn user_stack_top() -> usize {
    unsafe { (&raw const USER_STACK.0 as *const u8 as usize) + core::mem::size_of::<Stack>() }
}

#[no_mangle]
extern "C" fn rust_start() -> ! {
    let mut uart = Uart::new();

    // Hello banner
    let _ = writeln!(uart, "\r\nriscv-os: hello from S-mode at 0x8020_0000!");

    trap::init(); // set stvec + enable SIE/STIE
    timer::init(); // arm first tick
    enter_user(); // switch to user mode and run user_main
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut uart = Uart::new();
    let _ = writeln!(uart, "\r\n*** KERNEL PANIC ***");
    if let Some(loc) = info.location() {
        let _ = writeln!(uart, "at {}:{}:{}", loc.file(), loc.line(), loc.column());
    }

    let _ = writeln!(uart, "{}", info.message());

    loop {
        riscv::asm::wfi();
    }
}

fn enter_user() -> ! {
    use riscv::register::{sepc, sstatus};

    extern "C" {
        fn user_main() -> !;
    }

    unsafe {
        // Set the user entry point
        sepc::write(user_main as usize);

        // Give user a stack (we’re switching the current sp right before sret)
        let usp = user_stack_top();
        core::arch::asm!("mv sp, {}", in(reg) usp, options(nostack, preserves_flags));

        // Configure sstatus so sret drops to U
        // SPP = 0 (User), SPIE = 1 (enable interrupts in user after sret)
        sstatus::set_spie();
        sstatus::set_spp(sstatus::SPP::User);

        // Return to user_main at sepc with user privileges
        core::arch::asm!("sret", options(noreturn));
    }
}

#[no_mangle]
extern "C" fn after_user() -> ! {
    use core::fmt::Write;
    let mut uart = uart::Uart::new();
    let _ = writeln!(uart, "user program exited; back in S-mode.");
    loop {
        riscv::asm::wfi();
    }
}
