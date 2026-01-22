use std::f32::consts::PI;

use crate::render::vertex::Vertex;
use cgmath::{InnerSpace, Matrix3, Rad, Vector3};
use log::{debug, info};

pub trait Geometry {
    fn vertices(&self) -> &[Vertex];
    fn indices(&self) -> &[u16];
}

/// A Face belongs to a model, and its vertices should already be in model space.
pub struct Face {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Face {
    /// Try using one of the convenience functions (i.e. Face::from_function)
    pub fn new(y_up_vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            vertices: y_up_vertices,
            indices,
        }
    }

    /// Transformed flat mesh. [height] should accept x, z values within the domain described.
    /// Domain should be in a space where (0,1,0) is up. After the mesh is created, it will be rotated
    /// according to the provided up direction about the point (0,0,0).
    ///
    /// The middle of the domains is where the face will be centered.
    ///
    /// Resolution is the approximate number of vertices desired per 1.0 unit on each of the x and z axes.
    /// This will be floored to fit in the domain provided.
    ///
    /// Normals will be approximated by the negative inverse of the gradient of the height function.
    /// It's important that [height] is a function which is continuous and differentiable on the domain provided.
    pub fn from_function(
        up: Vector3<f32>,
        domain_x: (f32, f32),
        domain_z: (f32, f32),
        resolution: (f32, f32),
        height: fn(f32, f32) -> f32,
    ) -> Result<Self, String> {
        let length_x = domain_x.1 - domain_x.0;
        let length_z = domain_z.1 - domain_z.0;

        // expected verices
        let e_x = length_x * resolution.0;
        let e_z = length_z * resolution.1;

        // actual vertices
        let n_x = e_x.floor() as u32;
        let n_z = e_z.floor() as u32;
        if n_x == 0 || n_z == 0 {
            return Err(format!("Cannot create mesh (not enough vertices): domains:\n\tx: {:?}\n\tz: {:?}\nresolution: {:?}\nvertex count:\n\tx: {:?}\n\tz:{:?}", domain_x, domain_z, resolution, n_x, n_z).into());
        }
        let extra_x = (e_x - n_x as f32) * resolution.0;
        let extra_z = (e_z - n_z as f32) * resolution.1;

        let correction_x = extra_x / n_x as f32;
        let correction_z = extra_z / n_z as f32;

        let dx = 1.0 / n_x as f32 + correction_x;
        let dz = 1.0 / n_z as f32 + correction_z;

        let final_y_rotation = Matrix3::look_to_rh([0.0, 1.0, 0.0].into(), up);
        info!("{:?}", final_y_rotation);

        let mut vertices = vec![];
        let mut indices = vec![];
        let mut up = true;

        for k in 0..n_z {
            for i in 0..n_x {
                let x = domain_x.0 + i as f32 * dx;
                let z = domain_z.0 + k as f32 * dz;
                let y = height(x, z);

                let normal = approximate_normal(height, (x, z));
                info!("({},{},{})", x, y, z);
                vertices.push(Vertex {
                    position: (final_y_rotation * Vector3::new(x, y, z)).into(),
                    normal: (final_y_rotation * normal).into(),
                    tex_coords: [(x - domain_x.0) / length_x, (z - domain_z.0) / length_z],
                });

                // add indices if possible
                if k > 0 {
                    if up && i != n_x - 1 {
                        indices.push(((i + 1) + (k - 1) * n_x) as u16); // up-right
                        indices.push((i + (k - 1) * n_x) as u16); // up
                        indices.push((i + k * n_x) as u16); // this
                        up = false;
                    } else if !up {
                        indices.push(((i - 1) + k * n_x) as u16); // left
                        indices.push((i + k * n_x) as u16); // this
                        indices.push((i + (k - 1) * n_x) as u16); // up
                        if i != n_x - 1 {
                            indices.push(((i + 1) + (k - 1) * n_x) as u16); // up-right
                            indices.push((i + (k - 1) * n_x) as u16); // up
                            indices.push((i + k * n_x) as u16); // this
                            up = false;
                        } else {
                            up = true;
                        }
                    }
                }
            }
            up = true;
        }

        Ok(Self { vertices, indices })
    }
}

impl Geometry for Face {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    fn indices(&self) -> &[u16] {
        &self.indices
    }
}

/// Edges are expected to be linearly seperable (one set doesnt intersect another).
/// If this is not the case, a join will be done anyways but will look distorted.
pub struct EdgeJoin {
    // indices are in their own space
    edge_lower: Vec<u16>,
    lower_face_index: usize,

    edge_higher: Vec<u16>,
    higher_face_index: usize,
}

impl EdgeJoin {
    pub fn new(
        edge_lower: Vec<u16>,
        lower_face_index: usize,

        edge_higher: Vec<u16>,
        higher_face_index: usize,
    ) -> Result<Self, String> {
        if lower_face_index == higher_face_index {
            return Err("Edges belong to the same face".into());
        }
        Ok(Self {
            edge_lower,
            lower_face_index,
            edge_higher,
            higher_face_index,
        })
    }
}

// Represent any 3D shape
#[derive(Debug)]
pub struct Shape3 {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Shape3 {
    pub fn new(mut faces: Vec<Face>, face_joins: Vec<EdgeJoin>) -> Result<Self, String> {
        // Convert vertices and indices to model coordinates
        let mut face_index_start = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u16> = vec![];

        // Move vertices and indices out of faces
        let mut start = 0;
        for i in 0..faces.len() {
            let face = &mut faces[i];
            face_index_start.push(start);

            vertices.extend(face.vertices.iter());
            indices.extend(face.indices.iter().map(|index| index + start));

            start += face.vertices.len() as u16;
        }

        // Join faces
        for join in face_joins {
            if join.edge_higher.is_empty() || join.edge_lower.is_empty() {
                return Err("Received empty edge in EdgeJoin".into());
            }
            let mut i = 0;
            let mut j = 0;

            let l_n = join.edge_lower.len();
            let h_n = join.edge_higher.len();

            let mut i_up = false;

            for _ in 0..l_n + h_n - 1 {
                // translate face index to model index
                let l_idx = face_index_start[join.lower_face_index] + join.edge_lower[i];
                let h_idx = face_index_start[join.higher_face_index] + join.edge_higher[j];

                let v_i: Vector3<f32> = (&vertices[l_idx as usize]).position.into();
                let v_j: Vector3<f32> = (&vertices[h_idx as usize]).position.into();

                if i + j > 0 {
                    // join
                    if i_up {
                        let l_idx_prev =
                            face_index_start[join.lower_face_index] + join.edge_lower[i - 1];

                        indices.push(h_idx);
                        indices.push(l_idx);
                        indices.push(l_idx_prev);
                    } else {
                        let h_idx_prev =
                            face_index_start[join.higher_face_index] + join.edge_higher[j - 1];

                        indices.push(h_idx);
                        indices.push(l_idx);
                        indices.push(h_idx_prev);
                    }
                }
                // determine what counter to advance
                if i == l_n - 1 {
                    j += 1;
                    i_up = false;
                } else if j == h_n - 1 {
                    i += 1;
                    i_up = true;
                } else {
                    // determine next increment based on any criteria. either can be incremented here (and next vertex can be checked)

                    let v_i_next: Vector3<f32> = vertices
                        .get(
                            (face_index_start[join.lower_face_index] + join.edge_lower[i + 1])
                                as usize,
                        )
                        .unwrap()
                        .position
                        .into();
                    let v_j_next: Vector3<f32> = vertices
                        .get(
                            (face_index_start[join.higher_face_index] + join.edge_higher[j + 1])
                                as usize,
                        )
                        .unwrap()
                        .position
                        .into();

                    // implementation: minimize connecting edge lengths
                    if (v_i - v_j_next).magnitude() <= (v_j - v_i_next).magnitude() {
                        j += 1;
                        i_up = false;
                    } else {
                        i += 1;
                        i_up = true;
                    }
                }
            }
        }
        Ok(Self { vertices, indices })
    }
}

impl Geometry for Shape3 {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    fn indices(&self) -> &[u16] {
        &self.indices
    }
}

const H: f32 = 1e-04; // lower for more accurate results. too low risks underflow/floating point imprecision errors.
// y is up here
fn approximate_normal(f: fn(f32, f32) -> f32, p: (f32, f32)) -> Vector3<f32> {
    let dx = H / 2.0;
    let dz = H / 2.0;
    let dy_dx = (f(p.0 + dx, p.1) - f(p.0 - dx, p.1)) / (2.0 * H);
    let dy_dz = (f(p.0, p.1 + dz) - f(p.0, p.1 - dz)) / (2.0 * H);
    let grad_dx: Vector3<f32> = [dx, dy_dx, 0.0].into();
    let grad_dz: Vector3<f32> = [0.0, dy_dz, dz].into();

    grad_dx.cross(grad_dz).normalize()
}
