use log::{info, warn};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize, Size},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Icon, Window, WindowAttributes},
};

use crate::{assets::ICON, world::World};

pub struct App {
    world: World,
    width: u32,
    height: u32,
    window: Option<Window>,
}

impl App {
    pub fn new(world: World, width: u32, height: u32) -> Self {
        Self {
            world,
            width,
            height,
            window: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let mut win_attr = Window::default_attributes();
            win_attr.inner_size = Some(Size::Physical(PhysicalSize::new(self.width, self.height)));
            win_attr.title = "Rover".into();
            win_attr.window_icon = Some(Icon::from_rgba(ICON.to_vec(), 8, 8).unwrap());

            self.window = Some(event_loop.create_window(win_attr).unwrap());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::Resized(physical_size) => {
                self.width = physical_size.width;
                self.height = physical_size.height;
            }
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Destroyed => event_loop.exit(),
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                info!("Key pressed: {:?}", event)
            }
            winit::event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                info!("Mouse input: {:?}", event)
            }
            _ => {}
        }
    }
}

pub struct Renderer;

impl Renderer {
    pub fn start(world: World, width: u32, height: u32) {
        EventLoop::new()
            .unwrap()
            .run_app(&mut App::new(world, width, height))
            .unwrap();
    }
}
