// kernel/src/keyboard.rs
//! VirtIO keyboard driver for QEMU virt machine.
//!
//! This module implements a VirtIO input device driver that handles keyboard events.
//! It provides a unified input buffer that can be read by user programs via stdin.

use core::mem::size_of;
use spin::Mutex;

// VirtIO input device constants
const VIRTIO_INPUT_DEVICE_ID: u32 = 18; // VirtIO input device type

// VirtIO MMIO register offsets (version 1 & 2)
const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
const VIRTIO_MMIO_VERSION: usize = 0x004;
const VIRTIO_MMIO_DEVICE_ID: usize = 0x008;
const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x020;
const VIRTIO_MMIO_GUEST_PAGE_SIZE: usize = 0x028;
const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
const VIRTIO_MMIO_QUEUE_ALIGN: usize = 0x03c;
const VIRTIO_MMIO_QUEUE_PFN: usize = 0x040;
const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050;
const VIRTIO_MMIO_INTERRUPT_STATUS: usize = 0x060;
const VIRTIO_MMIO_INTERRUPT_ACK: usize = 0x064;
const VIRTIO_MMIO_STATUS: usize = 0x070;
const VIRTIO_MMIO_CONFIG: usize = 0x100;

// VirtIO status bits
const VIRTIO_STATUS_ACKNOWLEDGE: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_FEATURES_OK: u32 = 8;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;

// Virtqueue descriptor flags
const VIRTQ_DESC_F_WRITE: u16 = 2;

// VirtIO input event types (from Linux input.h)
const EV_KEY: u16 = 0x01;

// Queue size
const QUEUE_SIZE: usize = 16;
const PAGE_SIZE: usize = 4096;

// Input buffer size (circular buffer for keyboard input)
const INPUT_BUFFER_SIZE: usize = 256;

// VirtIO input event structure
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct VirtioInputEvent {
    event_type: u16,
    code: u16,
    value: u32,
}

// Virtqueue descriptor
#[repr(C)]
#[derive(Copy, Clone)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

// Available ring
#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
}

// Used ring element
#[repr(C)]
#[derive(Copy, Clone)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

// Used ring
#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; QUEUE_SIZE],
}

/// Global keyboard input buffer
/// This buffer holds decoded ASCII characters from keyboard events.
/// Both the keyboard driver and serial input feed into this buffer.
pub struct InputBuffer {
    buffer: [u8; INPUT_BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
}

impl InputBuffer {
    const fn new() -> Self {
        Self {
            buffer: [0; INPUT_BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// Push a byte into the buffer. Returns false if buffer is full.
    pub fn push(&mut self, byte: u8) -> bool {
        let next_write = (self.write_pos + 1) % INPUT_BUFFER_SIZE;
        if next_write == self.read_pos {
            return false; // Buffer full
        }
        self.buffer[self.write_pos] = byte;
        self.write_pos = next_write;
        true
    }

    /// Pop a byte from the buffer. Returns None if buffer is empty.
    pub fn pop(&mut self) -> Option<u8> {
        if self.read_pos == self.write_pos {
            return None; // Buffer empty
        }
        let byte = self.buffer[self.read_pos];
        self.read_pos = (self.read_pos + 1) % INPUT_BUFFER_SIZE;
        Some(byte)
    }

    /// Check if the buffer is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.read_pos == self.write_pos
    }

    /// Get number of bytes available to read.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            INPUT_BUFFER_SIZE - self.read_pos + self.write_pos
        }
    }
}

/// Global input buffer shared between keyboard and serial
static INPUT_BUFFER: Mutex<InputBuffer> = Mutex::new(InputBuffer::new());

/// Push a byte to the global input buffer (used by both keyboard and serial).
pub fn push_input(byte: u8) {
    let mut buf = INPUT_BUFFER.lock();
    let _ = buf.push(byte);
}

/// Pop a byte from the global input buffer.
pub fn pop_input() -> Option<u8> {
    let mut buf = INPUT_BUFFER.lock();
    buf.pop()
}

/// Check if there's input available in the buffer.
pub fn has_input() -> bool {
    let buf = INPUT_BUFFER.lock();
    !buf.is_empty()
}

/// VirtIO keyboard driver state
struct VirtioKeyboard {
    mmio_base: usize,
    last_used_idx: u16,
}

// Static storage for keyboard driver
static mut KEYBOARD: Option<VirtioKeyboard> = None;
static mut KEYBOARD_INITIALIZED: bool = false;

// Static memory for virtqueue (must be page-aligned)
const PADDING_SIZE: usize = PAGE_SIZE - size_of::<[VirtqDesc; QUEUE_SIZE]>() - size_of::<VirtqAvail>();

#[repr(C, align(4096))]
struct KeyboardQueueMemory {
    desc: [VirtqDesc; QUEUE_SIZE],
    avail: VirtqAvail,
    _padding: [u8; PADDING_SIZE],
    used: VirtqUsed,
}

static mut KEYBOARD_QUEUE: KeyboardQueueMemory = KeyboardQueueMemory {
    desc: [VirtqDesc {
        addr: 0,
        len: 0,
        flags: 0,
        next: 0,
    }; QUEUE_SIZE],
    avail: VirtqAvail {
        flags: 0,
        idx: 0,
        ring: [0; QUEUE_SIZE],
    },
    _padding: [0; PADDING_SIZE],
    used: VirtqUsed {
        flags: 0,
        idx: 0,
        ring: [VirtqUsedElem { id: 0, len: 0 }; QUEUE_SIZE],
    },
};

// Event buffers for receiving keyboard events
static mut EVENT_BUFFERS: [VirtioInputEvent; QUEUE_SIZE] = [VirtioInputEvent {
    event_type: 0,
    code: 0,
    value: 0,
}; QUEUE_SIZE];

/// Probe for and initialize VirtIO keyboard device.
/// Returns true if keyboard was found and initialized.
#[allow(static_mut_refs)]
pub fn init() -> bool {
    use core::fmt::Write;
    let mut uart = crate::uart::Uart::new();

    let _ = writeln!(uart, "[Keyboard] Starting VirtIO keyboard probe...");

    // Scan VirtIO MMIO slots
    const VIRTIO_MMIO_BASE: usize = 0x10001000;
    const VIRTIO_MMIO_SIZE: usize = 0x1000;
    const VIRTIO_MMIO_COUNT: usize = 8;

    for i in 0..VIRTIO_MMIO_COUNT {
        let base = VIRTIO_MMIO_BASE + i * VIRTIO_MMIO_SIZE;

        // Check magic value
        let magic = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_MAGIC_VALUE) as *const u32) };
        if magic != 0x74726976 {
            continue;
        }

        // Check version
        let version = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_VERSION) as *const u32) };
        if version != 1 && version != 2 {
            continue;
        }

        // Check device ID
        let device_id = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_DEVICE_ID) as *const u32) };
        if device_id == 0 {
            continue; // Empty slot
        }
        
        if device_id == VIRTIO_INPUT_DEVICE_ID {
            let _ = writeln!(uart, "[Keyboard] Found VirtIO input device at slot {} (0x{:08x})", i, base);
            
            // Check if this is a keyboard (subtype in config space)
            // For VirtIO input, the config space contains device identification
            let select_byte = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_CONFIG) as *const u8) };
            let _ = writeln!(uart, "[Keyboard]   Config select byte: 0x{:02x}", select_byte);
            
            // Initialize the keyboard device
            if init_device(base) {
                let _ = writeln!(uart, "[Keyboard] VirtIO keyboard initialized successfully");
                return true;
            }
        }
    }

    let _ = writeln!(uart, "[Keyboard] No VirtIO keyboard device found");
    false
}

/// Initialize a VirtIO input device at the given MMIO base address.
#[allow(static_mut_refs)]
fn init_device(mmio_base: usize) -> bool {
    use core::fmt::Write;
    let mut uart = crate::uart::Uart::new();

    unsafe {
        // Reset device
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, 0);

        // Acknowledge device
        let mut status = VIRTIO_STATUS_ACKNOWLEDGE;
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);

        // Set driver bit
        status |= VIRTIO_STATUS_DRIVER;
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);

        // Read and accept device features
        let _device_features = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_DEVICE_FEATURES) as *const u32);
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_DRIVER_FEATURES) as *mut u32, 0);

        // Features OK
        status |= VIRTIO_STATUS_FEATURES_OK;
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);

        // Verify features OK
        let status_check = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *const u32);
        if (status_check & VIRTIO_STATUS_FEATURES_OK) == 0 {
            let _ = writeln!(uart, "[Keyboard] ERROR: Device rejected features");
            return false;
        }

        // Set up eventq (queue 0) - receives keyboard events
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_SEL) as *mut u32, 0);
        let queue_max = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_QUEUE_NUM_MAX) as *const u32);
        if queue_max < QUEUE_SIZE as u32 {
            let _ = writeln!(uart, "[Keyboard] ERROR: Queue too small (max={})", queue_max);
            return false;
        }

        // Set queue size
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_NUM) as *mut u32, QUEUE_SIZE as u32);

        // Set guest page size
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_GUEST_PAGE_SIZE) as *mut u32, PAGE_SIZE as u32);

        // Set queue alignment
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_ALIGN) as *mut u32, PAGE_SIZE as u32);

        // Set queue PFN
        let queue_pfn = (&raw const KEYBOARD_QUEUE as usize) / PAGE_SIZE;
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_PFN) as *mut u32, queue_pfn as u32);

        // Initialize descriptors with event buffers (device writes events to us)
        for i in 0..QUEUE_SIZE {
            KEYBOARD_QUEUE.desc[i].addr = (&raw mut EVENT_BUFFERS[i]) as u64;
            KEYBOARD_QUEUE.desc[i].len = size_of::<VirtioInputEvent>() as u32;
            KEYBOARD_QUEUE.desc[i].flags = VIRTQ_DESC_F_WRITE; // Device writes to this
            KEYBOARD_QUEUE.desc[i].next = 0;

            // Add to available ring
            KEYBOARD_QUEUE.avail.ring[i] = i as u16;
        }
        KEYBOARD_QUEUE.avail.idx = QUEUE_SIZE as u16;

        // Driver OK - device is ready
        status |= VIRTIO_STATUS_DRIVER_OK;
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);

        // Notify device that we have buffers available
        core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_NOTIFY) as *mut u32, 0);

        // Store keyboard state
        KEYBOARD = Some(VirtioKeyboard {
            mmio_base,
            last_used_idx: 0,
        });
        KEYBOARD_INITIALIZED = true;

        let _ = writeln!(uart, "[Keyboard] Device initialized at 0x{:08x}", mmio_base);
        true
    }
}

/// Poll for keyboard events and process them.
/// Should be called periodically (e.g., in timer interrupt or main loop).
#[allow(static_mut_refs)]
pub fn poll() {
    unsafe {
        if !KEYBOARD_INITIALIZED {
            return;
        }

        let keyboard = match KEYBOARD.as_mut() {
            Some(k) => k,
            None => return,
        };

        let mmio_base = keyboard.mmio_base;

        // Check for completed events in the used ring
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        let used_idx = core::ptr::read_volatile(&KEYBOARD_QUEUE.used.idx);

        while keyboard.last_used_idx != used_idx {
            let idx = keyboard.last_used_idx as usize % QUEUE_SIZE;
            let used_elem = core::ptr::read_volatile(&KEYBOARD_QUEUE.used.ring[idx]);
            let desc_idx = used_elem.id as usize;

            if desc_idx < QUEUE_SIZE {
                // Read the event
                let event = core::ptr::read_volatile(&EVENT_BUFFERS[desc_idx]);
                process_event(&event);

                // Resubmit the buffer
                KEYBOARD_QUEUE.avail.ring[KEYBOARD_QUEUE.avail.idx as usize % QUEUE_SIZE] = desc_idx as u16;
                core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
                KEYBOARD_QUEUE.avail.idx = KEYBOARD_QUEUE.avail.idx.wrapping_add(1);
            }

            keyboard.last_used_idx = keyboard.last_used_idx.wrapping_add(1);
        }

        // Notify device if we resubmitted buffers
        if keyboard.last_used_idx != used_idx {
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_NOTIFY) as *mut u32, 0);
        }

        // Acknowledge any pending interrupts
        let isr = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_INTERRUPT_STATUS) as *const u32);
        if isr != 0 {
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_INTERRUPT_ACK) as *mut u32, isr);
        }
    }
}

/// Process a keyboard event and convert it to ASCII if applicable.
fn process_event(event: &VirtioInputEvent) {
    // Only process key press events (value=1 means key down)
    if event.event_type != EV_KEY || event.value != 1 {
        return;
    }

    // Convert Linux key code to ASCII
    if let Some(ascii) = keycode_to_ascii(event.code) {
        push_input(ascii);
    }
}

/// Convert a Linux key code to ASCII character.
/// This handles standard US keyboard layout.
fn keycode_to_ascii(code: u16) -> Option<u8> {
    // Linux key codes (from linux/input-event-codes.h)
    // This is a subset covering common keys
    match code {
        // Number row
        2 => Some(b'1'),
        3 => Some(b'2'),
        4 => Some(b'3'),
        5 => Some(b'4'),
        6 => Some(b'5'),
        7 => Some(b'6'),
        8 => Some(b'7'),
        9 => Some(b'8'),
        10 => Some(b'9'),
        11 => Some(b'0'),
        12 => Some(b'-'),
        13 => Some(b'='),
        14 => Some(0x08), // Backspace
        
        // Top row (QWERTY)
        15 => Some(b'\t'), // Tab
        16 => Some(b'q'),
        17 => Some(b'w'),
        18 => Some(b'e'),
        19 => Some(b'r'),
        20 => Some(b't'),
        21 => Some(b'y'),
        22 => Some(b'u'),
        23 => Some(b'i'),
        24 => Some(b'o'),
        25 => Some(b'p'),
        26 => Some(b'['),
        27 => Some(b']'),
        28 => Some(b'\n'), // Enter
        
        // Home row (ASDF)
        30 => Some(b'a'),
        31 => Some(b's'),
        32 => Some(b'd'),
        33 => Some(b'f'),
        34 => Some(b'g'),
        35 => Some(b'h'),
        36 => Some(b'j'),
        37 => Some(b'k'),
        38 => Some(b'l'),
        39 => Some(b';'),
        40 => Some(b'\''),
        41 => Some(b'`'),
        43 => Some(b'\\'),
        
        // Bottom row (ZXCV)
        44 => Some(b'z'),
        45 => Some(b'x'),
        46 => Some(b'c'),
        47 => Some(b'v'),
        48 => Some(b'b'),
        49 => Some(b'n'),
        50 => Some(b'm'),
        51 => Some(b','),
        52 => Some(b'.'),
        53 => Some(b'/'),
        
        // Space
        57 => Some(b' '),
        
        // Arrow keys (send ANSI escape sequences would require multiple bytes)
        // For now, skip them
        
        // Escape
        1 => Some(0x1b),
        
        _ => None,
    }
}

/// Check if keyboard is initialized.
pub fn is_initialized() -> bool {
    unsafe { KEYBOARD_INITIALIZED }
}
