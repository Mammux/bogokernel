# BogoForth - Forth Interpreter for BogoKernel

A simple Forth interpreter implementation for BogoKernel, providing a stack-based programming environment.

## Features

- **Stack-based computation**: Classic Forth data stack with 64-element capacity
- **REPL interface**: Interactive Read-Eval-Print-Loop for immediate feedback
- **Essential Forth words**: Arithmetic, stack manipulation, I/O, and logical operations
- **Integer arithmetic**: 32-bit signed integer operations

## Running Forth

From the BogoKernel shell, simply type:

```
> forth
```

The Forth interpreter will start and display:

```
BogoForth v0.1
A simple Forth interpreter for BogoKernel
Type 'words' for available words, or 'bye' to exit

ok 
```

## Available Words

### Arithmetic Operations
- `+` — Add two numbers (a b -- a+b)
- `-` — Subtract (a b -- a-b)
- `*` — Multiply (a b -- a*b)
- `/` — Divide (a b -- a/b)
- `mod` — Modulo (a b -- a%b)

### Stack Manipulation
- `dup` — Duplicate top of stack (a -- a a)
- `drop` — Remove top of stack (a -- )
- `swap` — Swap top two items (a b -- b a)
- `over` — Copy second item to top (a b -- a b a)
- `rot` — Rotate top three items (a b c -- b c a)

### I/O Operations
- `.` — Pop and print top of stack
- `.s` — Display entire stack contents without modifying it
- `cr` — Print newline
- `emit` — Pop and print character with given ASCII code

### Comparison Operations
- `=` — Equal (a b -- flag) where flag is -1 (true) or 0 (false)
- `<` — Less than (a b -- flag)
- `>` — Greater than (a b -- flag)

### Logical Operations
- `and` — Bitwise AND (a b -- a&b)
- `or` — Bitwise OR (a b -- a|b)
- `xor` — Bitwise XOR (a b -- a^b)
- `invert` — Bitwise NOT (a -- ~a)
- `negate` — Arithmetic negation (a -- -a)

### Constants
- `true` — Push -1 (true flag)
- `false` — Push 0 (false flag)

### Special Commands
- `words` — Display list of available words
- `bye` or `quit` — Exit the Forth interpreter

## Usage Examples

### Basic Arithmetic

```forth
ok 3 4 +
ok .
7 
ok 
```

### Stack Manipulation

```forth
ok 5 dup *
ok .
25 
ok 
```

### Using .s to View Stack

```forth
ok 10 20 30
ok .s
<3> 10 20 30 
ok 
```

### Computing Factorial (iterative)

```forth
ok 5 1 swap            ( start with result=1, n=5 )
ok dup . cr           ( print current n )
5 
ok swap over *        ( multiply result by n )
ok swap 1 -           ( decrement n )
ok dup . cr
4 
ok swap over *
ok swap 1 -
ok dup . cr
3 
ok swap over *
ok swap 1 -
ok dup . cr
2 
ok swap over *
ok swap 1 -
ok dup . cr
1 
ok swap over *
ok swap drop          ( remove counter, keep result )
ok .
120 
ok 
```

### Character Output

```forth
ok 72 emit 101 emit 108 emit 108 emit 111 emit
Hello
ok 
```

## Implementation Details

- **Language**: Rust (no_std)
- **Stack size**: 64 elements (32-bit signed integers)
- **Input buffer**: 128 bytes
- **Error handling**: Stack overflow/underflow detection, division by zero protection

## Limitations

This is a minimal Forth implementation with the following limitations:

- No dictionary or user-defined words
- No compilation mode (only interpretation)
- No control structures (if-then-else, do-loop)
- No string support beyond character I/O
- No floating point arithmetic
- No file I/O operations
- Fixed 32-bit integer arithmetic only

## Building

The Forth interpreter is built as part of the BogoKernel workspace:

```bash
# Build forth binary
cargo build -p forth --release

# Copy to kernel directory for embedding
cp target/riscv64gc-unknown-none-elf/release/forth kernel/forth.elf

# Build kernel with embedded forth.elf
cargo build -p kernel
```

## Future Enhancements

Possible extensions:

- User-defined words (dictionary support)
- Compilation mode with IMMEDIATE words
- Control structures (IF-THEN-ELSE, BEGIN-UNTIL, DO-LOOP)
- String handling and string literals
- File I/O integration
- Memory access words (@ ! C@ C!)
- More stack words (2dup, 2drop, etc.)
- Return stack operations (>R R> R@)
