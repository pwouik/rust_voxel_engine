use crate::block::Block;
use crate::inputs::*;
use crate::world::World;
use glam::{vec3, Mat4, Vec3};
use winit::event::*;

pub struct Camera {
    pub pos: Vec3,
    pub velocity: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
}

impl Camera {
    pub fn new(speed: f32) -> Self {
        Self {
            pos: (10.0, 50.0, 10.0).into(),
            velocity: (0.0, 0.0, 0.0).into(),
            yaw: 0.0,
            pitch: 0.0,
            speed,
        }
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(
            vec3(
                self.pos.x.rem_euclid(32.0),
                self.pos.y.rem_euclid(32.0),
                self.pos.z.rem_euclid(32.0),
            ),
            vec3(
                self.yaw.cos() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.sin() * self.pitch.cos(),
            ),
            Vec3::Y,
        )
    }

    pub fn update(&mut self, inputs: &Inputs, world: &mut World) {
        self.pitch = (self.pitch + (-inputs.mouse_motion_y / 200.0) as f32).clamp(-1.5, 1.5);
        self.yaw += (inputs.mouse_motion_x / 200.0) as f32;
        if inputs.keyboard[VirtualKeyCode::Z as usize] {
            self.velocity += vec3(self.yaw.cos(), 0.0, self.yaw.sin()) * self.speed;
        }
        if inputs.keyboard[VirtualKeyCode::A as usize] {
            self.velocity += vec3(self.yaw.cos(), 0.0, self.yaw.sin()) * self.speed * 5.0;
        }
        if inputs.keyboard[VirtualKeyCode::S as usize] {
            self.velocity += -vec3(self.yaw.cos(), 0.0, self.yaw.sin()) * self.speed;
        }

        if inputs.keyboard[VirtualKeyCode::D as usize] {
            self.velocity += vec3(-self.yaw.sin(), 0.0, self.yaw.cos()) * self.speed;
        }
        if inputs.keyboard[VirtualKeyCode::Q as usize] {
            self.velocity += vec3(self.yaw.sin(), 0.0, -self.yaw.cos()) * self.speed;
        }
        if inputs.keyboard[VirtualKeyCode::R as usize] {
            self.velocity.y -= self.speed.min(0.1);
        }
        if inputs.keyboard[VirtualKeyCode::Space as usize] {
            self.velocity.y += self.speed.min(0.1);
        }
        if inputs.keyboard[VirtualKeyCode::W as usize] {
            self.speed /= 1.05;
        }
        if inputs.keyboard[VirtualKeyCode::X as usize] {
            self.speed *= 1.05;
        }
        if inputs.mouse_button_states[0] {
            world.set_block(
                world.raycast(
                    self.pos,
                    vec3(
                        self.yaw.cos() * self.pitch.cos(),
                        self.pitch.sin(),
                        self.yaw.sin() * self.pitch.cos(),
                    ),
                    false,
                ),
                Block { block_type: 0 },
            );
        }
        if inputs.mouse_button_states[2] {
            world.set_block(
                world.raycast(
                    self.pos,
                    vec3(
                        self.yaw.cos() * self.pitch.cos(),
                        self.pitch.sin(),
                        self.yaw.sin() * self.pitch.cos(),
                    ),
                    true,
                ),
                Block { block_type: 3 },
            );
        }
        self.pos += self.velocity;
        self.velocity *= 0.8;
    }
}
