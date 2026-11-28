//! Wax: an interpreted, embeddable, sandboxed scripting language (D1).
//!
//! This is the library a host embeds (D47). A script is loaded from text, checked, compiled to bytecode and run by a
//! register VM (D46); it reaches nothing outside itself except through the doors its host opens (D11).
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

pub mod big;
pub mod check;
pub mod compile;
pub mod decimal;
pub mod host;
pub mod maths;
pub mod message;
pub mod number;
pub mod signature;
pub mod syntax;
pub mod text;
pub mod vm;
mod wide;

pub use host::{
    Argument, Arguments, Cache, Called, DoorOptions, DoorResult, Doors, Error, Failures, FromWax, IntoArgument,
    IntoDoor, IntoWax, Labelled, Limits, Pause, Script, TaskId, Value, WaxType, build, describing, labelled, names,
};
pub use message::{Message, MessageKind, Messages, Place};
pub use signature::{DoorParameter, DoorSignature, HostType, HostUnitGroup};
/// Makes a struct a Wax record, or an enum a Wax `one_of`, at the doors (D27, D35, D48).
pub use wax_derive::Wax;
