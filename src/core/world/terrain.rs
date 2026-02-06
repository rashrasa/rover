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

use crate::{
    CHUNK_RESOLUTION, CHUNK_SIZE, ContiguousView, ContiguousViewMut,
    core::entity::{Dynamic, Entity, Position, Transform},
};

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
    chunk_loader: fn(f32, f32) -> Chunk,
}

/// Currently modelled as a sphere. This is the smallest unit of terrain.
///
/// The given radius isn't guaranteed to be the resulting radius,
/// to ensure the planet's mesh is uniform and visually appealing.
/// radius_adjustment specifies if the radius should be (minimally)
/// expanded or shrunk to the next best value.
///
/// chunk_loader should accept an approximate location to a chunk and return that chunk.
///
/// sampling_frequency should specify the size of each chunk in the x and z directions.
#[derive(Debug)]
pub struct TerrestrialBody {
    id: u64,
    radius: f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,

    terrain: Terrain,
}

pub struct GasBody {
    id: u64,
    radius: f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
}

#[derive(Debug)]
/// Composed of a finite number of Large Bodies (planets).
/// Each have their own hardcoded special chunk loader (may implement more customized world generation).
pub struct World {
    seed: u64,
    bodies: [TerrestrialBody; 0],
}

impl World {
    pub fn new(seed: u64) -> Self {
        let mut sun = TerrestrialBody {
            id: 0,
            radius: 1000.0,
            position: Vector3::zeros(),
            acceleration: Vector3::zeros(),
            velocity: Vector3::zeros(),
            terrain: Terrain {
                chunks_loaded: HashMap::new(),
                chunk_loader: |lat, long| Chunk {
                    latitude: lat,
                    longitude: long,
                    heights: [[0.0; CHUNK_RESOLUTION]; CHUNK_RESOLUTION],
                },
            },
        };

        Self {
            seed: seed,
            bodies: [],
        }
    }
}

impl Entity for GasBody {
    fn id(&self) -> &u64 {
        &self.id
    }
}

impl Dynamic for GasBody {
    fn velocity<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.velocity.generic_view_with_steps(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (0, 0),
        )
    }

    fn velocity_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.velocity.generic_view_with_steps_mut(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (0, 0),
        )
    }

    fn acceleration<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.acceleration.generic_view_with_steps(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (0, 0),
        )
    }

    fn acceleration_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.acceleration.generic_view_with_steps_mut(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (0, 0),
        )
    }
}

impl Position for GasBody {
    fn position<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.position.generic_view_with_steps(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (0, 0),
        )
    }

    fn position_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.position.generic_view_with_steps_mut(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (0, 0),
        )
    }
}

impl Entity for TerrestrialBody {
    fn id(&self) -> &u64 {
        &self.id
    }
}
