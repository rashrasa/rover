use bytemuck::{Pod, Zeroable};
use cgmath::{Array, Matrix4, SquareMatrix, Vector3, Vector4};
use log::debug;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::{OPENGL_TO_WGPU_MATRIX, render::data::Vertex};

#[derive(Clone, Debug)]
/// Long-lived struct containing all data relevant to an entity.
///
/// Ideally never cloned.
pub struct Entity {
    id: String,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    bounding_box: (Vector3<f32>, Vector3<f32>),
    mesh_type: MeshType,
    model: Matrix4<f32>,
}

impl Entity {
    pub fn new(
        id: String,
        velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
        bounding_box: (Vector3<f32>, Vector3<f32>),
        mesh_type: MeshType,
        model: Matrix4<f32>,
    ) -> Self {
        // TODO: Extremely inefficent, only for testing

        Self {
            id,
            velocity,
            acceleration,
            bounding_box,
            mesh_type,
            model,
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

        let translation = (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;
        self.model.w.x += translation.x;
        self.model.w.y += translation.y;
        self.model.w.z += translation.z;
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MeshType {
    Cube,
}

#[derive(Debug)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Mesh {
    pub const fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            vertices: vertices,
            indices: indices,
        }
    }
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
}
