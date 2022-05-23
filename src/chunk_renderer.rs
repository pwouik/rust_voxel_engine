use std::{fs, iter};
use std::num::NonZeroU32;
use cgmath::{EuclideanSpace, Point3, point3, vec3};
use wgpu::util::DeviceExt;
use crate::mipmap;
use crate::mesh::Face;
use crate::texture::*;
use crate::chunk_loader::{NB_CHUNKS, RENDER_DIST, RENDER_DIST2, RENDER_DIST_HEIGHT, RENDER_DIST_HEIGHT2};
pub const IMAGES:[&str;4]=["textures/grass_side.png","textures/grass_top.png","textures/grass_bottom.png","textures/stone.png"];

pub struct ChunkRenderer {
    index:Vec<[u32;3]>,
    rendered_chunks:Vec<u32>,
    box_buffer:wgpu::Buffer,
    count_buffer:wgpu::Buffer,
    visibility_buffer:wgpu::Buffer,
    indirect_buffer:wgpu::Buffer,
    indirect_buffer2:wgpu::Buffer,
    chunk_table_buffer:wgpu::Buffer,
    face_buffer:wgpu::Buffer,
    face_bind_group:wgpu::BindGroup,
    diffuse_bind_group:wgpu::BindGroup,
    occlusion_bind_group: wgpu::BindGroup,
    occlusion_collect_bind_group: wgpu::BindGroup,
    vs_module:wgpu::ShaderModule,
    fs_module:wgpu::ShaderModule,
    textures:Vec<Texture>,
    occlusion_pipeline:wgpu::RenderPipeline,
    compute_pipeline:wgpu::ComputePipeline,
    render_pipeline:wgpu::RenderPipeline,
}
impl ChunkRenderer {
    pub fn new(device: &wgpu::Device, init_encoder: &mut wgpu::CommandEncoder,queue: &wgpu::Queue, config:&wgpu::SurfaceConfiguration, context_bind_group_layout:&wgpu::BindGroupLayout) -> Self {
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
                        count: NonZeroU32::new(IMAGES.len() as u32),
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
        let mut textures2 = vec![];
        let mut textures_views = vec![];
        for i in 0..IMAGES.len() {
            let diffuse_bytes = fs::read(IMAGES[i]).expect("texture could not be loaded");
            textures2.push(Texture::from_bytes(&device, &queue, &diffuse_bytes, IMAGES[i]).unwrap());
            mipmap::generate_mipmaps(
                init_encoder,
                &device,
                &textures2[i].texture,
                MIP_LEVEL_COUNT,
            );
        }
        let textures = textures2;
        for i in 0..IMAGES.len() {
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
        let box_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("box Buffer"),
                size: (NB_CHUNKS*4).into(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            }
        );
        let visibility_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("visibility Buffer"),
                size: (NB_CHUNKS*4).into(),
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false
            }
        );
        let occlusion_bind_group_layout =
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
                },wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("occlusion_bind_group_layout"),
            });
        let occlusion_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &occlusion_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: box_buffer.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: visibility_buffer.as_entire_binding(),
            }],
            label: Some("occlusion_bind_group"),
        });
        let count_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Count Buffer"),
                contents: bytemuck::cast_slice(&[0 as u32]),
                usage: wgpu::BufferUsages::STORAGE,
            }
        );
        let indirect_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Indirect Buffer"),
                size: (NB_CHUNKS * 4 * 4).into(),
                usage: wgpu::BufferUsages::STORAGE|wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false
            }
        );
        let indirect_buffer2 = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Indirect Buffer"),
                size: (NB_CHUNKS * 4 * 4).into(),
                usage: wgpu::BufferUsages::INDIRECT|wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            }
        );
        let chunk_table_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Storage Buffer"),
                size: (NB_CHUNKS * 4).into(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            }
        );
        let occlusion_collect_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage{ read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("command_bind_group_layout"),
            });
        let occlusion_collect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &occlusion_collect_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: count_buffer.as_entire_binding(),
            },wgpu::BindGroupEntry {
                binding: 1,
                resource: visibility_buffer.as_entire_binding(),
            },wgpu::BindGroupEntry {
                binding: 2,
                resource: indirect_buffer.as_entire_binding(),
            },wgpu::BindGroupEntry {
                binding: 3,
                resource: chunk_table_buffer.as_entire_binding(),
            }],
            label: Some("occlusion_collect_bind_group"),
        });
        let vs_module = unsafe { device.create_shader_module_spirv(&wgpu::include_spirv_raw!("chunkv.spv")) };
        let fs_module = unsafe { device.create_shader_module_spirv(&wgpu::include_spirv_raw!("chunkf.spv")) };
        let vs_occlusion_module = unsafe { device.create_shader_module_spirv(&wgpu::include_spirv_raw!("occlusionv.spv")) };
        let fs_occlusion_module = unsafe { device.create_shader_module_spirv(&wgpu::include_spirv_raw!("occlusionf.spv")) };
        let cs_occlusion_module = unsafe { device.create_shader_module_spirv(&wgpu::include_spirv_raw!("occlusion_collect.spv")) };
        let occlusion_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&context_bind_group_layout,&occlusion_bind_group_layout],
                push_constant_ranges: &[]
            });
        let occlusion_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Occlusion Pipeline"),
            layout: Some(&occlusion_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_occlusion_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_occlusion_module,
                entry_point: "main",
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None
        });
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&occlusion_collect_bind_group_layout, &context_bind_group_layout],
            push_constant_ranges: &[]
        });
        let compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("compute_pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &(cs_occlusion_module),
                entry_point: "main"
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
        let face_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Storage Buffer"),
                size: (NB_CHUNKS* 32 *32 * 3 * 12).into(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST ,
                mapped_at_creation: false
            }
        );
        let face_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("face bind group"),
            layout: &face_bind_group_layout,
            entries: &[wgpu::BindGroupEntry{ binding: 0, resource: face_buffer.as_entire_binding() }]
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&context_bind_group_layout, &face_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Chunk Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[],
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
            multiview: None
        });


        Self {
            index: vec![],
            rendered_chunks: vec![],
            box_buffer,
            count_buffer,
            visibility_buffer,
            indirect_buffer,
            indirect_buffer2,
            chunk_table_buffer,
            face_buffer,
            face_bind_group,
            textures,
            diffuse_bind_group,
            occlusion_bind_group,
            occlusion_collect_bind_group,
            vs_module,
            fs_module,
            render_pipeline,
            compute_pipeline,
            occlusion_pipeline,
        }
    }

    pub fn add_chunk(&mut self, pos: Point3<i32>, data:&mut [Vec<Face>; 6], queue: &wgpu::Queue) {
        let mut sizes = [0 as u32; 7];
        let mut mesh = vec![];
        for i in 1..7 {
            mesh.append(&mut data[i-1]);
            sizes[i] = sizes[i - 1] + data[i-1].len() as u32;
        }
        let mut i:usize = 0;
        let mut location = 0;
        if self.index.len()!=0 || self.index[0][1]<sizes[5] {
            while i < self.index.len() - 1 {
                if self.index[i + 1][1] - self.index[i][2] >= sizes[5] {
                    break;
                }
                i += 1;
            }
            location = self.index[i][2];
        }
        for i in 0..7 {
            sizes[i] += location;
        }
        let id= (pos.x.rem_euclid(RENDER_DIST2) +
            RENDER_DIST2 * pos.y.rem_euclid(RENDER_DIST_HEIGHT2) +
            RENDER_DIST2 * RENDER_DIST_HEIGHT2 * pos.z.rem_euclid(RENDER_DIST2) * 4 * 7) as u32;
        self.index.insert(i as usize,[id,sizes[0],sizes[6]]);
        queue.write_buffer(&self.chunk_table_buffer,id.into(),&bytemuck::cast_slice(&sizes));
        queue.write_buffer(&self.face_buffer, self.index[i][1].into(), &bytemuck::cast_slice(&mesh));
    }

    pub fn remove_chunk(&mut self,pos:Point3<i32>){
        let id= (pos.x.rem_euclid(RENDER_DIST2) +
            RENDER_DIST2 * pos.y.rem_euclid(RENDER_DIST_HEIGHT2) +
            RENDER_DIST2 * RENDER_DIST_HEIGHT2 * pos.z.rem_euclid(RENDER_DIST2) * 4 * 7) as u32;
        let i=0;
        for i in 0..self.index.len(){
            if self.index[i][0]==id{break;}
        }
        if i!=self.index.len(){self.index.remove(i);};
        //queue.write_buffer(&self.chunk_table_buffer,id.into(),&vec![0u32;7]);
    }

    pub fn render_chunks(&mut self, device: &wgpu::Device,queue:&wgpu::Queue,view:&wgpu::TextureView, depth_texture:&Texture, player_pos:Point3<i32>, norms: [Point3<f32>; 4], context_bind_group:&wgpu::BindGroup) {
        self.rendered_chunks.clear();
        for i in 0..self.index.len(){
            let mut id = self.index[i][0];
            let x = (id as i32 - player_pos.x).rem_euclid(RENDER_DIST2) - RENDER_DIST;
            id /= RENDER_DIST2 as u32;
            let y = (id as i32 - player_pos.y).rem_euclid(RENDER_DIST_HEIGHT2) - RENDER_DIST_HEIGHT;
            id /= RENDER_DIST_HEIGHT2 as u32;
            let z = (id as i32 - player_pos.x).rem_euclid(RENDER_DIST2) - RENDER_DIST;
            let pos=point3(x,y,z);
            let fpos=vec3(x as f32,y as f32,z as f32);
            let mut culled=false;
            for j in 0..4 {
                if norms[j].dot(fpos) < 0.0
                    && norms[j].dot(fpos + vec3(0., 0., 1.)) < 0.0
                    && norms[j].dot(fpos + vec3(0., 1., 0.)) < 0.0
                    && norms[j].dot(fpos + vec3(0., 1., 1.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 0., 0.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 0., 1.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 1., 0.)) < 0.0
                    && norms[j].dot(fpos + vec3(1., 1., 1.)) < 0.0 {
                    culled=true;
                    break;
                }
            }
            if !culled{
                self.rendered_chunks.push((pos.x as u32+128)+(pos.y as u32+128)<<5+(pos.y as u32+128)<<10);
            }
        }
        queue.write_buffer(&self.box_buffer,0u64,bytemuck::cast_slice(&self.rendered_chunks));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Chunk Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Chunk Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
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
                }],
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
            render_pass.multi_draw_indirect_count(&self.indirect_buffer2, 0u64, &self.count_buffer, 0u64, (RENDER_DIST * RENDER_DIST * RENDER_DIST_HEIGHT) as u32);
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
            render_pass.set_bind_group(0, &self.occlusion_bind_group, &[]);
            render_pass.set_bind_group(1, &context_bind_group, &[]);
            render_pass.draw(0..(self.rendered_chunks.len()*4) as u32,0..1)

        }
        {
            let mut occlusion_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Occlusion Pass") });
            occlusion_pass.set_pipeline(&self.compute_pipeline);
            occlusion_pass.set_bind_group(0, &context_bind_group, &[]);
            occlusion_pass.set_bind_group(1, &self.occlusion_collect_bind_group, &[]);
            occlusion_pass.dispatch(NB_CHUNKS, 6, 1)

        }
        encoder.copy_buffer_to_buffer(&self.indirect_buffer,0u64,&self.indirect_buffer2,0u64, NB_CHUNKS as u64 * 4 * 4);
        queue.submit(iter::once(encoder.finish()));
    }
}