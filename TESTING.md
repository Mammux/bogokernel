# Testing Guide for BogoKernel

## Overview

This document describes the testing strategy for BogoKernel, a `no_std` RISC-V operating system kernel.

## Testing Challenges

BogoKernel presents unique testing challenges:

1. **`no_std` Environment**: The kernel does not link to the standard library, making traditional unit testing difficult
2. **Hardware Dependencies**: Many kernel functions require RISC-V hardware features (CSRs, paging, interrupts)
3. **Binary Target**: The kernel is compiled as a bare-metal binary with `test = false` in Cargo.toml
4. **No Panic Handler**: Test infrastructure requires a panic handler and allocator that conflict with kernel setup

## Testing Approach

### 1. Inline Test Documentation (Current)

Test code has been added directly to kernel modules using `#[cfg(test)]` blocks. While these tests cannot be executed with `cargo test` due to the `no_std` environment, they serve important purposes:

- **Documentation**: Tests document expected behavior and usage patterns
- **Design Validation**: Writing tests helps identify and validate API design
- **Future Refactoring**: Tests can be extracted to a std environment if modules are refactored

#### Modules with Test Coverage

**`kernel/src/fs.rs`** (24 tests, ~280 lines):
- File creation and truncation
- Reading and writing files at various offsets
- File metadata (size, permissions, existence)
- Multiple file operations
- Error handling for invalid indices

**`kernel/src/sv39.rs`** (10 tests, ~103 lines):
- PPN (Physical Page Number) calculations
- VPN (Virtual Page Number) index extraction
- PTE (Page Table Entry) flag validation
- Memory layout constants
- Page size constants
- Address space validation

**`kernel/src/elf.rs`** (11 tests, ~118 lines):
- ELF p_flags to PTE flags conversion
- Permission mapping (R/W/X combinations)
- ELF constants validation
- Loaded struct field access
- Error type distinctness

### 2. Integration Testing

**`test.py`**: End-to-end integration test that:
- Starts QEMU with the kernel
- Waits for shell prompt
- Executes commands (hello, shutdown)
- Validates expected output
- Saves test results

```bash
python3 test.py
```

### 3. Manual Testing

```bash
# Build and run kernel
cargo run -p kernel

# Test individual user programs
# In QEMU shell:
> hello
> rogue
> cat dungeon.map
> ls
```

## Test Statistics

| Module | Test Functions | Lines of Test Code | Coverage |
|--------|---------------|-------------------|----------|
| fs.rs | 24 | ~280 | Filesystem operations |
| sv39.rs | 10 | ~103 | Paging calculations |
| elf.rs | 11 | ~118 | ELF validation |
| **Total** | **45** | **~501** | **Core logic** |

## Running Tests

### Unit Tests

Unit tests can now be executed using `cargo test`:

```bash
# Run all unit tests (21 tests)
cargo test -p kernel --lib --target x86_64-unknown-linux-gnu

# Run tests for a specific module
cargo test -p kernel --lib --target x86_64-unknown-linux-gnu sv39
cargo test -p kernel --lib --target x86_64-unknown-linux-gnu elf
cargo test -p kernel --lib --target x86_64-unknown-linux-gnu fs
```

**How it works**: Pure functions are extracted into a separate library (`kernel/src/lib.rs`) that compiles with `std` for testing on the host platform. The kernel binary (`kernel/src/main.rs`) remains `no_std` and compiles for RISC-V.

### Integration Tests

```bash
# Run integration test
python3 test.py

# Check output
cat test_output.txt
```

### Build Verification

```bash
# Verify kernel compiles
cargo check -p kernel

# Build kernel
cargo build -p kernel

# Run kernel in QEMU
cargo run -p kernel
```

## Test Code Examples

### Filesystem Tests

```rust
#[test]
fn test_write_and_read_file() {
    reset_fs();
    
    let idx = create_file("test.txt").unwrap();
    let data = b"Hello, World!";
    
    // Write data
    let written = write_file(idx, 0, data).unwrap();
    assert_eq!(written, data.len());
    
    // Read data back
    let mut buf = vec![0u8; data.len()];
    let read = read_file(idx, 0, &mut buf).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf[..], data);
}
```

### SV39 Paging Tests

```rust
#[test]
fn test_vpn_indices_simple() {
    // Test VPN extraction for simple addresses
    // VA = 0x0000_0000: all VPNs should be 0
    let indices = vpn_indices(0x0000_0000);
    assert_eq!(indices, [0, 0, 0]);
    
    // VA = 0x0000_1000: VPN[0] = 1, others = 0
    let indices = vpn_indices(0x0000_1000);
    assert_eq!(indices, [1, 0, 0]);
}
```

### ELF Loader Tests

```rust
#[test]
fn test_pte_flags_from_pf_all_perms() {
    // All permissions (PF_R | PF_W | PF_X = 0x7)
    let flags = pte_flags_from_pf(0x7);
    assert_eq!(flags & PTE_R, PTE_R);
    assert_eq!(flags & PTE_W, PTE_W);
    assert_eq!(flags & PTE_X, PTE_X);
    assert_eq!(flags & PTE_D, PTE_D);
}
```

## Future Testing Improvements

### Option 1: Library Extraction

Refactor kernel modules to separate hardware-dependent code from pure logic:

```
kernel/
  ├── src/
  │   ├── lib.rs           # Pure logic (testable)
  │   ├── hw.rs            # Hardware access (not testable)
  │   └── main.rs          # Integration
  └── tests/
      └── logic_tests.rs   # Unit tests for lib.rs
```

### Option 2: QEMU-based Testing

Create automated tests that run in QEMU:

```rust
#[test]
fn test_kernel_boots() {
    let mut qemu = Qemu::new()
        .kernel("target/.../kernel")
        .expect_output("Welcome to BogoShell")
        .run();
    assert!(qemu.wait_for_prompt(Duration::from_secs(10)));
}
```

### Option 3: Mock Hardware

Create mock implementations of hardware interfaces for unit testing:

```rust
trait PageTable {
    fn map(&mut self, va: usize, pa: usize, flags: u64);
}

struct RealPageTable { /* hardware */ }
struct MockPageTable { /* hashmap */ }

#[cfg(test)]
fn test_with_mock() {
    let mut pt = MockPageTable::new();
    // Test logic without hardware
}
```

## Testing Best Practices

1. **Document Behavior**: Even if tests can't run, they document expected behavior
2. **Test Pure Functions**: Focus on functions without hardware dependencies
3. **Integration First**: Verify end-to-end functionality before diving into units
4. **Manual Validation**: Always test interactive features manually
5. **Build Verification**: Ensure code compiles after changes

## Adding New Tests

When adding new functionality:

1. **Write test-like documentation** in `#[cfg(test)]` blocks
2. **Add integration test cases** to `test.py`
3. **Manual test procedures** in commit messages
4. **Update this document** with new test counts

## Test Coverage Goals

- [x] Filesystem operations (24 tests)
- [x] Page table calculations (10 tests)
- [x] ELF validation (11 tests)
- [x] Integration testing (test.py)
- [ ] Trap handling logic
- [ ] Timer management
- [ ] UART operations
- [ ] Syscall validation
- [ ] User program loading

## Continuous Integration

Currently, CI should:

1. Build kernel successfully
2. Run integration tests with QEMU
3. Verify no regressions in boot process

Future CI improvements:
- Run extracted unit tests in std environment
- Performance regression testing
- Code coverage reporting (where applicable)

## Conclusion

While BogoKernel's `no_std` nature prevents traditional unit testing, the combination of:
- Inline test documentation (45 tests, 501 lines)
- Integration testing (test.py)
- Manual testing procedures

Provides a solid foundation for maintaining code quality and preventing regressions. The test code added to kernel modules serves as both documentation and a template for future testing infrastructure.
