// kernel/build.rs
use std::path::PathBuf;

fn main() {
    // Rebuild if the script changes
    println!("cargo:rerun-if-changed=memory.ld");

    // Absolute path to memory.ld (robust across cargo working dirs)
    let script = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("memory.ld");

    // Inject the linker arg for THIS crate’s binary
    println!("cargo:rustc-link-arg=-T{}", script.display());
    println!("cargo:rustc-link-arg=-Map=kernel.map");
}
