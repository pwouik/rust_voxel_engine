#![feature(hash_extract_if)]

use crate::camera::Camera;
use crate::inputs::Inputs;
use crate::renderer::Renderer;
use crate::world::World;
#[cfg(feature = "profile-with-tracy")]
use profiling::tracy_client;
use winit::keyboard::{Key, NamedKey};
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

mod block;
mod camera;
mod chunk;
mod chunk_loader;
mod chunk_map;
mod chunk_renderer;
mod entity;
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
    env_logger::init();
    #[cfg(feature = "profile-with-tracy")]
    tracy_client::Client::start();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    use futures::executor::block_on;

    let mut renderer = block_on(Renderer::new(&window));
    let mut camera = Camera::new(0.1);
    let mut inputs = Inputs::new();
    let mut world = World::new();
    camera.update(&inputs, &mut world);
    let mut counter: i32 = 0;
    event_loop
        .run(move |event, elwt| {
            if !inputs.update(&event, &window) {
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    } if window_id == window.id() => match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        } => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            renderer.render(&camera);
                        }
                        _ => {}
                    },
                    Event::AboutToWait => {
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
        })
        .unwrap();
}
