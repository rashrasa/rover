/*
   Main set of modules containing all data types, logic, systems, constants, and traits, which are concerned with
   representing and modelling app/engine behaviour, all of which is windowing- and rendering-agnostic.
*/

use std::hash::Hash;

pub mod assets;
pub mod audio;
pub mod camera;
mod constants;
pub mod continuous;
mod data_types;
pub mod entity;
pub mod geometry;
pub mod input;
mod lifecycle;
pub mod lights;
pub mod prefabs;
pub mod world;

// Exports
pub use lifecycle::{
    AfterRenderArgs, AfterTickArgs, BeforeInputArgs, BeforeRenderArgs, BeforeStartArgs,
    BeforeTickArgs, HandleInputArgs, HandleTickArgs, System,
};

pub use constants::*;

pub use data_types::{Completer, CompleterError};

pub trait Instanced<I> {
    fn instance(&self) -> I;
}

pub trait Unique<U: Hash + Eq + PartialEq> {
    fn id(&self) -> &U;
}

pub trait Meshed<U: Hash + Eq + PartialEq> {
    fn mesh_id(&self) -> &U;
}
