use std::f32::consts::PI;

use cgmath::{Matrix4, Rad, Vector3, Vector4};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::Integrator;

#[derive(Clone, Debug)]
/// Long-lived struct containing all data relevant to an entity.
///
/// Ideally never cloned.
pub struct Entity {
    id: String,
    mesh_id: String,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    bounding_box: (Vector3<f32>, Vector3<f32>),
    model: Matrix4<f32>,
}

impl Entity {
    pub fn new(
        id: &str,
        mesh_id: &str,
        velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
        bounding_box: (Vector3<f32>, Vector3<f32>),
        model: Matrix4<f32>,
    ) -> Self {
        Self {
            id: id.into(),
            mesh_id: mesh_id.into(),
            velocity,
            acceleration,
            bounding_box,
            model,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn mesh_id(&self) -> &str {
        &self.mesh_id
    }

    pub fn tick(&mut self, dt: f32) {
        match crate::GLOBAL_INTEGRATOR {
            Integrator::RK4 => {
                let k1 = self.acceleration;
                let k2 = self.acceleration + k1 * dt / 2.0;
                let k3 = self.acceleration + k2 * dt / 2.0;
                let k4 = self.acceleration + k3 * dt;
                self.velocity += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;

                let k1 = self.velocity;
                let k2 = self.velocity + k1 * dt / 2.0;
                let k3 = self.velocity + k2 * dt / 2.0;
                let k4 = self.velocity + k3 * dt;

                let translation = (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;
                self.model.w.x += translation.x;
                self.model.w.y += translation.y;
                self.model.w.z += translation.z;
            }
            Integrator::Euler => {
                todo!();
            }
        }
    }

    pub fn position(&self) -> &Vector4<f32> {
        &self.model.w
    }

    /// Returns Top-Left-Front and Bottom-Right-Back vertex vectors
    pub fn bounding_box(&self) -> &(Vector3<f32>, Vector3<f32>) {
        &self.bounding_box
    }

    pub fn translate(&mut self, by: Vector3<f32>) {
        self.model.w.x += by.x;
        self.model.w.y += by.y;
        self.model.w.z += by.z;
    }

    pub fn model(&self) -> &Matrix4<f32> {
        &self.model
    }

    pub const fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<[[f32; 4]; 4]>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}
