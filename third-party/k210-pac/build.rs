use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    // Put the linker script somewhere the linker can find it
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out_dir.display());

    fs::File::create(out_dir.join("memory-k210.x"))
        .unwrap()
        .write_all(include_bytes!("memory-k210.x"))
        .unwrap();
    println!("cargo:rerun-if-changed=memory-k210.x");
}
