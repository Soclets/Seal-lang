//! What a host declares about its doors: their names, types, labels, failures, and whether they can pause
//! (D11, D13, D44, D48, D52).

/// A type as it crosses a door, on the Rust side (D16, D27, D35, D38).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostType {
    /// No value: a door that gives nothing back.
    Nothing,
    /// `Whole`, Rust's `i64`.
    Whole,
    /// `Float`, Rust's `f64`.
    Float,
    /// `Bool`, Rust's `bool`.
    Bool,
    /// `Text`, Rust's `String`.
    Text,
    /// `List<T>`, Rust's `Vec<T>`.
    List(Box<HostType>),
    /// `Dict<key: K, value: V>`, Rust's `HashMap<K, V>` (D32).
    Dict(Box<HostType>, Box<HostType>),
    /// `Maybe<T>`, Rust's `Option<T>` (D38).
    Maybe(Box<HostType>),
    /// A record, a Rust struct (D27).
    Record { name: String, fields: Vec<(String, HostType)> },
    /// A `one_of`, a Rust enum (D35): its cases, each with its values.
    OneOf { name: String, cases: Vec<HostCase> },
}

/// One case of a `one_of` as the host declares it: its name and the values it carries.
pub type HostCase = (String, Vec<(String, HostType)>);

/// One parameter of a door: its label, when the host gave it one (D51, D55), and its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoorParameter {
    pub label: Option<String>,
    pub host_type: HostType,
}

/// A door as a script sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoorSignature {
    /// The door's name, with its narrowings joined by `.`, as `show_msg.same_line` (D13).
    pub name: String,
    pub parameters: Vec<DoorParameter>,
    pub result: HostType,
    /// The failures the door can give (D44).
    pub failures: Vec<String>,
    /// Whether the door can pause the script (D49, D52).
    pub can_pause: bool,
}
