use std::{collections::HashMap, f32::consts::PI, time::Duration};

use cgmath::{Angle, EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, Rad, Vector3};
use rodio::Sink;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::keyboard::{Key, KeyCode};

use crate::{CAMERA_SPEED, GROUND_HEIGHT, OPENGL_TO_WGPU_MATRIX};

pub trait Camera {
    fn look_up(&mut self, amount: Rad<f32>);
    fn look_ccw(&mut self, amount: Rad<f32>);
    fn update(&mut self, keys_pressed: &HashMap<KeyCode, bool>, sink: &mut Sink, dt: f32);
    fn update_gpu(&mut self, queue: &mut Queue);
    fn bind_group(&self) -> &BindGroup;
}

#[derive(Debug, Clone)]
pub struct NoClipCamera {
    buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,

    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    roll: Rad<f32>,

    projection: Projection,

    // generated
    view_proj: [[f32; 4]; 4],
}

impl NoClipCamera {
    pub fn new(
        device: &mut Device,
        position: Point3<f32>,
        yaw: Rad<f32>,
        pitch: Rad<f32>,
        roll: Rad<f32>,
        projection: Projection,
    ) -> Self {
        let (sin_yaw, cos_yaw) = yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = pitch.0.sin_cos();

        let view = Matrix4::look_at_rh(
            position,
            Point3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw) + position.to_vec(),
            [0.0, 1.0, 0.0].into(),
        );
        let view_proj = (OPENGL_TO_WGPU_MATRIX * projection.projection() * view).into();

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[view_proj]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            position,
            yaw,
            pitch,
            projection,
            roll,

            view_proj,
            bind_group,
            buffer,
            bind_group_layout,
        }
    }

    pub fn set_position(&mut self, position: &Point3<f32>) {
        self.position = *position;
    }

    pub fn translate(&mut self, by: &Vector3<f32>) {
        self.position += *by;
    }

    pub fn forward(&mut self, amount: f32) {
        let (sin, cos) = self.yaw.sin_cos();
        let p_sin = self.pitch.sin();
        let dy = {
            if crate::CAMERA_USES_PITCH {
                amount * p_sin
            } else {
                0.0
            }
        };
        self.translate(&[amount * cos, dy, amount * sin].into());
    }
    pub fn right(&mut self, amount: f32) {
        let (sin, cos) = self.yaw.sin_cos();
        self.translate(&[-amount * sin, 0.0, amount * cos].into());
    }

    pub fn roll_ccw(&mut self, amount: Rad<f32>) {
        self.roll += amount;
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}

impl Camera for NoClipCamera {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    fn look_up(&mut self, amount: Rad<f32>) {
        self.pitch += amount;
        self.pitch = Rad(self.pitch.0.max(-PI / 2.0 + 0.1).min(PI / 2.0 - 0.1));
    }
    fn look_ccw(&mut self, amount: Rad<f32>) {
        self.yaw += amount;
    }
    fn update(&mut self, keys_pressed: &HashMap<KeyCode, bool>, sink: &mut Sink, dt: f32) {
        let mut camera_forward: f32 = 0.0;
        let mut camera_right: f32 = 0.0;
        let mut yaw_ccw: f32 = 0.0;
        let mut fly: f32 = 0.0;
        let mut fly_speed: f32 = CAMERA_SPEED;
        let mut roll_ccw: f32 = 0.0;

        if let Some(p) = keys_pressed.get(&KeyCode::KeyW) {
            if *p {
                camera_forward += 1.0;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::KeyS) {
            if *p {
                camera_forward -= 1.0;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::KeyA) {
            if *p {
                camera_right -= 1.0;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::KeyD) {
            if *p {
                camera_right += 1.0;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::KeyQ) {
            if *p {
                roll_ccw += 0.025;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::KeyE) {
            if *p {
                roll_ccw -= 0.025;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::Space) {
            if *p {
                fly += 1.0;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::ShiftLeft) {
            if *p {
                fly -= 1.0;
            }
        }
        if let Some(p) = keys_pressed.get(&KeyCode::ControlLeft) {
            if *p {
                fly_speed *= 20.0;
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

        self.forward(camera_forward);
        self.right(camera_right);
        self.look_ccw(Rad(yaw_ccw));
        self.roll_ccw(Rad(roll_ccw));
        self.translate(&[0.0, fly, 0.0].into());
        self.position.y = GROUND_HEIGHT as f32; // TODO: remove

        if camera_forward.abs() + camera_right.abs() > 1.0e-2 * dt {
            sink.play();
            if sink.get_pos() > Duration::new(5, 0) {
                sink.try_seek(Duration::ZERO).unwrap();
            }
        } else {
            sink.pause();
        }
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let center = Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        let view = Matrix4::look_at_rh(
            self.position,
            Into::<Point3<f32>>::into([center.x, center.y, center.z]) + self.position.to_vec(),
            Matrix3::from_axis_angle(center, self.roll) * Vector3::new(0.0, 1.0, 0.0),
        );

        self.view_proj = (OPENGL_TO_WGPU_MATRIX * self.projection.projection() * view).into();
    }
    fn update_gpu(&mut self, queue: &mut Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.view_proj));
    }
}

#[derive(Debug, Clone)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    near: f32,
    far: f32,

    model: Matrix4<f32>,
}

impl Projection {
    pub fn new(width: f32, height: f32, fovy: Rad<f32>, near: f32, far: f32) -> Self {
        Self {
            aspect: width / height,
            fovy: fovy,
            near,
            far,
            model: OPENGL_TO_WGPU_MATRIX * cgmath::perspective(fovy, width / height, near, far),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
        self.update();
    }

    pub fn projection(&self) -> &Matrix4<f32> {
        &self.model
    }

    fn update(&mut self) {
        self.model = OPENGL_TO_WGPU_MATRIX
            * cgmath::perspective(self.fovy, self.aspect, self.near, self.far);
    }
}
