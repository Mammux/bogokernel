#![no_std]
#![no_main]

#[inline(always)]
unsafe fn sys_write(ch: u8) {
    core::arch::asm!(
        "ecall",
        in("a7") 1usize, in("a0") ch as usize,
        lateout("a0") _,
        options(nostack),
    );
}

#[inline(always)]
unsafe fn sys_exit() -> ! {
    core::arch::asm!("ecall", in("a7") 2usize, options(noreturn, nostack));
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        for &b in b"Hello from ELF!\n" { sys_write(b); }
        sys_exit();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
