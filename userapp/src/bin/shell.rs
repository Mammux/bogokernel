#![no_std]
#![no_main]

use usys::{print, println, IoRead};

#[no_mangle]
pub extern "C" fn _start(_argc: usize, _argv: *const *const u8, _envp: *const *const u8) -> ! {
    main();
    usys::exit();
}

fn main() {
    println!("Welcome to BogoShell!");
    println!("Commands: hello, rogue, crogue, bigrogue, curses_test, fstest, mkfiles, ls, shutdown, help");

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
        let mut tokens: [&str; 16] = [""; 16];
        let mut token_count = 0;
        for token in input.split_whitespace() {
            if token_count < 16 {
                tokens[token_count] = token;
                token_count += 1;
            }
        }
        
        if token_count == 0 { continue; }
        
        let cmd = tokens[0];
        
        match cmd {
            "help" => println!("Available commands: hello, rogue, crogue, bigrogue, curses_test, fstest, mkfiles, ls, shutdown"),
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
            "hello" => {
                println!("Executing hello...");
                // Build argv array with command line arguments
                let mut argv_cstrs: [usys::CStrBuf<64>; 16] = Default::default();
                
                // First arg is program name
                argv_cstrs[0] = usys::CStrBuf::from_str("hello.elf").unwrap();
                let mut argv_count = 1;
                
                // Add remaining arguments
                for i in 1..token_count {
                    if let Ok(cstr) = usys::CStrBuf::from_str(tokens[i]) {
                        argv_cstrs[argv_count] = cstr;
                        argv_count += 1;
                    }
                }
                
                // Build references array
                let mut argv_refs: [&core::ffi::CStr; 16] = [usys::cstr!(""); 16];
                for i in 0..argv_count {
                    argv_refs[i] = argv_cstrs[i].as_cstr();
                }
                
                usys::execv(usys::cstr!("hello.elf"), &argv_refs[..argv_count]);
            },
            "rogue" => {
                println!("Executing rogue...");
                let argv_cstrs: [usys::CStrBuf<64>; 1] = [usys::CStrBuf::from_str("rogue.elf").unwrap()];
                let argv_refs = [argv_cstrs[0].as_cstr()];
                usys::execv(usys::cstr!("rogue.elf"), &argv_refs);
            },
            "crogue" => {
                println!("Executing crogue...");
                let argv_cstrs: [usys::CStrBuf<64>; 1] = [usys::CStrBuf::from_str("crogue.elf").unwrap()];
                let argv_refs = [argv_cstrs[0].as_cstr()];
                usys::execv(usys::cstr!("crogue.elf"), &argv_refs);
            },
            "bigrogue" => {
                println!("Executing bigrogue...");
                let argv_cstrs: [usys::CStrBuf<64>; 1] = [usys::CStrBuf::from_str("bigrogue.elf").unwrap()];
                let argv_refs = [argv_cstrs[0].as_cstr()];
                usys::execv(usys::cstr!("bigrogue.elf"), &argv_refs);
            },
            "curses_test" => {
                println!("Executing curses_test...");
                let argv_cstrs: [usys::CStrBuf<64>; 1] = [usys::CStrBuf::from_str("curses_test.elf").unwrap()];
                let argv_refs = [argv_cstrs[0].as_cstr()];
                usys::execv(usys::cstr!("curses_test.elf"), &argv_refs);
            },
            "fstest" => {
                println!("Executing fstest...");
                let argv_cstrs: [usys::CStrBuf<64>; 1] = [usys::CStrBuf::from_str("fstest.elf").unwrap()];
                let argv_refs = [argv_cstrs[0].as_cstr()];
                usys::execv(usys::cstr!("fstest.elf"), &argv_refs);
            },            
            "mkfiles" => {
                println!("Executing mkfiles...");
                let argv_cstrs: [usys::CStrBuf<64>; 1] = [usys::CStrBuf::from_str("mkfiles.elf").unwrap()];
                let argv_refs = [argv_cstrs[0].as_cstr()];
                usys::execv(usys::cstr!("mkfiles.elf"), &argv_refs);
            },
            _ => println!("Unknown command: {}", cmd),
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
