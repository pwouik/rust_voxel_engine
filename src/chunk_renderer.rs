use crate::mesh::Face;
use crate::mipmap;
use crate::render_region::{RenderRegion, RENDER_REGION_CHUNKS};
use crate::texture::*;
use glam::{ivec3, IVec3, Vec3};
use std::borrow::Cow;
use std::collections::HashMap;

pub const IMAGES: [&str; 5] = [
    "textures/grass_side.png",
    "textures/grass_top.png",
    "textures/grass_bottom.png",
    "textures/stone.png",
    "textures/brick.png",
];

pub struct ChunkRenderer {
    texture_array: Texture,
    render_pipeline: wgpu::RenderPipeline,
    occlusion_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    diffuse_bind_group: wgpu::BindGroup,
    map: HashMap<IVec3, RenderRegion>,
    face_bind_group_layout: wgpu::BindGroupLayout,
    occlusion_bind_group_layout: wgpu::BindGroupLayout,
    scan_bind_group_layout: wgpu::BindGroupLayout,
}
impl ChunkRenderer {
    #[profiling::function]
    pub fn new(
        device: &wgpu::Device,
        init_encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        context_bind_group_layout: &wgpu::BindGroupLayout,
        depth_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
                label: Some("texture_bind_group_layout"),
            });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let mut images = vec![];
        for img in IMAGES {
            images.push(image::open(img).expect("texture could not be loaded"));
        }
        let texture_array =
            Texture::from_images(device, queue, &images, Some("texture_array")).unwrap();
        mipmap::generate_mipmaps(
            init_encoder,
            &device,
            &texture_array.texture,
            IMAGES.len() as u32,
            MIP_LEVEL_COUNT,
        );
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_array.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let scan_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("command_bind_group_layout"),
            });
        let occlusion_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("occlusion_bind_group_layout"),
            });
        let occlusion_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("occlusion.wgsl"))),
        });
        let occlusion_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &context_bind_group_layout,
                    depth_bind_group_layout,
                    &occlusion_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let occlusion_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Occlusion Pipeline"),
            layout: Some(&occlusion_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &occlusion_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &occlusion_module,
                entry_point: "fs_main",
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Never,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        let cs_occlusion_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: "scan shader".into(),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("scan.wgsl"))),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&context_bind_group_layout, &scan_bind_group_layout],
                push_constant_ranges: &[],
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &(cs_occlusion_module),
            entry_point: "main",
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chunk shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("chunk.wgsl"))),
        });

        let region_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("chunk_bind_group_layout"),
            });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &context_bind_group_layout,
                    &region_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..4,
                }],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Chunk Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.into(),
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        Self {
            texture_array,
            render_pipeline,
            occlusion_pipeline,
            compute_pipeline,
            diffuse_bind_group,
            map: HashMap::new(),
            face_bind_group_layout: region_bind_group_layout,
            occlusion_bind_group_layout,
            scan_bind_group_layout,
        }
    }

    #[profiling::function]
    pub fn add_chunk(
        &mut self,
        pos: IVec3,
        data: &mut [Vec<Face>; 6],
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        let ipos = ivec3(pos.x & !15, pos.y & !7, pos.z & !15);
        let region = self.map.get_mut(&ipos);
        if let Some(region) = region {
            region.add_chunk(
                ivec3(pos.x & 15, pos.y & 7, pos.z & 15),
                data,
                queue,
                device,
                &self.face_bind_group_layout,
            );
        } else {
            self.map.insert(
                ipos,
                RenderRegion::new(
                    device,
                    &self.face_bind_group_layout,
                    &self.scan_bind_group_layout,
                    &self.occlusion_bind_group_layout,
                ),
            );
        }
    }
    #[profiling::function]
    pub fn remove_chunk(&mut self, pos: IVec3, queue: &mut wgpu::Queue) {
        let ipos = ivec3(pos.x & !15, pos.y & !7, pos.z & !15);
        let region = self.map.get_mut(&ipos);
        if let Some(region) = region {
            region.remove_chunk(ivec3(pos.x & 15, pos.y & 7, pos.z & 15), queue);
        } else {
        }
    }
    #[profiling::function]
    pub fn culling(
        &mut self,
        frustum: [Vec3; 5],
        player_pos: IVec3,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        depth_texture: &Texture,
        context_bind_group: &wgpu::BindGroup,
        depth_bind_group: &wgpu::BindGroup,
    ) {
        {
            let mut occlusion_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Occlusion Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: None,
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            occlusion_pass.set_pipeline(&self.occlusion_pipeline);
            occlusion_pass.set_bind_group(0, context_bind_group, &[]);
            occlusion_pass.set_bind_group(1, depth_bind_group, &[]);
            for region in &mut self.map {
                region.1.frustum(region.0, player_pos, frustum, queue);
                region.1.occlusion(&mut occlusion_pass);
            }
        }
        for region in &mut self.map {
            region.1.read_count(encoder);
        }
        {
            let mut scan_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Scan Pass"),
                timestamp_writes: None,
            });
            scan_pass.set_pipeline(&self.compute_pipeline);
            scan_pass.set_bind_group(0, &context_bind_group, &[]);
            for region in &mut self.map {
                region.1.gen_commands(&mut scan_pass);
            }
        }
    }
    #[profiling::function]
    pub fn render_chunks(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_texture: &Texture,
        context_bind_group: &wgpu::BindGroup,
        player_pos: IVec3,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Chunk Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &context_bind_group, &[]);
        render_pass.set_bind_group(2, &self.diffuse_bind_group, &[]);
        for region in &mut self.map {
            let relative_region_pos = *region.0 - player_pos;
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                &[
                    (relative_region_pos.x + 128) as u8,
                    (relative_region_pos.y + 128) as u8,
                    (relative_region_pos.z + 128) as u8,
                    0u8,
                ],
            );
            region.1.draw(&mut render_pass);
        }
    }
}
impl Drop for ChunkRenderer {
    fn drop(&mut self) {
        println!("drop chunkrenderer");
    }
}
