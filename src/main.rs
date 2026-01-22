use std::f32::consts::PI;

use cgmath::{Matrix4, Rad, Vector3, Vector4};
use image::imageops::FilterType;
use log::{debug, info};
use rover::{
    CHUNK_SIZE_M, CUBE_MESH_INDICES, GROUND_HEIGHT, GROUND_MESH,
    core::{
        entity::Entity,
        geometry::{EdgeJoin, Face, Geometry, Shape3},
    },
    render::{App, textures::ResizeStrategy, vertex::Vertex},
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();

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
        |x, z| 5.0 * x.sin() + 5.0 * z.sin(),
    )
    .unwrap();

    let _isq3: f32 = 1.0 / 3.0_f32.sqrt();
    let cube_mesh = Shape3::new(
        vec![
            Face::new(
                vec![
                    Vertex {
                        position: [-0.5, 0.5, 0.5],
                        normal: [-_isq3, _isq3, _isq3],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [-0.5, 0.5, -0.5],
                        normal: [-_isq3, _isq3, -_isq3],
                        tex_coords: [1.0, 0.0],
                    },
                    Vertex {
                        position: [-0.5, -0.5, -0.5],
                        normal: [-_isq3, -_isq3, -_isq3],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [-0.5, -0.5, 0.5],
                        normal: [-_isq3, -_isq3, _isq3],
                        tex_coords: [0.0, 1.0],
                    },
                ],
                vec![0, 1, 2, 2, 3, 0],
            ),
            Face::new(
                vec![
                    Vertex {
                        position: [0.5, 0.5, 0.5],
                        normal: [_isq3, _isq3, _isq3],
                        tex_coords: [0.0, 0.0],
                    },
                    Vertex {
                        position: [0.5, 0.5, -0.5],
                        normal: [_isq3, _isq3, -_isq3],
                        tex_coords: [1.0, 0.0],
                    },
                    Vertex {
                        position: [0.5, -0.5, -0.5],
                        normal: [_isq3, -_isq3, -_isq3],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.5],
                        normal: [_isq3, -_isq3, _isq3],
                        tex_coords: [0.0, 1.0],
                    },
                ],
                vec![0, 3, 2, 2, 1, 0],
            ),
        ],
        vec![
            EdgeJoin::new(vec![0, 1], 0, vec![0, 1], 1).unwrap(),
            EdgeJoin::new(vec![1, 2], 0, vec![1, 2], 1).unwrap(),
            EdgeJoin::new(vec![2, 3], 0, vec![2, 3], 1).unwrap(),
            EdgeJoin::new(vec![3, 0], 0, vec![3, 0], 1).unwrap(),
        ],
    )
    .unwrap();

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
                [-1.0, 0.0, 0.0].into(),
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
    app.add_meshes(
        [
            (
                "Experimental_Cube2",
                cube2_mesh.vertices(),
                cube2_mesh.indices(),
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

    for i in -10..=10 {
        for j in -10..=10 {
            for k in -10..=10 {
                app.add_entity(Entity::new(
                    &format!("rover_{}_{}_{}", i, j, k),
                    "Experimental_Cube2",
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, 0.0, 0.0),
                    (
                        Vector3::new(1.0, 1.0, 1.0) / 2.0,
                        Vector3::new(-1.0, -1.0, -1.0) / 2.0,
                    ),
                    Matrix4::from_translation(
                        [2.0 * i as f32, 2.0 * j as f32, 2.0 * k as f32].into(),
                    ) * Matrix4::from_angle_z(Rad(-PI / 2.0)),
                ));
            }
        }
    }

    for x in -0..0 {
        for z in -0..0 {
            app.load_chunk(x, z);
        }
    }

    info!("Starting");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}
