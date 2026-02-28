/*  agate_engine

   Notes:
       - Tick and Render tied to WindowEvent::RedrawRequested event from the main window
       - Entities and World live in app::ActiveState
       - lifecycle::System's act on state, input, camera through lifecycle hooks
    Issues:
       - Inter-System communication is not currently possible (merging systems is necessary)
*/

pub mod core;
pub mod render;

pub fn init_logging(level: log::LevelFilter) {
    env_logger::builder()
        .filter_level(level)
        .target(env_logger::Target::Stdout)
        .init();
}

#[derive(Clone, Debug)]
pub enum Integrator {
    Euler,
    RK4,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: nalgebra::Matrix4<f32> = nalgebra::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
