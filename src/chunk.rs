use crate::block::*;
use glam::{ivec3, DVec3, IVec3, UVec3,dvec3};
use noise::{NoiseFn, Simplex};

pub struct Chunk {
    pub data: [Block; 32 * 32 * 32],
}
impl Chunk {
    #[profiling::function]
    pub fn new() -> Self {
        let data: [Block; 32 * 32 * 32] = [Block { block_type: 0 }; 32 * 32 * 32];
        Chunk { data }
    }
    #[profiling::function]
    pub fn generate(&mut self, pos: IVec3) {
        let ssn = Simplex::new(0);
        for x in 0i32..32 {
            for z in 0i32..32 {
                for y in 0i32..32 {
                    let world_pos: DVec3 = (pos * 32 + ivec3(x, y, z)).into();
                    let height = 60.0 * ssn.get([world_pos.x / 400.0, world_pos.z / 400.0])
                            + 30.0 * (1.0 - ssn.get([world_pos.x / 80.0, world_pos.z / 80.0]).abs());
                    let depth = height - world_pos.y;
                    let mut block = Block { block_type: 0 };
                    if ssn.get((world_pos/90.0).into()).abs()+ssn.get((world_pos/90.0 + dvec3(0.0,7.0,0.0)).into()).abs() > 0.1 {
                        if depth > 5.0 {
                            block = Block { block_type: 3 };
                        } else if depth > 1.0 {
                            block = Block { block_type: 2 };
                        } else if depth > 0.0 {
                            block = Block { block_type: 1 };
                        }
                    }
                    self.data[(x + (z << 5) + (y << 10)) as usize] = block;
                }
            }
        }
    }
    pub fn get_block(&self, pos: UVec3) -> Block {
        self.data[(pos.x + (pos.z << 5) + (pos.y << 10)) as usize]
    }
    pub fn set_block(&mut self, pos: UVec3, value: Block) {
        self.data[(pos.x + (pos.z << 5) + (pos.y << 10)) as usize] = value;
    }
}
