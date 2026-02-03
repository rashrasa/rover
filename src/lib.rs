pub mod core;
pub mod input;
pub mod render;

// All constants will be stored here.

use crate::render::textures::MipLevel;
use std::time::Duration;

pub const GLOBAL_INTEGRATOR: Integrator = Integrator::RK4;

/// Number of vertices per chunk per side (regardless of chunk size). Higher numbers increase performance demands.
pub const CHUNK_RESOLUTION: usize = 4;

/// Units of distance covered by a chunk. Lower numbers increase performance demands.
pub const CHUNK_SIZE: f32 = 16.0;

pub const CAMERA_SPEED: f32 = 20.0;
pub const CAMERA_USES_PITCH: bool = true;

pub const MUTE: bool = false;

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

type FLOAT = f32;

type Mat<const N: usize, const M: usize> = nalgebra::Matrix<
    FLOAT,
    nalgebra::Const<N>,
    nalgebra::Const<M>,
    nalgebra::ArrayStorage<FLOAT, N, M>,
>;

type View<'a, const N: usize, const M: usize> = nalgebra::Matrix<
    FLOAT,
    nalgebra::Const<N>,
    nalgebra::Const<M>,
    nalgebra::ViewStorageMut<
        'a,
        FLOAT,
        nalgebra::Const<N>,
        nalgebra::Const<M>,
        nalgebra::Const<1>,
        nalgebra::Const<1>,
    >,
>;

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
