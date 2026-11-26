//! The host's side of Seal (D48): doors as typed Rust closures, scripts loaded with their own doors and limits,
//! calls into a script, and tasks that pause, resume and cancel (D49, D51 to D54).

pub mod compiled;
pub mod convert;
pub mod doors;
pub mod script;
pub mod signing;
pub mod value;

pub use compiled::{Cache, build, names};
pub use convert::{Argument, Arguments, Failures, FromSeal, IntoArgument, IntoSeal, Labelled, SealType, labelled};
pub use doors::{DoorOptions, DoorResult, Doors, IntoDoor, Pause};
pub use script::{Called, Error, Limits, Script, TaskId};
pub use value::Value;
