use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
    ops::RangeBounds,
    rc::Rc,
};

use cgmath::{Matrix4, Vector4};
use log::error;
use wgpu::{
    Buffer, BufferSlice, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{INITIAL_INSTANCE_CAPACITY, render::vertex::Vertex};

pub mod entity;
pub mod world;

/// Stores a vertex and index buffer on main memory, can be hashed into with a string id to get the start and end indices.
///
/// Meshes can't be removed once added, for now. Max vertices: 2^16 = 65536
#[derive(Debug)]
pub struct MeshStorage {
    map: HashMap<String, (usize, usize, usize, usize)>, // vertex inclusive start, exclusive end, index inclusive start, exclusive end

    vertex_storage: Vec<Vertex>,
    vertex_buffer: Buffer,
    vertex_buffer_cap: usize,

    index_storage: Vec<u16>,
    index_buffer: Buffer,
    index_buffer_cap: usize,
}

impl MeshStorage {
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
    pub fn add_mesh(
        &mut self,
        mesh_id: &str,
        vertices: &[Vertex],
        indices: &[u16],
    ) -> Result<(), MeshStorageError> {
        let before_count_vertices = self.vertex_storage.len();
        let before_count_indexes = self.index_storage.len();
        let n = vertices.len();
        if before_count_vertices + n > u16::MAX as usize {
            return Err(MeshStorageError::MaxVerticesExceeded);
        }

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
            mesh_id.into(),
            (
                before_count_vertices,
                self.vertex_storage.len(),
                before_count_indexes,
                self.index_storage.len(),
            ),
        );

        Ok(())
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

    // TODO: Currently only re-allocates exactly the amount of storage it needs,
    //       which may be sufficient if all vertices and indexes are added at initialization.

    /// Copies the vertex and index buffers into the GPU.
    ///
    /// Should be run during initialization since some re-allocations may be performed.
    pub fn update_gpu(&mut self, queue: &mut Queue, device: &Device) {
        if self.vertex_buffer_cap < self.vertex_storage.len() {
            self.vertex_buffer.destroy();
            self.vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertex_storage),
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
            self.index_buffer.destroy();
            self.index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.index_storage),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });
            self.index_buffer_cap = self.index_storage.len();
        } else {
            queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&self.index_storage),
            );
        }
    }

    /// Returns the start and end of mesh in index buffer to be used in draw calls.
    pub fn get_mesh_index_bounds(&self, mesh_id: &str) -> Option<(&usize, &usize)> {
        self.map.get(mesh_id).map(|(_, _, s_i, e_i)| (s_i, e_i))
    }

    /// Returns a direct representation of a mesh.
    ///
    /// Likely not needed for draw calls. Use get_mesh_index_bounds instead.
    pub fn get_mesh(&self, mesh_id: &str) -> Option<(&[Vertex], &[u16])> {
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
}

/// Maps an entity id to an index into a transform array. Once an entity is added,
///
/// Indirection is needed since instances are expected to have a specific ordering.
/// Also, it's likely faster to borrow the transforms Vec than it is to iterate over values in a Hashmap.
#[derive(Debug)]
pub struct InstanceStorage {
    map: HashMap<String, usize>,
    transforms: Vec<[f32; 4]>,

    instance_buffer: Buffer,
}

impl InstanceStorage {
    pub fn new(device: &Device) -> Self {
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(
                &[0 as u8; INITIAL_INSTANCE_CAPACITY * size_of::<[[f64; 4]; 4]>()],
            ),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        Self {
            map: HashMap::new(),
            transforms: Vec::new(),
            instance_buffer,
        }
    }

    pub fn get_instance(&self, entity_id: &str) -> Option<&usize> {
        self.map.get(entity_id)
    }

    pub fn slice<S: RangeBounds<u64>>(&self, bounds: S) -> BufferSlice<'_> {
        self.instance_buffer.slice(bounds)
    }

    // TODO: Currently does not resize once it exceeds crate::INITIAL_INSTANCE_CAPACITY

    /// Inserts a new instance if it wasn't in the buffer, updates existing one if it was.
    pub fn upsert_instance(&mut self, entity_id: &str, transform: &Matrix4<f32>) {
        let cols: [[f32; 4]; 4] = [
            transform.x.into(),
            transform.y.into(),
            transform.z.into(),
            transform.w.into(),
        ];
        match self.map.entry(entity_id.into()) {
            Entry::Occupied(occ) => {
                let i = *occ.get() * 4;
                for j in 0..3 {
                    match self.transforms.get_mut(i + j) {
                        Some(v) => *v = cols[j],
                        None => {}
                    }
                }
            }
            Entry::Vacant(vac) => {
                self.transforms.extend(cols);
                vac.insert(self.transforms.len() / 4);
            }
        }
    }

    pub fn update_gpu(&self, queue: &mut Queue) {
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.transforms),
        );
    }
}
