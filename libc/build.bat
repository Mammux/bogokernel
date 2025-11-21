@echo off
REM libc/build.bat - Build libc.a static library

set PATH=e:\sysgcc\risc-v\bin;%PATH%
set CC=riscv64-unknown-elf-gcc
set AR=riscv64-unknown-elf-ar
set CFLAGS=-march=rv64gc -mabi=lp64d -ffreestanding -nostdlib -Iinclude

echo Building libc...

REM Compile source files
%CC% %CFLAGS% -c src/crt0.s -o src/crt0.o
%CC% %CFLAGS% -c src/syscall.c -o src/syscall.o
%CC% %CFLAGS% -c src/unistd.c -o src/unistd.o
%CC% %CFLAGS% -c src/string.c -o src/string.o
%CC% %CFLAGS% -c src/stdlib.c -o src/stdlib.o
%CC% %CFLAGS% -c src/stdio.c -o src/stdio.o

REM Create static library
%AR% rcs libc.a src/crt0.o src/syscall.o src/unistd.o src/string.o src/stdlib.o src/stdio.o

echo Build complete: libc.a
