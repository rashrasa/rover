pub mod assets;
pub mod audio;
pub mod core;
pub mod input;
pub mod render;

// All constants will be stored here.

use crate::render::textures::MipLevel;
use std::time::Duration;

pub const GLOBAL_INTEGRATOR: Integrator = Integrator::RK4;
pub const RENDER_DISTANCE: usize = 8;

pub const CHUNK_SIZE_M: usize = 64;
pub const GROUND_HEIGHT: i64 = -5;
pub const GROUND_COLOR: [f32; 3] = [0.37, 0.36, 0.26];

pub const CAMERA_SPEED: f32 = 1000.0;
pub const CAMERA_USES_PITCH: bool = true;

pub const MUTE: bool = true;

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
