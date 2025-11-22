#!/bin/bash
# rogue/build.sh - Build rogue.elf using libc

set -e

CC=riscv64-linux-gnu-gcc
LD=riscv64-linux-gnu-ld
CFLAGS="-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include"

echo "Building rogue.elf with libc..."

# Compile crt0.s (startup code)
$CC $CFLAGS -c crt0.s -o crt0.o

# Compile all c files
$CC $CFLAGS -o rogue armor.c chase.c command.c daemon.c daemons.c extern.c fight.c init.c io.c list.c mach_dep.c main.c mdport.c misc.c monsters.c move.c new_level.c options.c pack.c passages.c potions.c rings.c rip.c rooms.c save.c scrolls.c state.c sticks.c things.c vers.c weapons.c wizard.c xcrypt.c

# Link with libc.a
$LD -T linker.ld -o rogue rogue.o crt0.o ../libc/libc.a

echo "Build complete: rogue"
echo "To test: cp rogue.elf ../kernel/rogue.elf"
