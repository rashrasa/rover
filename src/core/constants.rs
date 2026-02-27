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

// must be in decreasing quality
pub const MIPMAP_LEVELS: [MipLevel; 1] = [MipLevel::Square(2048)];
