#![no_std]
#![no_main]
extern crate alloc;

mod entry;
mod sbi;
mod timer;
mod trap;
mod trap_entry;
mod uart;
mod user;
mod sv39;
mod kalloc;

use core::fmt::Write;
use uart::Uart;
use alloc::vec::Vec;
use alloc::boxed::Box;


#[allow(dead_code)]
#[repr(align(16))]
struct Stack([u8; 4096]);
#[no_mangle]
static mut USER_STACK: Stack = Stack([0; 4096]);

#[inline(always)]
fn _user_stack_top() -> usize {
    unsafe { (&raw const USER_STACK.0 as *const u8 as usize) + core::mem::size_of::<Stack>() }
}



#[no_mangle]
extern "C" fn rust_start() -> ! {
    let mut uart = Uart::new();

    // Hello banner
    let _ = writeln!(uart, "\r\nriscv-os: hello from S-mode at 0x8020_0000!");

    trap::init(); // set stvec + enable SIE/STIE
    timer::init(); // arm first tick

    unsafe { sv39::enable_sv39(); }
    let _ = writeln!(uart, "SV39 paging enabled (identity map + UART)");

    // --- init kernel heap ---
    kalloc::init();
    let _ = writeln!(uart, "Heap init OK.");

    // --- quick sanity checks ---
    // 1) Box
    let b = Box::new(0xC0FFEEu64);
    let _ = writeln!(uart, "Box value = 0x{:x}", *b);

    // 2) Vec
    let mut v: Vec<u32> = Vec::with_capacity(8);
    for i in 0..8 { v.push(i*i); }
    let _ = writeln!(uart, "Vec sum = {}", v.iter().sum::<u32>());    

    // enter_user(); // switch to user mode and run user_main

    loop { riscv::asm::wfi(); }
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

fn _enter_user() -> ! {
    use riscv::register::{sepc, sstatus};

    extern "C" {
        fn _user_main() -> !;
    }

    unsafe {
        // Set the user entry point
        sepc::write(_user_main as usize);

        // Give user a stack (we’re switching the current sp right before sret)
        let usp = _user_stack_top();
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
