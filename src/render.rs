pub mod camera;
pub mod data;

use std::{f32::consts::PI, sync::Arc, time::Instant};

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
    window::{Icon, Window, WindowAttributes},
};

use crate::{
    METRICS_INTERVAL,
    assets::ICON,
    core::entity::Entity,
    core::world::World,
    render::{
        camera::{Camera, CameraUniform, Projection},
        data::Vertex,
    },
};

pub struct App {
    proxy: Option<EventLoopProxy<State>>,
    state: Option<State>,
    window_created: bool,
    width: u32,
    height: u32,
    world: Option<World>,
}

impl App {
    pub fn new(event_loop: &EventLoop<State>, width: u32, height: u32, world: World) -> Self {
        Self {
            proxy: Some(event_loop.create_proxy()),
            state: None,
            window_created: false,
            width,
            height,
            world: Some(world),
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self.window_created {
            let mut win_attr = Window::default_attributes();
            win_attr.inner_size = Some(Size::Physical(PhysicalSize::new(self.width, self.height)));
            win_attr.title = "Rover".into();
            win_attr.window_icon = Some(Icon::from_rgba(ICON.to_vec(), 8, 8).unwrap());
            win_attr.visible = false;

            let window = Arc::new(event_loop.create_window(win_attr).unwrap());

            window.request_redraw();

            self.state = Some(pollster::block_on(State::new(
                window.clone(),
                self.world.take().unwrap(),
            )));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size.width, physical_size.height);
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
                        if let Some(state) = &mut self.state {
                            state.camera.translate(&(0.0, 0.0, -1.0).into());
                            state.camera_uniform.update(&state.camera);
                        }
                    }
                    KeyCode::KeyS => {
                        if let Some(state) = &mut self.state {
                            state.camera.translate(&(0.0, 0.0, 1.0).into());
                            state.camera_uniform.update(&state.camera);
                        }
                    }
                    KeyCode::KeyA => {
                        if let Some(state) = &mut self.state {
                            state.camera.translate(&(-1.0, 0.0, 0.0).into());
                            state.camera_uniform.update(&state.camera);
                        }
                    }
                    KeyCode::KeyD => {
                        if let Some(state) = &mut self.state {
                            state.camera.translate(&(1.0, 0.0, 0.0).into());
                            state.camera_uniform.update(&state.camera);
                        }
                    }

                    KeyCode::Space => {
                        if let Some(state) = &mut self.state {
                            state.camera.translate(&(0.0, 1.0, 0.0).into());
                            state.camera_uniform.update(&state.camera);
                        }
                    }

                    KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                        if let Some(state) = &mut self.state {
                            state.camera.translate(&(0.0, -1.0, 0.0).into());
                            state.camera_uniform.update(&state.camera);
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                debug!("Mouse input: {:?}", event)
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(e) => error!("{}", e),
                    }
                    state.window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct State {
    window: Arc<Window>,
    world: World,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,

    render_pipeline: RenderPipeline,
    render_pipeline_layout: PipelineLayout,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    num_indices: u32,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    instances: Vec<(Vec<[[f32; 4]; 4]>, usize, usize, usize)>,
    instance_buffer: Buffer,

    // metrics
    start: Instant,
    n_renders: u64,
}

impl State {
    pub async fn new(window: Arc<Window>, mut world: World) -> Self {
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

        let mut current_index_offset: usize = 0;
        let mut num_indices: u32 = 0;

        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u16> = vec![];

        let mesh = world.get_mesh("Cube").unwrap();
        vertices.extend_from_slice(mesh.vertices());
        indices.extend_from_slice(
            &mesh
                .indices()
                .iter()
                .map(|i| i.clone() + current_index_offset as u16)
                .collect::<Vec<u16>>(),
        );

        current_index_offset += mesh.vertices().len();
        num_indices += mesh.indices().len() as u32;

        let mesh = world.get_mesh("Flat16").unwrap();
        vertices.extend_from_slice(mesh.vertices());
        indices.extend_from_slice(
            &mesh
                .indices()
                .iter()
                .map(|i| i.clone() + current_index_offset as u16)
                .collect::<Vec<u16>>(),
        );

        current_index_offset += mesh.vertices().len();
        num_indices += mesh.indices().len() as u32;

        error!("nm indices {}", num_indices);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let instances = world.instances_to_draw();

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(
                &(instances
                    .iter()
                    .map(|a| a.0.clone())
                    .collect::<Vec<Vec<[[f32; 4]; 4]>>>()
                    .into_iter()
                    .flatten()
                    .collect::<Vec<[[f32; 4]; 4]>>()),
            ),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        window.set_visible(true);

        Self {
            window,
            world,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,

            render_pipeline,
            render_pipeline_layout,

            vertex_buffer,
            index_buffer,
            num_indices,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            instances,
            instance_buffer,

            start: Instant::now(),
            n_renders: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.is_surface_configured = true;
    }

    pub fn update(&mut self) {
        self.world.tick(1.0 / 240.0);

        // TODO: Extremely inefficient
        let target = self
            .world
            .iter_entities()
            .nth(0)
            .unwrap()
            .position()
            .clone();

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(
                &(self
                    .world
                    .instances_to_draw()
                    .iter()
                    .map(|a| a.0.clone())
                    .collect::<Vec<Vec<[[f32; 4]; 4]>>>()
                    .into_iter()
                    .flatten()
                    .collect::<Vec<[[f32; 4]; 4]>>()),
            ),
        );
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

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            for (_, num, start, end) in &self.instances {
                render_pass.draw_indexed((*start) as u32..(*end) as u32, 0, 0..(*num) as u32);
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
