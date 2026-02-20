use std::f32::consts::PI;

use image::imageops::FilterType;
use log::info;
use nalgebra::{Matrix4, Rotation3, UnitQuaternion, UnitVector3, Vector3};
use rover::{
    CHUNK_RESOLUTION, CHUNK_SIZE, IDBank, MESH_CUBE2, MESH_FLAT16, MESH_ROUNDISH,
    core::{
        entity::{BoundingBox, CollisionResponse},
        geometry::{EdgeJoin, Face, Mesh, Shape3},
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
    let meshes = get_sample_meshes();
    let n_meshes = meshes.len();
    app.add_meshes(meshes);

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
        scale: Vector3::identity(),
        rotation: UnitQuaternion::identity(),
        translation: Vector3::new(0.0, 10.0, 0.0),
        response: CollisionResponse::Inelastic(1.0),
        mass: 100.0,
    });

    for i in 0..10 {
        for j in 0..1 {
            for k in 0..10 {
                app.add_object(ObjectInitData {
                    id: id_bank.next(),
                    mesh_id: (i + j + k) % n_meshes as u64,
                    texture_id: 0,
                    velocity: Vector3::zeros(),
                    acceleration: Vector3::zeros(),
                    bounding_box: BoundingBox::new(
                        (1.0 / 2.0, 1.0 / 2.0, 1.0 / 2.0),
                        (-1.0 / 2.0, -1.0 / 2.0, -1.0 / 2.0),
                    ),
                    mass: 5.0,
                    scale: Vector3::identity() * 5.0,
                    rotation: UnitQuaternion::from_axis_angle(
                        &UnitVector3::new_normalize([0.0, 0.0, 1.0].into()),
                        -(PI * (i + k + j) as f32) / 4.0,
                    ),
                    translation: Vector3::new(10.0 * i as f32, 2.0 * j as f32, 10.0 * k as f32),
                    response: CollisionResponse::Inelastic(0.9),
                });
            }
        }
    }

    info!("Starting");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}

fn get_sample_meshes() -> Vec<MeshInitData<Vertex>> {
    let resolution = 32.0;

    let ground = Face::from_function(
        [0.0, 1.0, 0.0].into(),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (CHUNK_RESOLUTION as f32, CHUNK_RESOLUTION as f32),
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
                (resolution, resolution),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [0.0, -1.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [-1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, 1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |_, _| 0.5,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, -1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
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
                (resolution, resolution),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [0.0, -1.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [-1.0, 0.0, 0.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, 1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
            Face::from_function(
                [0.0, 0.0, -1.0].into(),
                (-0.5, 0.5),
                (-0.5, 0.5),
                (resolution, resolution),
                |x, z| 1.0 - x * x - z * z,
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap();

    let sphere_top = Face::from_function(
        [0.0, 1.0, 0.0].into(),
        (-0.5, 0.5),
        (-0.5, 0.5),
        (resolution, resolution),
        |x, z| 1.0 - x * x - z * z,
    )
    .unwrap();
    let sphere_bottom = Face::from_function(
        [0.0, -1.0, 0.0].into(),
        (-0.5, 0.5),
        (-0.5, 0.5),
        (resolution, resolution),
        |x, z| 1.0 - x * x - z * z,
    )
    .unwrap();

    let face_joins = vec![
        EdgeJoin::new(
            sphere_bottom.edge_px().clone(),
            1,
            sphere_top.edge_nx().clone(),
            0,
        )
        .unwrap(),
        EdgeJoin::new(
            sphere_bottom.edge_nx().clone(),
            1,
            sphere_top.edge_px().clone(),
            0,
        )
        .unwrap(),
        EdgeJoin::new(
            sphere_bottom.edge_pz().clone(),
            1,
            sphere_top.edge_pz().clone(),
            0,
        )
        .unwrap(),
        EdgeJoin::new(
            sphere_bottom.edge_nz().clone(),
            1,
            sphere_top.edge_nz().clone(),
            0,
        )
        .unwrap(),
    ];

    let sphere_mesh = Shape3::new(vec![sphere_top, sphere_bottom], face_joins).unwrap();

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
        MeshInitData {
            id: 3,
            vertices: sphere_mesh.vertices().to_vec(),
            indices: sphere_mesh.indices().to_vec(),
        },
    ]
}
