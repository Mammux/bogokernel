# BogoKernel

A tiny experimental operating system kernel written in Rust, targeting **RISC-V (rv64)** and running under **QEMU** with **OpenSBI**.  
This project is educational — it demonstrates how to bring up a kernel in S-mode, set up paging (Sv39), run user programs in U-mode loaded from ELF binaries, and implement a basic syscall interface with file I/O.

---

## Features

- **Rust, no_std** kernel built with `cargo`.
- Runs in **Supervisor mode (S-mode)** on RISC-V.
- **Custom entry** (`_start`) with trap stack, vectored trap handling, and timer interrupts.
- **Sv39 paging** enabled with identity mapping for the kernel and U=1 mappings for user code/data.
- **Minimal heap** (via `linked_list_allocator`) to allow kernel allocations.
- **ELF64 loader**: Maps PT_LOAD segments, sets up argv/envp on user stack, and jumps to entry point in U-mode.
- **Unified writable filesystem**: Embedded files are copied to a writable in-memory filesystem at boot, supporting file creation, modification, and deletion.
- **File descriptor table**: Supports stdin (fd 0), stdout (fd 1), stderr (fd 2), and regular files (fd 3+).
- **Dynamic program loading**: `exec()` and `execv()` syscalls to load and run programs.
- **User-space library** (`usys`): Syscall wrappers, I/O traits, and convenience macros (`print!`, `println!`).
- **Minimal libc**: Standard C library with file I/O, string functions, malloc/free, printf, and curses support.
- **Multiple user applications**: Interactive shell, rogue-like games, cat utility, filesystem tests, GPU tests, and hello world examples.
- **VirtIO GPU support** (optional): Framebuffer-based display with font rendering for graphical output.
- **System calls** (20 total):  
  - `write(ptr, len)` → write bytes to stdout  
  - `write_cstr(ptr)` → write NUL-terminated string  
  - `write_fd(fd, buf, len)` → write to file descriptor  
  - `read(fd, buf, len)` → read from file or stdin (blocking for stdin)  
  - `open(path)` → open file from filesystem, returns fd  
  - `creat(path, mode)` → create/truncate writable file  
  - `close(fd)` → close file descriptor  
  - `lseek(fd, offset, whence)` → seek in file  
  - `unlink(path)` → delete file  
  - `stat(path, buf)` → get file metadata  
  - `chmod(path, mode)` → change file permissions  
  - `brk(addr)` → manage user heap (allocate/free pages)  
  - `gettime()` → get system ticks  
  - `exec(path)` → execute program  
  - `execv(path, argv)` → execute program with arguments  
  - `poweroff()` → shutdown via SBI  
  - `exit()` → reload shell  
  - `readdir(buf, len)` → list files in the filesystem  
  - `get_fb_info(buf)` → get framebuffer information (GPU mode)  
  - `fb_flush()` → flush framebuffer to display (GPU mode)  
- Works under **QEMU virt machine** with `-bios default` (OpenSBI).

---

## Project Structure

This is a Cargo workspace with multiple packages:

- **`kernel`** — The S-mode kernel (main binary)
- **`uapi`** — Syscall number definitions (shared between kernel and userspace)
- **`usys`** — User-space syscall wrapper library with I/O helpers
- **`userapp`** — User applications (shell, rogue, fstest, mkfiles, gputest)
- **`cat`** — Cat utility for reading files
- **`forth`** — Forth interpreter (stack-based programming language)
- **`c_hello`** — C language hello world example
- **`crogue`** — C mini rogue game
- **`curses_test`** — C curses library test
- **`rogue`** — Full rogue game port (C, builds as bigrogue.elf)
- **`libc`** — C standard library with curses support

---

## Filesystem

At boot, the kernel initializes a unified writable filesystem by copying all embedded files into memory. This supports:

- **File creation**: Create new files with `creat()`
- **File modification**: Write to existing files with `write_fd()`
- **File deletion**: Remove files with `unlink()`
- **Directory listing**: List files with `readdir()`

### Embedded Files

The following files are embedded at compile time and available at boot:

- `dungeon.map` — Map data for the rogue game
- `shell.elf` — Interactive shell (loaded at boot)
- `rogue.elf` — Rogue-like game (Rust)
- `forth.elf` — Forth interpreter (Rust)
- `hello.elf` — Hello world example (C)
- `crogue.elf` — Mini rogue game (C)
- `curses_test.elf` — Curses library test (C)
- `bigrogue.elf` — Full rogue game port (C)
- `fstest.elf` — Filesystem test utility (Rust)
- `mkfiles.elf` — File creation test (Rust)
- `gputest.elf` — GPU/display test (Rust)
- `etc/motd` — Message of the day

Files are embedded at compile time via `include_bytes!` in `kernel/src/fs.rs`.

---

## User Applications

### Shell (`shell.elf`)
Interactive command shell loaded at boot. Built-in commands:
- `ls` — List files in the filesystem
- `help` — Show available commands
- `shutdown` — Power off the system

Run any program by typing its name (e.g., `hello`, `rogue`, `crogue`, `forth`, `lisp`).

### Programming Languages
- **`lisp.elf`** — Interactive LISP REPL with lambda functions and first-class functions ([see LISP README](lisp/README.md))
- **`forth.elf`** — Forth interpreter with REPL (stack-based programming language)

### Games
- **`rogue.elf`** — Rust rogue-like dungeon game
- **`crogue.elf`** — C mini rogue game
- **`bigrogue.elf`** — Full rogue game port (classic BSD rogue)

### Utilities
- **`fstest.elf`** — Filesystem test utility (tests file creation/writing)
- **`mkfiles.elf`** — File creation test


### Tests
- **`curses_test.elf`** — Curses library test
- **`gputest.elf`** — GPU/framebuffer test (requires GPU mode)

### Hello World
- **`hello.elf`** — Simple hello world (C version)

### Not Embedded (Build Separately)
- **`cat`** — Display file contents (Rust package in `cat/`)

---

## Building and Running

### Requirements

- **Rust nightly** with `riscv64gc-unknown-none-elf` target and `rust-src`:
  ```sh
  rustup toolchain install nightly
  rustup target add riscv64gc-unknown-none-elf --toolchain nightly
  rustup component add rust-src --toolchain nightly
  ```
    
- **QEMU with RISC-V support** (`qemu-system-riscv64`)

### Build Steps

The project uses a Cargo workspace. Build individual applications, then build the kernel which embeds them. See `run.bat` for a complete build script.

### Run in QEMU

```sh
qemu-system-riscv64 \
  -machine virt -m 128M -nographic \
  -bios default \
  -kernel target/riscv64gc-unknown-none-elf/debug/kernel
```

Or use the provided scripts:
- Windows: `run.bat` or `test.bat`
- Linux/macOS: `run.sh`

### Expected Output

```
riscv-os: hello from S-mode at 0x8020_0000!
trap stack initialized
traps enabled
timers initialized
SV39 paging enabled (identity map + UART)
Heap init OK.
Box value = 0xc0ffee
Vec sum = 140
Loaded shell: entry=0x10000, sp=0x40008000
shell> _
```

---

## Testing

BogoKernel includes comprehensive test coverage for kernel logic:

- **45 unit tests** covering ~501 lines of test code
- **Filesystem tests** (24 tests): File operations, read/write, metadata
- **Paging tests** (10 tests): PPN/VPN calculations, memory layout validation
- **ELF loader tests** (11 tests): Permission mapping, ELF validation
- **Integration tests**: `test.py` for end-to-end QEMU validation

### Running Tests

```sh
# Run unit tests (21 tests for pure functions)
cargo test -p kernel --lib --target x86_64-unknown-linux-gnu

# Integration test (requires built kernel)
python3 test.py

# Verify kernel compiles
cargo check -p kernel
```

**Note**: Unit tests are extracted into a library that can be tested with `cargo test`. The library contains pure functions from `sv39`, `elf`, and `fs` modules. Tests run on the host platform (x86_64) while the kernel binary remains `no_std` for RISC-V.

For detailed testing information, see **[TESTING.md](TESTING.md)**.

---

## System Calls Reference

| Number | Name | Signature | Description |
|--------|------|-----------|-------------|
| 1 | `WRITE` | `write(ptr, len) -> usize` | Write bytes to stdout |
| 2 | `EXIT` | `exit() -> !` | Reload shell |
| 3 | `WRITE_CSTR` | `write_cstr(ptr) -> usize` | Write NUL-terminated string |
| 4 | `OPEN` | `open(path) -> fd` | Open file from filesystem |
| 5 | `READ` | `read(fd, buf, len) -> n` | Read from file/stdin |
| 6 | `WRITE_FD` | `write_fd(fd, buf, len) -> n` | Write to file descriptor |
| 7 | `CLOSE` | `close(fd) -> result` | Close file descriptor |
| 8 | `LSEEK` | `lseek(fd, offset, whence) -> new_offset` | Seek in file |
| 9 | `BRK` | `brk(addr) -> new_brk` | Manage heap (allocate pages) |
| 10 | `GETTIME` | `gettime() -> ticks` | Get system ticks |
| 11 | `POWEROFF` | `poweroff() -> !` | Shutdown system |
| 12 | `EXEC` | `exec(path) -> !` | Execute program |
| 13 | `EXECV` | `execv(path, argv) -> !` | Execute program with arguments |
| 14 | `CREAT` | `creat(path, mode) -> fd` | Create/truncate file |
| 15 | `UNLINK` | `unlink(path) -> result` | Delete file |
| 16 | `STAT` | `stat(path, buf) -> result` | Get file metadata |
| 17 | `CHMOD` | `chmod(path, mode) -> result` | Change file permissions |
| 18 | `READDIR` | `readdir(buf, len) -> n` | List files in filesystem |
| 19 | `GET_FB_INFO` | `get_fb_info(buf) -> result` | Get framebuffer info (GPU) |
| 20 | `FB_FLUSH` | `fb_flush() -> result` | Flush framebuffer (GPU) |

All syscalls use the RISC-V calling convention: `a7` = syscall number, `a0-a2` = arguments, `a0` = return value.

---

## Roadmap

Completed features:
- ✅ Syscall table (open, read, write, close, lseek, brk, exec, poweroff, gettime)
- ✅ User heap via brk
- ✅ Embedded filesystem (RAMFS)
- ✅ Dynamic program loading (exec/execv)
- ✅ Writable filesystem (in-memory)
- ✅ File creation, deletion, and modification syscalls
- ✅ Directory listing (readdir)
- ✅ VirtIO GPU support with framebuffer console
- ✅ Curses library for C applications

Possible next steps:

  -  Harden paging with correct section permissions (W^X enforcement)

  -  Add a real frame allocator (buddy allocator or bitmap)

  -  Abstract user programs as processes with PCB, add scheduler

  -  Extend ELF loader with relocations and auxv

  -  Add persistent storage via virtio-blk driver

  -  Improve trap/interrupt handling (PLIC for external interrupts)

  -  Prepare for SMP with per-hart stacks and spinlocks

  -  Developer tooling: GDB scripts, xtask runner, automated testing