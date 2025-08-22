use core::fmt::Write;

// kernel/src/trap.rs
use riscv::{interrupt::supervisor::{Exception, Interrupt}, register::{scause, sepc, sie, sstatus, stval, stvec::{self, Stvec}}};
pub use scause::Trap;

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

