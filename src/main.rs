use log::info;
use rover::{render::App, world::World};
use winit::event_loop::EventLoop;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();

    // Winit wants to own app state
    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut app = App::new(&event_loop, 1920, 1080, World::new(0));

    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
