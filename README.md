# (WIP) Rover

A rover driving simulator focused on stunning graphics, audio, procedural generation, and weather events. Built without a game/graphics engine, using wgpu as the WebGPU API provider.

**Testing MVP Release (Video not working on FireFox)**

https://github.com/user-attachments/assets/e3c861e7-889f-4313-b2e2-1c446752a3e2

Early camera testing       |  Lighting testing
:-------------------------:|:-------------------------:
<img src="https://github.com/user-attachments/assets/c9903064-b028-4226-ac45-40e4edb63471" width="480"> | <img width="480" src="https://github.com/user-attachments/assets/b971fce0-d7d7-48e6-b35d-b655a94486e2" />

## Components

These are specific components which are needed for the functioning of this application.

### App (Event Loop + Window + Renderer + World)

- Owns world state, window + rendering context, renderer
- Advances the world
- Triggers renders
- Propogates inputs into physics

### World

- Stores materials, properties, meshes, textures, entities
- Physics logic
- Provides data for rendering
- Includes a deterministic procedural world generator seeded from a 64-bit value

### Other

- Input event dispatch system
- Audio system

## Dev Highlights

### Trait System

Using traits and trait-bounds to enforce the minimum requirements for functions. This way, we keep logic in one place, without tying the logic to specific concrete types. Can be found in `src/core/entity.rs`.

```rust
pub trait Transform: Entity {
    fn transform(&self) -> &Matrix4<f32>;
    fn transform_mut(&mut self) -> &mut Matrix4<f32>;
}
pub trait Dynamic: Transform + Entity {
    fn velocity(&self) -> &Vector3<f32>;
    fn velocity_mut(&mut self) -> &mut Vector3<f32>;

    fn acceleration(&self) -> &Vector3<f32>;
    fn acceleration_mut(&mut self) -> &mut Vector3<f32>;
}

pub fn tick(a: &mut impl Dynamic, dt: f32) {
    // tick logic written once and used for every type
}
```

### State Machine Pattern

Using Rust enums to statically enforce invariants, preventing access to certain data when pre-conditions aren't met. Can be found in `src/render.rs`.

```rust
enum AppState {
    NeedsInit(
        // Data temporarily stored before the app starts.
        AppInitData,
    ),
    Started {
        // Data available once the window is created.
        renderer: Renderer,
        // ...
    },
}
pub struct App {
    state: AppState,
    // ...
}

impl App {
    // ...
    pub fn add_meshes(&mut self, mut meshes: Vec<MeshInitData>) {
        match &mut self.state {
            AppState::NeedsInit(init_data) => {
                // **** No renderer access. ****
            }
            AppState::Started { renderer, state: _ } => {
                // **** Renderer can only be accessed when the app is started. **** 
                renderer.add_meshes(meshes).unwrap();
            }
        }
    }
}
```

## Credits

Marble texture sourced from [https://www.manytextures.com/texture/48/white-marble/], licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).

Motorcycle Engine_60Km/h.wav by Cmart94 -- [https://freesound.org/s/518177/] -- License: Attribution 4.0
