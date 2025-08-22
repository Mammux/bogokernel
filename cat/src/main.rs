#![no_std]
#![no_main]

#[inline(always)]
unsafe fn sys_open(path: *const u8) -> isize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") 4usize,  // open_cstr(path)
        in("a0") path,
        lateout("a0") ret,
        options(nostack),
    );
    if ret == usize::MAX { -1 } else { ret as isize }
}

#[inline(always)]
unsafe fn sys_read(fd: isize, buf: *mut u8, len: usize) -> isize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") 5usize,  // read(fd, buf, len)
        in("a0") fd as usize,
        in("a1") buf as usize,
        in("a2") len,
        lateout("a0") ret,
        options(nostack),
    );
    if ret == usize::MAX { -1 } else { ret as isize }
}

#[inline(always)]
unsafe fn sys_close(fd: isize) -> isize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") 7usize,  // close(fd)
        in("a0") fd as usize,
        lateout("a0") ret,
        options(nostack),
    );
    if ret == usize::MAX { -1 } else { ret as isize }
}

#[inline(always)]
unsafe fn sys_write(buf: *const u8, len: usize) -> usize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") 1usize,  // write(ptr,len) -> stdout
        in("a0") buf,
        in("a1") len,
        lateout("a0") ret,
        options(nostack),
    );
    ret
}

#[inline(always)]
unsafe fn sys_write_cstr(s: *const u8) -> usize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") 3usize,  // write_cstr
        in("a0") s,
        lateout("a0") ret,
        options(nostack),
    );
    ret
}

#[no_mangle]
pub extern "C" fn _start(argc: usize, argv: *const *const u8, _envp: *const *const u8) -> ! {
    unsafe {
        let path = if argc > 1 {
            core::ptr::read(argv.add(1))
        } else {
            b"hello.txt\0".as_ptr()
        };

        let fd = sys_open(path);
        if fd < 0 {
            if fd == -1 {
                let _ = sys_write_cstr(b"cat: got -1 indicating error\n\0".as_ptr());
            } else {
                let _ = sys_write_cstr(b"cat: open failed with unknown error\n\0".as_ptr());
            }
            sys_exit();
        }

        let mut buf = [0u8; 128];
        loop {
            let n = sys_read(fd, buf.as_mut_ptr(), buf.len());
            if n <= 0 { break; }
            let _ = sys_write(buf.as_ptr(), n as usize);
        }
        let _ = sys_close(fd);
        let _ = sys_write_cstr(b"\n".as_ptr());

        sys_exit();
    }
}

#[inline(always)]
unsafe fn sys_exit() -> ! {
    core::arch::asm!("ecall", in("a7") 2usize, options(noreturn, nostack));
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }
