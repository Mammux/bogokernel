use crate::boot::cmdline;
use crate::display::{virtio_gpu, fb_console};
use core::fmt::Write;

pub fn init_console() {
    let mut uart = crate::uart::Uart::new();
    
    match cmdline::display_mode() {
        crate::display::DisplayMode::Gpu => {
            let _ = writeln!(uart, "Attempting to initialize GPU display...");
            if let Some(_vg) = virtio_gpu::VirtioGpu::probe() {
                match fb_console::init_fb_console() {
                    Ok(()) => {
                        let _ = writeln!(uart, "GPU framebuffer console initialized");
                    }
                    Err(()) => {
                        let _ = writeln!(uart, "Failed to initialize framebuffer console, falling back to UART");
                    }
                }
            } else {
                let _ = writeln!(uart, "virtio-gpu not found, falling back to UART console");
            }
        }
        crate::display::DisplayMode::Ansi => {
            let _ = writeln!(uart, "Using UART (ANSI) console");
        }
    }
}
