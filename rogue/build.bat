@echo off
REM rogue/build.bat - Build rogue.elf using libc

set PATH=e:\sysgcc\risc-v\bin;%PATH%
set CC=riscv64-unknown-elf-gcc
set LD=riscv64-unknown-elf-ld
set CFLAGS=-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include

echo Building rogue.elf with libc...

REM Compile rogue
%CC% %CFLAGS% -o rogue armor.c chase.c command.c daemon.c daemons.c extern.c fight.c init.c io.c list.c mach_dep.c main.c mdport.c misc.c monsters.c move.c new_level.c options.c pack.c passages.c potions.c rings.c rip.c rooms.c save.c scrolls.c state.c sticks.c things.c vers.c weapons.c wizard.c xcrypt.c

REM Link with libc.a
%LD% -T linker.ld -o rogue.elf rogue.o ../libc/libc.a

echo Build complete: rogue.elf
copy rogue.elf ..\kernel\rogue.elf
