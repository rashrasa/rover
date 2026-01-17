use bytemuck::{Pod, Zeroable};
use cgmath::{
    Angle, EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Transform, Vector3,
};

use crate::OPENGL_TO_WGPU_MATRIX;

#[derive(Debug, Clone)]
pub struct Camera {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    projection: Projection,

    // generated
    view_proj: Matrix4<f32>,
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        yaw: Rad<f32>,
        pitch: Rad<f32>,
        projection: Projection,
    ) -> Self {
        let (sin_yaw, cos_yaw) = yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = pitch.0.sin_cos();

        let view = Matrix4::look_at_rh(
            position,
            Point3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw) + position.to_vec(),
            [0.0, 1.0, 0.0].into(),
        );
        let view_proj = OPENGL_TO_WGPU_MATRIX * projection.projection() * view;

        Self {
            position,
            yaw,
            pitch,
            projection,

            view_proj,
        }
    }

    pub fn set_position(&mut self, position: &Point3<f32>) {
        self.position = *position;
        self.update();
    }

    pub fn translate(&mut self, by: &Vector3<f32>) {
        self.position += *by;
        self.update();
    }

    pub fn forward(&mut self, amount: f32) {
        let (sin, cos) = self.yaw.sin_cos();
        self.translate(&[amount * cos, 0.0, amount * sin].into());
        self.update();
    }
    pub fn backward(&mut self, amount: f32) {
        let (sin, cos) = self.yaw.sin_cos();
        self.translate(&[-amount * cos, 0.0, -amount * sin].into());
        self.update();
    }
    pub fn left(&mut self, amount: f32) {
        let (sin, cos) = self.yaw.sin_cos();
        self.translate(&[amount * sin, 0.0, -amount * cos].into());
        self.update();
    }
    pub fn right(&mut self, amount: f32) {
        let (sin, cos) = self.yaw.sin_cos();
        self.translate(&[-amount * sin, 0.0, amount * cos].into());
        self.update();
    }

    pub fn look_right(&mut self, amount: Rad<f32>) {
        self.yaw += amount;
        self.update();
    }

    pub fn look_left(&mut self, amount: Rad<f32>) {
        self.yaw -= amount;
        self.update();
    }

    pub fn view_projection(&self) -> &Matrix4<f32> {
        &self.view_proj
    }

    fn update(&mut self) {
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let center = Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();
        let view = Matrix4::look_at_rh(
            self.position,
            Into::<Point3<f32>>::into([center.x, center.y, center.z]) + self.position.to_vec(),
            [0.0, 1.0, 0.0].into(),
        );

        self.view_proj = OPENGL_TO_WGPU_MATRIX * self.projection.projection() * view;
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

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = (*camera.view_projection()).into();
    }
}
