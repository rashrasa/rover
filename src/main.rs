use std::f32::consts::PI;

use cgmath::{Matrix4, Rad, Vector3, Vector4};
use image::imageops::FilterType;
use log::info;
use rover::{
    CHUNK_SIZE_M, CUBE_MESH_INDICES, GROUND_MESH,
    core::entity::Entity,
    render::{App, textures::ResizeStrategy, vertex::Vertex},
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
    let (g_v, g_i) = GROUND_MESH(CHUNK_SIZE_M, CHUNK_SIZE_M);

    app.add_meshes(
        [
            (
                "Cube",
                get_cube_vertices().as_slice(),
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

    for i in -2..2 {
        for j in -2..2 {
            for k in -2..2 {
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

    for x in -20..20 {
        for z in -20..20 {
            app.load_chunk(x, z);
        }
    }

    info!("Starting");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();

    info!("Starting shutdown");
}

fn get_cube_vertices() -> [Vertex; 8] {
    let tfl = Vector3::new(-0.5, 0.5, 0.5);
    let tfr = Vector3::new(0.5, 0.5, 0.5);
    let bfr = Vector3::new(0.5, -0.5, 0.5);
    let bfl = Vector3::new(-0.5, -0.5, 0.5);

    let tbl = Vector3::new(-0.5, 0.5, -0.5);
    let tbr = Vector3::new(0.5, 0.5, -0.5);
    let bbr = Vector3::new(0.5, -0.5, -0.5);
    let bbl = Vector3::new(-0.5, -0.5, -0.5);

    // normal: cross two adjacent vertices and rotate PI/4 rad towards the last adjacent vertex

    let tfl_norm = Matrix4::from_angle_z(Rad(-PI / 4.0))
        * Matrix4::from_angle_y(Rad(PI / 4.0))
        * Vector4::new(0.0, 0.0, 1.0, 1.0);

    let tfr_norm = Matrix4::from_angle_z(Rad(PI / 4.0))
        * Matrix4::from_angle_y(Rad(PI / 4.0))
        * Vector4::new(0.0, 0.0, 1.0, 1.0);

    let tbr_norm = Matrix4::from_angle_z(Rad(PI / 4.0))
        * Matrix4::from_angle_y(Rad(PI / 4.0))
        * Vector4::new(1.0, 0.0, 0.0, 1.0);

    let tbl_norm = Matrix4::from_angle_z(Rad(-PI / 4.0))
        * Matrix4::from_angle_y(Rad(PI / 4.0))
        * Vector4::new(1.0, 0.0, 0.0, 1.0);

    let bfl_norm = Matrix4::from_angle_z(Rad(-PI / 4.0))
        * Matrix4::from_angle_y(Rad(-PI / 4.0))
        * Vector4::new(0.0, 0.0, 1.0, 1.0);

    let bfr_norm = Matrix4::from_angle_z(Rad(PI / 4.0))
        * Matrix4::from_angle_y(Rad(-PI / 4.0))
        * Vector4::new(0.0, 0.0, 1.0, 1.0);

    let bbr_norm = Matrix4::from_angle_z(Rad(PI / 4.0))
        * Matrix4::from_angle_y(Rad(-PI / 4.0))
        * Vector4::new(1.0, 0.0, 0.0, 1.0);

    let bbl_norm = Matrix4::from_angle_z(Rad(-PI / 4.0))
        * Matrix4::from_angle_y(Rad(-PI / 4.0))
        * Vector4::new(1.0, 0.0, 0.0, 1.0);

    [
        Vertex {
            position: tfl.into(),
            normal: tfl_norm.truncate().into(),
            tex_coords: [1.0, 0.0],
        },
        Vertex {
            position: tfr.into(),
            normal: tfr_norm.truncate().into(),
            tex_coords: [1.0, 1.0],
        },
        Vertex {
            position: bfr.into(),
            normal: bfr_norm.truncate().into(),
            tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: bfl.into(),
            normal: bfl_norm.truncate().into(),
            tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: tbl.into(),
            normal: tbl_norm.truncate().into(),
            tex_coords: [0.0, 0.0],
        },
        Vertex {
            position: tbr.into(),
            normal: tbr_norm.truncate().into(),
            tex_coords: [1.0, 0.0],
        },
        Vertex {
            position: bbr.into(),
            normal: bbr_norm.truncate().into(),
            tex_coords: [1.0, 1.0],
        },
        Vertex {
            position: bbl.into(),
            normal: bbl_norm.truncate().into(),
            tex_coords: [1.0, 1.0],
        },
    ]
}
