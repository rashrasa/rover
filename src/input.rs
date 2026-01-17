use std::collections::HashMap;

use cgmath::Rad;
use winit::{
    dpi::PhysicalPosition,
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    CAMERA_SPEED,
    render::{
        Renderer,
        camera::{Camera, CameraUniform},
    },
};

pub struct InputController {
    keys_pressed: HashMap<KeyCode, bool>,
}

impl InputController {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashMap::with_capacity(100),
        }
    }

    /// This will only handle events relevant to input. Other events should be handled in App.window_event().
    pub fn window_event(&mut self, event: &WindowEvent, window: &Window) {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if let PhysicalKey::Code(k) = event.physical_key {
                    self.keys_pressed.insert(k, event.state.is_pressed());
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let size = window.inner_size();
                // window
                //     .set_cursor_position(PhysicalPosition::new(size.width / 2, size.height / 2))
                //     .unwrap();
            }
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32, camera: &mut Camera, camera_uniform: &mut CameraUniform) {
        let mut camera_forward: f32 = 0.0;
        let mut camera_right: f32 = 0.0;
        let mut yaw_ccw: f32 = 0.0;
        let mut fly: f32 = 0.0;

        if let Some(p) = self.keys_pressed.get(&KeyCode::KeyW) {
            if *p {
                camera_forward += 1.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::KeyS) {
            if *p {
                camera_forward -= 1.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::KeyA) {
            if *p {
                camera_right -= 1.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::KeyD) {
            if *p {
                camera_right += 1.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::KeyQ) {
            if *p {
                yaw_ccw += 1.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::KeyE) {
            if *p {
                yaw_ccw -= 1.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::Space) {
            if *p {
                fly += 1.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::ShiftLeft) {
            if *p {
                fly -= 1.0;
            }
        }

        let mag = (camera_forward * camera_forward + camera_right * camera_right).sqrt();
        camera_forward /= mag;
        camera_right /= mag;

        camera_forward *= CAMERA_SPEED * dt;
        camera_right *= CAMERA_SPEED * dt;
        yaw_ccw *= CAMERA_SPEED * dt;
        fly *= CAMERA_SPEED * dt;

        if camera_forward.is_nan() {
            camera_forward = 0.0;
        }
        if camera_right.is_nan() {
            camera_right = 0.0;
        }
        if yaw_ccw.is_nan() {
            yaw_ccw = 0.0;
        }
        if fly.is_nan() {
            fly = 0.0;
        }

        camera.forward(camera_forward);
        camera.right(camera_right);
        camera.look_left(Rad(yaw_ccw));
        camera.translate(&[0.0, fly, 0.0].into());
        camera_uniform.update(&camera);
    }
}
