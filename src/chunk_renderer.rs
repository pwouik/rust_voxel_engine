use crate::chunk_loader::{
    NB_CHUNKS, RENDER_DIST, RENDER_DIST2, RENDER_DIST_HEIGHT, RENDER_DIST_HEIGHT2,
};
use crate::mesh::Face;
use crate::mipmap;
use crate::texture::*;
use cgmath::{point3, vec3, EuclideanSpace, Point3};
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::{fs, iter};
use wgpu::util::DeviceExt;
pub const IMAGES: [&str; 4] = [
    "textures/grass_side.png",
    "textures/grass_top.png",
    "textures/grass_bottom.png",
    "textures/stone.png",
];

pub struct ChunkRenderer {
    index: Vec<[u32; 3]>,
    rendered_chunks: Vec<u32>,
    box_buffer: wgpu::Buffer,
    count_buffer: wgpu::Buffer,
    visibility_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    chunk_table_buffer: wgpu::Buffer,
    face_buffer: wgpu::Buffer,
    face_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    occlusion_bind_group: wgpu::BindGroup,
    occlusion_collect_bind_group: wgpu::BindGroup,
    texture_array: Texture,
    occlusion_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
}
impl ChunkRenderer {
    pub fn new(
        device: &wgpu::Device,
        init_encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        context_bind_group_layout: &wgpu::BindGroupLayout,
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
            compare: None,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        let mut images = vec![];
        for img in IMAGES {
            images.push(image::open(img).expect("texture could not be loaded"));
        }
        let texture_array =
            Texture::from_images(device, queue, &images, Some("texture_array")).unwrap();
        /*mipmap::generate_mipmaps(
            init_encoder,
            &device,
            &texture_array.texture,
            IMAGES.len() as u32,
            MIP_LEVEL_COUNT,
        );*/
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

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chunk shader module"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("chunk.wgsl"))),
        });
        let occlusion_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("occlusion.wgsl"))),
        });
        let cs_occlusion_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("occlusion_collect.wgsl"))),
        });
        let box_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("box Buffer"),
            size: (NB_CHUNKS * 4).into(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let visibility_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("visibility Buffer"),
            size: (NB_CHUNKS * 4 * 6).into(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
        let occlusion_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &occlusion_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: box_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: visibility_buffer.as_entire_binding(),
                },
            ],
            label: Some("occlusion_bind_group"),
        });
        let occlusion_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&context_bind_group_layout, &occlusion_bind_group_layout],
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
                format: DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        let count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Count Buffer"),
            contents: bytemuck::cast_slice(&[0 as u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
        });
        let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Buffer"),
            size: (NB_CHUNKS * 6 * 4 * 4).into(),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let chunk_table_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Storage Buffer"),
            size: (NB_CHUNKS * 7 * 4).into(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let occlusion_collect_bind_group_layout =
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
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("command_bind_group_layout"),
            });
        let occlusion_collect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &occlusion_collect_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: visibility_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: indirect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: chunk_table_buffer.as_entire_binding(),
                },
            ],
            label: Some("occlusion_collect_bind_group"),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &context_bind_group_layout,
                    &occlusion_collect_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &(cs_occlusion_module),
            entry_point: "main",
        });
        let face_bind_group_layout =
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
        let face_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Storage Buffer"),
            size: (RENDER_DIST2 * RENDER_DIST2) as u64 * 32 * 32 * 3 * 12 * 2 + 240000,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let face_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("face bind group"),
            layout: &face_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: face_buffer.as_entire_binding(),
            }],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &context_bind_group_layout,
                    &face_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            index: vec![],
            rendered_chunks: vec![],
            box_buffer,
            count_buffer,
            visibility_buffer,
            indirect_buffer,
            chunk_table_buffer,
            face_buffer,
            face_bind_group,
            texture_array,
            diffuse_bind_group,
            occlusion_bind_group,
            occlusion_collect_bind_group,
            render_pipeline,
            compute_pipeline,
            occlusion_pipeline,
        }
    }

    pub fn add_chunk(&mut self, pos: Point3<i32>, data: &mut [Vec<Face>; 6], queue: &wgpu::Queue) {
        let mut sizes = [0 as u32; 7];
        let mut mesh = vec![];
        for i in 0..6 {
            sizes[i + 1] = sizes[i] + data[i].len() as u32;
            mesh.append(&mut data[i]);
        }
        let mut i: usize = 0;
        let mut location = 0;
        if self.index.len() != 0 && sizes[6] > self.index[0][1] {
            loop {
                i += 1;
                if i >= self.index.len() || self.index[i][1] - self.index[i - 1][2] >= sizes[6] {
                    break;
                }
            }
            location = self.index[i - 1][2];
        }
        for j in 0..7 {
            sizes[j] += location;
        }
        let id = (pos.x.rem_euclid(RENDER_DIST2)
            + RENDER_DIST2 * pos.y.rem_euclid(RENDER_DIST_HEIGHT2)
            + RENDER_DIST2 * RENDER_DIST_HEIGHT2 * pos.z.rem_euclid(RENDER_DIST2))
            as u32;
        self.index.insert(i as usize, [id, sizes[0], sizes[6]]);
        queue.write_buffer(
            &self.chunk_table_buffer,
            id as u64 * 4 * 7,
            &bytemuck::cast_slice(&sizes),
        );
        queue.write_buffer(
            &self.face_buffer,
            sizes[0] as u64 * 12,
            &bytemuck::cast_slice(&mesh),
        );
    }

    pub fn remove_chunk(&mut self, pos: Point3<i32>) {
        let id = (pos.x.rem_euclid(RENDER_DIST2)
            + RENDER_DIST2 * pos.y.rem_euclid(RENDER_DIST_HEIGHT2)
            + RENDER_DIST2 * RENDER_DIST_HEIGHT2 * pos.z.rem_euclid(RENDER_DIST2))
            as u32;
        let mut i = 0;
        while i < self.index.len() && self.index[i][0] != id {
            i += 1;
        }
        if i != self.index.len() {
            self.index.remove(i);
        };
        //queue.write_buffer(&self.chunk_table_buffer,id.into(),&vec![0u32;7]);
    }

    pub fn render_chunks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        depth_texture: &Texture,
        old_pos: Point3<i32>,
        player_pos: Point3<i32>,
        norms: [Point3<f32>; 4],
        context_bind_group: &wgpu::BindGroup,
    ) {
        self.rendered_chunks.clear();
        for i in 0..self.index.len() {
            let mut id = self.index[i][0];
            let x = (id as i32 - player_pos.x + RENDER_DIST).rem_euclid(RENDER_DIST2) - RENDER_DIST;
            id /= RENDER_DIST2 as u32;
            let y = (id as i32 - player_pos.y + RENDER_DIST_HEIGHT).rem_euclid(RENDER_DIST_HEIGHT2)
                - RENDER_DIST_HEIGHT;
            id /= RENDER_DIST_HEIGHT2 as u32;
            let z = (id as i32 - player_pos.z + RENDER_DIST).rem_euclid(RENDER_DIST2) - RENDER_DIST;
            let pos = point3(x, y, z);
            let fpos = vec3(x as f32, y as f32, z as f32);
            let mut culled = false;
            for j in 0..4 {
                if norms[j].dot(fpos) < 0.0
                    && norms[j].dot(fpos + vec3(0., 0., 1.)) < 0.0
                    && norms[j].dot(fpos + vec3(0., 1., 0.)) < 0.0
                    && norms[j].dot(fpos + vec3(0., 1., 1.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 0., 0.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 0., 1.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 1., 0.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 1., 1.)) < 0.0
                {
                    culled = false;
                    break;
                }
            }
            if !culled {
                self.rendered_chunks.push(
                    (pos.x + 128) as u32
                        + (((pos.y + 128) as u32) << 8)
                        + (((pos.z + 128) as u32) << 16),
                );
            }
        }
        queue.write_buffer(
            &self.box_buffer,
            0u64,
            bytemuck::cast_slice(&self.rendered_chunks),
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Chunk Render Encoder"),
        });
        {
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
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &context_bind_group, &[]);
            render_pass.set_bind_group(1, &self.face_bind_group, &[]);
            render_pass.set_bind_group(2, &self.diffuse_bind_group, &[]);
            render_pass.multi_draw_indirect_count(
                &self.indirect_buffer,
                0u64,
                &self.count_buffer,
                0u64,
                NB_CHUNKS * 6,
            );
            //render_pass.multi_draw_indirect(&self.indirect_buffer, 0u64, NB_CHUNKS*6);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Occlusion Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.occlusion_pipeline);
            render_pass.set_bind_group(0, &context_bind_group, &[]);
            render_pass.set_bind_group(1, &self.occlusion_bind_group, &[]);
            render_pass.draw(0..(self.rendered_chunks.len() * 18) as u32, 0..1)
        }
        {
            let mut occlusion_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Occlusion Pass"),
            });
            occlusion_pass.set_pipeline(&self.compute_pipeline);
            occlusion_pass.set_bind_group(0, &context_bind_group, &[]);
            occlusion_pass.set_bind_group(1, &self.occlusion_collect_bind_group, &[]);
            occlusion_pass.dispatch_workgroups((NB_CHUNKS / 128) + 1, 1, 1);
        }
        encoder.clear_buffer(&self.visibility_buffer, 0u64, None);
        queue.submit(iter::once(encoder.finish()));
    }
}
impl Drop for ChunkRenderer{
    fn drop(&mut self) {
        println!("drop chunkrenderer");
    }
}
