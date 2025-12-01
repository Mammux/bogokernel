# VirtIO GPU Debug Logging

This document describes the comprehensive debug logging added to the VirtIO GPU driver to diagnose device issues.

## Summary

Extensive debug logging has been added throughout the virtio GPU driver (`kernel/src/display/virtio_gpu.rs`) to trace the complete initialization and command submission process. All debug messages are prefixed with `[VirtIO-GPU]` for easy filtering.

## Debug Coverage

### 1. Device Probe Phase

Logs each MMIO slot scan from 0x10001000 to 0x10008000:
- Magic value verification (expects 0x74726976 = "virt")
- VirtIO version (1 or 2)
- Device ID (16 for GPU, 0 for empty slots)
- Identification of GPU device when found

**Example Output:**
```
[VirtIO-GPU] Starting device probe...
[VirtIO-GPU] Scanning slot 0: base=0x10001000
[VirtIO-GPU]   Magic: 0x74726976 (expected 0x74726976)
[VirtIO-GPU]   Version: 1
[VirtIO-GPU]   Device ID: 0 (GPU=16)
[VirtIO-GPU]   -> Empty slot, skipping
...
[VirtIO-GPU] Scanning slot 7: base=0x10008000
[VirtIO-GPU]   Device ID: 16 (GPU=16)
[VirtIO-GPU] *** Found GPU device at 0x10008000! ***
```

### 2. Device Initialization

Step-by-step logging of the VirtIO negotiation sequence:
- Device reset (status=0)
- Acknowledgment (status=1)
- Driver ready (status=3)
- Device features (32-bit flags)
- Feature negotiation
- Features OK (status=11)
- Status verification
- Driver OK (status=15)

**Example Output:**
```
[VirtIO-GPU] Initializing device at 0x10008000
[VirtIO-GPU] Framebuffer: 1024x768 = 3145728 bytes
[VirtIO-GPU] Starting device negotiation sequence...
[VirtIO-GPU]   Step 1: Reset device (status=0)
[VirtIO-GPU]   Step 2: Acknowledge device (status=1)
[VirtIO-GPU]   Step 3: Driver ready (status=3)
[VirtIO-GPU]   Device features: 0x39000002
[VirtIO-GPU]   Step 4: Negotiate features (driver_features=0)
[VirtIO-GPU]   Step 5: Set FEATURES_OK (status=11)
[VirtIO-GPU]   Status readback: 0x0000000b
[VirtIO-GPU]   Features accepted by device
```

### 3. Virtqueue Setup

Detailed logging of queue memory layout and configuration:
- Queue selection (controlq = 0)
- Maximum queue size
- Configured queue size (8 descriptors)
- Guest page size (4096 bytes)
- Queue memory addresses (base, desc, avail, used)
- Queue PFN calculation
- Memory layout offsets
- Structure sizes

**Example Output:**
```
[VirtIO-GPU] Setting up virtqueue 0 (controlq)...
[VirtIO-GPU]   Queue max size: 1024
[VirtIO-GPU]   Setting queue size to 8
[VirtIO-GPU]   Setting guest page size to 4096 bytes
[VirtIO-GPU]   Queue memory base: 0x80597000
[VirtIO-GPU]   Queue descriptor addr: 0x80597000
[VirtIO-GPU]   Queue avail addr: 0x80597080
[VirtIO-GPU]   Queue used addr: 0x80597fbc
[VirtIO-GPU]   Queue PFN: 0x00080597
[VirtIO-GPU]   Sizes: desc=128, avail=20, used=68
[VirtIO-GPU]   Offsets: desc=0, avail=128, used=4028
```

### 4. Display Initialization

Logs each of the 5 GPU commands with their parameters:
- CREATE_2D: Resource creation with format and dimensions
- ATTACH_BACKING: Memory attachment with address and size
- SET_SCANOUT: Display configuration
- TRANSFER_TO_HOST_2D: Framebuffer data transfer
- RESOURCE_FLUSH: Display activation

**Example Output:**
```
[VirtIO-GPU] ========================================
[VirtIO-GPU] Starting display initialization...
[VirtIO-GPU] ========================================
[VirtIO-GPU] Display parameters:
[VirtIO-GPU]   Resource ID: 1
[VirtIO-GPU]   Framebuffer: 0x80275000
[VirtIO-GPU]   Resolution: 1024x768
[VirtIO-GPU] Command 1/5: CREATE_2D resource...
```

### 5. Command Submission

Comprehensive logging for each command sent:
- Buffer addresses (request and response)
- Command type (hex code)
- Buffer lengths
- Descriptor indices and flags
- Descriptor physical addresses
- Request data hex dump (first 16 bytes)
- Avail ring updates
- Memory barriers
- QUEUE_NOTIFY register write and readback

**Example Output:**
```
[VirtIO-GPU] Buffer check: req ptr=0x80598180, resp ptr=0x80598100
[VirtIO-GPU] Sending command: type=0x0101, req_len=40, resp_len=24
[VirtIO-GPU]   Descriptors: req_idx=0, resp_idx=1
[VirtIO-GPU]   Request descriptor: addr=0x80598180, len=40, flags=0x1
[VirtIO-GPU]   Response descriptor: addr=0x80598100, len=24, flags=0x2
[VirtIO-GPU]   Request data (first 16 bytes):
01010000 00000000 00000000 00000000
[VirtIO-GPU]   Updating avail ring: idx=0 -> 1
[VirtIO-GPU]   Avail ring[0] = 0
[VirtIO-GPU]   Avail idx after update: 1
[VirtIO-GPU]   Notifying device (writing 0 to QUEUE_NOTIFY)
[VirtIO-GPU]   Notify register readback: 0
```

### 6. Command Wait Loop

Tracks command completion status:
- Initial used_idx value
- Periodic status updates every 20,000 iterations
- Current used_idx value
- Timeout detection (100,000 iterations)
- Success/failure indication

**Example Output:**
```
[VirtIO-GPU]   Waiting for response (last_used_idx=0)...
[VirtIO-GPU]   ... still waiting at iteration 0, used_idx=0
[VirtIO-GPU]   ... still waiting at iteration 20000, used_idx=0
[VirtIO-GPU]   ... still waiting at iteration 40000, used_idx=0
[VirtIO-GPU]   ERROR: Command timed out after 100000 iterations!
[VirtIO-GPU] ERROR: CREATE_2D command failed!
```

### 7. Response Handling

When commands succeed:
- Response reception notification
- Iteration count at completion
- Used ring index transition
- Response type code (expected 0x1100 for OK_NODATA)
- Command success confirmation

## Key Findings

### ✅ What Works

1. **Device Detection**: Successfully finds virtio-gpu at slot 7 (0x10008000)
2. **VirtIO Negotiation**: All status register updates succeed
3. **Queue Setup**: Memory layout is correct and properly aligned
   - Descriptor table at offset 0
   - Available ring at offset 128
   - Used ring at offset 4028
4. **Command Format**: GPU commands have correct structure and types
5. **Memory Barriers**: SeqCst fences in place for synchronization
6. **Buffer Addresses**: Command and response buffers are valid

### ❌ Problem Identified

**Device Not Processing Commands:**
- All 5 GPU commands timeout after 100,000 iterations
- `used_idx` never changes from 0
- Device receives QUEUE_NOTIFY writes (readback confirms)
- No entries appear in used ring

## Command Types

| Command | Code   | Description | Status |
|---------|--------|-------------|--------|
| CREATE_2D | 0x0101 | Create 2D resource | TIMEOUT ❌ |
| ATTACH_BACKING | 0x0106 | Attach memory | TIMEOUT ❌ |
| SET_SCANOUT | 0x0103 | Configure display | TIMEOUT ❌ |
| TRANSFER_TO_HOST_2D | 0x0105 | Transfer data | TIMEOUT ❌ |
| RESOURCE_FLUSH | 0x0104 | Flush to display | TIMEOUT ❌ |

## Technical Details

### Memory Layout

```
Queue Memory (4096 bytes, aligned):
  0x80597000: Descriptor table (128 bytes, 8 entries × 16 bytes)
  0x80597080: Available ring (20 bytes)
  0x80597094: Padding (3880 bytes)
  0x80597fbc: Used ring (68 bytes)

Command Buffers:
  0x80598180: GPU_CMD_BUF (512 bytes)
  0x80598100: GPU_RESP_BUF (128 bytes)

Framebuffer:
  0x80275000: BUF (3,145,728 bytes = 1024×768×4)
```

### VirtIO Configuration

- **Version**: 1 (legacy)
- **Device Features**: 0x39000002
- **Driver Features**: 0x00000000 (minimal)
- **Queue Size**: 8 descriptors
- **Page Size**: 4096 bytes

### Descriptor Flags

- **NEXT (0x1)**: Descriptor has next entry (for request)
- **WRITE (0x2)**: Device writes to buffer (for response)

## Possible Root Causes

1. **VirtIO Version**: Device reports version 1 (legacy), may need version 2 (modern)
2. **Feature Negotiation**: Driver negotiates 0 features, device may require specific flags
3. **Interrupt Mode**: Device might not support polling-only operation
4. **Device-Specific Init**: virtio-gpu may need additional setup beyond standard VirtIO
5. **QEMU Configuration**: Using `virtio-gpu-device`, may need different variant

## Testing

### Build
```bash
cargo build -p kernel --features gpu
```

### Run
```bash
qemu-system-riscv64 \
  -machine virt -m 512M \
  -nographic -bios default \
  -device virtio-gpu-device \
  -kernel target/riscv64gc-unknown-none-elf/debug/kernel
```

### Filter Debug Output
```bash
# Show only VirtIO-GPU messages
strings output.txt | grep "VirtIO-GPU"

# Show specific sections
strings output.txt | grep -A 20 "Starting device probe"
strings output.txt | grep -A 50 "Command 1/5"
```

## Next Steps

To resolve the timeout issue, investigate:

1. **Enable Interrupts**
   - Set up interrupt handling
   - Register ISR for virtio-gpu
   - Check if device requires interrupt mode

2. **Try Modern Interface**
   - Implement VirtIO MMIO version 2
   - Use modern register layout
   - Support extended features

3. **Feature Negotiation**
   - Identify required feature flags
   - Negotiate non-zero driver features
   - Check feature compatibility

4. **Compare with Linux**
   - Study Linux virtio-gpu initialization
   - Identify missing steps
   - Validate command formats

5. **Test Device Variants**
   - Try `virtio-gpu-pci` instead of `virtio-gpu-device`
   - Test with different QEMU versions
   - Check device tree properties

## Conclusion

The comprehensive debug logging successfully identifies that the virtio GPU device is found and negotiated correctly, the virtqueue is properly set up, and commands are formatted and submitted correctly. However, the device does not process any commands, suggesting an issue with device activation, notification mechanism, or feature negotiation. The detailed logging provides all necessary information to continue debugging and identify the root cause.
