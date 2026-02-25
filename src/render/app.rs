use std::{sync::Arc, time::Instant};

use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use log::{error, info};
use nalgebra::{UnitQuaternion, Vector3};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Icon, Window, WindowId},
};

use crate::{
    core::{
        AfterRenderArgs, AfterTickArgs, BeforeInputArgs, BeforeRenderArgs, BeforeStartArgs,
        BeforeTickArgs, Completer, DisposeArgs, HandleInputArgs, HandleTickArgs, RENDER_DISTANCE,
        System,
        assets::ICON,
        camera::{NoClipCamera, Projection},
        entity::{BoundingBox, CollisionResponse, Entity, EntityType},
        input::InputController,
        prefabs::DEFAULT_SYSTEMS,
        world::terrain::World,
    },
    render::{
        mesh::MeshStorageError, renderer::Renderer, textures::ResizeStrategy, vertex::Vertex,
    },
};

const APP_START_PRECOND: Option<&str> = Some("App is started and renderer is available.");

pub struct AppInitData<'a> {
    pub width: u32,
    pub height: u32,
    pub transform_meshes: Vec<(Completer<'a, u64>, MeshInitData<Vertex>)>,
    pub textures: Vec<(Completer<'a, u64>, TextureInitData)>,
    pub players: Vec<(Completer<'a, u64>, PlayerInitData<'a>)>,
    pub objects: Vec<(Completer<'a, u64>, ObjectInitData<'a>)>,
}

impl<'a> AppInitData<'a> {
    pub fn inner(
        self,
    ) -> (
        (u32, u32),
        Vec<(Completer<'a, u64>, MeshInitData<Vertex>)>,
        Vec<(Completer<'a, u64>, PlayerInitData<'a>)>,
        Vec<(Completer<'a, u64>, TextureInitData)>,
        Vec<(Completer<'a, u64>, ObjectInitData<'a>)>,
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
    pub vertices: Vec<V>,
    pub indices: Vec<u16>,
}

pub struct ObjectInitData<'a> {
    pub mesh_id: Completer<'a, u64>,
    pub texture_id: Completer<'a, u64>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub bounding_box: BoundingBox,
    pub scale: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub translation: Vector3<f32>,
    pub response: CollisionResponse,
    pub mass: f32,
}

pub struct PlayerInitData<'a> {
    pub mesh_id: Completer<'a, u64>,
    pub texture_id: Completer<'a, u64>,
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
    pub image: DynamicImage,
    pub resize: ResizeStrategy,
}

// Data only available once the window and renderer are created.
pub struct ActiveState {
    current_camera: NoClipCamera,
    entities: Vec<Entity>,

    last_update: Instant,
}

impl ActiveState {
    pub fn update(&mut self, _elapsed: f32, world: &mut World) {
        let pos = self.current_camera.position();
        world.load((pos[0], pos[2]), RENDER_DISTANCE);
    }

    pub fn current_camera(&self) -> &NoClipCamera {
        &self.current_camera
    }

    pub fn current_camera_mut(&mut self) -> &mut NoClipCamera {
        &mut self.current_camera
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }
}

enum AppState<'a> {
    NeedsInit(
        // Data temporarily stored before the app starts.
        AppInitData<'a>,
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
/// App needs to be started with:
///
/// ```rust
///     use agate_engine::render::app::App;
///     let mut app = App::new(1920, 1080, 0);
///     App::start(&mut app);
/// ```
///
/// Contains all communications between:
///     - World
///     - Renderer
///     - Input
///     - Window
pub struct App<'a> {
    // Always available fields
    state: AppState<'a>,
    world: World,
    input: InputController,

    systems: Vec<Box<dyn System>>,
}

impl App<'_> {
    // static method
    pub fn start(app: &mut Self) {
        let event_loop: EventLoop<Event> = EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(app).unwrap();
    }
}

impl<'a> App<'a> {
    pub fn new(width: u32, height: u32, seed: u64) -> Self {
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
    ///
    /// Returns a Completer which resolves to a mesh id.
    pub fn add_mesh(
        &mut self,
        mesh: MeshInitData<Vertex>,
    ) -> Result<Completer<'static, u64>, MeshStorageError> {
        let completer = Completer::new(APP_START_PRECOND);
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                init_data.transform_meshes.push((completer.clone(), mesh));

                Ok(completer)
            }
            AppState::Started {
                renderer, state: _, ..
            } => {
                let mesh_id = renderer.add_mesh_instanced(mesh)?;
                Ok(Completer::from_value(mesh_id))
            }
        }
    }

    pub fn add_player(&mut self, player: PlayerInitData<'a>) -> Completer<'static, u64> {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                let completer = Completer::new(APP_START_PRECOND);
                init_data.players.push((completer.clone(), player));
                completer
            }
            AppState::Started {
                renderer, state, ..
            } => {
                let id = state.entities.len() as u64;
                let player = Entity::new(
                    id,
                    player.mesh_id.consume().unwrap(),
                    player.texture_id.consume().unwrap(),
                    player.scale,
                    player.rotation,
                    player.translation,
                    player.velocity,
                    player.acceleration,
                    player.bounding_box,
                    EntityType::Player {
                        camera: NoClipCamera::new(
                            renderer.device(),
                            renderer.camera_bind_group_layout(),
                            player.translation,
                            0.0,
                            0.0,
                            0.0,
                            Projection::new(
                                renderer.config().width as f32,
                                renderer.config().height as f32,
                                90.0,
                                0.1,
                                10000.0,
                            ),
                        ),
                    },
                    player.response,
                    player.mass,
                );
                renderer.update_instances(state); // TODO: Fine-grained instance updates
                state.entities.push(player);
                Completer::from_value(id)
            }
        }
    }

    pub fn add_object(&mut self, object: ObjectInitData<'a>) -> Completer<'static, u64> {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                let completer = Completer::new(APP_START_PRECOND);
                init_data.objects.push((completer.clone(), object));
                completer
            }
            AppState::Started {
                renderer, state, ..
            } => {
                let id = state.entities.len() as u64;
                let object = Entity::new(
                    id,
                    object.mesh_id.consume().unwrap(),
                    object.texture_id.consume().unwrap(),
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
                renderer.update_instances(state);

                state.entities.push(object);
                Completer::from_value(id)
            }
        }
    }

    pub fn add_texture(&mut self, data: TextureInitData) -> Completer<'static, u64> {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                let completer = Completer::new(APP_START_PRECOND);
                init_data.textures.push((completer.clone(), data));
                completer
            }
            AppState::Started {
                renderer, state: _, ..
            } => Completer::from_value(renderer.new_texture(data)),
        }
    }
}

impl<'a> ApplicationHandler<Event> for App<'a> {
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

            info!("Adding meshes");
            while let Some((mut completer, mesh)) = meshes.pop() {
                let mesh_id = renderer.add_mesh_instanced(mesh).unwrap();
                completer.complete(mesh_id).unwrap();
            }

            info!("Adding textures");
            while let Some((mut completer, texture_init)) = textures.pop() {
                let texture_id = renderer.new_texture(TextureInitData {
                    image: texture_init.image,
                    resize: texture_init.resize,
                });
                completer.complete(texture_id).unwrap();
            }

            info!("Adding entities");
            let mut entities = vec![];
            while let Some((mut completer, entity)) = players_init.pop() {
                let id = entities.len() as u64;
                let player = Entity::new(
                    id,
                    entity.mesh_id.consume().unwrap(),
                    entity.texture_id.consume().unwrap(),
                    entity.scale,
                    entity.rotation,
                    entity.translation,
                    entity.velocity,
                    entity.acceleration,
                    entity.bounding_box,
                    EntityType::Player {
                        camera: NoClipCamera::new(
                            renderer.device(),
                            renderer.camera_bind_group_layout(),
                            entity.translation,
                            0.0,
                            0.0,
                            0.0,
                            Projection::new(
                                renderer.config().width as f32,
                                renderer.config().height as f32,
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
                completer.complete(id).unwrap();
            }

            while let Some((mut completer, object_init)) = objects_init.pop() {
                let id = entities.len() as u64;
                let object = Entity::new(
                    id,
                    object_init.mesh_id.consume().unwrap(),
                    object_init.texture_id.consume().unwrap(),
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
                completer.complete(id).unwrap();
            }

            let mut active_state = ActiveState {
                current_camera: NoClipCamera::new(
                    renderer.device(),
                    renderer.camera_bind_group_layout(),
                    Vector3::identity(),
                    0.0,
                    0.0,
                    0.0,
                    Projection::new(
                        renderer.config().width as f32,
                        renderer.config().height as f32,
                        90.0,
                        0.1,
                        10000.0,
                    ),
                ),
                entities,
                last_update: Instant::now(),
            };

            renderer.update_instances(&mut active_state);

            {
                let mut args = BeforeStartArgs {
                    state: &mut active_state,
                    input: &self.input,
                    renderer: &renderer,
                };
                for system in self.systems.iter_mut() {
                    system.before_start(&mut args);
                }
            }

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
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let AppState::Started {
            renderer, state, ..
        } = &mut self.state
        {
            self.input
                .window_event(&event, renderer.window(), &mut state.current_camera);
        }

        match event {
            WindowEvent::Resized(physical_size) => {
                if let AppState::Started {
                    renderer, state: _, ..
                } = &mut self.state
                {
                    renderer.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                info!("Started Shutdown");
                {
                    // Run dispose and drop each system.
                    let mut args = DisposeArgs {};
                    while let Some(mut system) = self.systems.pop() {
                        system.dispose(&mut args);
                    }
                }
                event_loop.exit()
            }

            WindowEvent::RedrawRequested => {
                if let AppState::Started { renderer, state } = &mut self.state {
                    let elapsed_dur = state.last_update.elapsed();
                    let elapsed = elapsed_dur.as_secs_f32();
                    state.last_update = Instant::now();

                    // start redraw
                    {
                        let mut before_input = BeforeInputArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.before_input(&mut before_input);
                        }
                    }
                    self.input.update(elapsed, &mut state.current_camera);
                    {
                        let mut handle_input = HandleInputArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.handle_input(&mut handle_input);
                        }
                    }

                    {
                        let mut before_tick = BeforeTickArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.before_tick(&mut before_tick);
                        }
                    }

                    {
                        let mut handle_tick = HandleTickArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.handle_tick(&mut handle_tick);
                        }
                    }

                    {
                        let mut after_tick = AfterTickArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.after_tick(&mut after_tick);
                        }
                    }

                    state.update(elapsed, &mut self.world);

                    {
                        let mut before_render = BeforeRenderArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.before_render(&mut before_render);
                        }
                    }

                    renderer.update_instances(state);

                    match renderer.render(state) {
                        Ok(_) => {}
                        Err(e) => error!("{}", e),
                    }

                    {
                        let mut after_render = AfterRenderArgs {
                            elapsed: &elapsed_dur,
                            state,
                            input: &self.input,
                        };
                        for system in self.systems.iter_mut() {
                            system.after_render(&mut after_render);
                        }
                    }

                    renderer.window().request_redraw();
                }
            }
            _ => {}
        }
    }
}
