@echo off
REM list/build.bat - Build lisp.elf using libc

set PATH=e:\sysgcc\risc-v\bin;%PATH%
set CC=riscv64-unknown-elf-gcc
set LD=riscv64-unknown-elf-ld
set CFLAGS=-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include

echo Building lisp.elf with libc...

REM Compile lisp.c
%CC% %CFLAGS% -c lisp.c -o lisp.o

REM Link with libc.a
%LD% -T linker.ld -o lisp.elf lisp.o ../libc/libc.a

echo Build complete: lisp.elf
copy lisp.elf ..\kernel\lisp.elf
