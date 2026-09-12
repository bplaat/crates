/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Draws a colored triangle with wgpu and bwindow.

use bwindow::{Event, EventLoop, KeyCode, LogicalSize, Theme, WindowBuilder, WindowEvent};

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

impl Gpu {
    fn new(attachment: bwindow::WindowAttachment) -> Self {
        // Configure the surface and device
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(attachment).expect("Create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
            })
            .expect("Find adapter");
        let (device, queue) = adapter
            .request_device(&Default::default())
            .expect("Create device");

        let (width, height) = surface.size();
        let mut config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .expect("Surface format");
        config.format = surface
            .get_capabilities(&adapter)
            .formats
            .into_iter()
            .find(|format| *format == wgpu::TextureFormat::Bgra8Unorm)
            .unwrap_or(config.format);
        surface.configure(&device, &config);

        // Create the triangle pipeline
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
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
        }
    }

    fn resize(&mut self) {
        let (width, height) = self.surface.size();
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self) -> bool {
        let (frame, reconfigure) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.resize();
                return true;
            }
            wgpu::CurrentSurfaceTexture::Timeout => return true,
            // Do not spin while hidden. A native paint/resize event resumes drawing.
            wgpu::CurrentSurfaceTexture::Occluded => return false,
            _ => {
                eprintln!("Surface lost; close and restart the example");
                return false;
            }
        };

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.015,
                            g: 0.02,
                            b: 0.04,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        if reconfigure {
            self.resize();
        }

        false
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .title("WGPU Triangle")
        .size(LogicalSize::new(900.0, 650.0))
        .min_size(LogicalSize::new(480.0, 360.0))
        .resizable(true)
        .theme(Theme::Dark)
        .center()
        .build();
    let mut gpu = Gpu::new(window.attach_content().expect("Attach graphics surface"));
    let window_id = window.id();

    window.request_redraw();
    event_loop.run(move |event| match event {
        Event::Window(id, WindowEvent::RedrawRequested) if id == window_id => {
            if gpu.render() {
                window.request_animation_frame();
            }
        }
        Event::Window(id, WindowEvent::Resize(_)) if id == window_id => {
            gpu.resize();
            window.request_redraw();
        }
        Event::Window(id, WindowEvent::KeyDown(event))
            if id == window_id && event.code == Some(KeyCode::Escape) =>
        {
            window.close();
        }
        _ => {}
    });
}
