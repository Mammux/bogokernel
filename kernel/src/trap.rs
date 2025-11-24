use core::fmt::Write;
use spin::Mutex;
use uapi::nr;

// kernel/src/trap.rs
use riscv::{
    interrupt::supervisor::{Exception, Interrupt},
    register::{
        scause, sepc, sie, sstatus, stval,
        stvec::{self, Stvec},
    },
};
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
        stvec::write(Stvec::from_bits(__trap_entry as *const () as usize));
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
            /*
            {
                let mut uart = crate::uart::Uart::new();
                let _ = writeln!(uart, "[syscall a7={} a0=0x{:x} a1=0x{:x}]", tf.a7, tf.a0, tf.a1);
            }
            */

            // Syscall ABI: a7 = nr, a0.. = args; ecall is 4-byte insn
            match tf.a7 {
                nr::WRITE => sys_write_ptrlen(tf),    // write(ptr, len)
                nr::EXIT => sys_exit(tf),             // exit()
                nr::WRITE_CSTR => sys_write_cstr(tf), // write_cstr(ptr)
                nr::OPEN => sys_open(tf),             // open_cstr(path)
                nr::READ => sys_read(tf),             // read(fd, buf, len)
                nr::WRITE_FD => sys_write_fd(tf),     // write(fd, buf, len)
                nr::CLOSE => sys_close(tf),           // close(fd)
                nr::LSEEK => sys_lseek(tf),           // lseek(fd, offset, whence)
                nr::BRK => sys_brk(tf),               // brk(addr)
                nr::GETTIME => sys_gettime(tf),       // gettime(ts_ptr)
                nr::POWEROFF => sys_poweroff(tf),     // poweroff()
                nr::EXEC => sys_exec(tf),             // exec(path)
                nr::EXECV => sys_execv(tf),           // execv(path, argv)
                nr::CREAT => sys_creat(tf),           // creat(path, mode)
                nr::UNLINK => sys_unlink(tf),         // unlink(path)
                nr::STAT => sys_stat(tf),             // stat(path, buf)
                nr::CHMOD => sys_chmod(tf),           // chmod(path, mode)
                nr::READDIR => sys_readdir(tf),       // readdir(buf, len)
                nr::GET_FB_INFO => sys_get_fb_info(tf), // get_fb_info(buf)
                nr::FB_FLUSH => sys_fb_flush(tf),     // fb_flush()
                nr => {
                    let mut uart = crate::uart::Uart::new();
                    let _ = writeln!(uart, "\r\nunknown syscall: {}", nr);
                    tf.a0 = usize::MAX;
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
where
    F: FnOnce() -> R,
{
    // Set SUM (bit 18) then clear it after
    sstatus::set_sum();
    let r = f();
    sstatus::clear_sum();
    r
}

fn sys_write_ptrlen(tf: &mut super::trap::TrapFrame) {
    let uptr = tf.a0 as *const u8; // user VA
    let len = tf.a1 as usize;
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
                    let _ = Write::write_str(
                        &mut uart,
                        core::str::from_utf8_unchecked(core::slice::from_ref(&b)),
                    );
                }
            }
        });
    }
    tf.a0 = len; // return value (bytes "written")
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
                if b == 0 {
                    break;
                }
                if b == b'\n' {
                    uart.write_byte(b'\r');
                }
                uart.write_byte(b);
                wrote += 1;
                p = p.add(1);
            }
        });
    }

    tf.a0 = wrote; // return value
    tf.sepc = tf.sepc.wrapping_add(4); // advance past ecall
}

fn sys_exit(tf: &mut TrapFrame) {
    // exit code in a0
    let _ = writeln!(crate::uart::Uart::new(), "sys_exit: reloading shell");
    
    // Clear the FD table - use lock() which will wait if needed
    let _ = writeln!(crate::uart::Uart::new(), "sys_exit: clearing FD_TABLE");
    {
        let mut tbl = FD_TABLE.lock();
        for fd in 3..MAX_FD {
            tbl[fd] = FdEntry::EMPTY;
        }
    } // Guard is dropped here
    let _ = writeln!(crate::uart::Uart::new(), "sys_exit: FD_TABLE cleared");
    
    // Load shell.elf
    let _ = writeln!(crate::uart::Uart::new(), "sys_exit: loading shell");
    load_program(tf, "shell.elf", &["shell.elf"]);
    let _ = writeln!(crate::uart::Uart::new(), "sys_exit: shell loaded, returning");
}

// File system stuff

const MAX_FD: usize = 32;

#[derive(Clone, Copy)]
enum FileType {
    ReadOnly(usize), // index into fs::FILES
    Writable(usize), // index into writable files
}

#[derive(Clone, Copy)]
struct FdEntry {
    in_use: bool,
    file_type: FileType,
    offset: usize,
    writable: bool,
}

impl FdEntry {
    const EMPTY: Self = Self {
        in_use: false,
        file_type: FileType::ReadOnly(0),
        offset: 0,
        writable: false,
    };
}

static FD_TABLE: Mutex<[FdEntry; MAX_FD]> = Mutex::new([FdEntry::EMPTY; MAX_FD]);

fn fd_alloc(file_type: FileType, writable: bool) -> Option<usize> {
    let _ = writeln!(crate::uart::Uart::new(), "fd_alloc: about to lock FD_TABLE");
    let mut tbl = FD_TABLE.lock();
    let _ = writeln!(crate::uart::Uart::new(), "fd_alloc: acquired FD_TABLE lock");
    for fd in 3..MAX_FD {
        if !tbl[fd].in_use {
            tbl[fd] = FdEntry {
                in_use: true,
                file_type,
                offset: 0,
                writable,
            };
            let _ = writeln!(crate::uart::Uart::new(), "fd_alloc: allocated fd={}", fd);
            return Some(fd);
        }
    }
    let _ = writeln!(crate::uart::Uart::new(), "fd_alloc: no free fds");
    None
}
fn fd_get(fd: usize) -> Option<FdEntry> {
    let tbl = FD_TABLE.lock();
    if fd < MAX_FD && tbl[fd].in_use {
        Some(tbl[fd])
    } else {
        None
    }
}
fn fd_advance(fd: usize, inc: usize) {
    let mut tbl = FD_TABLE.lock();
    if fd < MAX_FD && tbl[fd].in_use {
        tbl[fd].offset = tbl[fd].offset.saturating_add(inc);
    }
}
fn fd_seek(fd: usize, offset: usize) {
    let mut tbl = FD_TABLE.lock();
    if fd < MAX_FD && tbl[fd].in_use {
        tbl[fd].offset = offset;
    }
}
fn fd_close(fd: usize) -> bool {
    let mut tbl = FD_TABLE.lock();
    if fd < MAX_FD && tbl[fd].in_use {
        tbl[fd] = FdEntry::EMPTY;
        true
    } else {
        false
    }
}

// ---- safe-ish user memory helpers ----

#[inline(always)]
unsafe fn with_sum_no_timer<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
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
    if va == 0 {
        return Err(());
    }
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
                out[n] = b;
                n += 1;
                p = p.add(1);
            }
            Err(())
        })
    }
}

// Copy bytes from kernel slice to user buffer
fn copy_to_user(dst_va: usize, src: &[u8]) -> usize {
    if dst_va == 0 || src.is_empty() {
        return 0;
    }
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
        Err(_) => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    let _ = write!(crate::uart::Uart::new(), "sys_open: path='{}'\r\n", path);

    // First check writable files
    if let Some(idx) = fs::lookup_writable(path) {
        let _ = write!(
            crate::uart::Uart::new(),
            "sys_open: writable file found at idx={}\r\n",
            idx
        );
        if let Some(fd) = fd_alloc(FileType::Writable(idx), false) {
            let _ = write!(
                crate::uart::Uart::new(),
                "sys_open: allocated fd={}\r\n",
                fd
            );
            tf.a0 = fd;
        } else {
            let _ = write!(crate::uart::Uart::new(), "sys_open: unable to alloc fd\r\n");
            tf.a0 = usize::MAX; // no fds
        }
    } else if let Some((idx, _f)) = fs::FILES.iter().enumerate().find(|(_, f)| f.name == path) {
        let _ = write!(
            crate::uart::Uart::new(),
            "sys_open: file found at idx={}\r\n",
            idx
        );
        if let Some(fd) = fd_alloc(FileType::ReadOnly(idx), false) {
            let _ = write!(
                crate::uart::Uart::new(),
                "sys_open: allocated fd={}\r\n",
                fd
            );
            tf.a0 = fd;
        } else {
            let _ = write!(crate::uart::Uart::new(), "sys_open: unable to alloc fd\r\n");
            tf.a0 = usize::MAX; // no fds
        }
    } else {
        let _ = write!(crate::uart::Uart::new(), "sys_open: file not found\r\n");
        tf.a0 = usize::MAX; // not found
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_poweroff(_tf: &mut TrapFrame) {
    crate::sbi::shutdown();
}

fn sys_exec(tf: &mut TrapFrame) {
    // a0 = path
    let path_va = tf.a0;
    let mut buf = [0u8; 256];
    let path = match read_user_cstr_in_page(path_va, 255, &mut buf) {
        Ok(s) => s,
        Err(_) => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    let _ = writeln!(crate::uart::Uart::new(), "sys_exec: path='{}'", path);
    
    // Debug: check if FD_TABLE is locked
    if FD_TABLE.is_locked() {
        let _ = writeln!(crate::uart::Uart::new(), "sys_exec: WARNING - FD_TABLE is locked!");
    }
    
    // Use path as argv[0]
    load_program(tf, path, &[path]);
    
    // Debug: check if FD_TABLE is locked after load_program
    if FD_TABLE.is_locked() {
        let _ = writeln!(crate::uart::Uart::new(), "sys_exec: WARNING - FD_TABLE locked after load_program!");
    }
}

fn sys_execv(tf: &mut TrapFrame) {
    // a0 = path, a1 = argv (NULL-terminated array of C string pointers)
    let path_va = tf.a0;
    let argv_va = tf.a1;

    let _ = writeln!(crate::uart::Uart::new(), "sys_execv: starting");
    
    // Debug: check if FD_TABLE is locked at start
    if FD_TABLE.is_locked() {
        let _ = writeln!(crate::uart::Uart::new(), "sys_execv: WARNING - FD_TABLE is locked at start!");
    }

    let mut path_buf = [0u8; 256];
    let path = match read_user_cstr_in_page(path_va, 255, &mut path_buf) {
        Ok(s) => s,
        Err(_) => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    // Read argv array from user memory using fixed arrays
    let mut argv_bufs: [[u8; 64]; 16] = [[0; 64]; 16];
    let mut argv_lens: [usize; 16] = [0; 16];
    let mut argv_count = 0usize;

    unsafe {
        with_sum_no_timer(|| {
            let mut i = 0usize;
            loop {
                if i >= 16 {
                    break;
                } // Max 16 arguments

                // Read pointer from argv array
                let ptr_addr = argv_va + i * core::mem::size_of::<usize>();
                let arg_ptr = core::ptr::read(ptr_addr as *const usize);

                if arg_ptr == 0 {
                    break;
                } // NULL terminator

                // Read the string
                let mut arg_len = 0usize;
                let page_end = ((arg_ptr + 4096) & !4095) as *const u8;
                let mut p = arg_ptr as *const u8;

                while p < page_end && arg_len < 63 {
                    let b = core::ptr::read(p);
                    if b == 0 {
                        break;
                    }
                    argv_bufs[argv_count][arg_len] = b;
                    arg_len += 1;
                    p = p.add(1);
                }

                argv_lens[argv_count] = arg_len;
                argv_count += 1;
                i += 1;
            }
        });
    }

    // Convert to &[&str] for load_program
    let mut argv_strs: [&str; 16] = [""; 16];
    for i in 0..argv_count {
        if let Ok(s) = core::str::from_utf8(&argv_bufs[i][..argv_lens[i]]) {
            argv_strs[i] = s;
        }
    }

    load_program(tf, path, &argv_strs[..argv_count]);
    
    // Debug: check if FD_TABLE is locked after load_program
    if FD_TABLE.is_locked() {
        let _ = writeln!(crate::uart::Uart::new(), "sys_execv: WARNING - FD_TABLE locked after load_program!");
    }
}

fn load_program(tf: &mut TrapFrame, name: &str, argv: &[&str]) {
    let _ = writeln!(crate::uart::Uart::new(), "load_program: starting for '{}'", name);
    
    // Check if FD_TABLE is locked at start
    if FD_TABLE.is_locked() {
        let _ = writeln!(crate::uart::Uart::new(), "load_program: WARNING - FD_TABLE is locked at start!");
    }
    
    // Find file in writable filesystem
    let file_data = match fs::get_file_data(name) {
        Some(data) => data,
        None => {
            let _ = writeln!(crate::uart::Uart::new(), "exec: file not found '{}'", name);
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };
    
    let envp = ["PATH=/"];

    // Reuse stack constants from main.rs (should be shared)
    let user_stack_top_va: usize = 0x4000_8000;
    let user_stack_bytes: usize = 16 * 1024;

    // CRITICAL: Clear old user mappings and reset allocator before loading new program
    let _ = writeln!(crate::uart::Uart::new(), "load_program: clearing user pages");
    unsafe {
        crate::sv39::reset_user_pages();
        crate::sv39::clear_user_mappings();
    }

    let _ = writeln!(crate::uart::Uart::new(), "load_program: calling load_user_elf");
    match crate::elf::load_user_elf(&file_data, user_stack_top_va, user_stack_bytes, argv, &envp) {
        Ok(img) => {
            let _ = writeln!(crate::uart::Uart::new(), "load_program: load_user_elf succeeded");
            
            // Flush TLB to ensure old mappings are invalidated
            riscv::asm::sfence_vma_all();

            // Update TrapFrame to start new program
            tf.sepc = img.entry_va;
            tf.sp = img.user_sp;
            tf.a0 = img.argc;
            tf.a1 = img.argv_va;
            tf.a2 = img.envp_va;

            unsafe {
                USER_BRK = img.brk;
            }

            let _ = writeln!(crate::uart::Uart::new(), "load_program: TrapFrame updated");
            
            // Check if FD_TABLE is locked before returning
            if FD_TABLE.is_locked() {
                let _ = writeln!(crate::uart::Uart::new(), "load_program: WARNING - FD_TABLE is locked before return!");
            }
            
            // Success: do NOT increment sepc, just return to new entry
        }
        Err(e) => {
            let _ = writeln!(crate::uart::Uart::new(), "exec failed: {:?}", e);
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
        }
    }
    
    let _ = writeln!(crate::uart::Uart::new(), "load_program: returning");
}

fn sys_read(tf: &mut TrapFrame) {
    // a0 = fd, a1 = buf (user VA), a2 = len
    let fd = tf.a0 as isize;
    let buf = tf.a1;
    let mut len = tf.a2;

    if buf == 0 || len == 0 {
        tf.a0 = 0;
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }
    len = cap_to_page(buf, len);

    // --- STDIN (UART RX) ---
    if fd == 0 {
        let mut uart = crate::uart::Uart::new();
        let mut n = 0usize;

        // Block for the first byte so read() is not spurious
        let first = uart.read_byte();
        unsafe {
            with_sum_no_timer(|| {
                core::ptr::write((buf as *mut u8).add(n), first);
            });
        }
        n += 1;

        // Drain any immediately available bytes without blocking further
        while n < len {
            if let Some(b) = uart.try_read_byte() {
                unsafe {
                    with_sum_no_timer(|| {
                        core::ptr::write((buf as *mut u8).add(n), b);
                    });
                }
                n += 1;
            } else {
                break;
            }
        }

        tf.a0 = n;
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }

    // --- Not readable: stdout/stderr ---
    if fd == 1 || fd == 2 {
        tf.a0 = usize::MAX; // error
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }

    // --- Regular files via RAMFS or writable files ---
    let entry = match fd_get(fd as usize) {
        Some(e) => e,
        None => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    match entry.file_type {
        FileType::ReadOnly(idx) => {
            let file = &crate::fs::FILES[idx];
            if entry.offset >= file.data.len() {
                tf.a0 = 0;
                tf.sepc = tf.sepc.wrapping_add(4);
                return;
            }
            let remain = &file.data[entry.offset..];
            let chunk = &remain[..core::cmp::min(len, remain.len())];

            let n = copy_to_user(buf, chunk);
            fd_advance(fd as usize, n);
            tf.a0 = n;
        }
        FileType::Writable(idx) => {
            // Read from writable file
            let mut temp_buf = [0u8; 4096];
            let read_len = core::cmp::min(len, temp_buf.len());

            match fs::read_file(idx, entry.offset, &mut temp_buf[..read_len]) {
                Ok(n) => {
                    let copied = copy_to_user(buf, &temp_buf[..n]);
                    fd_advance(fd as usize, copied);
                    tf.a0 = copied;
                }
                Err(_) => {
                    tf.a0 = usize::MAX;
                }
            }
        }
    }

    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_close(tf: &mut TrapFrame) {
    let fd = tf.a0;
    tf.a0 = if fd_close(fd) { 0 } else { usize::MAX };
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_write_fd(tf: &mut TrapFrame) {
    // a0 = fd, a1 = buf, a2 = len
    let fd = tf.a0 as isize;
    let buf = tf.a1;
    let mut len = tf.a2;

    if buf == 0 || len == 0 {
        tf.a0 = 0;
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }
    len = cap_to_page(buf, len);

    // Check if this is stdout/stderr - write to UART
    if fd == 1 || fd == 2 {
        let mut uart = crate::uart::Uart::new();
        unsafe {
            with_sum_no_timer(|| {
                for i in 0..len {
                    let b = core::ptr::read((buf + i) as *const u8);
                    if b == b'\n' {
                        uart.write_byte(b'\r');
                    }
                    uart.write_byte(b);
                }
            });
        }
        tf.a0 = len;
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }

    // Handle file writes
    let entry = match fd_get(fd as usize) {
        Some(e) => e,
        None => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    if !entry.writable {
        // Read-only file
        tf.a0 = usize::MAX;
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }

    match entry.file_type {
        FileType::Writable(idx) => {
            // Copy from user to kernel buffer, then write
            let mut temp_buf = [0u8; 4096];
            let write_len = core::cmp::min(len, temp_buf.len());

            unsafe {
                with_sum_no_timer(|| {
                    core::ptr::copy_nonoverlapping(
                        buf as *const u8,
                        temp_buf.as_mut_ptr(),
                        write_len,
                    );
                });
            }

            match fs::write_file(idx, entry.offset, &temp_buf[..write_len]) {
                Ok(n) => {
                    fd_advance(fd as usize, n);
                    tf.a0 = n;
                }
                Err(_) => {
                    tf.a0 = usize::MAX;
                }
            }
        }
        FileType::ReadOnly(_) => {
            // Should not happen (checked writable above)
            tf.a0 = usize::MAX;
        }
    }

    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_lseek(tf: &mut TrapFrame) {
    // a0 = fd, a1 = offset, a2 = whence
    let fd = tf.a0;
    let offset = tf.a1 as isize;
    let whence = tf.a2;

    let entry = match fd_get(fd) {
        Some(e) => e,
        None => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    let file_len = match entry.file_type {
        FileType::ReadOnly(idx) => crate::fs::FILES[idx].data.len(),
        FileType::Writable(idx) => match fs::file_size(idx) {
            Some(sz) => sz,
            None => {
                tf.a0 = usize::MAX;
                tf.sepc = tf.sepc.wrapping_add(4);
                return;
            }
        },
    };

    let new_off = match whence {
        0 => offset,                         // SEEK_SET
        1 => entry.offset as isize + offset, // SEEK_CUR
        2 => file_len as isize + offset,     // SEEK_END
        _ => -1,
    };

    if new_off < 0 {
        tf.a0 = usize::MAX;
    } else {
        let new_off = new_off as usize;
        fd_seek(fd, new_off);
        tf.a0 = new_off;
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

pub static mut USER_BRK: usize = 0;

fn sys_brk(tf: &mut TrapFrame) {
    // a0 = new_brk
    let req_brk = tf.a0;
    let cur_brk = unsafe { USER_BRK };

    if req_brk == 0 {
        tf.a0 = cur_brk;
    } else if req_brk > cur_brk {
        // Allocate pages
        let page_mask = 4095;
        let old_page_end = (cur_brk + page_mask) & !page_mask;
        let new_page_end = (req_brk + page_mask) & !page_mask;

        if new_page_end > old_page_end {
            let pages_needed = (new_page_end - old_page_end) / 4096;
            let root = unsafe { crate::sv39::root_pt() };
            for i in 0..pages_needed {
                let va = old_page_end + i * 4096;
                unsafe {
                    let pa = crate::sv39::alloc_user_page();
                    crate::sv39::map_4k(
                        root,
                        va,
                        pa,
                        crate::sv39::PTE_V
                            | crate::sv39::PTE_U
                            | crate::sv39::PTE_R
                            | crate::sv39::PTE_W
                            | crate::sv39::PTE_A
                            | crate::sv39::PTE_D,
                    );
                }
            }
        }
        unsafe { USER_BRK = req_brk };
        tf.a0 = req_brk;
    } else {
        // Shrink? Not implemented, just accept
        unsafe { USER_BRK = req_brk };
        tf.a0 = req_brk;
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_gettime(tf: &mut TrapFrame) {
    // a0 = ptr to timeval/timespec (ignored for now, just return ticks)
    // Return ticks in a0
    tf.a0 = crate::timer::TICKS.load(core::sync::atomic::Ordering::Relaxed) as usize;
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_creat(tf: &mut TrapFrame) {
    // a0 = path (C string in user VA), a1 = mode (ignored for now)
    let path_va = tf.a0;
    let mut buf = [0u8; 256];
    let path = match read_user_cstr_in_page(path_va, 255, &mut buf) {
        Ok(s) => s,
        Err(_) => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    let _ = write!(crate::uart::Uart::new(), "sys_creat: path='{}'\r\n", path);
    
    // Debug: check if FD_TABLE is locked before we do anything
    if FD_TABLE.is_locked() {
        let _ = write!(crate::uart::Uart::new(), "sys_creat: WARNING - FD_TABLE is already locked!\r\n");
    }

    match fs::create_file(path) {
        Ok(idx) => {
            let _ = write!(crate::uart::Uart::new(), "sys_creat: create_file ok\r\n");
            
            // Debug: check again before calling fd_alloc
            if FD_TABLE.is_locked() {
                let _ = write!(crate::uart::Uart::new(), "sys_creat: WARNING - FD_TABLE locked after create_file!\r\n");
            }
            
            if let Some(fd) = fd_alloc(FileType::Writable(idx), true) {
                let _ = write!(crate::uart::Uart::new(), "sys_creat: created fd={}\r\n", fd);
                tf.a0 = fd;
            } else {
                let _ = write!(crate::uart::Uart::new(), "sys_creat: fd_alloc failed\r\n");
                tf.a0 = usize::MAX;
            }
        }
        Err(_) => {
            let _ = write!(
                crate::uart::Uart::new(),
                "sys_creat: create_file failed\r\n"
            );
            tf.a0 = usize::MAX;
        }
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_unlink(tf: &mut TrapFrame) {
    // a0 = path (C string in user VA)
    let path_va = tf.a0;
    let mut buf = [0u8; 256];
    let path = match read_user_cstr_in_page(path_va, 255, &mut buf) {
        Ok(s) => s,
        Err(_) => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    let _ = write!(crate::uart::Uart::new(), "sys_unlink: path='{}'\r\n", path);

    match fs::unlink_file(path) {
        Ok(_) => tf.a0 = 0,
        Err(_) => tf.a0 = usize::MAX,
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_stat(tf: &mut TrapFrame) {
    // a0 = path (C string in user VA), a1 = stat buffer (user VA)
    let path_va = tf.a0;
    let stat_buf = tf.a1;

    let mut path_buf = [0u8; 256];
    let path = match read_user_cstr_in_page(path_va, 255, &mut path_buf) {
        Ok(s) => s,
        Err(_) => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    let _ = write!(crate::uart::Uart::new(), "sys_stat: path='{}'\r\n", path);

    match fs::stat_file(path) {
        Some(stat) => {
            // Write simplified stat structure to user buffer
            // For now, just write size and mode (8 bytes each)
            let stat_data = [stat.size as u64, stat.mode as u64];

            unsafe {
                with_sum_no_timer(|| {
                    let ptr = stat_buf as *mut u64;
                    core::ptr::write(ptr, stat_data[0]);
                    core::ptr::write(ptr.add(1), stat_data[1]);
                });
            }
            tf.a0 = 0;
        }
        None => {
            tf.a0 = usize::MAX;
        }
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_chmod(tf: &mut TrapFrame) {
    // a0 = path (C string in user VA), a1 = mode
    let path_va = tf.a0;
    let mode = tf.a1 as u32;

    let mut buf = [0u8; 256];
    let path = match read_user_cstr_in_page(path_va, 255, &mut buf) {
        Ok(s) => s,
        Err(_) => {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
    };

    let _ = write!(
        crate::uart::Uart::new(),
        "sys_chmod: path='{}' mode={:o}\r\n",
        path,
        mode
    );

    match fs::chmod_file(path, mode) {
        Ok(_) => tf.a0 = 0,
        Err(_) => tf.a0 = usize::MAX,
    }
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_readdir(tf: &mut TrapFrame) {
    // a0 = buffer (user VA), a1 = buffer length
    let buf_va = tf.a0;
    let mut len = tf.a1;

    if buf_va == 0 || len == 0 {
        tf.a0 = 0;
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }

    // Cap to page boundary
    len = cap_to_page(buf_va, len);

    // Use a kernel buffer to collect the filenames
    let mut kernel_buf = [0u8; 4096];
    let safe_len = core::cmp::min(len, kernel_buf.len());
    
    // Get list of writable files
    let count = fs::list_writable_files(&mut kernel_buf[..safe_len]);

    // Copy to user space
    if count > 0 {
        // Find actual bytes used (up to the last null terminator)
        let mut bytes_used = 0usize;
        let mut nulls_found = 0usize;
        for i in 0..safe_len {
            if kernel_buf[i] == 0 {
                nulls_found += 1;
                bytes_used = i + 1;
                if nulls_found == count {
                    break;
                }
            }
        }

        let _copied = copy_to_user(buf_va, &kernel_buf[..bytes_used]);
        tf.a0 = count;
    } else {
        tf.a0 = 0;
    }

    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_get_fb_info(tf: &mut TrapFrame) {
    // a0 = pointer to FbInfo struct in user space
    let info_va = tf.a0;
    
    if info_va == 0 {
        tf.a0 = usize::MAX;
        tf.sepc = tf.sepc.wrapping_add(4);
        return;
    }
    
    // Get framebuffer info from display subsystem
    if let Some(fb) = crate::display::get_framebuffer() {
        let fb_info = fb.info();
        
        // Map the framebuffer into user space
        let user_fb_va = unsafe { 
            crate::sv39::map_framebuffer_to_user(fb_info.phys_addr, fb_info.size) 
        };
        
        if user_fb_va == 0 {
            tf.a0 = usize::MAX;
            tf.sepc = tf.sepc.wrapping_add(4);
            return;
        }
        
        // Struct to write to user space (must match usys::FbInfo)
        #[repr(C)]
        struct FbInfoReply {
            width: usize,
            height: usize,
            stride: usize,
            addr: usize,
        }
        
        let reply = FbInfoReply {
            width: fb_info.width,
            height: fb_info.height,
            stride: fb_info.stride,
            addr: user_fb_va, // Return user VA, not physical address
        };
        
        // Copy to user space
        unsafe {
            with_sum(|| {
                let reply_bytes = core::slice::from_raw_parts(
                    &reply as *const _ as *const u8,
                    core::mem::size_of::<FbInfoReply>()
                );
                let user_ptr = info_va as *mut u8;
                core::ptr::copy_nonoverlapping(reply_bytes.as_ptr(), user_ptr, reply_bytes.len());
            });
        }
        
        tf.a0 = 0; // Success
    } else {
        tf.a0 = usize::MAX; // No framebuffer available
    }
    
    tf.sepc = tf.sepc.wrapping_add(4);
}

fn sys_fb_flush(tf: &mut TrapFrame) {
    // Flush framebuffer changes to the display device
    if crate::display::flush_framebuffer() {
        tf.a0 = 0; // Success
    } else {
        tf.a0 = usize::MAX; // No framebuffer or flush failed
    }
    
    tf.sepc = tf.sepc.wrapping_add(4);
}
