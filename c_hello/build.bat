@echo off
set PATH=e:\sysgcc\risc-v\bin;%PATH%
riscv64-unknown-elf-gcc -c crt0.s -o crt0.o -march=rv64gc -mabi=lp64d
riscv64-unknown-elf-gcc -c syscalls.c -o syscalls.o -march=rv64gc -mabi=lp64d -ffreestanding -nostdlib
riscv64-unknown-elf-gcc -c hello.c -o hello.o -march=rv64gc -mabi=lp64d -ffreestanding -nostdlib
riscv64-unknown-elf-ld -T linker.ld -o hello.elf crt0.o syscalls.o hello.o
echo Build complete. Copy hello.elf to kernel/userapp.elf to run.
