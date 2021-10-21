use std::{iter, fs};
use cgmath::prelude::*;
use crate::{texture, camera::Camera, mesh};
use crate::mesh::*;
use crate::world::World;

use std::path;
use wgpu::util::DeviceExt;
use winit::{
    window::Window,
};
use cgmath::*;
use std::num::NonZeroU32;
use std::time::Instant;
use egui_wgpu_backend::*;
use egui_winit_platform::Platform;
use crate::texture::Texture;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
pub const IMAGES:[&str;3]=["textures\\grass_side.png","textures\\grass_top.png","textures\\grass_bottom.png"];

pub struct Renderer {
    surface: wgpu::Surface,
    config:wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    egui_rpass:RenderPass,
    fovy: Deg<f32>,
    znear: f32,
    zfar: f32,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    textures:Vec<Texture>,
    diffuse_bind_group: wgpu::BindGroup,
    projection :Matrix4<f32>,
    uniform: Uniform,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    depth_texture: Texture,
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
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::PUSH_CONSTANTS
                        |wgpu::Features::TEXTURE_BINDING_ARRAY
                        |wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                    limits: wgpu::Limits {
                        max_push_constant_size: 12,
                        ..wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &config);
        let mut egui_rpass = RenderPass::new(&device, config.format, 1);

        let fovy= Deg(45.0);
        let znear= 0.01;
        let zfar = 1000.0;
        let projection = OPENGL_TO_WGPU_MATRIX * perspective(fovy, config.width as f32 / config.height as f32, znear, zfar);
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: NonZeroU32::new(3),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            compare: None,//(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        let mut textures2 = vec![];
        let mut textures_views = vec![];
        for i in 0..3{
            let diffuse_bytes = fs::read(IMAGES[i]).expect("texture could not be loaded");
            textures2.push(Texture::from_bytes(&device, &queue, &diffuse_bytes, IMAGES[i]).unwrap());
        }
        let textures = textures2;
        for i in 0..3{
            textures_views.push(&textures[i].view);
        }
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&textures_views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });


        let uniform = Uniform::new();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let vs_module = unsafe{device.create_shader_module_spirv(&wgpu::include_spirv_raw!("vertex.spv"))};
        let fs_module = unsafe{device.create_shader_module_spirv(&wgpu::include_spirv_raw!("fragment.spv"))};
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange{
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..12
                }],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[mesh::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format.into(),
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                clamp_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });


        Self {
            surface,
            config,
            device,
            queue,
            egui_rpass,
            fovy,
            znear,
            zfar,
            size,
            render_pipeline,
            textures,
            diffuse_bind_group,
            projection,
            uniform_buffer,
            uniform_bind_group,
            uniform,
            depth_texture,
        }
    }
    pub fn create_index_buffer(&self,indices:&Vec<u32>)->wgpu::Buffer{
        self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        )
    }
    pub fn create_vertex_buffer(&self,vertices:&Vec<Vertex>)->wgpu::Buffer{
        self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = self.size.width.max(1);
        self.config.height = self.size.height.max(1);
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        self.projection = OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.config.width as f32 / self.config.height as f32, self.znear, self.zfar);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update(&mut self, camera : &Camera) {
        self.uniform.view_proj = (self.projection* camera.build_view_matrix()).into();
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    pub fn render(&mut self, world:&World, platform: &mut Platform){
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                println!("fail {}",e);
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture().expect("Failed to acquire next surface texture!")
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let render_time=Instant::now();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view:&view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            for (pos,mesh) in &world.mesh_map {
                render_pass.set_push_constants(wgpu::ShaderStages::VERTEX,0,bytemuck::cast_slice(&[pos.x<<5,pos.y<<5,pos.z<<5]));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..),wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(0,mesh.vertex_buffer.slice(..));
                render_pass.draw_indexed(0..mesh.num_elements,0,0..1);
            }
        }
        self.queue.submit(iter::once(encoder.finish()));


        platform.begin_frame();
        egui::Window::new("Info").show(&platform.context(), |ui| {
            ui.label(format!("render time:{}",render_time.elapsed().as_millis()));
        });
        let (_output, paint_commands) = platform.end_frame(None);
        let paint_jobs = platform.context().tessellate(paint_commands);

        let mut encoder_gui = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Egui encoder"),
        });
        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: self.config.width,
            physical_height: self.config.height,
            scale_factor: 1.0,
        };
        self.egui_rpass.update_texture(&self.device, &self.queue, &platform.context().texture());
        self.egui_rpass.update_user_textures(&self.device, &self.queue);
        self.egui_rpass.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.egui_rpass
            .execute(
                &mut encoder_gui,
                &view,
                &paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();
        // Submit the commands.
        self.queue.submit(iter::once(encoder_gui.finish()));

        frame.present();

    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    view_proj: [[f32; 4]; 4],
}

impl Uniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}