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

fn enter_user_with(entry: usize, sp: usize, argc: usize, argv: usize, envp: usize) -> ! {
    use riscv::register::{sepc};
    unsafe {
        sepc::write(entry);

        core::arch::asm!(
            // --- configure sstatus for sret to U-mode ---
            // Clear SPP (bit 8) -> return to User
            "li   t0, 0x100",
            "csrc sstatus, t0",
            // Set SPIE (bit 5) -> enable interrupts in U after sret
            "li   t0, 1 << 5",
            "csrs sstatus, t0",

            // --- load user context (no calls after this!) ---
            "mv   sp,   {usp}",
            "mv   a0,   {arg0}",     // argc
            "mv   a1,   {arg1}",     // argv
            "mv   a2,   {arg2}",     // envp

            // go!
            "sret",
            usp  = in(reg) sp,
            arg0 = in(reg) argc,
            arg1 = in(reg) argv,
            arg2 = in(reg) envp,
            options(noreturn)
        );
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

    // Example argv/envp to demonstrate
    let argv = ["userapp", "hello", "42"];
    let envp = ["TERM=xterm", "LANG=C"];

    let user_stack_top_va: usize = 0x4000_8000;  // choose a low VA for user stack top
    let user_stack_bytes: usize  = 16 * 1024;    // 16 KiB

    match elf::load_user_elf(USER_ELF, user_stack_top_va, user_stack_bytes, &argv, &envp) {
        Ok(img) => {
            use core::fmt::Write;
            let mut uart = crate::uart::Uart::new();
            let _ = writeln!(uart, "Loaded user ELF: entry=0x{:x}, sp=0x{:x}, argc={}", img.entry_va, img.user_sp, img.argc);

            let env0 = unsafe { read_user_usize(img.envp_va) };
            let argv0 = unsafe { read_user_usize(img.argv_va) };
            let _ = writeln!(uart, "argv_va=0x{:x} envp_va=0x{:x}", img.argv_va, img.envp_va);
            let _ = writeln!(uart, "argv[0]=0x{:x} envp[0]=0x{:x}", argv0, env0);

            // show first 32 bytes of envp[0] (should look like ASCII)
            // unsafe { peek_user_bytes(env0, 32); }

            enter_user_with(img.entry_va, img.user_sp, img.argc, img.argv_va, img.envp_va);
        }
        Err(e) => {
            use core::fmt::Write;
            let mut uart = crate::uart::Uart::new();
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

use riscv::register::sstatus;
unsafe fn read_user_usize(va: usize) -> usize {
    sstatus::set_sum();
    let v = core::ptr::read(va as *const usize);
    sstatus::clear_sum();
    v
}
unsafe fn _peek_user_bytes(va: usize, n: usize) {
    use core::fmt::Write;
    let mut uart = crate::uart::Uart::new();

    // cap reads to end of the current 4 KiB page
    let page_end = (va + 4096) & !4095;
    let n_safe = core::cmp::min(n, page_end.saturating_sub(va));

    sstatus::set_sum();
    for i in 0..n_safe {
        let b = core::ptr::read((va + i) as *const u8);
        let _ = write!(uart, "{:02x} ", b);
    }
    sstatus::clear_sum();
    let _ = writeln!(uart, "");
}