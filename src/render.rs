use std::sync::Arc;

use log::{info, warn};
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, Device, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, Trace,
    wgt::DeviceDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Icon, Window, WindowAttributes},
};

use crate::{assets::ICON, world::World};

#[derive(Debug)]
pub struct State {
    window: Arc<Window>,
    world: World,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,
}

impl State {
    pub async fn new(window: Arc<Window>, world: World) -> Self {
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
        Self {
            window,
            world,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
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
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        self.window.request_redraw();

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
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

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

            let window = Arc::new(event_loop.create_window(win_attr).unwrap());
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
                self.width = physical_size.width;
                self.height = physical_size.height;
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                info!("Key pressed: {:?}", event)
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                info!("Mouse input: {:?}", event)
            }
            WindowEvent::RedrawRequested => {}
            _ => {}
        }
    }
}
