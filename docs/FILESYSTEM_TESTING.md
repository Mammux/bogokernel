# Writable Filesystem Testing Guide

## Overview
The writable filesystem implementation adds support for creating, modifying, and deleting files in BogoKernel. This enables applications like rogue to save game state.

## New Syscalls

### 14. CREAT
- **Signature**: `creat(path, mode) -> fd`
- **Description**: Create a new file or truncate an existing one
- **Returns**: File descriptor on success, -1 on error

### 15. UNLINK
- **Signature**: `unlink(path) -> result`
- **Description**: Delete a file
- **Returns**: 0 on success, -1 on error

### 16. STAT
- **Signature**: `stat(path, buf) -> result`
- **Description**: Get file metadata (size, mode)
- **Returns**: 0 on success, -1 on error
- **Buffer Format**: `[size: u64, mode: u64]`

### 17. CHMOD
- **Signature**: `chmod(path, mode) -> result`
- **Description**: Change file permissions
- **Returns**: 0 on success, -1 on error

## Testing

### Build the System

1. Build user applications:
```bash
cargo build -p userapp --release --bin shell
cargo build -p userapp --release --bin fstest
cargo build -p cat --release
```

2. Copy binaries to kernel directory:
```bash
cp target/riscv64gc-unknown-none-elf/release/shell kernel/shell.elf
cp target/riscv64gc-unknown-none-elf/release/fstest kernel/fstest.elf
cp target/riscv64gc-unknown-none-elf/release/cat kernel/cat.elf
```

3. Build the kernel:
```bash
cargo build -Z build-std=core,alloc --target riscv64gc-unknown-none-elf -p kernel
```

### Run in QEMU

```bash
qemu-system-riscv64 \
  -machine virt -m 128M -nographic \
  -bios default \
  -kernel target/riscv64gc-unknown-none-elf/debug/kernel
```

### Test Filesystem

At the shell prompt, run the filesystem test:
```
shell> exec fstest.elf
```

Expected output:
```
=== Filesystem Test ===

[Test 1] Creating test.txt...
✓ Created test.txt (fd=3)

[Test 2] Writing to test.txt...
✓ Wrote 28 bytes

[Test 3] Reading test.txt...
✓ Opened test.txt (fd=3)
✓ Read 28 bytes:
  Content: Hello, writable filesystem!

[Test 4] Checking file stats...
✓ File exists
  Size: 28 bytes
  Mode: 0644

[Test 5] Changing permissions...
✓ chmod() succeeded
  New mode: 0400

[Test 6] Deleting test.txt...
✓ File deleted
✓ File no longer exists

[Test 7] Multiple writes...
✓ Multiple writes successful:
Line 1
Line 2
Line 3

=== All tests complete ===
```

### Test with Rogue

1. Build or copy rogue executable:
```bash
# If building from source with RISC-V toolchain:
cd rogue && ./build.sh
cp rogue ../kernel/rogue.elf
```

2. Rebuild kernel and run
3. At shell prompt: `exec rogue.elf`
4. Play the game and use the save command (typically 'S')
5. Exit and restart rogue - saved game should be loadable

## Implementation Details

### Kernel Side
- **fs.rs**: Added `WritableFile` struct and `Vec<WritableFile>` storage
- **trap.rs**: Added syscall handlers for creat, unlink, stat, chmod
- **trap.rs**: Modified FdEntry to track read-only vs writable files
- **trap.rs**: Updated sys_read, sys_write_fd, sys_lseek for both file types

### User Space
- **uapi**: Added syscall numbers 14-17
- **usys**: Added Rust syscall wrappers
- **libc**: Updated syscall.c, unistd.c, stat.c, stdio.c
- **libc**: Implemented fopen/fclose/fread/fwrite/putc for real files

### File Storage
- Files are stored in kernel memory as `Vec<u8>`
- No persistence across reboots (in-memory only)
- Files can grow dynamically as data is written
- Maximum size limited by available kernel heap

## Limitations

1. **No Persistence**: Files are lost on reboot (RAM-based)
2. **No Directories**: Flat namespace (all files in root)
3. **No File Locking**: No concurrent access protection
4. **Limited Metadata**: Only size and mode tracked
5. **No Sparse Files**: All bytes allocated on write

## Future Enhancements

- Add directory support
- Implement file persistence (virtio-blk driver)
- Add timestamps (atime, mtime, ctime)
- Implement file locking mechanisms
- Add proper permission checking
- Support symbolic links
