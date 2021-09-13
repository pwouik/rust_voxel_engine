use crate::mesh::*;
use crate::chunk::*;
use crate::renderer::*;
use cgmath::Vector3;
use std::collections::HashMap;
use crate::chunk_loader::ChunkLoader;

pub struct World{
    pub mesh_map:HashMap<Vector3<i32>,Mesh>,
    pub chunk_map:HashMap<Vector3<i32>,Chunk>,
    chunk_loader:ChunkLoader
}
impl World{
    pub fn new(renderer:&Renderer)->World{
        let mesh_map:HashMap<Vector3<i32>,Mesh>=HashMap::new();
        let chunk_map:HashMap<Vector3<i32>,Chunk>=HashMap::new();
        let chunk_loader=ChunkLoader::new();
        for x in 0..0 {
            for z in 0..1{
                for y in 0..1{
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
        println!("d");
        loop{
            println!("x");
            let chunk_result=self.chunk_loader.try_get_chunk();
            println!("z");
            match chunk_result{
                Ok(chunk) => {
                    self.mesh_map.insert(chunk.0,chunk.1.create_mesh(renderer));
                    self.chunk_map.insert(chunk.0,chunk.1);
                }
                Err(_) => {break}
            }
        }
        println!("e");
    }
    pub fn tick(&mut self,renderer:&Renderer){
        println!("c");
        self.add_chunks(renderer);
    }
}