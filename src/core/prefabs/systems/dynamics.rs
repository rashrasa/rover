use nalgebra::Vector3;

use crate::{Integrator, core};

pub struct DynamicsSystem;

impl core::System for DynamicsSystem {
    fn handle_tick(&mut self, args: &mut core::HandleTickArgs) {
        let dt = args.elapsed.as_secs_f32();
        for entity in args.state.entities_mut() {
            match crate::core::GLOBAL_INTEGRATOR {
                Integrator::RK4 => {
                    let acceleration = Vector3::from(entity.acceleration);
                    let a_k1 = acceleration;
                    let a_k2 = acceleration + a_k1 * dt / 2.0;
                    let a_k3 = acceleration + a_k2 * dt / 2.0;
                    let a_k4 = acceleration + a_k3 * dt;
                    entity.velocity += (a_k1 + 2.0 * a_k2 + 2.0 * a_k3 + a_k4) / 6.0 * dt;

                    let velocity = Vector3::from(entity.velocity);
                    let v_k1 = velocity;
                    let v_k2 = velocity + v_k1 * dt / 2.0;
                    let v_k3 = velocity + v_k2 * dt / 2.0;
                    let v_k4 = velocity + v_k3 * dt;

                    entity.translation += (v_k1 + 2.0 * v_k2 + 2.0 * v_k3 + v_k4) / 6.0 * dt;
                }
                Integrator::Euler => {
                    let acceleration = Vector3::from(entity.acceleration);
                    entity.velocity += acceleration * dt;

                    let velocity = Vector3::from(entity.velocity);
                    entity.translation += velocity * dt;
                }
            }
        }
    }
}
