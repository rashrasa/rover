use std::f32::consts::PI;

use image::imageops::FilterType;
use log::info;
use nalgebra::{Matrix4, Rotation3, UnitVector3, Vector3};
use rover::{
    CHUNK_RESOLUTION, CHUNK_SIZE, IDBank, MESH_CUBE2, MESH_FLAT16, MESH_ROUNDISH,
    core::{
        entity::{BoundingBox, CollisionResponse},
        geometry::{Face, Mesh, Shape3},
    },
    render::{
        App, MeshInitData, ObjectInitData, PlayerInitData, TextureInitData,
        textures::ResizeStrategy, vertex::Vertex,
    },
};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    rover::init_logging();
    // Winit wants to own app state
    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut app = App::new(&event_loop, 1920, 1080, 0);

    app.add_meshes(get_sample_meshes());

    app.add_texture(TextureInitData {
        id: 0,
        image: image::load_from_memory(include_bytes!("../assets/white-marble-2048x2048.png"))
            .unwrap(),
        resize: ResizeStrategy::Stretch(FilterType::Gaussian),
    });

    let mut id_bank = IDBank::new();

    app.add_player(PlayerInitData {
        id: id_bank.next(),
        mesh_id: MESH_CUBE2,
        texture_id: 0,
        velocity: Vector3::new(0.0, 0.0, 0.0),
        acceleration: Vector3::new(0.0, 0.0, 0.0),
        bounding_box: BoundingBox::new(
            (1.0 / 2.0, 1.0 / 2.0, 1.0 / 2.0),
            (-1.0 / 2.0, -1.0 / 2.0, -1.0 / 2.0),
        ),
        transform: Matrix4::new_translation(&[0.0, 10.0, 0.0].into()),
        response: CollisionResponse::Inelastic(1.0),
        mass: 1.0,
    });

    for i in -10..11 {
        for j in 15..16 {
            for k in -10..11 {
                app.add_object(ObjectInitData {
                    id: id_bank.next(),
                    mesh_id: if ((i + k) as i64).rem_euclid(2) == 0 {
                        MESH_CUBE2
                    } else {
                        MESH_ROUNDISH
                    },
                    texture_id: 0,
                    velocity: Vector3::new(0.9 * i as f32, 0.0, 0.9 * k as f32),
                    acceleration: Vector3::new(i as f32 * 0.2, 0.0, k as f32 * 0.2),
                    bounding_box: BoundingBox::new(
                        (1.0 / 2.0, 1.0 / 2.0, 1.0 / 2.0),
                        (-1.0 / 2.0, -1.0 / 2.0, -1.0 / 2.0),
                    ),
                    mass: 1.0,
                    transform: Matrix4::new_translation(
                        &[2.0 * i as f32, 2.0 * j as f32, 2.0 * k as f32].into(),
                    ) * Rotation3::from_axis_angle(
                        &UnitVector3::new_normalize([0.0, 0.0, 1.0].into()),
                        -PI / 4.0,
                    )
                    .to_homogeneous()
                        * Matrix4::new_scaling(10.0),

                    response: CollisionResponse::Inelastic(0.9),
                });
            }
        }
    }

    for x in -0..1 {
        for z in -0..1 {
            app.load_chunk(x, z);
        }
    }

    info!("Starting");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}

fn get_sample_meshes() -> Vec<MeshInitData<Vertex>> {
    let ground = Face::from_function(
        [0.0, 1.0, 0.0].into(),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (
            CHUNK_RESOLUTION as f32 / CHUNK_SIZE as f32,
            CHUNK_RESOLUTION as f32 / CHUNK_SIZE as f32,
        ),
        |x, z| 5.0,
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

    vec![
        MeshInitData {
            id: MESH_CUBE2,
            vertices: cube2_mesh.vertices().to_vec(),
            indices: cube2_mesh.indices().to_vec(),
        },
        MeshInitData {
            id: MESH_ROUNDISH,
            vertices: roundish_mesh.vertices().to_vec(),
            indices: roundish_mesh.indices().to_vec(),
        },
        MeshInitData {
            id: MESH_FLAT16,
            vertices: ground.vertices().to_vec(),
            indices: ground.indices().to_vec(),
        },
    ]
}
