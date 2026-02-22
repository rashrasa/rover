mod systems;

pub use systems::PhysicsSystem;

use crate::core::System;

pub const DEFAULT_SYSTEMS: fn() -> Vec<Box<dyn System>> = || vec![Box::new(PhysicsSystem {})];
