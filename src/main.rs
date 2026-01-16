use cgmath::{Matrix4, Vector3};
use log::info;
use rover::{
    core::{Entity, MeshType},
    render::App,
    world::World,
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Stdout)
        .init();

    // Winit wants to own app state
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut world = World::new(0);

    for i in 0..105 {
        world.add_entity(Entity::new(
            format!("rover_{}", i).into(),
            Vector3::new(0.0, 0.2, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            (
                Vector3::new(1.0, 1.0, 1.0) / 2.0,
                Vector3::new(-1.0, -1.0, -1.0) / 2.0,
            ),
            MeshType::Cube,
            Matrix4::from_translation([-5.0 + i as f32, 0.0, 0.0].into()),
        ));
    }

    let mut app = App::new(&event_loop, 1920, 1080, world);

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
