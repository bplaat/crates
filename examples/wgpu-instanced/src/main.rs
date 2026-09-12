/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Showcases instancing, storage buffers, texture arrays, mipmaps, and render textures.

use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use bcanvas::{Color, OffscreenCanvas, TextAlign, TextBaseline};
use bwindow::{Event, EventLoop, KeyCode, LogicalSize, Theme, WindowBuilder, WindowEvent};

const RENDER_TEXTURE_SIZE: u32 = 1024;
const MATERIAL_SIZE: u32 = 128;
const MATERIAL_MIP_COUNT: u32 = MATERIAL_SIZE.ilog2() + 1;
const GRID_SIDE: u32 = 4;
const INSTANCE_COUNT: u32 = GRID_SIDE * GRID_SIDE;
const INSTANCED_UNIFORM_SIZE: u64 = 32;
const DISPLAY_UNIFORM_SIZE: u64 = 16;
const INSTANCE_SIZE: u64 = 48;
const INSTANCE_BUFFER_SIZE: u64 = INSTANCE_SIZE * INSTANCE_COUNT as u64;

#[derive(rust_embed::Embed)]
#[folder = "assets"]
struct MaterialAssets;

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    surface_depth: wgpu::TextureView,
    render_texture_view: wgpu::TextureView,
    render_texture_depth: wgpu::TextureView,
    instanced_pipeline: wgpu::RenderPipeline,
    display_pipeline: wgpu::RenderPipeline,
    overlay_pipeline: wgpu::RenderPipeline,
    instanced_group: wgpu::BindGroup,
    display_group: wgpu::BindGroup,
    overlay_group: wgpu::BindGroup,
    overlay_texture: wgpu::Texture,
    overlay_canvas: OffscreenCanvas,
    displayed_fps: u32,
    instanced_uniforms: wgpu::Buffer,
    display_uniforms: wgpu::Buffer,
    vertices: wgpu::Buffer,
    vertex_count: u32,
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
            .expect("Find graphics adapter");
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
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(config.format);
        surface.configure(&device, &config);

        // Create the scene bind group layouts
        let instanced_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("instance layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(INSTANCED_UNIFORM_SIZE),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(INSTANCE_BUFFER_SIZE),
                    },
                    count: None,
                },
            ],
        });
        let display_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(DISPLAY_UNIFORM_SIZE),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create the offscreen and surface render targets
        let render_texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("instanced scene texture"),
            size: wgpu::Extent3d {
                width: RENDER_TEXTURE_SIZE,
                height: RENDER_TEXTURE_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: render_texture_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let render_texture_view = render_texture.create_view(&Default::default());
        let render_texture_depth = Self::depth(&device, RENDER_TEXTURE_SIZE, RENDER_TEXTURE_SIZE);
        let surface_depth = Self::depth(&device, config.width, config.height);

        // Upload the scene resources
        let instanced_uniforms =
            Self::uniform_buffer(&device, "instanced uniforms", INSTANCED_UNIFORM_SIZE);
        let display_uniforms =
            Self::uniform_buffer(&device, "display uniforms", DISPLAY_UNIFORM_SIZE);
        let material_names = material_names();
        let material_view = upload_materials(&device, &queue, &material_names);
        let instance_data = instances(&block_materials(&material_names));
        let instance_buffer = Self::mapped_buffer(
            &device,
            "instance storage",
            wgpu::BufferUsages::STORAGE,
            &instance_bytes(&instance_data),
        );
        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let instanced_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("instanced group"),
            layout: &instanced_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instanced_uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&material_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&material_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: instance_buffer.as_entire_binding(),
                },
            ],
        });
        let display_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let display_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture group"),
            layout: &display_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: display_uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&render_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&display_sampler),
                },
            ],
        });

        // Create the scene pipelines
        let instanced_shader = device.create_shader_module(wgpu::include_wgsl!("instances.wgsl"));
        let display_shader = device.create_shader_module(wgpu::include_wgsl!("textured-cube.wgsl"));
        let instanced_pipeline = Self::pipeline(
            &device,
            &instanced_layout,
            &instanced_shader,
            render_texture_format,
        );
        let display_pipeline =
            Self::pipeline(&device, &display_layout, &display_shader, config.format);

        // Create the FPS overlay pipeline and resources
        let overlay_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let overlay_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&overlay_layout)],
            });
        let overlay_shader = device.create_shader_module(wgpu::include_wgsl!("overlay.wgsl"));
        let overlay_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay pipeline"),
            layout: Some(&overlay_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &overlay_shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &overlay_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let overlay_canvas = OffscreenCanvas::new(120, 36);
        let overlay_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("FPS overlay texture"),
            size: wgpu::Extent3d {
                width: overlay_canvas.width(),
                height: overlay_canvas.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let overlay_view = overlay_texture.create_view(&Default::default());
        let overlay_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let overlay_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay group"),
            layout: &overlay_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&overlay_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&overlay_sampler),
                },
            ],
        });

        // Upload geometry shared by both scene pipelines
        let vertex_data = cube_vertices();
        let vertex_bytes = vertex_bytes(&vertex_data);
        let vertices = Self::mapped_buffer(
            &device,
            "cube vertices",
            wgpu::BufferUsages::VERTEX,
            &vertex_bytes,
        );

        Self {
            surface,
            device,
            queue,
            config,
            surface_depth,
            render_texture_view,
            render_texture_depth,
            instanced_pipeline,
            display_pipeline,
            overlay_pipeline,
            instanced_group,
            display_group,
            overlay_group,
            overlay_texture,
            overlay_canvas,
            displayed_fps: u32::MAX,
            instanced_uniforms,
            display_uniforms,
            vertices,
            vertex_count: vertex_data.len() as u32,
        }
    }

    fn pipeline(
        device: &wgpu::Device,
        group_layout: &wgpu::BindGroupLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(group_layout)],
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 24,
                            shader_location: 2,
                        },
                    ],
                })],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    }

    fn uniform_buffer(device: &wgpu::Device, label: &'static str, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn mapped_buffer(
        device: &wgpu::Device,
        label: &'static str,
        usage: wgpu::BufferUsages,
        bytes: &[u8],
    ) -> wgpu::Buffer {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes.len() as u64,
            usage,
            mapped_at_creation: true,
        });
        buffer
            .slice(..)
            .get_mapped_range_mut()
            .expect("Mapped buffer")
            .copy_from_slice(bytes);
        buffer.unmap();
        buffer
    }

    fn depth(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("depth texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default())
    }

    fn resize(&mut self) {
        let (width, height) = self.surface.size();
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.surface_depth = Self::depth(&self.device, width, height);
    }

    fn render(&mut self, elapsed: f32, fps: u32) -> bool {
        let (frame, reconfigure) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.resize();
                return true;
            }
            wgpu::CurrentSurfaceTexture::Timeout => return true,
            wgpu::CurrentSurfaceTexture::Occluded => return false,
            _ => {
                eprintln!("Surface lost; close and restart the example");
                return false;
            }
        };

        // Update per-frame resources
        self.queue.write_buffer(
            &self.instanced_uniforms,
            0,
            &instanced_uniform_bytes(elapsed),
        );
        self.queue.write_buffer(
            &self.display_uniforms,
            0,
            &display_uniform_bytes(
                elapsed,
                self.config.width as f32 / self.config.height as f32,
            ),
        );
        self.update_fps(fps);

        let surface_view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        // Render the grid into the offscreen texture
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render instances to texture"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.025,
                            g: 0.035,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.render_texture_depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
            });
            pass.set_pipeline(&self.instanced_pipeline);
            pass.set_bind_group(0, &self.instanced_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_scissor_rect(8, 8, RENDER_TEXTURE_SIZE - 16, RENDER_TEXTURE_SIZE - 16);
            pass.draw(0..self.vertex_count, 0..INSTANCE_COUNT);
        }

        // Render the outer cube and overlay to the surface
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("draw textured cube"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.005,
                            g: 0.008,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.surface_depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
            });
            pass.set_pipeline(&self.display_pipeline);
            pass.set_bind_group(0, &self.display_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.draw(0..self.vertex_count, 0..1);

            if self.config.width >= self.overlay_canvas.width() + 8
                && self.config.height >= self.overlay_canvas.height() + 8
            {
                let overlay_width = self.overlay_canvas.width() as f32;
                let overlay_height = self.overlay_canvas.height() as f32;
                pass.set_viewport(
                    self.config.width as f32 - overlay_width - 8.0,
                    8.0,
                    overlay_width,
                    overlay_height,
                    0.0,
                    1.0,
                );
                pass.set_pipeline(&self.overlay_pipeline);
                pass.set_bind_group(0, &self.overlay_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        if reconfigure {
            self.resize();
        }

        true
    }

    fn update_fps(&mut self, fps: u32) {
        if self.displayed_fps == fps {
            return;
        }

        self.displayed_fps = fps;
        self.overlay_canvas.draw(|ctx| {
            ctx.clear_rect(0.0, 0.0, ctx.width(), ctx.height());
            ctx.set_fill_style(Color::rgb(255, 255, 255));
            ctx.set_font("sans-serif", 18.0);
            ctx.set_text_align(TextAlign::Right);
            ctx.set_text_baseline(TextBaseline::Top);
            ctx.fill_text(format!("{fps} FPS"), ctx.width() - 8.0, 8.0);
        });

        let width = self.overlay_canvas.width();
        let height = self.overlay_canvas.height();
        let bytes_per_row = self.overlay_canvas.bytes_per_row();
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.overlay_texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            self.overlay_canvas.pixels(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

#[derive(Default)]
struct FrameRate {
    sample_start: Duration,
    frames: u32,
    value: u32,
}

impl FrameRate {
    fn sample(&mut self, timestamp: Duration) -> u32 {
        self.frames += 1;
        let elapsed = timestamp.saturating_sub(self.sample_start);
        if elapsed >= Duration::from_secs(1) {
            self.value = (f64::from(self.frames) / elapsed.as_secs_f64()).round() as u32;
            self.sample_start = timestamp;
            self.frames = 0;
        }
        self.value
    }
}

fn display_uniform_bytes(elapsed: f32, aspect: f32) -> [u8; 16] {
    let mut bytes = [0; 16];
    for (index, value) in [elapsed, aspect, 0.0, 0.0].into_iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_ne_bytes());
    }
    bytes
}

fn instanced_uniform_bytes(elapsed: f32) -> [u8; 32] {
    let mut bytes = [0; 32];
    for (index, value) in [elapsed, 1.0, 0.0, 0.0, 7.5, 0.0, 0.0, 0.0]
        .into_iter()
        .enumerate()
    {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_ne_bytes());
    }
    bytes
}

struct Instance {
    position_scale: [f32; 4],
    rotation: [f32; 4],
    material: [u32; 4],
}

fn instances(materials: &[[u32; 3]]) -> Vec<Instance> {
    assert_eq!(materials.len(), INSTANCE_COUNT as usize);

    let mut random = Random(0x7a31_9d4b);
    let mut material_order = (0..materials.len()).collect::<Vec<_>>();
    for index in (1..material_order.len()).rev() {
        let other = random.next() as usize % (index + 1);
        material_order.swap(index, other);
    }

    let mut instances = Vec::with_capacity(INSTANCE_COUNT as usize);
    let center = (GRID_SIDE - 1) as f32 * 0.5;
    for y in 0..GRID_SIDE {
        for x in 0..GRID_SIDE {
            let material = materials[material_order[instances.len()]];
            instances.push(Instance {
                position_scale: [
                    (x as f32 - center) * 1.45,
                    (y as f32 - center) * 1.45,
                    0.0,
                    0.55,
                ],
                rotation: [
                    random.unit() * std::f32::consts::TAU,
                    random.unit() * std::f32::consts::TAU,
                    0.25 + random.unit() * 0.75,
                    0.25 + random.unit() * 0.75,
                ],
                material: [material[0], material[1], material[2], 0],
            });
        }
    }
    instances
}

fn material_names() -> Vec<String> {
    let mut names = MaterialAssets::iter()
        .filter(|name| name.ends_with(".png"))
        .map(|name| name.into_owned())
        .collect::<Vec<_>>();
    names.sort_unstable();
    assert!(!names.is_empty(), "Assets must contain at least one PNG");

    names
}

fn block_materials(material_names: &[String]) -> Vec<[u32; 3]> {
    let same = |name| {
        let layer = texture_layer(material_names, name);
        [layer, layer, layer]
    };
    vec![
        same("brick_red.png"),
        [
            texture_layer(material_names, "cactus_side.png"),
            texture_layer(material_names, "cactus_top.png"),
            texture_layer(material_names, "cactus_top.png"),
        ],
        same("dirt.png"),
        [
            texture_layer(material_names, "dirt_grass.png"),
            texture_layer(material_names, "grass_top.png"),
            texture_layer(material_names, "dirt.png"),
        ],
        same("greystone.png"),
        same("lava.png"),
        same("leaves.png"),
        same("sand.png"),
        same("stone_coal.png"),
        same("stone_diamond.png"),
        same("stone_gold.png"),
        same("stone_iron.png"),
        same("stone.png"),
        [
            texture_layer(material_names, "trunk_side.png"),
            texture_layer(material_names, "trunk_top.png"),
            texture_layer(material_names, "trunk_top.png"),
        ],
        same("water.png"),
        same("wood.png"),
    ]
}

fn texture_layer(material_names: &[String], name: &str) -> u32 {
    material_names
        .iter()
        .position(|candidate| candidate == name)
        .map(|index| index as u32)
        .unwrap_or_else(|| panic!("Missing texture asset: {name}"))
}

fn instance_bytes(instances: &[Instance]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(instances.len() * INSTANCE_SIZE as usize);
    for instance in instances {
        for value in instance.position_scale.into_iter().chain(instance.rotation) {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
        for value in instance.material {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }

    bytes
}

struct Random(u32);

impl Random {
    const fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }

    fn unit(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
}

fn upload_materials(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    material_names: &[String],
) -> wgpu::TextureView {
    let layer_count = u32::try_from(material_names.len()).expect("Material layer count");

    // Decode the embedded PNG layers
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("material texture array"),
        size: wgpu::Extent3d {
            width: MATERIAL_SIZE,
            height: MATERIAL_SIZE,
            depth_or_array_layers: layer_count,
        },
        mip_level_count: MATERIAL_MIP_COUNT,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut layers = material_names
        .iter()
        .map(|name| {
            let asset = MaterialAssets::get(name)
                .unwrap_or_else(|| panic!("Missing texture asset: {name}"));
            let image = image::decode(asset.data.as_ref())
                .unwrap_or_else(|error| panic!("Decode {name}: {error}"));
            assert_eq!(image.width(), MATERIAL_SIZE, "Unexpected width for {name}");
            assert_eq!(
                image.height(),
                MATERIAL_SIZE,
                "Unexpected height for {name}"
            );
            image.pixels().to_vec()
        })
        .collect::<Vec<_>>();

    // Upload every generated mip level as one texture-array write
    let mut width = MATERIAL_SIZE;
    let mut height = MATERIAL_SIZE;
    for mip_level in 0..MATERIAL_MIP_COUNT {
        let pixels = layers.iter().flatten().copied().collect::<Vec<_>>();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: layer_count,
            },
        );
        if width > 1 || height > 1 {
            layers = layers
                .iter()
                .map(|pixels| downsample_rgba(pixels, width, height))
                .collect();
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }
    }

    texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
    })
}

fn downsample_rgba(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let next_width = (width / 2).max(1);
    let next_height = (height / 2).max(1);
    let mut output = Vec::with_capacity((next_width * next_height * 4) as usize);
    for y in 0..next_height {
        for x in 0..next_width {
            let mut alpha_sum = 0.0;
            let mut color_sum = [0.0; 3];
            for offset_y in 0..2 {
                for offset_x in 0..2 {
                    let source_x = (x * 2 + offset_x).min(width - 1);
                    let source_y = (y * 2 + offset_y).min(height - 1);
                    let index = ((source_y * width + source_x) * 4) as usize;
                    let alpha = f32::from(pixels[index + 3]) / 255.0;
                    alpha_sum += alpha;
                    for channel in 0..3 {
                        color_sum[channel] += srgb_to_linear(pixels[index + channel]) * alpha;
                    }
                }
            }
            for color in color_sum {
                output.push(linear_to_srgb(if alpha_sum > 0.0 {
                    color / alpha_sum
                } else {
                    0.0
                }));
            }
            output.push(((alpha_sum / 4.0) * 255.0).round() as u8);
        }
    }

    output
}

fn srgb_to_linear(value: u8) -> f32 {
    let value = f32::from(value) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> u8 {
    let value = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

fn vertex_bytes(vertices: &[Vertex]) -> Vec<u8> {
    vertices
        .iter()
        .flat_map(|vertex| {
            [
                vertex.position[0],
                vertex.position[1],
                vertex.position[2],
                vertex.normal[0],
                vertex.normal[1],
                vertex.normal[2],
                vertex.uv[0],
                vertex.uv[1],
            ]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
        })
        .collect()
}

fn cube_vertices() -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(36);
    for (normal, corners) in [
        (
            [0., 0., 1.],
            [[-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]],
        ),
        (
            [0., 0., -1.],
            [
                [1., -1., -1.],
                [-1., -1., -1.],
                [-1., 1., -1.],
                [1., 1., -1.],
            ],
        ),
        (
            [1., 0., 0.],
            [[1., -1., 1.], [1., -1., -1.], [1., 1., -1.], [1., 1., 1.]],
        ),
        (
            [-1., 0., 0.],
            [
                [-1., -1., -1.],
                [-1., -1., 1.],
                [-1., 1., 1.],
                [-1., 1., -1.],
            ],
        ),
        (
            [0., 1., 0.],
            [[-1., 1., 1.], [1., 1., 1.], [1., 1., -1.], [-1., 1., -1.]],
        ),
        (
            [0., -1., 0.],
            [
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., -1., 1.],
                [-1., -1., 1.],
            ],
        ),
    ] {
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(Vertex {
                position: corners[index],
                normal,
                uv: [[0., 1.], [1., 1.], [1., 0.], [0., 0.]][index],
            });
        }
    }

    vertices
}

fn main() {
    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
        .title("WGPU Instanced")
        .size(LogicalSize::new(960.0, 720.0))
        .theme(Theme::Dark)
        .background_color(0x000000)
        .center()
        .build();
    let mut gpu = Gpu::new(window.attach_content().expect("Attach graphics surface"));
    let started = Instant::now();
    let mut frame_rate = FrameRate::default();
    let window_id = window.id();

    window.request_animation_frame();
    event_loop.run(move |event| match event {
        Event::Window(id, WindowEvent::RedrawRequested) if id == window_id => {
            let elapsed = started.elapsed();
            if gpu.render(elapsed.as_secs_f32(), frame_rate.sample(elapsed)) {
                window.request_animation_frame();
            }
        }
        Event::Window(id, WindowEvent::Resize(_)) if id == window_id => {
            gpu.resize();
            window.request_animation_frame();
        }
        Event::Window(id, WindowEvent::KeyDown(event))
            if id == window_id && event.code == Some(KeyCode::Escape) =>
        {
            window.close();
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_are_uniform_png_layers() {
        let material_names = material_names();
        for name in &material_names {
            let asset = MaterialAssets::get(name).expect("Embedded material");
            let image = image::decode(asset.data.as_ref())
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            assert_eq!(image.format(), image::Format::Png);
            assert_eq!(
                (image.width(), image.height()),
                (MATERIAL_SIZE, MATERIAL_SIZE)
            );
        }
    }

    #[test]
    fn instances_fill_storage_buffer_with_valid_materials() {
        let material_names = material_names();
        let material_count = material_names.len() as u32;
        let materials = block_materials(&material_names);
        let instances = instances(&materials);
        assert_eq!(instances.len(), INSTANCE_COUNT as usize);
        assert_eq!(
            instance_bytes(&instances).len(),
            instances.len() * INSTANCE_SIZE as usize
        );
        assert!(
            instances
                .iter()
                .flat_map(|instance| instance.material[..3].iter())
                .all(|material| *material < material_count)
        );
        let mut assigned = instances
            .iter()
            .map(|instance| instance.material[..3].to_vec())
            .collect::<Vec<_>>();
        assigned.sort_unstable();
        assigned.dedup();
        assert_eq!(assigned.len(), INSTANCE_COUNT as usize);
    }

    #[test]
    fn special_blocks_use_face_specific_textures() {
        let material_names = material_names();
        let materials = block_materials(&material_names);
        assert_eq!(materials.len(), INSTANCE_COUNT as usize);
        assert_eq!(
            materials[1],
            [
                texture_layer(&material_names, "cactus_side.png"),
                texture_layer(&material_names, "cactus_top.png"),
                texture_layer(&material_names, "cactus_top.png")
            ]
        );
        assert_eq!(
            materials[3],
            [
                texture_layer(&material_names, "dirt_grass.png"),
                texture_layer(&material_names, "grass_top.png"),
                texture_layer(&material_names, "dirt.png")
            ]
        );
        assert_ne!(materials[2], materials[3]);
    }

    #[test]
    fn mip_downsampling_averages_in_linear_color_space() {
        let pixels = [
            0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
        ];
        assert_eq!(downsample_rgba(&pixels, 2, 2), [188, 188, 188, 255]);
    }

    #[test]
    fn mip_downsampling_weights_color_by_alpha() {
        let pixels = [255, 0, 0, 255, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0];
        assert_eq!(downsample_rgba(&pixels, 2, 2), [255, 0, 0, 64]);
    }

    #[test]
    fn instanced_camera_remains_fixed() {
        let start = instanced_uniform_bytes(0.0);
        let later = instanced_uniform_bytes(2.0);
        assert_ne!(&start[..4], &later[..4]);
        assert_eq!(&start[4..], &later[4..]);
        assert_eq!(
            f32::from_ne_bytes(later[16..20].try_into().expect("Distance")),
            7.5
        );
    }

    #[test]
    fn frame_rate_updates_after_one_second() {
        let mut frame_rate = FrameRate::default();
        for frame in 1..=60 {
            frame_rate.sample(Duration::from_secs_f64(f64::from(frame) / 60.0));
        }
        assert_eq!(frame_rate.value, 60);
    }
}
