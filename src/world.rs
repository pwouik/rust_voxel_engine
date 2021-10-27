use crate::mesh::*;
use crate::chunk::*;
use crate::renderer::*;
use cgmath::{Vector3, Point3};
use std::collections::{HashMap, HashSet};
use crate::block::Block;
use crate::chunk_loader::*;
use crate::camera::Camera;

const TEXTURE_INDEX:[[u32;6];3]=[[0,0,2,1,0,0],[2,2,2,2,2,2],[3,3,3,3,3,3]];

fn add_face(storage: &mut Vec<Face>, pos:[u8;3], direction:usize, texture:u32){
    storage.push(Face{ pos_dir: [pos[0],pos[1],pos[2],direction as u8], texture})
}
pub struct World{
    pub mesh_map:HashMap<Point3<i32>,Mesh>,
    pub chunk_map:HashMap<Point3<i32>,Box<Chunk>>,
    pub chunk_updates:HashSet<Point3<i32>>,
    pub chunk_loader:ChunkLoader
}

pub fn create_mesh(chunk_map:&HashMap<Point3<i32>,Box<Chunk>>,pos:Point3<i32>,renderer:&Renderer)->Mesh{
    let chunk = chunk_map.get(&pos).unwrap();
    let adjacent_chunks=[
        chunk_map.get(&(pos+Vector3::new(1,0,0))),
        chunk_map.get(&(pos+Vector3::new(0,0,1))),
        chunk_map.get(&(pos+Vector3::new(0,1,0)))];
    let mut storage = Vec::new();
    let mut index =0;
    //add_face(&mut storage,  [0 as u8,0 as u8,0 as u8], 5,TEXTURE_INDEX[0][5]);
    for y in 0..32 {
        for z in 0..32 {
            for x in 0..32 {
                let block1 = chunk.data[index];
                let mut block2= Block{ block_type: 0 };
                if x < 31{
                    block2 = chunk.data[index + 1];
                }
                else {
                    if let Some(chunk) = adjacent_chunks[0]{
                        block2=chunk.get_block(0,y,z);
                    }
                }
                let b1= block1.is_full_block();
                let mut b2 = block2.is_full_block();
                if b1 != b2 {
                    if b1 == true {
                        add_face(&mut storage,  [x as u8,y as u8,z as u8], 1,TEXTURE_INDEX[block1.block_type as usize-1][1]);
                    } else {
                        add_face(&mut storage,  [(x+1) as u8,y as u8,z as u8], 0,TEXTURE_INDEX[block2.block_type as usize-1][0]);
                    }
                }

                if z < 31
                {
                    block2 = chunk.data[index + 32];
                }
                else {
                    if let Some(chunk) = adjacent_chunks[1]{
                        block2=chunk.get_block(x,y,0);
                    }
                }
                b2= block2.is_full_block();
                if b1 != b2 {
                    if b1 == true {
                        add_face(&mut storage,  [x as u8,y as u8,z as u8], 5,TEXTURE_INDEX[block1.block_type as usize-1][5]);
                    } else {
                        add_face(&mut storage, [x as u8,y as u8,(z+1) as u8], 4,TEXTURE_INDEX[block2.block_type as usize-1][4]);
                    }
                }

                if y < 31
                {
                    block2 = chunk.data[index + 32 * 32];
                }
                else {
                    if let Some(chunk) = adjacent_chunks[2]{
                        block2 = chunk.get_block(x,0,z);
                    }
                }
                b2= block2.is_full_block();
                if b1 != b2 {
                    if b1 == true {
                        add_face(&mut storage,  [x as u8,y as u8,z as u8], 3,TEXTURE_INDEX[block1.block_type as usize-1][3]);
                    } else {
                        add_face(&mut storage,  [x as u8,(y+1) as u8,z as u8], 2,TEXTURE_INDEX[block2.block_type as usize-1][2]);
                    }
                }
                index += 1;
            }
        }
    }
    let storage_buffer = renderer.create_storage_buffer(&storage);
    let mut bind_group=None;
    if storage.len()!=0{
        bind_group = Some(renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.chunk_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        }));
    }
    Mesh{
        storage_buffer,
        bind_group,
        num_elements: storage.len() as u32 *6,
    }
}
impl World{
    pub fn new()->World{
        let mesh_map:HashMap<Point3<i32>,Mesh>=HashMap::new();
        let chunk_map:HashMap<Point3<i32>,Box<Chunk>>=HashMap::new();
        let chunk_loader=ChunkLoader::new();
        let chunk_updates= HashSet::new();
        World{
            mesh_map,
            chunk_map,
            chunk_loader,
            chunk_updates,
        }
    }
    fn add_chunks(&mut self){
        loop{
            let chunk_result=self.chunk_loader.try_get_chunk();
            match chunk_result{
                Some(chunk) => {
                    self.chunk_map.insert(chunk.0,chunk.1);
                    self.chunk_updates.insert(chunk.0);
                    self.chunk_updates.insert(chunk.0+Vector3::new(-1,0,0));
                    self.chunk_updates.insert(chunk.0+Vector3::new(0,-1,0));
                    self.chunk_updates.insert(chunk.0+Vector3::new(0,0,-1));
                }
                None => {break}
            }
        }
    }
    fn unload_chunks(&mut self, player_pos:Point3<i32>){
        self.chunk_map.retain(|pos, _| {
            let rel_pos= pos-player_pos;
            rel_pos.x > -RENDER_DIST && rel_pos.x <RENDER_DIST && rel_pos.y >= -RENDER_DIST_HEIGHT && rel_pos.y <=RENDER_DIST_HEIGHT && rel_pos.z > -RENDER_DIST && rel_pos.z <RENDER_DIST
        });
        self.mesh_map.retain(|pos, _| {
            let rel_pos= pos-player_pos;
            rel_pos.x > -RENDER_DIST && rel_pos.x <RENDER_DIST && rel_pos.y >= -RENDER_DIST_HEIGHT && rel_pos.y <=RENDER_DIST_HEIGHT && rel_pos.z > -RENDER_DIST && rel_pos.z <RENDER_DIST
        })
    }
    pub fn tick(&mut self,renderer:&Renderer,camera:&Camera){
        let player_pos= Point3::new((camera.pos.x/32.0) as i32,(camera.pos.y/32.0) as i32,(camera.pos.z/32.0) as i32);
        self.unload_chunks(player_pos);
        self.chunk_loader.tick(&self.chunk_map,player_pos);
        self.add_chunks();
    }
    pub fn update_display(&mut self,renderer:&Renderer){
        for pos in self.chunk_updates.drain(){
            if let Some(mesh)=self.mesh_map.get(&pos){
                mesh.destroy();
            }
            if let Some(_chunk)=self.chunk_map.get(&pos){
                self.mesh_map.insert(pos,create_mesh(&self.chunk_map,pos,renderer));
            }
        }
    }
}