pub mod fb_console;
pub mod font;
pub mod virtio_gpu;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Ansi,
    Gpu,
}

#[derive(Debug)]
pub struct FramebufferInfo {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub phys_addr: usize,
    pub size: usize,
}

pub trait Framebuffer {
    fn info(&self) -> &FramebufferInfo;
    fn back_buffer(&self) -> *mut u8; // unsafe pointer to back buffer
    #[allow(dead_code)]
    fn present(&self);
}

// Global framebuffer registration
// SAFETY: This is only accessed during single-threaded kernel initialization.
// In a multi-threaded environment, this should use proper synchronization (Mutex/RwLock).
static mut GLOBAL_FB: Option<&'static dyn Framebuffer> = None;

pub fn register_framebuffer(fb: &'static dyn Framebuffer) {
    unsafe {
        GLOBAL_FB = Some(fb);
    }
}

pub fn get_framebuffer() -> Option<&'static dyn Framebuffer> {
    unsafe { GLOBAL_FB }
}

pub fn flush_framebuffer() -> bool {
    virtio_gpu::flush_gpu()
}
