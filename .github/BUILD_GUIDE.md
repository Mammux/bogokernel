# BogoKernel Build Guide

Complete guide for building BogoKernel from scratch.

## Prerequisites

### Install Required Tools

```bash
# Rust nightly toolchain
rustup toolchain install nightly
rustup target add riscv64gc-unknown-none-elf --toolchain nightly
rustup component add rust-src --toolchain nightly

# RISC-V GCC cross-compiler (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y gcc-riscv64-linux-gnu

# QEMU RISC-V emulator
sudo apt-get install -y qemu-system-riscv64
```

### Verify Installation

```bash
# Check Rust
rustc +nightly --version
rustup target list --installed | grep riscv64gc-unknown-none-elf

# Check GCC
riscv64-linux-gnu-gcc --version

# Check QEMU
qemu-system-riscv64 --version
```

## Build Process

The build process follows this order:
1. Build C library (libc.a)
2. Build C user applications
3. Build Rust user applications
4. Build kernel (which embeds all applications)

### Step 1: Build C Library

```bash
cd libc
bash build.sh
```

This creates `libc.a` which is used by all C applications.

**Output**: `libc/libc.a`

### Step 2: Build C User Applications

```bash
# Hello world example
cd c_hello
bash build.sh
cp hello.elf ../kernel/hello.elf

# Mini rogue game
cd ../crogue
bash build.sh
cp crogue.elf ../kernel/crogue.elf

# Curses test application
cd ../curses_test
bash build.sh
cp curses_test.elf ../kernel/curses_test.elf

# Full rogue game (bigrogue)
cd ../rogue
bash build.sh
cp rogue ../kernel/bigrogue.elf
```

**Output**: ELF files in `kernel/` directory

### Step 3: Build Rust User Applications

```bash
cd ..

# Build cat utility
cargo build -p cat --release
cp target/riscv64gc-unknown-none-elf/release/cat kernel/cat.elf

# Build shell and rogue (userapp package)
cargo build -p userapp --release
cp target/riscv64gc-unknown-none-elf/release/shell kernel/shell.elf
cp target/riscv64gc-unknown-none-elf/release/rogue kernel/rogue.elf
```

**Output**: More ELF files in `kernel/` directory

### Step 4: Build Kernel

```bash
# Build kernel (embeds all .elf files from kernel/ directory)
cargo build -p kernel

# Or for optimized build (may have issues with build-std)
cargo build -p kernel --release
```

**Output**: `target/riscv64gc-unknown-none-elf/debug/kernel`

## Quick Build (All Steps)

```bash
#!/bin/bash
# Build everything from scratch

set -e  # Exit on error

echo "Building libc..."
cd libc && bash build.sh && cd ..

echo "Building C applications..."
cd c_hello && bash build.sh && cp hello.elf ../kernel/ && cd ..
cd crogue && bash build.sh && cp crogue.elf ../kernel/ && cd ..
cd curses_test && bash build.sh && cp curses_test.elf ../kernel/ && cd ..
cd rogue && bash build.sh && cp rogue ../kernel/bigrogue.elf && cd ..

echo "Building Rust applications..."
cargo build -p cat --release
cargo build -p userapp --release
cp target/riscv64gc-unknown-none-elf/release/cat kernel/cat.elf
cp target/riscv64gc-unknown-none-elf/release/shell kernel/shell.elf
cp target/riscv64gc-unknown-none-elf/release/rogue kernel/rogue.elf

echo "Building kernel..."
cargo build -p kernel

echo "Build complete!"
echo "Kernel: target/riscv64gc-unknown-none-elf/debug/kernel"
```

## Running BogoKernel

### Using Cargo Runner

```bash
cargo run -p kernel
```

This automatically runs QEMU with the correct parameters (configured in `.cargo/config.toml`).

### Manual QEMU Command

```bash
qemu-system-riscv64 \
  -machine virt \
  -m 128M \
  -nographic \
  -bios default \
  -kernel target/riscv64gc-unknown-none-elf/debug/kernel
```

### Using Run Script

```bash
./run.sh
```

### Exit QEMU

Press `Ctrl-A` then `X` to exit QEMU console.

## Testing

### Automated Test

```bash
python3 test.py
```

This script:
1. Starts QEMU
2. Waits for shell prompt
3. Runs test commands
4. Captures output
5. Verifies expected strings
6. Saves output to `test_output.txt`

### Manual Testing

1. Start kernel: `cargo run -p kernel`
2. Wait for shell prompt: `> `
3. Available commands:
   - `hello` - Run hello world
   - `rogue` - Run Rust rogue game
   - `crogue` - Run C mini rogue
   - `bigrogue` - Run full rogue game
   - `curses_test` - Run curses test
   - `shutdown` - Shutdown system
   - `help` - Show available commands

## Incremental Builds

### Rebuilding After Changes

#### Modified C Library Code
```bash
cd libc && bash build.sh && cd ..
# Rebuild all C apps that depend on libc
cd rogue && bash build.sh && cp rogue ../kernel/bigrogue.elf && cd ..
# Rebuild kernel to pick up new .elf
cargo build -p kernel
```

#### Modified Rust User Application
```bash
cargo build -p userapp --release
cp target/riscv64gc-unknown-none-elf/release/shell kernel/shell.elf
cargo build -p kernel
```

#### Modified Kernel Code Only
```bash
cargo build -p kernel
```

## Troubleshooting

### Problem: `riscv64-linux-gnu-gcc: command not found`

**Solution**: Install GCC cross-compiler:
```bash
sudo apt-get install gcc-riscv64-linux-gnu
```

### Problem: `qemu-system-riscv64: command not found`

**Solution**: Install QEMU:
```bash
sudo apt-get install qemu-system-riscv64
```

### Problem: Rust target not found

**Solution**: Install target:
```bash
rustup target add riscv64gc-unknown-none-elf --toolchain nightly
```

### Problem: Build fails with "duplicate lang item"

This happens when using `-Z build-std=core` with certain dependencies.

**Solution**: Build without `-Z build-std`:
```bash
cargo build -p kernel
```

### Problem: Kernel can't find .elf files

The kernel embeds .elf files at compile time using `include_bytes!()` in `kernel/src/fs.rs`.

**Solution**: Ensure all .elf files exist in `kernel/` directory before building kernel:
```bash
ls kernel/*.elf
# Should show: bigrogue.elf, cat.elf, crogue.elf, curses_test.elf, hello.elf, rogue.elf, shell.elf
```

### Problem: Linker warnings about RWX segments

This is expected and not an error. RISC-V loader in kernel sets proper permissions.

## Clean Build

```bash
# Clean Rust builds
cargo clean

# Clean C builds
cd libc && rm -f *.o libc.a && cd ..
cd c_hello && rm -f *.o hello.elf && cd ..
cd crogue && rm -f *.o crogue.elf && cd ..
cd curses_test && rm -f *.o curses_test.elf && cd ..
cd rogue && rm -f *.o rogue && cd ..

# Clean kernel .elf files (optional - will need to rebuild all apps)
rm -f kernel/*.elf
```

## Build Artifacts to .gitignore

The following build artifacts should not be committed:
- `target/` - Rust build directory
- `libc/libc.a` - Compiled C library
- `libc/src/*.o` - C object files
- `c_hello/*.o`, `c_hello/hello.elf`
- `crogue/*.o`, `crogue/crogue.elf`
- `curses_test/*.o`, `curses_test/curses_test.elf`
- `rogue/*.o`, `rogue/rogue` - Note: `rogue` is the binary
- `kernel/*.elf` - Embedded application files
- `Cargo.lock` - May be regenerated

## Performance Notes

- Debug builds are much faster than release builds
- Kernel debug build is usually sufficient for development
- Release builds are only needed for performance testing
- C applications build quickly (<5 seconds each)
- Rust applications may take 10-30 seconds each
- Kernel build takes 5-15 seconds (after apps are built)

## CI/CD Considerations

For automated builds:
1. Cache `~/.cargo` and `target/` directories
2. Build in order: libc → C apps → Rust apps → kernel
3. Run automated tests with `test.py`
4. Archive kernel binary and test output

Example GitHub Actions workflow:
```yaml
- uses: actions/checkout@v4
- uses: dtolnay/rust-toolchain@nightly
- run: rustup target add riscv64gc-unknown-none-elf
- run: sudo apt-get install -y gcc-riscv64-linux-gnu qemu-system-riscv64
- run: bash libc/build.sh
- run: bash c_hello/build.sh && cp c_hello/hello.elf kernel/
- run: bash rogue/build.sh && cp rogue/rogue kernel/bigrogue.elf
- run: cargo build -p userapp --release
- run: cargo build -p kernel
- run: python3 test.py
```
