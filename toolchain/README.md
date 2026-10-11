# Toolchain

What the ports share to build Nintendo DS programs with plain clang and lld, without devkitPro.
`build.sh` builds it all in `target/`, the port build scripts run it for you:

- devkitPro's [calico](https://github.com/devkitPro/calico) v1.2.0,
  [libnds](https://github.com/devkitPro/libnds) v2.0.2,
  [libdvm](https://github.com/devkitPro/libdvm) v2.1.0 with
  [FatFs](https://github.com/devkitPro/fatfs-mod) v0.15.3 and the
  [default ARM7 program](https://github.com/devkitPro/default-arm7) v0.8.4, with their
  `<name>/patches/`, to `target/lib*.a` and `target/ds7.elf`
- our C library in `libc/` to `target/libc9.a` and `target/libc7.a`
- [ndstool](https://github.com/devkitPro/ndstool) v2.3.1 to `target/bin/ndstool`
- the builtins of LLVM's [compiler-rt](https://github.com/llvm/llvm-project/tree/main/compiler-rt/lib/builtins)
  llvmorg-23.1.3 (64-bit integer math, division and software floating point) to
  `target/libcompiler-rt9.a` and `target/libcompiler-rt7.a`

```sh
./build.sh                 # fetch, patch and build
./build.sh clean           # remove target/
./build.sh export-patches  # write the commits in target/<name>/ back to <name>/patches/
./build.sh test            # build test/ and run it in melonDS
```

The ARM9 (ARMv5TE) and ARM7 (ARMv4T) have no floating point unit. Code is Thumb, except objects
named `*.32.o` or `*.itcm.o` (a port's `ITCM_SOURCES`), which are ARM code in fast memory.

## Files

- `libc/` replaces newlib: word copying `memcpy`/`memset` in ITCM (VRAM safe), a first-fit
  `malloc`, buffered `stdio` on the devoptab devices (SD card, NitroFS, console), the `printf` and
  `scanf` families, `strtof`, float math, time and `setjmp`. Floats are single precision
  throughout, `strtod` and `atof` too, nothing does double precision math. `stdin` is always
  empty. It only uses the calico functions in `libc/calico.h`.
- `nds.mk` has the make rules for the ports' `Makefile.nds`
- `run.sh` starts a program in melonDS, `make run` uses it
- `test/` is a DS program that checks the startup code, the C library, compiler-rt, threads, ITCM,
  FAT and NitroFS. `./build.sh test` runs it in melonDS with an SD card in `test/target/sd/` and
  fails when a check fails or it doesn't finish.

## Patches

Edit and commit in `target/<name>/`, run `./build.sh` to try it and `./build.sh export-patches`
to write the commits back.

- calico: unified assembly syntax for LLVM, no newlib glue, linker scripts that work with lld
- libnds: builds with clang and without newlib, with a pregenerated `libversion.h`
- compiler-rt: the optimized float functions for ARMv5TE, divisions that return to Thumb on
  ARMv4T
