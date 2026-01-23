use na::{ArrayStorage, Const, Matrix};
use nalgebra as na;

use crate::Integrator;

pub type Mat<T, const N: usize, const M: usize> =
    Matrix<T, Const<N>, Const<M>, ArrayStorage<T, N, M>>;

// Non-Linear Time-Invariant System
pub type FunctionXUT<const N: usize, const R: usize> =
    fn(&Mat<f64, N, 1>, &Mat<f64, R, 1>, &f64) -> f64;
pub type StateDifferentialEquations<const N: usize, const R: usize> = Mat<FunctionXUT<N, R>, N, 1>;

#[derive(Debug, Clone)]

/// N - Number of states
/// R - Number of inputs
///
/// This was modified to be used for managing an object's physical state with no concern for observability/measurements, state variance.
///
/// Inputs can be used to affect any state and states can be clamped to simulate physical limitations.
/// It was also expanded to be not time-invariant. Time can be used in the system.
/// Usually, this would be time since the start of the application.
///
/// Only use time if needed, most simple dynamics can be represented as a time-invariant system.
/// Complexities can be handled by manipulating inputs, clamps and adding more inputs.
pub struct DynamicSystem<const N: usize, const R: usize> {
    // dx/dt = f(x(t),u(t),t)
    dx_dt: StateDifferentialEquations<N, R>,
    x: Mat<f64, N, 1>,
}

impl<const N: usize, const R: usize> DynamicSystem<N, R> {
    pub fn new(dx_dt: StateDifferentialEquations<N, R>, x0: Mat<f64, N, 1>) -> Self {
        Self {
            dx_dt: dx_dt,
            x: x0,
        }
    }

    pub fn state(&self) -> &Mat<f64, N, 1> {
        &self.x
    }

    /// t has to be passed in, but can be some arbitrary value if not used.
    pub fn step(
        &mut self,
        dt: f64,
        t: f64,
        u: Mat<f64, R, 1>,
        min_clamp: Mat<f64, N, 1>,
        max_clamp: Mat<f64, N, 1>,
    ) {
        // TODO: Add gaussian noise
        match crate::GLOBAL_INTEGRATOR {
            Integrator::Euler => {
                for i in 0..N {
                    self.x[i] = (self.x[i] + dt * (self.dx_dt[i](&self.x, &u, &t)))
                        .min(max_clamp[i])
                        .max(min_clamp[i]);
                }
            }

            Integrator::RK4 => {
                for i in 0..N {
                    let k1 = self.dx_dt[i](&self.x, &u, &t);
                    let k2 = self.dx_dt[i](&self.x.add_scalar(k1 * dt / 2.0), &u, &(t + dt / 2.0));
                    let k3 = self.dx_dt[i](&self.x.add_scalar(k2 * dt / 2.0), &u, &(t + dt / 2.0));
                    let k4 = self.dx_dt[i](&self.x.add_scalar(k3 * dt), &u, &t);
                    self.x[i] = self.x[i] + (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt
                }
            }
        }
        self.x = self.x.sup(&min_clamp);
        self.x = self.x.inf(&max_clamp);
    }
}
