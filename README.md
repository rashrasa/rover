# (WIP) Rover

A rover driving simulator focused on stunning graphics, audio, procedural generation, and weather events. Built without a game/graphics engine, using wgpu as the WebGPU API provider.

Early camera testing       |  Lighting testing
:-------------------------:|:-------------------------:
<img src="https://github.com/user-attachments/assets/c9903064-b028-4226-ac45-40e4edb63471" width="480"> | <img width="480" src="https://github.com/user-attachments/assets/b971fce0-d7d7-48e6-b35d-b655a94486e2" />

**Testing MVP Release (Video not working on FireFox)**

https://github.com/user-attachments/assets/e3c861e7-889f-4313-b2e2-1c446752a3e2

## Components

These are specific components which are needed for the functioning of this application.

### Global Tick

- Owns world state, window + rendering context, renderer
- Advances the world
- Triggers renders
- Propogates inputs into physics

### World System

- Materials, properties
- Collision state
- Weather events and their state
- Camera position
- Entities
- Atmosphere
- Specialized physics "functions" to be called in the step function

#### World Generator

- Creates an entire world from a single seed

### Rendering Engine

- Renders visuals based on world state, atmosphere, camera position
- Renders fog, storms

### Input Event System

- Emits input events

### Audio System

- Generates sounds based on world state

## Credits

Marble texture sourced from [https://www.manytextures.com/texture/48/white-marble/], licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).

Motorcycle Engine_60Km/h.wav by Cmart94 -- [https://freesound.org/s/518177/] -- License: Attribution 4.0
