#!/bin/bash
# lisp/build.sh - Build lisp.elf using libc

set -e

CC=riscv64-linux-gnu-gcc
LD=riscv64-linux-gnu-ld
CFLAGS="-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include"

echo "Building lisp.elf with libc..."

# Compile lisp.c
$CC $CFLAGS -c lisp.c -o lisp.o

# Link with libc.a
$LD -T linker.ld -o lisp.elf lisp.o ../libc/libc.a

echo "Build complete: lisp.elf"
echo "To test: cp lisp.elf ../kernel/lisp.elf"
