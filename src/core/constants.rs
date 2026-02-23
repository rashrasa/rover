use std::time::Duration;

use crate::{Integrator, render::textures::MipLevel};

pub const G: f64 = 6.6743e-11;

pub const GLOBAL_INTEGRATOR: Integrator = Integrator::RK4;

/// Number of vertices per chunk per side (regardless of chunk size). Higher numbers increase performance demands.
pub const CHUNK_RESOLUTION: usize = 4;

/// Units of distance covered by a chunk. Lower numbers increase performance demands.
pub const CHUNK_SIZE: f32 = 16.0;

pub const CAMERA_SPEED: f32 = 20.0;
pub const CAMERA_USES_PITCH: bool = true;
pub const RENDER_DISTANCE: f32 = 16.0;

pub const MUTE: bool = false;

pub const MESH_CUBE2: u64 = 0;
pub const MESH_ROUNDISH: u64 = 1;
pub const MESH_FLAT16: u64 = 2;

// must be in decreasing quality
pub const MIPMAP_LEVELS: [MipLevel; 1] = [MipLevel::Square(2048)];

// metrics
pub const METRICS_INTERVAL: Duration = Duration::new(10, 0);
