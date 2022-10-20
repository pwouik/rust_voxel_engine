use crate::camera::Camera;
use crate::chunk_loader::{RENDER_DIST, RENDER_DIST2, RENDER_DIST_HEIGHT, RENDER_DIST_HEIGHT2};
use crate::texture::Texture;
use cgmath::prelude::*;

use crate::chunk_renderer::ChunkRenderer;
use cgmath::*;
use std::iter;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    viewproj: [[f32; 4]; 4],
}

pub struct Renderer {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    fovy: Deg<f32>,
    znear: f32,
    zfar: f32,
    pub size: winit::dpi::PhysicalSize<u32>,
    projection: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    viewproj_buffer: wgpu::Buffer,
    pos_buffer: wgpu::Buffer,
    context_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
    depth_bind_group: wgpu::BindGroup,
    pub chunk_renderer: ChunkRenderer,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::MULTI_DRAW_INDIRECT
                        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
                        | wgpu::Features::PUSH_CONSTANTS
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits {
                        max_compute_invocations_per_workgroup: 1024,
                        max_compute_workgroup_size_x: 1024,
                        max_push_constant_size: 4,
                        ..wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            //alpha_mode: wgpu::CompositeAlphaMode::Auto
        };
        surface.configure(&device, &config);

        let fovy = Deg(45.0);
        let znear = 0.01;
        let zfar = 10000.0;
        let projection = OPENGL_TO_WGPU_MATRIX
            * perspective(
                fovy,
                config.width as f32 / config.height as f32,
                znear,
                zfar,
            );

        let view_matrix: Matrix4<f32> = Matrix4::identity();

        let viewproj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("viewproj Buffer"),
            contents: bytemuck::cast_slice(&[Uniform {
                viewproj: Matrix4::identity().into(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let pos_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pos Buffer"),
            contents: bytemuck::cast_slice(&[
                0u32,
                0u32,
                0u32,
                RENDER_DIST_HEIGHT2 as u32,
                RENDER_DIST2 as u32,
                0u32,
                0u32,
                0u32,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let context_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("context_bind_group_layout"),
            });

        let context_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &context_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: viewproj_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pos_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let depth_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture{
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("depth_bind_group_layout"),
            });
        let depth_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &depth_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&depth_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&depth_sampler),
                },
            ],
            label: Some("depth_bind_group"),
        });


        let chunk_renderer = ChunkRenderer::new(
            &device,
            &mut init_encoder,
            &queue,
            &config,
            &context_bind_group_layout,
            &depth_bind_group_layout,
        );
        queue.submit(Some(init_encoder.finish()));
        Self {
            surface,
            config,
            device,
            queue,
            fovy,
            znear,
            zfar,
            size,
            projection,
            view_matrix,
            viewproj_buffer,
            pos_buffer,
            depth_texture,
            depth_bind_group,
            chunk_renderer,
            context_bind_group,
        }
    }
    pub fn create_index_buffer(&self, indices: &Vec<u32>) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            })
    }
    pub fn get_frustum(&self, camera: &Camera) -> [Vector3<f32>; 5] {
        //origin and norms
        let angle = 45.0f32.to_radians();
        let up = vec4(0.0, (angle).cos(), -(angle).sin(), 0.0);
        let down = vec4(0.0, -up.y, up.z, 0.0);
        let aspect = self.config.width as f32 / self.config.height as f32;
        let right = vec4(up.y, 0.0, up.z * aspect, 0.0);
        let left = vec4(-up.y, 0.0, up.z * aspect, 0.0);
        let inv_rot_matrix = self.view_matrix.transpose();
        [
            vec3(
                camera.pos.x.rem_euclid(32.0),
                camera.pos.y.rem_euclid(32.0),
                camera.pos.z.rem_euclid(32.0),
            ) / 32.0,
            (inv_rot_matrix * down).truncate(),
            (inv_rot_matrix * up).truncate(),
            (inv_rot_matrix * left).truncate(),
            (inv_rot_matrix * right).truncate(),
        ]
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = self.size.width.max(1);
        self.config.height = self.size.height.max(1);
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        self.projection = OPENGL_TO_WGPU_MATRIX
            * perspective(
                self.fovy,
                self.config.width as f32 / self.config.height as f32,
                self.znear,
                self.zfar,
            );
        self.surface.configure(&self.device, &self.config);
    }

    #[profiling::function]
    pub fn render(&mut self, camera: &Camera) {
        let player_pos = point3(
            (camera.pos.x / 32.0).floor() as i32,
            (camera.pos.y / 32.0).floor() as i32,
            (camera.pos.z / 32.0).floor() as i32,
        );
        self.queue.write_buffer(
            &self.pos_buffer,
            0,
            bytemuck::cast_slice(&[player_pos.x, player_pos.y, player_pos.z]),
        );
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                println!("fail {}", e);
                self.surface.configure(&self.device, &self.config);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Chunk Render Encoder"),
            });

        self.chunk_renderer.culling(
            self.get_frustum(&camera),
            player_pos,
            &self.queue,
            &mut encoder,
            &view,
            &self.depth_texture,
            &self.context_bind_group,
            &self.depth_bind_group,
        );
        self.view_matrix = camera.build_view_matrix();
        self.queue.write_buffer(
            &self.viewproj_buffer,
            0,
            bytemuck::cast_slice(&[Uniform {
                viewproj: (self.projection * self.view_matrix).into(),
            }]),
        );
        self.chunk_renderer.render_chunks(
            &mut encoder,
            &view,
            &self.depth_texture,
            &self.context_bind_group,
        );
        self.queue.submit(iter::once(encoder.finish()));
        frame.present();
        self.chunk_renderer.map_count();
    }
}
