//! Seal: an interpreted, embeddable, sandboxed scripting language (D1).
//!
//! This is the library a host embeds (D47). A script is loaded from text, checked, compiled to bytecode and run by a
//! register VM (D46); it reaches nothing outside itself except through the doors its host opens (D11).
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

pub mod check;
pub mod compile;
pub mod host;
pub mod message;
pub mod number;
pub mod signature;
pub mod syntax;
pub mod text;
pub mod vm;

pub use host::{
    Argument, Arguments, Called, DoorOptions, DoorResult, Doors, Error, Failures, FromSeal, IntoArgument, IntoDoor,
    IntoSeal, Labelled, Limits, Pause, Script, SealType, TaskId, Value, labelled,
};
pub use message::{Message, MessageKind, Messages, Place};
/// Makes a struct a Seal record, or an enum a Seal `one_of`, at the doors (D27, D35, D48).
pub use seal_derive::Seal;
pub use signature::{DoorParameter, DoorSignature, HostType};
