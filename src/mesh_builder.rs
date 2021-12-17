use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use cgmath::{Point3, point3, vec3};
use futures::task::noop_waker;
use wgpu;
use wgpu::util::DeviceExt;
use crate::Renderer;
use crate::block::Block;
use crate::chunk::Chunk;
use crate::chunk_map::ChunkMap;
use crate::mesh::Mesh;

fn to_34_cube_index(x:i32,y:i32,z:i32)->usize{
    (x + z * 34 + y * 34 * 34) as usize
}
pub struct MeshBuilder{
    count:usize,
    buffers:Vec<wgpu::Buffer>,
    tasks_results:Vec<(Point3<i32>,Pin<Box<dyn Future<Output=Result<(), wgpu::BufferAsyncError>>>>)>,
    compute_pipeline:wgpu::ComputePipeline
}
impl MeshBuilder
{
    pub fn new(renderer:&Renderer)->Self{
        let cs_module = renderer.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("chunk_mesh.wgsl"))),
        });
        let compute_pipeline = renderer.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &cs_module,
            entry_point: "main",
        });
        MeshBuilder{
            count: 0,
            tasks_results:Vec::new(),
            buffers:Vec::new(),
            compute_pipeline
        }
    }
    pub fn get_computing_meshes(&self)->usize{
        self.count
    }
    pub fn mesh_chunk(&mut self,pos:Point3<i32>,chunk_map:&ChunkMap,renderer:&mut Renderer) {
        if let Some(center_chunk) = chunk_map.get_chunk(pos){
            let mut data = [0; 34 * 43 * 43];
            for y in 0..32 {
                for z in 0..32 {
                    for x in 0..32 {
                        data[to_34_cube_index(x + 1, y + 1, z+1)] = center_chunk.get_block(point3(x as u32,y as u32,z as u32)).block_type as u32;
                    }
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, 0, -1)) {
                for y in 0..32 {
                    for x in 0..32 {
                        data[to_34_cube_index(x + 1, y + 1, 0)] = chunk.get_block(point3(x as u32,y as u32,31)).block_type as u32;
                    }
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, 0, 1)) {
                for y in 0..32 {
                    for x in 0..32 {
                        data[to_34_cube_index(x + 1, y + 1, 33)] = chunk.get_block(point3(x as u32,y as u32,0)).block_type as u32;
                    }
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, -1, 0)) {
                for z in 0..32 {
                    for x in 0..32 {
                        data[to_34_cube_index(x + 1, 0, z+1)] = chunk.get_block(point3(x as u32,31,z as u32)).block_type as u32;
                    }
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, 1, 0)) {
                for z in 0..32 {
                    for x in 0..32 {
                        data[to_34_cube_index(x + 1, 33, z+1)] = chunk.get_block(point3(x as u32,0,z as u32)).block_type as u32;
                    }
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, 1, 1)) {
                for x in 0..32 {
                    data[to_34_cube_index(x + 1, 33, 33)] = chunk.get_block(point3(x as u32,0,0)).block_type as u32;
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, -1, 1)) {
                for x in 0..32 {
                    data[to_34_cube_index(x + 1, 0, 33)] = chunk.get_block(point3(x as u32,31,0)).block_type as u32;
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, -1, -1)) {
                for x in 0..32 {
                    data[to_34_cube_index(x + 1, 0, 0)] = chunk.get_block(point3(x as u32,31,31)).block_type as u32;
                }
            }
            if let Some(chunk) = chunk_map.get_chunk(pos + vec3(0, 1, -1)) {
                for x in 0..32 {
                    data[to_34_cube_index(x + 1, 33, 0)] = chunk.get_block(point3(x as u32,0,31)).block_type as u32;
                }
            }
            for y in -1..33{
                for z in -1..33{
                    data[to_34_cube_index(0,y+1,z+1)]=chunk_map.get_block(point3((pos.x<<5)-1 ,(pos.y<<5)+y,(pos.z<<5)+z)).block_type as u32;
                    data[to_34_cube_index(33,y+1,z+1)]=chunk_map.get_block(point3((pos.x<<5)+32 ,(pos.y<<5)+y,(pos.z<<5)+z)).block_type as u32;
                }
            }
            let chunk_buffer = renderer.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Storage Buffer"),
                    contents: bytemuck::cast_slice(&data),
                    usage: wgpu::BufferUsages::STORAGE,
                }
            );
            let staging_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size:4,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let atomic_buffer = renderer.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Storage Buffer"),
                    contents: &[0,0,0,0],
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                }
            );
            self.buffers.push(renderer.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Storage Buffer"),
                    contents: bytemuck::cast_slice(&data),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                }
            ));
            let bind_group_layout = self.compute_pipeline.get_bind_group_layout(0);
            let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: chunk_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.buffers[self.buffers.len()-1].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: atomic_buffer.as_entire_binding(),
                }],
            });
            let mut encoder =
                renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                cpass.set_pipeline(&self.compute_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.insert_debug_marker("generate chunk mesh");
                cpass.dispatch(1, 1, 32);
            }
            encoder.copy_buffer_to_buffer(&atomic_buffer, 0, &staging_buffer, 0, 4);
            renderer.queue.submit(Some(encoder.finish()));

            self.tasks_results.push((pos,Box::pin(staging_buffer.slice(..).map_async(wgpu::MapMode::Read))));
            self.count+=1;
        }
    }
    pub fn check_for_meshes(&mut self,mesh_map:&mut HashMap<Point3<i32>,Mesh>,renderer:&Renderer){
        let mut i = 0;
        while i < self.tasks_results.len() {
            let waker = noop_waker();
            let mut ctx = Context::from_waker(&waker);
            match Pin::new(&mut self.tasks_results[i].1).poll(&mut ctx){
                Poll::Ready(result)=>{
                    if let Ok(()) = result{
                        let size;
                        {
                            let slice = self.buffers[i].slice(..).get_mapped_range();
                            let size_slice: &[u32] = bytemuck::cast_slice(&slice);
                            size = size_slice[0];
                        }
                        if let Some(mesh) = mesh_map.get_mut(&self.tasks_results[i].0) {
                            mesh.storage_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
                                label: None,
                                size: size as wgpu::BufferAddress,
                                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            });
                            let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                            encoder.copy_buffer_to_buffer(&self.buffers[i], 0, &mesh.storage_buffer, 0, size as wgpu::BufferAddress);
                            renderer.queue.submit(Some(encoder.finish()));
                            self.tasks_results.swap_remove(i);
                            self.buffers.swap_remove(i);
                        }
                    }
                }
                Poll::Pending=>{i+=1;}
            }
        }
    }
}