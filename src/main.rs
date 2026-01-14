use log::info;
use rover::{render::Renderer, world::World};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();

    // Winit wants to own app state
    Renderer::start(World::new(0), 1280, 720);

    info!("Starting shutdown");
}
