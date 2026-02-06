use nalgebra::{Matrix, Matrix4, Vector3, Vector4};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::{
    ContiguousView, ContiguousViewMut, Integrator,
    core::{
        camera::{Camera, NoClipCamera},
        entity::{
            self, BoundingBox, Collide, CollisionResponse, Dynamic, Mass, RenderInstanced,
            Transform,
        },
    },
};

#[derive(Debug)]
/// Long-lived struct containing all data relevant to a player.
///
/// Ideally never cloned.
pub struct Player {
    id: u64,
    mesh_id: u64,
    texture_id: u64,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    bounding_box: BoundingBox,
    transform: Matrix4<f32>,
    camera: NoClipCamera, // TODO: camera and transform both store a position
    response: CollisionResponse,
    mass: f32,

    // cached
    instance: [[f32; 4]; 4],
}

impl Player {
    pub fn new(
        id: u64,
        mesh_id: u64,
        texture_id: u64,
        velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
        bounding_box: BoundingBox,
        transform: Matrix4<f32>,
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
            transform,
            camera,
            response,
            mass,
            instance: transform.into(),
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

impl super::Dynamic for Player {
    fn velocity<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.acceleration.fixed_view::<3, 1>(0, 0)
    }

    fn velocity_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.acceleration.fixed_view_mut::<3, 1>(0, 0)
    }

    fn acceleration<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.acceleration.fixed_view::<3, 1>(0, 0)
    }

    fn acceleration_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.acceleration.fixed_view_mut::<3, 1>(0, 0)
    }
}

impl super::Transform for Player {
    fn transform<'a>(&'a self) -> ContiguousView<'a, 4, 4> {
        self.transform.fixed_view::<4, 4>(0, 3)
    }

    fn transform_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 4, 4> {
        self.transform.fixed_view_mut::<4, 4>(0, 3)
    }
}

impl super::Position for Player {
    fn position<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.transform.position()
    }

    fn position_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.transform.position_mut()
    }
}

impl super::Mass for Player {
    fn mass(&self) -> &f32 {
        &self.mass
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

impl super::RenderUniform for Player {
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
    fn view_proj<'a>(&'a self) -> ContiguousView<'a, 4, 4> {
        self.camera.view_proj().fixed_view::<4, 4>(0, 0)
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

impl RenderInstanced<[[f32; 4]; 4]> for Player {
    fn texture_id(&self) -> &u64 {
        &self.texture_id
    }

    fn mesh_id(&self) -> &u64 {
        &self.mesh_id
    }

    fn instance(&self) -> &[[f32; 4]; 4] {
        &self.instance
    }
}
