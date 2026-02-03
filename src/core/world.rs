use std::{collections::HashMap, ops::Index};

use cgmath::{InnerSpace, Vector2, Vector3};
use rand::RngCore;

use crate::{
    CHUNK_RESOLUTION, Integrator,
    core::{
        entity::{self, Collide, Dynamic, Mass, Transform, player::Player},
        world::terrain::LargeBody,
    },
};

pub mod terrain;

#[derive(Debug)]
/// Composed of a finite number of Large Bodies (planets).
/// Each have their own hardcoded special chunk loader (may implement more customized world generation).
pub struct World {
    seed: u64,
    bodies: [LargeBody; 0],
}

impl World {
    pub fn new(seed: u64) -> Self {
        //todo!()

        // let mut sun = LargeBody {};
        // let bodies = [];

        Self {
            seed: seed,
            bodies: [],
        }
    }
}
