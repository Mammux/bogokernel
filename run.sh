cargo build -p userapp --release
cp target/riscv64gc-unknown-none-elf/release/userapp kernel/userapp.elf

cargo build -p cat --release
cp target/riscv64gc-unknown-none-elf/release/cat kernel/cat.elf

cargo build -Z build-std=core,alloc --target riscv64gc-unknown-none-elf -p kernel
cargo run --target riscv64gc-unknown-none-elf --bin kernel
