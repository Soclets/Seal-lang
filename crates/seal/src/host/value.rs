//! A value as the host holds it, outside any script (D15): values cross the doors as copies.

/// A Seal value on the host's side. Hosts mostly use plain Rust types through [`crate::IntoSeal`] and
/// [`crate::FromSeal`]; this is what those turn into and come from.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Value {
    /// What a procedure or door that gives nothing gives.
    #[default]
    Nothing,
    Whole(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    HugeWhole(crate::big::Big),
    Decimal(crate::decimal::Decimal),
    HugeDecimal(crate::decimal::HugeDecimal),
    /// A list or an array.
    List(Vec<Value>),
    /// A dictionary's pairs, in the order their keys were added (D32).
    Dict(Vec<(Value, Value)>),
    /// A `Maybe` (D38).
    Maybe(Option<Box<Value>>),
    /// A record's fields, in the order it declares them (D27).
    Record(Vec<Value>),
    /// A case of a `one_of`: its number among the type's cases, and its values in declared order (D35).
    Case {
        case: u32,
        fields: Vec<Value>,
    },
}

impl Value {
    /// The value as it would be written in a script, for messages.
    #[must_use]
    pub fn describe(&self) -> &'static str {
        match self {
            Value::Nothing => "nothing",
            Value::Whole(_) => "a `Whole`",
            Value::Float(_) => "a `Float`",
            Value::Bool(_) => "a `Bool`",
            Value::Text(_) => "a `Text`",
            Value::HugeWhole(_) => "a `HugeWhole`",
            Value::Decimal(_) => "a `Decimal`",
            Value::HugeDecimal(_) => "a `HugeDecimal`",
            Value::List(_) => "a list",
            Value::Dict(_) => "a `Dict`",
            Value::Maybe(_) => "a `Maybe`",
            Value::Record(_) => "a record",
            Value::Case { .. } => "a case of a `one_of`",
        }
    }
}
