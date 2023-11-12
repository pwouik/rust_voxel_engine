use std::sync::mpsc;
use crate::block::Block;
use crate::camera::Camera;
use crate::chunk_loader::*;
use crate::chunk_map::ChunkMap;
use crate::mesh::*;
use crate::renderer::*;
use crate::util::direction::*;
use crate::chunk::Chunk;
use crate::util::threadpool::ThreadPool;
use ahash::AHashSet;
use glam::{ivec3, uvec3, IVec3, UVec3, Vec3};

const TEXTURE_INDEX: [[u32; 6]; 4] = [
    [0, 0, 0, 0, 2, 1],
    [2, 2, 2, 2, 2, 2],
    [3, 3, 3, 3, 3, 3],
    [4, 4, 4, 4, 4, 4],
];
const FACES_LIGHT: [f32; 6] = [0.4, 0.4, 0.7, 0.7, 0.1, 1.0];

pub struct World {
    pub chunk_map: ChunkMap,
    chunk_updates: AHashSet<IVec3>,
    chunk_loader: ChunkLoader,
    threadpool:ThreadPool<(IVec3,[Box<Chunk>;27]),(IVec3, [Vec<Face>; 6])>,
    threadpool_receiver: mpsc::Receiver<(IVec3, [Vec<Face>; 6])>,
}

impl World {
    pub fn new() -> World {
        let (threadpool_receiver,threadpool) = ThreadPool::new(|chunks:(IVec3,[Box<Chunk>;27])|{
            return (chunks.0,World::create_mesh(chunks.1));
        });
        World {
            chunk_map: ChunkMap::new(),
            chunk_loader: ChunkLoader::new(),
            chunk_updates: AHashSet::new(),
            threadpool,
            threadpool_receiver,
        }
    }
    #[profiling::function]
    fn add_chunks(&mut self) {
        loop {
            let chunk_result = self.chunk_loader.try_get_chunk();
            match chunk_result {
                Some(chunk) => {
                    self.chunk_map.hash_map.insert(chunk.0, chunk.1);
                    let mut surrounded = [[[true; 3]; 3]; 3];
                    for x in -2..3{
                        for y in -2..3{
                            for z in -2..3{
                                if self.chunk_map.get_chunk(chunk.0+ivec3(x,y,z)).is_none(){
                                    for x2 in 0.max(x+1)..3.min(x+3){
                                        for y2 in 0.max(y+1)..3.min(y+3){
                                            for z2 in 0.max(z+1)..3.min(z+3){
                                                surrounded[x2 as usize][y2 as usize][z2 as usize]=false;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    for x in 0..3 {
                        for y in 0..3 {
                            for z in 0..3 {
                                if surrounded[x][y][z]{
                                    self.chunk_updates.insert(chunk.0+ivec3(x as i32-1,y as i32-1,z as i32-1));
                                }
                            }
                        }
                    }
                }
                None => break,
            }
        }
    }
    #[profiling::function]
    fn unload_chunks(&mut self, player_pos: IVec3, renderer: &mut Renderer) {
        let unloaded = self.chunk_map.hash_map.extract_if(|pos, _| {
            let rel_pos = *pos - player_pos;
            rel_pos.x < -RENDER_DIST
                || rel_pos.x > RENDER_DIST
                || rel_pos.y < -RENDER_DIST_HEIGHT
                || rel_pos.y > RENDER_DIST_HEIGHT
                || rel_pos.z < -RENDER_DIST
                || rel_pos.z > RENDER_DIST
        });
        for i in unloaded {
            renderer
                .chunk_renderer
                .remove_chunk(i.0, &mut renderer.queue);
            self.chunk_loader.save(i);
        }
    }
    #[profiling::function]
    pub fn tick(&mut self, camera: &Camera, renderer: &mut Renderer) {
        let player_pos = ivec3(
            camera.pos.x.floor() as i32 >> 5,
            camera.pos.y.floor() as i32 >> 5,
            camera.pos.z.floor() as i32 >> 5,
        );
        self.unload_chunks(player_pos, renderer);
        self.chunk_loader.tick(&self.chunk_map, player_pos);
        self.add_chunks();
    }
    #[profiling::function]
    pub fn update_display(&mut self, renderer: &mut Renderer) {
        for pos in self.chunk_updates.drain(){
            let mut chunks:[Box<Chunk>;27] = std::array::from_fn(|_| Box::new(Chunk::new()));
            for z in -1..2{
                for y in -1..2{
                    for x in -1..2 {
                        chunks[(x+1 + 3*(y+1) + 9*(z+1)) as usize] = self.chunk_map.get_chunk(pos+ivec3(x,y,z)).unwrap_or(&Box::new(Chunk::new())).clone();
                    }
                }
            }
            self.threadpool.send((pos,chunks)).unwrap();
        }
        loop {
            if let Ok(mut mesh)=self.threadpool_receiver.try_recv() {
                if self.chunk_map.get_chunk(mesh.0).is_some() {
                    renderer
                        .chunk_renderer
                        .remove_chunk(mesh.0, &mut renderer.queue);
                    renderer.chunk_renderer.add_chunk(
                        mesh.0,
                        &mut mesh.1,
                        &renderer.queue,
                        &renderer.device,
                    );
                }
            }
            else {
                break
            }
        }
    }
    #[profiling::function]
    fn add_face(
        chunks:&[Box<Chunk>;27],
        storage: &mut [Vec<Face>; 6],
        pos: UVec3,
        dir: Direction,
        texture: u32,
    ) {
        let ao_blocks = [
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(1, 1, 0)))
                .is_full_block(),
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(1, 1, 1)))
                .is_full_block(),
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(0, 1, 1)))
                .is_full_block(),
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(-1, 1, 1)))
                .is_full_block(),
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(-1, 1, 0)))
                .is_full_block(),
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(-1, 1, -1)))
                .is_full_block(),
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(0, 1, -1)))
                .is_full_block(),
            World::get_local_block(&chunks, pos.as_ivec3() + dir.transform(ivec3(1, 1, -1)))
                .is_full_block(),
        ];
        let light: [u8; 4] = [
            ((255 - (ao_blocks[4] || ao_blocks[5] || ao_blocks[6]) as u32 * 200) as f32
                * FACES_LIGHT[dir.id as usize]) as u8,
            ((255 - (ao_blocks[2] || ao_blocks[3] || ao_blocks[4]) as u32 * 200) as f32
                * FACES_LIGHT[dir.id as usize]) as u8,
            ((255 - (ao_blocks[0] || ao_blocks[1] || ao_blocks[2]) as u32 * 200) as f32
                * FACES_LIGHT[dir.id as usize]) as u8,
            ((255 - (ao_blocks[6] || ao_blocks[7] || ao_blocks[0]) as u32 * 200) as f32
                * FACES_LIGHT[dir.id as usize]) as u8,
        ];
        storage[dir.id as usize].push(Face {
            pos_dir_tex: (pos.x & 63)
                | ((pos.y & 63) << 6)
                | ((pos.z & 63) << 12)
                | ((dir.id as u32 & 7) << 18)
                | ((texture & 2047) << 21),
            light,
        })
    }
    fn get_local_block(chunks: &[Box<Chunk>;27],pos:IVec3)->Block{
        let chunk_pos:IVec3 = (pos>>5) + ivec3(1,1,1);
        return chunks[(chunk_pos.x + 3*chunk_pos.y + 9*chunk_pos.z) as usize].get_block((pos&31).as_uvec3())
    }
    #[profiling::function]
    pub fn create_mesh(chunks: [Box<Chunk>;27]) -> [Vec<Face>; 6] {
        let mut storage: [Vec<Face>; 6] = Default::default();
        for y in 0..32 {
            for z in 0..32 {
                for x in 0..32 {
                    let block1 = chunks[13].get_block(uvec3(x, y, z));
                    let mut block2: Block = if x < 31 {
                        chunks[13].get_block(uvec3(x + 1, y, z))
                    } else {
                        chunks[13+1].get_block(uvec3(0, y, z))
                    };
                    let b1 = block1.is_full_block();
                    let mut b2 = block2.is_full_block();
                    if b1 != b2 {
                        if b1 == true {
                            World::add_face(
                                &chunks,
                                &mut storage,
                                uvec3(x, y, z),
                                Direction { id: 1 },
                                TEXTURE_INDEX[block1.block_type as usize - 1][1],
                            );
                        } else {
                            World::add_face(
                                &chunks,
                                &mut storage,
                                uvec3(x + 1, y, z),
                                Direction { id: 0 },
                                TEXTURE_INDEX[block2.block_type as usize - 1][0],
                            );
                        }
                    }
                    block2 = if z < 31 {
                        chunks[13].get_block(uvec3(x, y, z + 1))
                    } else {
                        chunks[13+9].get_block(uvec3(x, y, 0))
                    };
                    b2 = block2.is_full_block();
                    if b1 != b2 {
                        if b1 == true {
                            World::add_face(
                                &chunks,
                                &mut storage,
                                uvec3(x, y, z),
                                Direction { id: 3 },
                                TEXTURE_INDEX[block1.block_type as usize - 1][3],
                            );
                        } else {
                            World::add_face(
                                &chunks,
                                &mut storage,
                                uvec3(x, y, z + 1),
                                Direction { id: 2 },
                                TEXTURE_INDEX[block2.block_type as usize - 1][2],
                            );
                        }
                    }

                    block2 = if y < 31 {
                        chunks[13].get_block(uvec3(x, y + 1, z))
                    } else {
                        chunks[13+3].get_block(uvec3(x, 0, z))
                    };
                    b2 = block2.is_full_block();
                    if b1 != b2 {
                        if b1 == true {
                            World::add_face(
                                &chunks,
                                &mut storage,
                                uvec3(x, y, z),
                                Direction { id: 5 },
                                TEXTURE_INDEX[block1.block_type as usize - 1][5],
                            );
                        } else {
                            World::add_face(
                                &chunks,
                                &mut storage,
                                uvec3(x, y + 1, z),
                                Direction { id: 4 },
                                TEXTURE_INDEX[block2.block_type as usize - 1][4],
                            );
                        }
                    }
                }
            }
        }
        storage
    }
    #[profiling::function]
    pub fn set_block(&mut self, pos: IVec3, value: Block) {
        let chunk_pos = ivec3(pos.x >> 5, pos.y >> 5, pos.z >> 5);
        if let Some(chunk) = self.chunk_map.get_chunk_mut(chunk_pos) {
            let loc_pos = uvec3(
                (pos.x as u32) & 31,
                (pos.y as u32) & 31,
                (pos.z as u32) & 31,
            );
            self.chunk_updates.insert(chunk_pos);

            if loc_pos.x == 0 {
                self.chunk_updates.insert(chunk_pos + ivec3(-1, 0, 0));
            } else if loc_pos.x == 31 {
                self.chunk_updates.insert(chunk_pos + ivec3(1, 0, 0));
            }
            if loc_pos.y == 0 {
                self.chunk_updates.insert(chunk_pos + ivec3(0, -1, 0));
            } else if loc_pos.y == 31 {
                self.chunk_updates.insert(chunk_pos + ivec3(0, 1, 0));
            }
            if loc_pos.z == 0 {
                self.chunk_updates.insert(chunk_pos + ivec3(0, 0, -1));
            } else if loc_pos.z == 31 {
                self.chunk_updates.insert(chunk_pos + ivec3(0, 0, 1));
            }
            chunk.set_block(loc_pos, value);
        }
    }
    #[profiling::function]
    pub fn raycast(&self, pos: Vec3, dir: Vec3, place: bool) -> IVec3 {
        let mut block_pos = ivec3(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );
        let mut side_dist_x;
        let mut side_dist_y;
        let mut side_dist_z;
        let delta_dist_x = (1.0 / dir.x).abs();
        let delta_dist_y = (1.0 / dir.y).abs();
        let delta_dist_z = (1.0 / dir.z).abs();
        let step_x;
        let step_y;
        let step_z;
        let mut side = 0;
        if dir.x < 0.0 {
            step_x = -1;
            side_dist_x = (pos.x - block_pos.x as f32) * delta_dist_x;
        } else {
            step_x = 1;
            side_dist_x = (block_pos.x as f32 - pos.x + 1.0) * delta_dist_x;
        }
        if dir.y < 0.0 {
            step_y = -1;
            side_dist_y = (pos.y - block_pos.y as f32) * delta_dist_y;
        } else {
            step_y = 1;
            side_dist_y = (block_pos.y as f32 - pos.y + 1.0) * delta_dist_y;
        }
        if dir.z < 0.0 {
            step_z = -1;
            side_dist_z = (pos.z - block_pos.z as f32) * delta_dist_z;
        } else {
            step_z = 1;
            side_dist_z = (block_pos.z as f32 - pos.z + 1.0) * delta_dist_z;
        }
        let mut i = 0;
        while !self.chunk_map.get_block(block_pos).is_full_block() && i < 200 {
            i += 1;
            if side_dist_x < side_dist_y && side_dist_x < side_dist_z {
                side_dist_x += delta_dist_x;
                block_pos.x += step_x;
                side = 1;
            } else if side_dist_y < side_dist_z {
                side_dist_y += delta_dist_y;
                block_pos.y += step_y;
                side = 2;
            } else {
                side_dist_z += delta_dist_z;
                block_pos.z += step_z;
                side = 3;
            }
        }
        return if place {
            let mut face = ivec3(0, 0, 0);
            match side {
                1 => {
                    if dir.x < 0.0 {
                        face.x = 1;
                    } else {
                        face.x = -1;
                    }
                }
                2 => {
                    if dir.y < 0.0 {
                        face.y = 1;
                    } else {
                        face.y = -1;
                    }
                }
                _ => {
                    if dir.z < 0.0 {
                        face.z = 1;
                    } else {
                        face.z = -1;
                    }
                }
            }
            block_pos + face
        } else {
            block_pos
        };
    }
}

impl Drop for World {
    fn drop(&mut self) {
        for i in self.chunk_map.hash_map.drain() {
            self.chunk_loader.save(i);
        }
    }
}
