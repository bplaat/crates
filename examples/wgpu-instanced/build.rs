/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Compiles the shaders.

fn main() -> wgpu_shader_build::Result<()> {
    wgpu_shader_build::Builder::new()
        .shader("instances.wgsl", "src/instances.wgsl")
        .shader("textured-cube.wgsl", "src/textured-cube.wgsl")
        .shader("overlay.wgsl", "src/overlay.wgsl")
        .compile()
}
