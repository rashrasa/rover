pub mod camera;
pub mod mesh;
pub mod vertex;

use std::{collections::HashMap, f32::consts::PI, slice::Iter, sync::Arc, time::Instant};

use bytemuck::cast_slice;
use cgmath::{InnerSpace, Matrix4, Rad};
use log::{debug, error, info, warn};
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, ExperimentalFeatures, Face,
    Features, FragmentState, FrontFace, IndexFormat, Instance, InstanceDescriptor, Limits, LoadOp,
    MultisampleState, Operations, PipelineCompilationOptions, PipelineLayout,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages,
    TextureViewDescriptor, Trace, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::DeviceDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize, Size},
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    window::{Icon, Window, WindowAttributes, WindowId},
};

use crate::{
    METRICS_INTERVAL,
    assets::ICON,
    core::{InstanceStorage, MeshStorage, MeshStorageError, entity::Entity, world::World},
    render::{
        camera::{Camera, CameraUniform, Projection},
        mesh::Mesh,
        vertex::Vertex,
    },
};

enum AppState {
    NeedsInit(u32, u32, Vec<(String, Vec<Vertex>, Vec<u16>)>, Vec<Entity>), // window width, height, meshes to push, entities to push
    Started(Renderer),
}

pub enum Event {
    WindowEvent(WindowId, WindowEvent),
}

pub struct App {
    state: AppState,

    proxy: EventLoopProxy<Event>,
    world: World,
}

impl App {
    pub fn new(event_loop: &EventLoop<Event>, width: u32, height: u32, seed: u64) -> Self {
        Self {
            proxy: event_loop.create_proxy(),
            state: AppState::NeedsInit(width, height, Vec::new(), Vec::new()),

            world: World::new(seed),
        }
    }

    pub fn add_meshes(&mut self, meshes: Iter<(&str, &[vertex::Vertex], &[u16])>) {
        match &mut self.state {
            AppState::NeedsInit(_, _, mesh_queue, _) => {
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
            AppState::NeedsInit(_, _, _, entity_queue) => {
                entity_queue.push(entity);
            }
            AppState::Started(renderer) => {
                renderer.upsert_instances([(entity.id(), entity.model())].iter());
                self.world.add_entity(entity);
            }
        }
    }
}

impl ApplicationHandler<Event> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match &mut self.state {
            AppState::NeedsInit(w, h, meshes, entities) => {
                let mut win_attr = Window::default_attributes();
                win_attr.inner_size = Some(Size::Physical(PhysicalSize::new(*w, *h)));
                win_attr.title = "Rover".into();
                win_attr.window_icon = Some(Icon::from_rgba(ICON.to_vec(), 8, 8).unwrap());
                win_attr.visible = false;

                let window = Arc::new(event_loop.create_window(win_attr).unwrap());

                window.request_redraw();
                let mut renderer = pollster::block_on(Renderer::new(window.clone()));

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
                while let Some(entity) = entities.pop() {
                    renderer.upsert_instances([(entity.id(), entity.model())].iter());
                    self.world.add_entity(entity);
                }
                self.state = AppState::Started(renderer);
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
        match event {
            WindowEvent::Resized(physical_size) => {
                if let AppState::Started(renderer) = &mut self.state {
                    renderer.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => match event.physical_key {
                PhysicalKey::Code(k) => match k {
                    KeyCode::KeyW => {
                        if let AppState::Started(renderer) = &mut self.state {
                            renderer.camera.translate(&(0.0, 0.0, -1.0).into());
                            renderer.camera_uniform.update(&renderer.camera);
                        }
                    }
                    KeyCode::KeyS => {
                        if let AppState::Started(renderer) = &mut self.state {
                            renderer.camera.translate(&(0.0, 0.0, 1.0).into());
                            renderer.camera_uniform.update(&renderer.camera);
                        }
                    }
                    KeyCode::KeyA => {
                        if let AppState::Started(renderer) = &mut self.state {
                            renderer.camera.translate(&(-1.0, 0.0, 0.0).into());
                            renderer.camera_uniform.update(&renderer.camera);
                        }
                    }
                    KeyCode::KeyD => {
                        if let AppState::Started(renderer) = &mut self.state {
                            renderer.camera.translate(&(1.0, 0.0, 0.0).into());
                            renderer.camera_uniform.update(&renderer.camera);
                        }
                    }

                    KeyCode::Space => {
                        if let AppState::Started(renderer) = &mut self.state {
                            renderer.camera.translate(&(0.0, 1.0, 0.0).into());
                            renderer.camera_uniform.update(&renderer.camera);
                        }
                    }

                    KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                        if let AppState::Started(renderer) = &mut self.state {
                            renderer.camera.translate(&(0.0, -1.0, 0.0).into());
                            renderer.camera_uniform.update(&renderer.camera);
                        }
                    }
                    _ => {}
                },
                _ => {}
            },

            WindowEvent::RedrawRequested => {
                if let AppState::Started(renderer) = &mut self.state {
                    self.world.update();

                    match renderer.render(self.world.iter_entities()) {
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
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    meshes: MeshStorage,
    instances: InstanceStorage,

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

        let (device, queue) = adapter
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
            (0.0, 5.0, 10.0).into(),
            Rad(0.0),
            Rad(0.0),
            Projection::new(
                config.width as f32,
                config.height as f32,
                Rad(PI / 2.0),
                0.1,
                1000.0,
            ),
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
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
            depth_stencil: None,
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
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            meshes: mesh_storage,
            instances: instance_storage,

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

    // TODO: All instances get synced even ones not updated, optimize later.

    /// Batch updating of instances. All instances will be synced to the GPU in this call.
    ///
    /// This is the main update function to be called before each render call.
    ///
    /// instances: (entity_id, transform)
    pub fn upsert_instances(&mut self, instances: Iter<(&str, &Matrix4<f32>)>) {
        for (entity_id, transform) in instances {
            self.instances.upsert_instance(entity_id, transform);
        }

        self.instances.update_gpu(&mut self.queue);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    pub fn render(&mut self, entities: &Vec<Entity>) -> Result<(), SurfaceError> {
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
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.meshes.vertex_slice(..));

            render_pass.set_vertex_buffer(1, self.instances.slice(..));
            render_pass.set_index_buffer(self.meshes.index_slice(..), IndexFormat::Uint16);

            for entity in entities {
                let (start, end) = self.meshes.get_mesh_index_bounds(entity.mesh_id()).unwrap();
                let i = *self.instances.get_instance(entity.id()).unwrap();
                render_pass.draw_indexed(
                    (*start) as u32..(*end) as u32,
                    0,
                    i as u32..(i as u32 + 1),
                );
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.n_renders += 1;
        if self.start.elapsed() > METRICS_INTERVAL {
            info!("FPS: {}", self.n_renders);
            self.start = Instant::now();
            self.n_renders = 0;
        }
        Ok(())
    }
}
