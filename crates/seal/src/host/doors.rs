//! Doors: the procedures a host gives a script, the only way a script reaches outside itself (D11).
//!
//! A door is an ordinary Rust closure, and Seal reads its types from the closure's own signature (D48):
//!
//! ```
//! # fn main() -> Result<(), seal::Message> {
//! let mut doors = seal::Doors::new();
//! doors.add("show_msg", |text: String| println!("{text}"))?;
//! doors.add("rand", |low: i64, _high: i64| low)?.labels(["from", "to"])?;
//! # Ok(())
//! # }
//! ```

use std::marker::PhantomData;
use std::sync::Arc;

use super::convert::{Failures, FromSeal, IntoSeal, SealType};
use super::value::Value;
use crate::message::{Message, MessageKind, Place};
use crate::signature::{DoorParameter, DoorSignature, HostType};
use crate::syntax::lexer::{Keyword, check_name_spelling};
use crate::vm::machine::{DoorAnswer, DoorCaller};

/// What a pausing door gives back: the script pauses there, and the host resumes it later with an `R`
/// (D49, D52). A door whose closure returns `Pause<R>` is a door that can pause.
#[derive(Clone, Copy, Debug)]
pub struct Pause<R> {
    result: PhantomData<fn() -> R>,
}

impl<R> Pause<R> {
    #[must_use]
    pub fn new() -> Self {
        Pause { result: PhantomData }
    }
}

impl<R> Default for Pause<R> {
    fn default() -> Self {
        Pause::new()
    }
}

/// What a door's closure may return: a value, a `Result` whose error is its failures (D44), or a [`Pause`].
pub trait DoorResult {
    /// The type the script sees the door give.
    fn result_type() -> HostType;
    /// The failures the door can give.
    fn failure_names() -> Vec<String>;
    /// Whether the door pauses the script.
    fn pauses() -> bool;
    fn into_answer(self) -> DoorAnswer;
}

impl<T: IntoSeal> DoorResult for T {
    fn result_type() -> HostType {
        T::host_type()
    }

    fn failure_names() -> Vec<String> {
        Vec::new()
    }

    fn pauses() -> bool {
        false
    }

    fn into_answer(self) -> DoorAnswer {
        DoorAnswer::Value(self.into_seal())
    }
}

impl<T: IntoSeal, E: Failures> DoorResult for Result<T, E> {
    fn result_type() -> HostType {
        T::host_type()
    }

    fn failure_names() -> Vec<String> {
        E::names()
    }

    fn pauses() -> bool {
        false
    }

    fn into_answer(self) -> DoorAnswer {
        match self {
            Ok(value) => DoorAnswer::Value(value.into_seal()),
            Err(failure) => DoorAnswer::Failure(failure.name()),
        }
    }
}

impl<R: SealType> DoorResult for Pause<R> {
    fn result_type() -> HostType {
        R::host_type()
    }

    fn failure_names() -> Vec<String> {
        Vec::new()
    }

    fn pauses() -> bool {
        true
    }

    fn into_answer(self) -> DoorAnswer {
        DoorAnswer::Pause
    }
}

type DoorFunction = Arc<dyn Fn(Vec<Value>) -> DoorAnswer + Send + Sync>;

/// A Rust closure that can be a door. `Marker` only tells closures of different shapes apart.
pub trait IntoDoor<Marker> {
    /// The types of the closure's parameters, as the script sees them.
    fn parameter_types() -> Vec<HostType>;
    fn result_type() -> HostType;
    fn failure_names() -> Vec<String>;
    fn pauses() -> bool;
    fn into_function(self) -> DoorFunction;
}

macro_rules! door_closures {
    ($($argument:ident),*) => {
        impl<Function, Output, $($argument),*> IntoDoor<(Output, $($argument,)*)> for Function
        where
            Function: Fn($($argument),*) -> Output + Send + Sync + 'static,
            Output: DoorResult,
            $($argument: FromSeal,)*
        {
            fn parameter_types() -> Vec<HostType> {
                vec![$($argument::host_type()),*]
            }

            fn result_type() -> HostType {
                Output::result_type()
            }

            fn failure_names() -> Vec<String> {
                Output::failure_names()
            }

            fn pauses() -> bool {
                Output::pauses()
            }

            fn into_function(self) -> DoorFunction {
                Arc::new(move |arguments: Vec<Value>| {
                    #[allow(unused_mut, unused_variables)] // a door with no parameters reads nothing
                    let mut arguments = arguments.into_iter();
                    $(
                        #[allow(non_snake_case)] // each argument takes its type's name, as the macro writes it
                        let Some($argument) = arguments.next().and_then($argument::from_seal) else {
                            return DoorAnswer::Mistake("a door was handed a value that isn't its parameter's type".to_owned());
                        };
                    )*
                    self($($argument),*).into_answer()
                })
            }
        }
    };
}

door_closures!();
door_closures!(A);
door_closures!(A, B);
door_closures!(A, B, C);
door_closures!(A, B, C, D);
door_closures!(A, B, C, D, E);
door_closures!(A, B, C, D, E, F);
door_closures!(A, B, C, D, E, F, G);
door_closures!(A, B, C, D, E, F, G, H);

#[derive(Clone)]
struct Door {
    signature: DoorSignature,
    function: DoorFunction,
}

/// The doors a host opens for a script (D11). A host chooses each script's doors when it loads it (D48).
#[derive(Clone, Default)]
pub struct Doors {
    doors: Vec<Door>,
}

impl std::fmt::Debug for Doors {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.doors.iter().map(|door| &door.signature.name)).finish()
    }
}

fn door_refusal(name: &str, what: impl Into<String>, why: &str) -> Message {
    let mut message = Message::new(MessageKind::Refused, Place::default(), what).with_why(why);
    message.file = format!("the host's door `{name}`");
    message
}

/// Checks a door's or a label's spelling: Seal's own, checked when the host opens the door (D24).
fn check_spelling(door: &str, word: &str) -> Result<(), Message> {
    let why = "a door's name follows Seal's spelling, so a script reads the same in every host.";
    if Keyword::is_keyword(word) {
        return Err(door_refusal(
            door,
            format!("`{word}` is one of Seal's keywords, so it can't name a door or a label"),
            why,
        ));
    }
    check_name_spelling(word, Place::default())
        .map_err(|message| door_refusal(door, message.what.clone().unwrap_or_default(), why))
}

impl Doors {
    #[must_use]
    pub fn new() -> Self {
        Doors::default()
    }

    /// Opens a door: `name` is what scripts call it, with narrowings after dots, as `show_msg.same_line` (D13).
    /// Its types are read from the closure (D48).
    ///
    /// # Errors
    ///
    /// Refuses a name that doesn't follow Seal's spelling (D24), one of Seal's own words, or a name opened twice.
    pub fn add<Marker>(&mut self, name: &str, door: impl IntoDoor<Marker>) -> Result<DoorOptions<'_>, Message> {
        fn parts<Marker, Closure: IntoDoor<Marker>>(_: &Closure) -> (Vec<HostType>, HostType, Vec<String>, bool) {
            (Closure::parameter_types(), Closure::result_type(), Closure::failure_names(), Closure::pauses())
        }
        for word in name.split('.') {
            check_spelling(name, word)?;
        }
        let first = name.split('.').next().unwrap_or(name);
        if ["fmt", "range", "sqrt"].contains(&first) {
            return Err(door_refusal(
                name,
                format!("`{first}` is one of Seal's own words, so it can't name a door"),
                "each word has one meaning.",
            ));
        }
        if self.doors.iter().any(|door| door.signature.name == name) {
            return Err(door_refusal(name, format!("the door `{name}` is opened twice"), "each door is opened once."));
        }
        let (parameters, result, failures, can_pause) = parts(&door);
        for failure in &failures {
            check_spelling(name, failure)?;
        }
        let signature = DoorSignature {
            name: name.to_owned(),
            parameters: parameters.into_iter().map(|host_type| DoorParameter { label: None, host_type }).collect(),
            result,
            failures,
            can_pause,
        };
        self.doors.push(Door { signature, function: door.into_function() });
        match self.doors.last_mut() {
            Some(door) => Ok(DoorOptions { signature: &mut door.signature }),
            None => Err(Message::internal("a door went missing as it was opened")),
        }
    }

    /// What a script sees of each door.
    #[must_use]
    pub fn signatures(&self) -> Vec<DoorSignature> {
        self.doors.iter().map(|door| door.signature.clone()).collect()
    }
}

impl DoorCaller for Doors {
    fn call_door(&self, door: u32, arguments: Vec<Value>) -> DoorAnswer {
        match self.doors.get(door as usize) {
            Some(door) => (door.function)(arguments),
            None => DoorAnswer::Mistake("a door that was never opened was called".to_owned()),
        }
    }
}

/// What more a host says about a door it just opened: its labels (D51, D55).
#[derive(Debug)]
pub struct DoorOptions<'doors> {
    signature: &'doors mut DoorSignature,
}

impl DoorOptions<'_> {
    fn set_labels(self, skip: usize, labels: &[&str]) -> Result<Self, Message> {
        let name = self.signature.name.clone();
        if labels.len() + skip != self.signature.parameters.len() {
            return Err(door_refusal(
                &name,
                format!(
                    "`{name}` takes {} values, but {} labels are given",
                    self.signature.parameters.len(),
                    labels.len() + skip
                ),
                "each of a door's values has one label, or none.",
            ));
        }
        for (position, label) in labels.iter().enumerate() {
            check_spelling(&name, label)?;
            if labels.iter().take(position).any(|earlier| earlier == label) {
                return Err(door_refusal(
                    &name,
                    format!("two of `{name}`'s values have the label `{label}`"),
                    "each label names one argument.",
                ));
            }
        }
        for (parameter, label) in self.signature.parameters.iter_mut().skip(skip).zip(labels) {
            parameter.label = Some((*label).to_owned());
        }
        Ok(self)
    }

    /// Gives every value of the door a label, in order, as `["from", "to"]` for `rand(from: 1, to: 6)`.
    ///
    /// # Errors
    ///
    /// Refuses labels that don't follow Seal's spelling (D24), or that don't match the door's values in number.
    pub fn labels<const COUNT: usize>(self, labels: [&str; COUNT]) -> Result<Self, Message> {
        self.set_labels(0, &labels)
    }

    /// Labels every value after the first, which by convention has none, as `["by"]` for `heal(goblin, by: 5)`.
    ///
    /// # Errors
    ///
    /// Refuses labels that don't follow Seal's spelling (D24), or that don't match the door's values in number.
    pub fn labels_after_first<const COUNT: usize>(self, labels: [&str; COUNT]) -> Result<Self, Message> {
        self.set_labels(1, &labels)
    }
}
