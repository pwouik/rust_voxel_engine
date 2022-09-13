use cgmath::{vec3, Vector3};

const NORMS: [Vector3<i32>; 6] = [
    vec3(-1, 0, 0),
    vec3(1, 0, 0),
    vec3(0, 0, -1),
    vec3(0, 0, 1),
    vec3(0, -1, 0),
    vec3(0, 1, 0),
];

pub struct Direction {
    pub id: u8,
}
impl Direction {
    pub fn get_norm(&self) -> Vector3<i32> {
        NORMS[self.id as usize]
    }
    pub fn transform(&self, pos: Vector3<i32>) -> Vector3<i32> {
        match self.id {
            0 => vec3(-pos.y, pos.x, pos.z),
            1 => vec3(pos.y, pos.x, -pos.z),
            2 => vec3(-pos.z, pos.x, -pos.y),
            3 => vec3(pos.z, pos.x, pos.y),
            4 => vec3(pos.x, -pos.y, -pos.z),
            _ => pos,
        }
    }
}
