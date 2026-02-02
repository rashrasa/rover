use std::{
    collections::{HashMap, hash_map::Entry},
    ops::RangeBounds,
};

use log::debug;
use nalgebra::Matrix4;
use wgpu::{
    AddressMode, Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, Buffer, BufferSlice, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState,
    Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::DeviceDescriptor,
};

/// Maps an entity id to an index into a transform array. Once an entity is added,
///
/// Indirection is needed since instances are expected to have a specific ordering.
/// Also, it's likely faster to borrow the transforms Vec than it is to iterate over values in a Hashmap.
#[derive(Debug)]
pub struct InstanceStorage {
    map: HashMap<u64, usize>,
    transforms: Vec<[f32; 4]>, // 4 [f32;4] chunks

    instance_buffer: Buffer,
}

impl InstanceStorage {
    pub fn new(device: &Device) -> Self {
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: &[0 as u8; 0],
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        Self {
            map: HashMap::new(),
            transforms: Vec::new(),
            instance_buffer,
        }
    }

    pub fn get_instance(&self, entity_id: &u64) -> Option<&usize> {
        self.map.get(entity_id)
    }

    pub fn len(&self) -> usize {
        self.transforms.len() / 4
    }

    pub fn slice<S: RangeBounds<u64>>(&self, bounds: S) -> BufferSlice<'_> {
        self.instance_buffer.slice(bounds)
    }

    /// Inserts a new instance if it wasn't in the buffer, updates existing one if it was.
    pub fn upsert_instance(&mut self, entity_id: &u64, transform: &Matrix4<f32>) {
        let cols: [[f32; 4]; 4] = (*transform).into();
        match self.map.entry(*entity_id) {
            Entry::Occupied(occ) => {
                let i = *occ.get() * 4;
                for j in 0..4 {
                    self.transforms[i + j] = cols[j];
                }
            }
            Entry::Vacant(vac) => {
                vac.insert(self.transforms.len() / 4);
                self.transforms.extend(cols);
            }
        }
    }

    /// May re-allocate buffer. Should not be called during a render pass.
    pub fn update_gpu(&mut self, queue: &mut Queue, device: &mut Device) {
        let bytes = bytemuck::cast_slice(&self.transforms);
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
        }
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.transforms),
        );
    }
}
