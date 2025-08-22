use core::fmt::Write;

use spin::Mutex;

// kernel/src/trap.rs
use riscv::{interrupt::supervisor::{Exception, Interrupt}, register::{scause, sepc, sie, sstatus, stval, stvec::{self, Stvec}}};
pub use scause::Trap;

use crate::fs;

#[repr(C)]
pub struct TrapFrame {
    pub ra: usize, // x1
    pub sp: usize, // x2 (interrupted SP)
    pub t0: usize, // x5
    pub t1: usize, // x6
    pub t2: usize, // x7
    pub a0: usize, // x10
    pub a1: usize, // x11
    pub a2: usize, // x12
    pub a3: usize, // x13
    pub a4: usize, // x14
    pub a5: usize, // x15
    pub a6: usize, // x16
    pub a7: usize, // x17
    pub t3: usize, // x28
    pub t4: usize, // x29
    pub t5: usize, // x30
    pub t6: usize, // x31
    pub sepc: usize,
    pub sstatus_bits: usize,
}

extern "C" {
    fn __trap_entry();
}

#[inline]
pub fn init() {
    unsafe {
        stvec::write(Stvec::from_bits(__trap_entry as usize));
        sstatus::set_sie(); // global S interrupts
        sie::set_stimer(); // Supervisor timer interrupt enable
    }
}

#[no_mangle]
extern "C" fn rust_trap(tf: &mut TrapFrame) {
    let raw_trap: Trap<usize, usize> = scause::read().cause();
    let standard_trap: Trap<Interrupt, Exception> = raw_trap.try_into().unwrap();

    match standard_trap {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::timer::on_timer();
        }
        Trap::Exception(Exception::UserEnvCall) => {

            // DEBUG: see what user passed
            /* {
                let mut uart = crate::uart::Uart::new();
                let _ = writeln!(uart, "[syscall a7={} a0=0x{:x} a1=0x{:x}]", tf.a7, tf.a0, tf.a1);
            } */

            // Syscall ABI: a7 = nr, a0.. = args; ecall is 4-byte insn
            match tf.a7 {
                1 => sys_write_ptrlen(tf),      // write(ptr, len)
                2 => sys_exit_to_kernel(tf),    // exit()
                3 => sys_write_cstr(tf),        // write_cstr(ptr)
                4 => sys_open(tf),              // open_cstr(path)
                5 => sys_read(tf),              // read(fd, buf, len)
                7 => sys_close(tf),             // close(fd)                
                nr => {
                    let mut uart = crate::uart::Uart::new();
                    use core::fmt::Write;
                    let _ = writeln!(uart, "\r\nunknown syscall: {}", nr);
                    tf.sepc = tf.sepc.wrapping_add(4);
                }
            }
        }
        other => {
            use core::fmt::Write;
            let mut uart = crate::uart::Uart::new();
            let _ = writeln!(
                uart,
                "\r\n*** TRAP *** scause={:?} sepc=0x{:016x} stval={:#x}",
                other,
                sepc::read(),
                stval::read()
            );
            loop {
                unsafe { core::arch::asm!("wfi") }
            }
        }
        
    }

}

// helper: temporarily allow S-mode to load/store user pages
#[inline(always)]
unsafe fn with_sum<F, R>(f: F) -> R
where F: FnOnce() -> R {
    // Set SUM (bit 18) then clear it after
    sstatus::set_sum();
    let r = f();
    sstatus::clear_sum();
    r
}

fn sys_write_ptrlen(tf: &mut super::trap::TrapFrame) {
    let uptr = tf.a0 as *const u8;     // user VA
    let len  = tf.a1 as usize;
    let mut uart = crate::uart::Uart::new();
    unsafe {
        with_sum(|| {
            for i in 0..len {
                let b = core::ptr::read(uptr.add(i));
                // raw byte out; keep it simple
                if b == b'\n' {
                      let _ = Write::write_str(&mut uart, "\r\n");
                } else {
                    // single byte write
                    let _ = Write::write_str(&mut uart, core::str::from_utf8_unchecked(core::slice::from_ref(&b)));
                }
            }
        });
    }
    tf.a0 = len;                 // return value (bytes "written")
    tf.sepc = tf.sepc.wrapping_add(4);
}

// write a NUL-terminated user string; returns byte count
fn sys_write_cstr(tf: &mut super::trap::TrapFrame) {
    let uptr = tf.a0 as *const u8;
    let mut wrote = 0usize;
    let mut uart = crate::uart::Uart::new();

    unsafe {
        with_sum(|| {
            // stay within the current 4 KiB page to avoid crossing into unmapped memory
            let page_end = ((uptr as usize + 4096) & !4095) as *const u8;
            let mut p = uptr;
            while p < page_end {
                let b = core::ptr::read(p);
                if b == 0 { break; }
                if b == b'\n' { uart.write_byte(b'\r'); }
                uart.write_byte(b);
                wrote += 1;
                p = p.add(1);
            }
        });
    }

    tf.a0 = wrote;                         // return value
    tf.sepc = tf.sepc.wrapping_add(4);     // advance past ecall
}

fn sys_exit_to_kernel(tf: &mut super::trap::TrapFrame) {
    extern "C" { fn after_user() -> !; }
    // Return to S-mode at after_user()
    tf.sepc = after_user as usize;
    const SSTATUS_SPP_BIT: usize = 1 << 8;
    const SSTATUS_SPIE_BIT: usize = 1 << 5;
    tf.sstatus_bits |= SSTATUS_SPP_BIT | SSTATUS_SPIE_BIT;
}

// File system stuff

const MAX_FD: usize = 32;

#[derive(Clone, Copy)]
struct FdEntry {
    in_use: bool,
    file_idx: usize,   // index into fs::FILES
    offset: usize,
}

impl FdEntry {
    const EMPTY: Self = Self { in_use: false, file_idx: 0, offset: 0 };
}

static FD_TABLE: Mutex<[FdEntry; MAX_FD]> = Mutex::new([FdEntry::EMPTY; MAX_FD]);

fn fd_alloc(file_idx: usize) -> Option<usize> {
    let mut tbl = FD_TABLE.lock();
    for fd in 3..MAX_FD {
        if !tbl[fd].in_use {
            tbl[fd] = FdEntry { in_use: true, file_idx, offset: 0 };
            return Some(fd);
        }
    }
    None
}
fn fd_get(fd: usize) -> Option<FdEntry> {
    let tbl = FD_TABLE.lock();
    if fd < MAX_FD && tbl[fd].in_use { Some(tbl[fd]) } else { None }
}
fn fd_advance(fd: usize, inc: usize) {
    let mut tbl = FD_TABLE.lock();
    if fd < MAX_FD && tbl[fd].in_use { tbl[fd].offset = tbl[fd].offset.saturating_add(inc); }
}
fn fd_close(fd: usize) -> bool {
    let mut tbl = FD_TABLE.lock();
    if fd < MAX_FD && tbl[fd].in_use { tbl[fd] = FdEntry::EMPTY; true } else { false }
}

// ---- safe-ish user memory helpers ----

#[inline(always)]
unsafe fn with_sum_no_timer<F, R>(f: F) -> R
where F: FnOnce() -> R {
    // Disable S-timer to avoid re-entry while SUM is set
    sie::clear_stimer();
    sstatus::set_sum();
    let r = f();
    sstatus::clear_sum();
    sie::set_stimer();
    r
}
#[inline(always)]
fn cap_to_page(va: usize, len: usize) -> usize {
    let page_end = (va + 4096) & !4095;
    core::cmp::min(len, page_end.saturating_sub(va))
}

// Read a NUL-terminated user string (up to max bytes) that must fit in one page.
fn read_user_cstr_in_page(va: usize, max: usize, out: &mut [u8]) -> Result<&str, ()> {
    if va == 0 { return Err(()); }
    let max = max.min(out.len());
    unsafe {
        with_sum_no_timer(|| {
            let mut p = va as *const u8;
            let page_end = ((va + 4096) & !4095) as *const u8;
            let mut n = 0usize;
            while p < page_end && n < max {
                let b = core::ptr::read(p);
                if b == 0 { 
                    return core::str::from_utf8(&out[..n]).map_err(|_| ());
                }
                out[n] = b; n += 1; p = p.add(1);
            }
            Err(())
        })
    }
}

// Copy bytes from kernel slice to user buffer
fn copy_to_user(dst_va: usize, src: &[u8]) -> usize {
    if dst_va == 0 || src.is_empty() { return 0; }
    let n = cap_to_page(dst_va, src.len());
    unsafe {
        with_sum_no_timer(|| {
            core::ptr::copy_nonoverlapping(src.as_ptr(), dst_va as *mut u8, n);
        });
    }
    n
}

fn sys_open(tf: &mut TrapFrame) {
    // a0 = path (C string in user VA)
    let path_va = tf.a0;
    let mut buf = [0u8; 256];
    let path = match read_user_cstr_in_page(path_va, 255, &mut buf) {
        Ok(s) => s,
        Err(_) => { tf.a0 = usize::MAX; tf.sepc = tf.sepc.wrapping_add(4); return; }
    };

    let _ = write!(crate::uart::Uart::new(), "sys_open: path='{}'\n", path);

    if let Some((idx, _f)) = fs::FILES.iter().enumerate().find(|(_, f)| f.name == path) {
        if let Some(fd) = fd_alloc(idx) {
            let _ = write!(crate::uart::Uart::new(), "sys_open: found fd\n");
            tf.a0 = fd;
        } else {
            let _ = write!(crate::uart::Uart::new(), "sys_open: unable to alloc fd \n");
            tf.a0 = usize::MAX; // no fds
        }
    } else {
        let _ = write!(crate::uart::Uart::new(), "sys_open: file not found\n");
        tf.a0 = usize::MAX;     // not found
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_read(tf: &mut TrapFrame) {
    // a0 = fd, a1 = buf, a2 = len
    let fd = tf.a0;
    let buf = tf.a1;
    let mut len = tf.a2;

    if buf == 0 || len == 0 {
        tf.a0 = 0; tf.sepc = tf.sepc.wrapping_add(4); return;
    }
    len = cap_to_page(buf, len);

    let entry = match fd_get(fd) {
        Some(e) => e,
        None => { tf.a0 = usize::MAX; tf.sepc = tf.sepc.wrapping_add(4); return; }
    };
    let file = &crate::fs::FILES[entry.file_idx];
    if entry.offset >= file.data.len() {
        tf.a0 = 0; tf.sepc = tf.sepc.wrapping_add(4); return;
    }
    let remain = &file.data[entry.offset..];
    let chunk = &remain[..core::cmp::min(len, remain.len())];

    let n = copy_to_user(buf, chunk);
    fd_advance(fd, n);
    tf.a0 = n;
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_close(tf: &mut TrapFrame) {
    let fd = tf.a0;
    tf.a0 = if fd_close(fd) { 0 } else { usize::MAX };
    tf.sepc = tf.sepc.wrapping_add(4);
}
