use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector3};
use wgpu::BindGroup;

use crate::{ContiguousView, ContiguousViewMut, Integrator, core::camera::Projection};

// Each entity module is simply just a unique composition of these traits below.
pub mod object;
pub mod player;

/// Entities are unique objects in the world.
/// Any traits functions with &self or &mut self as a parameter will (likely) need to be unique.
pub trait Entity {
    fn id(&self) -> &u64;
}

/// Entities that have a mutable transform component.
pub trait Transform {
    fn transform<'a>(&'a self) -> ContiguousView<'a, 4, 4>;
    fn transform_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 4, 4>;
}

/// Entities that have a mutable position component.
pub trait Position {
    fn position<'a>(&'a self) -> ContiguousView<'a, 3, 1>;
    fn position_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1>;
}

/// Entities that have a mutable acceleration and velocity component.
pub trait Dynamic: Position {
    fn velocity<'a>(&'a self) -> ContiguousView<'a, 3, 1>;
    fn velocity_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1>;

    fn acceleration<'a>(&'a self) -> ContiguousView<'a, 3, 1>;
    fn acceleration_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1>;
}

/// Entities that have a mass component.
pub trait Mass {
    fn mass(&self) -> &f32;
}

/// Entities that can be used as a view.
pub trait View {
    fn set_projection(&mut self, projection: Projection);
    fn view_proj<'a>(&'a self) -> ContiguousView<'a, 4, 4>;
}

/// Entities that can be rendered.
pub trait RenderUniform: Transform {
    fn texture_id(&self) -> &u64;
    fn mesh_id(&self) -> &u64;
    fn bind_group(&self) -> &BindGroup;
}

/// Entities that can be rendered and should be instanced.
pub trait RenderInstanced<I>
where
    I: Pod + Zeroable + Clone + Copy + Debug,
{
    fn texture_id(&self) -> &u64;
    fn mesh_id(&self) -> &u64;
    fn instance(&self) -> &I;
}

/// Entities that can collide. Bounding box should be in world space.
///
/// Entity trait bound is necessary here to avoid checking collisions with same element
pub trait Collide: Dynamic + Mass + Entity {
    fn bounding_box(&self) -> &BoundingBox;
    fn response(&self) -> &CollisionResponse;
}

pub trait Illuminate: Position {
    fn luminance(&self) -> &f64;
}

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
        let min_y_size = self.y_size.min(other.y_size);
        let min_z_size = self.z_size.min(other.z_size);

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
pub fn perform_collisions(objects: &Vec<impl Collide>) {
    // Calculate each collision and add them up for each object.
    let mut net_collisions: Vec<Vector3<f32>> = Vec::new();
    for (i, a) in objects.iter().enumerate() {
        net_collisions.push(Vector3::zeros());
        for (j, b) in objects.iter().enumerate() {
            if j != i {
                let (v_a, _) = calculate_single_collision(a, b);
                net_collisions[i] += v_a;
            }
        }
    }

    for o in objects.iter() {}
}

/// Checks for a collision between the two objects and returns new velocities.
pub fn calculate_single_collision(
    a: &impl Collide,
    b: &impl Collide,
) -> (Vector3<f32>, Vector3<f32>) {
    todo!();

    // TODO: Use position and velocity to determine whether to skip certain collision tests.

    // match a.bounding_box().intersects(b.bounding_box()) {
    //     None => {
    //         return;
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
    //                     return;
    //                 }
    //                 CollisionResponse::Inelastic(p_b) => {}
    //             },
    //             CollisionResponse::Inelastic(p_a) => match b.response() {
    //                 CollisionResponse::Immovable => {
    //                     return;
    //                 }
    //                 CollisionResponse::Inelastic(p_b) => {
    //                     let a_v0 = *a.velocity();
    //                     let b_v0 = *b.velocity();
    //                     let a_m = *a.mass();
    //                     let b_m = *b.mass();

    //                     // Needs to be solved
    //                     let a_v1 = (((0.5 * (1.0 + p_a) * a_m * a_v0 * a_v0)
    //                         + (0.5 * (1.0 + p_b) * b_m * b_v0 * b_v0)
    //                         - (0.5 * b_m * b_v1 * b_v1))
    //                         / (0.5 * a_m))
    //                         .sqrt();
    //                     let b_v1 = (a_m * a_v0 + b_m * b_v0 - a_m * a_v1) / b_m;
    //                 }
    //             },
    //         }
    //     }
    // };
}

pub fn tick(a: &mut impl Dynamic, dt: f32) {
    match crate::GLOBAL_INTEGRATOR {
        Integrator::RK4 => {
            let acceleration = Vector3::from(a.acceleration());
            let k1 = acceleration;
            let k2 = acceleration + k1 * dt / 2.0;
            let k3 = acceleration + k2 * dt / 2.0;
            let k4 = acceleration + k3 * dt;
            let mut velocity = a.velocity_mut();
            velocity += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;

            let velocity = Vector3::from(a.velocity());
            let k1 = velocity;
            let k2 = velocity + k1 * dt / 2.0;
            let k3 = velocity + k2 * dt / 2.0;
            let k4 = velocity + k3 * dt;

            let translation = (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;
            let mut transform = a.position_mut();
            transform += translation;
        }
        Integrator::Euler => {
            todo!();
        }
    }
}

impl Position for Matrix4<f32> {
    fn position<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.generic_view_with_steps((0, 3), (nalgebra::Const::<3>, nalgebra::Const::<1>), (1, 1))
    }

    fn position_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.generic_view_with_steps_mut(
            (0, 3),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (1, 1),
        )
    }
}
