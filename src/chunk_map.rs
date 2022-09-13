use crate::block::Block;
use crate::chunk::Chunk;
use cgmath::{point3, Point3};
use std::collections::HashMap;

pub struct ChunkMap {
    pub hash_map: HashMap<Point3<i32>, Box<Chunk>>,
}
impl ChunkMap {
    pub fn new() -> Self {
        ChunkMap {
            hash_map: HashMap::new(),
        }
    }
    pub fn get_chunk(&self, pos: Point3<i32>) -> Option<&Box<Chunk>> {
        self.hash_map.get(&pos)
    }
    pub fn get_chunk_mut(&mut self, pos: Point3<i32>) -> Option<&mut Box<Chunk>> {
        self.hash_map.get_mut(&pos)
    }
    pub fn get_block(&self, pos: Point3<i32>) -> Block {
        if let Some(chunk) = self.get_chunk(point3(pos.x >> 5, pos.y >> 5, pos.z >> 5)) {
            chunk.get_block(point3(
                (pos.x as u32) & 31,
                (pos.y as u32) & 31,
                (pos.z as u32) & 31,
            ))
        } else {
            Block { block_type: 0 }
        }
    }
}
