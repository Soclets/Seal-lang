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
use std::sync::{Arc, Mutex};

use super::convert::{Failures, FromSeal, IntoSeal, SealType};
use super::value::Value;
use crate::compile::bytecode::Compiled;
use crate::message::{Message, MessageKind, Place};
use crate::signature::{DoorParameter, DoorSignature, HostType, HostUnitGroup};
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
        match self.plain_bits() {
            Some(bits) => DoorAnswer::Plain(bits),
            None => DoorAnswer::Value(self.into_seal()),
        }
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
            Ok(value) => match value.plain_bits() {
                Some(bits) => DoorAnswer::Plain(bits),
                None => DoorAnswer::Value(value.into_seal()),
            },
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

/// A door as the VM calls it: every argument's bare bits, and, for the arguments that are more than their bits,
/// their values in the same places.
type DoorFunction = Arc<dyn Fn(&[u64], &mut Vec<Value>) -> DoorAnswer + Send + Sync>;

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
                #[allow(unused_variables)] // a door with no parameters reads nothing
                Arc::new(move |bits: &[u64], values: &mut Vec<Value>| {
                    #[allow(unused_mut, unused_variables)] // a door with no parameters reads nothing
                    let mut place = 0usize;
                    $(
                        // A plain number or `Bool` is read straight from its bits; anything else from its value.
                        #[allow(non_snake_case)] // each argument takes its type's name, as the macro writes it
                        let Some($argument) = bits
                            .get(place)
                            .copied()
                            .and_then($argument::from_bits)
                            .or_else(|| values.get_mut(place).map(std::mem::take).and_then($argument::from_seal))
                        else {
                            return DoorAnswer::Mistake("a door was handed a value that isn't its parameter's type".to_owned());
                        };
                        #[allow(unused_assignments)] // the last place is never read
                        {
                            place += 1;
                        }
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

/// A script's source and what it compiled to, kept so that loading the same source again skips the work.
#[derive(Clone)]
pub(crate) struct Loaded {
    pub(crate) source: Arc<str>,
    pub(crate) compiled: Arc<Compiled>,
}

/// How many compiled scripts a set of doors keeps for loading again.
const KEPT_SCRIPTS: usize = 64;

#[derive(Default)]
struct Inner {
    doors: Vec<Door>,
    /// The records and `one_of`s the host opens for its scripts beyond its doors' own (D85).
    types: Vec<HostType>,
    /// The unit groups the host declares for its scripts (D111, D123).
    unit_groups: Vec<HostUnitGroup>,
    /// The scripts compiled against these doors, most recently used last.
    kept: Mutex<Vec<Loaded>>,
}

/// The doors a host opens for a script (D11). A host chooses each script's doors when it loads it (D48).
///
/// Copying a `Doors` is cheap, and the copies share their doors. Loading the same source again with the same doors
/// reuses what it compiled to; each loaded script still has its own names and memory (D49).
#[derive(Clone, Default)]
pub struct Doors {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for Doors {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.inner.doors.iter().map(|door| &door.signature.name)).finish()
    }
}

fn door_refusal(name: &str, what: impl Into<String>, why: &str) -> Message {
    let mut message = Message::new(MessageKind::Refused, Place::default(), what).with_why(why);
    message.file = format!("the host's door `{name}`");
    message
}

/// A record's or a `one_of`'s name.
fn type_name(host_type: &HostType) -> Option<&str> {
    match host_type {
        HostType::Record { name, .. } | HostType::OneOf { name, .. } => Some(name),
        _ => None,
    }
}

/// Checks a host type's names: its own is a type's, capitalised with no `_`, and its fields' and cases' are names
/// (D24, D85, D93).
fn check_type_spelling(host_type: &HostType) -> Result<(), Message> {
    let Some(name) = type_name(host_type) else { return Ok(()) };
    if !name.starts_with(|first: char| first.is_ascii_uppercase())
        || !name.chars().all(|character| character.is_ascii_alphanumeric())
    {
        return Err(door_refusal(
            name,
            format!("`{name}` isn't spelt as a type: a capital first, then letters and digits"),
            "a type's name joins its words with capitals, as `HighScore`, so a capital always means a type.",
        ));
    }
    let words: Vec<&str> = match host_type {
        HostType::Record { fields, .. } => fields.iter().map(|(field, _)| field.as_str()).collect(),
        HostType::OneOf { cases, .. } => cases
            .iter()
            .flat_map(|(case, fields)| {
                std::iter::once(case.as_str()).chain(fields.iter().map(|(field, _)| field.as_str()))
            })
            .collect(),
        _ => Vec::new(),
    };
    for word in words {
        check_spelling(name, word)?;
    }
    Ok(())
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
        // Doors shared with loaded scripts are left as they are; this set gets a list of its own to add to.
        if Arc::get_mut(&mut self.inner).is_none() {
            self.inner = Arc::new(Inner {
                doors: self.inner.doors.clone(),
                types: self.inner.types.clone(),
                unit_groups: self.inner.unit_groups.clone(),
                kept: Mutex::default(),
            });
        }
        let Some(inner) = Arc::get_mut(&mut self.inner) else {
            return Err(Message::internal("a set of doors couldn't be changed"));
        };
        // What was compiled against the doors as they were may not fit them any more.
        if let Ok(kept) = inner.kept.get_mut() {
            kept.clear();
        }
        if inner.doors.iter().any(|door| door.signature.name == name) {
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
        inner.doors.push(Door { signature, function: door.into_function() });
        match inner.doors.last_mut() {
            Some(door) => Ok(DoorOptions { signature: &mut door.signature }),
            None => Err(Message::internal("a door went missing as it was opened")),
        }
    }

    /// What a script sees of each door.
    #[must_use]
    pub fn signatures(&self) -> Vec<DoorSignature> {
        self.inner.doors.iter().map(|door| door.signature.clone()).collect()
    }

    /// Opens one of the host's records or `one_of`s for its scripts, made with `#[derive(Seal)]` (D48). A script
    /// uses it by name with nothing declared, as it uses a door (D85). A type a door takes or gives is opened with
    /// the door, so this is for the ones only the host's calls into the script use.
    ///
    /// # Errors
    ///
    /// Refuses a type that isn't a record or a `one_of`, or whose names don't follow Seal's spelling (D24, D85).
    pub fn add_type<T: crate::host::SealType>(&mut self) -> Result<(), Message> {
        let host_type = T::host_type();
        let name = match &host_type {
            HostType::Record { name, .. } | HostType::OneOf { name, .. } => name.clone(),
            _ => {
                return Err(door_refusal(
                    "?",
                    "only a record or a `one_of` is opened as a type",
                    "a host opens its own records and `one_of`s for a script; Seal's own types are there already.",
                ));
            }
        };
        check_type_spelling(&host_type)?;
        if Arc::get_mut(&mut self.inner).is_none() {
            self.inner = Arc::new(Inner {
                doors: self.inner.doors.clone(),
                types: self.inner.types.clone(),
                unit_groups: self.inner.unit_groups.clone(),
                kept: Mutex::default(),
            });
        }
        let Some(inner) = Arc::get_mut(&mut self.inner) else {
            return Err(Message::internal("a set of doors couldn't be changed"));
        };
        if let Ok(kept) = inner.kept.get_mut() {
            kept.clear();
        }
        if !inner.types.iter().any(|known| matches!(known, HostType::Record { name: known, .. } | HostType::OneOf { name: known, .. } if *known == name)) {
            inner.types.push(host_type);
        }
        Ok(())
    }

    /// Declares a unit group for the host's scripts, as a script's `group.unit` does (D111, D123): each unit is
    /// `base`, for exactly one, or so many of a unit listed before it, as `("dollars", "100 cents")`, with `~` before
    /// an inexact amount.
    ///
    /// ```
    /// # fn main() -> Result<(), seal::Message> {
    /// let mut doors = seal::Doors::new();
    /// doors.add_unit_group("money", &[("cents", "base"), ("dollars", "100 cents")])?;
    /// doors.add("pay", |_cents: i64| ())?.units([Some("cents")])?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Refuses names that don't follow Seal's spelling, a group declared twice or named as one of Seal's own, and
    /// sizes that aren't `base` or an amount and an earlier unit.
    pub fn add_unit_group(&mut self, name: &str, units: &[(&str, &str)]) -> Result<(), Message> {
        check_spelling(name, name)?;
        let why = "a unit is `base`, for exactly one, or so many of a unit listed before it, as `100 cents`.";
        if ["time", "angle", "distance"].contains(&name)
            || self.inner.unit_groups.iter().any(|group| group.name == name)
        {
            return Err(door_refusal(
                name,
                format!("the unit group `{name}` is declared twice, or is one of Seal's own"),
                "each unit group is declared once; `time`, `angle` and `distance` are Seal's own.",
            ));
        }
        let mut members: Vec<(String, String)> = Vec::new();
        for (unit, size) in units {
            check_spelling(name, unit)?;
            if members.iter().any(|(known, _)| known == unit) {
                return Err(door_refusal(name, format!("the unit `{unit}` is listed twice in `{name}`"), why));
            }
            if *size != "base" {
                let fits = size.split_once(' ').is_some_and(|(amount, earlier)| {
                    crate::check::types::Size::written(amount).is_some()
                        && members.iter().any(|(known, _)| known == earlier)
                });
                if !fits {
                    return Err(door_refusal(name, format!("`{unit}`'s size, `{size}`, isn't a size"), why));
                }
            }
            members.push(((*unit).to_owned(), (*size).to_owned()));
        }
        if members.iter().filter(|(_, size)| size == "base").count() != 1 {
            return Err(door_refusal(name, format!("`{name}` needs exactly one unit marked `base`"), why));
        }
        let inner = self.inner_mut()?;
        inner.unit_groups.push(HostUnitGroup { name: name.to_owned(), members });
        Ok(())
    }

    /// The unit groups the host declares (D111, D123).
    #[must_use]
    pub fn unit_groups(&self) -> &[HostUnitGroup] {
        &self.inner.unit_groups
    }

    /// The doors' own list, to change: copied first when a loaded script shares it, and with what was compiled
    /// against it forgotten.
    fn inner_mut(&mut self) -> Result<&mut Inner, Message> {
        if Arc::get_mut(&mut self.inner).is_none() {
            self.inner = Arc::new(Inner {
                doors: self.inner.doors.clone(),
                types: self.inner.types.clone(),
                unit_groups: self.inner.unit_groups.clone(),
                kept: Mutex::default(),
            });
        }
        let Some(inner) = Arc::get_mut(&mut self.inner) else {
            return Err(Message::internal("a set of doors couldn't be changed"));
        };
        if let Ok(kept) = inner.kept.get_mut() {
            kept.clear();
        }
        Ok(inner)
    }

    /// The records and `one_of`s the host opens, its doors' included, each once, inner ones too (D85).
    #[must_use]
    pub fn types(&self) -> Vec<HostType> {
        let mut found: Vec<HostType> = Vec::new();
        let mut waiting: Vec<HostType> = self.inner.types.clone();
        for door in &self.inner.doors {
            waiting.extend(door.signature.parameters.iter().map(|parameter| parameter.host_type.clone()));
            waiting.push(door.signature.result.clone());
        }
        while let Some(host_type) = waiting.pop() {
            match &host_type {
                HostType::List(inner) | HostType::Maybe(inner) | HostType::Measured { base: inner, .. } => {
                    waiting.push((**inner).clone());
                }
                HostType::Dict(key, value) => {
                    waiting.push((**key).clone());
                    waiting.push((**value).clone());
                }
                HostType::Record { name, fields }
                    if !found.iter().any(|known| type_name(known) == Some(name.as_str())) =>
                {
                    waiting.extend(fields.iter().map(|(_, field)| field.clone()));
                    found.push(host_type);
                }
                HostType::OneOf { name, cases }
                    if !found.iter().any(|known| type_name(known) == Some(name.as_str())) =>
                {
                    waiting.extend(cases.iter().flat_map(|(_, fields)| fields.iter().map(|(_, field)| field.clone())));
                    found.push(host_type);
                }
                _ => {}
            }
        }
        found.reverse();
        found
    }

    /// What a source compiled to the last time it was loaded with these doors, when that's still kept.
    pub(crate) fn kept(&self, source: &str) -> Option<Loaded> {
        let mut kept = self.inner.kept.lock().ok()?;
        // Looked for from the most recently used, which is the likeliest to be loaded again.
        let position = kept.iter().rposition(|loaded| *loaded.source == *source)?;
        // Moved to the end, as the most recently used, unless it's there already.
        if position + 1 != kept.len() {
            let loaded = kept.remove(position);
            kept.push(loaded);
        }
        kept.last().cloned()
    }

    /// Keeps what a source compiled to, for the next time it's loaded with these doors.
    pub(crate) fn keep(&self, loaded: Loaded) {
        if let Ok(mut kept) = self.inner.kept.lock() {
            if kept.len() >= KEPT_SCRIPTS {
                kept.remove(0);
            }
            kept.push(loaded);
        }
    }
}

impl DoorCaller for Doors {
    fn call_door(&self, door: u32, bits: &[u64], values: &mut Vec<Value>) -> DoorAnswer {
        match self.inner.doors.get(door as usize) {
            Some(door) => (door.function)(bits, values),
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

    /// Gives the door's values their units, in order, `None` for a value with none, as `[Some("secs")]` for
    /// `wait(2 secs)` (D111). A script then passes a value in that unit, or another of its group that converts
    /// to it exactly, and a plain number is refused.
    ///
    /// # Errors
    ///
    /// Refuses units that don't match the door's values in number, a unit on a value that isn't an `i64` or an
    /// `f64`, and a unit that isn't written as a script writes one.
    pub fn units<const COUNT: usize>(self, units: [Option<&str>; COUNT]) -> Result<Self, Message> {
        let name = self.signature.name.clone();
        if units.len() != self.signature.parameters.len() {
            return Err(door_refusal(
                &name,
                format!(
                    "`{name}` takes {} values, but {} units are given",
                    self.signature.parameters.len(),
                    units.len()
                ),
                "each of a door's values has one unit, or `None`.",
            ));
        }
        for (parameter, unit) in self.signature.parameters.iter_mut().zip(units) {
            if let Some(unit) = unit {
                parameter.host_type = measured(&name, parameter.host_type.clone(), unit)?;
            }
        }
        Ok(self)
    }

    /// Gives the value the door gives back a unit, as `"secs"` for a `now()` that gives a time (D111).
    ///
    /// # Errors
    ///
    /// Refuses a unit on a value that isn't an `i64` or an `f64`, and one that isn't written as a script writes it.
    pub fn result_unit(self, unit: &str) -> Result<Self, Message> {
        let name = self.signature.name.clone();
        self.signature.result = measured(&name, self.signature.result.clone(), unit)?;
        Ok(self)
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

/// A door's `i64` or `f64` with a unit (D111), the unit's spelling checked here and its name when a script loads.
fn measured(door: &str, base: HostType, unit: &str) -> Result<HostType, Message> {
    if !matches!(base, HostType::Whole | HostType::Float) {
        return Err(door_refusal(
            door,
            format!("only an `i64` or an `f64` has a unit, but `{door}` is given `{unit}` for another type"),
            "a unit says what a number measures, as `secs` or `px per secs`.",
        ));
    }
    let (top, per) = unit.split_once(" per ").map_or((unit, None), |(top, per)| (top, Some(per)));
    let top = top.strip_suffix(".point").unwrap_or(top);
    for part in std::iter::once(top).chain(per) {
        let name = part.split_once("::").map_or(part, |(group, name)| {
            if check_name_spelling(group, Place::default()).is_err() { "" } else { name }
        });
        if name.is_empty() || check_name_spelling(name, Place::default()).is_err() {
            return Err(door_refusal(
                door,
                format!("`{unit}` isn't written as a unit"),
                "a unit is written as a script writes it: `secs`, `px.point`, `px per secs`, or `time::secs`.",
            ));
        }
    }
    Ok(HostType::Measured { base: Box::new(base), unit: unit.to_owned() })
}
