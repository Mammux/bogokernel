# BogoKernel

A tiny experimental operating system kernel written in Rust, targeting **RISC-V (rv64)** and running under **QEMU** with **OpenSBI**.  
This project is educational — it shows how to bring up a kernel in S-mode, set up paging (Sv39), and run user programs in U-mode loaded from ELF binaries.

---

## Features

- **Rust, no_std** kernel built with `cargo`.
- Runs in **Supervisor mode (S-mode)** on RISC-V.
- **Custom entry** (`_start`) with trap stack, vectored trap handling, and timer interrupts.
- **Sv39 paging** enabled with identity mapping for the kernel and U=1 mappings for user code/data.
- **Minimal heap** (via `linked_list_allocator`) to allow kernel allocations.
- **User programs**: ELF64 loader that maps PT_LOAD segments, lays out argv/envp on the user stack, and jumps to the ELF entry point in U-mode.
- **System calls**:  
  - `write(ptr,len)` → fd 1 (UART output)  
  - `write_cstr(ptr)` → print a NUL-terminated string  
  - `exit()` → return to kernel shell loop  
- Works under **QEMU virt machine** with `-bios default` (OpenSBI).

---

## Building and running

### Requirements

- **Rust nightly** with `riscv64gc-unknown-none-elf` target and `rust-src`:
  ```sh
  rustup toolchain install nightly
  rustup target add riscv64gc-unknown-none-elf --toolchain nightly
  rustup component add rust-src --toolchain nightly
  ```
    
- **QEMU with RISC-V support (qemu-system-riscv64).**

## Build steps

### Build the user program:

```
cargo build -p userapp --release
cp target/riscv64gc-unknown-none-elf/release/userapp kernel/userapp.elf
```

### Build the kernel (linking in userapp.elf):

```
cargo build -Z build-std=core --target riscv64gc-unknown-none-elf -p kernel
```

### Run in QEMU:

```
qemu-system-riscv64 \
  -machine virt -m 128M -nographic \
  -bios default \
  -kernel target/riscv64gc-unknown-none-elf/debug/kernel
```

### Expected output

```
riscv-os: hello from S-mode at 0x8020_0000!
trap stack initialized
traps enabled
timers initialized
SV39 paging enabled (identity map + UART)
Heap init OK.
Loaded user ELF: entry=..., sp=..., argc=3
userapp hello 42
TERM=xterm LANG=C
user program exited; back in S-mode.
tick 50
tick 100
...
```

## Roadmap

Some possible next steps for BogoKernel:

  -  Harden paging with correct section permissions.

  -  Add a real frame allocator.

  -  Abstract user programs as processes, add scheduler.

  -  Extend ELF loader with relocations and auxv.

  -  Expand syscall table (read, open, brk, mmap, …).

  -  Implement userland heap via brk/mmap.

  -  Add filesystem or virtio-blk driver.

  -  Improve trap/interrupt handling (PLIC, vectored).

  -  Prepare for SMP with per-hart stacks.

  -  Developer tooling: GDB scripts, xtask runner.