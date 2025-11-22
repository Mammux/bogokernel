@echo off
REM c_hello/build.bat - Build hello.elf using libc

set PATH=e:\sysgcc\risc-v\bin;%PATH%
set CC=riscv64-unknown-elf-gcc
set LD=riscv64-unknown-elf-ld
set CFLAGS=-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -I../libc/include

echo Building hello.elf with libc...

REM Compile hello.c
%CC% %CFLAGS% -c hello.c -o hello.o

REM Link with libc.a
%LD% -T linker.ld -o hello.elf hello.o ../libc/libc.a

echo Build complete: hello.elf
copy hello.elf ..\kernel\hello.elf
