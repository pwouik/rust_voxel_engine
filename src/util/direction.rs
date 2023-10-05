use glam::{ivec3, IVec3};

const NORMS: [IVec3; 6] = [
    ivec3(-1, 0, 0),
    ivec3(1, 0, 0),
    ivec3(0, 0, -1),
    ivec3(0, 0, 1),
    ivec3(0, -1, 0),
    ivec3(0, 1, 0),
];

pub struct Direction {
    pub id: u8,
}
impl Direction {
    pub fn get_norm(&self) -> IVec3 {
        NORMS[self.id as usize]
    }
    pub fn transform(&self, pos: IVec3) -> IVec3 {
        match self.id {
            0 => ivec3(-pos.y, pos.x, pos.z),
            1 => ivec3(pos.y, pos.x, -pos.z),
            2 => ivec3(-pos.z, pos.x, -pos.y),
            3 => ivec3(pos.z, pos.x, pos.y),
            4 => ivec3(pos.x, -pos.y, -pos.z),
            _ => pos,
        }
    }
}
