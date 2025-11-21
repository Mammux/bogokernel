#!/bin/bash
# c_hello/build.sh - Build hello.elf using libc

set -e

CC=riscv64-linux-gnu-gcc
LD=riscv64-linux-gnu-ld
CFLAGS="-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include"

echo "Building hello.elf with libc..."

# Compile hello.c
$CC $CFLAGS -c hello.c -o hello.o

# Link with libc.a
$LD -T linker.ld -o hello.elf hello.o ../libc/libc.a

echo "Build complete: hello.elf"
echo "To test: cp hello.elf ../kernel/hello.elf"
