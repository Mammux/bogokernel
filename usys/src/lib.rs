#![no_std]

use core::ffi::CStr;
use core::fmt::{self, Write};
use uapi::{is_err_sentinel, nr, SysErr, SysResult};

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Fd(pub i32);

pub const STDIN:  Fd = Fd(0);
pub const STDOUT: Fd = Fd(1);
pub const STDERR: Fd = Fd(2);

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
unsafe fn sys_ecall0_noreturn(nr: usize) -> ! {
    core::arch::asm!("ecall", in("a7") nr, options(noreturn, nostack));
}

/* -------- basic I/O ---------- */

pub fn write(buf: &[u8]) -> usize {
    unsafe { sys_ecall3(nr::WRITE, buf.as_ptr() as usize, buf.len(), 0) }
}
pub fn write_cstr(s: &CStr) -> usize {
    unsafe { sys_ecall1(nr::WRITE_CSTR, s.as_ptr() as usize) }
}
pub fn write_fd(fd: Fd, buf: &[u8]) -> SysResult<usize> {
    let r = unsafe { sys_ecall3(nr::WRITE_FD, fd.0 as usize, buf.as_ptr() as usize, buf.len()) };
    if is_err_sentinel(r) { Err(SysErr::Fail) } else { Ok(r) }
}
pub fn exit() -> ! {
    unsafe { sys_ecall0_noreturn(nr::EXIT) }
}

/* -------- file-like API ---------- */

impl Fd {
    fn read_priv(&self, buf: &mut [u8]) -> SysResult<usize> {
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
    pub fn write(&self, buf: &[u8]) -> SysResult<usize> { write_fd(*self, buf) }    
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

/* ---------- tiny io traits ---------- */

pub trait IoWrite {
    fn write(&self, buf: &[u8]) -> SysResult<usize>;
    fn write_all(&self, mut buf: &[u8]) -> SysResult<()> {
        while !buf.is_empty() {
            let n = self.write(buf)?;
            if n == 0 { return Err(SysErr::Fail); }
            buf = &buf[n..];
        }
        Ok(())
    }
}
pub trait IoRead {
    fn read(&self, buf: &mut [u8]) -> SysResult<usize>;
}

impl IoWrite for Fd {
    fn write(&self, b: &[u8]) -> SysResult<usize> { write_fd(*self, b) }
}

impl IoRead for Fd { fn read(&self, b: &mut [u8]) -> SysResult<usize> { self.read_priv(b) } }


/* -------- tiny stdio-style helpers ---------- */

pub struct Stdout;
pub struct Stderr;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = write_fd(STDOUT, s.as_bytes());
        Ok(())
    }
}

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = write_fd(STDERR, s.as_bytes());
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

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {{
        let _ = core::fmt::write(&mut $crate::Stderr, format_args!($($arg)*));
    }}
}

#[macro_export]
macro_rules! eprintln {
    () => { $crate::eprint!("\n") };
    ($fmt:literal $(, $($arg:tt)+)?) => {{
        $crate::eprint!(concat!($fmt, "\n") $(, $($arg)+)?);
    }}
}

/* ---------- line input helpers ---------- */

/// Read a single line (without the trailing newline) into `buf`.
/// - Accepts `\n` or `\r\n` or `\r` as end-of-line
/// - Supports backspace/delete (0x08/0x7f)
/// - If `echo` is Some, echoes typed characters; backspace is echoed as `\b \b`
/// - Returns the number of bytes written to `buf` (no NUL terminator added)
pub fn read_line<R: IoRead, W: IoWrite>(
    reader: &R,
    echo: Option<&W>,
    buf: &mut [u8],
) -> SysResult<usize> {
    if buf.is_empty() { return Ok(0); }

    let mut i = 0usize;
    let mut ch = [0u8; 1];

    loop {
        let n = reader.read(&mut ch)?;
        if n == 0 { break; } // EOF
        let b = ch[0];

        match b {
            b'\n' => {
                if let Some(w) = echo { let _ = w.write(b"\r\n"); }
                break;
            }
            b'\r' => {
                // swallow optional following '\n' without requiring another read
                if let Some(w) = echo { let _ = w.write(b"\r\n"); }
                break;
            }
            0x08 | 0x7f => { // backspace / delete
                if i > 0 {
                    i -= 1;
                    if let Some(w) = echo { let _ = w.write(b"\x08 \x08"); }
                } else {
                    // nothing to delete; optionally beep
                    // if let Some(w) = echo { let _ = w.write(b"\x07"); }
                }
            }
            _ => {
                if i < buf.len() {
                    buf[i] = b; i += 1;
                    if let Some(w) = echo { let _ = w.write(&[b]); }
                } else {
                    // buffer full: ignore further input, but still allow user to hit Enter to finish
                    // optionally beep: if let Some(w) = echo { let _ = w.write(b"\x07"); }
                }
            }
        }
    }

    Ok(i)
}

/// Convenience: read a line from STDIN, echoing to STDOUT.
pub fn read_line_stdin(buf: &mut [u8]) -> SysResult<usize> {
    read_line(&STDIN, Some(&STDOUT), buf)
}

/// Convenience: read a line from STDIN without echo.
pub fn read_line_stdin_silent(buf: &mut [u8]) -> SysResult<usize> {
    read_line(&STDIN, None::<&Fd>, buf)
}


/// -------- NUL-terminated literal helper ---------- */

#[macro_export]
macro_rules! cstr {
    ($lit:literal) => {{
        const S: &str = concat!($lit, "\0");
        // SAFETY: we appended a NUL ourselves, and $lit can't contain interior NUL
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(S.as_bytes()) }
    }};
}

pub struct CStrBuf<const N: usize> {
    buf: [u8; N],
    len: usize, // number of bytes before the NUL (0..=N-1)
}

impl<const N: usize> CStrBuf<N> {
    /// Create from `&str`, rejecting interior NULs and truncating if needed (to N-1).
    pub fn from_str(s: &str) -> Result<Self, ()> {
        if s.as_bytes().iter().any(|&b| b == 0) {
            return Err(());
        }
        let mut out = Self { buf: [0; N], len: 0 };
        let max = N.saturating_sub(1);
        let copy = core::cmp::min(s.len(), max);
        out.buf[..copy].copy_from_slice(&s.as_bytes()[..copy]);
        out.buf[copy] = 0;
        out.len = copy;
        Ok(out)
    }

    /// Borrow as `&CStr`
    pub fn as_cstr(&self) -> &CStr {
        // SAFETY: we ensured exactly one trailing NUL and no interior NULs
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.buf[..=self.len]) }
    }
}