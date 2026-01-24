use std::{f32::consts::PI, fs::File};

use cgmath::{Matrix4, Rad, Vector3, Vector4};
use image::imageops::FilterType;
use log::{debug, info};
use rodio::Decoder;
use rover::{
    CHUNK_SIZE_M, CUBE_MESH_INDICES, GROUND_HEIGHT, GROUND_MESH,
    core::{
        entity::Entity,
        geometry::{EdgeJoin, Face, Mesh, Shape3},
    },
    render::{App, textures::ResizeStrategy, vertex::Vertex},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    rover::init_logging();
    // Winit wants to own app state
    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut app = App::new(&event_loop, 1920, 1080, 0);

    let ground = Face::from_function(
        [0.0, 1.0, 0.0].into(),
        (-(CHUNK_SIZE_M as f32) / 2.0, CHUNK_SIZE_M as f32 / 2.0),
        (-(CHUNK_SIZE_M as f32) / 2.0, CHUNK_SIZE_M as f32 / 2.0),
        (
            32.0 / (CHUNK_SIZE_M as f32 / 2.0),
            32.0 / (CHUNK_SIZE_M as f32 / 2.0),
        ),
        |x, z| -0.01 * (x * x + z * z),
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
            (
                "Experimental_Cube2",
                cube2_mesh.vertices(),
                cube2_mesh.indices(),
            ),
            (
                "Roundish",
                roundish_mesh.vertices(),
                roundish_mesh.indices(),
            ),
            ("Flat16", ground.vertices(), ground.indices()),
        ]
        .iter(),
    );

    app.add_texture(
        "test".into(),
        image::load_from_memory(include_bytes!("../assets/white-marble-2048x2048.png")).unwrap(),
        ResizeStrategy::Stretch(FilterType::Gaussian),
    );

    for i in -10..11 {
        for j in 15..16 {
            for k in -10..11 {
                app.add_entity(Entity::new(
                    &format!("rover_{}_{}_{}", i, j, k),
                    if ((i + k) as i64).rem_euclid(2) == 0 {
                        "Experimental_Cube2"
                    } else {
                        "Roundish"
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

    for x in -50..51 {
        for z in -50..51 {
            app.load_chunk(x, z);
        }
    }

    info!("Starting");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
