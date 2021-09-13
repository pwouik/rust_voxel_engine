#[derive(Copy, Clone)]
pub struct Block{
    pub block_type:u16
}

impl Block{
    pub fn is_full_block(&self)->bool{
        self.block_type>0
    }
}