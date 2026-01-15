use log::info;
use nalgebra::Vector3;
use rover::{core::Entity, render::App, world::World};
use winit::event_loop::EventLoop;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Stdout)
        .init();

    // Winit wants to own app state
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut world = World::new(0);
    world.add_entity(Entity::new(
        "rover".into(),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        (Vector3::new(5.0, 5.0, 5.0), Vector3::new(-5.0, -5.0, -5.0)),
    ));
    let mut app = App::new(&event_loop, 1920, 1080, world);

    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
