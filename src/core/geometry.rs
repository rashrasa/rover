use std::f32::consts::PI;

use crate::render::vertex::Vertex;
use cgmath::{InnerSpace, Matrix3, Rad, SquareMatrix, Vector3};

pub trait Mesh {
    fn vertices(&self) -> &[Vertex];
    fn indices(&self) -> &[u16];
}

/// A Face belongs to a model, and its vertices should already be in model space.
pub struct Face {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,

    // Currently only makes sense for a flat rectangular mesh.
    // Needs to be updated if other types are added.
    edge_px: Vec<u16>,
    edge_nx: Vec<u16>,
    edge_pz: Vec<u16>,
    edge_nz: Vec<u16>,
}

impl Face {
    /// Try using one of the convenience functions (i.e. Face::from_function)
    pub fn new(
        y_up_vertices: Vec<Vertex>,
        indices: Vec<u16>,
        edge_px: Vec<u16>,
        edge_nx: Vec<u16>,
        edge_pz: Vec<u16>,
        edge_nz: Vec<u16>,
    ) -> Self {
        Self {
            vertices: y_up_vertices,
            indices,

            edge_px,
            edge_nx,
            edge_pz,
            edge_nz,
        }
    }

    pub fn edge_px(&self) -> &Vec<u16> {
        &self.edge_px
    }

    pub fn edge_nx(&self) -> &Vec<u16> {
        &self.edge_nx
    }

    pub fn edge_pz(&self) -> &Vec<u16> {
        &self.edge_pz
    }

    pub fn edge_nz(&self) -> &Vec<u16> {
        &self.edge_nz
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
    /// Normals will be approximated by the gradient of the height function.
    /// It's important that [height] is a function which is continuous and differentiable on the domain provided.
    ///
    /// edge_{p/n}{x/z} are lists of indices of the vertices on the border of the flat mesh before any transformations.
    /// They are in counter-clockwise order when looking down on the x/z plane, with -z on the top and +x on the right.
    pub fn from_function(
        up: Vector3<f32>,
        domain_x: (f32, f32),
        domain_z: (f32, f32),
        resolution: (f32, f32),
        height: fn(f32, f32) -> f32,
    ) -> Result<Self, String> {
        let up = up.normalize();

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

        let correction_x = extra_x / (n_x - 1) as f32;
        let correction_z = extra_z / (n_z - 1) as f32;

        let dx = length_x / (n_x - 1) as f32 + correction_x;
        let dz = length_z / (n_z - 1) as f32 + correction_z;

        let final_rotation = rotate_to_axis(up, [0.0, 1.0, 0.0].into());

        let mut vertices = vec![];
        let mut indices = vec![];

        let mut edge_px: Vec<u16> = vec![];
        let mut edge_pz: Vec<u16> = vec![];
        let mut edge_nx: Vec<u16> = vec![];
        let mut edge_nz: Vec<u16> = vec![];

        let mut v_up = true;

        for k in 0..n_z {
            for i in 0..n_x {
                let this_index = i as u16 + k as u16 * n_x as u16;
                let x = domain_x.0 + i as f32 * dx;
                let z = domain_z.0 + k as f32 * dz;
                let y = height(x, z);

                let mut position = final_rotation * Vector3::new(x, 0.0, z);
                position += y * up;

                let normal = approximate_normal(height, (x, z));

                vertices.push(Vertex {
                    position: position.into(),
                    normal: (final_rotation * normal).into(),
                    tex_coords: [(x - domain_x.0) / length_x, (z - domain_z.0) / length_z],
                });

                if i == 0 {
                    edge_nx.push(this_index);
                }
                if i == n_x - 1 {
                    edge_px.insert(0, this_index);
                }
                if k == 0 {
                    edge_nz.insert(0, this_index);
                }
                if k == n_z - 1 {
                    edge_pz.push(this_index);
                }

                // add indices if possible
                if k > 0 {
                    if v_up && i != n_x - 1 {
                        indices.push(((i + 1) + (k - 1) * n_x) as u16); // up-right
                        indices.push((i + (k - 1) * n_x) as u16); // up
                        indices.push(this_index); // this
                        v_up = false;
                    } else if !v_up {
                        indices.push(((i - 1) + k * n_x) as u16); // left
                        indices.push(this_index); // this
                        indices.push((i + (k - 1) * n_x) as u16); // up
                        if i != n_x - 1 {
                            indices.push(((i + 1) + (k - 1) * n_x) as u16); // up-right
                            indices.push((i + (k - 1) * n_x) as u16); // up
                            indices.push(this_index); // this
                            v_up = false;
                        } else {
                            v_up = true;
                        }
                    }
                }
            }
            v_up = true;
        }

        Ok(Self {
            vertices,
            indices,

            edge_px,
            edge_nx,
            edge_pz,
            edge_nz,
        })
    }
}

impl Mesh for Face {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    fn indices(&self) -> &[u16] {
        &self.indices
    }
}

/// Edges are expected to be linearly seperable (one set doesnt intersect another).
/// If this is not the case, a join will be done anyways but may look distorted.
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

impl Mesh for Shape3 {
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
    let grad_dx: Vector3<f32> = [1.0, dy_dx, 0.0].into();
    let grad_dz: Vector3<f32> = [0.0, dy_dz, 1.0].into();

    grad_dx.cross(-grad_dz).normalize()
}

fn rotate_to_axis(axis: Vector3<f32>, original: Vector3<f32>) -> Matrix3<f32> {
    let axis = axis.normalize();
    let original = original.normalize();
    if axis == -original {
        let orthogonal = get_orthogonal(original).normalize();

        return Matrix3::from_axis_angle(original.cross(orthogonal), Rad(PI));
    }
    if axis == original {
        return Matrix3::identity();
    }

    let v = original.cross(axis);
    let s = v.magnitude();
    let c = original.dot(axis);
    let v_x = Matrix3::from_cols(
        Vector3::new(0.0, v.z, -v.y),
        Vector3::new(-v.z, 0.0, v.x),
        Vector3::new(v.y, -v.x, 0.0),
    );

    Matrix3::identity() + v_x + v_x * v_x * ((1.0 - c) / (s * s))
}

fn get_orthogonal(original: Vector3<f32>) -> Vector3<f32> {
    Vector3::new(
        original.y + original.z,
        original.z - original.x,
        -original.x - original.y,
    )
}

mod test {
    #![allow(unused_imports, dead_code)]

    use cgmath::assert_relative_eq;

    use super::*;

    const X_AXIS: Vector3<f32> = Vector3::<f32>::new(1.0, 0.0, 0.0);
    const Y_AXIS: Vector3<f32> = Vector3::<f32>::new(0.0, 1.0, 0.0);
    const Z_AXIS: Vector3<f32> = Vector3::<f32>::new(0.0, 0.0, 1.0);

    const ZERO: Vector3<f32> = Vector3::<f32>::new(0.0, 0.0, 0.0);

    const AXES: [Vector3<f32>; 3] = [X_AXIS, Y_AXIS, Z_AXIS];

    #[test]
    fn rotate_to_axis_basic_test() {
        crate::init_logging(log::LevelFilter::Debug);

        let _isq3 = 1.0 / 3.0_f32.sqrt();
        let orthonormal_p: Vector3<f32> = [_isq3, _isq3, _isq3].into();
        let orthonormal_n: Vector3<f32> = [-_isq3, -_isq3, -_isq3].into();

        assert_relative_eq!(rotate_to_axis(-Y_AXIS, Y_AXIS) * Y_AXIS, -Y_AXIS);
        assert_relative_eq!(
            rotate_to_axis(orthonormal_n, orthonormal_p) * orthonormal_p,
            orthonormal_n
        );
        assert_relative_eq!(rotate_to_axis(-X_AXIS, Y_AXIS) * Y_AXIS, -X_AXIS);
        assert_relative_eq!(rotate_to_axis(X_AXIS, Y_AXIS) * Y_AXIS, X_AXIS);

        assert_relative_eq!(rotate_to_axis(Y_AXIS, X_AXIS) * X_AXIS, Y_AXIS);

        assert_relative_eq!(rotate_to_axis(-X_AXIS, Y_AXIS) * ZERO, ZERO);

        assert_relative_eq!(rotate_to_axis(-Z_AXIS, Y_AXIS) * Y_AXIS, -Z_AXIS);
    }

    #[test]
    fn approx_normal_test() {
        let flat_normal = approximate_normal(|_, _| 0.0, (0.0, 0.0));

        assert_relative_eq!(flat_normal, Y_AXIS);
    }

    #[test]
    fn ortho_test() {
        let test_axes = [
            X_AXIS,
            Y_AXIS,
            Z_AXIS,
            -X_AXIS,
            -Y_AXIS,
            -Z_AXIS,
            Vector3::new(
                1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
            ),
            Vector3::new(
                -1.0 / 3.0_f32.sqrt(),
                -1.0 / 3.0_f32.sqrt(),
                -1.0 / 3.0_f32.sqrt(),
            ),
            Vector3::new(
                -1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
            ),
            Vector3::new(
                1.0 / 3.0_f32.sqrt(),
                -1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
            ),
            Vector3::new(
                1.0 / 3.0_f32.sqrt(),
                1.0 / 3.0_f32.sqrt(),
                -1.0 / 3.0_f32.sqrt(),
            ),
        ];

        for axis in test_axes {
            assert_relative_eq!(get_orthogonal(axis).dot(axis), 0.0);
        }
    }
}
