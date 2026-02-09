use std::collections::{HashMap, hash_map::Entry};

use bytemuck::{Pod, Zeroable};
use nalgebra::Vector3;

use crate::{CHUNK_RESOLUTION, core::entity::Position};

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
struct Chunk {
    latitude: f32,
    longitude: f32,
    heights: [[f32; CHUNK_RESOLUTION]; CHUNK_RESOLUTION],
}

/// Belongs to a LargeBody.
#[derive(Debug)]
struct Terrain {
    chunks_loaded: HashMap<(i64, i64), Chunk>, // TODO: Implement as quadtree
    chunk_loader: fn(i64, i64) -> Chunk,
}

/// In this world, the sun and moon orbit this infinite world
pub struct World {
    terrain: Terrain,
    time: f32,
    sun: Sun,
    moon: Moon,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            terrain: Terrain {
                chunks_loaded: HashMap::new(),
                chunk_loader: |x, z| Chunk {
                    latitude: x as f32,
                    longitude: z as f32,
                    heights: [[0.0; CHUNK_RESOLUTION]; CHUNK_RESOLUTION],
                },
            },
            time: 0.0,
            sun: Sun {
                radius: 6.963e8,
                distance: 150.0e9,
                luminance: 3.75e28,
            },
            moon: Moon {
                radius: 1.738e6,
                distance: 3.844e8,
            },
        }
    }

    /// Blocks until all chunks load
    pub fn load(&mut self, at: (f32, f32), radius: f32) {
        for x in ((at.0 - radius).floor() as i64)..((at.0 + radius).ceil() as i64) {
            for z in ((at.1 - radius).floor() as i64)..((at.1 + radius).ceil() as i64) {
                if let Entry::Vacant(not_loaded) = self.terrain.chunks_loaded.entry((x, z)) {
                    not_loaded.insert((self.terrain.chunk_loader)(x, z));
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct Sun {
    radius: f32,
    distance: f32,
    luminance: f32,
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct Moon {
    radius: f32,
    distance: f32,
}
