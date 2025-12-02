# BogoLISP - Simple LISP REPL for BogoKernel

A minimal but functional LISP interpreter running on BogoKernel.

## Features

- **S-expression parsing**: Support for atoms, numbers, and lists
- **Core primitives**: `car`, `cdr`, `cons`, `atom`, `eq`
- **Arithmetic**: `+`, `-`, `*`, `/`
- **Control flow**: `if` conditionals
- **Special forms**: `quote`, `lambda`, `define`
- **Function application**: First-class functions with closures
- **Simple REPL**: Read-Eval-Print Loop with basic line editing

## Usage

From the BogoShell prompt, run:

```
> lisp
```

You'll see the LISP prompt:

```
BogoLISP v0.1
Type expressions to evaluate, or 'quit' to exit

lisp> 
```

To exit the REPL, type `quit` or `exit`.

## Examples

### Basic Arithmetic

```lisp
lisp> (+ 1 2 3)
6

lisp> (* 5 5)
25

lisp> (- 10 3 2)
5

lisp> (/ 20 4)
5
```

### List Operations

```lisp
lisp> '(1 2 3)
(1 2 3)

lisp> (cons 1 2)
(1 . 2)

lisp> (cons 1 (cons 2 (cons 3 nil)))
(1 2 3)

lisp> (car '(1 2 3))
1

lisp> (cdr '(1 2 3))
(2 3)

lisp> (car (cdr '(1 2 3)))
2
```

### Predicates

```lisp
lisp> (atom 42)
t

lisp> (atom '(1 2))
nil

lisp> (eq 1 1)
t

lisp> (eq 1 2)
nil
```

### Conditionals

```lisp
lisp> (if (eq 1 1) 42 0)
42

lisp> (if (eq 1 2) 42 0)
0

lisp> (if (atom 5) 100 200)
100
```

### Lambda Functions

```lisp
lisp> ((lambda (x) (* x x)) 5)
25

lisp> ((lambda (x y) (+ x y)) 3 4)
7

lisp> ((lambda (x) (+ x 1)) 10)
11
```

### Define Variables and Functions

```lisp
lisp> (define pi 314)
314

lisp> pi
314

lisp> (define square (lambda (x) (* x x)))
<lambda>

lisp> (square 7)
49

lisp> (define add (lambda (x y) (+ x y)))
<lambda>

lisp> (add 10 20)
30
```

### Complex Examples

Factorial-like computation:
```lisp
lisp> (define mult-add (lambda (a b c) (+ (* a b) c)))
<lambda>

lisp> (mult-add 5 4 2)
22
```

Check if a number equals zero:
```lisp
lisp> (define zero? (lambda (x) (eq x 0)))
<lambda>

lisp> (zero? 0)
t

lisp> (zero? 5)
nil
```

## Language Reference

### Primitives

- `(car list)` - Returns the first element of a list
- `(cdr list)` - Returns the rest of the list (all but first element)
- `(cons x y)` - Constructs a pair or list with x as head and y as tail
- `(atom expr)` - Returns `t` if expr is an atom, `nil` otherwise
- `(eq x y)` - Returns `t` if x and y are equal, `nil` otherwise

### Arithmetic Operators

- `(+ x y ...)` - Addition (variadic)
- `(- x y ...)` - Subtraction (variadic, unary negation if one arg)
- `(* x y ...)` - Multiplication (variadic)
- `(/ x y ...)` - Integer division (variadic)

### Special Forms

- `(quote expr)` or `'expr` - Returns expr unevaluated
- `(if cond then else)` - Evaluates `then` if `cond` is non-nil, else evaluates `else`
- `(lambda (params...) body)` - Creates an anonymous function
- `(define symbol value)` - Binds symbol to value in global environment

### Data Types

- **Numbers**: Integer literals (e.g., `42`, `-5`, `0`)
- **Symbols**: Alphanumeric identifiers (e.g., `x`, `foo`, `add1`)
- **Lists**: S-expressions enclosed in parentheses (e.g., `(1 2 3)`, `(+ 1 2)`)
- **Nil**: The empty list, represented as `nil`
- **T**: The truth value, represented as `t`

## Implementation Details

- **Memory**: Fixed allocation of 1024 cells, 128 environment slots, 8KB string pool
- **Garbage Collection**: Currently uses static allocation (no automatic GC)
- **Numbers**: Integer-only arithmetic (no floating-point)
- **Input**: Line-based input with up to 256 characters per expression
- **Evaluation**: Tree-walking interpreter with lexical scoping for lambdas

## Limitations

- No floating-point numbers
- No string type (only symbols)
- No macros
- No tail-call optimization
- Limited error messages
- Fixed memory limits (will error when full)
- No file I/O
- Single-line input only (no multi-line expressions)

## Building

The LISP REPL is built as a C application using the BogoKernel libc:

```bash
cd lisp
bash build.sh
cp lisp.elf ../kernel/lisp.elf
```

Then rebuild the kernel to embed the LISP REPL:

```bash
cd ..
cargo build -p kernel
```

## Future Enhancements

Possible improvements for the LISP interpreter:

- Garbage collection with mark-and-sweep
- More primitive functions (list, length, append, etc.)
- Better error messages with line numbers
- Multi-line input support
- String data type
- Read/write from files
- More comprehensive standard library
- Tail-call optimization for recursion
- Macro system
- REPL history and line editing (arrow keys, etc.)
