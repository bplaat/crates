# WGPU instanced showcase

This example renders a 4x4 grid of 16 animated cubes into an offscreen texture,
then maps that texture onto a rotating cube. Each inner cube reads its transform,
rotation speed, and randomly shuffled unique block material from a read-only
storage buffer. Grass, cactus, and trunk blocks select distinct side, top, and
bottom texture-array layers.

The material textures are embedded with the local `rust-embed` crate, decoded
from PNG with the local `image` crate, uploaded as a 2D texture array, and
supplied with eight CPU-generated mip levels.

The 4x4 grid stays fixed while every block rotates at its own speed and the outer
cube rotates automatically. Press `Escape` to close the window.

Run the example with:

```sh
cargo run -p example-wgpu-instanced
```

Besides storage buffers and texture arrays, the example demonstrates uniform and
mapped vertex buffers, queue buffer and texture uploads, mip levels, samplers,
instanced draws, depth testing, back-face culling, render-to-texture, texture
sampling, alpha blending, viewports, scissor rectangles, and surface resizing.
