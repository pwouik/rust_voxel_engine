use crate::mesh::*;
use crate::chunk::*;
use crate::renderer::*;
use cgmath::{Vector3, Point3};
use std::collections::HashMap;
use crate::chunk_loader::{ChunkLoader, RENDER_DIST};
use crate::camera::Camera;

pub struct World{
    pub mesh_map:HashMap<Point3<i32>,Mesh>,
    pub chunk_map:HashMap<Point3<i32>,Box<Chunk>>,
    pub chunk_loader:ChunkLoader
}
impl World{
    pub fn new(renderer:&Renderer)->World{
        let mesh_map:HashMap<Point3<i32>,Mesh>=HashMap::new();
        let chunk_map:HashMap<Point3<i32>,Box<Chunk>>=HashMap::new();
        let mut chunk_loader=ChunkLoader::new();

        World{
            mesh_map,
            chunk_map,
            chunk_loader
        }
    }
    fn add_chunks(&mut self,renderer:&Renderer){
        loop{
            let chunk_result=self.chunk_loader.try_get_chunk();
            match chunk_result{
                Some(chunk) => {
                    self.mesh_map.insert(chunk.0,chunk.1.create_mesh(renderer));
                    self.chunk_map.insert(chunk.0,chunk.1);
                }
                None => {break}
            }
        }
    }
    fn unload_chunks(&mut self, player_pos:Point3<i32>){
        self.chunk_map.retain(|pos, _| {
            let rel_pos= pos-player_pos;
            rel_pos.x > -RENDER_DIST && rel_pos.x <RENDER_DIST && rel_pos.y >= -4 && rel_pos.y <=4 && rel_pos.z > -RENDER_DIST && rel_pos.z <RENDER_DIST
        });
        self.mesh_map.retain(|pos, _| {
            let rel_pos= pos-player_pos;
            rel_pos.x > -RENDER_DIST && rel_pos.x <RENDER_DIST && rel_pos.y >= -4 && rel_pos.y <=4 && rel_pos.z > -RENDER_DIST && rel_pos.z <RENDER_DIST
        })
    }
    pub fn tick(&mut self,renderer:&Renderer,camera:&Camera){
        let player_pos= Point3::new((camera.pos.x/32.0) as i32,(camera.pos.y/32.0) as i32,(camera.pos.z/32.0) as i32);
        self.unload_chunks(player_pos);
        self.chunk_loader.tick(&self.chunk_map,player_pos);
        self.add_chunks(renderer);
    }
}