use std::time::{Duration, Instant};

use log::info;

use crate::core;

pub struct MetricsSystem {
    window_start: Instant,
    window: Duration,

    // cpu
    start_tick: Instant,
    window_ticking: Duration,
    n_ticks: u64,

    // gpu
    start_render: Instant,
    window_rendering: Duration,
    n_renders: u64,
}

impl MetricsSystem {
    pub fn new(window: Duration) -> Self {
        Self {
            window_start: Instant::now(),
            window,

            start_tick: Instant::now(),
            window_ticking: Duration::ZERO,
            n_ticks: 0,

            start_render: Instant::now(),
            window_rendering: Duration::ZERO,
            n_renders: 0,
        }
    }
}

impl core::System for MetricsSystem {
    fn before_start(&mut self, _args: &core::BeforeStartArgs) {
        self.window_start = Instant::now();

        self.start_tick = Instant::now();
        self.window_ticking = Duration::ZERO;
        self.n_ticks = 0;

        self.start_render = Instant::now();
        self.window_rendering = Duration::ZERO;
        self.n_renders = 0;
    }

    fn before_input(&mut self, _args: &core::BeforeInputArgs) {
        self.start_tick = Instant::now();
    }

    fn after_tick(&mut self, _args: &core::AfterTickArgs) {
        self.window_ticking += self.start_tick.elapsed();
        self.n_ticks += 1;
    }

    fn before_render(&mut self, _args: &core::BeforeRenderArgs) {
        self.start_render = Instant::now();
    }
    fn after_render(&mut self, _args: &core::AfterRenderArgs) {
        self.window_rendering += self.start_render.elapsed();
        self.n_renders += 1;

        // evaluate
        let window_time = self.window_start.elapsed();
        if window_time > self.window {
            let window_time = window_time.as_secs_f64();
            info!(
                "CPU: {:.2}ms, GPU: {:.2}ms, FPS: {:.2}",
                (self.window_ticking.as_secs_f64() / self.n_ticks as f64) * 1000.0,
                (self.window_rendering.as_secs_f64() / self.n_renders as f64) * 1000.0,
                (self.n_renders as f64 / window_time)
            );

            self.window_rendering = Duration::ZERO;
            self.window_ticking = Duration::ZERO;
            self.n_renders = 0;
            self.n_ticks = 0;
            self.window_start = Instant::now();
        }
    }
}
