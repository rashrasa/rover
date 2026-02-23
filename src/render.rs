pub mod instance;
pub mod mesh;
pub mod shader;
pub mod textures;
pub mod vertex;

use std::{
    fs::File,
    sync::Arc,
    time::{Duration, Instant},
};

use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use log::{debug, error, info};
use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use rayon::iter::IntoParallelRefMutIterator;
use rodio::{Decoder, OutputStream, Sink};
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
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Icon, Window, WindowId},
};

use crate::{
    core::{
        AfterRenderArgs, AfterTickArgs, BeforeInputArgs, BeforeRenderArgs, BeforeStartArgs,
        BeforeTickArgs, HandleInputArgs, HandleTickArgs, METRICS_INTERVAL, RENDER_DISTANCE, System,
        assets::ICON,
        camera::{Camera, NoClipCamera, Projection},
        entity::{self, BoundingBox, CollisionResponse, Entity, EntityType},
        input::InputController,
        lights::LightSourceStorage,
        prefabs::DEFAULT_SYSTEMS,
        world::terrain::World,
    },
    render::{
        shader::{InstancedRenderModule, RenderPipelineSpec, ShaderSpec, UniformSpec, VertexSpec},
        textures::{ResizeStrategy, TextureStorage},
        vertex::Vertex,
    },
};

pub struct AppInitData {
    pub width: u32,
    pub height: u32,
    pub transform_meshes: Vec<MeshInitData<Vertex>>,
    pub textures: Vec<TextureInitData>,
    pub players: Vec<PlayerInitData>,
    pub objects: Vec<ObjectInitData>,
}

impl AppInitData {
    pub fn inner(
        self,
    ) -> (
        (u32, u32),
        Vec<MeshInitData<Vertex>>,
        Vec<PlayerInitData>,
        Vec<TextureInitData>,
        Vec<ObjectInitData>,
    ) {
        (
            (self.width, self.height),
            self.transform_meshes,
            self.players,
            self.textures,
            self.objects,
        )
    }
}

pub struct MeshInitData<V>
where
    V: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    pub id: u64,
    pub vertices: Vec<V>,
    pub indices: Vec<u16>,
}

pub struct ObjectInitData {
    pub id: u64,
    pub mesh_id: u64,
    pub texture_id: u64,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub bounding_box: BoundingBox,
    pub scale: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub translation: Vector3<f32>,
    pub response: CollisionResponse,
    pub mass: f32,
}

pub struct PlayerInitData {
    pub id: u64,
    pub mesh_id: u64,
    pub texture_id: u64,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub bounding_box: BoundingBox,
    pub scale: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub translation: Vector3<f32>,
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
    current_camera: NoClipCamera,
    entities: Vec<Entity>,
}

impl ActiveState {
    fn update(&mut self, elapsed: f32, world: &mut World) {
        let pos = self.current_camera.position();
        world.load((pos[0], pos[2]), RENDER_DISTANCE);
    }
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

// TODO: Global ID system for objects, meshes, textures, etc.

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

    systems: Vec<Box<dyn System>>,
}

impl App {
    pub fn new(_: &EventLoop<Event>, width: u32, height: u32, seed: u64) -> Self {
        Self {
            state: AppState::NeedsInit(AppInitData {
                width,
                height,
                transform_meshes: vec![],
                players: vec![],
                objects: vec![],
                textures: vec![],
            }),
            world: World::new(seed),
            input: InputController::new(),
            systems: DEFAULT_SYSTEMS(),
        }
    }

    /// "Vertex" is the most common vertex type to be used in most cases and
    /// is the only one available to add for now.
    pub fn add_meshes(&mut self, mut meshes: Vec<MeshInitData<Vertex>>) -> Vec<usize> {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                let mut ids = vec![];
                while meshes.len() > 0 {
                    let data = meshes.remove(0);
                    ids.push(init_data.transform_meshes.len());
                    init_data.transform_meshes.push(data);
                }
                ids
            }
            AppState::Started { renderer, state: _ } => {
                return renderer
                    .render_module_transformed
                    .add_meshes(&renderer.device, &renderer.queue, meshes)
                    .unwrap();
            }
        }
    }

    pub fn add_player(&mut self, player: PlayerInitData) {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                init_data.players.push(player);
            }
            AppState::Started { renderer, state } => {
                let player = Entity::new(
                    player.id,
                    player.mesh_id,
                    player.texture_id,
                    player.scale,
                    player.rotation,
                    player.translation,
                    player.velocity,
                    player.acceleration,
                    player.bounding_box,
                    EntityType::Player {
                        camera: NoClipCamera::new(
                            &mut renderer.device,
                            &renderer.camera_bind_group_layout,
                            player.translation,
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
                    },
                    player.response,
                    player.mass,
                );
                renderer
                    .render_module_transformed
                    .upsert_instances(std::iter::once(&player))
                    .unwrap();
                state.entities.push(player);
            }
        }
    }

    pub fn add_object(&mut self, object: ObjectInitData) {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                init_data.objects.push(object);
            }
            AppState::Started { renderer, state } => {
                let object = Entity::new(
                    object.id,
                    object.mesh_id,
                    object.texture_id,
                    object.scale,
                    object.rotation,
                    object.translation,
                    object.velocity,
                    object.acceleration,
                    object.bounding_box,
                    EntityType::Object,
                    object.response,
                    object.mass,
                );
                renderer
                    .render_module_transformed
                    .upsert_instances(std::iter::once(&object))
                    .unwrap();
                state.entities.push(object);
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
}

impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let AppState::NeedsInit(data) = &mut self.state {
            let mut old_data = AppInitData {
                width: 0,
                height: 0,
                transform_meshes: vec![],
                players: vec![],
                objects: vec![],
                textures: vec![],
            };
            std::mem::swap(&mut old_data, data);
            let (size, mut meshes, mut players_init, mut textures, mut objects_init) =
                old_data.inner();
            let mut win_attr = Window::default_attributes();
            win_attr.inner_size = Some(Size::Physical(PhysicalSize::new(size.0, size.1)));
            win_attr.title = "Rover".into();
            win_attr.window_icon = Some(Icon::from_rgba(ICON.to_vec(), 8, 8).unwrap());
            win_attr.visible = false;

            let window = Arc::new(event_loop.create_window(win_attr).unwrap());

            let mut renderer = pollster::block_on(Renderer::new(window.clone()));
            {
                let args = BeforeStartArgs {
                    renderer: &renderer,
                };
                for system in self.systems.iter_mut() {
                    system.before_start(&args);
                }
            }

            info!("Adding meshes");
            renderer
                .render_module_transformed
                .add_meshes(&renderer.device, &renderer.queue, meshes)
                .unwrap();

            info!("Adding entities");
            let mut entities = vec![];
            while players_init.len() > 0 {
                let entity = players_init.remove(0);
                let player = Entity::new(
                    entity.id,
                    entity.mesh_id,
                    entity.texture_id,
                    entity.scale,
                    entity.rotation,
                    entity.translation,
                    entity.velocity,
                    entity.acceleration,
                    entity.bounding_box,
                    EntityType::Player {
                        camera: NoClipCamera::new(
                            &mut renderer.device,
                            &renderer.camera_bind_group_layout,
                            entity.translation,
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
                    },
                    entity.response,
                    entity.mass,
                );
                entities.push(player);
            }

            while objects_init.len() > 0 {
                let object_init = objects_init.remove(0);
                let object = Entity::new(
                    object_init.id,
                    object_init.mesh_id,
                    object_init.texture_id,
                    object_init.scale,
                    object_init.rotation,
                    object_init.translation,
                    object_init.velocity,
                    object_init.acceleration,
                    object_init.bounding_box,
                    EntityType::Object,
                    object_init.response,
                    object_init.mass,
                );

                entities.push(object);
            }

            info!("Creating textures");
            while textures.len() > 0 {
                let data = textures.remove(0);
                renderer.new_texture(TextureInitData {
                    id: data.id,
                    image: data.image,
                    resize: data.resize,
                });
            }

            info!("Creating GPU buffers");
            let mut active_state = ActiveState {
                current_camera: NoClipCamera::new(
                    &mut renderer.device,
                    &renderer.camera_bind_group_layout,
                    Vector3::identity(),
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
                entities,
            };

            renderer.update_instances(&mut active_state);

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
                .window_event(&event, &renderer.window, &mut state.current_camera);
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
                    let elapsed_dur = renderer.last_update.elapsed();
                    let elapsed = elapsed_dur.as_secs_f32();
                    renderer.last_update = Instant::now();
                    let start = Instant::now();

                    // start redraw
                    {
                        let before_input = BeforeInputArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.before_input(&before_input);
                        }
                    }
                    self.input
                        .update(elapsed, &mut state.current_camera, &mut renderer.sink);
                    {
                        let handle_input = HandleInputArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.handle_input(&handle_input);
                        }
                    }

                    {
                        let before_tick = BeforeTickArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.before_tick(&before_tick);
                        }
                    }

                    {
                        let handle_tick = HandleTickArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.handle_tick(&handle_tick);
                        }
                    }

                    {
                        let after_tick = AfterTickArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.after_tick(&after_tick);
                        }
                    }

                    state.update(elapsed, &mut self.world);

                    {
                        let before_render = BeforeRenderArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.before_render(&before_render);
                        }
                    }

                    state.current_camera.update_gpu(&mut renderer.queue);
                    renderer.update_instances(state);

                    renderer.t_ticking += start.elapsed();
                    renderer.n_ticks += 1;

                    match renderer.render(state) {
                        Ok(_) => {}
                        Err(e) => error!("{}", e),
                    }

                    {
                        let after_render = AfterRenderArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.after_render(&after_render);
                        }
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

    render_module_transformed: InstancedRenderModule<Vertex, [[f32; 4]; 4]>,

    textures: TextureStorage,
    texture_bind_group_layout: BindGroupLayout,
    camera_bind_group_layout: BindGroupLayout,

    lights: LightSourceStorage,

    depth_texture: Texture,
    depth_view: TextureView,
    depth_sampler: Sampler,
    depth_bind_group: BindGroup,

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
            Some("Transformed"),
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

        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        sink.pause();
        if crate::core::MUTE {
            sink.set_volume(0.0);
        } else {
            sink.set_volume(0.2);
        }
        // sink.append(Decoder::try_from(File::open("assets/engine.wav").unwrap()).unwrap());
        // Should be own system

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

            last_update: Instant::now(),

            sink,
            stream_handle,

            start: Instant::now(),
            n_renders: 0,
            t_ticking: Duration::ZERO,
            n_ticks: 0,
        }
    }

    pub fn update_instances(&mut self, active_state: &mut ActiveState) {
        self.render_module_transformed
            .upsert_instances(active_state.entities.iter())
            .unwrap();

        self.render_module_transformed
            .update_gpu(&self.device, &self.queue);
    }

    pub fn new_texture(&mut self, data: TextureInitData) {
        self.textures.new_texture(
            &mut self.device,
            &mut self.queue,
            data.image,
            data.resize,
            &self.texture_bind_group_layout,
        );
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

            self.render_module_transformed.draw_all(
                &mut render_pass,
                [
                    &state.current_camera.bind_group(),
                    &&self.textures.get(&0).unwrap().3,
                    &self.lights.bind_group(),
                    &&self.depth_bind_group,
                ]
                .iter(),
            );
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        // sample every 1000 renders for GPU time
        if self.n_renders % 1000 == 0 {
            let start = Instant::now();
            self.queue.on_submitted_work_done(move || {
                debug!("GPU time: {} ms", start.elapsed().as_secs_f64() * 1000.0);
            });
        }

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
