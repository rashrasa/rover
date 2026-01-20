use cgmath::{Matrix4, Vector3};
use image::imageops::FilterType;
use log::info;
use rand::RngCore;
use rover::{
    CHUNK_SIZE_M, CUBE_MESH_INDICES, CUBE_MESH_VERTICES, GROUND_MESH,
    core::{entity::Entity, world::World},
    render::{App, mesh::Mesh, textures::ResizeStrategy},
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
    let (g_v, g_i) = GROUND_MESH(CHUNK_SIZE_M, CHUNK_SIZE_M);

    app.add_meshes(
        [
            (
                "Cube",
                CUBE_MESH_VERTICES.as_slice(),
                CUBE_MESH_INDICES.as_slice(),
            ),
            ("Flat16", g_v.as_slice(), g_i.as_slice()),
        ]
        .iter(),
    );

    app.add_texture(
        "test".into(),
        image::load_from_memory(include_bytes!("../assets/white-marble-2048x2048.png")).unwrap(),
        ResizeStrategy::Stretch(FilterType::Gaussian),
    );

    for i in -1..2 {
        for j in -1..2 {
            for k in -1..2 {
                app.add_entity(Entity::new(
                    &format!("rover_{}_{}_{}", i, j, k),
                    "Cube",
                    Vector3::new(i as f32, j as f32, k as f32),
                    Vector3::new(0.0, 0.0, 0.0),
                    (
                        Vector3::new(1.0, 1.0, 1.0) / 2.0,
                        Vector3::new(-1.0, -1.0, -1.0) / 2.0,
                    ),
                    Matrix4::from_translation([0.0, 0.0, 0.0].into()),
                ));
            }
        }
    }

    for x in -5..5 {
        for z in -5..5 {
            app.load_chunk(x, z);
        }
    }

    info!("Starting");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
