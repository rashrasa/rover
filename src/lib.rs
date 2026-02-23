pub mod core;
pub mod render;

// Configuration constants.

use crate::render::textures::MipLevel;
use std::time::Duration;

pub fn init_logging(level: log::LevelFilter) {
    env_logger::builder()
        .filter_level(level)
        .target(env_logger::Target::Stdout)
        .init();
}

#[derive(Clone, Debug)]
pub enum Integrator {
    Euler,
    RK4,
}

pub struct IDBank {
    next: u64,
}
impl IDBank {
    pub fn new() -> Self {
        Self { next: 0 }
    }
    pub fn next(&mut self) -> u64 {
        self.next += 1;
        self.next - 1
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: nalgebra::Matrix4<f32> = nalgebra::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
