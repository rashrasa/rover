use egui::{Color32, RichText};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use nalgebra::Vector3;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
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
    Float,
    core::{camera::Camera, entity::Entity, lights::LightSourceStorage},
    render::{
        app::{ActiveState, MeshInitData, TextureInitData},
        gui::EguiRenderer,
        module::{InstancedRenderModule, RenderPipelineSpec, ShaderSpec, UniformSpec, VertexSpec},
        storage::{mesh, textures::TextureStorage},
        vertex::{
            DefaultInstanceType, DefaultVertexType, MarkerInstanceType, MarkerVertexType,
            TerrainInstanceType, TerrainVertexType,
            marker::{MARKER_INDICES, MARKER_VERTICES, MarkerEntity},
        },
    },
};

pub struct Renderer {
    window: Arc<Window>,

    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,

    render_module_transformed: InstancedRenderModule<DefaultVertexType, DefaultInstanceType>,
    render_module_terrain: InstancedRenderModule<TerrainVertexType, TerrainInstanceType>,
    render_module_markers: InstancedRenderModule<MarkerVertexType, MarkerInstanceType>,

    textures: TextureStorage,
    texture_bind_group_layout: BindGroupLayout,
    camera_bind_group_layout: BindGroupLayout,

    lights: LightSourceStorage,

    depth_texture: Texture,
    depth_view: TextureView,
    depth_sampler: Sampler,
    depth_bind_group: BindGroup,

    egui_renderer: EguiRenderer,
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
            [255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 1.0],
            1.0e6,
        );

        let render_module_transformed =
            InstancedRenderModule::<DefaultVertexType, DefaultInstanceType>::new(
                &device,
                Some("Main Render Module"),
                &VertexSpec {
                    vertex_layout: DefaultVertexType::vertex_desc(),
                    instance_layout: DefaultVertexType::instance_desc(),
                },
                &ShaderSpec {
                    path: "src/render/shaders/default.wgsl".into(),
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

        let render_module_terrain =
            InstancedRenderModule::<TerrainVertexType, TerrainInstanceType>::new(
                &device,
                Some("Terrain Render Module"),
                &VertexSpec {
                    vertex_layout: TerrainVertexType::vertex_desc(),
                    instance_layout: TerrainVertexType::instance_desc(),
                },
                &ShaderSpec {
                    path: "src/render/shaders/terrain.wgsl".into(),
                    vertex_shader_name: "vs_main".into(),
                    fragment_shader_name: "fs_main".into(),
                },
                (vec![
                    // TODO: Add sun and moon
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

        let mut render_module_markers =
            InstancedRenderModule::<MarkerVertexType, MarkerInstanceType>::new(
                &device,
                Some("Markers' Render Module"),
                &VertexSpec {
                    vertex_layout: MarkerVertexType::vertex_desc(),
                    instance_layout: MarkerVertexType::instance_desc(),
                },
                &ShaderSpec {
                    path: "src/render/shaders/marker.wgsl".into(),
                    vertex_shader_name: "vs_main".into(),
                    fragment_shader_name: "fs_main".into(),
                },
                (vec![UniformSpec {
                    bind_group_layout: camera_bind_group_layout.clone(),
                }])
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

        let right_mesh = render_module_markers
            .add_mesh(
                &device,
                &queue,
                MeshInitData {
                    vertices: MARKER_VERTICES([1.0, 0.0, 0.0].into()),
                    indices: MARKER_INDICES.to_vec(),
                },
            )
            .unwrap();
        let up_mesh = render_module_markers
            .add_mesh(
                &device,
                &queue,
                MeshInitData {
                    vertices: MARKER_VERTICES([0.0, 1.0, 0.0].into()),
                    indices: MARKER_INDICES.to_vec(),
                },
            )
            .unwrap();
        let forward_mesh = render_module_markers
            .add_mesh(
                &device,
                &queue,
                MeshInitData {
                    vertices: MARKER_VERTICES([0.0, 0.0, 1.0].into()),
                    indices: MARKER_INDICES.to_vec(),
                },
            )
            .unwrap();

        render_module_markers
            .upsert_instances(&vec![
                MarkerEntity {
                    position: Vector3::zeros(),
                    direction: Vector3::new(1.0, 0.0, 0.0),
                    color: Vector3::new(1.0, 0.0, 0.0),
                    id: 0,
                    mesh_id: right_mesh,
                },
                MarkerEntity {
                    position: Vector3::zeros(),
                    direction: Vector3::new(0.0, 1.0, 0.0),
                    color: Vector3::new(0.0, 1.0, 0.0),
                    id: 1,
                    mesh_id: up_mesh,
                },
                MarkerEntity {
                    position: Vector3::zeros(),
                    direction: Vector3::new(0.0, 0.0, 1.0),
                    color: Vector3::new(0.0, 0.0, 1.0),
                    id: 2,
                    mesh_id: forward_mesh,
                },
            ])
            .unwrap();
        render_module_markers.update_gpu(&device, &queue);

        let depth_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Depth Bind Group"),
            layout: &depth_texture_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&depth_view),
            }],
        });

        let egui_renderer = EguiRenderer::new(
            &device,
            surface_format,
            RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: true,
                predictable_texture_filtering: false,
            },
            window.clone(),
            |ui, data| {
                let mut metrics_str = String::new();
                let mut debug_str = String::new();
                if let Ok(data) = data.read() {
                    if let Some(cpu) = data.get("cpu") {
                        metrics_str += &format!("CPU Time: {:.2}", cpu.as_f64().unwrap_or(-1.0));
                    }

                    if let Some(gpu) = data.get("gpu") {
                        metrics_str += &format!(" GPU Time: {:.2}", gpu.as_f64().unwrap_or(-1.0))
                    }

                    if let Some(fps) = data.get("fps") {
                        metrics_str += &format!(" FPS: {:.2}", fps.as_f64().unwrap_or(-1.0));
                    }

                    if let Some(anomalies) = data.get("anomalies") {
                        metrics_str += &format!(
                            " Entities with NaN accelerations: {:.2}",
                            anomalies.as_i64().unwrap_or(-1)
                        );
                    }

                    if let Some(up) = data.get("v_up") {
                        if let Some(up) = up.as_array() {
                            debug_str += &format!(
                                " up: ({:.2},{:.2},{:.2})",
                                up[0].as_f64().unwrap_or(-1.0),
                                up[1].as_f64().unwrap_or(-1.0),
                                up[2].as_f64().unwrap_or(-1.0)
                            );
                        }
                    }

                    if let Some(right) = data.get("v_right") {
                        if let Some(right) = right.as_array() {
                            debug_str += &format!(
                                " right: ({:.2},{:.2},{:.2})",
                                right[0].as_f64().unwrap_or(-1.0),
                                right[1].as_f64().unwrap_or(-1.0),
                                right[2].as_f64().unwrap_or(-1.0)
                            );
                        }
                    }

                    if let Some(center) = data.get("v_center") {
                        if let Some(center) = center.as_array() {
                            debug_str += &format!(
                                " center: ({:.2},{:.2},{:.2})",
                                center[0].as_f64().unwrap_or(-1.0),
                                center[1].as_f64().unwrap_or(-1.0),
                                center[2].as_f64().unwrap_or(-1.0)
                            );
                        }
                    }

                    if let Some(position) = data.get("v_position") {
                        if let Some(position) = position.as_array() {
                            debug_str += &format!(
                                " position: ({:.2},{:.2},{:.2})",
                                position[0].as_f64().unwrap_or(-1.0),
                                position[1].as_f64().unwrap_or(-1.0),
                                position[2].as_f64().unwrap_or(-1.0)
                            );
                        }
                    }
                }
                ui.label(RichText::new(metrics_str).color(Color32::from_rgb(0, 0, 0)));
                ui.label(RichText::new(debug_str).color(Color32::from_rgb(0, 0, 0)));
            },
        );

        window.set_visible(true);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,

            render_module_transformed,
            render_module_terrain,
            render_module_markers,

            depth_texture,
            depth_view,
            depth_sampler,
            depth_bind_group,

            lights,

            textures: TextureStorage::new(),
            texture_bind_group_layout,

            camera_bind_group_layout,

            egui_renderer,
        }
    }

    pub fn gui_data(&self) -> Arc<RwLock<HashMap<String, Value>>> {
        self.egui_renderer.data()
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
        mesh: MeshInitData<DefaultVertexType>,
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
            self.render_module_terrain.draw_all(
                &mut render_pass,
                [
                    &state.current_camera().bind_group(),
                    &&self.textures.get(&1).unwrap().3,
                    &self.lights.bind_group(),
                    &&self.depth_bind_group,
                ]
                .iter(),
            );
            self.render_module_transformed.draw_all(
                &mut render_pass,
                [
                    &state.current_camera().bind_group(),
                    &&self.textures.get(&1).unwrap().3,
                    &self.lights.bind_group(),
                    &&self.depth_bind_group,
                ]
                .iter(),
            );
            // Draw markers above everything else
            self.render_module_markers.draw_all(
                &mut render_pass,
                [&state.current_camera().bind_group()].iter(),
            );
        }
        self.egui_renderer.render(
            &self.device,
            &self.queue,
            &ScreenDescriptor {
                size_in_pixels: [view.texture().width(), view.texture().height()],
                pixels_per_point: self.window.scale_factor() as f32 * 1.0,
            },
            &mut encoder,
            &view,
        );
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
