# Minimal Libc for BogoKernel

A minimal C standard library for BogoKernel user programs.

## Features

- **Syscall wrappers**: All 12 BogoKernel syscalls (write, read, open, close, lseek, brk, exit, etc.)
- **String functions**: strlen, strcpy, strcmp, memcpy, memset, memcmp, etc.
- **Memory allocation**: malloc/free using brk syscall (simple bump allocator)
- **Standard I/O**: printf, puts, putchar with basic format support
- **POSIX compatibility**: Standard headers (unistd.h, string.h, stdlib.h, stdio.h)

## Building

Run `build.bat` to compile libc.a:

```batch
cd libc
build.bat
```

This creates `libc.a` which can be linked with C programs.

## Usage

Include the headers and link against libc.a:

```c
#include <stdio.h>
#include <string.h>

int main() {
    printf("Hello from libc!\n");
    return 0;
}
```

Compile and link:

```batch
riscv64-unknown-elf-gcc -c hello.c -o hello.o -Ilibc/include -ffreestanding -nostdlib
riscv64-unknown-elf-ld -T linker.ld -o hello.elf hello.o libc/libc.a
```

## Supported Functions

### unistd.h
- `write()`, `read()`, `open()`, `close()`, `lseek()`
- `_exit()`, `brk()`, `sbrk()`

### string.h
- `strlen()`, `strcpy()`, `strncpy()`, `strcmp()`, `strncmp()`
- `strchr()`, `strrchr()`
- `memcpy()`, `memmove()`, `memset()`, `memcmp()`

### stdlib.h
- `exit()`, `abort()`
- `malloc()`, `free()`, `calloc()`, `realloc()`
- `atoi()`, `atol()`

### stdio.h
- `printf()` - supports %d, %i, %u, %x, %s, %c, %%
- `puts()`, `putchar()`

## Implementation Notes

- **malloc**: Simple bump allocator using `sbrk()`, no free list
- **free**: No-op (memory not reclaimed)
- **printf**: Minimal implementation, no width/precision specifiers
- All syscalls return -1 on error (usize::MAX from kernel)
