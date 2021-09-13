use std::thread;
use std::thread::JoinHandle;
use std::time;
use crossbeam::channel::bounded;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use cgmath::Vector3;
use crate::chunk::Chunk;
use std::sync::mpsc::*;
use std::time::Duration;

pub struct ChunkLoader{
    sender_pos:crossbeam::channel::Sender<Vector3<i32>>,
    receiver_chunk:Receiver<(Vector3<i32>,Box<Chunk>)>,
    thread_handles:Vec<JoinHandle<()>>,
    running:Arc<AtomicBool>
}
impl ChunkLoader{
    pub fn new()->Self{
        let (sender_pos,receiver_pos) = bounded(10);
        let (sender_chunk,receiver_chunk)=channel();
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
        ChunkLoader{
            sender_pos,
            receiver_chunk,
            thread_handles,
            running
        }
    }
    pub fn load(&self,pos:Vector3<i32>){
        self.sender_pos.send(pos).unwrap();
    }
    pub fn try_get_chunk(&self) -> Result<(Vector3<i32>, Box<Chunk>), TryRecvError> {
        self.receiver_chunk.try_recv()
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