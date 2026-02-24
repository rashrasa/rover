use std::time::{Duration, Instant};

use crate::core;

pub struct MetricsSystem {
    last_update: Instant,
    start: Instant,
    n_renders: u64,
    t_ticking: Duration,
    n_ticks: u64,
}

impl MetricsSystem {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            start: Instant::now(),
            n_renders: 0,
            t_ticking: Duration::ZERO,
            n_ticks: 0,
        }
    }
}

impl core::System for MetricsSystem {
    fn before_start(&mut self, _args: &core::BeforeStartArgs) {
        self.last_update = Instant::now();
        self.n_renders = 0;
        self.n_ticks = 0;
        self.start = Instant::now();
        self.t_ticking = Duration::ZERO;
    }
}
