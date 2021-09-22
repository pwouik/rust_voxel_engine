use std::thread;
use std::thread::JoinHandle;
use std::time;
use crossbeam::channel::{bounded,TrySendError};
use crossbeam::channel;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use cgmath::{Vector3, Point3};
use crate::chunk::Chunk;
use std::sync::mpsc;
use std::sync::mpsc::*;
use std::time::Duration;
use std::collections::{HashSet, HashMap};
use crate::camera::Camera;

pub const RENDER_DIST:i32=3;

pub struct ChunkLoader{
    loading_chunks:HashSet<Point3<i32>>,
    sender_pos:crossbeam::channel::Sender<Point3<i32>>,
    receiver_chunk:Receiver<(Point3<i32>,Box<Chunk>)>,
    thread_handles:Vec<JoinHandle<()>>,
    running:Arc<AtomicBool>
}
impl ChunkLoader{
    pub fn new()->Self{
        let (sender_pos,receiver_pos) = bounded(10) as (channel::Sender<Point3<i32>>,channel::Receiver<Point3<i32>>);
        let (sender_chunk,receiver_chunk)=channel() as (mpsc::Sender<(Point3<i32>,Box<Chunk>)>,mpsc::Receiver<(Point3<i32>,Box<Chunk>)>);
        let mut thread_handles=Vec::new();
        let running = Arc::new(AtomicBool::new(true));
        for i in 0..4{
            let loc_receiver_pos = receiver_pos.clone();
            let loc_sender_chunk= sender_chunk.clone();
            let loc_running= running.clone();
            thread_handles.push(thread::spawn(move || {
                while loc_running.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(20));
                    let pos = loc_receiver_pos.try_recv();
                    match pos{
                        Ok(chunk_pos)=>{
                            loc_sender_chunk.send((chunk_pos,Box::new(Chunk::new(chunk_pos)))).unwrap();
                        }
                        Err(_)=>{}
                    }

                }
            }));
        }
        let loading_chunks = HashSet::new();
        ChunkLoader{
            loading_chunks,
            sender_pos,
            receiver_chunk,
            thread_handles,
            running
        }
    }
    pub fn load(&mut self, pos:Point3<i32>){
        let result = self.sender_pos.try_send(pos);
        if result.is_ok(){
            self.loading_chunks.insert(pos);
        }
    }
    pub fn try_get_chunk(&mut self) -> Option<(Point3<i32>, Box<Chunk>)> {
        let result=self.receiver_chunk.try_recv();
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
        for y in (-4..0).rev() {
            self.try_load(player_pos,Vector3::new(0,y,0),chunk_map);
        }
        for y in 0..4 {
            self.try_load(player_pos,Vector3::new(0,y,0),chunk_map);
        }
        for s in 1..RENDER_DIST {
            for x in -s..s {
                for y in (-4..0).rev() {
                    self.try_load(player_pos,Vector3::new(x,y,s),chunk_map);
                    self.try_load(player_pos,Vector3::new(s,y,-x),chunk_map);
                    self.try_load(player_pos,Vector3::new(-x,y,-s),chunk_map);
                    self.try_load(player_pos,Vector3::new(-s,y,x),chunk_map);
                }
                for y in 0..4 {
                    self.try_load(player_pos,Vector3::new(x,y,s),chunk_map);
                    self.try_load(player_pos,Vector3::new(s,y,-x),chunk_map);
                    self.try_load(player_pos,Vector3::new(-x,y,-s),chunk_map);
                    self.try_load(player_pos,Vector3::new(-s,y,x),chunk_map);
                }
            }
        }
    }
}
impl Drop for ChunkLoader {
    fn drop(&mut self) {
        self.running.store(false,Ordering::Relaxed);
        for i in 0..4{
            self.thread_handles.remove(0).join().unwrap();
        }
    }
}