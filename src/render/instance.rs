use bytemuck::{Pod, Zeroable};
use log::debug;
use wgpu::{
    Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device, Queue,
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
            contents: &[0 as u8; 100],
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

    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn capacity(&self) -> u64 {
        self.instance_buffer.size()
    }

    pub fn slice(&self) -> BufferSlice<'_> {
        self.instance_buffer
            .slice(0..self.len() * size_of::<[[f32; 4]; 4]>() as u64)
    }

    /// Inserts a new instance if it wasn't in the buffer, updates existing one if it was.
    pub fn upsert_instance(&mut self, entity_id: &u64, data: I) {
        if *entity_id < self.len() {
            self.data[*entity_id as usize] = data;
        } else {
            self.data.push(data);
        }
    }

    /// May re-allocate buffer.
    pub fn update_gpu(&mut self, queue: &Queue, device: &Device) {
        let bytes = bytemuck::cast_slice(&self.data);
        if bytes.len() > self.capacity() as usize {
            let new_size = (self.capacity() * 2).max(bytes.len() as u64);
            debug!(
                "re-allocating instance buffer to {:.8} MB",
                new_size as f32 / (1024.0 * 1024.0)
            );
            self.instance_buffer.destroy();
            self.instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Instance Buffer"),
                size: new_size,
                mapped_at_creation: false,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });
        }
        queue.write_buffer(&self.instance_buffer, 0, bytes);
    }
}
