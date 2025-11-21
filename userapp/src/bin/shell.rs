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
    println!("Commands: hello, rogue, shutdown, help");

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
        
        let cmd = core::str::from_utf8(&buf[..len]).unwrap_or("");
        
        match cmd {
            "help" => println!("Available commands: hello, rogue, shutdown"),
            "shutdown" => {
                println!("Shutting down...");
                usys::poweroff();
            },
            "hello" => {
                println!("Executing hello...");
                usys::exec(usys::cstr!("hello.elf"));
            },
            "rogue" => {
                println!("Executing rogue...");
                usys::exec(usys::cstr!("rogue.elf"));
            },
            _ => println!("Unknown command: {}", cmd),
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
