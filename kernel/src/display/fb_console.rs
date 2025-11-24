use crate::display::{get_framebuffer};

pub fn init_fb_console() -> Result<(), ()> {
    if let Some(fb) = get_framebuffer() {
        // Hook the existing TTY/console backend to render into the framebuffer.
        // This function should register callbacks so the kernel's TTY layer uses
        // the same text handling but sends pixels to fb.back_buffer(). For the scaffold
        // we'll set a simple renderer that fills the framebuffer with a test color.
        let info = fb.info();
        unsafe {
            let buf = fb.back_buffer();
            let pixels = core::slice::from_raw_parts_mut(buf as *mut u32, info.width * info.height);
            for p in pixels.iter_mut() { *p = 0xff00_0000u32; }
        }
        fb.present();
        Ok(())
    } else { Err(()) }
}
