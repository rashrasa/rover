mod systems;

use std::time::Duration;

pub use systems::PhysicsSystem;

use crate::core::{
    System,
    prefabs::systems::{AudioSystem, MetricsSystem},
};

pub const DEFAULT_SYSTEMS: fn() -> Vec<Box<dyn System>> = || {
    vec![
        Box::new(PhysicsSystem),
        Box::new(MetricsSystem::new(Duration::new(5, 0))),
        Box::new(AudioSystem::new()),
    ]
};
