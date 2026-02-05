// Meshes, Vertices, Instances, Transforms, Uniforms, Textures, Shaders, Rendering
//
// ORGANIZATION:
//
// Each "RenderModule" deals with a unique combination of:
//  - Vertex layout
//  - Instance layout (or none)
//  - Uniforms (cameras, lights, textures, data which is constant across all vertices/instances)
//  - Vertex shader
//  - Fragment shader
//  - Render Pipeline (draw order, backface culling, render configuration)

use std::{io::Read, num::NonZero, ops::Deref, slice::Iter};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupLayout, ColorTargetState, DepthStencilState, Device, FragmentState,
    IndexFormat, MultisampleState, PipelineCache, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, VertexBufferLayout,
    VertexState,
};

use crate::{
    core::entity::{Entity, RenderInstanced},
    render::{
        MeshInitData,
        instance::InstanceStorage,
        mesh::{MeshStorage, MeshStorageError},
    },
};

// Utility data types

pub struct VertexSpec {
    pub vertex_layout: VertexBufferLayout<'static>,
    pub instance_layout: VertexBufferLayout<'static>,
}

pub struct ShaderSpec {
    pub path: String,
    pub vertex_shader_name: String,
    pub fragment_shader_name: String,
}

pub struct UniformSpec {
    pub bind_group_layout: BindGroupLayout,
}

/// Render pipeline configuration options that need to be specified manually in
/// InstancedRenderModule::new.
pub struct RenderPipelineSpec<'a> {
    pub fragment_color_target_state: Option<ColorTargetState>,
    pub primitive: PrimitiveState,
    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: MultisampleState,
    pub multiview: Option<NonZero<u32>>,
    pub cache: Option<&'a PipelineCache>,
}

/// Expects that the instance data comes after the vertex data in the shader.
///
/// Main data type for managing instanced geometry.
/// Contains relevant meshes, textures, etc.
///
/// The reason for this separation is that some mesh/instance data may need to be handled in a special manner,
/// in a different shader, with different uniforms.
pub struct InstancedRenderModule<V, I>
where
    V: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
    I: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    render_pipeline: RenderPipeline,
    meshes: MeshStorage<V>,
    instances: Vec<InstanceStorage<I>>,
}

impl<V, I> InstancedRenderModule<V, I>
where
    V: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
    I: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    pub fn new<'a>(
        device: &Device,
        debug_name: Option<&str>,
        vertex_spec: &VertexSpec,
        shader_spec: &ShaderSpec,
        uniform_specs: impl Iterator<Item = &'a UniformSpec>,
        pipeline_spec: &RenderPipelineSpec,
    ) -> Result<Self, std::io::Error> {
        let mut shader = String::new();
        std::fs::File::open(&shader_spec.path)?
            .read_to_string(&mut shader)
            .unwrap();

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(shader.into()),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: debug_name
                .map(|n| n.to_owned() + " Render Pipeline Layout")
                .as_deref(),
            bind_group_layouts: &uniform_specs
                .map(|s| &s.bind_group_layout)
                .collect::<Vec<&BindGroupLayout>>(),
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some(&shader_spec.vertex_shader_name),
                buffers: &[
                    vertex_spec.vertex_layout.clone(),
                    vertex_spec.instance_layout.clone(),
                ],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some(&shader_spec.fragment_shader_name),
                targets: &[pipeline_spec.fragment_color_target_state.clone()],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: pipeline_spec.primitive,
            depth_stencil: pipeline_spec.depth_stencil.clone(),
            multisample: pipeline_spec.multisample,
            multiview: pipeline_spec.multiview,
            cache: pipeline_spec.cache,
        });

        Ok(Self {
            render_pipeline,
            meshes: MeshStorage::new(device),
            instances: Vec::new(),
        })
    }

    /// Batch adding of meshes. Meshes will be synced to the GPU in this call.
    /// Mesh ids are returned in order, or the first error is returned.
    pub fn add_meshes(
        &mut self,
        device: &Device,
        queue: &Queue,
        meshes: Vec<MeshInitData<V>>,
    ) -> Result<Vec<usize>, MeshStorageError> {
        let ids: Result<Vec<usize>, MeshStorageError> = meshes
            .iter()
            .map(|data| {
                let mesh_result = self.meshes.add_mesh(&data.vertices, &data.indices)?;

                self.instances.push(InstanceStorage::new(device));

                return Ok(mesh_result);
            })
            .collect();

        self.meshes.update_gpu(queue, device);

        ids
    }

    pub fn upsert_instances<'a>(
        &mut self,
        entities: impl Iterator<Item = &'a (impl Entity + RenderInstanced<I> + 'a)>,
    ) -> Result<(), String> {
        for entity in entities {
            let mesh_id = entity.mesh_id();
            let entity_id = entity.id();
            let instance = entity.instance();

            self.instances[*mesh_id as usize].upsert_instance(entity_id, instance.clone());
        }

        Ok(())
    }

    pub fn update_gpu(&mut self, device: &Device, queue: &Queue) {
        self.meshes.update_gpu(queue, device);
        for instance in self.instances.iter_mut() {
            instance.update_gpu(queue, device);
        }
    }

    pub fn draw_all<'a>(
        &self,
        render_pass: &mut RenderPass,
        uniforms: impl Iterator<Item = &'a (impl Deref<Target = &'a BindGroup> + 'a)>, // TODO: May be too convoluted but works for now
    ) {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.meshes.vertex_slice(..));
        render_pass.set_index_buffer(self.meshes.index_slice(..), IndexFormat::Uint16);

        for (i, bg) in uniforms.enumerate() {
            render_pass.set_bind_group(i as u32, Into::<&BindGroup>::into(**bg), &[]);
        }

        for (mesh_id, storage) in self.instances.iter().enumerate() {
            if storage.len() > 0 {
                render_pass.set_vertex_buffer(1, storage.slice(..));
                let (start, end) = self.meshes.get_mesh_index_bounds(&mesh_id).unwrap();
                render_pass.draw_indexed(start as u32..end as u32, 0, 0..storage.len() as u32);
            }
        }
    }
}
