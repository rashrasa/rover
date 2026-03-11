use std::fmt::Debug;

use nalgebra::{Matrix4, UnitQuaternion, Vector3, Vector4};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::core::{Instanced, Meshed, Unique, camera::NoClipCamera};

/// Elastic collisions have CollisionResponse::Inelastic(1.0).
/// Inelastic takes any value. Values exceeding 1.0 will result in
/// energy magically being added to the system. Values below 0.0 will
/// be clamped to 0.0.
#[derive(Debug)]
pub enum CollisionResponse {
    Immovable,
    Inelastic(f32),
}

#[derive(Debug)]
pub struct BoundingBox {
    x: f32,
    y: f32,
    z: f32,

    x_size: f32,
    y_size: f32,
    z_size: f32,
}

impl BoundingBox {
    pub const fn new(top_left_front: (f32, f32, f32), size: (f32, f32, f32)) -> Self {
        Self {
            x: top_left_front.0,
            y: top_left_front.1,
            z: top_left_front.2,
            x_size: size.0,
            y_size: size.1,
            z_size: size.2,
        }
    }

    pub const ZERO: BoundingBox = Self::new((0.0, 0.0, 0.0), (0.0, 0.0, 0.0));

    /// Returns None if they don't intersect.
    ///
    /// Result vector is a signed distance of how far they intersect in each axis.
    pub fn intersects(&self, other: &BoundingBox) -> Option<[f32; 3]> {
        let x = (other.x + other.x_size) / 2.0 - (self.x + self.x_size) / 2.0;
        let y = (other.y + other.y_size) / 2.0 - (self.y + self.y_size) / 2.0;
        let z = (other.z + other.z_size) / 2.0 - (self.z + self.z_size) / 2.0;

        let min_x_size = self.x_size.min(other.x_size);
        let min_y_size = self.y_size.min(other.y_size);
        let min_z_size = self.z_size.min(other.z_size);

        if x.abs() > min_x_size || y.abs() > min_y_size || z.abs() > min_z_size {
            return None;
        }

        Some([x, y, z])
    }
}

pub enum EntityType {
    Player {
        // TODO: camera and transform both store a position
        camera: NoClipCamera,
    },
    Object,
}

pub struct Entity {
    // Keys
    id: u64,
    pub mesh_id: u64,
    pub texture_id: u64,

    // Transforms, in order
    pub scale: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub translation: Vector3<f32>,

    // Physics
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub bounding_box: BoundingBox,

    pub entity_type: EntityType,
    pub response: CollisionResponse,
    pub mass: f32,
}

impl Entity {
    pub fn new(
        id: u64,
        mesh_id: u64,
        texture_id: u64,
        scale: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        translation: Vector3<f32>,
        velocity: Vector3<f32>,
        acceleration: Vector3<f32>,
        bounding_box: BoundingBox,

        entity_type: EntityType,
        response: CollisionResponse,
        mass: f32,
    ) -> Self {
        Self {
            id,
            mesh_id,
            texture_id,
            scale,
            rotation,
            translation,
            velocity,
            acceleration,
            bounding_box,
            entity_type,
            response,
            mass,
        }
    }

    pub fn texture_id(&self) -> &u64 {
        &self.texture_id
    }
}

impl Meshed<u64> for Entity {
    fn mesh_id(&self) -> &u64 {
        &self.mesh_id
    }
}

impl Unique<u64> for Entity {
    fn id(&self) -> &u64 {
        &self.id
    }
}

impl Instanced<[[f32; 4]; 4]> for Entity {
    fn instance(&self) -> [[f32; 4]; 4] {
        let mut mat =
            Matrix4::from_diagonal(&Vector4::new(self.scale.x, self.scale.y, self.scale.z, 1.0))
                * self.rotation.to_rotation_matrix().to_homogeneous();
        let mut column = mat.column_mut(3);
        column.x = self.translation.x;
        column.y = self.translation.y;
        column.z = self.translation.z;

        Into::<[[f32; 4]; 4]>::into(mat)
    }
}

#[allow(unused_imports)]
mod tests {
    use std::f32::consts::PI;

    use assertables::{assert_abs_diff_eq_x, assert_abs_diff_lt_x};
    use nalgebra::{UnitQuaternion, UnitVector3, Vector3};

    use crate::core::{
        Instanced,
        entity::{BoundingBox, CollisionResponse, Entity, EntityType},
    };

    #[test]
    fn correct_basic_transformation() {
        let entity = Entity::new(
            0,
            0,
            0,
            Vector3::new(5.0, 6.0, 7.0),
            UnitQuaternion::from_axis_angle(
                &UnitVector3::new_normalize(Vector3::new(0.0, 0.0, 1.0)),
                PI / 4.0,
            ) * UnitQuaternion::from_axis_angle(
                &UnitVector3::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
                PI / 5.0,
            ) * UnitQuaternion::from_axis_angle(
                &UnitVector3::new_normalize(Vector3::new(1.0, 0.0, 0.0)),
                PI / 6.0,
            ),
            Vector3::new(10.0, 11.0, 12.0),
            Vector3::zeros(),
            Vector3::zeros(),
            BoundingBox::ZERO,
            EntityType::Object,
            CollisionResponse::Immovable,
            1.0,
        );
        let rotation: [[f32; 3]; 3] = (*entity.rotation.to_rotation_matrix().matrix()).into();
        let expected_rotation = [
            [0.5721, 0.5721, -0.5878],
            [-0.4046, 0.8202, 0.4045],
            [0.7135, 0.0064, 0.7006],
        ];

        for i in 0..3 {
            for j in 0..3 {
                assert_abs_diff_lt_x!(expected_rotation[i][j], rotation[i][j], 1.0e-2);
            }
        }

        let instance = entity.instance();
        // used matlab for values
        let expected_instance = [
            [2.8603, 3.4324, -4.1145, 0.0],
            [-2.0228, 4.9211, 2.8316, 0.0],
            [3.5675, 0.0383, 4.9044, 0.0],
            [10.0, 11.0, 12.0, 1.0],
        ];
        for i in 0..4 {
            for j in 0..4 {
                assert_abs_diff_lt_x!(expected_instance[i][j], instance[i][j], 1.0e-2);
            }
        }
    }
}
