// Systems handled by the window (Input, RawWindowHandle/Surface) need to be exposed in lifecycle events explicitly
// These hooks should provide access to as much as possible.

use std::time::Duration;

use crate::{
    core::input::InputController,
    render::{app::ActiveState, renderer::Renderer},
};

pub struct BeforeStartArgs<'a> {
    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
    pub renderer: &'a Renderer,
}

pub struct BeforeInputArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

pub struct HandleInputArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

pub struct BeforeTickArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

pub struct HandleTickArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

pub struct AfterTickArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

pub struct BeforeRenderArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

pub struct AfterRenderArgs<'a> {
    pub elapsed: &'a Duration,

    pub state: &'a mut ActiveState,
    pub input: &'a InputController,
}

pub struct DisposeArgs {}

/// A system is a composition of lifecycle hooks, with all being no-op's as default to reduce code spam.
/// To override a specific lifecycle hook, it needs to be specified in the system's core::System impl block.
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
    /// This lifecycle hook is most appropriate for updates and initialization which run right before the app starts.
    /// It may be necessary to access the world state and renderer for initialization. It is only run once.
    fn before_start(&mut self, args: &mut BeforeStartArgs) {}

    /// This lifecycle hook is most appropriate for handling queued updates (Network, etc.).
    fn before_input(&mut self, args: &mut BeforeInputArgs) {}
    /// This lifecycle hook is most appropriate for updating the state based on the input state.
    fn handle_input(&mut self, args: &mut HandleInputArgs) {}
    /// This lifecycle hook is most appropriate for updating state before the world state advances (Physics, etc.).
    fn before_tick(&mut self, args: &mut BeforeTickArgs) {}
    /// This lifecycle hook is most appropriate for advancing the world/system state.
    fn handle_tick(&mut self, args: &mut HandleTickArgs) {}
    /// This lifecycle hook is most appropriate for updating systems and world state based on the result of the world tick.
    fn after_tick(&mut self, args: &mut AfterTickArgs) {}
    /// This lifecycle hook is most appropriate for updating systems and world state before the world renders.
    /// It is not guaranteed to run right after a tick has completed, as there are plans to isolate rendering to a thread.
    fn before_render(&mut self, args: &mut BeforeRenderArgs) {}
    /// This lifecycle hook is most appropriate for updating systems and world state after the world renders.
    /// It is not guaranteed to run right before the next tick, as there are plans to isolate rendering to a thread.
    fn after_render(&mut self, args: &mut AfterRenderArgs) {}

    /// This lifecycle hook is most appropriate for disposing of systems including any shutdown actions such as
    /// saving data to a file, closing any threads, etc. The system will also be dropped from memory after this call.
    /// It is only run once.
    fn dispose(&mut self, args: &mut DisposeArgs) {}
}
