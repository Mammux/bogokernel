@echo off
REM rogue/build.bat - Build rogue.elf using libc

set PATH=e:\sysgcc\risc-v\bin;%PATH%
set CC=riscv64-unknown-elf-gcc
set LD=riscv64-unknown-elf-ld
set CFLAGS=-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -nostdinc -I../libc/include

echo Building rogue.elf with libc...

REM Compile crt0.s (startup code)
%CC% %CFLAGS% -c crt0.s -o crt0.o

REM Compile rogue
%CC% %CFLAGS% -c armor.c -o armor.o
%CC% %CFLAGS% -c chase.c -o chase.o
%CC% %CFLAGS% -c command.c -o command.o
%CC% %CFLAGS% -c daemon.c -o daemon.o
%CC% %CFLAGS% -c daemons.c -o daemons.o
%CC% %CFLAGS% -c extern.c -o extern.o
%CC% %CFLAGS% -c fight.c -o fight.o
%CC% %CFLAGS% -c init.c -o init.o
%CC% %CFLAGS% -c io.c -o io.o
%CC% %CFLAGS% -c list.c -o list.o
%CC% %CFLAGS% -c mach_dep.c -o mach_dep.o
%CC% %CFLAGS% -c main.c -o main.o
%CC% %CFLAGS% -c mdport.c -o mdport.o
%CC% %CFLAGS% -c misc.c -o misc.o
%CC% %CFLAGS% -c monsters.c -o monsters.o
%CC% %CFLAGS% -c move.c -o move.o
%CC% %CFLAGS% -c new_level.c -o new_level.o
%CC% %CFLAGS% -c options.c -o options.o
%CC% %CFLAGS% -c pack.c -o pack.o
%CC% %CFLAGS% -c passages.c -o passages.o
%CC% %CFLAGS% -c potions.c -o potions.o
%CC% %CFLAGS% -c rings.c -o rings.o
%CC% %CFLAGS% -c rip.c -o rip.o
%CC% %CFLAGS% -c rooms.c -o rooms.o
%CC% %CFLAGS% -c save.c -o save.o
%CC% %CFLAGS% -c scrolls.c -o scrolls.o
%CC% %CFLAGS% -c state.c -o state.o
%CC% %CFLAGS% -c sticks.c -o sticks.o
%CC% %CFLAGS% -c things.c -o things.o
%CC% %CFLAGS% -c vers.c -o vers.o
%CC% %CFLAGS% -c weapons.c -o weapons.o
%CC% %CFLAGS% -c wizard.c -o wizard.o
%CC% %CFLAGS% -c xcrypt.c -o xcrypt.o

REM Link with libc.a
%LD% -T linker.ld -o rogue.elf crt0.o armor.o chase.o command.o daemon.o daemons.o extern.o fight.o init.o io.o list.o mach_dep.o main.o mdport.o misc.o monsters.o move.o new_level.o options.o pack.o passages.o potions.o rings.o rip.o rooms.o save.o scrolls.o state.o sticks.o things.o vers.o weapons.o wizard.o xcrypt.o ../libc/libc.a

echo Build complete: rogue.elf
copy rogue.elf ..\kernel\bigrogue.elf
