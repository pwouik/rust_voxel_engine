use cgmath::{Vector3, Point3, vec3, point3};
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;
use crate::chunk::Chunk;
use crate::chunk_map::ChunkMap;
use crate::region::Region;
use crate::util::threadpool::ThreadPool;

pub const RENDER_DIST:i32=50;
pub const RENDER_DIST_HEIGHT:i32=4;

pub struct ChunkLoader{
    loading_chunks:HashSet<Point3<i32>>,
    storage_thread_handle:Option<JoinHandle<()>>,
    load_sender:crossbeam_channel::Sender<Point3<i32>>,
    save_sender:mpsc::Sender<(Point3<i32>,Box<Chunk>)>,
    threadpool_receiver:mpsc::Receiver<(Point3<i32>,Box<Chunk>)>,
    running:Arc<AtomicBool>
}
impl ChunkLoader{
    pub fn new()->Self{

        let (threadpool_receiver,mut threadpool)  = ThreadPool::new(|pos:Point3<i32>|{
            let mut chunk =Box::new(Chunk::new());
            chunk.generate(pos);
            (pos, chunk)
        });
        let loading_chunks = HashSet::new();
        let (load_sender,load_receiver) = crossbeam_channel::bounded(60) as (crossbeam_channel::Sender<Point3<i32>>,crossbeam_channel::Receiver<Point3<i32>>);
        let (save_sender,save_receiver) = mpsc::channel() as (mpsc::Sender<(Point3<i32>,Box<Chunk>)>,mpsc::Receiver<(Point3<i32>,Box<Chunk>)>);
        let running = Arc::new(AtomicBool::new(true));
        let loc_running = running.clone();
        let mut region_map:HashMap<Point3<i32>,Box<Region>> = HashMap::new();
        let storage_thread_handle=Some(thread::spawn(move || {
            loop {
                match save_receiver.try_recv(){
                    Ok(chunk)=>{
                        let region_pos :Point3<i32>= point3(chunk.0.x>>4,chunk.0.y>>2,chunk.0.z>>4);
                        let region = region_map.get_mut(&region_pos).unwrap();//region should be created when the chunk is generated
                        region.save_chunk(chunk.1,chunk.0);
                        if region.chunk_count==0{
                            region_map.remove(&region_pos);
                        }
                    }
                    _=>{
                        if !loc_running.load(Ordering::Relaxed){
                            break;
                        }
                        thread::sleep(Duration::from_millis(20));
                        loop {
                            match load_receiver.try_recv() {
                                Ok(pos) => {
                                    let region_pos: Point3<i32> = point3(pos.x >> 4, pos.y >> 2, pos.z >> 4);
                                    let region_option = region_map.get_mut(&region_pos);
                                    let region = match region_option {
                                        Some(region)=>{region}
                                        None=>{
                                            region_map.insert(region_pos, Box::new(Region::new(String::from("save"),region_pos)));
                                            region_map.get_mut(&region_pos).unwrap()
                                        }
                                    };
                                    match region.load_chunk(pos) {
                                        Some(chunk) => {
                                            threadpool.pass((pos, chunk));
                                        }
                                        None => {
                                            threadpool.send(pos).unwrap();
                                        }
                                    }
                                }
                                _ => {break;}
                            }
                        }
                    }
                }
            }
        }));
        ChunkLoader{
            loading_chunks,
            storage_thread_handle,
            load_sender,
            save_sender,
            threadpool_receiver,
            running
        }
    }
    pub fn try_load(&mut self,player_pos:Point3<i32>, pos:Vector3<i32>, chunk_map:&ChunkMap){
        let chunk_pos=player_pos+pos;
        if self.loading_chunks.len()<60 && !self.loading_chunks.contains(&chunk_pos) && chunk_map.get_chunk(chunk_pos).is_none(){
            if self.load_sender.send(chunk_pos).is_ok(){
                self.loading_chunks.insert(chunk_pos);
            }
        }
    }
    pub fn save(&mut self,chunk:(Point3<i32>,Box<Chunk>)){
        self.save_sender.send(chunk).unwrap();
    }
    pub fn try_get_chunk(&mut self) -> Option<(Point3<i32>, Box<Chunk>)> {
        let result=self.threadpool_receiver.try_recv();
        match result
        {
            Ok(chunk)=>{
                self.loading_chunks.remove(&chunk.0);
                Some(chunk)
            }
            Err(_)=>{None}
        }
    }
    #[profiling::function]
    pub fn tick(&mut self, chunk_map:&ChunkMap, player_pos:Point3<i32>){
        for y in (-RENDER_DIST_HEIGHT..0).rev() {
            self.try_load(player_pos,vec3(0,y,0),chunk_map);
        }
        for y in 0..RENDER_DIST_HEIGHT {
            self.try_load(player_pos,vec3(0,y,0),chunk_map);
        }
        for s in 1..RENDER_DIST {
            for x in -s..s {
                for y in (-RENDER_DIST_HEIGHT..0).rev() {
                    self.try_load(player_pos,vec3(x,y,s),chunk_map);
                    self.try_load(player_pos,vec3(s,y,-x),chunk_map);
                    self.try_load(player_pos,vec3(-x,y,-s),chunk_map);
                    self.try_load(player_pos,vec3(-s,y,x),chunk_map);
                }
                for y in 0..RENDER_DIST_HEIGHT
                {
                    self.try_load(player_pos,vec3(x,y,s),chunk_map);
                    self.try_load(player_pos,vec3(s,y,-x),chunk_map);
                    self.try_load(player_pos,vec3(-x,y,-s),chunk_map);
                    self.try_load(player_pos,vec3(-s,y,x),chunk_map);
                }
            }
        }
    }
}
impl Drop for ChunkLoader {
    fn drop(&mut self) {
        self.running.store(false,Ordering::Relaxed);
        self.storage_thread_handle.take().unwrap().join().unwrap();
        println!("storage joined");
    }
}