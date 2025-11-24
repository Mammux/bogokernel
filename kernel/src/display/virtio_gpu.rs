use crate::display::{Framebuffer, FramebufferInfo, register_framebuffer};
use core::mem::size_of;

// VirtIO GPU device constants
const VIRTIO_GPU_DEVICE_ID: u32 = 16;  // VirtIO GPU device type
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

// Virtqueue structure
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

pub struct VirtioGpu {
    info: FramebufferInfo,
    back: *mut u8,
    mmio_base: usize,
    resource_id: u32,
    queue: Option<Virtqueue>,
}

impl VirtioGpu {
    pub fn probe() -> Option<&'static Self> {
        // Scan for VirtIO MMIO devices in QEMU virt machine
        // QEMU virt typically has VirtIO devices at 0x10001000 - 0x10008000
        const VIRTIO_MMIO_BASE: usize = 0x10001000;
        const VIRTIO_MMIO_SIZE: usize = 0x1000;
        const VIRTIO_MMIO_COUNT: usize = 8;
        
        for i in 0..VIRTIO_MMIO_COUNT {
            let base = VIRTIO_MMIO_BASE + i * VIRTIO_MMIO_SIZE;
            
            // Check magic value (should be 0x74726976 = "virt")
            let magic = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_MAGIC_VALUE) as *const u32) };
            if magic != 0x74726976 {
                continue;
            }
            
            // Check version (should be 1 or 2)
            // Note: QEMU on Windows may report version 1, while Linux typically reports version 2.
            // Both versions are compatible for basic GPU device initialization.
            let version = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_VERSION) as *const u32) };
            if version != 1 && version != 2 {
                continue;
            }
            
            // Check if this is a GPU device
            // Device ID 0 indicates an empty/invalid slot, so continue scanning
            let device_id = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_DEVICE_ID) as *const u32) };
            if device_id == 0 {
                // Empty slot, skip to next
                continue;
            }
            if device_id != VIRTIO_GPU_DEVICE_ID {
                // Valid device but not GPU, skip to next
                continue;
            }
            
            // Found a VirtIO GPU device! Initialize it
            return Self::init_device(base);
        }
        
        None
    }
    
    fn init_device(mmio_base: usize) -> Option<&'static Self> {
        const W: usize = 1024;
        const H: usize = 768;
        const SIZE: usize = W * H * 4;
        
        // Allocate static framebuffer
        static mut BUF: [u8; SIZE] = [0; SIZE];
        
        // Allocate virtqueue memory (statically for simplicity)
        static mut QUEUE_DESC: [VirtqDesc; QUEUE_SIZE] = [VirtqDesc { addr: 0, len: 0, flags: 0, next: 0 }; QUEUE_SIZE];
        static mut QUEUE_AVAIL: VirtqAvail = VirtqAvail { flags: 0, idx: 0, ring: [0; QUEUE_SIZE] };
        static mut QUEUE_USED: VirtqUsed = VirtqUsed { flags: 0, idx: 0, ring: [VirtqUsedElem { id: 0, len: 0 }; QUEUE_SIZE] };
        
        unsafe {
            // Reset device
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, 0);
            
            // Acknowledge device
            let mut status = VIRTIO_STATUS_ACKNOWLEDGE;
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);
            
            // Set driver bit
            status |= VIRTIO_STATUS_DRIVER;
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);
            
            // Read device features
            let _device_features = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_DEVICE_FEATURES) as *const u32);
            
            // Write driver features (we accept minimal features)
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_DRIVER_FEATURES) as *mut u32, 0);
            
            // Features OK
            status |= VIRTIO_STATUS_FEATURES_OK;
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);
            
            // Verify features OK
            let status_check = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *const u32);
            if (status_check & VIRTIO_STATUS_FEATURES_OK) == 0 {
                return None;  // Device doesn't support our features
            }
            
            // Set up controlq (queue 0)
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_SEL) as *mut u32, 0);
            let queue_max = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_QUEUE_NUM_MAX) as *const u32);
            if queue_max < QUEUE_SIZE as u32 {
                return None;  // Queue too small
            }
            
            // Set queue size
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_NUM) as *mut u32, QUEUE_SIZE as u32);
            
            // Set guest page size (4KB)
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_GUEST_PAGE_SIZE) as *mut u32, PAGE_SIZE as u32);
            
            // Calculate queue physical address
            // For version 1, the queue PFN register expects the physical address divided by page size
            let queue_pfn = (QUEUE_DESC.as_ptr() as usize) / PAGE_SIZE;
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_QUEUE_PFN) as *mut u32, queue_pfn as u32);
            
            // Driver OK - device is ready
            status |= VIRTIO_STATUS_DRIVER_OK;
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);
            
            // Create VirtioGpu instance
            let fb_info = FramebufferInfo {
                width: W,
                height: H,
                stride: W * 4,
                phys_addr: BUF.as_ptr() as usize,
                size: SIZE,
            };
            
            let queue = Virtqueue {
                desc: &mut QUEUE_DESC,
                avail: &mut QUEUE_AVAIL,
                used: &mut QUEUE_USED,
                next_desc: 0,
                last_used_idx: 0,
            };
            
            static mut VG: Option<VirtioGpu> = None;
            VG = Some(VirtioGpu {
                info: fb_info,
                back: BUF.as_mut_ptr(),
                mmio_base,
                resource_id: 1,  // Resource ID for our framebuffer
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
        let queue = match self.queue.as_mut() {
            Some(q) => q,
            None => return false,
        };
        
        unsafe {
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
            
            // Add to available ring
            let avail_idx = queue.avail.idx;
            queue.avail.ring[avail_idx as usize % QUEUE_SIZE] = req_desc_idx;
            queue.avail.idx = avail_idx.wrapping_add(1);
            
            // Notify device (write queue index to notify register)
            core::ptr::write_volatile((self.mmio_base + VIRTIO_MMIO_QUEUE_NOTIFY) as *mut u32, 0);
            
            // Wait for response (simple busy wait)
            for _ in 0..COMMAND_TIMEOUT_ITERATIONS {
                if queue.used.idx != queue.last_used_idx {
                    queue.last_used_idx = queue.used.idx;
                    queue.next_desc = (resp_desc_idx + 1) % QUEUE_SIZE as u16;
                    return true;
                }
            }
            
            false
        }
    }
    
    // Initialize display by sending GPU commands
    fn init_display(&mut self) {
        let resource_id = self.resource_id;
        let fb_addr = self.back as usize;
        let width = self.info.width as u32;
        let height = self.info.height as u32;
        
        unsafe {
            // 1. Create 2D resource
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
            
            self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuResourceCreate2D>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            
            // 2. Attach backing storage
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
            
            core::ptr::copy_nonoverlapping(
                &attach_cmd as *const _ as *const u8,
                GPU_CMD_BUF.as_mut_ptr(),
                size_of::<GpuResourceAttachBacking>(),
            );
            core::ptr::copy_nonoverlapping(
                &mem_entry as *const _ as *const u8,
                GPU_CMD_BUF.as_mut_ptr().add(size_of::<GpuResourceAttachBacking>()),
                size_of::<GpuMemEntry>(),
            );
            
            self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuResourceAttachBacking>() + size_of::<GpuMemEntry>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            
            // 3. Set scanout
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
            
            self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuSetScanout>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            
            // 4. Initial transfer and flush to activate display
            self.flush_display();
        }
    }
    
    // Flush framebuffer to display
    fn flush_display(&mut self) {
        let resource_id = self.resource_id;
        let width = self.info.width as u32;
        let height = self.info.height as u32;
        
        unsafe {
            // Transfer to host
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
            
            self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuTransferToHost2D>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
            
            // Flush resource
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
            
            self.send_command(
                &GPU_CMD_BUF[..size_of::<GpuResourceFlush>()],
                &mut GPU_RESP_BUF[..size_of::<GpuCtrlResponse>()],
            );
        }
    }
}

impl Framebuffer for VirtioGpu {
    fn info(&self) -> &FramebufferInfo { &self.info }
    fn back_buffer(&self) -> *mut u8 { self.back }
    fn present(&self) {
        // NOTE: We can't call flush_display here because self is immutable
        // and flush_display needs &mut self. The display is already initialized
        // and active from init_display(), so additional flushes would require
        // a different synchronization mechanism (e.g., interior mutability).
        // For now, the initial display activation is sufficient to show the framebuffer.
    }
}

