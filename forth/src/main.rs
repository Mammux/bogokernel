#![no_std]
#![no_main]

use usys::{print, println, IoRead, STDIN, exit};
use forth::Forth;

// Read a line from stdin
fn read_line(buf: &mut [u8]) -> usize {
    let mut len = 0;
    loop {
        let mut c = [0u8; 1];
        if let Ok(1) = STDIN.read(&mut c) {
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
    len
}

// Execute a word with I/O operations
fn execute_word_with_io(forth: &mut Forth, word: &str) -> Result<(), &'static str> {
    match word {
        // I/O operations that need usys
        "." => {
            let val = forth.pop()?;
            println!("{} ", val);
        }
        ".s" => {
            print!("<{}> ", forth.depth());
            for val in forth.stack_contents() {
                print!("{} ", val);
            }
            println!();
        }
        "cr" => {
            println!();
        }
        "emit" => {
            let val = forth.pop()?;
            if val >= 0 && val <= 127 {
                print!("{}", val as u8 as char);
            } else {
                return Err("Invalid character code");
            }
        }
        // For all other words, use the library implementation
        _ => {
            forth.execute_word(word)?;
        }
    }
    Ok(())
}

// Evaluate a line with I/O support
fn eval_with_io(forth: &mut Forth, line: &str) -> Result<(), &'static str> {
    let words = line.split_whitespace();
    for word in words {
        execute_word_with_io(forth, word)?;
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn _start(_argc: usize, _argv: *const *const u8, _envp: *const *const u8) -> ! {
    println!("BogoForth v0.1");
    println!("A simple Forth interpreter for BogoKernel");
    println!("Type 'words' for available words, or 'bye' to exit");
    println!();
    
    let mut forth = Forth::new();
    let mut input_buf = [0u8; 128];
    
    loop {
        print!("ok ");
        
        let len = read_line(&mut input_buf);
        
        if len == 0 {
            continue;
        }
        
        let input = core::str::from_utf8(&input_buf[..len]).unwrap_or("");
        
        // Check for special commands
        if input.trim() == "bye" || input.trim() == "quit" {
            println!("Goodbye!");
            exit();
        }
        
        if input.trim() == "words" {
            println!("Available words:");
            println!("  Arithmetic: + - * / mod");
            println!("  Stack:      dup drop swap over rot");
            println!("  I/O:        . .s cr emit");
            println!("  Comparison: = < >");
            println!("  Logical:    and or xor invert negate");
            println!("  Constants:  true false");
            println!("  Special:    words bye");
            continue;
        }
        
        // Evaluate the input
        match eval_with_io(&mut forth, input) {
            Ok(()) => {
                // Success - show "ok" on next iteration
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
