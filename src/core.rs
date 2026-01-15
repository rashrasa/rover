use bytemuck::{Pod, Zeroable};
use cgmath::SquareMatrix;
use nalgebra::Vector3;

use crate::{OPENGL_TO_WGPU_MATRIX, render::data::Vertex};

#[derive(Clone, Debug)]
/// Long-lived struct containing all data relevant to an entity.
///
/// Ideally never cloned.
pub struct Entity {
    id: String,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    geometry: Geometry<8, 36>,
    bounding_box: (Vector3<f32>, Vector3<f32>),
}

impl Entity {
    pub fn new(
        id: String,
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
        bounding_box: (Vector3<f32>, Vector3<f32>),
    ) -> Self {
        let t = Vector3::new(0.0, bounding_box.0.y, 0.0);
        let l = Vector3::new(bounding_box.0.x, 0.0, 0.0);
        let f = Vector3::new(0.0, 0.0, bounding_box.0.z);

        let bo = Vector3::new(0.0, bounding_box.1.y, 0.0);
        let r = Vector3::new(bounding_box.1.x, 0.0, 0.0);
        let ba = Vector3::new(0.0, 0.0, bounding_box.1.z);
        Self {
            id,
            position,
            velocity,
            acceleration,
            bounding_box,
            geometry: Geometry {
                vertices: [
                    Vertex {
                        position: (position + t + l + f).into(),
                        color: [1.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: (position + t + r + f).into(),
                        color: [1.0, 1.0, 1.0],
                    },
                    Vertex {
                        position: (position + bo + r + f).into(),
                        color: [0.0, 1.0, 0.0],
                    },
                    Vertex {
                        position: (position + bo + l + f).into(),
                        color: [0.0, 1.0, 1.0],
                    },
                    Vertex {
                        position: (position + t + l + ba).into(),
                        color: [0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: (position + t + r + ba).into(),
                        color: [1.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: (position + bo + r + ba).into(),
                        color: [1.0, 1.0, 0.0],
                    },
                    Vertex {
                        position: (position + bo + l + ba).into(),
                        color: [1.0, 1.0, 1.0],
                    },
                ],
                #[rustfmt::skip]
                indices: [
                    0, 3, 2,    2, 1, 0,
                    1, 2, 6,    6, 5, 1,
                    5, 6, 7,    7, 4, 5,
                    4, 7, 3,    3, 0, 4,
                    4, 0, 1,    1, 5, 4,
                    3, 7, 6,    6, 2, 3
                ],
            },
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn tick(&mut self, dt: f32) {
        let k1 = self.acceleration;
        let k2 = self.acceleration + k1 * dt / 2.0;
        let k3 = self.acceleration + k2 * dt / 2.0;
        let k4 = self.acceleration + k3 * dt;
        self.velocity += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;

        let k1 = self.velocity;
        let k2 = self.velocity + k1 * dt / 2.0;
        let k3 = self.velocity + k2 * dt / 2.0;
        let k4 = self.velocity + k3 * dt;
        self.position += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;
    }
    pub fn position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn bounding_box(&self) -> &(Vector3<f32>, Vector3<f32>) {
        &self.bounding_box
    }
    /// Returns Top-Left-Front and Bottom-Right-Back vertex vectors
    pub fn translate(&mut self, by: Vector3<f32>) {
        self.position += by;
    }

    pub fn geometry(&self) -> &Geometry<8, 36> {
        &self.geometry
    }
}

#[derive(Debug, Clone)]
pub struct Geometry<const N: usize, const I: usize> {
    pub vertices: [Vertex; N],
    pub indices: [u16; I],
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(
            cgmath::Deg(self.fov_y),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        return OPENGL_TO_WGPU_MATRIX * proj * view;
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
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
