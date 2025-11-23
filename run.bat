@echo off
rem Build libc
cd libc
call build.bat
cd ..

rem Build User Apps
cd userapp
cargo build --release --bin rogue
cargo build --release --bin shell
cargo build --release --bin fstest
cargo build --release --bin mkfiles
cd ..
cd rogue
call build.bat
cd ..
cd crogue
call build.bat
cd ..
cd c_hello
call build.bat
cd ..

rem Copy binaries
copy target\riscv64gc-unknown-none-elf\release\shell kernel\shell.elf
copy target\riscv64gc-unknown-none-elf\release\rogue kernel\rogue.elf
copy target\riscv64gc-unknown-none-elf\release\fstest kernel\fstest.elf
copy target\riscv64gc-unknown-none-elf\release\mkfiles kernel\mkfiles.elf

rem Build C App (if compiler exists, otherwise assume hello.elf exists or skip)
if exist c_hello\hello.elf copy c_hello\hello.elf kernel\hello.elf

rem Build Kernel
cargo build -Z build-std=core,alloc --target riscv64gc-unknown-none-elf -p kernel
cargo run --target riscv64gc-unknown-none-elf --bin kernel
