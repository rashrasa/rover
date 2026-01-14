use std::{collections::HashMap, ops::Index};

use nalgebra::{Vector2, Vector3};

use crate::{CHUNK_SIZE_M, RENDER_DISTANCE, core::Entity};

pub struct World {
    seed: u64,
    entities: Vec<Entity>,
    chunks_loaded: HashMap<(i64, i64), HeightMap<{ CHUNK_SIZE_M }>>,
    chunk_loader: fn(i64, i64) -> HeightMap<{ CHUNK_SIZE_M }>,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            seed: seed,
            entities: vec![],
            chunks_loaded: HashMap::with_capacity(RENDER_DISTANCE * RENDER_DISTANCE),
            chunk_loader: |_, _| HeightMap::flat(32),
        }
    }

    pub fn tick(&mut self, dt: f64) {
        // Perform physics calculations
        self.perform_collisions();

        // Do at the end
        let mut old_states = Vec::with_capacity(self.entities.len());
        for entity in self.entities.iter() {
            old_states.push((*entity).clone());
        }
        for entity in old_states.iter_mut() {
            let pos = entity.position();
            let ground_y = self.height((pos / CHUNK_SIZE_M as f64).xz());

            entity.translate(Vector3::new(0.0, pos.y - ground_y, 0.0));

            entity.tick(dt);
        }
    }

    pub fn height(&mut self, xz: Vector2<f64>) -> f64 {
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
        )] as f64;
        let h_1 = chunk_1[(
            x_1.rem_euclid(CHUNK_SIZE_M as i64) as usize,
            z_1.rem_euclid(CHUNK_SIZE_M as i64) as usize,
        )] as f64;
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
        let mut a_actions: Vec<Box<dyn Fn(&mut Entity) -> ()>> = vec![];
        for _ in self.entities.iter().map(|a| {
            for b in self.entities.iter() {
                if a as *const Entity != b as *const Entity {
                    let a_action = {
                        let top_intersection = (a.position().y + a.bounding_box().0.y)
                            - (b.position().y + b.bounding_box().1.y);

                        a_actions.push(Box::new(move |a: &mut Entity| {
                            a.translate(Vector3::new(0.0, top_intersection.clone(), 0.0))
                        }));
                        break; // Performs only the first collision
                    };
                } else {
                    a_actions.push(Box::new(|a: &mut Entity| {}));
                    break;
                }
            }
        }) {}

        for i in 0..self.entities.len() {
            a_actions[i](&mut self.entities[0]);
        }
    }
}

#[derive(Clone)]
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
