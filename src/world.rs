use std::{collections::HashMap, ops::Index};

use cgmath::{InnerSpace, Vector2, Vector3};
use rand::RngCore;

use crate::{CHUNK_SIZE_M, RENDER_DISTANCE, entity::Entity};

#[derive(Debug)]
pub struct World {
    seed: u64,
    entities: Vec<Entity>,
    chunks_loaded: HashMap<(i64, i64), HeightMap>,
    chunk_loader: fn(i64, i64, u64) -> HeightMap,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            seed: seed,
            entities: Vec::with_capacity(16),
            chunks_loaded: HashMap::with_capacity(RENDER_DISTANCE * RENDER_DISTANCE),
            chunk_loader: |_, _, _| {
                let mut rng = rand::rng();
                let mut sample = || ((rng.next_u32() as f64 / u32::MAX as f64) * 1.0) as i64;
                let mut data = [[0; CHUNK_SIZE_M]; CHUNK_SIZE_M];
                for i in 0..CHUNK_SIZE_M {
                    for j in 0..CHUNK_SIZE_M {
                        data[j][i] = sample()
                    }
                }
                HeightMap::new(data)
            },
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn iter_entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn tick(&mut self, dt: f32) {
        // // Perform physics calculations
        // self.perform_collisions();

        // // Do at the end
        // let mut translate_height = vec![];

        // for entity in self.entities.iter() {
        //     let pos = entity.position();
        //     let ground = match self.try_height((pos.x, pos.y).into()) {
        //         Ok(h) => h,
        //         Err(()) => {
        //             error!("Failed to clip entity to ground, skipping");
        //             translate_height.push(0.0);
        //             continue;
        //         }
        //     };
        //     if pos.y - ground < 0.0 {
        //         translate_height.push(pos.y - ground);
        //     } else {
        //         translate_height.push(0.0);
        //     }
        // }

        self.entities.iter_mut().for_each(|e| e.tick(dt));
    }

    pub fn height(&mut self, xz: Vector2<f32>) -> f32 {
        let x = xz.x;
        let z = xz.y;

        let lower = Vector2::new(x.floor(), z.floor());
        let higher = Vector2::new(x.floor() + 1.0, z.floor() + 1.0);

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

    /// Will not attempt to load chunk if not already loaded.
    pub fn try_height(&self, xz: Vector2<f32>) -> Result<f32, ()> {
        let x = xz.x;
        let z = xz.y;

        let lower = Vector2::new(x.floor(), z.floor());
        let higher = Vector2::new(x.floor() + 1.0, z.floor() + 1.0);

        let dist = (higher - lower).magnitude();
        let p_dist = (higher - xz).magnitude();

        let alpha_0 = p_dist / dist;

        let x_0 = lower.x as i64;
        let x_1 = higher.x as i64;
        let z_0 = lower.y as i64;
        let z_1 = higher.y as i64;

        let chunk_0 = match self.try_chunk_exact(x_0, z_0) {
            Some(h) => h,
            None => return Err(()),
        };
        let chunk_1 = match self.try_chunk_exact(x_1, z_1) {
            Some(h) => h,
            None => return Err(()),
        };

        let h_0 = chunk_0[(
            x_0.rem_euclid(CHUNK_SIZE_M as i64) as usize,
            z_0.rem_euclid(CHUNK_SIZE_M as i64) as usize,
        )] as f32;
        let h_1 = chunk_1[(
            x_1.rem_euclid(CHUNK_SIZE_M as i64) as usize,
            z_1.rem_euclid(CHUNK_SIZE_M as i64) as usize,
        )] as f32;

        Ok(h_0 * alpha_0 + h_1 * (1.0 - alpha_0))
    }

    /// Will not attempt to load chunk if not already loaded.
    pub fn try_chunk_exact(&self, x: i64, z: i64) -> Option<&HeightMap> {
        self.chunks_loaded.get(&(x, z))
    }

    pub fn request_chunk(&mut self, x: f64, z: f64) -> HeightMap {
        let x = x as i64;
        let z = z as i64;
        self.request_chunk_exact(x, z)
    }

    pub fn request_chunk_exact(&mut self, x: i64, z: i64) -> HeightMap {
        match self.chunks_loaded.get(&(x, z)) {
            Some(s) => s.clone(),
            None => {
                self.chunks_loaded
                    .insert((x, z), (self.chunk_loader)(x, z, self.seed));
                self.chunks_loaded
                    .get(&(x, z))
                    .expect("Unexpected error loading chunks")
                    .clone()
            }
        }
    }

    fn perform_collisions(&mut self) {
        // TODO: redo

        // Perform ground collisions
        // // Checks all possible collisions
        // let mut actions: HashMap<u64, Vec<Box<dyn Fn(&mut Entity) -> ()>>> =
        //     HashMap::with_capacity(self.entities.len());
        // for a in self.entities.iter() {
        //     let a_id = a.id();
        //     for b in self.entities.iter() {
        //         if a as *const Entity != b as *const Entity {
        //             let a_actions = match actions.get_mut(a_id) {
        //                 Some(a) => a,
        //                 None => {
        //                     actions.insert(*a_id, vec![]);
        //                     actions.get_mut(a_id).unwrap()
        //                 }
        //             };
        //             let top_intersection = (a.position().y + a.bounding_box().0.y)
        //                 - (b.position().y + b.bounding_box().1.y);

        //             a_actions.push(Box::new(move |a: &mut Entity| {
        //                 a.translate(Vector3::new(0.0, top_intersection.clone(), 0.0))
        //             }));
        //         }
        //     }
        // }
        // let mut entities = HashMap::with_capacity(self.entities.len());

        // for entity in self.entities.iter_mut() {
        //     entities.insert(entity.id(), entity);
        // }
        // for (entity_id, actions) in actions {
        //     let entity = entities.get_mut(&entity_id).unwrap();
        //     for action in actions {
        //         action(entity);
        //     }
        // }
    }
}

#[derive(Clone, Debug)]
pub struct HeightMap {
    map: [[i64; CHUNK_SIZE_M]; CHUNK_SIZE_M],
}

impl HeightMap {
    fn new(map: [[i64; CHUNK_SIZE_M]; CHUNK_SIZE_M]) -> Self {
        Self { map }
    }
    fn flat(height: i64) -> Self {
        Self {
            map: [[height; CHUNK_SIZE_M]; CHUNK_SIZE_M],
        }
    }
}

impl Index<(usize, usize)> for HeightMap {
    type Output = i64;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.map[index.1][index.0]
    }
}
