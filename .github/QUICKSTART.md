# BogoKernel Quick Start Guide

Get BogoKernel up and running in 5 minutes.

## One-Line Setup (Ubuntu/Debian)

```bash
rustup target add riscv64gc-unknown-none-elf && \
sudo apt-get update && \
sudo apt-get install -y gcc-riscv64-linux-gnu qemu-system-riscv64
```

## Quick Build & Run

```bash
# Clone the repository (if not already done)
git clone https://github.com/Mammux/bogokernel.git
cd bogokernel

# Build everything
bash libc/build.sh
bash rogue/build.sh && cp rogue/rogue kernel/bigrogue.elf
bash crogue/build.sh && cp crogue/crogue.elf kernel/
bash curses_test/build.sh && cp curses_test/curses_test.elf kernel/
bash c_hello/build.sh && cp c_hello/hello.elf kernel/
cargo build -p cat --release && cp target/riscv64gc-unknown-none-elf/release/cat kernel/cat.elf
cargo build -p userapp --release && \
  cp target/riscv64gc-unknown-none-elf/release/shell kernel/shell.elf && \
  cp target/riscv64gc-unknown-none-elf/release/rogue kernel/rogue.elf
cargo build -p kernel

# Run
cargo run -p kernel
```

## What You'll See

```
riscv-os: hello from S-mode at 0x8020_0000!
trap stack initialized
traps enabled
timers initialized
SV39 paging enabled
Heap init OK.
Box value = 0xc0ffee
Vec sum = 140
Loaded shell: entry=0x10000, sp=0x40008000

Welcome to BogoShell!
Commands: hello, rogue, crogue, bigrogue, curses_test, shutdown, help
> _
```

## Try It Out

At the `>` prompt, try these commands:

```bash
> hello                # Hello world in C
> rogue                # Rogue game (Rust version)
> bigrogue             # Full Rogue game (C version)
> curses_test          # Curses library test
> shutdown             # Exit QEMU
```

### Playing Bigrogue

When you run `bigrogue`:
1. You'll see a dungeon map with your character (@)
2. Use keys: `h`(left), `j`(down), `k`(up), `l`(right) to move
3. Press `i` to view your inventory
4. Press `space` to close inventory (this is the bug we fixed!)
5. Press `q` then `y` to quit the game

## Exit QEMU

- From shell: Type `shutdown` command
- From QEMU console: Press `Ctrl-A` then `X`

## Common Issues

### "command not found: riscv64-linux-gnu-gcc"

```bash
sudo apt-get install gcc-riscv64-linux-gnu
```

### "target 'riscv64gc-unknown-none-elf' not found"

```bash
rustup target add riscv64gc-unknown-none-elf
```

### "qemu-system-riscv64: command not found"

```bash
sudo apt-get install qemu-system-riscv64
```

### Build fails with "No such file: kernel/bigrogue.elf"

You need to build all the .elf files first. See the build commands above.

### QEMU won't start or hangs

Make sure you're using the right kernel path:
```bash
qemu-system-riscv64 -machine virt -m 128M -nographic -bios default \
  -kernel target/riscv64gc-unknown-none-elf/debug/kernel
```

## File Locations

After building, you'll have:
- **Kernel binary**: `target/riscv64gc-unknown-none-elf/debug/kernel`
- **User apps**: `kernel/*.elf` (embedded in kernel at build time)
- **C library**: `libc/libc.a`

## Next Steps

- Read [PROJECT_STRUCTURE.md](.github/PROJECT_STRUCTURE.md) for architecture overview
- Read [BUILD_GUIDE.md](.github/BUILD_GUIDE.md) for detailed build instructions
- Read [CURSES_NOTES.md](.github/CURSES_NOTES.md) for curses library details
- Modify code and rebuild:
  - C library change → rebuild libc → rebuild C apps → rebuild kernel
  - C app change → rebuild that app → rebuild kernel
  - Rust app change → rebuild that app → rebuild kernel
  - Kernel change → rebuild kernel only

## Development Workflow

```bash
# 1. Make your changes
vim libc/src/curses.c

# 2. Rebuild affected components
cd libc && bash build.sh && cd ..
cd rogue && bash build.sh && cp rogue ../kernel/bigrogue.elf && cd ..

# 3. Rebuild kernel
cargo build -p kernel

# 4. Test
cargo run -p kernel
```

## Documentation

All documentation is in the `.github/` directory:
- `PROJECT_STRUCTURE.md` - Repository layout and architecture
- `BUILD_GUIDE.md` - Detailed build instructions
- `CURSES_NOTES.md` - Curses library implementation details
- `QUICKSTART.md` - This file

## Getting Help

The project structure is designed to be self-documenting:
1. Read the documentation in `.github/`
2. Look at existing code for patterns
3. Each directory has a `build.sh` for C applications
4. The `Cargo.toml` defines the workspace structure

## Project Statistics

- **Languages**: Rust (kernel + some apps), C (libc + most apps)
- **Lines of Code**: ~15,000+
- **Applications**: 7 user programs (shell, rogue games, utilities)
- **System Calls**: 12 different syscalls
- **Build Time**: ~1-2 minutes from scratch (debug build)

Enjoy exploring BogoKernel! 🚀
