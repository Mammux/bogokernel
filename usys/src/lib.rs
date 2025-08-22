#![no_std]

use core::ffi::CStr;
use core::fmt::{self, Write};
use uapi::{is_err_sentinel, nr, SysErr, SysResult};

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Fd(pub i32);

#[inline(always)]
unsafe fn sys_ecall3(nr: usize, a0: usize, a1: usize, a2: usize) -> usize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") nr,
        in("a0") a0,
        in("a1") a1,
        in("a2") a2,
        lateout("a0") ret,
        options(nostack),
    );
    ret
}
#[inline(always)]
unsafe fn sys_ecall1(nr: usize, a0: usize) -> usize {
    let mut ret: usize;
    core::arch::asm!(
        "ecall",
        in("a7") nr,
        in("a0") a0,
        lateout("a0") ret,
        options(nostack),
    );
    ret
}
#[inline(always)]
unsafe fn sys_ecall0(nr: usize) -> ! {
    core::arch::asm!("ecall", in("a7") nr, options(noreturn, nostack));
}

/* -------- basic I/O ---------- */

pub fn write(buf: &[u8]) -> usize {
    unsafe { sys_ecall3(nr::WRITE, buf.as_ptr() as usize, buf.len(), 0) }
}
pub fn write_cstr(s: &CStr) -> usize {
    unsafe { sys_ecall1(nr::WRITE_CSTR, s.as_ptr() as usize) }
}
pub fn exit() -> ! {
    unsafe { sys_ecall0(nr::EXIT) }
}

/* -------- file-like API ---------- */

impl Fd {
    pub fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        let r = unsafe {
            sys_ecall3(
                nr::READ,
                self.0 as usize,
                buf.as_mut_ptr() as usize,
                buf.len(),
            )
        };
        if is_err_sentinel(r) {
            Err(SysErr::Fail)
        } else {
            Ok(r)
        }
    }
    pub fn close(self) -> SysResult<()> {
        let r = unsafe { sys_ecall1(nr::CLOSE, self.0 as usize) };
        if is_err_sentinel(r) {
            Err(SysErr::Fail)
        } else {
            Ok(())
        }
    }
}

pub fn open(path: &CStr) -> SysResult<Fd> {
    let r = unsafe { sys_ecall1(nr::OPEN, path.as_ptr() as usize) };
    if is_err_sentinel(r) {
        Err(SysErr::Fail)
    } else {
        Ok(Fd(r as i32))
    }
}

/* -------- tiny stdio-style helpers ---------- */

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write as _;
        let _ = core::fmt::write(&mut $crate::Stdout, format_args!($($arg)*));
    }}
}
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($fmt:literal $(, $($arg:tt)+)?) => {{
        $crate::print!(concat!($fmt, "\n") $(, $($arg)+)?);
    }}
}

/* -------- NUL-terminated literal helper ---------- */

#[macro_export]
macro_rules! cstr {
    ($lit:literal) => {{
        const S: &str = concat!($lit, "\0");
        // SAFETY: we appended a NUL ourselves, and $lit can't contain interior NUL
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(S.as_bytes()) }
    }};
}
