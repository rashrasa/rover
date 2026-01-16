// All constants will be stored here.

use std::time::Duration;

use crate::{core::Mesh, render::data::Vertex};

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

// Meshes:
pub const CUBE_MESH_VERTICES: [Vertex; 8] = [
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 1.0],
    },
];

#[rustfmt::skip]
pub const CUBE_MESH_INDICES: [u16; 36] = [
    0, 3, 2,    2, 1, 0,
    1, 2, 6,    6, 5, 1,
    5, 6, 7,    7, 4, 5,
    4, 7, 3,    3, 0, 4,
    4, 0, 1,    1, 5, 4,
    3, 7, 6,    6, 2, 3
];
