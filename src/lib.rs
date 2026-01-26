pub mod assets;
pub mod audio;
pub mod core;
pub mod entity;
pub mod input;
pub mod render;
pub mod world;

// All constants will be stored here.

use crate::render::textures::MipLevel;
use std::time::Duration;

pub const GLOBAL_INTEGRATOR: Integrator = Integrator::RK4;
pub const RENDER_DISTANCE: usize = 8;

pub const CHUNK_RESOLUTION: usize = 8;
pub const CHUNK_SIZE: f64 = 20000.0;
pub const GROUND_HEIGHT: i64 = -5;
pub const GROUND_COLOR: [f32; 3] = [0.37, 0.36, 0.26];

pub const CAMERA_SPEED: f32 = 1000.0;
pub const CAMERA_USES_PITCH: bool = true;

pub const MUTE: bool = true;

pub const MESH_CUBE2: u64 = 0;
pub const MESH_ROUNDISH: u64 = 1;
pub const MESH_FLAT16: u64 = 2;

// must be in decreasing quality
pub const MIPMAP_LEVELS: [MipLevel; 1] = [MipLevel::Square(2048)];

// metrics
pub const METRICS_INTERVAL: Duration = Duration::new(10, 0);

pub fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
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

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
