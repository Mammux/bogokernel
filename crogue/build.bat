@echo off
REM crogue/build.bat - Build crogue.elf using libc

set PATH=e:\sysgcc\risc-v\bin;%PATH%
set CC=riscv64-unknown-elf-gcc
set LD=riscv64-unknown-elf-ld
set CFLAGS=-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include

echo Building crogue.elf with libc...

REM Compile crogue.c
%CC% %CFLAGS% -c crogue.c -o crogue.o

REM Link with libc.a
%LD% -T linker.ld -o crogue.elf crogue.o ../libc/libc.a

echo Build complete: crogue.elf
echo Copy to kernel: copy crogue.elf ..\kernel\crogue.elf
