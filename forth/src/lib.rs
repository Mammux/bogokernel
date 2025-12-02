#![cfg_attr(not(test), no_std)]

// Forth interpreter core logic - can be tested on host

// Maximum stack depth
pub const STACK_SIZE: usize = 64;

// Forth interpreter state
pub struct Forth {
    stack: [i32; STACK_SIZE],
    sp: usize, // stack pointer (points to next free slot)
}

impl Forth {
    pub fn new() -> Self {
        Forth {
            stack: [0; STACK_SIZE],
            sp: 0,
        }
    }

    pub fn push(&mut self, val: i32) -> Result<(), &'static str> {
        if self.sp >= STACK_SIZE {
            Err("Stack overflow")
        } else {
            self.stack[self.sp] = val;
            self.sp += 1;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<i32, &'static str> {
        if self.sp == 0 {
            Err("Stack underflow")
        } else {
            self.sp -= 1;
            Ok(self.stack[self.sp])
        }
    }

    pub fn peek(&self) -> Result<i32, &'static str> {
        if self.sp == 0 {
            Err("Stack empty")
        } else {
            Ok(self.stack[self.sp - 1])
        }
    }

    pub fn depth(&self) -> usize {
        self.sp
    }

    pub fn stack_contents(&self) -> &[i32] {
        &self.stack[..self.sp]
    }

    // Execute a single word
    pub fn execute_word(&mut self, word: &str) -> Result<(), &'static str> {
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
    pub fn eval(&mut self, line: &str) -> Result<(), &'static str> {
        let words = line.split_whitespace();
        for word in words {
            self.execute_word(word)?;
        }
        Ok(())
    }
}

// Parse a number (handles negative numbers)
pub fn parse_number(s: &str) -> Option<i32> {
    if s.is_empty() {
        return None;
    }
    
    // Special case: i32::MIN cannot be parsed as positive then negated
    if s == "-2147483648" {
        return Some(i32::MIN);
    }
    
    let mut result = 0i32;
    let mut chars = s.chars();
    let mut negative = false;
    let mut has_digits = false;
    
    // Check for negative sign
    if let Some(first) = chars.next() {
        if first == '-' {
            negative = true;
            // Check if there's at least one digit after the minus sign
            if chars.as_str().is_empty() {
                return None;
            }
        } else if let Some(digit) = first.to_digit(10) {
            result = digit as i32;
            has_digits = true;
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
            has_digits = true;
        } else {
            return None;
        }
    }
    
    if !has_digits {
        return None;
    }
    
    if negative {
        result = result.checked_neg()?;
    }
    
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_push_pop() {
        let mut forth = Forth::new();
        assert_eq!(forth.depth(), 0);
        
        assert!(forth.push(42).is_ok());
        assert_eq!(forth.depth(), 1);
        
        assert_eq!(forth.pop(), Ok(42));
        assert_eq!(forth.depth(), 0);
    }

    #[test]
    fn test_stack_overflow() {
        let mut forth = Forth::new();
        // Fill the stack
        for i in 0..STACK_SIZE {
            assert!(forth.push(i as i32).is_ok());
        }
        // Try to overflow
        assert_eq!(forth.push(999), Err("Stack overflow"));
    }

    #[test]
    fn test_stack_underflow() {
        let mut forth = Forth::new();
        assert_eq!(forth.pop(), Err("Stack underflow"));
    }

    #[test]
    fn test_peek() {
        let mut forth = Forth::new();
        assert!(forth.push(42).is_ok());
        assert_eq!(forth.peek(), Ok(42));
        assert_eq!(forth.depth(), 1); // peek doesn't change depth
    }

    #[test]
    fn test_addition() {
        let mut forth = Forth::new();
        assert!(forth.eval("3 4 +").is_ok());
        assert_eq!(forth.pop(), Ok(7));
    }

    #[test]
    fn test_subtraction() {
        let mut forth = Forth::new();
        assert!(forth.eval("10 3 -").is_ok());
        assert_eq!(forth.pop(), Ok(7));
    }

    #[test]
    fn test_multiplication() {
        let mut forth = Forth::new();
        assert!(forth.eval("6 7 *").is_ok());
        assert_eq!(forth.pop(), Ok(42));
    }

    #[test]
    fn test_division() {
        let mut forth = Forth::new();
        assert!(forth.eval("20 4 /").is_ok());
        assert_eq!(forth.pop(), Ok(5));
    }

    #[test]
    fn test_division_by_zero() {
        let mut forth = Forth::new();
        assert_eq!(forth.eval("10 0 /"), Err("Division by zero"));
    }

    #[test]
    fn test_modulo() {
        let mut forth = Forth::new();
        assert!(forth.eval("17 5 mod").is_ok());
        assert_eq!(forth.pop(), Ok(2));
    }

    #[test]
    fn test_dup() {
        let mut forth = Forth::new();
        assert!(forth.eval("5 dup").is_ok());
        assert_eq!(forth.pop(), Ok(5));
        assert_eq!(forth.pop(), Ok(5));
    }

    #[test]
    fn test_drop() {
        let mut forth = Forth::new();
        assert!(forth.eval("5 10 drop").is_ok());
        assert_eq!(forth.pop(), Ok(5));
        assert_eq!(forth.depth(), 0);
    }

    #[test]
    fn test_swap() {
        let mut forth = Forth::new();
        assert!(forth.eval("5 10 swap").is_ok());
        assert_eq!(forth.pop(), Ok(5));
        assert_eq!(forth.pop(), Ok(10));
    }

    #[test]
    fn test_over() {
        let mut forth = Forth::new();
        assert!(forth.eval("1 2 over").is_ok());
        assert_eq!(forth.pop(), Ok(1));
        assert_eq!(forth.pop(), Ok(2));
        assert_eq!(forth.pop(), Ok(1));
    }

    #[test]
    fn test_rot() {
        let mut forth = Forth::new();
        assert!(forth.eval("1 2 3 rot").is_ok());
        assert_eq!(forth.pop(), Ok(1));
        assert_eq!(forth.pop(), Ok(3));
        assert_eq!(forth.pop(), Ok(2));
    }

    #[test]
    fn test_comparison_equal() {
        let mut forth = Forth::new();
        assert!(forth.eval("5 5 =").is_ok());
        assert_eq!(forth.pop(), Ok(-1)); // true
        
        let mut forth = Forth::new();
        assert!(forth.eval("5 10 =").is_ok());
        assert_eq!(forth.pop(), Ok(0)); // false
    }

    #[test]
    fn test_comparison_less_than() {
        let mut forth = Forth::new();
        assert!(forth.eval("3 7 <").is_ok());
        assert_eq!(forth.pop(), Ok(-1)); // true
        
        let mut forth = Forth::new();
        assert!(forth.eval("10 5 <").is_ok());
        assert_eq!(forth.pop(), Ok(0)); // false
    }

    #[test]
    fn test_comparison_greater_than() {
        let mut forth = Forth::new();
        assert!(forth.eval("8 2 >").is_ok());
        assert_eq!(forth.pop(), Ok(-1)); // true
    }

    #[test]
    fn test_logical_and() {
        let mut forth = Forth::new();
        assert!(forth.eval("15 7 and").is_ok());
        assert_eq!(forth.pop(), Ok(7));
    }

    #[test]
    fn test_logical_or() {
        let mut forth = Forth::new();
        assert!(forth.eval("8 4 or").is_ok());
        assert_eq!(forth.pop(), Ok(12));
    }

    #[test]
    fn test_logical_xor() {
        let mut forth = Forth::new();
        assert!(forth.eval("12 10 xor").is_ok());
        assert_eq!(forth.pop(), Ok(6));
    }

    #[test]
    fn test_invert() {
        let mut forth = Forth::new();
        assert!(forth.eval("0 invert").is_ok());
        assert_eq!(forth.pop(), Ok(-1));
    }

    #[test]
    fn test_negate() {
        let mut forth = Forth::new();
        assert!(forth.eval("5 negate").is_ok());
        assert_eq!(forth.pop(), Ok(-5));
    }

    #[test]
    fn test_constants() {
        let mut forth = Forth::new();
        assert!(forth.eval("true").is_ok());
        assert_eq!(forth.pop(), Ok(-1));
        
        let mut forth = Forth::new();
        assert!(forth.eval("false").is_ok());
        assert_eq!(forth.pop(), Ok(0));
    }

    #[test]
    fn test_negative_numbers() {
        let mut forth = Forth::new();
        assert!(forth.eval("-5 3 +").is_ok());
        assert_eq!(forth.pop(), Ok(-2));
    }

    #[test]
    fn test_complex_expression() {
        let mut forth = Forth::new();
        assert!(forth.eval("3 4 + 5 *").is_ok());
        assert_eq!(forth.pop(), Ok(35)); // (3+4)*5 = 35
    }

    #[test]
    fn test_overflow_protection_add() {
        let mut forth = Forth::new();
        assert_eq!(forth.eval("2147483647 1 +"), Err("Arithmetic overflow"));
    }

    #[test]
    fn test_overflow_protection_sub() {
        let mut forth = Forth::new();
        assert_eq!(forth.eval("-2147483648 1 -"), Err("Arithmetic overflow"));
    }

    #[test]
    fn test_overflow_protection_mul() {
        let mut forth = Forth::new();
        assert_eq!(forth.eval("2000000000 2 *"), Err("Arithmetic overflow"));
    }

    #[test]
    fn test_overflow_protection_div() {
        let mut forth = Forth::new();
        // i32::MIN / -1 causes overflow
        assert_eq!(forth.eval("-2147483648 -1 /"), Err("Arithmetic overflow"));
    }

    #[test]
    fn test_parse_number_positive() {
        assert_eq!(parse_number("42"), Some(42));
        assert_eq!(parse_number("0"), Some(0));
        assert_eq!(parse_number("123456"), Some(123456));
    }

    #[test]
    fn test_parse_number_negative() {
        assert_eq!(parse_number("-42"), Some(-42));
        assert_eq!(parse_number("-1"), Some(-1));
    }

    #[test]
    fn test_parse_number_invalid() {
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("12a3"), None);
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("-"), None);
    }

    #[test]
    fn test_parse_number_overflow() {
        // Number too large for i32
        assert_eq!(parse_number("9999999999999"), None);
    }

    #[test]
    fn test_stack_contents() {
        let mut forth = Forth::new();
        assert!(forth.eval("1 2 3").is_ok());
        let contents = forth.stack_contents();
        assert_eq!(contents, &[1, 2, 3]);
    }

    #[test]
    fn test_unknown_word() {
        let mut forth = Forth::new();
        assert_eq!(forth.eval("notaword"), Err("Unknown word"));
    }

    #[test]
    fn test_empty_eval() {
        let mut forth = Forth::new();
        assert!(forth.eval("").is_ok());
        assert_eq!(forth.depth(), 0);
    }

    #[test]
    fn test_multiple_spaces() {
        let mut forth = Forth::new();
        assert!(forth.eval("  5   10   +  ").is_ok());
        assert_eq!(forth.pop(), Ok(15));
    }
}
