#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Block{
    pub block_type:u8
}

impl Block{
    pub fn is_full_block(&self)->bool{
        self.block_type>0
    }
}