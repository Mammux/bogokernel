#![no_std]
#![no_main]

use usys::{print, println, debug, IoRead};

// Maximum number of command-line arguments
const MAX_ARGS: usize = 16;

#[no_mangle]
pub extern "C" fn _start(_argc: usize, _argv: *const *const u8, _envp: *const *const u8) -> ! {
    main();
    usys::exit();
}

fn main() {
    // Debug output goes to serial port, console output goes to framebuffer (when GPU enabled)
    debug!("BogoShell starting up");
    
    println!("Welcome to BogoShell!");
    println!("Type 'help' for available commands, 'ls' to list programs");

    let mut buf = [0u8; 64];
    loop {
        print!("> ");
        
        // Simple line reader
        let mut len = 0;
        loop {
            let mut c = [0u8; 1];
            if let Ok(1) = usys::STDIN.read(&mut c) {
                let ch = c[0];
                if ch == b'\r' || ch == b'\n' {
                    println!();
                    break;
                } else if ch == 8 || ch == 127 { // Backspace
                    if len > 0 {
                        len -= 1;
                        print!("\x08 \x08"); // Erase character
                    }
                } else if len < buf.len() - 1 {
                    buf[len] = ch;
                    len += 1;
                    print!("{}", ch as char);
                }
            }
        }
        
        if len == 0 { continue; }
        
        let input = core::str::from_utf8(&buf[..len]).unwrap_or("");
        
        // Parse command line: split by whitespace
        let mut tokens: [&str; MAX_ARGS] = [""; MAX_ARGS];
        let mut token_count = 0;
        for token in input.split_whitespace() {
            if token_count < MAX_ARGS {
                tokens[token_count] = token;
                token_count += 1;
            }
        }
        
        if token_count == 0 { continue; }
        
        let cmd = tokens[0];
        
        // Check for built-in commands first
        match cmd {
            "help" => {
                println!("Built-in commands: ls, help, shutdown");
                println!("To run a program, type its name without the .elf extension");
                println!("Example: hello, rogue, crogue, bigrogue, curses_test, fstest, mkfiles, lisp");
            },
            "ls" => {
                // List files in writable filesystem
                let mut buf = [0u8; 4096];
                match usys::readdir(&mut buf) {
                    Ok(count) => {
                        if count == 0 {
                            println!("No files in writable filesystem");
                        } else {
                            println!("Files in writable filesystem:");
                            let mut offset = 0;
                            for _ in 0..count {
                                // Find the null terminator, but cap search to buffer size
                                let mut end = offset;
                                while end < buf.len() && buf[end] != 0 {
                                    end += 1;
                                }
                                
                                // If we found a valid filename, print it
                                if end > offset && end < buf.len() {
                                    if let Ok(filename) = core::str::from_utf8(&buf[offset..end]) {
                                        println!("  {}", filename);
                                    }
                                }
                                
                                // Move to next filename (past the null terminator)
                                offset = end + 1;
                                
                                // Safety check: if we've gone past the buffer, stop
                                if offset >= buf.len() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(_) => println!("Error listing files"),
                }
            },
            "shutdown" => {
                println!("Shutting down...");
                usys::poweroff();
            },
            _ => {
                // Try to execute as a program from the filesystem
                // Append .elf if not already present
                let mut filename_buf = [0u8; 64];
                let filename = if cmd.ends_with(".elf") {
                    cmd
                } else {
                    // Build filename with .elf extension
                    let mut len = 0;
                    for (i, &b) in cmd.as_bytes().iter().enumerate() {
                        if i >= filename_buf.len() - 5 {
                            break;
                        }
                        filename_buf[i] = b;
                        len = i + 1;
                    }
                    // Add .elf extension
                    if len + 4 < filename_buf.len() {
                        filename_buf[len] = b'.';
                        filename_buf[len + 1] = b'e';
                        filename_buf[len + 2] = b'l';
                        filename_buf[len + 3] = b'f';
                        len += 4;
                    }
                    core::str::from_utf8(&filename_buf[..len]).unwrap_or(cmd)
                };
                
                // Check if file exists before trying to execute
                if let Ok(filename_cstr) = usys::CStrBuf::<64>::from_str(filename) {
                    let mut stat_buf = [0u64; 2];
                    match usys::stat(filename_cstr.as_cstr(), &mut stat_buf) {
                        Ok(_) => {
                            // File exists, proceed with execution
                            debug!("Executing: {}", filename);
                            
                            // Build argv array with command line arguments
                            let mut argv_cstrs: [usys::CStrBuf<64>; MAX_ARGS] = Default::default();
                            
                            // First arg is program name
                            argv_cstrs[0] = filename_cstr;
                            let mut argv_count = 1;
                            
                            // Add remaining arguments
                            for i in 1..token_count {
                                if argv_count >= MAX_ARGS {
                                    break;
                                }
                                if let Ok(cstr) = usys::CStrBuf::<64>::from_str(tokens[i]) {
                                    argv_cstrs[argv_count] = cstr;
                                    argv_count += 1;
                                }
                            }
                            
                            // Build references array
                            let mut argv_refs: [&core::ffi::CStr; MAX_ARGS] = [usys::cstr!(""); MAX_ARGS];
                            for i in 0..argv_count {
                                argv_refs[i] = argv_cstrs[i].as_cstr();
                            }
                            
                            // Execute the program
                            usys::execv(argv_cstrs[0].as_cstr(), &argv_refs[..argv_count]);
                        }
                        Err(_) => {
                            println!("Command not found: {}", filename);
                            println!("Type 'help' for available commands or 'ls' to see programs");
                        }
                    }
                } else {
                    println!("Error: Invalid filename '{}'", filename);
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
