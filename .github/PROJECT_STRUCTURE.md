# BogoKernel Project Structure

This document describes the structure and organization of the BogoKernel project for AI agents and developers.

## Overview

BogoKernel is a RISC-V educational operating system kernel written in Rust, with user-space applications written in both Rust and C. The project uses a workspace structure with multiple packages.

## Repository Layout

```
bogokernel/
├── .cargo/               # Cargo configuration
│   └── config.toml       # Build target and runner configuration
├── .github/              # GitHub and project documentation
│   └── PROJECT_STRUCTURE.md  # This file
├── kernel/               # Kernel source (Rust, S-mode)
│   ├── src/              # Kernel source code
│   │   ├── main.rs       # Kernel entry point
│   │   ├── fs.rs         # RAMFS embedded filesystem
│   │   └── dungeon.map   # Game data file
│   └── Cargo.toml        # Kernel dependencies
├── libc/                 # C standard library implementation
│   ├── include/          # Header files (stdio.h, curses.h, etc.)
│   ├── src/              # C source files
│   │   ├── curses.c      # Curses/ncurses implementation
│   │   ├── stdio.c       # Standard I/O functions
│   │   └── ...
│   ├── build.sh          # Build script for libc.a
│   └── libc.a            # Compiled static library
├── uapi/                 # Syscall API definitions (shared)
│   └── src/lib.rs        # Syscall numbers
├── usys/                 # User-space syscall wrapper library (Rust)
│   └── src/lib.rs        # Syscall wrappers, I/O traits
├── userapp/              # Rust user applications
│   └── src/bin/
│       ├── shell.rs      # Interactive shell (loads at boot)
│       └── rogue.rs      # Rogue game (Rust version)
├── cat/                  # Cat utility (Rust)
├── c_hello/              # Hello world example (C)
│   ├── hello.c
│   ├── crt0.s            # C runtime startup
│   ├── linker.ld         # Linker script
│   └── build.sh          # Build script
├── crogue/               # Mini rogue game (C)
│   └── build.sh
├── curses_test/          # Curses library test (C)
│   └── build.sh
├── rogue/                # Full rogue game implementation (C)
│   ├── *.c, *.h          # Rogue source files
│   ├── build.sh          # Builds as 'rogue' binary
│   └── ...               # (copied as bigrogue.elf)
└── Cargo.toml            # Workspace configuration

Build Artifacts (in kernel/):
├── bigrogue.elf          # Full rogue game (from rogue/)
├── crogue.elf            # Mini rogue game
├── curses_test.elf       # Curses test
├── hello.elf             # Hello world (C)
├── shell.elf             # Shell (Rust)
├── rogue.elf             # Rogue (Rust)
└── cat.elf               # Cat utility (Rust)
```

## Key Components

### 1. Kernel (kernel/)
- **Language**: Rust (no_std)
- **Mode**: RISC-V Supervisor mode (S-mode)
- **Features**:
  - Sv39 paging with identity mapping
  - ELF64 loader for user programs
  - RAMFS embedded filesystem
  - 12 system calls (write, read, open, close, exec, etc.)
  - Trap handling and timer interrupts

### 2. C Library (libc/)
- **Purpose**: Provides standard C library functions for C user applications
- **Key Files**:
  - `curses.c` - Curses/ncurses implementation with screen buffering
  - `stdio.c` - Standard I/O (printf, sprintf, etc.)
  - `stdlib.c` - Memory allocation, conversion functions
  - `string.c` - String manipulation
  - `syscall.c` - Low-level syscall interface
- **Build Output**: `libc.a` static library

### 3. User Applications

#### Rust Applications
- Built using Cargo with target `riscv64gc-unknown-none-elf`
- Linked against `usys` library for syscalls
- Examples: shell, rogue, cat

#### C Applications
- Built using `riscv64-linux-gnu-gcc` cross-compiler
- Linked against `libc.a`
- Use `crt0.s` for startup and `linker.ld` for memory layout
- Examples: hello.elf, crogue.elf, curses_test.elf, bigrogue.elf

### 4. Important Files

#### .cargo/config.toml
Configures the default target and QEMU runner for the project.

#### kernel/src/fs.rs
Defines the embedded RAMFS filesystem. All user programs are embedded here using `include_bytes!()`. To add a new application:
1. Build the ELF file
2. Copy it to `kernel/` directory
3. Add it to the `FILES` array in `fs.rs`

#### rust-toolchain.toml
Specifies the Rust toolchain version to use.

## Build Dependencies

### Required Tools
- **Rust**: Nightly toolchain with `riscv64gc-unknown-none-elf` target
- **GCC**: `riscv64-linux-gnu-gcc` cross-compiler for C applications
- **QEMU**: `qemu-system-riscv64` for testing
- **ld**: `riscv64-linux-gnu-ld` linker

### Installation Commands
```bash
# Rust toolchain
rustup toolchain install nightly
rustup target add riscv64gc-unknown-none-elf --toolchain nightly
rustup component add rust-src --toolchain nightly

# RISC-V GCC toolchain (Ubuntu/Debian)
sudo apt-get install gcc-riscv64-linux-gnu

# QEMU
sudo apt-get install qemu-system-riscv64
```

## Memory Layout

### User Space
- **Code/Data**: 0x10000 - 0x40000000 (U=1, RWX)
- **Stack**: Top of user memory, grows down
- **Heap**: Managed by `brk()` syscall

### Kernel Space
- **Base**: 0x8020_0000 (loaded by QEMU/OpenSBI)
- **Identity mapped** with Sv39 paging

## System Calls

| Number | Name       | Description                    |
|--------|------------|--------------------------------|
| 1      | WRITE      | Write bytes to stdout          |
| 2      | EXIT       | Exit and reload shell          |
| 3      | WRITE_CSTR | Write C string to stdout       |
| 4      | OPEN       | Open file from RAMFS           |
| 5      | READ       | Read from file or stdin        |
| 6      | WRITE_FD   | Write to file descriptor       |
| 7      | CLOSE      | Close file descriptor          |
| 8      | LSEEK      | Seek in file                   |
| 9      | BRK        | Manage user heap               |
| 10     | GETTIME    | Get system ticks               |
| 11     | POWEROFF   | Shutdown via SBI               |
| 12     | EXEC       | Execute program from RAMFS     |

## Common Tasks

See [BUILD_GUIDE.md](BUILD_GUIDE.md) for detailed build instructions.

Quick reference:
- **Build all**: See build guide
- **Build kernel only**: `cargo build -p kernel`
- **Build C app**: `cd <app_dir> && bash build.sh`
- **Run in QEMU**: `cargo run -p kernel` or `./run.sh`
- **Test**: `python3 test.py`

## Important Notes for AI Agents

1. **C Applications**: Always rebuild `libc.a` first if you modify any C library files
2. **Kernel**: Must rebuild kernel after updating any .elf files in `kernel/` directory
3. **Build Order**: libc → C apps → Rust apps → kernel
4. **Bigrogue**: The full rogue game in `rogue/` builds to `rogue` binary, copy as `bigrogue.elf`
5. **Curses**: The curses implementation in `libc/src/curses.c` uses a screen buffer and dirty flags for efficient updates
6. **Git**: Exclude build artifacts (*.o, *.a, rogue binary, *.elf in kernel/) from commits

## Debugging

- Kernel prints go to QEMU console
- User program I/O goes through syscalls to kernel
- Use `-d int,cpu_reset` QEMU flag for debugging traps
- GDB: `riscv64-linux-gnu-gdb` can attach to QEMU with `-s -S` flags

## Architecture Details

- **Target**: `riscv64gc-unknown-none-elf`
- **ISA**: RV64GC (RV64IMAFDCZicsr_Zifencei)
- **ABI**: lp64d
- **Machine**: QEMU virt
- **Privilege Modes**: M-mode (OpenSBI), S-mode (kernel), U-mode (user apps)
