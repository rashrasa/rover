use std::sync::Arc;
use wgpu::{
    AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    BufferBindingType, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
    CompareFunction, DepthBiasState, DepthStencilState, Device, ExperimentalFeatures, Extent3d,
    Face, Features, FilterMode, FrontFace, Instance, InstanceDescriptor, Limits, LoadOp,
    MultisampleState, Operations, PolygonMode, PowerPreference, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderStages, StencilState, StoreOp, Surface, SurfaceConfiguration, SurfaceError, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension, Trace, wgt::DeviceDescriptor,
};
use winit::window::Window;

use crate::{
    core::{camera::Camera, entity::Entity, lights::LightSourceStorage},
    render::{
        app::{ActiveState, MeshInitData, TextureInitData},
        mesh,
        shader::{InstancedRenderModule, RenderPipelineSpec, ShaderSpec, UniformSpec, VertexSpec},
        textures::TextureStorage,
        vertex::Vertex,
    },
};

pub struct Renderer {
    window: Arc<Window>,

    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,

    render_module_transformed: InstancedRenderModule<Vertex, [[f32; 4]; 4]>,

    textures: TextureStorage,
    texture_bind_group_layout: BindGroupLayout,
    camera_bind_group_layout: BindGroupLayout,

    lights: LightSourceStorage,

    depth_texture: Texture,
    depth_view: TextureView,
    depth_sampler: Sampler,
    depth_bind_group: BindGroup,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (mut device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: Limits::defaults(),
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let size = Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let depth_desc = TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = device.create_texture(&depth_desc);
        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());
        let depth_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Depth Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            ..Default::default()
        });

        let depth_texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
                label: Some("Depth Bind Group Layout"),
            });

        let lights = LightSourceStorage::new(
            &mut device,
            [1000.0, 1000.0, 1000.0, 1.0],
            [1.0 / 255.0, 75.0 / 255.0, 75.0 / 255.0, 1.0],
            1.0e7,
        );

        let render_module_transformed = InstancedRenderModule::<Vertex, [[f32; 4]; 4]>::new(
            &device,
            Some("Transformed Render Module"),
            &VertexSpec {
                vertex_layout: Vertex::desc(),
                instance_layout: Entity::desc(),
            },
            &ShaderSpec {
                path: "assets/shader.wgsl".into(),
                vertex_shader_name: "vs_main".into(),
                fragment_shader_name: "fs_main".into(),
            },
            (vec![
                UniformSpec {
                    bind_group_layout: camera_bind_group_layout.clone(),
                },
                UniformSpec {
                    bind_group_layout: texture_bind_group_layout.clone(),
                },
                UniformSpec {
                    bind_group_layout: lights.layout().clone(),
                },
                UniformSpec {
                    bind_group_layout: depth_texture_bind_group_layout.clone(),
                },
            ])
            .iter(),
            &RenderPipelineSpec {
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
                fragment_color_target_state: Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }),
            },
        )
        .unwrap();

        let depth_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Depth Bind Group"),
            layout: &depth_texture_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&depth_view),
            }],
        });

        window.set_visible(true);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,

            render_module_transformed,

            depth_texture,
            depth_view,
            depth_sampler,
            depth_bind_group,

            lights,

            textures: TextureStorage::new(),
            texture_bind_group_layout,

            camera_bind_group_layout,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.depth_view = self
            .depth_texture
            .create_view(&TextureViewDescriptor::default());

        self.is_surface_configured = true;
    }

    pub fn new_texture(&mut self, data: TextureInitData) -> u64 {
        self.textures.new_texture(
            &mut self.device,
            &mut self.queue,
            data.image,
            data.resize,
            &self.texture_bind_group_layout,
        )
    }

    /// Add mesh to the render module responsible for handling elements
    /// with a full transform as the instance and the default vertex type.
    pub fn add_mesh_instanced(
        &mut self,
        mesh: MeshInitData<Vertex>,
    ) -> Result<u64, mesh::MeshStorageError> {
        self.render_module_transformed
            .add_mesh(&self.device, &self.queue, mesh)
    }

    pub fn update_instances(&mut self, active_state: &mut ActiveState) {
        self.render_module_transformed
            .upsert_instances(active_state.entities())
            .unwrap();

        // temporary fix
        active_state
            .current_camera_mut()
            .update_gpu(&mut self.queue);
    }

    pub fn update_gpu(&mut self) {
        self.render_module_transformed
            .update_gpu(&self.device, &self.queue);
    }

    pub fn render(&mut self, state: &mut ActiveState) -> Result<(), SurfaceError> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.render_module_transformed.draw_all(
                &mut render_pass,
                [
                    &state.current_camera().bind_group(),
                    &&self.textures.get(&0).unwrap().3,
                    &self.lights.bind_group(),
                    &&self.depth_bind_group,
                ]
                .iter(),
            );
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn camera_bind_group_layout(&self) -> &BindGroupLayout {
        &self.camera_bind_group_layout
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }
}
