pub mod entity;
pub mod world;

use crate::render::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Mesh {
    pub const fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            vertices: vertices,
            indices: indices,
        }
    }
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
}
