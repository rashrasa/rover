use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::{
    Float,
    core::{Instanced, Meshed, Unique, geometry::rotate_to_axis},
    render::GlobalIndexType,
};

const BASE_WIDTH: Float = 0.125;
const BASE_HEIGHT: Float = 0.75;

const HEAD_WIDTH: Float = 0.5;
const HEAD_HEIGHT: Float = 0.25;

pub struct MarkerEntity {
    pub id: u64,
    pub position: Vector3<Float>,
    pub direction: Vector3<Float>,
    pub color: Vector3<Float>,
    pub mesh_id: u64,
}

impl Instanced<MarkerInstance> for MarkerEntity {
    fn instance(&self) -> MarkerInstance {
        let transform: Matrix4<Float> =
            Matrix4::from_diagonal(&((5.0 * self.direction.normalize()).to_homogeneous()))
                * rotate_to_axis(
                    Vector3::new(self.direction.x, self.direction.y, self.direction.z),
                    [0.0, 1.0, 0.0].into(),
                )
                .to_homogeneous()
                + Matrix4::new_translation(&self.position);
        // Marker model points up before any transforms.
        [0.0, 1.0, 0.0];
        MarkerInstance {
            x: transform.column(0).into(),
            y: transform.column(1).into(),
            z: transform.column(2).into(),
            w: transform.column(3).into(),
        }
    }
}

impl Unique<u64> for MarkerEntity {
    fn id(&self) -> &u64 {
        &self.id
    }
}

impl Meshed<u64> for MarkerEntity {
    fn mesh_id(&self) -> &u64 {
        &self.mesh_id
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MarkerVertex {
    pub position: [Float; 3],
    pub color: [Float; 3],

    pub _padding: [Float; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MarkerInstance {
    pub x: [Float; 4],
    pub y: [Float; 4],
    pub z: [Float; 4],
    pub w: [Float; 4],
}

impl MarkerVertex {
    pub const fn vertex_desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<super::DefaultVertexType>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }

    pub const fn instance_desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<super::MarkerInstanceType>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[rustfmt::skip]
pub const MARKER_VERTICES: fn(Vector3<Float>) -> Vec<MarkerVertex> = |color| {
    let _padding = [0.0; 2];
    let crnr: Float = HEAD_WIDTH / (2.0 + Float::sqrt(2.0));
    let edge: Float = Float::sqrt(2.0) * crnr;
    
    vec![
        // 0
        // Base
        MarkerVertex {
            position: [-BASE_WIDTH, 0.0, -BASE_WIDTH],
            color: (0.5 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [BASE_WIDTH, 0.0, -BASE_WIDTH],
            color: (0.5 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [BASE_WIDTH, 0.0, BASE_WIDTH],
            color: (0.5 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [-BASE_WIDTH, 0.0, BASE_WIDTH],
            color: (0.5 * color).into(),
            _padding,
        },

        // 4
        // Sides
        MarkerVertex {
            position: [-BASE_WIDTH, BASE_HEIGHT, -BASE_WIDTH],
            color: (0.75 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [BASE_WIDTH, BASE_HEIGHT, -BASE_WIDTH],
            color: (0.75 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [BASE_WIDTH, BASE_HEIGHT, BASE_WIDTH],
            color: (0.75 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [-BASE_WIDTH, BASE_HEIGHT, BASE_WIDTH],
            color: (0.75 * color).into(),
            _padding,
        },

        // 8
        // Head Base
        // bottom
        MarkerVertex {
            position: [-edge / 2.0, HEAD_WIDTH, -HEAD_WIDTH/2.0],
            color: (0.50 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [edge / 2.0, HEAD_WIDTH, -HEAD_WIDTH/2.0],
            color: (0.50 * color).into(),
            _padding,
        },
        
        // right
        MarkerVertex {
            position: [HEAD_WIDTH/2.0, HEAD_WIDTH, -edge / 2.0],
            color: (0.50 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [HEAD_WIDTH/2.0, HEAD_WIDTH, edge / 2.0],
            color: (0.50 * color).into(),
            _padding,
        },

        // top
        MarkerVertex {
            position: [edge / 2.0, HEAD_WIDTH, HEAD_WIDTH/2.0],
            color: (0.50 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [-edge / 2.0, HEAD_WIDTH, HEAD_WIDTH/2.0],
            color: (0.50 * color).into(),
            _padding,
        },

        // left
        MarkerVertex {
            position: [-HEAD_WIDTH/2.0, HEAD_WIDTH, edge / 2.0],
            color: (0.50 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [-HEAD_WIDTH/2.0, HEAD_WIDTH, -edge / 2.0],
            color: (0.50 * color).into(),
            _padding,
        },

        // 16
        // Head Base Center, Top
        MarkerVertex {
            position: [0.0, BASE_HEIGHT, 0.0],
            color: (0.25 * color).into(),
            _padding,
        },
        MarkerVertex {
            position: [0.0, BASE_HEIGHT + HEAD_HEIGHT, 0.0],
            color: color.into(),
            _padding,
        },
    ]
};

#[rustfmt::skip]
pub const MARKER_INDICES: &'static [GlobalIndexType] = &[
    // Base
    0, 1, 2,    2, 3, 0,

    // Sides
    0, 1, 5,    5, 4, 0,
    1, 2, 6,    6, 5, 1,
    2, 3, 7,    7, 6, 2,
    3, 0, 4,    4, 7, 3,

    // Base Top
    4, 5, 6,    6, 7, 4,

    // Head Base
    8, 9, 16,
    9, 10, 16,
    10, 11, 16,
    11, 12, 16,
    12, 13, 16,
    13, 14, 16,
    14, 15, 16,
    15, 8, 16,

    // Head Top
    8, 9, 17,
    9, 10, 17,
    10, 11, 17,
    11, 12, 17,
    12, 13, 17,
    13, 14, 17,
    14, 15, 17,
    15, 8, 17,
];
