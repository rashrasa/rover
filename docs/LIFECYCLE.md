# Lifecycle

Outlines all lifecycle events in a running application of this game engine.

## Outline

- Create
- Add All Systems
- Run "Before Start" Hooks (Loading Assets, Initializing Connections, etc.)
- Run Game:
    - Record Input State
    - Run "Before Input Handling" Systems (Network, etc.)
    - Run Input Systems
    - Run "Before Tick" Hooks (Physics, etc.)
    - Run "Tick" Hooks (Physics, Animation, )
    - Run "After Tick" Hooks (Audio, etc.)
    - Run "Before Render" Hooks
    - Update all instances that need it
    - Render
    - Run "After Render" Hooks
    - GOTO Run Game
- On Window Close, Run "Dispose" Hooks

## Examples of Systems

- Physics
- Animation
- Network
- Audio
- Bot Intelligence
- Navigation
- UI (through input events)
- Entity Spawner
