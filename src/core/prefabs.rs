mod systems;

use std::time::Duration;

pub use systems::CollisionsSystem;

use crate::core::{
    System,
    prefabs::systems::{AudioSystem, DynamicsSystem, MetricsSystem},
};

pub const DEFAULT_SYSTEMS: fn() -> Vec<Box<dyn System>> = || {
    vec![
        Box::new(CollisionsSystem),
        Box::new(MetricsSystem::new(Duration::new(5, 0))),
        Box::new(AudioSystem::new()),
        Box::new(DynamicsSystem),
    ]
};
