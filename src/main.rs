use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};
use crate::camera::Camera;
use crate::renderer::Renderer;
use crate::inputs::Inputs;
use crate::chunk::Chunk;
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


fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut renderer = block_on(Renderer::new(&window));
    let mut camera = Camera::new(0.1);
    let mut inputs = Inputs::new();
    let mut world= World::new(&renderer);
    camera.update(&inputs);
    renderer.update(&camera);
    let mut counter:i32=0;
    event_loop.run(move |event, _, control_flow| {
        if !inputs.update(&event,&window) {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                }if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
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
                    renderer.update(&camera);
                    renderer.render(&world);
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
                }
                _ => {}
            }
        }
        camera.update(&inputs);
        counter+=1;
        if counter%3==0{
            world.tick(&renderer,&camera);
        }
    });
}