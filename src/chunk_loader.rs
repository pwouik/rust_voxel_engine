use cgmath::{Vector3, Point3};
use std::collections::{HashSet, HashMap};
use crate::chunk::Chunk;
use crate::util::threadpool::ThreadPool;

pub const RENDER_DIST:i32=10;
pub const RENDER_DIST_HEIGHT:i32=4;

pub struct ChunkLoader{
    loading_chunks:HashSet<Point3<i32>>,
    threadpool:ThreadPool<Point3<i32>,(Point3<i32>,Box<Chunk>)>,
}
impl ChunkLoader{
    pub fn new()->Self{

        let threadpool= ThreadPool::new(|pos:Point3<i32>|(pos,Box::new(Chunk::new(pos))));
        let loading_chunks = HashSet::new();
        ChunkLoader{
            loading_chunks,
            threadpool
        }
    }
    pub fn load(&mut self, pos:Point3<i32>){
        let result = self.threadpool.send(pos);
        if result.is_ok(){
            self.loading_chunks.insert(pos);
        }
    }
    pub fn try_get_chunk(&mut self) -> Option<(Point3<i32>, Box<Chunk>)> {
        let result=self.threadpool.get();
        match result
        {
            Ok(chunk)=>{
                self.loading_chunks.remove(&chunk.0);
                Some(chunk)
            }
            Err(_)=>{None}
        }
    }
    pub fn try_load(&mut self,player_pos:Point3<i32>, pos:Vector3<i32>, chunk_map:&HashMap<Point3<i32>,Box<Chunk>>){
        let chunk_pos=player_pos+pos;
        if !self.loading_chunks.contains(&chunk_pos) && chunk_map.get(&chunk_pos).is_none(){
            self.load(chunk_pos);
        }
    }
    pub fn tick(&mut self, chunk_map:&HashMap<Point3<i32>,Box<Chunk>>, player_pos:Point3<i32>){
        for y in (-RENDER_DIST_HEIGHT..0).rev() {
            self.try_load(player_pos,Vector3::new(0,y,0),chunk_map);
        }
        for y in 0..RENDER_DIST_HEIGHT {
            self.try_load(player_pos,Vector3::new(0,y,0),chunk_map);
        }
        for s in 1..RENDER_DIST {
            for x in -s..s {
                for y in (-RENDER_DIST_HEIGHT..0).rev() {
                    self.try_load(player_pos,Vector3::new(x,y,s),chunk_map);
                    self.try_load(player_pos,Vector3::new(s,y,-x),chunk_map);
                    self.try_load(player_pos,Vector3::new(-x,y,-s),chunk_map);
                    self.try_load(player_pos,Vector3::new(-s,y,x),chunk_map);
                }
                for y in 0..RENDER_DIST_HEIGHT
                {
                    self.try_load(player_pos,Vector3::new(x,y,s),chunk_map);
                    self.try_load(player_pos,Vector3::new(s,y,-x),chunk_map);
                    self.try_load(player_pos,Vector3::new(-x,y,-s),chunk_map);
                    self.try_load(player_pos,Vector3::new(-s,y,x),chunk_map);
                }
            }
        }
    }
}