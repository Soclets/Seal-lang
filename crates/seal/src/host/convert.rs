//! Rust types standing for Seal's at the doors (D16, D27, D35, D38, D48): `i64` for `Whole`, `f64` for `Float`,
//! `bool` for `Bool`, `String` for `Text`, `Vec` for a list, `HashMap` for a `Dict`, `Option` for a `Maybe`, and
//! structs and enums with `#[derive(Seal)]` for records and `one_of`s.

use std::collections::HashMap;
use std::hash::Hash;

use super::value::Value;
use crate::signature::HostType;

/// A Rust type that stands for a Seal type.
pub trait SealType {
    /// The Seal type this Rust type stands for.
    fn host_type() -> HostType;
}

/// A Rust value that can go into a script, as a copy (D15).
pub trait IntoSeal: SealType {
    fn into_seal(self) -> Value;

    /// The value as the bare bits a script holds it in, for the types that are nothing more: `i64`, `f64`,
    /// `bool` and `()`. It lets a door hand such a value over with no copy made on the way.
    #[doc(hidden)]
    fn plain_bits(&self) -> Option<u64> {
        None
    }
}

/// A Rust value that can come out of a script, as a copy (D15).
pub trait FromSeal: SealType + Sized {
    /// Gives nothing when the value isn't of this type.
    fn from_seal(value: Value) -> Option<Self>;

    /// The value from the bare bits a script holds it in, for the types that are nothing more: `i64`, `f64`,
    /// `bool` and `()`.
    #[doc(hidden)]
    #[must_use]
    fn from_bits(_bits: u64) -> Option<Self> {
        None
    }
}

impl SealType for () {
    fn host_type() -> HostType {
        HostType::Nothing
    }
}

impl IntoSeal for () {
    fn into_seal(self) -> Value {
        Value::Nothing
    }

    fn plain_bits(&self) -> Option<u64> {
        Some(0)
    }
}

impl FromSeal for () {
    fn from_seal(value: Value) -> Option<Self> {
        matches!(value, Value::Nothing).then_some(())
    }
}

macro_rules! plain {
    ($rust:ty, $host:ident, $value:ident, $to_bits:expr, $from_bits:expr) => {
        plain!(@full $rust, $host, $value, {
            fn plain_bits(&self) -> Option<u64> {
                let to_bits: fn(&$rust) -> u64 = $to_bits;
                Some(to_bits(self))
            }
        }, {
            fn from_bits(bits: u64) -> Option<Self> {
                let from_bits: fn(u64) -> $rust = $from_bits;
                Some(from_bits(bits))
            }
        });
    };
    ($rust:ty, $host:ident, $value:ident) => {
        plain!(@full $rust, $host, $value, {}, {});
    };
    (@full $rust:ty, $host:ident, $value:ident, { $($into_extra:tt)* }, { $($from_extra:tt)* }) => {
        impl SealType for $rust {
            fn host_type() -> HostType {
                HostType::$host
            }
        }

        impl IntoSeal for $rust {
            fn into_seal(self) -> Value {
                Value::$value(self)
            }

            $($into_extra)*
        }

        impl FromSeal for $rust {
            fn from_seal(value: Value) -> Option<Self> {
                match value {
                    Value::$value(inner) => Some(inner),
                    _ => None,
                }
            }

            $($from_extra)*
        }
    };
}

plain!(i64, Whole, Whole, |whole| whole.cast_unsigned(), |bits| bits.cast_signed());
plain!(f64, Float, Float, |float| float.to_bits(), f64::from_bits);
plain!(bool, Bool, Bool, |flag| u64::from(*flag), |bits| bits != 0);
plain!(String, Text, Text);

impl<T: SealType> SealType for Vec<T> {
    fn host_type() -> HostType {
        HostType::List(Box::new(T::host_type()))
    }
}

impl<T: IntoSeal> IntoSeal for Vec<T> {
    fn into_seal(self) -> Value {
        Value::List(self.into_iter().map(IntoSeal::into_seal).collect())
    }
}

impl<T: FromSeal> FromSeal for Vec<T> {
    fn from_seal(value: Value) -> Option<Self> {
        match value {
            Value::List(items) => items.into_iter().map(T::from_seal).collect(),
            _ => None,
        }
    }
}

impl<T: SealType> SealType for Option<T> {
    fn host_type() -> HostType {
        HostType::Maybe(Box::new(T::host_type()))
    }
}

impl<T: IntoSeal> IntoSeal for Option<T> {
    fn into_seal(self) -> Value {
        Value::Maybe(self.map(|inner| Box::new(inner.into_seal())))
    }
}

impl<T: FromSeal> FromSeal for Option<T> {
    fn from_seal(value: Value) -> Option<Self> {
        match value {
            Value::Maybe(None) => Some(None),
            Value::Maybe(Some(inner)) => T::from_seal(*inner).map(Some),
            _ => None,
        }
    }
}

impl<K: SealType, V: SealType, S> SealType for HashMap<K, V, S> {
    fn host_type() -> HostType {
        HostType::Dict(Box::new(K::host_type()), Box::new(V::host_type()))
    }
}

/// How two keys are ordered when a `HashMap` goes into a script. A `HashMap` has no order of its own, and a
/// `Dict` keeps one (D32), so the keys go in sorted, the same on every machine (D50).
fn key_order(left: &Value, right: &Value) -> std::cmp::Ordering {
    match (left, right) {
        (Value::Whole(left), Value::Whole(right)) => left.cmp(right),
        (Value::Text(left), Value::Text(right)) => left.cmp(right),
        (Value::Bool(left), Value::Bool(right)) => left.cmp(right),
        _ => std::cmp::Ordering::Equal,
    }
}

impl<K: IntoSeal, V: IntoSeal, S> IntoSeal for HashMap<K, V, S> {
    fn into_seal(self) -> Value {
        let mut pairs: Vec<(Value, Value)> =
            self.into_iter().map(|(key, value)| (key.into_seal(), value.into_seal())).collect();
        pairs.sort_by(|(left, _), (right, _)| key_order(left, right));
        Value::Dict(pairs)
    }
}

impl<K: FromSeal + Eq + Hash, V: FromSeal, S: std::hash::BuildHasher + Default> FromSeal for HashMap<K, V, S> {
    fn from_seal(value: Value) -> Option<Self> {
        match value {
            Value::Dict(pairs) => {
                pairs.into_iter().map(|(key, value)| Some((K::from_seal(key)?, V::from_seal(value)?))).collect()
            }
            _ => None,
        }
    }
}

/// The failures a door can give (D44): a Rust enum whose cases are the failures' names. `#[derive(Seal)]` on an
/// enum whose cases carry nothing makes it one.
pub trait Failures {
    /// Every failure's name.
    fn names() -> Vec<String>;
    /// This failure's name.
    fn name(&self) -> String;
}

/// A value with its label, for a call into a script: `labelled("by", 5)` is `by: 5` (D41).
#[derive(Clone, Debug, PartialEq)]
pub struct Labelled<T> {
    pub label: String,
    pub value: T,
}

/// Gives a value its label for a call into a script, as `labelled("by", 5)` for `by: 5`.
pub fn labelled<T>(label: &str, value: T) -> Labelled<T> {
    Labelled { label: label.to_owned(), value }
}

/// One argument of a call into a script.
#[derive(Clone, Debug, PartialEq)]
pub struct Argument {
    pub label: Option<String>,
    pub value: Value,
}

pub trait IntoArgument {
    fn into_argument(self) -> Argument;
}

impl<T: IntoSeal> IntoArgument for T {
    fn into_argument(self) -> Argument {
        Argument { label: None, value: self.into_seal() }
    }
}

impl<T: IntoSeal> IntoArgument for Labelled<T> {
    fn into_argument(self) -> Argument {
        Argument { label: Some(self.label), value: self.value.into_seal() }
    }
}

/// The arguments of a call into a script: a tuple of values, each plain or [`labelled`].
pub trait Arguments {
    /// Hands the arguments, in order, to `call`.
    fn with_arguments<R>(self, call: impl FnOnce(&[Argument]) -> R) -> R;
}

impl Arguments for Vec<Argument> {
    fn with_arguments<R>(self, call: impl FnOnce(&[Argument]) -> R) -> R {
        call(&self)
    }
}

impl Arguments for &[Argument] {
    fn with_arguments<R>(self, call: impl FnOnce(&[Argument]) -> R) -> R {
        call(self)
    }
}

macro_rules! tuple_arguments {
    (@count) => { 0 };
    (@count $first:ident $($rest:ident)*) => { 1 + tuple_arguments!(@count $($rest)*) };
    ($($name:ident),*) => {
        impl<$($name: IntoArgument),*> Arguments for ($($name,)*) {
            #[allow(non_snake_case)] // each value takes its type's name, as the macro writes it
            fn with_arguments<Output>(self, call: impl FnOnce(&[Argument]) -> Output) -> Output {
                let ($($name,)*) = self;
                // Kept on the stack: a call into a script makes no list on the heap for its arguments.
                let arguments: [Argument; tuple_arguments!(@count $($name)*)] = [$($name.into_argument()),*];
                call(&arguments)
            }
        }
    };
}

tuple_arguments!();
tuple_arguments!(A);
tuple_arguments!(A, B);
tuple_arguments!(A, B, C);
tuple_arguments!(A, B, C, D);
tuple_arguments!(A, B, C, D, E);
tuple_arguments!(A, B, C, D, E, F);
tuple_arguments!(A, B, C, D, E, F, G);
tuple_arguments!(A, B, C, D, E, F, G, H);
