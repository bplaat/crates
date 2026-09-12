/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

struct Scene {
    settings: vec4<f32>,
}

@group(0) @binding(0) var<uniform> scene: Scene;
@group(0) @binding(1) var rendered_scene: texture_2d<f32>;
@group(0) @binding(2) var rendered_scene_sampler: sampler;

struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) shade: f32,
}

fn rotate(point: vec3<f32>, angles: vec2<f32>) -> vec3<f32> {
    let x = vec3(point.x, point.y*cos(angles.x)-point.z*sin(angles.x),
        point.y*sin(angles.x)+point.z*cos(angles.x));
    return vec3(x.x*cos(angles.y)+x.z*sin(angles.y), x.y,
        -x.x*sin(angles.y)+x.z*cos(angles.y));
}

@vertex fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> Out {
    let angles = vec2(scene.settings.x * 0.43, scene.settings.x * 0.31);
    let rotated = rotate(position, angles);
    let point = rotated - vec3(0.0, 0.0, 4.3);
    let focal_length = 2.41421356;

    var out: Out;
    out.position = vec4(point.x*focal_length/scene.settings.y, point.y*focal_length,
        (-100.0*point.z-10.0)/99.9, -point.z);
    out.uv = uv;
    out.shade = 0.35 + 0.65 * max(dot(rotate(normal, angles),
        normalize(vec3(0.4, 0.8, 0.6))), 0.0);
    return out;
}

@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {
    let color = textureSample(rendered_scene, rendered_scene_sampler,
        in.uv * 0.94 + vec2(0.03));
    return vec4(color.rgb * in.shade, 1.0);
}
