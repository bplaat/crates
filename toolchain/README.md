# Toolchain

Everything the examples share to build Wii programs with plain clang and lld, without devkitPro.
`build.sh` builds it all in `target/`, the example Makefiles and Rust `build.rs` files run it for
you:

- Clones upstream [libogc](https://github.com/devkitPro/libogc) v2.11.0 and applies
  `libogc/patches/` as commits, then builds the parts we use (listed in `build.sh`) together
  with our tiny libc to `target/libogc.a`
- Clones devkitPro's [gamecube-tools](https://github.com/devkitPro/gamecube-tools) v1.0.7 and
  builds `elf2dol` (`.elf` to `.dol`) and `gxtexconv` (images to `.tpl` textures, needs FreeImage)
- Checks out only the builtins of LLVM's [compiler-rt](https://github.com/llvm/llvm-project/tree/main/compiler-rt/lib/builtins)
  llvmorg-23.1.3 and builds the 64-bit integer math and conversions the PowerPC 750 can't do in
  instructions to `target/libcompiler-rt.a`, the C examples link it (Rust has its own)

```sh
./build.sh                 # fetch, patch and build
./build.sh dev             # fetch and patch, then edit and commit in target/libogc/
./build.sh export-patches  # write the commits in target/libogc/ back to libogc/patches/
./build.sh clean           # remove target/
```

All code is compiled for the PowerPC 750 (`-mcpu=750`) and optimized for size with `-Os`, like
the Rust examples (`opt-level = "s"`). LTO doesn't work with libogc: it waits for interrupts in
loops on plain variables, which LTO can optimize into endless loops.

## Files

- `libc/` is a tiny C library that replaces newlib: word-sized `memcpy`/`memmove`/`memset` (zero
  fills use `dcbz` on cached memory), a first-fit `malloc` over the MEM1 and MEM2 arenas, float
  math accurate to about 1e-6, and a `printf` family without floats
- `wii.mk` has the make rules for the C examples, `wii.ld` is the linker script
- `wii.json` is the Rust target, each Rust example's `build.rs` links `target/libogc.a`
- `run.sh` converts a program to a `.dol` and starts it in Dolphin

## Patches

libogc needs newlib and GNU as, the patches make it build with our libc and LLVM:

- `0001` to `0008` remove the newlib glue: reent structs, devoptab, malloc locks, crtmain
- `0009` to `0011` add assembler macros for the Gekko / Broadway paired single instructions,
  which LLVM doesn't know, so the fast paired single matrix code keeps working
- `0012` and `0013` stop using small data sections, which clang doesn't support
- `0014` does the float math of the IR cursor and spotlights in single precision
