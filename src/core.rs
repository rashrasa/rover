use std::hash::Hash;

pub mod assets;
pub mod audio;
pub mod camera;
pub mod continuous;
pub mod entity;
pub mod geometry;
pub mod lights;
pub mod world;

// Constants

pub const G: f64 = 6.6743e-11;

// Traits

pub trait Instanced<I> {
    fn instance(&self) -> I;
}

pub trait Unique<U: Hash + Eq + PartialEq> {
    fn id(&self) -> &U;
}

pub trait Meshed<U: Hash + Eq + PartialEq> {
    fn mesh_id(&self) -> &U;
}
