use crate::{ContiguousView, ContiguousViewMut};

pub struct Object {
    id: u64,
    mesh_id: u64,
    texture_id: u64,
    mass: f32,
    response: super::CollisionResponse,
    bounding_box: super::BoundingBox,
    transform: nalgebra::Matrix4<f32>,
    acceleration: nalgebra::Vector3<f32>,
    velocity: nalgebra::Vector3<f32>,

    // cached
    instance: [[f32; 4]; 4],
}

impl Object {
    pub fn new(
        id: u64,
        mesh_id: u64,
        texture_id: u64,
        mass: f32,
        response: super::CollisionResponse,
        bounding_box: super::BoundingBox,
        transform: nalgebra::Matrix4<f32>,
        acceleration: nalgebra::Vector3<f32>,
        velocity: nalgebra::Vector3<f32>,
    ) -> Self {
        Self {
            id,
            mesh_id,
            texture_id,
            mass,
            response,
            bounding_box,
            transform,
            acceleration,
            velocity,
            instance: transform.into(),
        }
    }
}

impl super::Dynamic for Object {
    fn velocity<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.acceleration.generic_view_with_steps(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (1, 1),
        )
    }

    fn velocity_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.acceleration.generic_view_with_steps_mut(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (1, 1),
        )
    }

    fn acceleration<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.acceleration.generic_view_with_steps(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (1, 1),
        )
    }

    fn acceleration_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.acceleration.generic_view_with_steps_mut(
            (0, 0),
            (nalgebra::Const::<3>, nalgebra::Const::<1>),
            (1, 1),
        )
    }
}

impl super::Transform for Object {
    fn transform<'a>(&'a self) -> ContiguousView<'a, 4, 4> {
        self.transform.generic_view_with_steps(
            (0, 3),
            (nalgebra::Const::<4>, nalgebra::Const::<4>),
            (1, 1),
        )
    }

    fn transform_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 4, 4> {
        self.transform.generic_view_with_steps_mut(
            (0, 3),
            (nalgebra::Const::<4>, nalgebra::Const::<4>),
            (1, 1),
        )
    }
}

impl super::Position for Object {
    fn position<'a>(&'a self) -> ContiguousView<'a, 3, 1> {
        self.transform.position()
    }

    fn position_mut<'a>(&'a mut self) -> ContiguousViewMut<'a, 3, 1> {
        self.transform.position_mut()
    }
}

impl super::Collide for Object {
    fn bounding_box(&self) -> &super::BoundingBox {
        &self.bounding_box
    }

    fn response(&self) -> &super::CollisionResponse {
        &self.response
    }
}

impl super::Mass for Object {
    fn mass(&self) -> &f32 {
        &self.mass
    }
}

impl super::Entity for Object {
    fn id(&self) -> &u64 {
        &self.id
    }
}

impl super::RenderInstanced<[[f32; 4]; 4]> for Object {
    fn texture_id(&self) -> &u64 {
        &self.texture_id
    }

    fn mesh_id(&self) -> &u64 {
        &self.mesh_id
    }

    fn instance(&self) -> &[[f32; 4]; 4] {
        &self.instance
    }
}
