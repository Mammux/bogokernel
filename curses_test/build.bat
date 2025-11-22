@echo off
REM curses_test/build.bat - Build curses_test.elf using libc

set PATH=e:\sysgcc\risc-v\bin;%PATH%
set CC=riscv64-unknown-elf-gcc
set LD=riscv64-unknown-elf-ld
set CFLAGS=-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include

echo Building curses_test.elf with libc...

REM Compile curses_test.c
%CC% %CFLAGS% -c curses_test.c -o curses_test.o

REM Link with libc.a
%LD% -T linker.ld -o curses_test.elf curses_test.o ../libc/libc.a

echo Build complete: curses_test.elf
copy curses_test.elf ..\kernel\curses_test.elf
