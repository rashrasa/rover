use std::fmt::Debug;

use nalgebra::{Matrix4, UnitQuaternion, Vector3, Vector4};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::{
    Integrator,
    core::{G, Instanced, Meshed, Unique, camera::NoClipCamera},
};

/// Elastic collisions have CollisionResponse::Inelastic(1.0).
/// Inelastic takes any value. Values exceeding 1.0 will result in
/// energy magically being added to the system. Values below 0.0 will
/// be clamped to 0.0.
#[derive(Debug)]
pub enum CollisionResponse {
    Immovable,
    Inelastic(f32),
}

#[derive(Debug)]
pub struct BoundingBox {
    x: f32,
    y: f32,
    z: f32,

    x_size: f32,
    y_size: f32,
    z_size: f32,
}

impl BoundingBox {
    pub const fn new(top_left_front: (f32, f32, f32), size: (f32, f32, f32)) -> Self {
        Self {
            x: top_left_front.0,
            y: top_left_front.1,
            z: top_left_front.2,
            x_size: size.0,
            y_size: size.1,
            z_size: size.2,
        }
    }

    pub const ZERO: BoundingBox = Self::new((0.0, 0.0, 0.0), (0.0, 0.0, 0.0));

    /// Returns None if they don't intersect.
    ///
    /// Result vector is a signed distance of how far they intersect in each axis.
    pub fn intersects(&self, other: &BoundingBox) -> Option<[f32; 3]> {
        let x = (other.x + other.x_size) / 2.0 - (self.x + self.x_size) / 2.0;
        let y = (other.y + other.y_size) / 2.0 - (self.y + self.y_size) / 2.0;
        let z = (other.z + other.z_size) / 2.0 - (self.z + self.z_size) / 2.0;

        let min_x_size = self.x_size.min(other.x_size);
        let _min_y_size = self.y_size.min(other.y_size);
        let _min_z_size = self.z_size.min(other.z_size);

        if x.abs() > min_x_size || x.abs() > min_x_size || x.abs() > min_x_size {
            return None;
        }

        Some([x, y, z])
    }
}

// ********************************************************************************** //
// ************************************ HELPERS ************************************* //
// ********************************************************************************** //

/* These helper functions are how each trait is handled.
 * If specific traits need more data, additional trait bounds should be specified.
 * Most world logic should live here.
 */

/// Performs object-object collisions for every element in the list.
pub fn perform_collisions(entities: &mut Vec<Entity>) {
    // Calculate each collision and add them up for each object.

    for i in 0..entities.len() {
        for j in 0..entities.len() {
            if i != j {
                let a = entities.get_mut(i).unwrap() as *mut Entity;
                let b = entities.get_mut(j).unwrap() as *mut Entity;
                // SAFETY: [a] and [b] are guaranteed to be valid, different Entity items,
                //         as long as no other threads use [entities] or modify any entities.
                unsafe {
                    a.as_mut().unwrap().apply_gravity(b.as_mut().unwrap());
                }
            }
        }
    }
}

pub enum EntityType {
    Player {
        // TODO: camera and transform both store a position
        camera: NoClipCamera,
    },
    Object,
}

pub struct Entity {
    // Keys
    id: u64,
    mesh_id: u64,
    texture_id: u64,

    // Transforms, in order
    scale: Vector3<f32>,
    rotation: UnitQuaternion<f32>,
    translation: Vector3<f32>,

    // Physics
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    bounding_box: BoundingBox,

    entity_type: EntityType,
    response: CollisionResponse,
    mass: f32,
}

impl Entity {
    pub fn new(
        id: u64,
        mesh_id: u64,
        texture_id: u64,
        scale: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        translation: Vector3<f32>,
        velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
        bounding_box: BoundingBox,

        entity_type: EntityType,
        response: CollisionResponse,
        mass: f32,
    ) -> Self {
        Self {
            id,
            mesh_id,
            texture_id,
            scale,
            rotation,
            translation,
            velocity,
            acceleration,
            bounding_box,
            entity_type,
            response,
            mass,
        }
    }
    pub fn apply_gravity(&mut self, other: &mut Entity) {
        // a1 = G * m2/r^2
        let to_other: Vector3<f64> = Into::<Vector3<f32>>::into(other.translation).cast::<f64>()
            - Into::<Vector3<f32>>::into(self.translation).cast::<f64>();
        let dist = to_other.magnitude();
        let dir_b = to_other.normalize();

        self.acceleration += ((G * other.mass as f64 / (dist * dist)) * dir_b).cast::<f32>();
        other.acceleration += ((G * self.mass as f64 / (dist * dist)) * -dir_b).cast::<f32>();
    }

    pub fn tick(&mut self, dt: f32) {
        match crate::core::GLOBAL_INTEGRATOR {
            Integrator::RK4 => {
                let acceleration = Vector3::from(self.acceleration);
                let a_k1 = acceleration;
                let a_k2 = acceleration + a_k1 * dt / 2.0;
                let a_k3 = acceleration + a_k2 * dt / 2.0;
                let a_k4 = acceleration + a_k3 * dt;
                self.velocity += (a_k1 + 2.0 * a_k2 + 2.0 * a_k3 + a_k4) / 6.0 * dt;

                let velocity = Vector3::from(self.velocity);
                let v_k1 = velocity;
                let v_k2 = velocity + v_k1 * dt / 2.0;
                let v_k3 = velocity + v_k2 * dt / 2.0;
                let v_k4 = velocity + v_k3 * dt;

                self.translation += (v_k1 + 2.0 * v_k2 + 2.0 * v_k3 + v_k4) / 6.0 * dt;
            }
            Integrator::Euler => {
                todo!();
            }
        }
    }

    /// Checks for a collision between the two objects and updates velocities.
    pub fn perform_single_collision(
        &mut self,
        _other: &mut Entity,
    ) -> (Vector3<f32>, Vector3<f32>) {
        todo!();
        // TODO: Use position and velocity to determine whether to skip certain collision tests.

        // match a.bounding_box().intersects(b.bounding_box()) {
        //     None => {
        //         return (Vector3::zeros(), Vector3::zeros());
        //     }
        //     Some(c) => {
        //         let c: Vector3<f32> = c.into();
        //         let collision_dir = match c.try_normalize(1.0e-6) {
        //             Some(n) => n,
        //             None => Vector3::new(1.0, 0.0, 0.0),
        //         };

        //         match a.response() {
        //             CollisionResponse::Immovable => match b.response() {
        //                 CollisionResponse::Immovable => {
        //                     return (Vector3::zeros(), Vector3::zeros());
        //                 }
        //                 CollisionResponse::Inelastic(p_b) => {}
        //             },
        //             CollisionResponse::Inelastic(p_a) => match b.response() {
        //                 CollisionResponse::Immovable => {
        //                     return (Vector3::zeros(), Vector3::zeros());
        //                 }
        //                 CollisionResponse::Inelastic(p_b) => {
        //                     let a_v0: Vector3<f32> = a.velocity().into();
        //                     let b_v0: Vector3<f32> = b.velocity().into();
        //                     let a_m = *a.mass();
        //                     let b_m = *b.mass();

        //                     // Needs to be solved
        //                     let a_v1: Vector3<f32> = (((0.5 * (1.0 + p_a) * a_m * a_v0 * a_v0)
        //                         + (0.5 * (1.0 + p_b) * b_m * b_v0 * b_v0)
        //                         - (0.5 * b_m * b_v1 * b_v1))
        //                         / (0.5 * a_m))
        //                         .sqrt();
        //                     let b_v1: Vector3<f32> = (a_m * a_v0 + b_m * b_v0 - a_m * a_v1) / b_m;
        //                 }
        //             },
        //         }
        //     }
        // };
    }

    pub fn texture_id(&self) -> &u64 {
        &self.texture_id
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

impl Meshed<u64> for Entity {
    fn mesh_id(&self) -> &u64 {
        &self.mesh_id
    }
}

impl Unique<u64> for Entity {
    fn id(&self) -> &u64 {
        &self.id
    }
}

impl Instanced<[[f32; 4]; 4]> for Entity {
    fn instance(&self) -> [[f32; 4]; 4] {
        let mut mat =
            Matrix4::from_diagonal(&Vector4::new(self.scale.x, self.scale.y, self.scale.z, 1.0))
                * self.rotation.to_rotation_matrix().to_homogeneous();
        let mut column = mat.column_mut(3);
        column.x += self.translation.x;
        column.y += self.translation.y;
        column.z += self.translation.z;

        Into::<[[f32; 4]; 4]>::into(mat)
    }
}
