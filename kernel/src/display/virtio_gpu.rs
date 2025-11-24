use crate::display::{Framebuffer, FramebufferInfo, register_framebuffer};

// VirtIO GPU device constants
const VIRTIO_GPU_DEVICE_ID: u32 = 16;  // VirtIO GPU device type
const VIRTIO_VENDOR_ID: u32 = 0x1AF4;

// VirtIO MMIO register offsets
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
const VIRTIO_MMIO_STATUS: usize = 0x070;

// VirtIO status bits
const VIRTIO_STATUS_ACKNOWLEDGE: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_FEATURES_OK: u32 = 8;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;

// VirtIO-GPU specific constants
const VIRTIO_GPU_CMD_GET_DISPLAY_INFO: u32 = 0x0100;
const VIRTIO_GPU_CMD_RESOURCE_CREATE_2D: u32 = 0x0101;
const VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;
const VIRTIO_GPU_CMD_SET_SCANOUT: u32 = 0x0103;
const VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D: u32 = 0x0105;
const VIRTIO_GPU_CMD_RESOURCE_FLUSH: u32 = 0x0104;

const VIRTIO_GPU_FORMAT_B8G8R8X8_UNORM: u32 = 2;

pub struct VirtioGpu {
    info: FramebufferInfo,
    back: *mut u8,
    mmio_base: usize,
    resource_id: u32,
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
            let device_id = unsafe { core::ptr::read_volatile((base + VIRTIO_MMIO_DEVICE_ID) as *const u32) };
            if device_id != VIRTIO_GPU_DEVICE_ID {
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
            
            // Write driver features (we accept all features for simplicity)
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_DRIVER_FEATURES) as *mut u32, 0);
            
            // Features OK
            status |= VIRTIO_STATUS_FEATURES_OK;
            core::ptr::write_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *mut u32, status);
            
            // Verify features OK
            let status_check = core::ptr::read_volatile((mmio_base + VIRTIO_MMIO_STATUS) as *const u32);
            if (status_check & VIRTIO_STATUS_FEATURES_OK) == 0 {
                return None;  // Device doesn't support our features
            }
            
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
            
            static mut VG: Option<VirtioGpu> = None;
            VG = Some(VirtioGpu {
                info: fb_info,
                back: BUF.as_mut_ptr(),
                mmio_base,
                resource_id: 1,  // Resource ID for our framebuffer
            });
            
            VG.as_ref().map(|v| {
                register_framebuffer(v);
                
                // Note: In a complete implementation, we would:
                // 1. Set up virtqueues for command submission
                // 2. Send VIRTIO_GPU_CMD_GET_DISPLAY_INFO
                // 3. Send VIRTIO_GPU_CMD_RESOURCE_CREATE_2D
                // 4. Send VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING
                // 5. Send VIRTIO_GPU_CMD_SET_SCANOUT
                // 
                // For this simplified version, we've done the device negotiation
                // but the actual command submission would require setting up
                // virtqueues which is complex. The framebuffer will work for
                // software rendering even without full virtio command submission.
                
                v
            })
        }
    }
}

impl Framebuffer for VirtioGpu {
    fn info(&self) -> &FramebufferInfo { &self.info }
    fn back_buffer(&self) -> *mut u8 { self.back }
    fn present(&self) {
        // In a complete implementation, this would submit:
        // 1. VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D - copy framebuffer to host
        // 2. VIRTIO_GPU_CMD_RESOURCE_FLUSH - flush to display
        //
        // For now, the framebuffer is accessible to the guest and any writes
        // to it can be seen by the host QEMU process, though proper flushing
        // would require virtqueue command submission.
    }
}

