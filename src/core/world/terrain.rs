use std::{
    collections::HashMap,
    ops::{Range, RangeBounds},
};

use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Matrix4, Vector2};
use nalgebra::Vector3;
use rand::RngCore;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{CHUNK_RESOLUTION, CHUNK_SIZE};

pub const TERRAIN_MESH: u64 = 2;

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
    chunk_loader: fn(i64, i64, u64) -> Chunk,
}

/// Currently modelled as a sphere. This is the smallest unit of terrain.
#[derive(Debug)]
pub struct LargeBody {
    id: u64,
    radius: f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,

    terrain: Terrain,
    buffer: Buffer,
}

pub enum RadiusAdjustmentStrategy {
    Expand,
    Shrink,
}

impl LargeBody {
    /// Provided radius isn't guaranteed to be the resulting radius,
    /// to ensure the planet's mesh is uniform and visually appealing.
    /// radius_adjustment specifies if the radius should be (minimally)
    /// expanded or shrunk to the next best value.
    ///
    /// chunk_loader should accept an approximate location to a chunk and return that chunk.
    ///
    /// sampling_frequency should specify the size of each chunk in the x and z directions.
    pub fn new(
        id: u64,
        radius: f32,
        radius_adjustment: RadiusAdjustmentStrategy,
        chunk_loader: fn() -> Chunk,
        sampling_frequency: (f32, f32),
        device: &Device,
        buffer: &Buffer,
    ) -> Self {
        todo!()
    }
}
