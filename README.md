# Wii Examples

Wii homebrew example programs written in both C and Rust.

## Getting Started

The examples build with a normal LLVM toolchain (clang, lld and llvm-ar). The [toolchain](toolchain/)
script clones upstream libogc, devkitPro's tools and LLVM's compiler-rt builtins, patches and builds
them the first time you build an example. Install LLVM, FreeImage (for the texture converter) and the
[Dolphin](https://dolphin-emu.org/) emulator:

```sh
# macOS
brew install llvm lld freeimage
brew install --cask dolphin

# Debian / Ubuntu
sudo apt install clang lld llvm make git libfreeimage-dev dolphin-emu
```

The Rust examples also need [rustup](https://rustup.rs/), it installs the nightly toolchain by itself.

Build and run a C example with make, the program ends up in `target/<name>.dol`:

```sh
cd triangles/c
make run
```

Build and run a Rust example with cargo, `cargo run` also writes `target/wii/release/<name>.dol`:

```sh
cd triangles/rust
cargo run --release
```

## Triangles

A minimal program that draws a few triangles using a display list.

## Canvas

2D rendering with PNG textures, bitmap font text and Wii Remote cursors.

## Blocks

Generates a random 64x64x64 voxel world and renders its visible block faces with GX display lists.
