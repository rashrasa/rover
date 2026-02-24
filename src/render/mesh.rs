use std::{collections::HashMap, ops::RangeBounds};

use bytemuck::{Pod, Zeroable};
use log::debug;
use wgpu::{
    Buffer, BufferSlice, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Stores a vertex and index buffer on main memory, can be hashed into with a string id to get the start and end indices.
///
/// Meshes can't be removed once added, for now. Max vertices: 2^16 = 65536
#[derive(Debug)]
pub struct MeshStorage<V>
where
    V: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    map: HashMap<u64, (usize, usize, usize, usize)>, // vertex inclusive start, exclusive end, index inclusive start, exclusive end

    vertex_storage: Vec<V>,
    vertex_buffer: Buffer,
    vertex_buffer_cap: usize,

    index_storage: Vec<u16>,
    index_buffer: Buffer,
    index_buffer_cap: usize,
}

impl<V> MeshStorage<V>
where
    V: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&[0 as u8; 100]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&[0 as u8; 100]),
            usage: BufferUsages::INDEX,
        });
        Self {
            map: HashMap::new(),

            vertex_storage: Vec::new(),
            vertex_buffer,
            vertex_buffer_cap: 0,

            index_storage: Vec::new(),
            index_buffer,
            index_buffer_cap: 0,
        }
    }

    /// [indices] should be relative to [vertices] location in slice.
    pub fn add_mesh(&mut self, vertices: &[V], indices: &[u16]) -> Result<u64, MeshStorageError> {
        let before_count_vertices = self.vertex_storage.len();
        let before_count_indexes = self.index_storage.len();
        let n = vertices.len();
        if before_count_vertices + n > u16::MAX as usize {
            return Err(MeshStorageError::MaxVerticesExceeded);
        }
        let id = self.map.len() as u64;

        for i in indices {
            let i = *i as usize;
            if i >= n {
                return Err(MeshStorageError::IndexOutOfBounds(i));
            }
        }

        self.vertex_storage.extend(vertices);
        self.index_storage.extend(
            indices
                .iter()
                .map(|index| *index + before_count_vertices as u16),
        );

        // TODO: Inserting a mesh with the same mesh_id multiple times will result in dead vertices/indices.
        self.map.insert(
            id,
            (
                before_count_vertices,
                self.vertex_storage.len(),
                before_count_indexes,
                self.index_storage.len(),
            ),
        );

        Ok(id)
    }

    pub fn vertex_slice<S: RangeBounds<u64>>(&self, bounds: S) -> BufferSlice<'_> {
        self.vertex_buffer.slice(bounds)
    }

    pub fn index_slice<S: RangeBounds<u64>>(&self, bounds: S) -> BufferSlice<'_> {
        self.index_buffer.slice(bounds)
    }

    pub fn num_indices(&self) -> usize {
        self.index_storage.len()
    }

    /// Copies the vertex and index buffers into the GPU.
    ///
    /// Should not be called during a render pass.
    pub fn update_gpu(&mut self, queue: &Queue, device: &Device) {
        if self.vertex_buffer_cap < self.vertex_storage.len() {
            let bytes = bytemuck::cast_slice(&self.vertex_storage);
            debug!(
                "re-allocating vertex buffer to {:.8} MB",
                bytes.len() as f32 / (1024.0 * 1024.0)
            );
            self.vertex_buffer.destroy();
            self.vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytes,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
            self.vertex_buffer_cap = self.vertex_storage.len();
        } else {
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.vertex_storage),
            );
        }
        if self.index_buffer_cap < self.index_storage.len() {
            let bytes = bytemuck::cast_slice(&self.index_storage);
            debug!(
                "re-allocating index buffer to {:.8} MB",
                bytes.len() as f32 / (1024.0 * 1024.0)
            );
            self.index_buffer.destroy();
            self.index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytes,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });
            self.index_buffer_cap = self.index_storage.len();
        }
        queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&self.index_storage),
        );
    }

    /// Returns the start and end of mesh in index buffer to be used in draw calls.
    pub fn get_mesh_index_bounds(&self, mesh_id: &u64) -> Option<(usize, usize)> {
        self.map.get(mesh_id).map(|(_, _, s_i, e_i)| (*s_i, *e_i))
    }

    /// Returns a direct representation of a mesh.
    ///
    /// Likely not needed for draw calls. Use get_mesh_index_bounds instead.
    pub fn get_mesh(&self, mesh_id: &u64) -> Option<(&[V], &[u16])> {
        self.map.get(mesh_id).map(|(s_v, e_v, s_i, e_i)| {
            (
                &self.vertex_storage[*s_v..*e_v],
                &self.index_storage[*s_i..*e_i],
            )
        })
    }
}

#[derive(Debug)]
pub enum MeshStorageError {
    /// Index of index array for which the value (index into vertices) is not within the bounds of [vertices].
    IndexOutOfBounds(usize),

    /// >2^16 vertices were added.
    MaxVerticesExceeded,

    MeshExists,
}
