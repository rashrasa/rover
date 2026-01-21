use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightSource {
    position: [f32; 4],
    colour: [f32; 4],
}

#[derive(Debug)]
pub struct LightSourceStorage {
    // TODO: Allow multiple lights
    light: LightSource,
    buffer: Buffer,
    layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl LightSourceStorage {
    pub fn new(device: &mut Device, light_p: [f32; 4], light_c: [f32; 4]) -> Self {
        let light = LightSource {
            position: light_p,
            colour: light_c,
        };

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light]),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            light,
            buffer,
            layout,
            bind_group,
        }
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
