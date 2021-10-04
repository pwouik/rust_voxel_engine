use crate::block::*;
use cgmath::{Point3, Vector3};
use noise::{SuperSimplex, NoiseFn};

fn p3i32_to_f64(point:Point3<i32>)->Point3<f64>{
    Point3::new(point.x as f64,point.y as f64,point.z as f64)
}
fn p3f64_to_array(point:Point3<f64>)->[f64;3]{
    point.into()
}
pub struct Chunk{
    pub data:[Block;32*32*32],
}
impl Chunk{
    pub fn new(pos:Point3<i32>)->Chunk{
        let mut data:[Block;32*32*32]=unsafe {
            std::mem::MaybeUninit::uninit().assume_init()
        };
        let ssn= SuperSimplex::new();
        for x in 0..32 {
            for z in 0..32{
                for y in 0..32{
                    let world_pos=p3i32_to_f64(pos*32+Vector3::new(x,y,z));
                    let height=world_pos.y as f64-60.0*ssn.get([world_pos.x/400.0,world_pos.z/400.0]);
                    if height < -20.0 {
                        data[(x + (z << 5) + (y << 10)) as usize] = Block { block_type: 1 };
                    }
                    else if height > 20.0 {
                        data[(x + (z << 5) + (y << 10)) as usize] = Block { block_type: 0 };
                    }
                    else if 20.0*ssn.get(p3f64_to_array(world_pos/30.0))<height{
                        data[(x+(z<<5)+(y<<10)) as usize]=Block{ block_type: 0 };
                    }
                    else {
                        data[(x+(z<<5)+(y<<10)) as usize]=Block{ block_type: 1 };
                    }
                }
            }
        }
        Chunk{
            data
        }
    }
    pub fn get_block(&self,x:u32,y:u32,z:u32) ->Block{self.data[ (x+(z<<5)+(y<<10)) as usize] }
    pub fn set_block(&mut self,x:u32,y:u32,z:u32,value:Block){self.data[ (x+(z<<5)+(y<<10)) as usize]=value; }
 }