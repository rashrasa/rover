// Systems handled by the window (Input, RawWindowHandle/Surface) need to be exposed in lifecycle events explicitly
// These hooks should provide access to as much as possible.

use std::time::Duration;

use crate::{
    core::input::InputController,
    render::{ActiveState, Renderer},
};

pub struct BeforeStartArgs<'a> {
    pub renderer: &'a Renderer,
}
/// This lifecycle hook is most appropriate for handling queued updates (Network, etc.).
pub struct BeforeInputArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

/// This lifecycle hook is most appropriate for updating the state based on the input state.
pub struct HandleInputArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

/// This lifecycle hook is most appropriate for updating state before the world state advances (Physics, etc.).
pub struct BeforeTickArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

/// This lifecycle hook is most appropriate for advancing the world state.
pub struct HandleTickArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

/// This lifecycle hook is most appropriate for updating systems and world state based on the result of the world tick.
pub struct AfterTickArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

/// This lifecycle hook is most appropriate for updating systems and world state before the world renders.
pub struct BeforeRenderArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

/// This lifecycle hook is most appropriate for updating systems and world state after the world renders.
pub struct AfterRenderArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

/// This lifecycle hook is most appropriate for disposing of systems including any shutdown actions such as
/// saving data to a file, closing any threads, etc.
pub struct DisposeArgs<'a> {
    pub elapsed: &'a Duration,
}

/// A system is a composition of lifecycle hooks, with all being no-op's as default to reduce code spam.
/// To override a specific lifecycle hook, it needs to be specified in the system's System impl block.
///
/// Example:
///
/// ```rust
/// pub struct AudioSystem {}
///
/// impl System for AudioSystem {
///     fn before_start(&mut self, args: BeforeStartArgs) {
///         // perform all operations
///     }
/// }
/// ```
#[allow(unused_variables)]
pub trait System {
    fn before_start(&mut self, args: &BeforeStartArgs) {}

    fn before_input(&mut self, args: &BeforeInputArgs) {}
    fn handle_input(&mut self, args: &HandleInputArgs) {}
    fn before_tick(&mut self, args: &BeforeTickArgs) {}
    fn handle_tick(&mut self, args: &HandleTickArgs) {}
    fn after_tick(&mut self, args: &AfterTickArgs) {}
    fn before_render(&mut self, args: &BeforeRenderArgs) {}
    fn after_render(&mut self, args: &AfterRenderArgs) {}

    fn dispose(&mut self, args: &DisposeArgs) {}
}
