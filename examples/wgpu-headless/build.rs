/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Compiles the triangle shader for the selected native backend.

fn main() -> wgpu_shader_build::Result<()> {
    wgpu_shader_build::Builder::new()
        .shader("triangle.wgsl", "src/triangle.wgsl")
        .compile()
}
