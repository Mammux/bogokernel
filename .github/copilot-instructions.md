# GitHub Copilot Instructions for BogoKernel

This file provides guidance for GitHub Copilot when working on the BogoKernel project.

## Project Overview

BogoKernel is an educational RISC-V operating system kernel written in Rust and C:
- **Target Architecture**: RISC-V 64-bit (rv64gc-unknown-none-elf)
- **Execution Environment**: QEMU virt machine with OpenSBI firmware
- **Privilege Modes**: S-mode (kernel), U-mode (user applications)
- **Memory Management**: Sv39 paging with identity mapping
- **Languages**: Rust (kernel, some apps), C (libc, most user apps)
- **Build System**: Cargo workspace + shell scripts for C components

## Repository Structure

```
bogokernel/
├── kernel/          # S-mode kernel (Rust, no_std)
├── libc/            # C standard library for user apps
├── uapi/            # Syscall definitions (shared)
├── usys/            # User-space syscall wrappers (Rust)
├── userapp/         # Rust user applications (shell, rogue)
├── cat/             # Cat utility (Rust)
├── c_hello/         # Hello world (C)
├── crogue/          # Mini rogue game (C)
├── curses_test/     # Curses test (C)
└── rogue/           # Full rogue game (C)
```

## Build Order & Dependencies

**Critical**: Always follow this build order:

1. **libc** → C standard library (`libc.a`)
2. **C applications** → Build all C apps, copy ELF files to `kernel/`
3. **Rust applications** → Build Rust apps, copy ELF files to `kernel/`
4. **kernel** → Embeds all `.elf` files from `kernel/` directory at compile time

### Build Commands

```bash
# 1. Build C library
cd libc && bash build.sh && cd ..

# 2. Build C applications
cd c_hello && bash build.sh && cp hello.elf ../kernel/ && cd ..
cd crogue && bash build.sh && cp crogue.elf ../kernel/ && cd ..
cd curses_test && bash build.sh && cp curses_test.elf ../kernel/ && cd ..
cd rogue && bash build.sh && cp rogue ../kernel/bigrogue.elf && cd ..

# 3. Build Rust applications
cargo build -p cat --release
cargo build -p userapp --release
cp target/riscv64gc-unknown-none-elf/release/cat kernel/cat.elf
cp target/riscv64gc-unknown-none-elf/release/shell kernel/shell.elf
cp target/riscv64gc-unknown-none-elf/release/rogue kernel/rogue.elf

# 4. Build kernel
cargo build -p kernel
```

## Code Style & Conventions

### Rust Code
- **Standard**: Follow Rust conventions (rustfmt)
- **Error Handling**: Use `Result` types where appropriate
- **no_std**: Kernel and user apps are `no_std` environments
- **Memory Safety**: Leverage Rust's ownership system
- **Unsafe**: Only use `unsafe` when necessary for low-level operations

### C Code
- **Standard**: K&R style with 4-space indentation
- **Headers**: Include guards in all header files
- **Memory**: Manual memory management via `brk()` syscall
- **Linking**: All C apps link against `libc.a`

### Assembly
- **Syntax**: GNU assembler syntax
- **Files**: Used for C runtime startup (`crt0.s`)

## Key Files & Their Purposes

### Kernel Files
- `kernel/src/main.rs` - Kernel entry point, initialization
- `kernel/src/fs.rs` - RAMFS filesystem, embeds user programs
- `kernel/linker.ld` - Memory layout for kernel

### C Library Files
- `libc/src/curses.c` - Curses implementation with screen buffering
- `libc/src/stdio.c` - Standard I/O (printf, sprintf)
- `libc/src/syscall.c` - Low-level syscall interface
- `libc/include/*.h` - Header files

### Shared Files
- `uapi/src/lib.rs` - Syscall number definitions
- `.cargo/config.toml` - Build target and QEMU runner config

## Testing & Validation

### Manual Testing
```bash
# Run kernel in QEMU
cargo run -p kernel

# Or use script
./run.sh

# Exit QEMU: Ctrl-A then X
```

### Automated Testing
```bash
# Run test script
python3 test.py
```

The test script:
1. Starts QEMU
2. Waits for shell prompt
3. Runs test commands
4. Validates output
5. Saves to `test_output.txt`

### Validation Checklist
When making changes:
- [ ] Build succeeds without errors
- [ ] Kernel boots and shows shell prompt
- [ ] Test commands work (hello, rogue, etc.)
- [ ] No regressions in existing functionality
- [ ] Changes follow project conventions

## System Calls

The kernel provides 12 system calls:

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

## Common Pitfalls & Best Practices

### 1. Build Order
**Problem**: Building kernel before user applications  
**Solution**: Always build apps first, then kernel (kernel embeds apps)

### 2. Missing ELF Files
**Problem**: Kernel build fails with "No such file"  
**Solution**: Ensure all `.elf` files exist in `kernel/` before building kernel

### 3. C Library Changes
**Problem**: C app doesn't reflect libc changes  
**Solution**: Rebuild libc → rebuild C app → rebuild kernel

### 4. Memory Layout
**Problem**: User program crashes  
**Solution**: Check linker scripts, ensure proper memory regions (code at 0x10000)

### 5. Curses Updates
**Problem**: Screen doesn't update  
**Solution**: Call `refresh()` after changes (curses uses screen buffer)

### 6. Build Artifacts in Git
**Problem**: Committing build artifacts  
**Solution**: Exclude: `target/`, `*.o`, `*.a`, `kernel/*.elf`, `rogue/rogue`

### 7. Cross-Compilation
**Problem**: Wrong GCC or target  
**Solution**: Use `riscv64-linux-gnu-gcc` for C, target `riscv64gc-unknown-none-elf`

## Adding New Features

### Adding a Syscall
1. Add syscall number to `uapi/src/lib.rs`
2. Implement handler in `kernel/src/main.rs`
3. Add wrapper in `usys/src/lib.rs` (Rust) or `libc/src/syscall.c` (C)
4. Document in README.md

### Adding a User Program
1. Create program source (Rust or C)
2. Build and copy to `kernel/program.elf`
3. Add to `kernel/src/fs.rs` FILES array
4. Rebuild kernel
5. Test via shell exec command

### Modifying Curses Library
1. Edit `libc/src/curses.c`
2. Rebuild libc: `cd libc && bash build.sh`
3. Rebuild C apps that use curses (rogue, crogue, curses_test)
4. Rebuild kernel
5. Test in QEMU

## Tool Requirements

### Required Tools
- **Rust**: Nightly toolchain with `riscv64gc-unknown-none-elf` target
- **GCC**: `riscv64-linux-gnu-gcc` cross-compiler
- **QEMU**: `qemu-system-riscv64` emulator
- **Python**: For test scripts

### Installation
```bash
# Rust toolchain
rustup toolchain install nightly
rustup target add riscv64gc-unknown-none-elf --toolchain nightly
rustup component add rust-src --toolchain nightly

# GCC cross-compiler (Ubuntu/Debian)
sudo apt-get install gcc-riscv64-linux-gnu

# QEMU
sudo apt-get install qemu-system-riscv64

# Python (usually pre-installed)
python3 --version
```

## Memory Layout

### User Space
- **Code/Data**: 0x10000 - 0x40000000 (U=1, RWX)
- **Stack**: Top of user memory, grows down
- **Heap**: Managed via `brk()` syscall, grows up

### Kernel Space
- **Base Address**: 0x80200000 (loaded by QEMU/OpenSBI)
- **Mapping**: Identity mapped with Sv39 paging

## Documentation References

For more detailed information, refer to:
- `.github/PROJECT_STRUCTURE.md` - Architecture and component details
- `.github/BUILD_GUIDE.md` - Comprehensive build instructions
- `.github/QUICKSTART.md` - Quick start guide
- `.github/CURSES_NOTES.md` - Curses implementation details
- `README.md` - Project overview and features

## When Making Changes

1. **Understand scope**: Identify which components are affected
2. **Follow build order**: libc → C apps → Rust apps → kernel
3. **Test incrementally**: Build and test after each change
4. **Verify boot**: Ensure kernel boots and shell appears
5. **Test functionality**: Run affected programs in shell
6. **Check output**: Verify expected behavior
7. **Clean builds**: Use `cargo clean` if strange errors occur

## Debugging Tips

- **Kernel logs**: Go to QEMU console (stdout)
- **User output**: Goes through syscalls to kernel
- **QEMU flags**: Add `-d int,cpu_reset` for trap debugging
- **GDB**: Use `riscv64-linux-gnu-gdb` with QEMU `-s -S` flags

## AI Agent Specific Notes

When assisting with BogoKernel:
1. Always check existing documentation first (`.github/` directory)
2. Respect the build order dependency chain
3. Don't modify working code unnecessarily
4. Test changes before marking tasks complete
5. Exclude build artifacts from git commits
6. Ask for clarification if task scope is unclear
7. Reference specific files and line numbers in explanations

## Contributing Guidelines

- **Small changes**: Keep PRs focused and minimal
- **Testing**: Always test changes before submitting
- **Documentation**: Update docs if changing interfaces
- **Commit messages**: Be descriptive and clear
- **Build artifacts**: Never commit to repository
- **Code review**: Expect feedback on architecture decisions
