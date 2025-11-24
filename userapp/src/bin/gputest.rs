#![no_std]
#![no_main]

use usys::{exit, get_fb_info, println, FbInfo};

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8, _envp: *const *const u8) -> isize {
    println!("GPU Test Application");
    println!("====================");
    
    // Get framebuffer info
    let mut fb_info = FbInfo {
        width: 0,
        height: 0,
        stride: 0,
        addr: 0,
    };
    
    match get_fb_info(&mut fb_info) {
        Ok(()) => {
            println!("Framebuffer info:");
            println!("  Width:  {} pixels", fb_info.width);
            println!("  Height: {} pixels", fb_info.height);
            println!("  Stride: {} bytes", fb_info.stride);
            println!("  Address: 0x{:x}", fb_info.addr);
            
            if fb_info.addr == 0 {
                println!("Error: Invalid framebuffer address");
                exit();
            }
            
            // Access the framebuffer
            let fb_ptr = fb_info.addr as *mut u32;
            // Calculate pixel count using stride (stride is in bytes, divide by 4 for u32)
            let pixels_per_row = fb_info.stride / 4;
            let pixel_count = pixels_per_row * fb_info.height;
            
            println!("\nDrawing test pattern...");
            
            unsafe {
                let fb_slice = core::slice::from_raw_parts_mut(fb_ptr, pixel_count);
                
                // Draw colored bars (XRGB8888 format)
                let bar_height = fb_info.height / 8;
                let colors = [
                    0x00FF0000, // Red
                    0x0000FF00, // Green
                    0x000000FF, // Blue
                    0x00FFFF00, // Yellow
                    0x00FF00FF, // Magenta
                    0x0000FFFF, // Cyan
                    0x00FFFFFF, // White
                    0x00000000, // Black
                ];
                
                for y in 0..fb_info.height {
                    let color_idx = (y / bar_height).min(7);
                    let color = colors[color_idx];
                    
                    for x in 0..fb_info.width {
                        // Use stride-aware indexing
                        let idx = y * pixels_per_row + x;
                        fb_slice[idx] = color;
                    }
                }
                
                // Draw a white square in the center
                let square_size = 100;
                let start_x = (fb_info.width - square_size) / 2;
                let start_y = (fb_info.height - square_size) / 2;
                let end_x = start_x + square_size;
                let end_y = start_y + square_size;
                
                for y in start_y..end_y {
                    // Bounds check
                    if y >= fb_info.height {
                        break;
                    }
                    for x in start_x..end_x {
                        // Bounds check
                        if x >= fb_info.width {
                            break;
                        }
                        let idx = y * pixels_per_row + x;
                        fb_slice[idx] = 0x00FFFFFF; // White
                    }
                }
            }
            
            println!("Test pattern drawn successfully!");
            println!("You should see 8 colored horizontal bars with a white square in the center.");
        }
        Err(_) => {
            println!("Error: Failed to get framebuffer info");
            println!("Make sure the kernel was built with GPU support.");
        }
    }
    
    exit();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("panic at {}:{}:{}", location.file(), location.line(), location.column());
    }
    println!("{}", info.message());
    exit();
}
