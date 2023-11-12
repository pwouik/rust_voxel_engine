use crate::block::Block;
use crate::chunk::Chunk;
use ahash::AHashMap;
use glam::{ivec3, uvec3, IVec3};

pub struct ChunkMap {
    pub hash_map: AHashMap<IVec3, Box<Chunk>>,
}
impl ChunkMap {
    pub fn new() -> Self {
        ChunkMap {
            hash_map: AHashMap::new(),
        }
    }
    pub fn get_chunk(&self, pos: IVec3) -> Option<&Box<Chunk>> {
        self.hash_map.get(&pos)
    }
    pub fn get_chunk_mut(&mut self, pos: IVec3) -> Option<&mut Box<Chunk>> {
        self.hash_map.get_mut(&pos)
    }
    pub fn get_block(&self, pos: IVec3) -> Block {
        if let Some(chunk) = self.get_chunk(ivec3(pos.x >> 5, pos.y >> 5, pos.z >> 5)) {
            chunk.get_block(uvec3(
                (pos.x as u32) & 31,
                (pos.y as u32) & 31,
                (pos.z as u32) & 31,
            ))
        } else {
            Block { block_type: 0 }
        }
    }
}
