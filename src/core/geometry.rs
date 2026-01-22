use crate::render::vertex::Vertex;
use cgmath::{InnerSpace, Matrix3, Rad, Vector3};
use log::debug;

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
    pub fn new(y_up_vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            vertices: y_up_vertices,
            indices,
        }
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
