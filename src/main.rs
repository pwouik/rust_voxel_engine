#![feature(hash_extract_if)]

use crate::camera::Camera;
use crate::inputs::Inputs;
use crate::renderer::Renderer;
use crate::world::World;
use profiling::tracy_client;
use renderdoc::{RenderDoc, V110};
use std::time::Instant;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod block;
mod camera;
mod chunk;
mod chunk_loader;
mod chunk_map;
mod chunk_renderer;
mod inputs;
mod mesh;
mod mipmap;
mod region;
mod render_region;
mod renderer;
mod texture;
mod util;
mod world;

fn main() {
    //let mut rd: RenderDoc<V110> = RenderDoc::new().expect("Unable to connect");

    env_logger::init();
    tracy_client::Client::start();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    use futures::executor::block_on;
    // Since main can't be async, we're going to need to block
    let mut renderer = block_on(Renderer::new(&window));
    let mut camera = Camera::new(0.1);
    let mut inputs = Inputs::new();
    let mut world = World::new();
    camera.update(&inputs, &mut world);
    let mut counter: i32 = 0;
    event_loop.run(move |event, _, control_flow| {
        if !inputs.update(&event, &window) {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
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
                    renderer.render(&camera);
                }
                Event::MainEventsCleared => {
                    camera.update(&inputs, &mut world);
                    inputs.reset();
                    counter += 1;
                    if counter % 3 == 0 {
                        world.tick(&camera, &mut renderer);
                    }
                    world.update_display(&mut renderer);
                    window.request_redraw();
                    profiling::finish_frame!();
                }
                _ => {}
            }
        }
    });
}
