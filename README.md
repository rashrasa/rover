# (WIP) Agate Engine

A game engine focused on stunning visuals, efficiency, extensibility, and ease of use. Mainly a learning project.

**Testing MVP Release (Video not working on FireFox)**

https://github.com/user-attachments/assets/e3c861e7-889f-4313-b2e2-1c446752a3e2

Early camera testing       |  Lighting testing
:-------------------------:|:-------------------------:
<img src="https://github.com/user-attachments/assets/c9903064-b028-4226-ac45-40e4edb63471" width="480"> | <img width="480" src="https://github.com/user-attachments/assets/b971fce0-d7d7-48e6-b35d-b655a94486e2" />

## Components

### App (Event Loop + Window + Renderer)

- Owns world state, window + rendering context, renderer
- Triggers renders
- Propogates input events
- Provides lifecycle hooks for systems

### World

- Stores materials, properties, meshes, textures, entities
- Provides data for rendering
- Stores a collection of entities

### Pre-fabricated Systems

- Physics / Dynamics (WIP)
- Audio (WIP)
- Camera (WIP)

## Dev Highlights

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
