/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Render a triangle without a window and save it as a PNG.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(&Default::default())?;
    let (device, queue) = adapter.request_device(&Default::default())?;
    let target = device.create_bitmap_target(800, 600);

    let shader = device.create_shader_module(wgpu::include_wgsl!("triangle.wgsl"));
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });

    let view = target.create_view();
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::from_rgb8(
                        0x101820,
                        wgpu::TextureFormat::Rgba8Unorm,
                    )),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }
    queue.submit([encoder.finish()]);
    let bitmap = target.read(&queue)?;
    std::fs::write("frame.png", image::encode(&bitmap, image::Format::Png)?)?;
    Ok(())
}
