use crate::display::{get_framebuffer, font};
use spin::Mutex;

/// Console state for text rendering
pub struct ConsoleState {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub width_chars: usize,
    pub height_chars: usize,
    pub fg_color: u32,  // Foreground color (XRGB8888)
    pub bg_color: u32,  // Background color (XRGB8888)
    pub cursor_visible: bool,  // Whether cursor is currently visible (for blinking)
    pub cursor_blink_counter: usize,  // Counter for cursor blinking
}

static CONSOLE_STATE: Mutex<Option<ConsoleState>> = Mutex::new(None);

pub fn init_fb_console() -> Result<(), ()> {
    if let Some(fb) = get_framebuffer() {
        let info = fb.info();
        
        // Calculate console dimensions in characters
        let width_chars = info.width / font::FONT_WIDTH;
        let height_chars = info.height / font::FONT_HEIGHT;
        
        // Initialize console state
        let state = ConsoleState {
            cursor_x: 0,
            cursor_y: 0,
            width_chars,
            height_chars,
            fg_color: 0x00FFFFFF,  // White text
            bg_color: 0x00000000,  // Black background
            cursor_visible: true,  // Start with visible cursor
            cursor_blink_counter: 0,
        };
        
        // Clear screen to background color
        clear_screen(fb, state.bg_color);
        
        // Store console state
        *CONSOLE_STATE.lock() = Some(state);
        
        // Draw initial cursor
        if let Some(ref state) = *CONSOLE_STATE.lock() {
            draw_cursor(fb, state);
        }
        
        // Flush framebuffer to display device (GPU)
        crate::display::flush_framebuffer();
        Ok(())
    } else { 
        Err(()) 
    }
}

/// Clear the entire screen to a specific color
fn clear_screen(fb: &dyn crate::display::Framebuffer, color: u32) {
    let info = fb.info();
    unsafe {
        let buf = fb.back_buffer();
        let pixels = core::slice::from_raw_parts_mut(buf as *mut u32, info.width * info.height);
        for p in pixels.iter_mut() { 
            *p = color; 
        }
    }
}

/// Internal function to write a single character without flushing
fn write_char_internal(c: u8) {
    let fb = match get_framebuffer() {
        Some(fb) => fb,
        None => return,
    };
    
    let mut state_guard = CONSOLE_STATE.lock();
    let state = match state_guard.as_mut() {
        Some(s) => s,
        None => return,
    };
    
    match c {
        b'\n' => {
            // Newline: move to start of next line
            state.cursor_x = 0;
            state.cursor_y += 1;
            if state.cursor_y >= state.height_chars {
                scroll_up(fb, state);
            }
        }
        b'\r' => {
            // Carriage return: move to start of line
            state.cursor_x = 0;
        }
        b'\t' => {
            // Tab: move to next tab stop (8 characters)
            let next_tab = ((state.cursor_x / 8) + 1) * 8;
            if next_tab < state.width_chars {
                state.cursor_x = next_tab;
            } else {
                state.cursor_x = 0;
                state.cursor_y += 1;
                if state.cursor_y >= state.height_chars {
                    scroll_up(fb, state);
                }
            }
        }
        b'\x08' => {
            // Backspace: move cursor back one position
            if state.cursor_x > 0 {
                state.cursor_x -= 1;
                // Optionally clear the character at cursor position
                draw_char(fb, state, b' ');
            }
        }
        c if c >= 32 && c <= 126 => {
            // Printable character
            draw_char(fb, state, c);
            state.cursor_x += 1;
            if state.cursor_x >= state.width_chars {
                state.cursor_x = 0;
                state.cursor_y += 1;
                if state.cursor_y >= state.height_chars {
                    scroll_up(fb, state);
                }
            }
        }
        _ => {
            // Ignore other control characters
        }
    }
}

/// Write a single character at the current cursor position
#[allow(dead_code)]
pub fn write_char(c: u8) {
    write_char_internal(c);
    // Update cursor after writing
    update_cursor();
    // Flush framebuffer to display device (GPU)
    crate::display::flush_framebuffer();
}

/// Write a string to the console
pub fn write_str(s: &str) {
    for byte in s.bytes() {
        write_char_internal(byte);
    }
    // Update cursor after writing
    update_cursor();
    // Flush once after writing all characters for better performance
    crate::display::flush_framebuffer();
}

/// Draw the cursor at the current position
fn draw_cursor(fb: &dyn crate::display::Framebuffer, state: &ConsoleState) {
    if !state.cursor_visible {
        return;
    }
    
    let info = fb.info();
    let x_pixel = state.cursor_x * font::FONT_WIDTH;
    let y_pixel = state.cursor_y * font::FONT_HEIGHT;
    
    // Draw cursor as a solid block at the bottom 3 pixels of the character cell
    unsafe {
        let buf = fb.back_buffer() as *mut u32;
        
        // Draw a 3-pixel high cursor bar at the bottom
        for row in (font::FONT_HEIGHT - 3)..font::FONT_HEIGHT {
            let y = y_pixel + row;
            if y >= info.height {
                break;
            }
            
            for col in 0..font::FONT_WIDTH {
                let x = x_pixel + col;
                if x >= info.width {
                    break;
                }
                
                let offset = y * info.width + x;
                *buf.add(offset) = state.fg_color;
            }
        }
    }
}

/// Update the cursor display (handle blinking)
pub fn update_cursor() {
    let fb = match get_framebuffer() {
        Some(fb) => fb,
        None => return,
    };
    
    let mut state_guard = CONSOLE_STATE.lock();
    let state = match state_guard.as_mut() {
        Some(s) => s,
        None => return,
    };
    
    // Increment blink counter
    state.cursor_blink_counter += 1;
    
    // Toggle cursor visibility every 30 calls (adjust for desired blink rate)
    if state.cursor_blink_counter >= 30 {
        state.cursor_visible = !state.cursor_visible;
        state.cursor_blink_counter = 0;
    }
    
    // Draw the cursor
    draw_cursor(fb, state);
}

/// Draw a character at the current cursor position
fn draw_char(fb: &dyn crate::display::Framebuffer, state: &ConsoleState, c: u8) {
    let bitmap = match font::get_char_bitmap(c) {
        Some(b) => b,
        None => return,  // Unsupported character
    };
    
    let info = fb.info();
    let x_pixel = state.cursor_x * font::FONT_WIDTH;
    let y_pixel = state.cursor_y * font::FONT_HEIGHT;
    
    unsafe {
        let buf = fb.back_buffer() as *mut u32;
        
        for row in 0..font::FONT_HEIGHT {
            let bitmap_row = bitmap[row];
            let y = y_pixel + row;
            if y >= info.height {
                break;
            }
            
            for col in 0..font::FONT_WIDTH {
                let x = x_pixel + col;
                if x >= info.width {
                    break;
                }
                
                // Check if pixel is set (bit position matches column)
                // This corrects the horizontally mirrored characters
                let pixel_set = (bitmap_row & (1 << col)) != 0;
                let color = if pixel_set {
                    state.fg_color
                } else {
                    state.bg_color
                };
                
                let offset = y * info.width + x;
                *buf.add(offset) = color;
            }
        }
    }
}

/// Scroll the screen up by one line
fn scroll_up(fb: &dyn crate::display::Framebuffer, state: &mut ConsoleState) {
    let info = fb.info();
    let line_height_pixels = font::FONT_HEIGHT;
    
    unsafe {
        let buf = fb.back_buffer() as *mut u32;
        
        // Copy lines up
        for y in line_height_pixels..info.height {
            for x in 0..info.width {
                let src_offset = y * info.width + x;
                let dst_offset = (y - line_height_pixels) * info.width + x;
                *buf.add(dst_offset) = *buf.add(src_offset);
            }
        }
        
        // Clear the bottom line
        let start_y = info.height - line_height_pixels;
        for y in start_y..info.height {
            for x in 0..info.width {
                let offset = y * info.width + x;
                *buf.add(offset) = state.bg_color;
            }
        }
    }
    
    // Move cursor up one line
    if state.cursor_y > 0 {
        state.cursor_y -= 1;
    }
}

