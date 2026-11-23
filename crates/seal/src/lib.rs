//! Seal: an interpreted, embeddable, sandboxed scripting language (D1).
//!
//! This is the library a host embeds (D47). A script is loaded from text, checked, compiled to bytecode and run by a
//! register VM (D46); it reaches nothing outside itself except through the doors its host opens (D11).
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

pub mod message;
pub mod syntax;

pub use message::{Message, MessageKind, Messages, Place};
