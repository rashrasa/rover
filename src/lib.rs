// All constants will be stored here.

use std::time::Duration;

pub const CHUNK_SIZE_M: usize = 16;
pub const RENDER_DISTANCE: usize = 8;
pub const GROUND_HEIGHT: u64 = 32;
pub const METRICS_INTERVAL: Duration = Duration::new(1, 0);

pub mod assets;
pub mod audio;
pub mod core;
pub mod input;
pub mod render;
pub mod rover;
pub mod world;

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
