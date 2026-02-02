use nalgebra::{Matrix, Matrix4, Vector3, Vector4};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::{
    Integrator,
    core::{
        camera::{Camera, NoClipCamera},
        entity::{self, BoundingBox, Collide, CollisionResponse, Dynamic, Mass, Transform},
    },
};

#[derive(Debug)]
/// Long-lived struct containing all data relevant to an entity.
///
/// Ideally never cloned.
pub struct Player {
    id: u64,
    mesh_id: u64,
    texture_id: u64,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    bounding_box: BoundingBox,
    model: Matrix4<f32>,
    camera: NoClipCamera,
    response: CollisionResponse,
    mass: f32,
}

impl Player {
    pub fn new(
        id: u64,
        mesh_id: u64,
        texture_id: u64,
        velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
        bounding_box: BoundingBox,
        model: Matrix4<f32>,
        camera: NoClipCamera,
        response: CollisionResponse,
        mass: f32,
    ) -> Self {
        Self {
            id,
            mesh_id,
            texture_id,
            velocity,
            acceleration,
            bounding_box,
            model,
            camera,
            response,
            mass,
        }
    }

    pub fn id(&self) -> &u64 {
        &self.id
    }

    pub fn mesh_id(&self) -> &u64 {
        &self.mesh_id
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

impl super::Transform for Player {
    fn transform(&self) -> &Matrix4<f32> {
        &self.model
    }
    fn transform_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.model
    }
}

impl super::Mass for Player {
    fn mass(&self) -> &f32 {
        &self.mass
    }
}

impl super::Dynamic for Player {
    fn velocity(&self) -> &Vector3<f32> {
        &self.velocity
    }

    fn velocity_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.velocity
    }

    fn acceleration(&self) -> &Vector3<f32> {
        &self.acceleration
    }

    fn acceleration_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.acceleration
    }
}

impl super::Collide for Player {
    fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
    fn response(&self) -> &CollisionResponse {
        &self.response
    }
}

impl super::Render for Player {
    fn texture_id(&self) -> &u64 {
        &self.texture_id
    }

    fn mesh_id(&self) -> &u64 {
        &self.mesh_id
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        self.camera.bind_group()
    }
}

impl super::Entity for Player {
    fn id(&self) -> &u64 {
        &self.id
    }
}

impl super::View for Player {
    fn view_proj(&self) -> &Matrix4<f32> {
        self.camera.view_proj()
    }

    fn set_projection(&mut self, projection: crate::core::camera::Projection) {
        self.camera.set_projection(projection);
    }
}

impl Camera for Player {
    fn look_up(&mut self, amount: f32) {
        self.camera.look_up(amount);
    }

    fn look_ccw(&mut self, amount: f32) {
        self.camera.look_ccw(amount);
    }

    fn update(
        &mut self,
        keys_pressed: &std::collections::HashMap<winit::keyboard::KeyCode, bool>,
        sink: &mut rodio::Sink,
        dt: f32,
    ) {
        self.camera.update(keys_pressed, sink, dt);
    }

    fn update_gpu(&mut self, queue: &mut wgpu::Queue) {
        self.camera.update_gpu(queue);
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        self.camera.bind_group()
    }
}
