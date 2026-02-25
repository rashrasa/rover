use std::f32::consts::PI;

use agate_engine::{
    core::{
        CHUNK_RESOLUTION, CHUNK_SIZE, Completer, MESH_CUBE2,
        entity::{BoundingBox, CollisionResponse},
        geometry::{EdgeJoin, Face, Mesh, Shape3},
    },
    render::{
        app::{App, MeshInitData, ObjectInitData, PlayerInitData, TextureInitData},
        textures::ResizeStrategy,
        vertex::Vertex,
    },
};
use image::imageops::FilterType;
use nalgebra::{UnitQuaternion, UnitVector3, Vector3};

fn main() {
    agate_engine::init_logging(log::LevelFilter::Debug);

    let mut app = App::new(1920, 1080, 0);
    let meshes = get_sample_meshes();
    let n_meshes = meshes.len() - 1; // excluding ground mesh

    let mut mesh_completers: Vec<Completer<u64>> = vec![];
    for mesh in meshes {
        let completer = app.add_mesh(mesh).unwrap();
        mesh_completers.push(completer);
    }

    let texture_completer = app.add_texture(TextureInitData {
        image: image::load_from_memory(include_bytes!("assets/white-marble-2048x2048.png"))
            .unwrap(),
        resize: ResizeStrategy::Stretch(FilterType::Gaussian),
    });

    app.add_player(PlayerInitData {
        mesh_id: mesh_completers.get(MESH_CUBE2 as usize).unwrap().clone(),
        texture_id: texture_completer.clone(),
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

    for i in -10..11 {
        for j in -2..3 {
            for k in -10..11 {
                app.add_object(ObjectInitData {
                    mesh_id: mesh_completers
                        .get(((i + j + k as i32).rem_euclid(n_meshes as i32)) as usize)
                        .unwrap()
                        .clone(),
                    texture_id: texture_completer.clone(),
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
