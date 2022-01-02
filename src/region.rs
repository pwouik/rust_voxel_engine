use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use bytemuck::Contiguous;
use cgmath::Point3;
use crate::block::Block;
use crate::chunk::Chunk;
use lz4_flex::{compress, decompress};

fn pos_to_id(pos:Point3<i32>)->usize
{
    return (((pos.x as u32&15)+16*(pos.y as u32&3)+16*4*(pos.z as u32&15))*2) as usize;
}
pub struct Region
{
    index:[u32;2048],
    file:File,
    pub chunk_count:u32,
}
impl Region {
    pub fn new(save_file:String,pos:Point3<i32>)->Self
    {
        let filename = save_file.clone()+ "/region/" + &*pos.x.to_string() + "," + &*pos.y.to_string() + "," + &*pos.z.to_string();
        let mut index=[0;2048];
        let file:File = match OpenOptions::new().read(true).write(true).open(&filename) {
            Ok(mut open_file)=> {
                for i in 0..2048{
                    open_file.seek(SeekFrom::Start((i*4) as u64)).unwrap();
                    let mut array =[0;4];
                    open_file.read_exact(&mut array).unwrap();
                    index[i]=u32::from_le_bytes(array);
                }
                open_file
            }
            Err(_)=> {
                let mut openfile = OpenOptions::new().read(true).write(true).create(true).open(&filename).unwrap();
                for i in (0..2048).step_by(2){
                    index[i]=8192;
                    index[i+1]=0;
                }
                openfile.write_all(&[0;8192]).unwrap();
                openfile
            }
        };
        Region{
            index,
            file,
            chunk_count: 0
        }
    }
    #[profiling::function]
    fn shrink_to_fit(&mut self){
        let mut pos= 8192;
        let mut old_pos=0;
        loop{
            let mut min =u32::MAX_VALUE;
            let mut index = 0;
            for j in (0..2048).step_by(2){
                if self.index[j]>old_pos && self.index[j]<min && self.index[j+1]!= 0{
                    min = self.index[j];
                    index = j;
                }
            }
            if min == u32::MAX_VALUE{break;}
            old_pos = min;
            if min!=pos {
                self.index[index] = pos;
                let mut buffer = vec![0; 2 + 2 * self.index[index + 1] as usize];
                self.file.seek(SeekFrom::Start(min as u64)).unwrap();
                self.file.read_exact(&mut buffer).unwrap();
                self.file.seek(SeekFrom::Start(pos as u64)).unwrap();
                self.file.write_all(&mut buffer).unwrap();
            }
            pos+=2+2*self.index[index+1];
        }
        self.file.set_len(pos as u64).unwrap();
    }
    #[profiling::function]
    pub fn save_index(&mut self){
        let mut buffer=vec![0;2048];
        for i in 0..2048{
            buffer[i]=self.index[i].to_le();
        }
        self.file.seek(SeekFrom::Start(0)).unwrap();
        self.file.write_all(bytemuck::cast_slice(&buffer)).unwrap();
    }
    #[profiling::function]
    fn available_space(&self,id:usize)->i32{
        let mut space=i32::MAX;
        for i in (0..2048).step_by(2)
        {
            let diff=self.index[i] as i32-self.index[id] as i32;
            if diff>=0 && diff<space {
                space = diff;
            }
        }
        return space;
    }
    #[profiling::function]
    fn compress_chunk(chunk:&mut Box<Chunk>) ->Vec<u16>
    {

        let mut compressed_data =vec![];
        let mut i =0;
        while i < 32*32*32
        {
            let block=chunk.data[i].block_type;
            let mut l =0;
            while i < 32*32*32 && block == chunk.data[i].block_type
            {
                i+=1;
                l+=1;
            }
            compressed_data.push(block);
            compressed_data.push(l);
        }
        compressed_data
    }
    #[profiling::function]
    pub fn save_chunk(&mut self, mut chunk: Box<Chunk>, pos:Point3<i32>)
    {
        self.chunk_count-=1;
        let location=pos_to_id(pos);
        let compressed_chunk = Region::compress_chunk(&mut chunk);
        let compression=compressed_chunk.len() < 32*32*32;
        let data_size=(compressed_chunk.len().min(32*32*32)+1) as u32;

        self.index[location+1]=data_size;
        let move_chunk=(self.available_space(location) as u32)<data_size;
        if move_chunk
        {
            self.index[location]=self.file.seek(SeekFrom::End(0)).unwrap() as u32;
        }
        self.file.seek(SeekFrom::Start(self.index[location] as u64)).unwrap();
        if compression
        {
            let mut buffer=vec![0;data_size as usize+1];
            buffer[0]=u16::to_le(1);
            for i in 0..data_size as usize - 1{
                buffer[i+1]=compressed_chunk[i].to_le();
            }
            self.file.write_all(bytemuck::cast_slice(&buffer)).unwrap();
        }
        else
        {
            let mut buffer=vec![0;32*32*32+1];
            for i in 0..32*32*32 as usize
            {
                buffer[i+1]=chunk.data[i].block_type.to_le();
            }
            self.file.write_all(bytemuck::cast_slice(&buffer)).unwrap();
        }
    }
    #[profiling::function]
    pub fn load_chunk(&mut self,pos:Point3<i32>)->Option<Box<Chunk>>
    {
        self.chunk_count+=1;
        let location=pos_to_id(pos);
        let data_size=self.index[location+1] as usize;

        if data_size==0 {
            return None;
        }
        let mut chunk=Box::new(Chunk::new());
        self.file.seek(SeekFrom::Start(self.index[location] as u64)).unwrap();
        let mut buffer=vec![0;data_size*2];
        self.file.read_exact(&mut buffer).unwrap();
        if u16::from_le_bytes([buffer[0],buffer[1]])==1
        {
            let mut it =0;
            for i in (1..data_size).step_by(2)
            {
                let block = Block{block_type:u16::from_le_bytes([buffer[i*2],buffer[i*2+1]])};
                let length=u16::from_le_bytes([buffer[(i+1)*2],buffer[(i+1)*2+1]]);
                for _ in 0..length
                {
                    chunk.data[it]=block;
                    it+=1;
                }
            }
        }
        else
        {
            for i in 0..32*32*32
            {
                chunk.data[i]=Block { block_type:u16::from_le_bytes([buffer[i*2+2],buffer[i*2+3]])};
            }
        }
        Some(chunk)
    }
}
impl Drop for Region{
    fn drop(&mut self) {
        self.shrink_to_fit();
        self.save_index();
    }
}