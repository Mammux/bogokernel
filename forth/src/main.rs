#![no_std]
#![no_main]

use usys::{print, println, IoRead, STDIN, exit};

// Maximum stack depth
const STACK_SIZE: usize = 64;

// Forth interpreter state
struct Forth {
    stack: [i32; STACK_SIZE],
    sp: usize, // stack pointer (points to next free slot)
}

impl Forth {
    fn new() -> Self {
        Forth {
            stack: [0; STACK_SIZE],
            sp: 0,
        }
    }

    fn push(&mut self, val: i32) -> Result<(), &'static str> {
        if self.sp >= STACK_SIZE {
            Err("Stack overflow")
        } else {
            self.stack[self.sp] = val;
            self.sp += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<i32, &'static str> {
        if self.sp == 0 {
            Err("Stack underflow")
        } else {
            self.sp -= 1;
            Ok(self.stack[self.sp])
        }
    }

    fn peek(&self) -> Result<i32, &'static str> {
        if self.sp == 0 {
            Err("Stack empty")
        } else {
            Ok(self.stack[self.sp - 1])
        }
    }

    // Execute a single word
    fn execute_word(&mut self, word: &str) -> Result<(), &'static str> {
        match word {
            // Arithmetic operations (using checked arithmetic to prevent overflow)
            "+" => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a.checked_add(b).ok_or("Arithmetic overflow")?;
                self.push(result)?;
            }
            "-" => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a.checked_sub(b).ok_or("Arithmetic overflow")?;
                self.push(result)?;
            }
            "*" => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = a.checked_mul(b).ok_or("Arithmetic overflow")?;
                self.push(result)?;
            }
            "/" => {
                let b = self.pop()?;
                if b == 0 {
                    return Err("Division by zero");
                }
                let a = self.pop()?;
                // Handle special case: i32::MIN / -1 causes overflow
                let result = a.checked_div(b).ok_or("Arithmetic overflow")?;
                self.push(result)?;
            }
            "mod" => {
                let b = self.pop()?;
                if b == 0 {
                    return Err("Division by zero");
                }
                let a = self.pop()?;
                // Handle special case: i32::MIN % -1 causes overflow
                let result = a.checked_rem(b).ok_or("Arithmetic overflow")?;
                self.push(result)?;
            }
            
            // Stack manipulation
            "dup" => {
                let a = self.peek()?;
                self.push(a)?;
            }
            "drop" => {
                self.pop()?;
            }
            "swap" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(b)?;
                self.push(a)?;
            }
            "over" => {
                if self.sp < 2 {
                    return Err("Stack underflow");
                }
                let val = self.stack[self.sp - 2];
                self.push(val)?;
            }
            "rot" => {
                // ( a b c -- b c a )
                if self.sp < 3 {
                    return Err("Stack underflow");
                }
                let c = self.pop()?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(b)?;
                self.push(c)?;
                self.push(a)?;
            }
            
            // I/O operations
            "." => {
                let val = self.pop()?;
                println!("{} ", val);
            }
            ".s" => {
                print!("<{}> ", self.sp);
                for i in 0..self.sp {
                    print!("{} ", self.stack[i]);
                }
                println!();
            }
            "cr" => {
                println!();
            }
            "emit" => {
                let val = self.pop()?;
                if val >= 0 && val <= 127 {
                    print!("{}", val as u8 as char);
                } else {
                    return Err("Invalid character code");
                }
            }
            
            // Comparison operations
            "=" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(if a == b { -1 } else { 0 })?;
            }
            "<" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(if a < b { -1 } else { 0 })?;
            }
            ">" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(if a > b { -1 } else { 0 })?;
            }
            
            // Logical operations
            "and" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a & b)?;
            }
            "or" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a | b)?;
            }
            "xor" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a ^ b)?;
            }
            "invert" => {
                let a = self.pop()?;
                self.push(!a)?;
            }
            "negate" => {
                let a = self.pop()?;
                self.push(-a)?;
            }
            
            // Constants
            "true" => {
                self.push(-1)?;
            }
            "false" => {
                self.push(0)?;
            }
            
            "" => {
                // Empty word, do nothing
            }
            
            _ => {
                // Try to parse as a number
                if let Some(num) = parse_number(word) {
                    self.push(num)?;
                } else {
                    return Err("Unknown word");
                }
            }
        }
        Ok(())
    }

    // Evaluate a line of Forth code
    fn eval(&mut self, line: &str) -> Result<(), &'static str> {
        let words = line.split_whitespace();
        for word in words {
            self.execute_word(word)?;
        }
        Ok(())
    }
}

// Parse a number (handles negative numbers)
fn parse_number(s: &str) -> Option<i32> {
    let mut result = 0i32;
    let mut chars = s.chars();
    let mut negative = false;
    
    // Check for negative sign
    if let Some(first) = chars.next() {
        if first == '-' {
            negative = true;
        } else if let Some(digit) = first.to_digit(10) {
            result = digit as i32;
        } else {
            return None;
        }
    } else {
        return None;
    }
    
    // Parse remaining digits
    for c in chars {
        if let Some(digit) = c.to_digit(10) {
            result = result.checked_mul(10)?;
            result = result.checked_add(digit as i32)?;
        } else {
            return None;
        }
    }
    
    if negative {
        result = -result;
    }
    
    Some(result)
}

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
        match forth.eval(input) {
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
