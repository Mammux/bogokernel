# GPU Test Application Instructions

This document describes how to test the new `gputest` user application that displays colored patterns on the virtio GPU framebuffer.

## Overview

The `gputest` application demonstrates framebuffer access from userspace by:
1. Using the new `GET_FB_INFO` syscall to get framebuffer information
2. Mapping the framebuffer into user address space
3. Drawing a test pattern directly to the framebuffer memory

## New Syscall: GET_FB_INFO

**Syscall Number**: 19  
**Signature**: `get_fb_info(buf: *mut FbInfo) -> Result<(), ()>`

**FbInfo Structure**:
```rust
#[repr(C)]
pub struct FbInfo {
    pub width: usize,   // Width in pixels
    pub height: usize,  // Height in pixels
    pub stride: usize,  // Bytes per row
    pub addr: usize,    // User-space virtual address
}
```

The syscall:
- Returns framebuffer dimensions and stride
- Automatically maps the framebuffer into user address space at 0x30000000
- Returns the user VA in the `addr` field

## Building

1. Build the gputest application:
```bash
cargo build -p userapp --release --bin gputest
cp target/riscv64gc-unknown-none-elf/release/gputest kernel/gputest.elf
```

2. Build the kernel with GPU support:
```bash
cargo build -p kernel --features gpu
```

The `gputest.elf` file is automatically embedded in the kernel's filesystem.

## Running

### With VirtIO GPU Device

Run QEMU with the virtio-gpu device:

```bash
qemu-system-riscv64 \
  -machine virt -m 512M \
  -nographic \
  -bios default \
  -kernel target/riscv64gc-unknown-none-elf/debug/kernel \
  -device virtio-gpu-device \
  -serial stdio
```

**Note**: The `-device virtio-gpu-device` parameter is required for the GPU to be available.

### From Shell

Once the kernel boots and you see the shell prompt:

```
shell> gputest.elf
```

### Expected Output

The application will print:
```
GPU Test Application
====================
Framebuffer info:
  Width:  1024 pixels
  Height: 768 pixels
  Stride: 4096 bytes
  Address: 0x30000000

Drawing test pattern...
Test pattern drawn successfully!
You should see 8 colored horizontal bars with a white square in the center.
```

### Visual Output

The framebuffer should display:
- **8 horizontal colored bars** (from top to bottom):
  1. Red
  2. Green
  3. Blue
  4. Yellow
  5. Magenta
  6. Cyan
  7. White
  8. Black
- **White square** (100x100 pixels) in the center

**Note**: To see the visual output, you need to run QEMU with a graphical display instead of `-nographic`. However, for CI testing, the text output confirms that the framebuffer syscall and memory mapping works correctly.

## Without GPU Device

If you run the kernel without `-device virtio-gpu-device`, the application will output:

```
GPU Test Application
====================
Error: Failed to get framebuffer info
Make sure the kernel was built with GPU support.
```

This is expected behavior - the GPU device must be present for the syscall to succeed.

## Implementation Details

### Memory Mapping

The framebuffer is mapped into user space as follows:
- **Physical Address**: Kernel framebuffer static buffer (e.g., 0x80400000)
- **User Virtual Address**: 0x30000000 (high in user address space)
- **Mapping Flags**: `URW` (User Read-Write)
- **Page Size**: 4K pages

The mapping is done automatically by the `GET_FB_INFO` syscall using the `map_framebuffer_to_user` function in `kernel/src/sv39.rs`.

### Pixel Format

The framebuffer uses **XRGB8888** format (32 bits per pixel):
- Byte 0: Blue
- Byte 1: Green
- Byte 2: Red
- Byte 3: Unused (padding)

Example color values:
```rust
0x00FF0000  // Red
0x0000FF00  // Green
0x000000FF  // Blue
0x00FFFFFF  // White
0x00000000  // Black
```

### Drawing Algorithm

The test pattern is drawn by:
1. Dividing the screen height into 8 equal bars
2. Filling each bar with a different color
3. Overwriting the center region with white

```rust
for y in 0..fb_info.height {
    let color_idx = (y / bar_height).min(7);
    let color = colors[color_idx];
    
    for x in 0..fb_info.width {
        let idx = y * fb_info.width + x;
        fb_slice[idx] = color;
    }
}
```

## Testing in CI

Since the CI environment doesn't have QEMU installed, testing is limited to:
1. ✅ Code compilation
2. ✅ Syscall implementation correctness (syntax)
3. ⚠️  Runtime behavior (requires QEMU)

For manual testing, run the commands above in an environment with QEMU installed.

## Troubleshooting

### "Failed to get framebuffer info"
- Ensure kernel was built with `--features gpu`
- Ensure QEMU was started with `-device virtio-gpu-device`
- Check kernel logs for VirtIO GPU initialization messages

### No visual output
- QEMU must have a display backend (not `-nographic` for graphics)
- Try adding `-display gtk` or `-display sdl` to QEMU command
- Ensure host system has graphics support

### Page fault
- Check that framebuffer VA (0x30000000) doesn't conflict with other mappings
- Verify page table entries are created correctly
- Check kernel logs for mapping failures

## Security Notes

- The framebuffer is mapped with user read-write permissions
- Only the actual framebuffer memory is exposed (not arbitrary kernel memory)
- The mapping is per-process (cleared on program exit)
- No validation is done on pixel values (direct memory writes)

## Future Enhancements

Possible improvements:
- Add `UNMAP_FB` syscall to explicitly unmap the framebuffer
- Support multiple framebuffers
- Add ioctl interface for display control
- Implement double-buffering
- Add vsync support
- Support different pixel formats

## Files Modified

- `uapi/src/lib.rs` - Added GET_FB_INFO syscall number
- `usys/src/lib.rs` - Added syscall wrapper and FbInfo struct
- `kernel/src/trap.rs` - Implemented sys_get_fb_info handler
- `kernel/src/sv39.rs` - Added map_framebuffer_to_user function
- `kernel/src/fs.rs` - Added gputest.elf to embedded filesystem
- `userapp/src/bin/gputest.rs` - New GPU test application
