use std::collections::HashMap;

use winit::{dpi::PhysicalPosition, keyboard::KeyCode};

pub mod object;
pub mod player;
pub mod terrain;

/// Entities that get advanced by the world tick.
pub trait Tick {
    fn tick(&mut self, dt: f32);
}

/// Entities that have a mutable position component.
pub trait Position {
    fn position(&self) -> &[f32; 3];
    fn set_position(&mut self, p: [f32; 3]);
}

pub trait Mass {
    fn mass(&self) -> &f32;
}

/// Entities that have a mutable acceleration and velocity component.
pub trait Dynamic: Position + Tick {
    fn velocity(&self) -> &[f32; 3];
    fn set_velocity(&mut self, v: [f32; 3]);

    fn acceleration(&self) -> &[f32; 3];
    fn set_acceleration(&mut self, a: [f32; 3]);
}

/// Entities that can be used as a view.
pub trait View {
    fn view_proj(&self) -> &[[f32; 4]; 4];
}

/// Entities that can be rendered.
pub trait Render {
    fn transform(&self) -> &[[f32; 4]; 4];
    fn texture_id(&self) -> u64;
    fn mesh_id(&self) -> u64;
}

/// Entities that can collide. Bounding box should be in world space.
pub trait Collide: Dynamic + Mass {
    fn bounding_box(&self) -> &BoundingBox;
    fn response(&self) -> &CollisionResponse;
}

pub trait KeyControl {
    fn handle_input(&mut self, state: &HashMap<KeyCode, bool>);
}

/// Elastic collisions have CollisionResponse::Inelastic(1.0).
/// Inelastic takes any value. Values exceeding 1.0 will result in
/// energy magically being added to the system. Values below 0.0 will
/// be clamped to 0.0.
pub enum CollisionResponse {
    Immovable,
    Inelastic(f32),
}

pub struct BoundingBox {
    x: f32,
    y: f32,
    z: f32,

    x_size: f32,
    y_size: f32,
    z_size: f32,
}
