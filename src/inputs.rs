use winit::event::*;
use winit::window::Window;

pub struct Inputs{
    pub keyboard:[bool;170],
    pub mouse_pos_x:f64,
    pub mouse_pos_y:f64,
    pub mouse_motion_x:f64,
    pub mouse_motion_y:f64,
}
impl Inputs{
    pub fn new() ->Self{
        Inputs{
            keyboard:[false;170],
            mouse_pos_x: 0.0,
            mouse_pos_y: 0.0,
            mouse_motion_x: 0.0,
            mouse_motion_y: 0.0
        }
    }
    pub fn update(&mut self, event:&Event<()>,window :&Window) ->bool{
        self.mouse_motion_x=0.0;
        self.mouse_motion_y=0.0;
        match event {
            Event::DeviceEvent{
                ref event,
                ..
            } => match event{
                DeviceEvent::Key(
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                }
                ) => {
                    let is_pressed= *state==ElementState::Pressed;
                    self.keyboard[*key as usize]=is_pressed;
                    true
                }
                DeviceEvent::MouseMotion { delta }=>{
                    self.mouse_motion_x=delta.0;
                    self.mouse_motion_y=delta.1;
                    true
                }
                _ =>{true}
            }
            Event::WindowEvent{
                ref event,
                window_id,
            }if *window_id == window.id() => {
                match event {
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    }=>{
                        self.mouse_pos_x = position.x;
                        self.mouse_pos_y = position.y;
                        true
                    }
                    _ => {false}
                }
            }
            _ => {false}
        }
    }
}