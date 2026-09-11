/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Compiles the cube shader for the selected native backend.

fn main() -> wgpu_shader_build::Result<()> {
    wgpu_shader_build::Builder::new()
        .shader("mesh.wgsl", "src/mesh.wgsl")
        .shader("overlay.wgsl", "src/overlay.wgsl")
        .compile()
}
