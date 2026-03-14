/*
   All unique vertex types are stored here.
*/

pub mod default;
pub type DefaultVertexType = default::Vertex;
pub type DefaultInstanceType = [[f32; 4]; 4];

pub mod terrain;
pub type TerrainVertexType = terrain::TerrainVertex;
pub type TerrainInstanceType = f32;

pub mod marker;
pub type MarkerVertexType = marker::MarkerVertex;
pub type MarkerInstanceType = marker::MarkerInstance;
