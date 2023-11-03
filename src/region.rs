use std::fs;
use crate::chunk::Chunk;
use bytemuck::Contiguous;
use glam::IVec3;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::convert::TryInto;
fn pos_to_id(pos: IVec3) -> usize {
    return ((pos.x as usize & 15) + 16 * (pos.y as usize & 3) + 16 * 4 * (pos.z as usize & 15)) * 2;
}
pub struct Region {
    index: [u32; 2048],
    file: File,
    pub chunk_count: u32,
}
impl Region {
    pub fn new(save_file: String, pos: IVec3) -> Self {
        if !Path::new(&(save_file.clone() + "/region")).exists() {
            fs::create_dir_all(save_file.clone() + "/region").unwrap();
        }
        let filename = save_file.clone()
            + "/region/"
            + &*pos.x.to_string()
            + ","
            + &*pos.y.to_string()
            + ","
            + &*pos.z.to_string();
        let mut index = [0; 2048];
        let file: File = match OpenOptions::new().read(true).write(true).open(&filename) {
            Ok(mut open_file) => {
                open_file.seek(SeekFrom::Start(0)).unwrap();
                let mut buffer = vec![0u8; 8192];
                open_file.read_exact(&mut buffer).unwrap();
                for i in 0..2048 {
                    index[i] = u32::from_le_bytes(buffer[i*4..i*4+4].try_into().unwrap());
                }
                open_file
            }
            Err(_) => {
                let mut openfile = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&filename)
                    .unwrap();
                for i in (0..2048).step_by(2) {
                    index[i] = 8192;
                    index[i + 1] = 0;
                }
                openfile.write_all(&[0; 8192]).unwrap();
                openfile
            }
        };
        Region {
            index,
            file,
            chunk_count: 0,
        }
    }
    #[profiling::function]
    fn shrink_to_fit(&mut self) {
        let mut pos = 8192;
        let mut old_pos = 0;
        loop {
            let mut min = u32::MAX_VALUE;
            let mut index = 0;
            for j in (0..2048).step_by(2) {
                if self.index[j] > old_pos && self.index[j] < min && self.index[j + 1] != 0 {
                    min = self.index[j];
                    index = j;
                }
            }
            if min == u32::MAX_VALUE {
                break;
            }
            old_pos = min;
            if min != pos {
                self.index[index] = pos;
                let mut buffer = vec![0; self.index[index + 1] as usize];
                self.file.seek(SeekFrom::Start(min as u64)).unwrap();
                self.file.read_exact(&mut buffer).unwrap();
                self.file.seek(SeekFrom::Start(pos as u64)).unwrap();
                self.file.write_all(&mut buffer).unwrap();
            }
            pos += self.index[index + 1];
        }
        self.file.set_len(pos as u64).unwrap();
    }
    #[profiling::function]
    pub fn save_index(&mut self) {
        let mut buffer = vec![0; 2048];
        for i in 0..2048 {
            buffer[i] = self.index[i].to_le();
        }
        self.file.seek(SeekFrom::Start(0)).unwrap();
        self.file.write_all(bytemuck::cast_slice(&buffer)).unwrap();
    }
    #[profiling::function]
    fn available_space(&self, id: usize) -> u32 {
        let mut space = i32::MAX;
        for i in (0..2048).step_by(2) {
            let diff = self.index[i] as i32 - self.index[id] as i32;
            if diff >= 0 && diff < space{
                space = diff;
            }
        }
        return space as u32;
    }
    #[profiling::function]
    pub fn save_chunk(&mut self, mut chunk: Box<Chunk>, pos: IVec3) {
        self.chunk_count -= 1;
        let location = pos_to_id(pos);
        let data= chunk.serialize();
        self.index[location + 1] = data.len() as u32;
        if self.available_space(location)< data.len() as u32 {
            self.index[location] = self.file.seek(SeekFrom::End(0)).unwrap() as u32;
        }
        else{
            self.file.seek(SeekFrom::Start(self.index[location] as u64)).unwrap();
        }
        self.file.write_all(&data).unwrap();
    }
    #[profiling::function]
    pub fn load_chunk(&mut self, pos: IVec3) -> Option<Box<Chunk>> {
        self.chunk_count += 1;
        let location = pos_to_id(pos);
        let data_size = self.index[location + 1] as usize;

        if data_size == 0 {
            return None;
        }
        let mut chunk = Box::new(Chunk::new());
        self.file.seek(SeekFrom::Start(self.index[location] as u64)).unwrap();
        let mut buffer = vec![0u8; data_size];
        self.file.read_exact(&mut buffer).unwrap();
        chunk.deserialize(&buffer);
        Some(chunk)
    }
}
impl Drop for Region {
    fn drop(&mut self) {
        self.shrink_to_fit();
        self.save_index();
    }
}
