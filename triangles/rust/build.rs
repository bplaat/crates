use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Build the toolchain and link its libogc.a
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let toolchain = manifest_dir.join("../../toolchain");
    let status = Command::new(toolchain.join("build.sh"))
        .status()
        .expect("failed to run build.sh");
    assert!(status.success(), "failed to build the toolchain");
    for path in ["build.sh", "libc", "libogc"] {
        println!("cargo:rerun-if-changed={}", toolchain.join(path).display());
    }
    println!(
        "cargo:rustc-link-search=native={}",
        toolchain.join("target").display()
    );
    println!("cargo:rustc-link-lib=static=ogc");
}
