use std::{collections::HashMap, f32::consts::PI};

use winit::{
    dpi::PhysicalPosition,
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::core::camera::Camera;

pub struct InputController {
    keys_pressed: HashMap<KeyCode, bool>,
    esc_toggle: bool,
}

impl InputController {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashMap::with_capacity(100),
            esc_toggle: false,
        }
    }

    /// This will only handle events relevant to input. Other events should be handled in App.window_event().
    pub fn window_event(&mut self, event: &WindowEvent, window: &Window, camera: &mut impl Camera) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(k) = event.physical_key {
                    self.keys_pressed.insert(k, event.state.is_pressed());
                    if k == KeyCode::Escape && event.state.is_pressed() {
                        self.esc_toggle = !self.esc_toggle;
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                if !self.esc_toggle {
                    let size = window.inner_size();
                    window
                        .set_cursor_position(PhysicalPosition::new(size.width / 2, size.height / 2))
                        .unwrap();
                    camera.look_up(
                        (size.height as f32 / 2.0 - position.y as f32) / size.height as f32 * PI,
                    );
                    camera.look_ccw(
                        (position.x as f32 - size.width as f32 / 2.0) / size.width as f32 * PI,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32, camera: &mut impl Camera) {
        camera.update(&self.keys_pressed, dt);
    }
}
