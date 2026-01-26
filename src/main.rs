use std::{f32::consts::PI, fs::File, thread};

use cgmath::{Matrix4, Rad, Vector3, Vector4};
use image::imageops::FilterType;
use log::{debug, info};
use rodio::Decoder;
use rover::{
    CHUNK_RESOLUTION, CHUNK_SIZE, GROUND_HEIGHT, IDBank, MESH_CUBE2, MESH_FLAT16, MESH_ROUNDISH,
    core::geometry::{EdgeJoin, Face, Mesh, Shape3},
    entity::player::Entity,
    render::{App, Event, textures::ResizeStrategy, vertex::Vertex},
};
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowId,
};

fn main() {
    rover::init_logging();
    // Winit wants to own app state
    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut app = App::new(&event_loop, 1920, 1080, 0);

    let ground = Face::from_function(
        [0.0, 1.0, 0.0].into(),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (
            CHUNK_RESOLUTION as f32 / CHUNK_SIZE as f32,
            CHUNK_RESOLUTION as f32 / CHUNK_SIZE as f32,
        ),
        |x, z| GROUND_HEIGHT as f32,
    )
    .unwrap();

    let _isq3: f32 = 1.0 / 3.0_f32.sqrt();

    let cube2_mesh = Shape3::new(
        vec![
            Face::from_function(
                [0.0, 1.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (2.0, 2.0),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [0.0, -1.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (2.0, 2.0),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (2.0, 2.0),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [-1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (2.0, 2.0),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, 1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (2.0, 2.0),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, -1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (2.0, 2.0),
                |_, _| 0.5,
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap();

    let roundish_mesh = Shape3::new(
        vec![
            Face::from_function(
                [0.0, 1.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (8.0, 8.0),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [0.0, -1.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (8.0, 8.0),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (8.0, 8.0),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [-1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (8.0, 8.0),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, 1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (8.0, 8.0),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, -1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (8.0, 8.0),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap();

    app.add_meshes(
        [
            (&MESH_CUBE2, cube2_mesh.vertices(), cube2_mesh.indices()),
            (
                &MESH_ROUNDISH,
                roundish_mesh.vertices(),
                roundish_mesh.indices(),
            ),
            (&MESH_FLAT16, ground.vertices(), ground.indices()),
        ]
        .iter(),
    );

    app.add_texture(
        0,
        image::load_from_memory(include_bytes!("../assets/white-marble-2048x2048.png")).unwrap(),
        ResizeStrategy::Stretch(FilterType::Gaussian),
    );
    let mut id_bank = IDBank::new();
    for i in -10..11 {
        for j in 15..16 {
            for k in -10..11 {
                app.add_entity(Entity::new(
                    id_bank.next(),
                    if ((i + k) as i64).rem_euclid(2) == 0 {
                        MESH_CUBE2
                    } else {
                        MESH_ROUNDISH
                    },
                    Vector3::new(0.9 * i as f32, 0.0, 0.9 * k as f32),
                    Vector3::new(i as f32 * 0.2, 0.0, k as f32 * 0.2),
                    (
                        Vector3::new(1.0, 1.0, 1.0) / 2.0,
                        Vector3::new(-1.0, -1.0, -1.0) / 2.0,
                    ),
                    Matrix4::from_translation(
                        [2.0 * i as f32, 2.0 * j as f32, 2.0 * k as f32].into(),
                    ) * Matrix4::from_angle_z(Rad(-PI / 4.0))
                        * Matrix4::from_scale(10.0),
                ));
            }
        }
    }

    for x in -0..1 {
        for z in -0..1 {
            app.load_chunk(x, z, &mut id_bank);
        }
    }

    info!("Starting");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
