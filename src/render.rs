pub mod camera;
pub mod lights;
pub mod mesh;
pub mod textures;
pub mod vertex;

use std::{f32::consts::PI, slice::Iter, sync::Arc, time::Instant};

use cgmath::{Matrix4, Rad};
use image::DynamicImage;
use log::{error, info};
use wgpu::{
    AddressMode, Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
    CompareFunction, DepthBiasState, DepthStencilState, Device, ExperimentalFeatures, Extent3d,
    Face, Features, FilterMode, FragmentState, FrontFace, IndexFormat, Instance,
    InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    StencilState, StoreOp, Surface, SurfaceConfiguration, SurfaceError, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, Trace, VertexState, wgt::DeviceDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Icon, Window, WindowId},
};

use crate::{
    CHUNK_SIZE_M, METRICS_INTERVAL, MIPMAP_LEVELS,
    assets::ICON,
    core::{InstanceStorage, MeshStorage, MeshStorageError, entity::Entity, world::World},
    input::InputController,
    render::{
        camera::{Camera, Projection},
        textures::{ResizeStrategy, TextureStorage},
        vertex::Vertex,
    },
};

enum AppState {
    /// window width, height, mesh queue, entity queue, texture queue
    NeedsInit(
        u32,
        u32,
        Vec<(String, Vec<Vertex>, Vec<u16>)>,
        Vec<Entity>,
        Vec<(String, DynamicImage, ResizeStrategy)>,
    ),
    Started(Renderer),
}

pub enum Event {
    WindowEvent(WindowId, WindowEvent),
}

pub struct App {
    state: AppState,

    world: World,
    input: InputController,
}

impl App {
    pub fn new(_: &EventLoop<Event>, width: u32, height: u32, seed: u64) -> Self {
        Self {
            state: AppState::NeedsInit(width, height, Vec::new(), Vec::new(), Vec::new()),

            world: World::new(seed),
            input: InputController::new(),
        }
    }

    pub fn add_meshes(&mut self, meshes: Iter<(&str, &[vertex::Vertex], &[u16])>) {
        match &mut self.state {
            AppState::NeedsInit(_, _, mesh_queue, _, _) => {
                for (mesh_id, vertices, indices) in meshes {
                    mesh_queue.push((mesh_id.to_string(), vertices.to_vec(), indices.to_vec()));
                }
            }
            AppState::Started(renderer) => {
                renderer.add_meshes(meshes).unwrap();
            }
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        match &mut self.state {
            AppState::NeedsInit(_, _, _, entity_queue, _) => {
                entity_queue.push(entity);
            }
            AppState::Started(renderer) => {
                self.world.add_entity(entity);
                renderer.upsert_instances(&self.world, true);
            }
        }
    }

    pub fn add_texture(
        &mut self,
        texture_id: String,
        full_size_image: DynamicImage,
        resize_strategy: ResizeStrategy,
    ) {
        match &mut self.state {
            AppState::NeedsInit(_, _, _, _, textures) => {
                textures.push((texture_id, full_size_image, resize_strategy));
            }
            AppState::Started(renderer) => {
                renderer.new_texture(texture_id, full_size_image, resize_strategy);
            }
        }
    }

    /// Loads chunk with (0,0) at (x/16, z/16)
    pub fn load_chunk(&mut self, x: i64, z: i64) {
        let height_map = self.world.request_chunk_exact(x, z);

        self.add_entity(Entity::new(
            &format!("ground_{}.{}", x, z),
            "Flat16",
            [0.0, 0.0, 0.0].into(),
            [0.0, 0.0, 0.0].into(),
            (
                [0.0, 0.0, 0.0].into(),
                [-f32::INFINITY, -f32::INFINITY, -f32::INFINITY].into(),
            ),
            Matrix4 {
                x: [CHUNK_SIZE_M as f32, 0.0, 0.0, 0.0].into(),
                y: [0.0, 1.0, 0.0, 0.0].into(),
                z: [0.0, 0.0, CHUNK_SIZE_M as f32, 0.0].into(),
                w: [
                    x as f32 * CHUNK_SIZE_M as f32,
                    height_map[(0, 0)] as f32,
                    z as f32 * CHUNK_SIZE_M as f32,
                    1.0,
                ]
                .into(),
            },
        ));
    }
}

impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match &mut self.state {
            AppState::NeedsInit(w, h, meshes, entities, textures) => {
                let mut win_attr = Window::default_attributes();
                win_attr.inner_size = Some(Size::Physical(PhysicalSize::new(*w, *h)));
                win_attr.title = "Rover".into();
                win_attr.window_icon = Some(Icon::from_rgba(ICON.to_vec(), 8, 8).unwrap());
                win_attr.visible = false;

                let window = Arc::new(event_loop.create_window(win_attr).unwrap());

                let mut renderer = pollster::block_on(Renderer::new(window.clone()));

                info!("Adding meshes");
                renderer
                    .add_meshes(
                        meshes
                            .iter()
                            .map(|(id, v, i)| (id.as_str(), v.as_slice(), i.as_slice()))
                            .collect::<Vec<(&str, &[Vertex], &[u16])>>()
                            .iter(),
                    )
                    .unwrap();

                // TODO: Use map instead
                info!("Adding entities");
                while let Some(entity) = entities.pop() {
                    self.world.add_entity(entity);
                }
                info!("Creating textures");
                while let Some((texture_id, full_size_image, resize_strategy)) = textures.pop() {
                    renderer.new_texture(texture_id, full_size_image, resize_strategy);
                }
                info!("Creating GPU buffers");
                renderer.upsert_instances(&self.world, true);
                self.state = AppState::Started(renderer);
                window.request_redraw();
            }
            AppState::Started(_) => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let AppState::Started(renderer) = &mut self.state {
            self.input
                .window_event(&event, &renderer.window, &mut renderer.camera);
        }

        match event {
            WindowEvent::Resized(physical_size) => {
                if let AppState::Started(renderer) = &mut self.state {
                    renderer.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Destroyed => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                if let AppState::Started(renderer) = &mut self.state {
                    self.world.update();
                    self.input.update(1.0 / 240.0, &mut renderer.camera);

                    renderer.upsert_instances(&self.world, false);

                    match renderer.render() {
                        Ok(_) => {}
                        Err(e) => error!("{}", e),
                    }
                    renderer.window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct Renderer {
    window: Arc<Window>,

    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,

    render_pipeline: RenderPipeline,
    render_pipeline_layout: PipelineLayout,

    camera: Camera,

    meshes: MeshStorage,

    instances: InstanceStorage,
    ground: InstanceStorage,

    textures: TextureStorage,
    texture_bind_group_layout: BindGroupLayout,

    depth_texture: Texture,
    depth_view: TextureView,
    depth_sampler: Sampler,

    // metrics
    start: Instant,
    n_renders: u64,
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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera = Camera::new(
            &mut device,
            (0.0, 5.0, 10.0).into(),
            Rad(-PI / 4.0),
            Rad(-PI / 12.0),
            Projection::new(
                config.width as f32,
                config.height as f32,
                Rad(PI / 2.0),
                0.1,
                10000.0,
            ),
        );

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
        let size = Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let depth_desc = TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: MIPMAP_LEVELS.len() as u32,
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

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[camera.bind_group_layout(), &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Entity::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
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
        });

        let mesh_storage = MeshStorage::new(&device);
        let instance_storage = InstanceStorage::new(&device);
        let ground = InstanceStorage::new(&device);

        window.set_visible(true);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,

            render_pipeline,
            render_pipeline_layout,

            camera,

            depth_texture,
            depth_view,
            depth_sampler,

            meshes: mesh_storage,
            instances: instance_storage,
            ground,
            textures: TextureStorage::new(),
            texture_bind_group_layout,

            start: Instant::now(),
            n_renders: 0,
        }
    }

    /// Batch adding of meshes. Meshes will be synced to the GPU in this call.
    ///
    /// meshes: (mesh_id, vertices, indices)
    pub fn add_meshes(
        &mut self,
        meshes: Iter<(&str, &[Vertex], &[u16])>,
    ) -> Result<(), MeshStorageError> {
        for (mesh_id, vertices, indices) in meshes {
            if let Err(e) = self.meshes.add_mesh(*mesh_id, *vertices, *indices) {
                return Err(e);
            }
        }

        self.meshes.update_gpu(&mut self.queue, &self.device);

        Ok(())
    }

    pub fn new_texture(
        &mut self,
        texture_id: String,
        full_size_image: DynamicImage,
        resize_strategy: ResizeStrategy,
    ) {
        self.textures.new_texture(
            &mut self.device,
            &mut self.queue,
            texture_id,
            full_size_image,
            resize_strategy,
            &self.texture_bind_group_layout,
        );
    }

    // TODO: All instances get synced even ones not updated, optimize later.

    /// Batch updating of instances. All instances will be synced to the GPU in this call.
    ///
    /// This is the main update function to be called before each render call.
    pub fn upsert_instances(&mut self, world: &World, include_ground: bool) {
        for entity in world.iter_entities() {
            let mesh_id = entity.mesh_id();
            let entity_id = entity.id();
            let transform = entity.model();

            if mesh_id == "Cube" {
                self.instances.upsert_instance(entity_id, transform);
            } else if mesh_id == "Flat16" && include_ground {
                self.ground.upsert_instance(entity_id, transform);
            }
        }

        self.instances.update_gpu(&mut self.queue, &mut self.device);
        self.ground.update_gpu(&mut self.queue, &mut self.device);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
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

        self.camera.update(&mut self.queue);
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);

            render_pass.set_vertex_buffer(0, self.meshes.vertex_slice(..));
            render_pass.set_index_buffer(self.meshes.index_slice(..), IndexFormat::Uint16);
            render_pass.set_bind_group(1, &self.textures.get("test").unwrap().3, &[]);
            if self.instances.len() > 0 {
                render_pass.set_vertex_buffer(1, self.instances.slice(..));
                let (start, end) = self.meshes.get_mesh_index_bounds("Cube").unwrap();
                render_pass.draw_indexed(
                    (*start) as u32..(*end) as u32,
                    0,
                    0..self.instances.len() as u32,
                );
            }
            if self.ground.len() > 0 {
                render_pass.set_vertex_buffer(1, self.ground.slice(..));
                let (start, end) = self.meshes.get_mesh_index_bounds("Flat16").unwrap();

                render_pass.draw_indexed(
                    (*start) as u32..(*end) as u32,
                    0,
                    0..self.ground.len() as u32,
                );
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.n_renders += 1;
        if self.start.elapsed() > METRICS_INTERVAL {
            info!(
                "FPS: {:.2}",
                self.n_renders as f64 / METRICS_INTERVAL.as_secs_f64()
            );
            self.start = Instant::now();
            self.n_renders = 0;
        }
        Ok(())
    }
}
