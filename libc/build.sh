#!/bin/bash
# libc/build.sh - Build libc.a static library for Linux

set -e

CC=riscv64-linux-gnu-gcc
AR=riscv64-linux-gnu-ar
CFLAGS="-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -Iinclude"

echo "Building libc..."

# Compile source files
$CC $CFLAGS -c src/crt0.s -o src/crt0.o
$CC $CFLAGS -c src/syscall.c -o src/syscall.o
$CC $CFLAGS -c src/unistd.c -o src/unistd.o
$CC $CFLAGS -c src/string.c -o src/string.o
$CC $CFLAGS -c src/stdlib.c -o src/stdlib.o
$CC $CFLAGS -c src/stdio.c -o src/stdio.o
$CC $CFLAGS -c src/curses.c -o src/curses.o
$CC $CFLAGS -c src/signal.c -o src/signal.o
$CC $CFLAGS -c src/time.c -o src/time.o
$CC $CFLAGS -c src/stat.c -o src/stat.o
$CC $CFLAGS -c src/ctype.c -o src/ctype.o

# Create static library
$AR rcs libc.a src/crt0.o src/syscall.o src/unistd.o src/string.o src/stdlib.o src/stdio.o src/curses.o src/signal.o src/time.o src/stat.o src/ctype.o

echo "Build complete: libc.a"
