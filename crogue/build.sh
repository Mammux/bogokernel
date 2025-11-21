#!/bin/bash
# crogue/build.sh - Build crogue.elf using libc

set -e

CC=riscv64-linux-gnu-gcc
LD=riscv64-linux-gnu-ld
CFLAGS="-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include"

echo "Building crogue.elf with libc..."

# Compile crogue.c
$CC $CFLAGS -c crogue.c -o crogue.o

# Link with libc.a
$LD -T linker.ld -o crogue.elf crogue.o ../libc/libc.a

echo "Build complete: crogue.elf"
echo "To test: cp crogue.elf ../kernel/crogue.elf"
