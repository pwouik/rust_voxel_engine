use crate::chunk::Chunk;
use crate::chunk_map::ChunkMap;
use crate::region::Region;
use crate::util::threadpool::ThreadPool;
use glam::{ivec3, IVec3};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub const RENDER_DIST: i32 = 6;
pub const RENDER_DIST_HEIGHT: i32 = 4;
pub const RENDER_DIST2: i32 = RENDER_DIST * 2 + 1;
pub const RENDER_DIST_HEIGHT2: i32 = RENDER_DIST_HEIGHT * 2 + 1;

pub struct ChunkLoader {
    loading_chunks: HashSet<IVec3>,
    storage_thread_handle: Option<JoinHandle<()>>,
    load_sender: crossbeam_channel::Sender<IVec3>,
    save_sender: mpsc::Sender<(IVec3, Box<Chunk>)>,
    threadpool_receiver: mpsc::Receiver<(IVec3, Box<Chunk>)>,
    running: Arc<AtomicBool>,
}
impl ChunkLoader {
    pub fn new() -> Self {
        let (threadpool_receiver, mut threadpool) = ThreadPool::new(|pos: IVec3| {
            let mut chunk = Box::new(Chunk::new());
            chunk.generate(pos);
            (pos, chunk)
        });
        let loading_chunks = HashSet::new();
        let (load_sender, load_receiver) = crossbeam_channel::bounded(60)
            as (
                crossbeam_channel::Sender<IVec3>,
                crossbeam_channel::Receiver<IVec3>,
            );
        let (save_sender, save_receiver) = mpsc::channel()
            as (
                mpsc::Sender<(IVec3, Box<Chunk>)>,
                mpsc::Receiver<(IVec3, Box<Chunk>)>,
            );
        let running = Arc::new(AtomicBool::new(true));
        let loc_running = running.clone();
        let mut region_map: HashMap<IVec3, Box<Region>> = HashMap::new();
        let storage_thread_handle = Some(thread::spawn(move || {
            loop {
                match save_receiver.try_recv() {
                    Ok(chunk) => {
                        let region_pos: IVec3 =
                            ivec3(chunk.0.x >> 4, chunk.0.y >> 2, chunk.0.z >> 4);
                        let region = region_map.get_mut(&region_pos).unwrap(); //region should be created when the chunk is generated
                        region.save_chunk(chunk.1, chunk.0);
                        if region.chunk_count == 0 {
                            region_map.remove(&region_pos);
                        }
                    }
                    _ => {
                        if !loc_running.load(Ordering::Relaxed) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(20));
                        loop {
                            match load_receiver.try_recv() {
                                Ok(pos) => {
                                    let region_pos: IVec3 =
                                        ivec3(pos.x >> 4, pos.y >> 2, pos.z >> 4);
                                    let region_option = region_map.get_mut(&region_pos);
                                    let region = match region_option {
                                        Some(region) => region,
                                        None => {
                                            region_map.insert(
                                                region_pos,
                                                Box::new(Region::new(
                                                    String::from("save"),
                                                    region_pos,
                                                )),
                                            );
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
                                _ => {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }));
        ChunkLoader {
            loading_chunks,
            storage_thread_handle,
            load_sender,
            save_sender,
            threadpool_receiver,
            running,
        }
    }
    pub fn try_load(&mut self, player_pos: IVec3, pos: IVec3, chunk_map: &ChunkMap) {
        let chunk_pos = player_pos + pos;
        if self.loading_chunks.len() < 60
            && !self.loading_chunks.contains(&chunk_pos)
            && chunk_map.get_chunk(chunk_pos).is_none()
        {
            if self.load_sender.send(chunk_pos).is_ok() {
                self.loading_chunks.insert(chunk_pos);
            }
        }
    }
    pub fn save(&mut self, chunk: (IVec3, Box<Chunk>)) {
        self.save_sender.send(chunk).unwrap();
    }
    pub fn try_get_chunk(&mut self) -> Option<(IVec3, Box<Chunk>)> {
        let result = self.threadpool_receiver.try_recv();
        match result {
            Ok(chunk) => {
                self.loading_chunks.remove(&chunk.0);
                Some(chunk)
            }
            Err(_) => None,
        }
    }
    #[profiling::function]
    pub fn tick(&mut self, chunk_map: &ChunkMap, player_pos: IVec3) {
        for y in (-RENDER_DIST_HEIGHT..0).rev() {
            self.try_load(player_pos, ivec3(0, y, 0), chunk_map);
        }
        for y in 0..RENDER_DIST_HEIGHT + 1 {
            self.try_load(player_pos, ivec3(0, y, 0), chunk_map);
        }
        for s in 1..RENDER_DIST + 1 {
            for x in -s..s {
                for y in (-RENDER_DIST_HEIGHT..0).rev() {
                    self.try_load(player_pos, ivec3(x, y, s), chunk_map);
                    self.try_load(player_pos, ivec3(s, y, -x), chunk_map);
                    self.try_load(player_pos, ivec3(-x, y, -s), chunk_map);
                    self.try_load(player_pos, ivec3(-s, y, x), chunk_map);
                }
                for y in 0..RENDER_DIST_HEIGHT + 1 {
                    self.try_load(player_pos, ivec3(x, y, s), chunk_map);
                    self.try_load(player_pos, ivec3(s, y, -x), chunk_map);
                    self.try_load(player_pos, ivec3(-x, y, -s), chunk_map);
                    self.try_load(player_pos, ivec3(-s, y, x), chunk_map);
                }
            }
        }
    }
}
impl Drop for ChunkLoader {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.storage_thread_handle.take().unwrap().join().unwrap();
        println!("storage joined");
    }
}
