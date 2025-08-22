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
// mod user_blob;
mod stack;
mod elf;

use core::fmt::Write;
use uart::Uart;
use alloc::vec::Vec;
use alloc::boxed::Box;

use crate::{stack::init_trap_stack};

// User mode stuff
static USER_ELF: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/userapp.elf"));

fn enter_user_with(entry: usize, user_sp_top: usize) -> ! {
    use riscv::register::{sepc, sstatus};
    unsafe {
        sepc::write(entry);
        core::arch::asm!("mv sp, {}", in(reg) user_sp_top, options(nostack));
        sstatus::set_spie();
        sstatus::set_spp(sstatus::SPP::User);
        core::arch::asm!("sret", options(noreturn));
    }
}

extern "C" {
    static __user_blob_start: u8;
    static __user_blob_end:   u8;
}

// Main entry point for the rust code

#[no_mangle]
extern "C" fn rust_start() -> ! {
    let mut uart = Uart::new();

    // Hello banner
    let _ = writeln!(uart, "\r\nriscv-os: hello from S-mode at 0x8020_0000!");

    init_trap_stack(); // init trap stack
    let _ = writeln!(uart, "trap stack initialized");

    trap::init(); // set stvec + enable SIE/STIE
    let _ = writeln!(uart, "traps enabled");

    timer::init(); // arm first tick
    let _ = writeln!(uart, "timers initialized");

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

    // 3) User code
    /*
    unsafe {
        let src = &__user_blob_start as *const u8 as usize;
        let end = &__user_blob_end   as *const u8 as usize;
        let len = end - src;
        core::ptr::copy_nonoverlapping(src as *const u8,
                                    USER_CODE_PA as *mut u8,
                                    len);
    }

    let _ = writeln!(uart, "User blob copied");

    enter_user(); // switch to user mode and run user_main

    loop { riscv::asm::wfi(); }
    */

        // --- Load the user ELF ---
    let user_stack_top_va: usize = 0x4000_8000;  // choose a low VA for user stack top
    let user_stack_bytes: usize  = 16 * 1024;    // 16 KiB

    match elf::load_user_elf(USER_ELF, user_stack_top_va, user_stack_bytes) {
        Ok(img) => {
            let _ = writeln!(uart, "Loaded user ELF: entry=0x{:x}, usp=0x{:x}", img.entry_va, img.user_stack_top_va);
            enter_user_with(img.entry_va, img.user_stack_top_va);
        }
        Err(e) => {
            let _ = writeln!(uart, "*** ELF load error: {}", e);
            loop { riscv::asm::wfi() }
        }
    }
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

#[no_mangle]
extern "C" fn after_user() -> ! {
    use core::fmt::Write;
    let mut uart = uart::Uart::new();
    let _ = writeln!(uart, "\r\nuser program exited; back in S-mode.");
    loop {
        riscv::asm::wfi();
    }
}
