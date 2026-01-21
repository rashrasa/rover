# (WIP) Rover

A rover driving simulator focused on stunning graphics, audio, procedural generation, and weather events.

![Recording 2026-01-17 204454](https://github.com/user-attachments/assets/c9903064-b028-4226-ac45-40e4edb63471)

*Early stage graphics development and camera testing*

<img width="1922" height="1128" alt="image" src="https://github.com/user-attachments/assets/b971fce0-d7d7-48e6-b35d-b655a94486e2" />

*Lighting testing*
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

# Credits

Textures are sourced from https://www.manytextures.com/, for which the textures are licensed under a [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).
