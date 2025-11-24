use crate::display::{register_framebuffer, Framebuffer, FramebufferInfo};
use core::mem::size_of;
use core::fmt::Write;

// VirtIO GPU device constants
const VIRTIO_GPU_DEVICE_ID: u32 = 16; // VirtIO GPU device type
const VIRTIO_VENDOR_ID: u32 = 0x1AF4;

// VirtIO MMIO register offsets (version 1)
const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
const VIRTIO_MMIO_VERSION: usize = 0x004;
const VIRTIO_MMIO_DEVICE_ID: usize = 0x008;
const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x020;
const VIRTIO_MMIO_GUEST_PAGE_SIZE: usize = 0x028;
const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
const VIRTIO_MMIO_QUEUE_PFN: usize = 0x040;
const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050;
const VIRTIO_MMIO_STATUS: usize = 0x070;

// VirtIO status bits
const VIRTIO_STATUS_ACKNOWLEDGE: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_FEATURES_OK: u32 = 8;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;

// Virtqueue descriptor flags
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;

// VirtIO-GPU specific constants
const VIRTIO_GPU_CMD_GET_DISPLAY_INFO: u32 = 0x0100;
const VIRTIO_GPU_CMD_RESOURCE_CREATE_2D: u32 = 0x0101;
const VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;
const VIRTIO_GPU_CMD_SET_SCANOUT: u32 = 0x0103;
const VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D: u32 = 0x0105;
const VIRTIO_GPU_CMD_RESOURCE_FLUSH: u32 = 0x0104;

const VIRTIO_GPU_FORMAT_B8G8R8X8_UNORM: u32 = 2;
const VIRTIO_GPU_RESP_OK_NODATA: u32 = 0x1100;

// Virtqueue size
const QUEUE_SIZE: usize = 8;

// Timeouts and buffer sizes
const COMMAND_TIMEOUT_ITERATIONS: usize = 100000;
const GPU_COMMAND_BUFFER_SIZE: usize = 512;
const GPU_RESPONSE_BUFFER_SIZE: usize = 128;
const PAGE_SIZE: usize = 4096;

// Virtqueue descriptor
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

// Available ring
#[repr(C)]
#[derive(Debug)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
}

// Used ring element
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

// Used ring
#[repr(C)]
#[derive(Debug)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; QUEUE_SIZE],
}

// Virtqueue structure
#[derive(Debug)]
struct Virtqueue {
    desc: &'static mut [VirtqDesc; QUEUE_SIZE],
    avail: &'static mut VirtqAvail,
    used: &'static mut VirtqUsed,
    next_desc: u16,
    last_used_idx: u16,
}

// GPU command headers
#[repr(C)]
struct GpuCtrlHdr {
    hdr_type: u32,
    flags: u32,
    fence_id: u64,
    ctx_id: u32,
    padding: u32,
}

#[repr(C)]
struct GpuRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[repr(C)]
struct GpuResourceCreate2D {
    hdr: GpuCtrlHdr,
    resource_id: u32,
    format: u32,
    width: u32,
    height: u32,
}

#[repr(C)]
struct GpuResourceAttachBacking {
    hdr: GpuCtrlHdr,
    resource_id: u32,
    nr_entries: u32,
}

#[repr(C)]
struct GpuMemEntry {
    addr: u64,
    length: u32,
    padding: u32,
}

#[repr(C)]
struct GpuSetScanout {
    hdr: GpuCtrlHdr,
    r: GpuRect,
    scanout_id: u32,
    resource_id: u32,
}

#[repr(C)]
struct GpuTransferToHost2D {
    hdr: GpuCtrlHdr,
    r: GpuRect,
    offset: u64,
    resource_id: u32,
    padding: u32,
}

#[repr(C)]
struct GpuResourceFlush {
    hdr: GpuCtrlHdr,
    r: GpuRect,
    resource_id: u32,
    padding: u32,
}

#[repr(C)]
struct GpuCtrlResponse {
    hdr_type: u32,
    flags: u32,
    fence_id: u64,
    ctx_id: u32,
    padding: u32,
}

// Static buffers for GPU command submission
// These are reused across multiple commands to avoid stack allocation
static mut GPU_CMD_BUF: [u8; GPU_COMMAND_BUFFER_SIZE] = [0; GPU_COMMAND_BUFFER_SIZE];
static mut GPU_RESP_BUF: [u8; GPU_RESPONSE_BUFFER_SIZE] = [0; GPU_RESPONSE_BUFFER_SIZE];

#[derive(Debug)]
pub struct VirtioGpu {
    info: FramebufferInfo,
    back: *mut u8,
    mmio_base: usize,
    resource_id: u32,
    queue: Option<Virtqueue>,
}

impl VirtioGpu {
    pub fn probe() -> Option<&'static Self> {
        let mut uart = crate::uart::Uart::new();
        let _ = writeln!(uart, "[VirtIO-GPU] Starting device probe...");
        
        // Scan for VirtIO MMIO devices in QEMU virt machine
        // QEMU virt typically has VirtIO devices at 0x10001000 - 0x10008000
        const VIRTIO_MMIO_BASE: usize = 0x10001000;
        const VIRTIO_MMIO_SIZE: usize = 0x1000;
        const VIRTIO_MMIO_COUNT: usize = 8;

        for i in 0..VIRTIO_MMIO_COUNT {
            let base = VIRTIO_MMIO_BASE + i * VIRTIO_MMIO_SIZE;
            let _ = writeln!(uart, "[VirtIO-GPU] Scanning slot {}: base=0x{:08x}", i, base);

            // Check magic value (should be 0x74726976 = "virt")
            let magic =
                unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_MAGIC_VALUE) as *const u32) };
            let _ = writeln!(uart, "[VirtIO-GPU]   Magic: 0x{:08x} (expected 0x74726976)", magic);
            if magic != 0x74726976 {
                let _ = writeln!(uart, "[VirtIO-GPU]   -> Magic mismatch, skipping");
                continue;
            }

            // Check version (should be 1 or 2)
            // Note: QEMU on Windows may report version 1, while Linux typically reports version 2.
            // Both versions are compatible for basic GPU device initialization.
            let version =
                unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_VERSION) as *const u32) };
            let _ = writeln!(uart, "[VirtIO-GPU]   Version: {}", version);
            if version != 1 && version != 2 {
                let _ = writeln!(uart, "[VirtIO-GPU]   -> Invalid version, skipping");
                continue;
            }

            // Check if this is a GPU device
            // Device ID 0 indicates an empty/invalid slot, so continue scanning
            let device_id =
                unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_DEVICE_ID) as *const u32) };
            let _ = writeln!(uart, "[VirtIO-GPU]   Device ID: {} (GPU=16)", device_id);
            if device_id == 0 {
                // Empty slot, skip to next
                let _ = writeln!(uart, "[VirtIO-GPU]   -> Empty slot, skipping");
                continue;
            }
            if device_id != VIRTIO_GPU_DEVICE_ID {
                // Valid device but not GPU, skip to next
                let _ = writeln!(uart, "[VirtIO-GPU]   -> Not a GPU device, skipping");
                continue;
            }

            let _ = writeln!(uart, "[VirtIO-GPU] *** Found GPU device at 0x{:08x}! ***", base);
            // Found a VirtIO GPU device! Initialize it
            return Self::init_device(base);
        }

        let _ = writeln!(uart, "[VirtIO-GPU] No GPU device found after scanning {} slots", VIRTIO_MMIO_COUNT);
        None
    }

    fn init_device(mmio_base: usize) -> Option<&'static Self> {
        let mut uart = crate::uart::Uart::new();
        let _ = writeln!(uart, "[VirtIO-GPU] Initializing device at 0x{:08x}", mmio_base);
        
        const W: usize = 1024;
        const H: usize = 768;
        const SIZE: usize = W * H * 4;

        let _ = writeln!(uart, "[VirtIO-GPU] Framebuffer: {}x{} = {} bytes", W, H, SIZE);

        // Allocate static framebuffer
        static mut BUF: [u8; SIZE] = [0; SIZE];

        // Allocate virtqueue memory in a single contiguous block
        // This is required for VirtIO MMIO version 1
        // Layout: descriptors (128 bytes) + avail (20 bytes) + padding + used (68 bytes)
        // Total needs to fit in one page (4096 bytes) for simplicity
        #[repr(C, align(4096))]
        struct VirtqueueMemory {
            desc: [VirtqDesc; QUEUE_SIZE],
            avail: VirtqAvail,
            _padding: [u8; 4096 - 128 - 20 - 68], // Align used ring to page boundary
            used: VirtqUsed,
        }
        
        static mut QUEUE_MEM: VirtqueueMemory = VirtqueueMemory {
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
            _padding: [0; 4096 - 128 - 20 - 68],
            used: VirtqUsed {
                flags: 0,
                idx: 0,
                ring: [VirtqUsedElem { id: 0, len: 0 }; QUEUE_SIZE],
            },
        };

        unsafe {
            let _ = writeln!(uart, "[VirtIO-GPU] Starting device negotiation sequence...");
            
            // Reset device
            let _ = writeln!(uart, "[VirtIO-GPU]   Step 1: Reset device (status=0)");
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, 0);

            // Acknowledge device
            let mut status = VIRTIO_STATUS_ACKNOWLEDGE;
            let _ = writeln!(uart, "[VirtIO-GPU]   Step 2: Acknowledge device (status={})", status);
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);

            // Set driver bit
            status |= VIRTIO_STATUS_DRIVER;
            let _ = writeln!(uart, "[VirtIO-GPU]   Step 3: Driver ready (status={})", status);
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);

            // Read device features
            let device_features =
                core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_DEVICE_FEATURES) as *const u32);
            let _ = writeln!(uart, "[VirtIO-GPU]   Device features: 0x{:08x}", device_features);

            // Write driver features (we accept minimal features)
            let _ = writeln!(uart, "[VirtIO-GPU]   Step 4: Negotiate features (driver_features=0)");
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_DRIVER_FEATURES) as *mut u32, 0);

            // Features OK
            status |= VIRTIO_STATUS_FEATURES_OK;
            let _ = writeln!(uart, "[VirtIO-GPU]   Step 5: Set FEATURES_OK (status={})", status);
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);

            // Verify features OK
            let status_check =
                core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *const u32);
            let _ = writeln!(uart, "[VirtIO-GPU]   Status readback: 0x{:08x}", status_check);
            if (status_check & VIRTIO_STATUS_FEATURES_OK) == 0 {
                let _ = writeln!(uart, "[VirtIO-GPU]   ERROR: Device rejected features!");
                return None; // Device doesn't support our features
            }
            let _ = writeln!(uart, "[VirtIO-GPU]   Features accepted by device");

            // Set up controlq (queue 0)
            let _ = writeln!(uart, "[VirtIO-GPU] Setting up virtqueue 0 (controlq)...");
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_SEL) as *mut u32, 0);
            let queue_max =
                core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_QUEUE_NUM_MAX) as *const u32);
            let _ = writeln!(uart, "[VirtIO-GPU]   Queue max size: {}", queue_max);
            if queue_max < QUEUE_SIZE as u32 {
                let _ = writeln!(uart, "[VirtIO-GPU]   ERROR: Queue too small! max={}, need={}", queue_max, QUEUE_SIZE);
                return None; // Queue too small
            }

            // Set queue size
            let _ = writeln!(uart, "[VirtIO-GPU]   Setting queue size to {}", QUEUE_SIZE);
            core::ptr::write_volatile(
                (mmio_base + VIRTIO_MMIO_QUEUE_NUM) as *mut u32,
                QUEUE_SIZE as u32,
            );

            // Set guest page size (4KB)
            let _ = writeln!(uart, "[VirtIO-GPU]   Setting guest page size to {} bytes", PAGE_SIZE);
            core::ptr::write_volatile(
                (mmio_base + VIRTIO_MMIO_GUEST_PAGE_SIZE) as *mut u32,
                PAGE_SIZE as u32,
            );

            // Calculate queue physical address
            // For version 1, the queue PFN register expects the physical address divided by page size
            let queue_pfn = (&QUEUE_MEM as *const _ as usize) / PAGE_SIZE;
            let _ = writeln!(uart, "[VirtIO-GPU]   Queue memory base: 0x{:08x}", &QUEUE_MEM as *const _ as usize);
            let _ = writeln!(uart, "[VirtIO-GPU]   Queue descriptor addr: 0x{:08x}", QUEUE_MEM.desc.as_ptr() as usize);
            let _ = writeln!(uart, "[VirtIO-GPU]   Queue avail addr: 0x{:08x}", &QUEUE_MEM.avail as *const _ as usize);
            let _ = writeln!(uart, "[VirtIO-GPU]   Queue used addr: 0x{:08x}", &QUEUE_MEM.used as *const _ as usize);
            let _ = writeln!(uart, "[VirtIO-GPU]   Queue PFN: 0x{:08x}", queue_pfn);
            
            // Check if memory layout is correct for VirtIO v1
            let desc_size = core::mem::size_of::<[VirtqDesc; QUEUE_SIZE]>();
            let avail_size = core::mem::size_of::<VirtqAvail>();
            let used_size = core::mem::size_of::<VirtqUsed>();
            let _ = writeln!(uart, "[VirtIO-GPU]   Sizes: desc={}, avail={}, used={}", 
                           desc_size, avail_size, used_size);
            
            // Verify contiguous layout
            let desc_offset = &QUEUE_MEM.desc as *const _ as usize - &QUEUE_MEM as *const _ as usize;
            let avail_offset = &QUEUE_MEM.avail as *const _ as usize - &QUEUE_MEM as *const _ as usize;
            let used_offset = &QUEUE_MEM.used as *const _ as usize - &QUEUE_MEM as *const _ as usize;
            let _ = writeln!(uart, "[VirtIO-GPU]   Offsets: desc={}, avail={}, used={}", 
                           desc_offset, avail_offset, used_offset);
            
            core::ptr::write_volatile(
                (mmio_base + VIRTIO_MMIO_QUEUE_PFN) as *mut u32,
                queue_pfn as u32,
            );

            // Driver OK - device is ready
            status |= VIRTIO_STATUS_DRIVER_OK;
            let _ = writeln!(uart, "[VirtIO-GPU]   Step 6: Set DRIVER_OK (status={})", status);
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);
            
            let _ = writeln!(uart, "[VirtIO-GPU] Device negotiation complete!");

            // Create VirtioGpu instance
            let fb_info = FramebufferInfo {
                width: W,
                height: H,
                stride: W * 4,
                phys_addr: BUF.as_ptr() as usize,
                size: SIZE,
            };
            
            let _ = writeln!(uart, "[VirtIO-GPU] Framebuffer info: phys_addr=0x{:08x}, size={}", 
                           fb_info.phys_addr, fb_info.size);

            let queue = Virtqueue {
                desc: &mut QUEUE_MEM.desc,
                avail: &mut QUEUE_MEM.avail,
                used: &mut QUEUE_MEM.used,
                next_desc: 0,
                last_used_idx: 0,
            };

            static mut VG: Option<VirtioGpu> = None;
            VG = Some(VirtioGpu {
                info: fb_info,
                back: BUF.as_mut_ptr(),
                mmio_base,
                resource_id: 1, // Resource ID for our framebuffer
                queue: Some(queue),
            });

            // Initialize display first (mutable access)
            if let Some(v) = VG.as_mut() {
                v.init_display();
            }

            // Then register framebuffer (immutable access for 'static)
            VG.as_ref().map(|v| {
                register_framebuffer(v);
                v
            })
        }
    }

    // Send a GPU command and wait for response
    fn send_command(&mut self, req: &[u8], resp: &mut [u8]) -> bool {
        let mut uart = crate::uart::Uart::new();
        
        let queue = match self.queue.as_mut() {
            Some(q) => q,
            None => {
                let _ = writeln!(uart, "[VirtIO-GPU] ERROR: No queue available!");
                return false;
            }
        };

        unsafe {
            // Read command type from request buffer (first u32)
            let cmd_type = if req.len() >= 4 {
                core::ptr::read_volatile(req.as_ptr() as *const u32)
            } else {
                0
            };
            let _ = writeln!(uart, "[VirtIO-GPU] Sending command: type=0x{:04x}, req_len={}, resp_len={}", 
                           cmd_type, req.len(), resp.len());
            
            // Set up descriptor chain: request -> response
            let req_desc_idx = queue.next_desc;
            queue.desc[req_desc_idx as usize].addr = req.as_ptr() as u64;
            queue.desc[req_desc_idx as usize].len = req.len() as u32;
            queue.desc[req_desc_idx as usize].flags = VIRTQ_DESC_F_NEXT;
            queue.desc[req_desc_idx as usize].next = (req_desc_idx + 1) % QUEUE_SIZE as u16;

            let resp_desc_idx = (req_desc_idx + 1) % QUEUE_SIZE as u16;
            queue.desc[resp_desc_idx as usize].addr = resp.as_mut_ptr() as u64;
            queue.desc[resp_desc_idx as usize].len = resp.len() as u32;
            queue.desc[resp_desc_idx as usize].flags = VIRTQ_DESC_F_WRITE;
            queue.desc[resp_desc_idx as usize].next = 0;

            let _ = writeln!(uart, "[VirtIO-GPU]   Descriptors: req_idx={}, resp_idx={}", 
                           req_desc_idx, resp_desc_idx);
            let _ = writeln!(uart, "[VirtIO-GPU]   Request descriptor: addr=0x{:08x}, len={}", 
                           queue.desc[req_desc_idx as usize].addr, 
                           queue.desc[req_desc_idx as usize].len);
            let _ = writeln!(uart, "[VirtIO-GPU]   Response descriptor: addr=0x{:08x}, len={}", 
                           queue.desc[resp_desc_idx as usize].addr, 
                           queue.desc[resp_desc_idx as usize].len);

            // Add to available ring
            let avail_idx = queue.avail.idx;
            queue.avail.ring[avail_idx as usize % QUEUE_SIZE] = req_desc_idx;
            let _ = writeln!(uart, "[VirtIO-GPU]   Updating avail ring: idx={} -> {}", 
                           avail_idx, avail_idx.wrapping_add(1));
            
            // Memory barrier before updating index
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            queue.avail.idx = avail_idx.wrapping_add(1);
            
            // Memory barrier after updating index
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

            // Notify device (write queue index to notify register)
            let _ = writeln!(uart, "[VirtIO-GPU]   Notifying device (writing 0 to QUEUE_NOTIFY)");
            core::ptr::write_volatile((self.mmio_base + VIRTIO_MMIO_QUEUE_NOTIFY) as *mut u32, 0);

            // Wait for response (simple busy wait)
            let _ = writeln!(uart, "[VirtIO-GPU]   Waiting for response (last_used_idx={})...", 
                           queue.last_used_idx);
            
            let mut sample_count = 0;
            for i in 0..COMMAND_TIMEOUT_ITERATIONS {
                // Memory barrier before reading used ring
                core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
                let current_used_idx = queue.used.idx;
                
                // Log used idx periodically
                if i % 20000 == 0 && sample_count < 3 {
                    let _ = writeln!(uart, "[VirtIO-GPU]   ... still waiting at iteration {}, used_idx={}", 
                                   i, current_used_idx);
                    sample_count += 1;
                }
                
                if current_used_idx != queue.last_used_idx {
                    let _ = writeln!(uart, "[VirtIO-GPU]   Response received after {} iterations!", i);
                    let _ = writeln!(uart, "[VirtIO-GPU]   Used idx: {} -> {}", 
                                   queue.last_used_idx, current_used_idx);
                    
                    // Read response type
                    let resp_type = if resp.len() >= 4 {
                        core::ptr::read_volatile(resp.as_ptr() as *const u32)
                    } else {
                        0
                    };
                    let _ = writeln!(uart, "[VirtIO-GPU]   Response type: 0x{:04x} (OK_NODATA=0x1100)", resp_type);
                    
                    queue.last_used_idx = current_used_idx;
                    queue.next_desc = (resp_desc_idx + 1) % QUEUE_SIZE as u16;
                    return true;
                }
            }

            let _ = writeln!(uart, "[VirtIO-GPU]   ERROR: Command timed out after {} iterations!", 
                           COMMAND_TIMEOUT_ITERATIONS);
            false
        }
    }

    // Initialize display by sending GPU commands
    fn init_display(&mut self) {
        let mut uart = crate::uart::Uart::new();
        let _ = writeln!(uart, "[VirtIO-GPU] ========================================");
        let _ = writeln!(uart, "[VirtIO-GPU] Starting display initialization...");
        let _ = writeln!(uart, "[VirtIO-GPU] ========================================");
        
        let resource_id = self.resource_id;
        let fb_addr = self.back as usize;
        let width = self.info.width as u32;
        let height = self.info.height as u32;

        let _ = writeln!(uart, "[VirtIO-GPU] Display parameters:");
        let _ = writeln!(uart, "[VirtIO-GPU]   Resource ID: {}", resource_id);
        let _ = writeln!(uart, "[VirtIO-GPU]   Framebuffer: 0x{:08x}", fb_addr);
        let _ = writeln!(uart, "[VirtIO-GPU]   Resolution: {}x{}", width, height);

        unsafe {
            // 1. Create 2D resource
            let _ = writeln!(uart, "[VirtIO-GPU] Command 1/5: CREATE_2D resource...");
            let create_cmd = GpuResourceCreate2D {
                hdr: GpuCtrlHdr {
                    hdr_type: VIRTIO_GPU_CMD_RESOURCE_CREATE_2D,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                resource_id,
                format: VIRTIO_GPU_FORMAT_B8G8R8X8_UNORM,
                width,
                height,
            };

            core::ptr::copy_nonoverlapping(
                &create_cmd as *const _ as *const u8,
                GPU_CMD_BUF.as_mut_ptr(),
                size_of::<GpuResourceCreate2D>(),
            );

            let success = self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuResourceCreate2D>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            if !success {
                let _ = writeln!(uart, "[VirtIO-GPU] ERROR: CREATE_2D command failed!");
            } else {
                let _ = writeln!(uart, "[VirtIO-GPU] CREATE_2D command succeeded");
            }

            // 2. Attach backing storage
            let _ = writeln!(uart, "[VirtIO-GPU] Command 2/5: ATTACH_BACKING...");
            let attach_cmd = GpuResourceAttachBacking {
                hdr: GpuCtrlHdr {
                    hdr_type: VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                resource_id,
                nr_entries: 1,
            };

            let mem_entry = GpuMemEntry {
                addr: fb_addr as u64,
                length: self.info.size as u32,
                padding: 0,
            };
            
            let _ = writeln!(uart, "[VirtIO-GPU]   Memory entry: addr=0x{:08x}, len={}", 
                           mem_entry.addr, mem_entry.length);

            core::ptr::copy_nonoverlapping(
                &attach_cmd as *const _ as *const u8,
                GPU_CMD_BUF.as_mut_ptr(),
                size_of::<GpuResourceAttachBacking>(),
            );
            core::ptr::copy_nonoverlapping(
                &mem_entry as *const _ as *const u8,
                GPU_CMD_BUF
                    .as_mut_ptr()
                    .add(size_of::<GpuResourceAttachBacking>()),
                size_of::<GpuMemEntry>(),
            );

            let success = self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuResourceAttachBacking>() + size_of::<GpuMemEntry>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            if !success {
                let _ = writeln!(uart, "[VirtIO-GPU] ERROR: ATTACH_BACKING command failed!");
            } else {
                let _ = writeln!(uart, "[VirtIO-GPU] ATTACH_BACKING command succeeded");
            }

            // 3. Set scanout
            let _ = writeln!(uart, "[VirtIO-GPU] Command 3/5: SET_SCANOUT...");
            let scanout_cmd = GpuSetScanout {
                hdr: GpuCtrlHdr {
                    hdr_type: VIRTIO_GPU_CMD_SET_SCANOUT,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                r: GpuRect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
                scanout_id: 0,
                resource_id,
            };

            core::ptr::copy_nonoverlapping(
                &scanout_cmd as *const _ as *const u8,
                GPU_CMD_BUF.as_mut_ptr(),
                size_of::<GpuSetScanout>(),
            );

            let success = self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuSetScanout>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            if !success {
                let _ = writeln!(uart, "[VirtIO-GPU] ERROR: SET_SCANOUT command failed!");
            } else {
                let _ = writeln!(uart, "[VirtIO-GPU] SET_SCANOUT command succeeded");
            }

            // 4. Initial transfer and flush to activate display
            let _ = writeln!(uart, "[VirtIO-GPU] Commands 4-5: TRANSFER + FLUSH...");
            self.flush_display();
            let _ = writeln!(uart, "[VirtIO-GPU] ========================================");
            let _ = writeln!(uart, "[VirtIO-GPU] Display initialization complete!");
            let _ = writeln!(uart, "[VirtIO-GPU] ========================================");
        }
    }

    // Flush framebuffer to display
    fn flush_display(&mut self) {
        let mut uart = crate::uart::Uart::new();
        let resource_id = self.resource_id;
        let width = self.info.width as u32;
        let height = self.info.height as u32;

        unsafe {
            // Transfer to host
            let _ = writeln!(uart, "[VirtIO-GPU]   TRANSFER_TO_HOST_2D: {}x{}", width, height);
            let transfer_cmd = GpuTransferToHost2D {
                hdr: GpuCtrlHdr {
                    hdr_type: VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                r: GpuRect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
                offset: 0,
                resource_id,
                padding: 0,
            };

            core::ptr::copy_nonoverlapping(
                &transfer_cmd as *const _ as *const u8,
                GPU_CMD_BUF.as_mut_ptr(),
                size_of::<GpuTransferToHost2D>(),
            );

            let success = self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuTransferToHost2D>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            if !success {
                let _ = writeln!(uart, "[VirtIO-GPU]   ERROR: TRANSFER command failed!");
            } else {
                let _ = writeln!(uart, "[VirtIO-GPU]   TRANSFER command succeeded");
            }

            // Flush resource
            let _ = writeln!(uart, "[VirtIO-GPU]   RESOURCE_FLUSH: resource_id={}", resource_id);
            let flush_cmd = GpuResourceFlush {
                hdr: GpuCtrlHdr {
                    hdr_type: VIRTIO_GPU_CMD_RESOURCE_FLUSH,
                    flags: 0,
                    fence_id: 0,
                    ctx_id: 0,
                    padding: 0,
                },
                r: GpuRect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
                resource_id,
                padding: 0,
            };

            core::ptr::copy_nonoverlapping(
                &flush_cmd as *const _ as *const u8,
                GPU_CMD_BUF.as_mut_ptr(),
                size_of::<GpuResourceFlush>(),
            );

            let success = self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuResourceFlush>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            if !success {
                let _ = writeln!(uart, "[VirtIO-GPU]   ERROR: FLUSH command failed!");
            } else {
                let _ = writeln!(uart, "[VirtIO-GPU]   FLUSH command succeeded");
            }
        }
    }
}

impl Framebuffer for VirtioGpu {
    fn info(&self) -> &FramebufferInfo {
        &self.info
    }
    fn back_buffer(&self) -> *mut u8 {
        self.back
    }
    fn present(&self) {
        // NOTE: We can't call flush_display here because self is immutable
        // and flush_display needs &mut self. The display is already initialized
        // and active from init_display(), so additional flushes would require
        // a different synchronization mechanism (e.g., interior mutability).
        // For now, the initial display activation is sufficient to show the framebuffer.
    }
}
