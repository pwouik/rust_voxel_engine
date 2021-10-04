use crate::mesh::*;
use crate::chunk::*;
use crate::renderer::*;
use cgmath::{Vector3, Point3};
use std::collections::{HashMap, HashSet};
use crate::chunk_loader::*;
use crate::camera::Camera;
use crate::block::Block;

const TEXTURE_INDEX:[[u32;6];1]=[[0,0,2,1,0,0]];
const FACES:[[f32;12];6]= [
    [0.0, 0.0, 0.0,
        0.0, 0.0, 1.0,
        0.0, 1.0, 1.0,
        0.0, 1.0, 0.0],
    [1.0, 0.0, 1.0,
        1.0, 0.0, 0.0,
        1.0, 1.0, 0.0,
        1.0, 1.0, 1.0],
    [0.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 1.0,
        0.0, 0.0, 1.0],
    [0.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0, 0.0,
        0.0, 1.0, 0.0],
    [1.0, 0.0, 0.0,
        0.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        1.0, 1.0, 0.0],
    [0.0, 0.0, 1.0,
        1.0, 0.0, 1.0,
        1.0, 1.0, 1.0,
        0.0, 1.0, 1.0]
];
fn add_face(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, pos:[f32;3], direction:usize, texture_id:u32){
    let index:u32 = vertices.len() as u32;
    indices.push(index);
    indices.push(index+1);
    indices.push(index+2);
    indices.push(index+2);
    indices.push(index+3);
    indices.push(index);
    vertices.push(Vertex{ pos: [
        pos[0]+FACES[direction][0],
        pos[1]+FACES[direction][1],
        pos[2]+FACES[direction][2]], texture:texture_id });
    vertices.push(Vertex{ pos: [
        pos[0]+FACES[direction][3],
        pos[1]+FACES[direction][4],
        pos[2]+FACES[direction][5]], texture:texture_id });
    vertices.push(Vertex{ pos: [
        pos[0]+FACES[direction][6],
        pos[1]+FACES[direction][7],
        pos[2]+FACES[direction][8]], texture:texture_id });
    vertices.push(Vertex{ pos: [
        pos[0]+FACES[direction][9],
        pos[1]+FACES[direction][10],
        pos[2]+FACES[direction][11]], texture:texture_id });
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
    let mut indices = Vec::new();
    let mut vertices = Vec::new();
    let mut index:usize=0;
    for y in 0..32 {
        for z in 0..32 {
            for x in 0..32 {
                let b1 = chunk.data[index].is_full_block();
                let mut b2= false;
                if x < 31{
                    b2 = chunk.data[index + 1].is_full_block();
                }
                else {
                    if let Some(chunk) = adjacent_chunks[0]{
                        b2=chunk.get_block(0,y,z).is_full_block();
                    }
                }
                if b1 != b2 {
                    if b1 == true {
                        add_face(&mut vertices, &mut indices, [x as f32,y as f32,z as f32], 1,TEXTURE_INDEX[0][1]);
                    } else {
                        add_face(&mut vertices, &mut indices, [(x+1) as f32,y as f32,z as f32], 0,TEXTURE_INDEX[0][0]);
                    }
                }

                if z < 31
                {
                    b2 = chunk.data[index + 32].is_full_block();
                }
                else {
                    if let Some(chunk) = adjacent_chunks[1]{
                        b2=chunk.get_block(x,y,0).is_full_block();
                    }
                }
                if b1 != b2 {
                    if b1 == true {
                        add_face(&mut vertices, &mut indices, [x as f32,y as f32,z as f32], 5,TEXTURE_INDEX[0][5]);
                    } else {
                        add_face(&mut vertices, &mut indices, [x as f32,y as f32,(z+1) as f32], 4,TEXTURE_INDEX[0][4]);
                    }
                }

                if y < 31
                {
                    b2 = chunk.data[index + 32 * 32].is_full_block();
                }
                else {
                    if let Some(chunk) = adjacent_chunks[2]{
                        b2=chunk.get_block(x,0,z).is_full_block();
                    }
                }
                if b1 != b2 {
                    if b1 == true {
                        add_face(&mut vertices, &mut indices, [x as f32,y as f32,z as f32], 3,TEXTURE_INDEX[0][3]);
                    } else {
                        add_face(&mut vertices, &mut indices, [x as f32,(y+1) as f32,z as f32], 2,TEXTURE_INDEX[0][2]);
                    }
                }
                index += 1;
            }
        }
    }
    let vertex_buffer = renderer.create_vertex_buffer(&vertices);
    let index_buffer = renderer.create_index_buffer(&indices);
    Mesh{
        vertex_buffer,
        index_buffer,
        num_elements: indices.len() as u32,
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
                mesh.unmap();
            }
            if let Some(chunk)=self.chunk_map.get(&pos){
                self.mesh_map.insert(pos,create_mesh(&self.chunk_map,pos,renderer));
            }
        }
    }
}