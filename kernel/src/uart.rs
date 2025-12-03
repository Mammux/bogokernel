use core::fmt;

const UART0_BASE: usize = 0x1000_0000;

/* 16550 registers (byte offsets) */
const RBR_THR_DLL: usize = 0x00; // Rx Buffer / Tx Holding / Div Latch Low
const LSR: usize = 0x05; // Line Status Register

/* LSR bits */
const LSR_TX_IDLE: u8 = 1 << 5; // THR empty

#[inline(always)]
fn mmio8(addr: usize) -> *mut u8 {
    addr as *mut u8
}

pub struct Uart;

impl Uart {
    pub const fn new() -> Self {
        Uart
    }

    #[inline(always)]
    fn lsr(&self) -> u8 {
        unsafe { core::ptr::read_volatile(mmio8(UART0_BASE + LSR)) }
    }

    #[inline(always)]
    pub fn write_byte(&mut self, byte: u8) {
        // Wait until TX holding register is empty
        while (self.lsr() & LSR_TX_IDLE) == 0 {}
        unsafe { core::ptr::write_volatile(mmio8(UART0_BASE + RBR_THR_DLL), byte) }
    }

        /// Is there a byte waiting in the RX FIFO? (LSR bit 0 = DR)
    pub fn can_read(&self) -> bool {
        self.lsr() & 0x01 != 0
    }

    /// Non-blocking read: returns Some(byte) if available.
    pub fn try_read_byte(&mut self) -> Option<u8> {
        if self.can_read() {
            // RBR at offset 0
            let b = unsafe { core::ptr::read_volatile(UART0_BASE as *const u8) };
            Some(b)
        } else {
            None
        }
    }

    /// Blocking read: waits until a byte arrives.
    #[allow(dead_code)]
    pub fn read_byte(&mut self) -> u8 {
        loop {
            if let Some(b) = self.try_read_byte() { return b; }
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            match b {
                b'\n' => {
                    self.write_byte(b'\r');
                    self.write_byte(b'\n');
                }
                byte => self.write_byte(byte),
            }
        }
        Ok(())
    }
}

