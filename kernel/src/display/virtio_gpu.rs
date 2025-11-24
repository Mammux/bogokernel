use crate::display::{Framebuffer, FramebufferInfo, register_framebuffer};

pub struct VirtioGpu {
    info: FramebufferInfo,
    back: *mut u8,
}

impl VirtioGpu {
    pub fn probe() -> Option<&'static Self> {
        // Minimal probe: detect virtio-gpu device on virtio bus.
        // TODO: perform real virtio negotiation here and allocate guest buffer.
        // For scaffold: allocate a statically sized framebuffer in kernel memory (guest RAM) and
        // pretend it's a virtio-gpu resource. Replace with real virtio ops later.
        const W: usize = 1024;
        const H: usize = 768;
        const SIZE: usize = W * H * 4;
        // SAFETY: use a static boxed slice to simulate guest RAM backing
        static mut BUF: [u8; 1024*768*4] = [0; 1024*768*4];
        let fb_info = FramebufferInfo { width: W, height: H, stride: W * 4, phys_addr: 0, size: SIZE };
        static mut VG: Option<VirtioGpu> = None;
        unsafe {
            VG = Some(VirtioGpu { info: fb_info, back: BUF.as_mut_ptr() });
            if let Some(v) = &VG { register_framebuffer(v); return Some(v); }
        }
        None
    }
}

impl Framebuffer for VirtioGpu {
    fn info(&self) -> &FramebufferInfo { &self.info }
    fn back_buffer(&self) -> *mut u8 { self.back }
    fn present(&self) {
        // TODO: in a real driver, submit a resource flush/present to virtio-gpu.
        // For QEMU testing with a fake guest buffer we may need to hook into QEMU's memory or
        // use virtio properly; left as a TODO in this scaffold.
    }
}
