use std::{
    collections::HashMap,
    ops::{Range, RangeBounds},
};

use cgmath::{InnerSpace, Matrix4, Vector2};
use rand::RngCore;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{CHUNK_RESOLUTION, GROUND_HEIGHT, RENDER_DISTANCE};

pub const TERRAIN_MESH: u64 = 2;

// Static instance of flat mesh
pub struct Chunk {
    pub model: [[f32; 4]; 4],
}

pub struct Terrain {
    // seed: u64,
    buffer: Buffer,

    // chunks_loaded: HashMap<(i64, i64), Chunk>, // TODO: Implement as quadtree
    // chunk_loader: fn(i64, i64, u64) -> Chunk,
    instances: Vec<[f32; 4]>,
}

impl Terrain {
    pub fn new(
        // seed: u64,
        device: &Device,
        buffer: &Buffer,
        x: Range<i64>,
        z: Range<i64>,
    ) -> Self {
        let mut instances = vec![];

        // let chunk_loader = |x, z, _| {
        //         let mut rng = rand::rng();
        //         let mut heights = [[0.0; CHUNK_RESOLUTION]; CHUNK_RESOLUTION];
        //         for i in 0..CHUNK_RESOLUTION {
        //             for j in 0..CHUNK_RESOLUTION {
        //                 heights[j][i] = GROUND_HEIGHT as f32;
        //             }
        //         }

        //     };

        for i in x.start..x.end {
            for k in z.start..z.end {
                instances.push([1.0, 0.0, 0.0, 0.0]);
                instances.push([0.0, 1.0, 0.0, 0.0]);
                instances.push([0.0, 0.0, 1.0, 0.0]);
                instances.push([i as f32, 0.0, k as f32, 1.0]);
            }
        }
        Self {
            // seed,
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Terrain Buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            }),
            // chunks_loaded: HashMap::with_capacity(RENDER_DISTANCE * RENDER_DISTANCE),
            instances,
        }
    }

    // pub fn height(&mut self, xz: Vector2<f32>) -> f32 {
    //     let x = xz.x;
    //     let z = xz.y;

    //     let lower = Vector2::new(x.floor(), z.floor());
    //     let higher = Vector2::new(x.floor() + 1.0, z.floor() + 1.0);

    //     let dist = (higher - lower).magnitude();
    //     let p_dist = (higher - xz).magnitude();

    //     let alpha_0 = p_dist / dist;

    //     let x_0 = lower.x as i64;
    //     let x_1 = higher.x as i64;
    //     let z_0 = lower.y as i64;
    //     let z_1 = higher.y as i64;

    //     let chunk_0 = self.request_chunk_exact(x_0, z_0);
    //     let chunk_1 = self.request_chunk_exact(x_1, z_1);

    //     let h_0 = chunk_0[(
    //         x_0.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //         z_0.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //     )] as f32;
    //     let h_1 = chunk_1[(
    //         x_1.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //         z_1.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //     )] as f32;
    //     h_0 * alpha_0 + h_1 * (1.0 - alpha_0)
    // }

    // /// Will not attempt to load chunk if not already loaded.
    // pub fn try_height(&self, xz: Vector2<f32>) -> Result<f32, ()> {
    //     let x = xz.x;
    //     let z = xz.y;

    //     let lower = Vector2::new(x.floor(), z.floor());
    //     let higher = Vector2::new(x.floor() + 1.0, z.floor() + 1.0);

    //     let dist = (higher - lower).magnitude();
    //     let p_dist = (higher - xz).magnitude();

    //     let alpha_0 = p_dist / dist;

    //     let x_0 = lower.x as i64;
    //     let x_1 = higher.x as i64;
    //     let z_0 = lower.y as i64;
    //     let z_1 = higher.y as i64;

    //     let chunk_0 = match self.try_chunk_exact(x_0, z_0) {
    //         Some(h) => h,
    //         None => return Err(()),
    //     };
    //     let chunk_1 = match self.try_chunk_exact(x_1, z_1) {
    //         Some(h) => h,
    //         None => return Err(()),
    //     };

    //     let h_0 = chunk_0[(
    //         x_0.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //         z_0.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //     )] as f32;
    //     let h_1 = chunk_1[(
    //         x_1.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //         z_1.rem_euclid(CHUNK_RESOLUTION as i64) as usize,
    //     )] as f32;

    //     Ok(h_0 * alpha_0 + h_1 * (1.0 - alpha_0))
    // }

    // /// Will not attempt to load chunk if not already loaded.
    // pub fn try_chunk_exact(&self, x: i64, z: i64) -> Option<&HeightMap> {
    //     self.chunks_loaded.get(&(x, z))
    // }

    // pub fn request_chunk(&mut self, x: f64, z: f64) -> HeightMap {
    //     let x = x as i64;
    //     let z = z as i64;
    //     self.request_chunk_exact(x, z)
    // }

    // pub fn request_chunk_exact(&mut self, x: i64, z: i64) -> HeightMap {
    //     match self.chunks_loaded.get(&(x, z)) {
    //         Some(s) => s.clone(),
    //         None => {
    //             self.chunks_loaded
    //                 .insert((x, z), (self.chunk_loader)(x, z, self.seed));
    //             self.chunks_loaded
    //                 .get(&(x, z))
    //                 .expect("Unexpected error loading chunks")
    //                 .clone()
    //         }
    //     }
    // }
}
