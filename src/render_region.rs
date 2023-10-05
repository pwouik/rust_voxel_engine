use crate::mesh::Face;
use glam::{ivec3, vec3, IVec3, Vec3};
use wgpu::Queue;

const RENDER_REGION_SIZE: i32 = 16;
const RENDER_REGION_SIZE_HEIGHT: i32 = 8;
pub(crate) const RENDER_REGION_CHUNKS: i32 =
    RENDER_REGION_SIZE * RENDER_REGION_SIZE * RENDER_REGION_SIZE_HEIGHT;
pub struct RenderRegion {
    index: Vec<[u32; 3]>,
    index_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    visibility_buffer: wgpu::Buffer,
    count_buffer: wgpu::Buffer,
    face_buffer: wgpu::Buffer,
    box_buffer: wgpu::Buffer,
    occlusion_collect_bind_group: wgpu::BindGroup,
    occlusion_bind_group: wgpu::BindGroup,
    bind_group: wgpu::BindGroup,
    rendered_chunks: Vec<u32>,
}

impl RenderRegion {
    pub fn new(
        device: &wgpu::Device,
        region_bind_group_layout: &wgpu::BindGroupLayout,
        occlusion_collect_bind_group_layout: &wgpu::BindGroupLayout,
        occlusion_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let face_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Storage Buffer"),
            size: (RENDER_REGION_CHUNKS) as u64 * 32 * 32 * 2 * 12 * 2,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Storage Buffer"),
            size: (RENDER_REGION_CHUNKS) as u64 * 7 * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Buffer"),
            size: (RENDER_REGION_CHUNKS * 6 * 4 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let visibility_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("visibility Buffer"),
            size: (RENDER_REGION_CHUNKS * 4 * 6) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("count Buffer"),
            size: 4u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });
        let occlusion_collect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &occlusion_collect_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: indirect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: visibility_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
            label: Some("occlusion_collect_bind_group"),
        });

        let box_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("box Buffer"),
            size: (RENDER_REGION_CHUNKS * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("face bind group"),
            layout: &region_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: face_buffer.as_entire_binding(),
            }],
        });
        Self {
            index: vec![],
            index_buffer,
            indirect_buffer,
            visibility_buffer,
            count_buffer,
            face_buffer,
            box_buffer,
            occlusion_collect_bind_group,
            occlusion_bind_group,
            bind_group,
            rendered_chunks: vec![],
        }
    }
    pub fn add_chunk(&mut self, pos: IVec3, data: &mut [Vec<Face>; 6], queue: &wgpu::Queue) {
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
                //find free space
                i += 1;
                if i >= self.index.len() || self.index[i][1] - self.index[i - 1][2] >= sizes[6] {
                    break;
                }
            }
            location = self.index[i - 1][2];
        }
        for j in 0..7 {
            //cumulative sum of sizes
            sizes[j] += location;
        }
        let id = (pos.x
            + RENDER_REGION_SIZE * pos.y
            + RENDER_REGION_SIZE * RENDER_REGION_SIZE_HEIGHT * pos.z) as u32;
        self.index.insert(i, [id, sizes[0], sizes[6]]);
        queue.write_buffer(
            &self.index_buffer,
            id as u64 * 4 * 7,
            &bytemuck::cast_slice(&sizes),
        );
        queue.write_buffer(
            &self.face_buffer,
            sizes[0] as u64 * 12,
            &bytemuck::cast_slice(&mesh),
        );
    }
    pub fn remove_chunk(&mut self, pos: IVec3, queue: &mut Queue) {
        let id = (pos.x
            + RENDER_REGION_SIZE * pos.y
            + RENDER_REGION_SIZE * RENDER_REGION_SIZE_HEIGHT * pos.z) as u32;
        let mut i = 0;
        while i < self.index.len() && self.index[i][0] != id {
            i += 1;
        }
        if i != self.index.len() {
            self.index.remove(i);
        };
        queue.write_buffer(
            &self.index_buffer,
            id as u64 * 4 * 7,
            &bytemuck::cast_slice(&[0u32; 7]),
        );
    }
    pub fn frustum(
        &mut self,
        region_pos: &IVec3,
        player_pos: IVec3,
        frustum: [Vec3; 5],
        queue: &wgpu::Queue,
    ) {
        self.rendered_chunks.clear();
        for i in 0..self.index.len() {
            let id = self.index[i][0];
            let x = id & 15;
            let y = (id >> 4) & 7;
            let z = (id >> 7) & 15;
            let pos = ivec3(
                region_pos.x + x as i32 - player_pos.x,
                region_pos.y + y as i32 - player_pos.y,
                region_pos.z + z as i32 - player_pos.z,
            );
            let fpos = vec3(pos.x as f32, pos.y as f32, pos.z as f32);
            let mut culled = false;
            for j in 1..5 {
                if frustum[j].dot(fpos - frustum[0]) < 0.0
                    && frustum[j].dot(fpos + vec3(0., 0., 1.) - frustum[0]) < 0.0
                    && frustum[j].dot(fpos + vec3(0., 1., 0.) - frustum[0]) < 0.0
                    && frustum[j].dot(fpos + vec3(0., 1., 1.) - frustum[0]) < 0.0
                    && frustum[j].dot(fpos + vec3(1., 0., 0.) - frustum[0]) < 0.0
                    && frustum[j].dot(fpos + vec3(1., 0., 1.) - frustum[0]) < 0.0
                    && frustum[j].dot(fpos + vec3(1., 1., 0.) - frustum[0]) < 0.0
                    && frustum[j].dot(fpos + vec3(1., 1., 1.) - frustum[0]) < 0.0
                {
                    culled = true;
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
    }
    pub fn occlusion<'a>(&'a mut self, occlusion_pass: &mut wgpu::RenderPass<'a>) {
        occlusion_pass.set_bind_group(2, &self.occlusion_bind_group, &[]);
        occlusion_pass.draw(0..(self.rendered_chunks.len() * 18) as u32, 0..1)
    }
    pub fn gen_commands<'a>(&'a mut self, occlusion_collect_pass: &mut wgpu::ComputePass<'a>) {
        occlusion_collect_pass.set_bind_group(1, &self.occlusion_collect_bind_group, &[]);
        occlusion_collect_pass.dispatch_workgroups(
            1,
            RENDER_REGION_SIZE_HEIGHT as u32,
            RENDER_REGION_SIZE as u32,
        );
    }
    pub fn draw<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        /*render_pass.multi_draw_indirect(
            &self.indirect_buffer,
            0u64,
            RENDER_REGION_CHUNKS as u32 * 6,
        );*/
        render_pass.multi_draw_indirect_count(
            &self.indirect_buffer,
            0u64,
            &self.count_buffer,
            0u64,
            RENDER_REGION_CHUNKS as u32 * 6,
        );
    }

    pub fn reset_visibility(&mut self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.visibility_buffer, 0u64, None);
    }
}
