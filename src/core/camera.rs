use std::{collections::HashMap, f32::consts::PI};

use nalgebra::{Matrix4, Point3, Rotation3, UnitVector3, Vector3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferUsages, Device,
    Queue,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::keyboard::KeyCode;

use crate::core::CAMERA_SPEED;

pub trait Camera {
    fn look_up(&mut self, amount: f32);
    fn look_ccw(&mut self, amount: f32);
    fn update(&mut self, keys_pressed: &HashMap<KeyCode, bool>, dt: f32);
    fn update_gpu(&mut self, queue: &mut Queue);
    fn bind_group(&self) -> &BindGroup;
}

#[derive(Debug, Clone)]
pub struct NoClipCamera {
    buffer: Buffer,
    bind_group: BindGroup,

    position: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    roll: f32,

    projection: Projection,

    // generated
    view_proj: nalgebra::Matrix4<f32>,
}

impl NoClipCamera {
    pub fn new(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
        position: Vector3<f32>,
        yaw: f32,
        pitch: f32,
        roll: f32,
        projection: Projection,
    ) -> Self {
        let roll = roll + PI;
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        let (sin_pitch, cos_pitch) = pitch.sin_cos();

        let view = Matrix4::look_at_rh(
            &position.into(),
            &(Point3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw) + position),
            &[0.0, 1.0, 0.0].into(),
        );
        let view_proj: Matrix4<f32> = (projection.projection() * view).into();

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[Into::<[[f32; 4]; 4]>::into(view_proj)]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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

            bind_group,
            buffer,
            view_proj,
        }
    }

    pub fn set_position(&mut self, position: &Vector3<f32>) {
        self.position = *position;
    }

    pub fn translate(&mut self, by: &Vector3<f32>) {
        self.position += *by;
    }

    pub fn forward(&mut self, amount: f32) {
        let (sin, cos) = self.yaw.sin_cos();
        let p_sin = self.pitch.sin();
        let dy = {
            if crate::core::CAMERA_USES_PITCH {
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

    pub fn roll_ccw(&mut self, amount: f32) {
        self.roll += amount;
    }

    fn create_view(&self) -> Matrix4<f32> {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let center = Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        Matrix4::look_at_rh(
            &(self.position.into()),
            &(Into::<Point3<f32>>::into([center.x, center.y, center.z]) + self.position),
            &(Rotation3::from_axis_angle(&(UnitVector3::new_normalize(center)), self.roll)
                * Vector3::new(0.0, 1.0, 0.0)),
        )
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.view_proj = self.projection.projection() * self.create_view();
    }

    pub fn view_proj(&self) -> &nalgebra::Matrix4<f32> {
        &self.view_proj
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.position
    }
}

impl Camera for NoClipCamera {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    fn look_up(&mut self, amount: f32) {
        self.pitch += amount;
        self.pitch = self.pitch.max(-PI / 2.0 + 0.1).min(PI / 2.0 - 0.1);
    }
    fn look_ccw(&mut self, amount: f32) {
        self.yaw += amount;
    }
    fn update(&mut self, keys_pressed: &HashMap<KeyCode, bool>, dt: f32) {
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
            }
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
        self.look_ccw(yaw_ccw);
        self.roll_ccw(roll_ccw);
        self.translate(&[0.0, fly, 0.0].into());

        self.view_proj = (self.projection.projection() * self.create_view()).into();
    }
    fn update_gpu(&mut self, queue: &mut Queue) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[Into::<[[f32; 4]; 4]>::into(self.view_proj)]),
        );
    }
}

#[derive(Debug, Clone)]
pub struct Projection {
    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,

    // generated
    transform: Matrix4<f32>,
}

impl Projection {
    pub fn new(width: f32, height: f32, fovy: f32, near: f32, far: f32) -> Self {
        Self {
            aspect: width / height,
            fovy: fovy,
            near,
            far,
            transform: *nalgebra::Perspective3::new(width / height, fovy * 180.0 / PI, near, far)
                .as_matrix(),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
        self.update();
    }

    pub fn projection(&self) -> &Matrix4<f32> {
        &self.transform
    }

    fn update(&mut self) {
        self.transform = nalgebra::Matrix4::new_perspective(
            self.aspect,
            self.fovy * 180.0 / PI,
            self.near,
            self.far,
        );
    }
}
