use crate::mesh::*;
use crate::renderer::*;
use cgmath::{Vector3, Point3, point3, vec3};
use cgmath::EuclideanSpace;
use std::collections::{HashMap, HashSet};
use crate::block::Block;
use crate::chunk_loader::*;
use crate::camera::Camera;
use crate::util::direction::*;
use crate::chunk_map::ChunkMap;

const TEXTURE_INDEX:[[u32;6];3]=[[0,0,0,0,2,1],[2,2,2,2,2,2],[3,3,3,3,3,3]];


pub struct World{
    pub mesh_map:HashMap<Point3<i32>,Mesh>,
    pub chunk_map:ChunkMap,
    chunk_updates:HashSet<Point3<i32>>,
    chunk_loader:ChunkLoader,
}

impl World{
    pub fn new()->World{
        let mesh_map:HashMap<Point3<i32>,Mesh>=HashMap::new();
        let chunk_map=ChunkMap::new();
        let chunk_loader=ChunkLoader::new();
        let chunk_updates= HashSet::new();
        World{
            mesh_map,
            chunk_map,
            chunk_loader,
            chunk_updates
        }
    }
    #[profiling::function]
    fn add_chunks(&mut self){
        loop{
            let chunk_result=self.chunk_loader.try_get_chunk();
            match chunk_result{
                Some(chunk) => {
                    self.chunk_map.hash_map.insert(chunk.0,chunk.1);
                    self.chunk_updates.insert(chunk.0);
                    self.chunk_updates.insert(chunk.0+vec3(-1,0,0));
                    self.chunk_updates.insert(chunk.0+vec3(0,-1,0));
                    self.chunk_updates.insert(chunk.0+vec3(0,0,-1));
                    self.chunk_updates.insert(chunk.0+vec3(1,0,0));
                    self.chunk_updates.insert(chunk.0+vec3(0,1,0));
                    self.chunk_updates.insert(chunk.0+vec3(0,0,1));
                }
                None => {break}
            }
        }
    }
    #[profiling::function]
    fn unload_chunks(&mut self, player_pos:Point3<i32>){
        let unloaded=self.chunk_map.hash_map.drain_filter(|pos, _| {
            let rel_pos= pos-player_pos;
            rel_pos.x <= -RENDER_DIST || rel_pos.x >= RENDER_DIST || rel_pos.y < -RENDER_DIST_HEIGHT || rel_pos.y > RENDER_DIST_HEIGHT || rel_pos.z <= -RENDER_DIST || rel_pos.z >= RENDER_DIST
        });
        for i in unloaded
        {
            self.chunk_loader.save(i);
        }
        self.mesh_map.retain(|pos, _| {
            let rel_pos= pos-player_pos;
            rel_pos.x > -RENDER_DIST && rel_pos.x <RENDER_DIST && rel_pos.y >= -RENDER_DIST_HEIGHT && rel_pos.y <=RENDER_DIST_HEIGHT && rel_pos.z > -RENDER_DIST && rel_pos.z < RENDER_DIST
        });
    }
    #[profiling::function]
    pub fn tick(&mut self,camera:&Camera){
        let player_pos= point3((camera.pos.x/32.0) as i32,(camera.pos.y/32.0) as i32,(camera.pos.z/32.0) as i32);
        self.unload_chunks(player_pos);
        self.chunk_loader.tick(&self.chunk_map,player_pos);
        self.add_chunks();
    }
    #[profiling::function]
    pub fn update_display(&mut self,renderer:&mut Renderer){
        let positions :Vec<Point3<i32>> = self.chunk_updates.drain().collect();
        for pos in positions{
            if self.chunk_map.get_chunk(pos).is_some(){
                if let Some(mesh)=self.mesh_map.get(&pos){
                    mesh.destroy();
                }
                self.mesh_map.insert(pos,self.create_mesh(pos,renderer));
            }
        }
    }
    #[profiling::function]
    fn add_face(&self,storage: &mut Vec<Face>, pos:Point3<i32>,chunk_pos:Point3<i32>, dir:Direction, texture:u32){
        let global_pos= point3(chunk_pos.x<<5,chunk_pos.y<<5,chunk_pos.z<<5)+pos.to_vec();
        let ao_blocks=[
            self.chunk_map.get_block(global_pos+dir.transform(vec3(1,1,0))).is_full_block(),
            self.chunk_map.get_block(global_pos+dir.transform(vec3(1,1,1))).is_full_block(),
            self.chunk_map.get_block(global_pos+dir.transform(vec3(0,1,1))).is_full_block(),
            self.chunk_map.get_block(global_pos+dir.transform(vec3(-1,1,1))).is_full_block(),
            self.chunk_map.get_block(global_pos+dir.transform(vec3(-1,1,0))).is_full_block(),
            self.chunk_map.get_block(global_pos+dir.transform(vec3(-1,1,-1))).is_full_block(),
            self.chunk_map.get_block(global_pos+dir.transform(vec3(0,1,-1))).is_full_block(),
            self.chunk_map.get_block(global_pos+dir.transform(vec3(1,1,-1))).is_full_block(),
        ];
        let light:[u8;4]=[
            255-(ao_blocks[4] || ao_blocks[5] || ao_blocks[6]) as u8 *200,
            255-(ao_blocks[2] || ao_blocks[3] || ao_blocks[4]) as u8 *200,
            255-(ao_blocks[0] || ao_blocks[1] || ao_blocks[2]) as u8 *200,
            255-(ao_blocks[6] || ao_blocks[7] || ao_blocks[0]) as u8 *200];
        storage.push(Face{ pos_dir: [pos.x as u8,pos.y as u8,pos.z as u8,dir.id as u8], texture, light})
    }
    #[profiling::function]
    pub fn create_mesh(&self,pos:Point3<i32>,renderer:&Renderer)->Mesh{
        let chunk = self.chunk_map.get_chunk(pos).unwrap();
        let adjacent_chunks=[
            self.chunk_map.get_chunk(pos+vec3(1,0,0)),
            self.chunk_map.get_chunk(pos+vec3(0,0,1)),
            self.chunk_map.get_chunk(pos+vec3(0,1,0))];
        let mut storage = Vec::new();
        let mut index =0;
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
                            block2=chunk.get_block(point3(0,y,z));
                        }
                    }
                    let b1= block1.is_full_block();
                    let mut b2 = block2.is_full_block();
                    if b1 != b2 {
                        if b1 == true {
                            self.add_face(&mut storage,  point3(x as i32,y as i32,z as i32),pos, Direction{id:1},TEXTURE_INDEX[block1.block_type as usize-1][1]);
                        } else {
                            self.add_face(&mut storage,  point3((x+1) as i32,y as i32,z as i32),pos, Direction{id:0},TEXTURE_INDEX[block2.block_type as usize-1][0]);
                        }
                    }

                    if z < 31
                    {
                        block2 = chunk.data[index + 32];
                    }
                    else {
                        if let Some(chunk) = adjacent_chunks[1]{
                            block2=chunk.get_block(point3(x,y,0));
                        }
                    }
                    b2= block2.is_full_block();
                    if b1 != b2 {
                        if b1 == true {
                            self.add_face(&mut storage,  point3(x as i32,y as i32,z as i32),pos, Direction{id:3},TEXTURE_INDEX[block1.block_type as usize-1][3]);
                        } else {
                            self.add_face(&mut storage, point3(x as i32,y as i32,(z+1) as i32),pos, Direction{id:2},TEXTURE_INDEX[block2.block_type as usize-1][2]);
                        }
                    }

                    if y < 31
                    {
                        block2 = chunk.data[index + 32 * 32];
                    }
                    else {
                        if let Some(chunk) = adjacent_chunks[2]{
                            block2 = chunk.get_block(point3(x,0,z));
                        }
                    }
                    b2= block2.is_full_block();
                    if b1 != b2 {
                        if b1 == true {
                            self.add_face(&mut storage,  point3(x as i32,y as i32,z as i32),pos, Direction{id:5},TEXTURE_INDEX[block1.block_type as usize-1][5]);
                        } else {
                            self.add_face(&mut storage,  point3(x as i32,(y+1) as i32,z as i32),pos, Direction{id:4},TEXTURE_INDEX[block2.block_type as usize-1][4]);
                        }
                    }
                    index += 1;
                }
            }
        }
        let storage_buffer = renderer.create_face_buffer(&storage);
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
            num_elements: storage.len() as u32 * 6,
        }
    }
    #[profiling::function]
    pub fn set_block(&mut self,pos:Point3<i32>,value:Block){
        let chunk_pos=point3(pos.x >> 5, pos.y >> 5, pos.z >> 5);
        if let Some(chunk) = self.chunk_map.get_chunk_mut(chunk_pos) {
            let loc_pos=point3((pos.x as u32) & 31,(pos.y as u32) & 31,(pos.z as u32) & 31);
            self.chunk_updates.insert(chunk_pos);

            if loc_pos.x==0{
                self.chunk_updates.insert(chunk_pos+vec3(-1,0,0));
            }
            else if loc_pos.x==31{
                self.chunk_updates.insert(chunk_pos+vec3(1,0,0));
            }
            if loc_pos.y==0{
                self.chunk_updates.insert(chunk_pos+vec3(0,-1,0));
            }
            else if loc_pos.y==31{
                self.chunk_updates.insert(chunk_pos+vec3(0,1,0));
            }
            if loc_pos.z==0{
                self.chunk_updates.insert(chunk_pos+vec3(0,0,-1));
            }
            else if loc_pos.z==31{
                self.chunk_updates.insert(chunk_pos+vec3(0,0,1));
            }
            chunk.set_block(loc_pos,value);
        }
    }
    #[profiling::function]
    pub fn raycast(&self,pos:Point3<f32>,dir:Vector3<f32>,place:bool)->Point3<i32>{
        let mut block_pos = point3(pos.x.floor() as i32, pos.y.floor() as i32, pos.z.floor() as i32);
        let mut side_dist_x;
        let mut side_dist_y;
        let mut side_dist_z;
        let delta_dist_x = (1.0 / dir.x).abs();
        let delta_dist_y = (1.0 / dir.y).abs();
        let delta_dist_z = (1.0 / dir.z).abs();
        let step_x;
        let step_y;
        let step_z;
        let mut side =0;
        if dir.x < 0.0
        {
            step_x = -1;
            side_dist_x = (pos.x - block_pos.x as f32) * delta_dist_x;
        }
        else
        {
            step_x = 1;
            side_dist_x = (block_pos.x as f32 - pos.x+1.0 ) * delta_dist_x;
        }
        if dir.y < 0.0
        {
            step_y = -1;
            side_dist_y = (pos.y - block_pos.y as f32) * delta_dist_y;
        }
        else
        {
            step_y = 1;
            side_dist_y = (block_pos.y as f32 - pos.y+1.0) * delta_dist_y;
        }
        if dir.z < 0.0
        {
            step_z = -1;
            side_dist_z = (pos.z - block_pos.z as f32) * delta_dist_z;
        }
        else
        {
            step_z = 1;
            side_dist_z = (block_pos.z as f32 - pos.z+1.0) * delta_dist_z;
        }
        let mut i=0;
        while !self.chunk_map.get_block(block_pos).is_full_block() && i<200
        {
            i+=1;
            if side_dist_x < side_dist_y && side_dist_x < side_dist_z
            {
                side_dist_x += delta_dist_x;
                block_pos.x += step_x;
                side=1;
            }
            else if side_dist_y < side_dist_z
            {
                side_dist_y += delta_dist_y;
                block_pos.y += step_y;
                side=2;
            }
            else
            {
                side_dist_z += delta_dist_z;
                block_pos.z += step_z;
                side=3;
            }
        }
        return if place {
            let mut face = vec3(0, 0, 0);
            match side {
                1 => {
                    if dir.x < 0.0 {
                        face.x = 1;
                    } else {
                        face.x = -1;
                    }
                }
                2 => {
                    if dir.y < 0.0 {
                        face.y = 1;
                    } else {
                        face.y = -1;
                    }
                }
                _ => {
                    if dir.z < 0.0 {
                        face.z = 1;
                    } else {
                        face.z = -1;
                    }
                }
            }
            block_pos + face
        } else {
            block_pos
        }
    }
}

impl Drop for World {
    fn drop(&mut self) {
        for i in self.chunk_map.hash_map.drain()
        {
            self.chunk_loader.save(i);
        }
    }
}