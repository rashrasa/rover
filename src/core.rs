use nalgebra::Vector3;

#[derive(Clone)]
pub struct Entity {
    position: Vector3<f64>,
    velocity: Vector3<f64>,
    acceleration: Vector3<f64>,
    bounding_box: (Vector3<f64>, Vector3<f64>),
}

impl Entity {
    pub fn new(
        position: Vector3<f64>,
        velocity: Vector3<f64>,
        acceleration: Vector3<f64>,
        bounding_box: (Vector3<f64>, Vector3<f64>),
    ) -> Self {
        Self {
            position,
            velocity,
            acceleration,
            bounding_box,
        }
    }
    pub fn tick(&mut self, dt: f64) {
        let k1 = self.acceleration;
        let k2 = self.acceleration + k1 * dt / 2.0;
        let k3 = self.acceleration + k2 * dt / 2.0;
        let k4 = self.acceleration + k3 * dt;
        self.velocity += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;

        let k1 = self.velocity;
        let k2 = self.velocity + k1 * dt / 2.0;
        let k3 = self.velocity + k2 * dt / 2.0;
        let k4 = self.velocity + k3 * dt;
        self.position += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt;
    }
    pub fn position(&self) -> &Vector3<f64> {
        &self.position
    }

    pub fn bounding_box(&self) -> &(Vector3<f64>, Vector3<f64>) {
        &self.bounding_box
    }
    /// Returns Top-Left-Front and Bottom-Right-Back vertex vectors
    pub fn translate(&mut self, by: Vector3<f64>) {
        self.position += by;
    }
}
