use cgmath::{Matrix4, Vector3};
use log::info;
use rand::RngCore;
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
    let mut rng = rand::rng();
    let mut next_float = || rng.next_u32() as f32 / u32::MAX as f32;
    let ground_vertices = GROUND_MESH_VERTICES.map(|mut v| {
        v.color = [next_float(), next_float(), next_float()];
        v
    });

    app.add_meshes(
        [
            (
                "Cube",
                CUBE_MESH_VERTICES.as_slice(),
                CUBE_MESH_INDICES.as_slice(),
            ),
            (
                "Flat16",
                ground_vertices.as_slice(),
                GROUND_MESH_INDICES.as_slice(),
            ),
        ]
        .iter(),
    );

    for i in -15..15 {
        for j in -15..15 {
            for k in -15..15 {
                app.add_entity(Entity::new(
                    &format!("rover_{}_{}_{}", i, j, k),
                    "Cube",
                    Vector3::new(i as f32, j as f32, k as f32),
                    Vector3::new(0.0, 0.0, 0.0),
                    (
                        Vector3::new(1.0, 1.0, 1.0) / 2.0,
                        Vector3::new(-1.0, -1.0, -1.0) / 2.0,
                    ),
                    Matrix4::from_translation([i as f32, j as f32, k as f32].into()),
                ));
            }
        }
    }

    for x in -100..100 {
        for z in -100..100 {
            app.load_chunk(2 * x, 2 * z);
        }
    }

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
