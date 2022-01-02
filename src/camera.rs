use winit::{
    event::*,
};
use cgmath::{Rad, Point3, Matrix4, Vector3, vec3, point3};
use crate::inputs::*;
use cgmath::num_traits::clamp;
use crate::block::Block;
use crate::world::World;

pub struct Camera {
    pub pos: Point3<f32>,
    pub velocity: Vector3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    speed: f32,
}

impl Camera {

    pub fn new(speed: f32) -> Self {
        Self {
            pos: (0.0, 50.0, 0.0).into(),
            velocity: (0.0, 0.0, 0.0).into(),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            speed,
        }
    }

    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        Matrix4::look_to_rh(
            point3(self.pos.x.rem_euclid(32.0),self.pos.y.rem_euclid(32.0),self.pos.z.rem_euclid(32.0)),
            vec3(
                self.yaw.0.cos()*self.pitch.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin()*self.pitch.0.cos(),
            ),
            Vector3::unit_y(),
        )
    }

    pub fn update(&mut self,inputs:&Inputs,world:&mut World) {
        self.pitch = clamp(self.pitch+Rad((-inputs.mouse_motion_y / 200.0) as f32),Rad(-1.5),Rad(1.5));
        self.yaw += Rad((inputs.mouse_motion_x / 200.0) as f32);
        if inputs.keyboard[VirtualKeyCode::Z as usize] {
            self.velocity += vec3(self.yaw.0.cos(),0.0,self.yaw.0.sin())*self.speed;
        }
        if inputs.keyboard[VirtualKeyCode::S as usize] {
            self.velocity += -vec3(self.yaw.0.cos(),0.0,self.yaw.0.sin())*self.speed;
        }

        if inputs.keyboard[VirtualKeyCode::D as usize] {
            self.velocity += vec3(-self.yaw.0.sin(),0.0,self.yaw.0.cos())*self.speed;
        }
        if inputs.keyboard[VirtualKeyCode::Q as usize] {
            self.velocity += vec3(self.yaw.0.sin(),0.0,-self.yaw.0.cos())*self.speed;
        }
        if inputs.keyboard[VirtualKeyCode::R as usize] {
            self.velocity.y -= self.speed.min(0.1);
        }
        if inputs.keyboard[VirtualKeyCode::Space as usize] {
            self.velocity.y += self.speed.min(0.1);
        }
        if inputs.keyboard[VirtualKeyCode::W as usize] {
            self.speed/=1.05;
        }
        if inputs.keyboard[VirtualKeyCode::X as usize] {
            self.speed*=1.05;
        }
        if inputs.mouse_button_states[0]{
            world.set_block(world.raycast(self.pos,vec3(
                self.yaw.0.cos()*self.pitch.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin()*self.pitch.0.cos(),
            ),false),Block{block_type:0});
        }
        if inputs.mouse_button_states[2]{
            world.set_block(world.raycast(self.pos,vec3(
                self.yaw.0.cos()*self.pitch.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin()*self.pitch.0.cos(),
            ),true),Block{block_type:3});
        }
        self.pos+=self.velocity;
        self.velocity*=0.8;
    }
}