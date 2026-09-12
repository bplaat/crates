/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

struct Scene {
    settings: vec4<f32>,
    camera: vec4<f32>,
}

struct Instance {
    position_scale: vec4<f32>,
    rotation: vec4<f32>,
    material: vec4<u32>,
}

@group(0) @binding(0) var<uniform> scene: Scene;
@group(0) @binding(1) var materials: texture_2d_array<f32>;
@group(0) @binding(2) var material_sampler: sampler;
@group(0) @binding(3) var<storage, read> instances: array<Instance>;

struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) shade: f32,
    @interpolate(flat) @location(2) material: u32,
}

fn rotate(point: vec3<f32>, angles: vec2<f32>) -> vec3<f32> {
    let x = vec3(point.x, point.y*cos(angles.x)-point.z*sin(angles.x),
        point.y*sin(angles.x)+point.z*cos(angles.x));
    return vec3(x.x*cos(angles.y)+x.z*sin(angles.y), x.y,
        -x.x*sin(angles.y)+x.z*cos(angles.y));
}

@vertex fn vs_main(
    @builtin(instance_index) instance: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> Out {
    let data = instances[instance];
    let angles = data.rotation.xy + scene.settings.x * data.rotation.zw;
    let model_position = rotate(position * data.position_scale.w, angles) +
        data.position_scale.xyz;
    let view_angles = vec2(scene.settings.w, scene.settings.z);
    let point = rotate(model_position, view_angles) - vec3(0.0, 0.0, scene.camera.x);
    let focal_length = 2.41421356;

    var out: Out;
    out.position = vec4(point.x*focal_length/scene.settings.y, point.y*focal_length,
        (-100.0*point.z-10.0)/99.9, -point.z);
    out.uv = uv;
    out.shade = 0.35 + 0.65 * max(dot(rotate(rotate(normal, angles), view_angles),
        normalize(vec3(0.4, 0.8, 0.6))), 0.0);
    out.material = data.material.x;
    if normal.y > 0.5 {
        out.material = data.material.y;
    } else if normal.y < -0.5 {
        out.material = data.material.z;
    }
    return out;
}

@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {
    let texel = textureSample(materials, material_sampler, in.uv, i32(in.material));
    if texel.a < 0.1 { discard; }
    return vec4(texel.rgb * in.shade, texel.a);
}
