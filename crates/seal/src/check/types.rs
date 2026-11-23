//! Seal's types (D16), and working them out (D5, D6).
//!
//! Every type has a [`TypeId`]. A type that isn't worked out yet is a variable, bound later by [`TypeTable::unify`];
//! once a script is checked, no variable is left and every type is fully static.

use std::collections::HashMap;

/// A type's number in its [`TypeTable`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// What a refused expression gets, so that nothing more is refused for using it (Unio D55).
pub const ERROR: TypeId = TypeId(0);
/// What a procedure that gives no result gives (D6).
pub const NOTHING: TypeId = TypeId(1);
pub const WHOLE: TypeId = TypeId(2);
pub const FLOAT: TypeId = TypeId(3);
pub const BOOL: TypeId = TypeId(4);
pub const TEXT: TypeId = TypeId(5);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    /// A type still being worked out.
    Variable(u32),
    Error,
    Nothing,
    Whole,
    Float,
    Bool,
    Text,
    /// `List<T>`, with the most it may hold when it has `len.max` (Unio D167).
    List {
        item: TypeId,
        max: Option<i64>,
    },
    /// `Array<T>[len: n]` (Unio D167).
    Array {
        item: TypeId,
        length: i64,
    },
    /// `Dict<key: K, value: V>` (D32).
    Dict {
        key: TypeId,
        value: TypeId,
    },
    /// A record, by its number among the script's records (D27).
    Record(u32),
    /// A `one_of`, by its number among the script's (D35).
    OneOf(u32),
    /// `Maybe<T>` (D38).
    Maybe(TypeId),
}

/// A record's declaration (D27).
#[derive(Clone, Debug)]
pub struct RecordType {
    pub name: String,
    pub fields: Vec<(String, TypeId)>,
}

/// One case of a `one_of` (D35).
#[derive(Clone, Debug)]
pub struct CaseType {
    pub name: String,
    pub fields: Vec<(String, TypeId)>,
}

/// A `one_of`'s declaration (D35, D36).
#[derive(Clone, Debug)]
pub struct OneOfType {
    pub name: String,
    pub cases: Vec<CaseType>,
}

#[derive(Clone, Debug, Default)]
pub struct TypeTable {
    types: Vec<Type>,
    interned: HashMap<Type, TypeId>,
    /// What each variable is bound to, once it is.
    bindings: Vec<Option<TypeId>>,
    pub records: Vec<RecordType>,
    pub one_ofs: Vec<OneOfType>,
}

impl TypeTable {
    #[must_use]
    pub fn new() -> Self {
        let mut table = TypeTable::default();
        for basic in [Type::Error, Type::Nothing, Type::Whole, Type::Float, Type::Bool, Type::Text] {
            table.intern(basic);
        }
        table
    }

    pub fn intern(&mut self, kind: Type) -> TypeId {
        if let Some(&id) = self.interned.get(&kind) {
            return id;
        }
        let id = TypeId(u32::try_from(self.types.len()).unwrap_or(u32::MAX));
        self.types.push(kind.clone());
        self.interned.insert(kind, id);
        id
    }

    /// A new type to be worked out.
    pub fn fresh(&mut self) -> TypeId {
        let variable = u32::try_from(self.bindings.len()).unwrap_or(u32::MAX);
        self.bindings.push(None);
        self.intern(Type::Variable(variable))
    }

    pub fn list(&mut self, item: TypeId) -> TypeId {
        self.intern(Type::List { item, max: None })
    }

    pub fn maybe(&mut self, inner: TypeId) -> TypeId {
        self.intern(Type::Maybe(inner))
    }

    pub fn dict(&mut self, key: TypeId, value: TypeId) -> TypeId {
        self.intern(Type::Dict { key, value })
    }

    fn raw(&self, id: TypeId) -> &Type {
        self.types.get(id.0 as usize).unwrap_or(&Type::Error)
    }

    /// Follows bound variables until a real type, or a variable still open.
    #[must_use]
    pub fn resolve(&self, mut id: TypeId) -> TypeId {
        // A chain of bindings is never longer than the number of variables.
        for _ in 0..=self.bindings.len() {
            match self.raw(id) {
                Type::Variable(variable) => match self.bindings.get(*variable as usize) {
                    Some(Some(bound)) => id = *bound,
                    _ => return id,
                },
                _ => return id,
            }
        }
        id
    }

    /// The type itself, with its outermost variable followed.
    #[must_use]
    pub fn get(&self, id: TypeId) -> &Type {
        self.raw(self.resolve(id))
    }

    #[must_use]
    pub fn is_error(&self, id: TypeId) -> bool {
        matches!(self.get(id), Type::Error)
    }

    /// Whether the type's outermost shape is still open.
    #[must_use]
    pub fn is_open(&self, id: TypeId) -> bool {
        matches!(self.get(id), Type::Variable(_))
    }

    /// Whether the type is worked out all the way down.
    #[must_use]
    pub fn is_settled(&self, id: TypeId) -> bool {
        match self.get(id) {
            Type::Variable(_) => false,
            Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => self.is_settled(*item),
            Type::Dict { key, value } => self.is_settled(*key) && self.is_settled(*value),
            _ => true,
        }
    }

    fn occurs(&self, variable: u32, inside: TypeId) -> bool {
        match self.get(inside) {
            Type::Variable(other) => *other == variable,
            Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => self.occurs(variable, *item),
            Type::Dict { key, value } => self.occurs(variable, *key) || self.occurs(variable, *value),
            _ => false,
        }
    }

    /// Makes two types the same, binding variables as needed. Gives `false` when they can't be the same.
    /// A refused expression's type fits anything (Unio D55).
    pub fn unify(&mut self, left: TypeId, right: TypeId) -> bool {
        let left = self.resolve(left);
        let right = self.resolve(right);
        if left == right {
            return true;
        }
        match (self.raw(left).clone(), self.raw(right).clone()) {
            (Type::Error, _) | (_, Type::Error) => true,
            (Type::Variable(variable), _) => self.bind(variable, right),
            (_, Type::Variable(variable)) => self.bind(variable, left),
            (Type::List { item: a, max: max_a }, Type::List { item: b, max: max_b }) => {
                max_a == max_b && self.unify(a, b)
            }
            (Type::Array { item: a, length: length_a }, Type::Array { item: b, length: length_b }) => {
                length_a == length_b && self.unify(a, b)
            }
            (Type::Dict { key: key_a, value: value_a }, Type::Dict { key: key_b, value: value_b }) => {
                self.unify(key_a, key_b) && self.unify(value_a, value_b)
            }
            (Type::Maybe(a), Type::Maybe(b)) => self.unify(a, b),
            _ => false,
        }
    }

    fn bind(&mut self, variable: u32, to: TypeId) -> bool {
        if self.occurs(variable, to) {
            return false;
        }
        match self.bindings.get_mut(variable as usize) {
            Some(slot) => {
                *slot = Some(to);
                true
            }
            None => false,
        }
    }

    /// Gives every variable still open inside a type the refused type, so nothing more is said about it.
    pub fn settle_as_error(&mut self, id: TypeId) {
        let id = self.resolve(id);
        match self.raw(id).clone() {
            Type::Variable(variable) => {
                if let Some(slot) = self.bindings.get_mut(variable as usize) {
                    *slot = Some(ERROR);
                }
            }
            Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => self.settle_as_error(item),
            Type::Dict { key, value } => {
                self.settle_as_error(key);
                self.settle_as_error(value);
            }
            _ => {}
        }
    }

    /// The type as it's written in a script, for messages.
    #[must_use]
    pub fn describe(&self, id: TypeId) -> String {
        match self.get(id) {
            Type::Variable(_) => "a type not worked out yet".to_owned(),
            Type::Error => "an unknown type".to_owned(),
            Type::Nothing => "nothing".to_owned(),
            _ => format!("`{}`", self.written(id)),
        }
    }

    /// The type as written, without quotes.
    #[must_use]
    pub fn written(&self, id: TypeId) -> String {
        match self.get(id) {
            Type::Variable(_) | Type::Error => "?".to_owned(),
            Type::Nothing => "nothing".to_owned(),
            Type::Whole => "Whole".to_owned(),
            Type::Float => "Float".to_owned(),
            Type::Bool => "Bool".to_owned(),
            Type::Text => "Text".to_owned(),
            Type::List { item, max } => match max {
                Some(max) => format!("List<{}>[len.max: {max}]", self.written(*item)),
                None => format!("List<{}>", self.written(*item)),
            },
            Type::Array { item, length } => format!("Array<{}>[len: {length}]", self.written(*item)),
            Type::Dict { key, value } => {
                format!("Dict<key: {}, value: {}>", self.written(*key), self.written(*value))
            }
            Type::Record(record) => {
                self.records.get(*record as usize).map_or_else(|| "?".to_owned(), |record| record.name.clone())
            }
            Type::OneOf(one_of) => {
                self.one_ofs.get(*one_of as usize).map_or_else(|| "?".to_owned(), |one_of| one_of.name.clone())
            }
            Type::Maybe(inner) => format!("Maybe<{}>", self.written(*inner)),
        }
    }

    /// Whether a value of this type lives on the heap, held by counted holders (D15).
    #[must_use]
    pub fn is_reference(&self, id: TypeId) -> bool {
        match self.get(id) {
            Type::Text
            | Type::List { .. }
            | Type::Array { .. }
            | Type::Dict { .. }
            | Type::Record(_)
            | Type::Maybe(_) => true,
            Type::OneOf(one_of) => self
                .one_ofs
                .get(*one_of as usize)
                .is_some_and(|one_of| one_of.cases.iter().any(|case| !case.fields.is_empty())),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d05_a_variable_is_bound_by_how_it_is_used() {
        let mut table = TypeTable::new();
        let parameter = table.fresh();
        let list = table.list(parameter);
        let whole_list = table.list(WHOLE);
        assert!(table.unify(list, whole_list));
        assert_eq!(table.resolve(parameter), WHOLE);
        assert!(!table.unify(parameter, FLOAT));
        assert_eq!(table.written(list), "List<Whole>");
    }
}
