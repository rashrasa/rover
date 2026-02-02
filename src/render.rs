pub mod textures;
pub mod vertex;

use std::{
    cell::RefCell,
    collections::HashMap,
    f32::consts::PI,
    fs::File,
    rc::Rc,
    slice::Iter,
    sync::Arc,
    time::{Duration, Instant},
};

use cgmath::{Deg, Rad};
use image::DynamicImage;
use log::{error, info};
use nalgebra::{Matrix4, Vector3};
use rodio::{Decoder, OutputStream, Sink};
use wgpu::{
    AddressMode, Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, BufferBindingType, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Device,
    ExperimentalFeatures, Extent3d, Face, Features, FilterMode, FragmentState, FrontFace,
    IndexFormat, Instance, InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType,
    SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StencilState, StoreOp,
    Surface, SurfaceConfiguration, SurfaceError, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, Trace, VertexState, wgt::DeviceDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Icon, Window, WindowId},
};

use crate::{
    IDBank, MESH_FLAT16, METRICS_INTERVAL,
    core::{
        assets::ICON,
        camera::{Camera, NoClipCamera, Projection},
        entity::{self, BoundingBox, CollisionResponse, Transform, player::Player},
        instance::InstanceStorage,
        lights::LightSourceStorage,
        mesh::{MeshStorage, MeshStorageError},
        world::World,
    },
    input::InputController,
    render::{
        textures::{ResizeStrategy, TextureStorage},
        vertex::Vertex,
    },
};

pub struct AppInitData {
    pub width: u32,
    pub height: u32,
    pub meshes: Vec<MeshInitData>,
    pub players: Vec<PlayerInitData>,
    pub textures: Vec<TextureInitData>,
}

impl AppInitData {
    pub fn inner(
        self,
    ) -> (
        (u32, u32),
        Vec<MeshInitData>,
        Vec<PlayerInitData>,
        Vec<TextureInitData>,
    ) {
        (
            (self.width, self.height),
            self.meshes,
            self.players,
            self.textures,
        )
    }
}

pub struct MeshInitData {
    pub id: u64,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub struct PlayerInitData {
    pub id: u64,
    pub mesh_id: u64,
    pub texture_id: u64,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub bounding_box: BoundingBox,
    pub model: Matrix4<f32>,
    pub response: CollisionResponse,
    pub mass: f32,
}

pub struct TextureInitData {
    pub id: u64,
    pub image: DynamicImage,
    pub resize: ResizeStrategy,
}

// Data only available once the window and renderer are created.
pub struct ActiveState {
    current_player: Player,
    players: Vec<Player>,
}

enum AppState {
    NeedsInit(
        // Data temporarily stored before the app starts.
        AppInitData,
    ),
    Started {
        // Data available once the window is created.
        renderer: Renderer,
        state: ActiveState,
    },
}

pub enum Event {
    WindowEvent(WindowId, WindowEvent),
}

/// Main struct for the entire app.
///
/// Contains all communications between:
///     - World
///     - Renderer
///     - Input
///     - Window
pub struct App {
    // Always available fields
    state: AppState,
    world: World,
    input: InputController,
}

impl App {
    pub fn new(_: &EventLoop<Event>, width: u32, height: u32, seed: u64) -> Self {
        Self {
            state: AppState::NeedsInit(AppInitData {
                width,
                height,
                meshes: Vec::new(),
                players: Vec::new(),
                textures: Vec::new(),
            }),
            world: World::new(seed),
            input: InputController::new(),
        }
    }

    pub fn add_meshes(&mut self, mut meshes: Vec<MeshInitData>) {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                while let Some(data) = meshes.pop() {
                    init_data.meshes.push(data);
                }
            }
            AppState::Started { renderer, state: _ } => {
                renderer.add_meshes(meshes).unwrap();
            }
        }
    }

    pub fn add_player(&mut self, player: PlayerInitData) {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                init_data.players.push(player);
            }
            AppState::Started { renderer, state } => {
                let player = Player::new(
                    player.id,
                    player.mesh_id,
                    player.texture_id,
                    player.velocity,
                    player.acceleration,
                    player.bounding_box,
                    player.model,
                    NoClipCamera::new(
                        &mut renderer.device,
                        &renderer.camera_bind_group_layout,
                        player.model.column(3).xyz(),
                        0.0,
                        0.0,
                        0.0,
                        Projection::new(
                            renderer.config.width as f32,
                            renderer.config.height as f32,
                            90.0,
                            0.1,
                            10000.0,
                        ),
                    ),
                    player.response,
                    player.mass,
                );
                state.players.push(player);
                renderer.insert_instances(state).unwrap();
            }
        }
    }

    pub fn add_texture(&mut self, data: TextureInitData) {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                init_data.textures.push(data);
            }
            AppState::Started { renderer, state: _ } => {
                renderer.new_texture(data);
            }
        }
    }

    /// Loads chunk with (0,0) at (x/16, z/16)
    pub fn load_chunk(&mut self, x: i64, z: i64, id: &mut IDBank) {
        let height_map = self.world.request_chunk_exact(x, z);

        // TODO: Create Terrain
        //todo!();
    }
}

impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let AppState::NeedsInit(data) = &mut self.state {
            let mut old_data = AppInitData {
                width: 0,
                height: 0,
                meshes: vec![],
                players: vec![],
                textures: vec![],
            };
            std::mem::swap(&mut old_data, data);
            let (size, mut meshes, mut players_init, mut textures) = old_data.inner();
            let mut win_attr = Window::default_attributes();
            win_attr.inner_size = Some(Size::Physical(PhysicalSize::new(size.0, size.1)));
            win_attr.title = "Rover".into();
            win_attr.window_icon = Some(Icon::from_rgba(ICON.to_vec(), 8, 8).unwrap());
            win_attr.visible = false;

            let window = Arc::new(event_loop.create_window(win_attr).unwrap());

            let mut renderer = pollster::block_on(Renderer::new(window.clone()));

            info!("Adding meshes");
            renderer.add_meshes(meshes);

            info!("Adding entities");
            let mut players = vec![];
            while let Some(entity) = players_init.pop() {
                let player = Player::new(
                    entity.id,
                    entity.mesh_id,
                    entity.texture_id,
                    entity.velocity,
                    entity.acceleration,
                    entity.bounding_box,
                    entity.model,
                    NoClipCamera::new(
                        &mut renderer.device,
                        &renderer.camera_bind_group_layout,
                        entity.model.column(3).xyz(),
                        0.0,
                        0.0,
                        0.0,
                        Projection::new(
                            renderer.config.width as f32,
                            renderer.config.height as f32,
                            90.0,
                            0.1,
                            10000.0,
                        ),
                    ),
                    entity.response,
                    entity.mass,
                );
                players.push(player);
            }
            info!("Creating textures");
            while let Some(data) = textures.pop() {
                renderer.new_texture(TextureInitData {
                    id: data.id,
                    image: data.image,
                    resize: data.resize,
                });
            }
            info!("Creating GPU buffers");
            let mut active_state = ActiveState {
                current_player: players.pop().unwrap(),
                players,
            };

            renderer.insert_instances(&mut active_state).unwrap();

            self.state = AppState::Started {
                renderer,
                state: active_state,
            };

            window.request_redraw();

            info!("Started! Use WASD for movement and Left Control for speed");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let AppState::Started { renderer, state } = &mut self.state {
            self.input
                .window_event(&event, &renderer.window, &mut state.current_player);
        }

        match event {
            WindowEvent::Resized(physical_size) => {
                if let AppState::Started { renderer, state: _ } = &mut self.state {
                    renderer.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Destroyed => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                if let AppState::Started { renderer, state } = &mut self.state {
                    let elapsed = renderer.last_update.elapsed().as_secs_f32();
                    renderer.last_update = Instant::now();
                    let start = Instant::now();

                    for player in state.players.iter_mut() {
                        entity::tick(player, elapsed);
                    }
                    entity::tick(&mut state.current_player, elapsed);

                    self.input
                        .update(elapsed, &mut state.current_player, &mut renderer.sink);

                    state.current_player.update_gpu(&mut renderer.queue);
                    renderer.update_instances(state).unwrap();

                    renderer.t_ticking += start.elapsed();
                    renderer.n_ticks += 1;

                    match renderer.render(state) {
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

pub struct Renderer {
    window: Arc<Window>,

    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,

    render_pipeline: RenderPipeline,
    render_pipeline_layout: PipelineLayout,

    meshes: MeshStorage,

    instances: HashMap<u64, InstanceStorage>,

    textures: TextureStorage,
    texture_bind_group_layout: BindGroupLayout,

    camera_bind_group_layout: BindGroupLayout,

    lights: LightSourceStorage,

    depth_texture: Texture,
    depth_view: TextureView,
    depth_sampler: Sampler,

    sink: Sink,
    stream_handle: OutputStream,

    last_update: Instant,

    // metrics
    start: Instant,
    n_renders: u64,
    t_ticking: Duration,
    n_ticks: u64,
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

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

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

        let lights = LightSourceStorage::new(
            &mut device,
            [1000.0, 1000.0, 1000.0, 1.0],
            [252.0 / 255.0, 150.0 / 255.0, 1.0 / 255.0, 1.0],
            1.0e7,
        );

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &texture_bind_group_layout,
                lights.layout(),
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Player::desc()],
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

        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        sink.pause();
        if crate::MUTE {
            sink.set_volume(0.0);
        } else {
            sink.set_volume(0.2);
        }
        sink.append(Decoder::try_from(File::open("assets/engine.wav").unwrap()).unwrap());

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

            depth_texture,
            depth_view,
            depth_sampler,

            lights,

            meshes: mesh_storage,
            instances: HashMap::new(),
            textures: TextureStorage::new(),
            texture_bind_group_layout,

            camera_bind_group_layout,

            last_update: Instant::now(),

            sink,
            stream_handle,

            start: Instant::now(),
            n_renders: 0,
            t_ticking: Duration::ZERO,
            n_ticks: 0,
        }
    }

    /// Batch adding of meshes. Meshes will be synced to the GPU in this call.
    pub fn add_meshes(&mut self, mut meshes: Vec<MeshInitData>) -> Result<(), MeshStorageError> {
        while let Some(data) = meshes.pop() {
            if let Err(e) = self
                .meshes
                .add_mesh(&data.id, &data.vertices, &data.indices)
            {
                return Err(e);
            }
            if let Some(_) = self
                .instances
                .insert(data.id, InstanceStorage::new(&self.device))
            {
                return Err(MeshStorageError::MeshExists);
            }
        }

        self.meshes.update_gpu(&mut self.queue, &self.device);

        Ok(())
    }

    pub fn new_texture(&mut self, data: TextureInitData) {
        self.textures.new_texture(
            &mut self.device,
            &mut self.queue,
            data.id,
            data.image,
            data.resize,
            &self.texture_bind_group_layout,
        );
    }

    pub fn insert_instances(&mut self, state: &mut ActiveState) -> Result<(), String> {
        for entity in state.players.iter() {
            let mesh_id = entity.mesh_id();
            let entity_id = entity.id();
            let transform = entity.transform();

            self.instances.entry(*mesh_id).and_modify(|e| {
                e.upsert_instance(entity_id, transform);
            });
        }

        for (mesh_id, storage) in self.instances.iter_mut() {
            storage.update_gpu(&mut self.queue, &mut self.device);
        }

        Ok(())
    }
    /// Batch updating of instances. All instances will be synced to the GPU in this call.
    ///
    /// This is the main update function to be called before each render call.
    pub fn update_instances(&mut self, state: &ActiveState) -> Result<(), String> {
        for entity in state.players.iter() {
            let mesh_id = entity.mesh_id();
            let entity_id = entity.id();
            let transform = entity.transform();
            if *mesh_id != MESH_FLAT16 {
                self.instances.entry(*mesh_id).and_modify(|e| {
                    e.upsert_instance(entity_id, transform);
                });
            }
        }

        for (mesh_id, storage) in self.instances.iter_mut() {
            if *mesh_id != MESH_FLAT16 {
                storage.update_gpu(&mut self.queue, &mut self.device);
            }
        }

        Ok(())
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

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_vertex_buffer(0, self.meshes.vertex_slice(..));
            render_pass.set_index_buffer(self.meshes.index_slice(..), IndexFormat::Uint16);

            render_pass.set_bind_group(0, state.current_player.bind_group(), &[]);
            render_pass.set_bind_group(1, &self.textures.get(&0).unwrap().3, &[]);
            render_pass.set_bind_group(2, self.lights.bind_group(), &[]);

            for (mesh_id, storage) in self.instances.iter() {
                if storage.len() > 0 {
                    render_pass.set_vertex_buffer(1, storage.slice(..));
                    let (start, end) = self.meshes.get_mesh_index_bounds(mesh_id).unwrap();
                    render_pass.draw_indexed(
                        (*start) as u32..(*end) as u32,
                        0,
                        0..storage.len() as u32,
                    );
                }
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
            info!(
                "Average update/copy duration (ms): {:.4}",
                (self.t_ticking.as_secs_f64() / self.n_ticks as f64) * 1000.0
            );
            self.start = Instant::now();
            self.n_renders = 0;
            self.t_ticking = Duration::ZERO;
            self.n_ticks = 0;
        }
        Ok(())
    }
}
