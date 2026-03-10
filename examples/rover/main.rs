use std::f32::consts::PI;

const MESH_CUBE2: u64 = 0;
const MESH_ROUNDISH: u64 = 1;
const MESH_SPHERE: u64 = 2;

use agate_engine::{
    core::{
        CHUNK_RESOLUTION, CHUNK_SIZE, Completer,
        entity::{BoundingBox, CollisionResponse},
        geometry::{EdgeJoin, Face, Mesh, Shape3},
    },
    render::{
        app::{App, MeshInitData, ObjectInitData, TextureInitData},
        storage::textures::ResizeStrategy,
        vertex::Vertex,
    },
};
use image::imageops::FilterType;
use nalgebra::{UnitQuaternion, UnitVector3, Vector3};

fn main() {
    agate_engine::init_logging(log::LevelFilter::Debug);

    let mut app = App::new(1920, 1080, 0);
    let meshes = get_sample_meshes();

    let mut mesh_completers: Vec<Completer<u64>> = vec![];
    for mesh in meshes.into_iter() {
        mesh_completers.push(app.add_mesh(mesh).unwrap());
    }

    let texture_completer = app.add_texture(TextureInitData {
        image: image::load_from_memory(include_bytes!("assets/white-marble-2048x2048.png"))
            .unwrap(),
        resize: ResizeStrategy::Stretch(FilterType::Gaussian),
    });

    let penguin_model_completer = app
        .add_obj_model("examples/rover/assets/PenguinBaseMesh.obj")
        .unwrap();
    let penguin_texture_completer = app.add_texture(TextureInitData {
        image: image::load_from_memory(include_bytes!("assets/Penguin Diffuse Color.png")).unwrap(),
        resize: ResizeStrategy::Stretch(FilterType::Gaussian),
    });

    // app.add_player(PlayerInitData {
    //     mesh_id: mesh_completers.get(MESH_CUBE2 as usize).unwrap().clone(),
    //     texture_id: texture_completer.clone(),
    //     velocity: Vector3::new(1.0, 1.0, 1.0) * 10.0,
    //     acceleration: Vector3::new(0.0, 0.0, 0.0),
    //     bounding_box: BoundingBox::new(
    //         (1.0 / 2.0, 1.0 / 2.0, 1.0 / 2.0),
    //         (-1.0 / 2.0, -1.0 / 2.0, -1.0 / 2.0),
    //     ),
    //     scale: Vector3::new(1.0, 1.0, 1.0),
    //     rotation: UnitQuaternion::identity(),
    //     translation: Vector3::new(0.0, 10.0, 0.0),
    //     response: CollisionResponse::Inelastic(1.0),
    //     mass: 100.0,
    // });

    // app.add_object(ObjectInitData {
    //     mesh_id: mesh_completers.get(0).unwrap().clone(),
    //     texture_id: texture_completer.clone(),
    //     velocity: Vector3::zeros(),
    //     acceleration: Vector3::zeros(),
    //     bounding_box: BoundingBox::ZERO,
    //     mass: 5.0e19,
    //     scale: Vector3::new(5.0, 5.0, 5.0),
    //     rotation: UnitQuaternion::identity(),
    //     translation: Vector3::zeros(),
    //     response: CollisionResponse::Inelastic(0.9),
    // });

    for i in -3..4 {
        for j in -3..4 {
            for k in -3..4 {
                app.add_object(ObjectInitData {
                    mesh_id: penguin_model_completer.clone(),
                    texture_id: penguin_texture_completer.clone(),
                    velocity: Vector3::new(1.0, 1.0, 1.0),
                    acceleration: Vector3::zeros(),
                    bounding_box: BoundingBox::new(
                        (1.0 / 2.0, 1.0 / 2.0, 1.0 / 2.0),
                        (-1.0 / 2.0, -1.0 / 2.0, -1.0 / 2.0),
                    ),
                    mass: 5.0e10,
                    scale: Vector3::new(5.0, 5.0, 5.0),
                    rotation: UnitQuaternion::from_axis_angle(
                        &UnitVector3::new_normalize([0.0, 0.0, 1.0].into()),
                        (rand::random::<f32>().abs() / f32::MAX) * 2.0 * PI,
                    ),
                    translation: Vector3::new(10.0 * i as f32, 10.0 * j as f32, 10.0 * k as f32),
                    response: CollisionResponse::Inelastic(0.9),
                });
            }
        }
    }

    App::start(&mut app);
}

fn get_sample_meshes() -> Vec<MeshInitData<Vertex>> {
    let resolution = 32.0;

    let ground = Face::from_function(
        [0.0, 1.0, 0.0].into(),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (-(CHUNK_SIZE as f32) / 2.0, CHUNK_SIZE as f32 / 2.0),
        (CHUNK_RESOLUTION as f32, CHUNK_RESOLUTION as f32),
        |_, _| 5.0,
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
            vertices: cube2_mesh.vertices().to_vec(),
            indices: cube2_mesh.indices().to_vec(),
        },
        MeshInitData {
            vertices: roundish_mesh.vertices().to_vec(),
            indices: roundish_mesh.indices().to_vec(),
        },
        MeshInitData {
            vertices: sphere_mesh.vertices().to_vec(),
            indices: sphere_mesh.indices().to_vec(),
        },
        MeshInitData {
            vertices: ground.vertices().to_vec(),
            indices: ground.indices().to_vec(),
        },
    ]
}
