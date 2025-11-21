@echo off
rem Build and run automated tests

echo Building kernel and user applications...
cd userapp
cargo build --release --bin rogue >nul 2>&1
cargo build --release --bin shell >nul 2>&1
cd ..

copy target\riscv64gc-unknown-none-elf\release\rogue kernel\rogue.elf >nul
copy target\riscv64gc-unknown-none-elf\release\shell kernel\shell.elf >nul

if exist c_hello\hello.elf copy c_hello\hello.elf kernel\hello.elf >nul

cargo build -Z build-std=core,alloc --target riscv64gc-unknown-none-elf -p kernel >nul 2>&1

echo.
echo Running automated test...
echo.

powershell -ExecutionPolicy Bypass -File test.ps1
