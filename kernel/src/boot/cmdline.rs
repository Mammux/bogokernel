use crate::display::DisplayMode;

static mut DISPLAY_MODE: DisplayMode = DisplayMode::Ansi;

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
