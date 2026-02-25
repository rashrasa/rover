use std::ops::RangeBounds;

use bytemuck::{Pod, Zeroable};
use log::debug;
use wgpu::{
    Buffer, BufferSlice, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Maps an entity id to an index into a transform array. Once an entity is added, it can't be removed (for now).
///
/// Indirection is needed since instances are expected to have a specific ordering.
#[derive(Debug)]
pub struct InstanceStorage<I>
where
    I: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    data: Vec<I>,

    instance_buffer: Buffer,
}

impl<I> InstanceStorage<I>
where
    I: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    pub fn new(device: &Device) -> Self {
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[0 as u8; 0],
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        Self {
            data: Vec::new(),
            instance_buffer,
        }
    }

    pub fn get_instance(&self, entity_id: &u64) -> Option<&I> {
        self.data.get(*entity_id as usize)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn slice<S: RangeBounds<u64>>(&self, bounds: S) -> BufferSlice<'_> {
        self.instance_buffer.slice(bounds)
    }

    /// Inserts a new instance if it wasn't in the buffer, updates existing one if it was.
    pub fn upsert_instance(&mut self, entity_id: &u64, data: I) {
        if (*entity_id as usize) < self.data.len() {
            self.data[*entity_id as usize] = data;
        } else {
            self.data.push(data);
        }
    }

    /// May re-allocate buffer. Should not be called during a render pass in its current state.
    pub fn update_gpu(&mut self, queue: &Queue, device: &Device) {
        let bytes = bytemuck::cast_slice(&self.data);
        if bytes.len() > self.instance_buffer.size() as usize {
            debug!(
                "re-allocating instance buffer to {:.8} MB",
                bytes.len() as f32 / (1024.0 * 1024.0)
            );
            self.instance_buffer.destroy();
            self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytes,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
        } else {
            queue.write_buffer(&self.instance_buffer, 0, bytes);
        }
    }
}
