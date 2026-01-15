use std::{
    collections::{HashMap, hash_map::Values},
    ops::Index,
};

use log::debug;
use nalgebra::{Vector2, Vector3};

use crate::{CHUNK_SIZE_M, GROUND_HEIGHT, RENDER_DISTANCE, core::Entity};

#[derive(Debug)]
pub struct World {
    seed: u64,
    entities: HashMap<String, Entity>,
    chunks_loaded: HashMap<(i64, i64), HeightMap<{ CHUNK_SIZE_M }>>,
    chunk_loader: fn(i64, i64) -> HeightMap<{ CHUNK_SIZE_M }>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            seed: seed,
            entities: HashMap::with_capacity(16),
            chunks_loaded: HashMap::with_capacity(RENDER_DISTANCE * RENDER_DISTANCE),
            chunk_loader: |_, _| HeightMap::flat(GROUND_HEIGHT),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id().into(), entity);
    }

    pub fn iter_entities(&self) -> Values<'_, String, Entity> {
        self.entities.values()
    }

    pub fn tick(&mut self, dt: f32) {
        // Perform physics calculations
        // self.perform_collisions();

        // // Do at the end
        // let mut old_states = Vec::with_capacity(self.entities.len());
        // for (id, entity) in self.entities.iter() {
        //     old_states.push((*entity).clone());
        // }
        for entity in self.entities.iter_mut() {
            // let pos = entity.position();
            // let ground_y = self.height((pos / CHUNK_SIZE_M as f32).xz());
            // entity.translate(Vector3::new(0.0, pos.y - ground_y, 0.0));

            entity.1.tick(dt);
        }
    }

    pub fn height(&mut self, xz: Vector2<f32>) -> f32 {
        let x = xz.x;
        let z = xz.y;

        let lower = Vector2::new(x.floor(), z.floor());
        let higher = Vector2::new(x.ceil(), z.ceil());

        let dist = (higher - lower).magnitude();
        let p_dist = (higher - xz).magnitude();

        let alpha_0 = p_dist / dist;

        let x_0 = lower.x as i64;
        let x_1 = higher.x as i64;
        let z_0 = lower.y as i64;
        let z_1 = higher.y as i64;

        let chunk_0 = self.request_chunk_exact(x_0, z_0);
        let chunk_1 = self.request_chunk_exact(x_1, z_1);

        let h_0 = chunk_0[(
            x_0.rem_euclid(CHUNK_SIZE_M as i64) as usize,
            z_0.rem_euclid(CHUNK_SIZE_M as i64) as usize,
        )] as f32;
        let h_1 = chunk_1[(
            x_1.rem_euclid(CHUNK_SIZE_M as i64) as usize,
            z_1.rem_euclid(CHUNK_SIZE_M as i64) as usize,
        )] as f32;
        h_0 * alpha_0 + h_1 * (1.0 - alpha_0)
    }

    pub fn request_chunk(&mut self, x: f64, z: f64) -> HeightMap<{ CHUNK_SIZE_M }> {
        let x = x as i64;
        let z = z as i64;
        self.request_chunk_exact(x, z)
    }

    pub fn request_chunk_exact(&mut self, x: i64, z: i64) -> HeightMap<{ CHUNK_SIZE_M }> {
        match self.chunks_loaded.get(&(x, z)) {
            Some(s) => s.clone(),
            None => {
                self.chunks_loaded.insert((x, z), (self.chunk_loader)(x, z));
                self.chunks_loaded
                    .get(&(x, z))
                    .expect("Unexpected error loading chunks")
                    .clone()
            }
        }
    }

    fn perform_collisions(&mut self) {
        // Perform ground collisions

        // Checks all possible collisions
        let mut actions: HashMap<String, Vec<Box<dyn Fn(&mut Entity) -> ()>>> =
            HashMap::with_capacity(self.entities.len());
        for (a_id, a) in self.entities.iter() {
            for (b_id, b) in self.entities.iter() {
                if a as *const Entity != b as *const Entity {
                    let a_actions = match actions.get_mut(a_id) {
                        Some(a) => a,
                        None => {
                            actions.insert(a_id.clone(), vec![]);
                            actions.get_mut(a_id).unwrap()
                        }
                    };
                    let top_intersection = (a.position().y + a.bounding_box().0.y)
                        - (b.position().y + b.bounding_box().1.y);

                    a_actions.push(Box::new(move |a: &mut Entity| {
                        a.translate(Vector3::new(0.0, top_intersection.clone(), 0.0))
                    }));
                }
            }
        }

        for (entity_id, actions) in actions {
            let entity = self.entities.get_mut(&entity_id).unwrap();
            for action in actions {
                action(entity);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct HeightMap<const LENGTH: usize> {
    map: [[u64; LENGTH]; LENGTH],
}

impl<const LENGTH: usize> HeightMap<LENGTH> {
    fn flat(height: u64) -> Self {
        Self {
            map: [[height; LENGTH]; LENGTH],
        }
    }
}

impl<const LENGTH: usize> Index<(usize, usize)> for HeightMap<{ LENGTH }> {
    type Output = u64;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.map[index.1][index.0]
    }
}
