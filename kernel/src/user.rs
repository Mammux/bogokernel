#![allow(dead_code)]

#[inline(always)]
unsafe fn sys_write(ch: u8) {
    core::arch::asm!(
        "ecall",
        in("a7") 1usize,       // syscall nr 1 = write
        in("a0") ch as usize, // arg0 = byte
        lateout("a0") _,          // some ABIs return in a0; we ignore
        options(nostack)
    );
}

#[inline(always)]
pub unsafe fn sys_exit() -> ! {
    core::arch::asm!(
        "ecall",
        in("a7") 2usize,      // syscall nr 2 = exit
        options(noreturn, nostack)
    );
}

#[no_mangle]
pub extern "C" fn user_main() -> ! {
    // A tiny “hello from user” using our syscalls
    unsafe {
        for &b in b"Hello from U-mode!\n" {
            sys_write(b);
        }
        sys_exit();
    }
}
