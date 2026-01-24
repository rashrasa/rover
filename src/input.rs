use std::{collections::HashMap, f32::consts::PI, time::Duration};

use cgmath::Rad;
use log::info;
use rodio::Sink;
use winit::{
    dpi::PhysicalPosition,
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{CAMERA_SPEED, render::camera::Camera};

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
    pub fn window_event(&mut self, event: &WindowEvent, window: &Window, camera: &mut Camera) {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if let PhysicalKey::Code(k) = event.physical_key {
                    self.keys_pressed.insert(k, event.state.is_pressed());
                    if k == KeyCode::Escape && event.state.is_pressed() {
                        self.esc_toggle = !self.esc_toggle;
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if !self.esc_toggle {
                    let size = window.inner_size();
                    window
                        .set_cursor_position(PhysicalPosition::new(size.width / 2, size.height / 2))
                        .unwrap();
                    camera.look_up(Rad((size.height as f32 / 2.0 - position.y as f32)
                        / size.height as f32
                        * PI));
                    camera.look_ccw(Rad((position.x as f32 - size.width as f32 / 2.0)
                        / size.width as f32
                        * PI));
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32, camera: &mut Camera, sink: &mut Sink) {
        let mut camera_forward: f32 = 0.0;
        let mut camera_right: f32 = 0.0;
        let mut yaw_ccw: f32 = 0.0;
        let mut fly: f32 = 0.0;
        let mut fly_speed: f32 = CAMERA_SPEED;
        let mut roll_ccw: f32 = 0.0;

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
                roll_ccw += 0.025;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::KeyE) {
            if *p {
                roll_ccw -= 0.025;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::Space) {
            if *p {
                fly += 2.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::ShiftLeft) {
            if *p {
                fly -= 2.0;
            }
        }
        if let Some(p) = self.keys_pressed.get(&KeyCode::ControlLeft) {
            if *p {
                fly_speed *= 100.0;
                sink.set_speed(2.0);
            } else {
                sink.set_speed(1.0);
            }
        } else {
            sink.set_speed(1.0);
        }

        let mag = (camera_forward * camera_forward + camera_right * camera_right).sqrt();
        camera_forward /= mag;
        camera_right /= mag;

        camera_forward *= fly_speed * dt;
        camera_right *= fly_speed * dt;
        yaw_ccw *= fly_speed * dt;
        fly *= fly_speed * dt;

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
        if roll_ccw.is_nan() {
            roll_ccw = 0.0;
        }

        camera.forward(camera_forward);
        camera.right(camera_right);
        camera.look_ccw(Rad(yaw_ccw));
        camera.roll_ccw(Rad(roll_ccw));
        camera.translate(&[0.0, fly, 0.0].into());

        if camera_forward.abs() + camera_right.abs() > 1.0e-2 {
            sink.play();
            if sink.get_pos() > Duration::new(5, 0) {
                sink.try_seek(Duration::ZERO).unwrap();
            }
        } else {
            sink.pause();
        }
    }
}
