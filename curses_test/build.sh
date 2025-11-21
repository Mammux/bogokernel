#!/bin/bash
# curses_test/build.sh - Build curses_test.elf using libc

set -e

CC=riscv64-linux-gnu-gcc
LD=riscv64-linux-gnu-ld
CFLAGS="-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include"

echo "Building curses_test.elf with libc..."

# Compile crt0.s (startup code)
$CC $CFLAGS -c crt0.s -o crt0.o

# Compile curses_test.c
$CC $CFLAGS -c curses_test.c -o curses_test.o

# Link with libc.a
$LD -T linker.ld -o curses_test.elf crt0.o curses_test.o ../libc/libc.a

echo "Build complete: curses_test.elf"
echo "To test: cp curses_test.elf ../kernel/curses_test.elf"
