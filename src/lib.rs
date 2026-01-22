pub mod assets;
pub mod audio;
pub mod core;
pub mod input;
pub mod render;

// All constants will be stored here.

use std::{f32::consts::PI, time::Duration};

use cgmath::{Matrix4, Rad, Rotation, Vector3, Vector4};

use crate::{
    core::world::HeightMap,
    render::{textures::MipLevel, vertex::Vertex},
};

pub const CHUNK_SIZE_M: usize = 64;
pub const RENDER_DISTANCE: usize = 8;
pub const GROUND_HEIGHT: i64 = -5;
pub const INITIAL_INSTANCE_CAPACITY: usize = 10;
pub const GROUND_COLOR: [f32; 3] = [0.37, 0.36, 0.26];
pub const CAMERA_SPEED: f32 = 5.0;

// must be in decreasing quality
pub const MIPMAP_LEVELS: [MipLevel; 1] = [MipLevel::Square(2048)];

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

#[rustfmt::skip]
pub const CUBE_MESH_INDICES: [u16; 36] = [
    0, 3, 2,    2, 1, 0,
    1, 2, 6,    6, 5, 1,
    5, 6, 7,    7, 4, 5,
    4, 7, 3,    3, 0, 4,
    4, 0, 1,    1, 5, 4,
    3, 7, 6,    6, 2, 3
];

pub const GROUND_MESH: fn(usize, usize) -> (Vec<Vertex>, Vec<u16>) = |w, h| {
    assert!((w >= 2 && h >= 1) || (w >= 1 && h >= 2)); // at least one triangle

    let dx = 1.0 / w as f32;
    let dz = 1.0 / h as f32;

    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<u16> = vec![];

    let mut up = true;
    let w = w + 1;
    let h = h + 1;
    for j in 0..h {
        for i in 0..w {
            let x = i as f32 * dx;
            let z = j as f32 * dz;
            vertices.push(Vertex {
                position: [x, GROUND_HEIGHT as f32, z],
                normal: [0.0, 1.0, 0.0], // TODO: Normal should be calculated with heightmap values
                tex_coords: [x, z],
            });

            if j > 0 {
                if up && i != w - 1 {
                    indices.push(((i + 1) + (j - 1) * w) as u16); // up-right
                    indices.push((i + (j - 1) * w) as u16); // up
                    indices.push((i + j * w) as u16); // this
                    up = false;
                } else if !up {
                    indices.push(((i - 1) + j * w) as u16); // left
                    indices.push((i + j * w) as u16); // this
                    indices.push((i + (j - 1) * w) as u16); // up
                    if i != w - 1 {
                        indices.push(((i + 1) + (j - 1) * w) as u16); // up-right
                        indices.push((i + (j - 1) * w) as u16); // up
                        indices.push((i + j * w) as u16); // this
                        up = false;
                    } else {
                        up = true;
                    }
                }
            }
        }
        up = true;
    }

    (vertices, indices)
};

// metrics
pub const METRICS_INTERVAL: Duration = Duration::new(10, 0);

pub fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();
}
