use crate::mesh::*;
use crate::block::*;
use crate::renderer::*;
use cgmath::Vector3;

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
pub struct Chunk{
    pub data:[Block;32*32*32],
}
impl Chunk{
    pub fn new(pos:Vector3<i32>)->Chunk{
        let mut data:[Block;32*32*32]=unsafe {
            std::mem::MaybeUninit::uninit().assume_init()
        };
        for x in 0..32 {
            for z in 0..32{
                for y in 0..32{
                    if y==x^z{
                        data[x+(z<<5)+(y<<10)]=Block{ block_type: 1 };
                    }
                    else{
                        data[x+(z<<5)+(y<<10)]=Block{ block_type: 0 };
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
    pub fn create_mesh(&self,renderer:&Renderer)->Mesh{
        let mut indices = Vec::new();
        let mut vertices = Vec::new();
        let mut index:usize=0;
        for y in 0..32 {
            for z in 0..32 {
                for x in 0..32 {
                    let b1 = self.data[index].is_full_block();
                    if x < 31
                    {
                        let b2 = self.data[index + 1].is_full_block();
                        if b1 != b2 {
                            if b1 == true {
                                add_face(&mut vertices, &mut indices, [x as f32,y as f32,z as f32], 1,TEXTURE_INDEX[0][1]);
                            } else {
                                add_face(&mut vertices, &mut indices, [(x+1) as f32,y as f32,z as f32], 0,TEXTURE_INDEX[0][0]);
                            }
                        }
                    }
                    if z < 31
                    {
                        let b2 = self.data[index + 32].is_full_block();
                        if b1 != b2 {
                            if b1 == true {
                                add_face(&mut vertices, &mut indices, [x as f32,y as f32,z as f32], 5,TEXTURE_INDEX[0][5]);
                            } else {
                                add_face(&mut vertices, &mut indices, [x as f32,y as f32,(z+1) as f32], 4,TEXTURE_INDEX[0][4]);
                            }
                        }
                    }
                    if y < 31
                    {
                        let b2 = self.data[index + 32 * 32].is_full_block();
                        if b1 != b2 {
                            if b1 == true {
                                add_face(&mut vertices, &mut indices, [x as f32,y as f32,z as f32], 3,TEXTURE_INDEX[0][3]);
                            } else {
                                add_face(&mut vertices, &mut indices, [x as f32,(y+1) as f32,z as f32], 2,TEXTURE_INDEX[0][2]);
                            }
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
 }