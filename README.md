# Wii Examples

Wii homebrew example programs written in both C and Rust.

## Getting Started

Install devkitPro and its pacman package manager, then add the
[libogc2 package repository](https://github.com/extremscorner/pacman-packages#adding-repository).
Then install the packages the examples need:

```sh
dkp-pacman -Syu
dkp-pacman -S devkitPPC libogc2
```

## Triangles

A minimal program that draws a few triangles using a display list.

## Canvas

2D rendering with PNG textures, bitmap font text and Wii Remote cursors.

## Blocks

Generates a random 64x64x64 voxel world and renders its visible block faces with GX display lists.
