# wgpu Rust library

A native GPU rendering library for [bwindow](../bwindow), backed by Metal on
macOS, Vulkan on Linux, and Direct3D 12 on Windows.

Despite sharing its name and part of its API with upstream `wgpu`, this crate is
not API-compatible with it and is not a replacement. It currently supports the
features needed by this repository, including surfaces, buffers, textures, bind
groups, shaders, pipelines, and render passes.

## Backends

| Operating system | Backends               | Fallback                                                                                                |
| ---------------- | ---------------------- | ------------------------------------------------------------------------------------------------------- |
| Windows          | Direct3D 12 (fl 11_0+) | D3D12 hardware, then the D3D12 WARP software renderer                                                   |
| macOS            | Metal                  | None; Metal does not provide a software-rendering fallback                                              |
| Linux            | Vulkan 1.1+            | A software Vulkan implementation can be selected when one is installed and exposed by the Vulkan loader |

## Getting Started

Create a surface from a bwindow content attachment:

```rust,no_run
let window = bwindow::WindowBuilder::new().build();
let instance = wgpu::Instance::default();
let surface = instance
    .create_surface(window.attach_content().expect("attach graphics surface"))
    .expect("create surface");
let (width, height) = surface.size();
```

WGSL shaders are compiled at build time with
[wgpu-shader-build](../wgpu-shader-build) and embedded as Metal libraries,
SPIR-V, or Direct3D bytecode. Native GPU objects remain on their creating
thread, and surface creation must happen on the main thread on macOS.

## Examples

```sh
cargo run -p example-wgpu-triangle
cargo run -p example-wgpu-cube
cargo run -p example-wgpu-instanced
```

See [wgpu-triangle](../../examples/wgpu-triangle),
[wgpu-cube](../../examples/wgpu-cube), and
[wgpu-instanced](../../examples/wgpu-instanced).

## License

Copyright (c) 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
