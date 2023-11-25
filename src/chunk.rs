use crate::block::*;
use glam::{dvec3, ivec3, uvec3, DVec3, IVec3, UVec3, Vec3, vec3};
use simdnoise::NoiseBuilder;
use std::convert::TryInto;

const BITSIZES: [u8; 13] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16];
#[derive(Clone)]
pub struct Chunk {
    palette: Vec<u16>,
    blocks_per_element: u64,
    bitsizes_index: usize,
    bitsize: u8,
    mask: u64,
    data: Vec<u64>,
}
impl Chunk {
    #[profiling::function]
    pub fn new() -> Self {
        Chunk {
            palette: vec![0],
            blocks_per_element: 64,
            bitsizes_index: 0,
            bitsize: 0,
            mask: 0,
            data: vec![],
        }
    }
    #[profiling::function]
    pub fn deserialize(&mut self, buffer: &[u8]) {
        let palette_len = u16::from_le_bytes(buffer[..2].try_into().unwrap()) as usize;
        self.palette.clear();
        for i in 0..palette_len {
            self.palette.push(u16::from_le_bytes(
                buffer[2 + i * 2..4 + i * 2].try_into().unwrap(),
            ));
        }
        let data_len = (buffer.len() - 2 - palette_len * 2) / 8;
        let start = 2 + palette_len * 2;
        self.data.clear();
        for i in 0..data_len {
            self.data.push(u64::from_le_bytes(
                buffer[start + 8 * i..start + 8 * i + 8].try_into().unwrap(),
            ));
        }
        self.bitsizes_index = 0;
        while self.palette.len() > 1 << BITSIZES[self.bitsizes_index] {
            self.bitsizes_index += 1;
        }
        self.bitsize = BITSIZES[self.bitsizes_index];
        self.mask = (1 << self.bitsize) - 1;
        self.blocks_per_element = if self.bitsize != 0 {
            64 / self.bitsize as u64
        } else {
            0
        };
    }
    #[profiling::function]
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.extend_from_slice(&(self.palette.len() as u16).to_le_bytes());
        for i in &self.palette {
            buffer.extend_from_slice(&i.to_le_bytes());
        }
        for i in &self.data {
            buffer.extend_from_slice(&i.to_le_bytes());
        }
        return buffer;
    }
    #[profiling::function]
    pub fn generate(&mut self, pos: IVec3) {
        let chunk_pos:Vec3 = vec3((pos.x * 32) as f32, (pos.y * 32) as f32, (pos.z*32)as f32);
        let (hills,_,_) = NoiseBuilder::gradient_2d_offset(chunk_pos.x,32,chunk_pos.z,32).with_freq(0.002).generate();
        let (valley,_,_) = NoiseBuilder::gradient_2d_offset(chunk_pos.x,32,chunk_pos.z,32).with_freq(0.007).with_seed(132487).generate();
        let (caves1,_,_) = NoiseBuilder::gradient_3d_offset(chunk_pos.x,32,chunk_pos.y,32,chunk_pos.z,32).with_freq(0.007).generate();
        let (caves2,_,_) = NoiseBuilder::gradient_3d_offset(chunk_pos.x,32,chunk_pos.y,32,chunk_pos.z,32).with_freq(0.01).with_seed(132487).generate();
        for z in 0usize..32 {
            for y in 0usize..32 {
                for x in 0usize..32 {
                    let height = hills[x+(z<<5)]*2000.0 - valley[x+(z<<5)].abs()*1000.0;
                    let depth = height - (pos.y * 32 + y as i32) as f32;
                    let mut block = Block { block_type: 0 };
                    let id = x+(y<<5)+(z<<10);
                    if caves1[id].abs()+caves2[id].abs() > 0.005
                    {
                        if depth > 5.0 {
                            block = Block { block_type: 3 };
                        } else if depth > 1.0 {
                            block = Block { block_type: 2 };
                        } else if depth > 0.0 {
                            block = Block { block_type: 1 };
                        }
                    }
                    self.set_block(uvec3(x as u32, y as u32, z as u32), block);
                }
            }
        }
    }
    pub fn get_block(&self, pos: UVec3) -> Block {
        if self.palette.len() == 1 {
            return Block {
                block_type: self.palette[0],
            };
        }
        let p = (pos.x + (pos.y << 5) + (pos.z << 10)) as u64;
        let b = (self.data[p as usize / self.blocks_per_element as usize]
            >> (p % self.blocks_per_element * self.bitsize as u64))
            & self.mask;
        return Block {
            block_type: self.palette[b as usize],
        };
    }
    pub fn set_block(&mut self, pos: UVec3, block: Block) {
        let value = self.block_to_id(block).unwrap_or_else(|| -> u16 {
            self.palette.push(block.block_type);
            if self.mask < self.palette.len() as u64 - 1 {
                profiling::scope!("Resize");
                self.bitsizes_index += 1;
                let new_bitsize = BITSIZES[self.bitsizes_index];
                let new_blocks_per_element = (64 / new_bitsize) as u64;
                let mut temp_data =
                    vec![0; (32u64 * 32 * 32).div_ceil(new_blocks_per_element) as usize];
                if self.bitsizes_index > 1 {
                    for i in 0..32 * 32 * 32 {
                        temp_data[i / new_blocks_per_element as usize] |= ((self.data
                            [i / self.blocks_per_element as usize]
                            >> (i as u64 % self.blocks_per_element * self.bitsize as u64))
                            & self.mask)
                            << (i as u64 % new_blocks_per_element * new_bitsize as u64);
                    }
                }
                self.bitsize = new_bitsize;
                self.blocks_per_element = new_blocks_per_element;
                self.mask = (1 << self.bitsize) - 1;
                self.data = temp_data;
            }
            self.palette.len() as u16 - 1
        }) as u64;
        if self.palette.len() == 1 {
            return;
        }
        let p = (pos.x + (pos.y << 5) + (pos.z << 10)) as u64;
        let i = p as usize / self.blocks_per_element as usize;
        let offset = p % self.blocks_per_element * self.bitsize as u64;
        self.data[i] = self.data[i] & !(self.mask << offset) | (value << offset);
    }
    fn block_to_id(&self, block: Block) -> Option<u16> {
        for i in 0..self.palette.len() {
            if self.palette[i] == block.block_type {
                return Some(i as u16);
            }
        }
        None
    }
    pub fn data(&self) -> &Vec<u64> {
        &self.data
    }
    pub fn palette(&self) -> &Vec<u16> {
        &self.palette
    }
}
