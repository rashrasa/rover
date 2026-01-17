use cgmath::{Matrix4, Vector3};
use log::info;
use rover::{
    CUBE_MESH_INDICES, CUBE_MESH_VERTICES, GROUND_MESH_INDICES, GROUND_MESH_VERTICES,
    core::{entity::Entity, world::World},
    render::{App, mesh::Mesh},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Stdout)
        .init();

    // Winit wants to own app state
    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut app = App::new(&event_loop, 1920, 1080, 0);

    app.add_meshes(
        [
            (
                "Cube",
                CUBE_MESH_VERTICES.as_slice(),
                CUBE_MESH_INDICES.as_slice(),
            ),
            (
                "Flat16",
                GROUND_MESH_VERTICES.as_slice(),
                GROUND_MESH_INDICES.as_slice(),
            ),
        ]
        .iter(),
    );

    for i in 0..105 {
        app.add_entity(Entity::new(
            &format!("rover_{}", i),
            "Cube",
            Vector3::new(0.0, 5.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            (
                Vector3::new(1.0, 1.0, 1.0) / 2.0,
                Vector3::new(-1.0, -1.0, -1.0) / 2.0,
            ),
            Matrix4::from_translation([-5.0 + i as f32, 0.0, 0.0].into()),
        ));
    }

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
