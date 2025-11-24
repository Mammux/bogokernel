use crate::display::DisplayMode;

// Global display mode configuration
// SAFETY: This is only accessed during single-threaded kernel initialization.
// In a multi-threaded environment, this should use AtomicU8 or similar.
static mut DISPLAY_MODE: DisplayMode = DisplayMode::Ansi;

/// Parse kernel command line arguments
/// In a full implementation, this would read from device tree /chosen/bootargs
/// For this scaffold, we support testing via cmdline string parameter
pub fn parse_cmdline(s: &str) {
    for param in s.split_whitespace() {
        if let Some(v) = param.strip_prefix("display=") {
            unsafe {
                DISPLAY_MODE = match v {
                    "gpu" => DisplayMode::Gpu,
                    _ => DisplayMode::Ansi,
                }
            }
        }
    }
}

pub fn display_mode() -> DisplayMode {
    unsafe { DISPLAY_MODE }
}

/// Set display mode directly (for testing)
#[allow(dead_code)]
pub fn set_display_mode(mode: DisplayMode) {
    unsafe { DISPLAY_MODE = mode; }
}
