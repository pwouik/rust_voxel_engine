use crate::block::*;
use cgmath::{Point3, point3, vec3, Vector3};
use noise::{SuperSimplex, NoiseFn};

fn p3i32_to_f64(point:Point3<i32>)->Point3<f64>{
    point3(point.x as f64,point.y as f64,point.z as f64)
}
fn p3f64_to_array(point:Point3<f64>)->[f64;3]{
    point.into()
}
pub struct Chunk{
    pub data:[Block;32*32*32],
}
impl Chunk{
    #[profiling::function]
    pub fn new()->Self {
        let mut data: [Block; 32 * 32 * 32] = [Block { block_type: 0 }; 32 * 32 * 32];
        Chunk{
            data
        }
    }
    #[profiling::function]
    pub fn generate(&mut self,pos:Point3<i32>){
        let ssn= SuperSimplex::new();
        for x in 0i32..32 {
            for z in 0i32..32{
                for y in 0i32..32{
                    let world_pos=p3i32_to_f64(pos*32+vec3(x,y,z));
                    let height=world_pos.y as f64-(60.0*ssn.get([world_pos.x/400.0,world_pos.z/400.0])+30.0*(1.0-ssn.get([world_pos.x/80.0,world_pos.z/80.0]).abs()));
                    let mut block =Block{block_type:0};
                    if height < -5.0 {
                        block=Block{block_type:3};
                    }
                    else if height < -1.0 {
                        block=Block{block_type:2};
                    }
                    else if height < 0.0 {
                        block=Block{block_type:1};
                    }/*
                    if ssn.get(p3f64_to_array(world_pos/90.0)).abs()+ssn.get(p3f64_to_array(world_pos/90.0 + vec3(0.0,7.0,0.0))).abs()<0.1 {
                        block=Block{block_type:0};
                    }*/
                    self.data[(x + (z << 5) + (y << 10)) as usize] = block;
                    //(ssn.get(p3f64_to_array(world_pos/90.0)).abs()>0.1 || ssn.get(p3f64_to_array(world_pos/90.0 + vec3(0.0,7.0,0.0))).abs()>0.1)
                }
            }
        }
    }
    pub fn get_block(&self,pos:Point3<u32>) ->Block{self.data[ (pos.x+(pos.z<<5)+(pos.y<<10)) as usize]}
    pub fn set_block(&mut self,pos:Point3<u32>,value:Block){self.data[ (pos.x+(pos.z<<5)+(pos.y<<10)) as usize]=value; }
 }