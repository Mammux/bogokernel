// kernel/src/trap.rs
use riscv::register::{scause, sepc, sie, sstatus, stvec};

pub use scause::{Exception, Interrupt, Trap};

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
        stvec::write(__trap_entry as usize, stvec::TrapMode::Direct);
        sstatus::set_sie(); // global S interrupts
        sie::set_stimer(); // Supervisor timer interrupt enable
    }
}

#[no_mangle]
extern "C" fn rust_trap(tf: &mut TrapFrame) {
    match scause::read().cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::timer::on_timer();
        }
        Trap::Exception(Exception::UserEnvCall) => {
            // Syscall ABI: a7 = nr, a0.. = args; ecall is 4-byte insn
            let nr = tf.a7;
            match nr {
                1 => {
                    // write(byte)
                    let ch = (tf.a0 & 0xFF) as u8;
                    let mut uart = crate::uart::Uart::new();
                    // Write raw (no newline translation)
                    // SAFETY: our driver is fine with direct bytes
                    use core::fmt::Write;
                    let _ = write!(uart, "{}", ch as char);
                    tf.sepc = tf.sepc.wrapping_add(4);
                }
                2 => {
                    // exit()
                    // Return to S-mode at after_user()
                    extern "C" {
                        fn after_user() -> !;
                    }
                    tf.sepc = after_user as usize;

                    // Set SPP=Supervisor in the *saved* sstatus
                    const SSTATUS_SPP_BIT: usize = 1 << 8;
                    tf.sstatus_bits |= SSTATUS_SPP_BIT;
                    // Also ensure interrupts enabled when we land in S
                    const SSTATUS_SPIE_BIT: usize = 1 << 5;
                    tf.sstatus_bits |= SSTATUS_SPIE_BIT;
                }
                _ => {
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
                "\r\n*** TRAP *** scause={:?} sepc=0x{:016x}",
                other,
                sepc::read()
            );
            loop {
                unsafe { core::arch::asm!("wfi") }
            }
        }
    }
}
