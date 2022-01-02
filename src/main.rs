#![feature(hash_drain_filter)]

use std::time::Instant;
use egui::FontDefinitions;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};
use egui_wgpu_backend::*;
use egui_winit_platform;
use egui_winit_platform::{Platform, PlatformDescriptor};
use crate::camera::Camera;
use crate::renderer::Renderer;
use crate::inputs::Inputs;
use crate::world::World;

mod texture;
mod renderer;
mod camera;
mod inputs;
mod chunk;
mod mesh;
mod block;
mod world;
mod chunk_loader;
mod util;
mod mipmap;
mod region;
mod chunk_map;
fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: window.inner_size().width as u32,
        physical_height: window.inner_size().height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });
    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut renderer = block_on(Renderer::new(&window));
    let mut camera = Camera::new(0.1);
    let mut inputs = Inputs::new();
    let mut world= World::new();
    camera.update(&inputs,&mut world);
    renderer.update(&camera);
    let mut counter:i32=0;
    let start_time=Instant::now();
    event_loop.run(move |event, _, control_flow| {
        platform.handle_event(&event);
        if !inputs.update(&event,&window) {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                }if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            renderer.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(_) => {
                    platform.update_time(start_time.elapsed().as_secs_f64());
                    renderer.update(&camera);
                    renderer.render(&world,&mut platform,&camera);
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    camera.update(&inputs,&mut world);
                    inputs.reset();
                    counter+=1;
                    world.update_display(&mut renderer);
                    if counter%3==0{
                        world.tick(&camera);
                    }
                    window.request_redraw();
                    profiling::finish_frame!()
                }
                _ => {}
            }
        }
    });
}