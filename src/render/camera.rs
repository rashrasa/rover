use std::f32::consts::PI;

use cgmath::{Angle, EuclideanSpace, InnerSpace, Matrix3, Matrix4, Point3, Rad, Vector3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::OPENGL_TO_WGPU_MATRIX;

#[derive(Debug, Clone)]
pub struct Camera {
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

impl Camera {
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
    pub fn look_up(&mut self, amount: Rad<f32>) {
        self.pitch += amount;
        self.pitch = Rad(self.pitch.0.max(-PI / 2.0 + 0.1).min(PI / 2.0 - 0.1));
    }
    pub fn look_ccw(&mut self, amount: Rad<f32>) {
        self.yaw += amount;
    }
    pub fn roll_ccw(&mut self, amount: Rad<f32>) {
        self.roll += amount;
    }

    pub fn update(&mut self, queue: &mut Queue) {
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let center = Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        let view = Matrix4::look_at_rh(
            self.position,
            Into::<Point3<f32>>::into([center.x, center.y, center.z]) + self.position.to_vec(),
            Matrix3::from_angle_z(self.roll) * Vector3::new(0.0, 1.0, 0.0),
        );

        self.view_proj = (OPENGL_TO_WGPU_MATRIX * self.projection.projection() * view).into();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.view_proj));
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
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
