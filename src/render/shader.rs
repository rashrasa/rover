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
//  - Render Pipeline (draw order, face culling options, render configuration)

use std::{collections::HashMap, io::Read, num::NonZero, ops::Deref};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupLayout, ColorTargetState, DepthStencilState, Device, FragmentState,
    IndexFormat, MultisampleState, PipelineCache, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, VertexBufferLayout,
    VertexState,
};

use crate::{
    core::{Instanced, Meshed, Unique},
    render::{
        app::MeshInitData,
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
    instances: HashMap<u64, InstanceStorage<I>>,
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
            instances: HashMap::new(),
        })
    }

    /// Batch adding of meshes. Meshes will be synced to the GPU in this call.
    /// Mesh ids are returned in order, or the first error is returned.
    pub fn add_mesh(
        &mut self,
        device: &Device,
        queue: &Queue,
        mesh: MeshInitData<V>,
    ) -> Result<u64, MeshStorageError> {
        let id = self.meshes.add_mesh(&mesh.vertices, &mesh.indices)?;

        self.instances.insert(id, InstanceStorage::new(device));

        self.meshes.update_gpu(queue, device);

        Ok(id)
    }

    pub fn upsert_instances(
        &mut self,
        entities: &Vec<impl Instanced<I> + Meshed<u64> + Unique<u64>>,
    ) -> Result<(), String> {
        for entity in entities {
            let mesh_id = entity.mesh_id();
            let entity_id = entity.id();

            self.instances
                .get_mut(mesh_id)
                .unwrap()
                .upsert_instance(entity_id, entity.instance());
        }

        Ok(())
    }

    pub fn update_gpu(&mut self, device: &Device, queue: &Queue) {
        self.meshes.update_gpu(queue, device);
        for (_id, instance) in self.instances.iter_mut() {
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

        for (mesh_id, storage) in self.instances.iter() {
            if *storage.len() > 0 {
                render_pass.set_vertex_buffer(1, storage.slice());
                let (start, end) = self.meshes.get_mesh_index_bounds(&mesh_id).unwrap();
                render_pass.draw_indexed(start as u32..end as u32, 0, 0..*storage.len() as u32);
            }
        }
    }
}
