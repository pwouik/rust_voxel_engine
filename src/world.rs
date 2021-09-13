use crate::mesh::*;
use crate::chunk::*;
use crate::renderer::*;
use cgmath::Vector3;
use std::collections::HashMap;
use crate::chunk_loader::ChunkLoader;

pub struct World{
    pub mesh_map:HashMap<Vector3<i32>,Mesh>,
    pub chunk_map:HashMap<Vector3<i32>,Box<Chunk>>,
    chunk_loader:ChunkLoader
}
impl World{
    pub fn new(renderer:&Renderer)->World{
        let mesh_map:HashMap<Vector3<i32>,Mesh>=HashMap::new();
        let chunk_map:HashMap<Vector3<i32>,Box<Chunk>>=HashMap::new();
        let chunk_loader=ChunkLoader::new();
        for x in 0..2 {
            for z in 0..2{
                for y in 0..2{
                    chunk_loader.load(Vector3{x,y,z});
                }
            }
        }

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
                Ok(chunk) => {
                    self.mesh_map.insert(chunk.0,chunk.1.create_mesh(renderer));
                    self.chunk_map.insert(chunk.0,chunk.1);
                }
                Err(_) => {break}
            }
        }
    }
    pub fn tick(&mut self,renderer:&Renderer){
        self.add_chunks(renderer);
    }
}