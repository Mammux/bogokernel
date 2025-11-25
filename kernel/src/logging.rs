//! Kernel logging system with configurable log levels.
//!
//! This module provides log level filtering and output routing for the kernel.
//! When the `gpu` feature is enabled, console output (stdout) goes to the framebuffer,
//! while debugging output (stderr) always goes to the serial port.
//!
//! Log levels are similar to log4j:
//! - TRACE: Fine-grained debugging information
//! - DEBUG: Debugging information
//! - INFO: Informational messages
//! - WARN: Warning messages
//! - ERROR: Error messages

#![allow(dead_code)]

use core::sync::atomic::{AtomicU8, Ordering};
use uapi::LogLevel;

/// Global log level filter. Messages below this level are suppressed.
static LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

/// Get the current log level threshold.
pub fn get_log_level() -> LogLevel {
    match LOG_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Trace,
        1 => LogLevel::Debug,
        2 => LogLevel::Info,
        3 => LogLevel::Warn,
        _ => LogLevel::Error,
    }
}

/// Set the log level threshold. Messages below this level will be suppressed.
pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Check if a message at the given level should be logged.
#[inline]
pub fn should_log(level: LogLevel) -> bool {
    level as u8 >= LOG_LEVEL.load(Ordering::Relaxed)
}

/// Write a debug/log message to the serial port (UART).
/// This always goes to serial, regardless of GPU mode.
pub fn debug_write(s: &str) {
    use core::fmt::Write;
    let mut uart = crate::uart::Uart::new();
    let _ = uart.write_str(s);
}

/// Write a debug/log message with newline to the serial port.
pub fn debug_writeln(s: &str) {
    use core::fmt::Write;
    let mut uart = crate::uart::Uart::new();
    let _ = uart.write_str(s);
    let _ = uart.write_str("\r\n");
}

/// Internal macro for kernel logging with level filtering.
#[macro_export]
macro_rules! klog {
    ($level:expr, $($arg:tt)*) => {{
        if $crate::logging::should_log($level) {
            use core::fmt::Write;
            let mut uart = $crate::uart::Uart::new();
            let level_str = match $level {
                uapi::LogLevel::Trace => "[TRACE] ",
                uapi::LogLevel::Debug => "[DEBUG] ",
                uapi::LogLevel::Info =>  "[INFO]  ",
                uapi::LogLevel::Warn =>  "[WARN]  ",
                uapi::LogLevel::Error => "[ERROR] ",
            };
            let _ = uart.write_str(level_str);
            let _ = write!(uart, $($arg)*);
            let _ = uart.write_str("\r\n");
        }
    }};
}

/// Log a trace-level message (finest granularity).
#[macro_export]
macro_rules! ktrace {
    ($($arg:tt)*) => {
        $crate::klog!(uapi::LogLevel::Trace, $($arg)*)
    };
}

/// Log a debug-level message.
#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => {
        $crate::klog!(uapi::LogLevel::Debug, $($arg)*)
    };
}

/// Log an info-level message.
#[macro_export]
macro_rules! kinfo {
    ($($arg:tt)*) => {
        $crate::klog!(uapi::LogLevel::Info, $($arg)*)
    };
}

/// Log a warning-level message.
#[macro_export]
macro_rules! kwarn {
    ($($arg:tt)*) => {
        $crate::klog!(uapi::LogLevel::Warn, $($arg)*)
    };
}

/// Log an error-level message.
#[macro_export]
macro_rules! kerror {
    ($($arg:tt)*) => {
        $crate::klog!(uapi::LogLevel::Error, $($arg)*)
    };
}
