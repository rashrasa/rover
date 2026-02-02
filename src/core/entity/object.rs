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
        }
    }
}

impl super::Dynamic for Object {
    fn velocity(&self) -> &nalgebra::Vector3<f32> {
        &self.velocity
    }

    fn velocity_mut(&mut self) -> &mut nalgebra::Vector3<f32> {
        &mut self.velocity
    }

    fn acceleration(&self) -> &nalgebra::Vector3<f32> {
        &self.acceleration
    }

    fn acceleration_mut(&mut self) -> &mut nalgebra::Vector3<f32> {
        &mut self.acceleration
    }
}

impl super::Transform for Object {
    fn transform(&self) -> &nalgebra::Matrix4<f32> {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut nalgebra::Matrix4<f32> {
        &mut self.transform
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

impl super::RenderInstanced for Object {
    fn texture_id(&self) -> &u64 {
        &self.texture_id
    }

    fn mesh_id(&self) -> &u64 {
        &self.mesh_id
    }
}
