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
/// A whole number of any size (D115).
pub const HUGE_WHOLE: TypeId = TypeId(6);
/// IEEE 754's decimal128 (D116).
pub const DECIMAL: TypeId = TypeId(7);
/// An exact decimal of any size (D116).
pub const HUGE_DECIMAL: TypeId = TypeId(8);

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
    /// A whole number of any size (D115).
    HugeWhole,
    /// IEEE 754's decimal128 (D116).
    Decimal,
    /// An exact decimal of any size (D116).
    HugeDecimal,
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
    /// A `Whole` or a `Float` with a unit, as `Float[unit: px per secs]` (D111, D112, D120, D121). Its value is a bare
    /// number in that unit.
    Measured {
        base: TypeId,
        unit: Unit,
    },
    /// A `Whole` or a `Float` with limits, as `Whole[min: 0]` (D107). It's its plain type plus a check wherever a
    /// value lands (D128), so resolving it gives the plain type, and only [`TypeTable::limits`] sees the limits.
    Ranged {
        base: TypeId,
        limits: Limits,
    },
}

/// One unit: a member of a unit group, by their numbers in [`TypeTable::unit_groups`] (D111, D113, D123).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnitRef {
    pub group: u16,
    pub member: u16,
}

/// A number's unit: a unit, or a rate of two written with `per`, and whether it's a point, counted from a fixed
/// place, rather than an amount (D112, D120, D121).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Unit {
    pub top: UnitRef,
    pub per: Option<UnitRef>,
    pub point: bool,
}

/// A unit group: what its units measure (D113, D123).
#[derive(Clone, Debug)]
pub struct UnitGroupType {
    pub name: String,
    pub members: Vec<UnitMemberType>,
}

/// A unit in its group, with its size in the group's base unit, when it has a fixed one (D113, D120, D123).
/// `frames` and `tiles` have none: they convert only through a rate.
#[derive(Clone, Debug)]
pub struct UnitMemberType {
    pub name: String,
    pub size: Option<Size>,
}

/// A unit's size, or the factor between two units: `numerator / denominator × π^pi`, exactly. It's `inexact` when
/// it comes from a size written with `~` (D123) or has π in it, and then a value converted by it is a `Float`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Size {
    pub numerator: u128,
    pub denominator: u128,
    pub pi: i8,
    pub inexact: bool,
}

impl Size {
    pub const ONE: Size = Size { numerator: 1, denominator: 1, pi: 0, inexact: false };

    #[must_use]
    pub fn whole(numerator: u128) -> Size {
        Size { numerator, ..Size::ONE }
    }

    /// This size times another, when it fits.
    #[must_use]
    pub fn times(self, other: Size) -> Option<Size> {
        Some(Size::reduced(
            self.numerator.checked_mul(other.numerator)?,
            self.denominator.checked_mul(other.denominator)?,
            self.pi.checked_add(other.pi)?,
            self.inexact || other.inexact,
        ))
    }

    /// This size divided by another, when it fits: the number of `other` in one `self`.
    #[must_use]
    pub fn over(self, other: Size) -> Option<Size> {
        Some(Size::reduced(
            self.numerator.checked_mul(other.denominator)?,
            self.denominator.checked_mul(other.numerator)?,
            self.pi.checked_sub(other.pi)?,
            self.inexact || other.inexact,
        ))
    }

    fn reduced(numerator: u128, denominator: u128, pi: i8, inexact: bool) -> Size {
        let (mut a, mut b) = (numerator, denominator);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        let divisor = a.max(1);
        Size { numerator: numerator / divisor, denominator: denominator / divisor, pi, inexact: inexact || pi != 0 }
    }

    /// An amount as written in a unit's size: a whole number or one with a point, `_` between digits, and `~`
    /// before it for an inexact one (D123).
    #[must_use]
    pub fn written(amount: &str) -> Option<Size> {
        let (inexact, amount) = match amount.strip_prefix('~') {
            Some(rest) => (true, rest),
            None => (false, amount),
        };
        let amount = amount.replace('_', "");
        let (whole, fraction) = amount.split_once('.').unwrap_or((&amount, ""));
        if whole.is_empty() || !whole.bytes().chain(fraction.bytes()).all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        let numerator = format!("{whole}{fraction}").parse::<u128>().ok()?;
        if numerator == 0 {
            return None;
        }
        let denominator = 10u128.checked_pow(u32::try_from(fraction.len()).ok()?)?;
        Some(Size::reduced(numerator, denominator, 0, inexact))
    }

    /// Whether this is a whole number exactly, so a `Whole` stays a `Whole` when converted by it.
    #[must_use]
    pub fn is_whole(self) -> bool {
        self.denominator == 1 && self.pi == 0 && !self.inexact
    }

    /// Roughly how big it is, only for choosing the finer of two units.
    #[must_use]
    pub fn rough(self) -> f64 {
        #[allow(clippy::cast_precision_loss, reason = "only compared, never shown or kept")]
        let fraction = self.numerator as f64 / self.denominator as f64;
        fraction * std::f64::consts::PI.powi(i32::from(self.pi))
    }
}

/// Seal's own unit groups (D113): `time`, `angle` and `distance`, each unit's size in the group's first. A radian
/// is 180/π degrees.
#[must_use]
pub fn seal_unit_groups() -> Vec<UnitGroupType> {
    let member = |name: &str, size: Option<u128>| UnitMemberType { name: name.to_owned(), size: size.map(Size::whole) };
    let radian = Size { numerator: 180, denominator: 1, pi: -1, inexact: true };
    vec![
        UnitGroupType {
            name: "time".to_owned(),
            members: vec![
                member("ms", Some(1)),
                member("secs", Some(1000)),
                member("mins", Some(60_000)),
                member("frames", None),
            ],
        },
        UnitGroupType {
            name: "angle".to_owned(),
            members: vec![
                member("degs", Some(1)),
                member("turns", Some(360)),
                UnitMemberType { name: "rads".to_owned(), size: Some(radian) },
            ],
        },
        UnitGroupType { name: "distance".to_owned(), members: vec![member("px", Some(1)), member("tiles", None)] },
    ]
}

/// A ranged number's limits (D107): each edge, when it has one, as its value's bits, and whether it pulls a value
/// back (`.clamp`) instead of stopping the script.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Limits {
    pub min: Option<Edge>,
    pub max: Option<Edge>,
    /// The values are `Float`s' bits, not `Whole`s'.
    pub float: bool,
    /// The value is a `HugeWhole`'s handle, its edges `Whole`s (D115).
    pub huge: bool,
}

/// One edge of a ranged number (D107).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Edge {
    pub bits: u64,
    pub clamp: bool,
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

#[derive(Clone, Debug)]
pub struct TypeTable {
    types: Vec<Type>,
    interned: HashMap<Type, TypeId>,
    /// What each variable is bound to, once it is.
    bindings: Vec<Option<TypeId>>,
    pub records: Vec<RecordType>,
    pub one_ofs: Vec<OneOfType>,
    /// Seal's unit groups, then the host's and the script's own (D113, D123), shared until one is added.
    pub unit_groups: std::sync::Arc<Vec<UnitGroupType>>,
}

impl Default for TypeTable {
    /// An empty table, sharing one empty list of unit groups, so making one makes nothing (D140).
    fn default() -> Self {
        static NO_UNIT_GROUPS: std::sync::OnceLock<std::sync::Arc<Vec<UnitGroupType>>> = std::sync::OnceLock::new();
        TypeTable {
            types: Vec::new(),
            interned: HashMap::new(),
            bindings: Vec::new(),
            records: Vec::new(),
            one_ofs: Vec::new(),
            unit_groups: NO_UNIT_GROUPS.get_or_init(std::sync::Arc::default).clone(),
        }
    }
}

impl TypeTable {
    #[must_use]
    pub fn new() -> Self {
        // Built once, then copied for each script: Seal's own types and unit groups are the same every time.
        static BASE: std::sync::OnceLock<TypeTable> = std::sync::OnceLock::new();
        BASE.get_or_init(TypeTable::base).clone()
    }

    fn base() -> Self {
        let mut table = TypeTable::default();
        // In the order of their fixed numbers, `ERROR` first; they're found by those, never through the table.
        table.types.extend([
            Type::Error,
            Type::Nothing,
            Type::Whole,
            Type::Float,
            Type::Bool,
            Type::Text,
            Type::HugeWhole,
            Type::Decimal,
            Type::HugeDecimal,
        ]);
        table.unit_groups = std::sync::Arc::new(seal_unit_groups());
        table
    }

    /// The unit groups, to add one to.
    pub fn unit_groups_mut(&mut self) -> &mut Vec<UnitGroupType> {
        std::sync::Arc::make_mut(&mut self.unit_groups)
    }

    pub fn intern(&mut self, kind: Type) -> TypeId {
        // Seal's own plain types have fixed numbers, so they're found without looking in the table (D140).
        let fixed = match kind {
            Type::Error => Some(ERROR),
            Type::Nothing => Some(NOTHING),
            Type::Whole => Some(WHOLE),
            Type::Float => Some(FLOAT),
            Type::Bool => Some(BOOL),
            Type::Text => Some(TEXT),
            Type::HugeWhole => Some(HUGE_WHOLE),
            Type::Decimal => Some(DECIMAL),
            Type::HugeDecimal => Some(HUGE_DECIMAL),
            _ => None,
        };
        if let Some(id) = fixed {
            return id;
        }
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
        // A type to be worked out is a new one every time, so it's never looked for, and isn't put in the table.
        let id = TypeId(u32::try_from(self.types.len()).unwrap_or(u32::MAX));
        self.types.push(Type::Variable(variable));
        id
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
        for _ in 0..=self.bindings.len() + 1 {
            match self.raw(id) {
                Type::Variable(variable) => match self.bindings.get(*variable as usize) {
                    Some(Some(bound)) => id = *bound,
                    _ => return id,
                },
                // trace: D128. A ranged number is its plain type, as far as every type rule goes.
                Type::Ranged { base, .. } => id = *base,
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

    /// A unit's name as it's written, with its group when another group has a unit of the same name (D112).
    #[must_use]
    pub fn unit_name(&self, unit: UnitRef) -> String {
        let name = self
            .unit_groups
            .get(usize::from(unit.group))
            .and_then(|group| group.members.get(usize::from(unit.member)))
            .map_or("?", |member| member.name.as_str());
        let clashes = self
            .unit_groups
            .iter()
            .enumerate()
            .filter(|(index, group)| {
                *index != usize::from(unit.group) && group.members.iter().any(|member| member.name == name)
            })
            .count();
        if clashes > 0 {
            let group = self.unit_groups.get(usize::from(unit.group)).map_or("?", |group| group.name.as_str());
            format!("{group}::{name}")
        } else {
            name.to_owned()
        }
    }

    /// A unit as it's written: `secs`, `px.point`, `px per secs` (D112, D120, D121).
    #[must_use]
    pub fn unit_text(&self, unit: Unit) -> String {
        let mut text = self.unit_name(unit.top);
        if unit.point {
            text.push_str(".point");
        }
        if let Some(per) = unit.per {
            text.push_str(" per ");
            text.push_str(&self.unit_name(per));
        }
        text
    }

    /// A number type's unit, when it has one, and its plain number type.
    #[must_use]
    pub fn measured(&self, id: TypeId) -> Option<(TypeId, Unit)> {
        match self.get(id) {
            Type::Measured { base, unit } => Some((*base, *unit)),
            _ => None,
        }
    }

    /// A ranged number's limits, when the type as written has them (D107, D128).
    #[must_use]
    pub fn limits(&self, id: TypeId) -> Option<Limits> {
        match self.raw(id) {
            Type::Ranged { limits, .. } => Some(*limits),
            _ => None,
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
            Type::HugeWhole => "HugeWhole".to_owned(),
            Type::Decimal => "Decimal".to_owned(),
            Type::HugeDecimal => "HugeDecimal".to_owned(),
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
            Type::Ranged { base, .. } => self.written(*base),
            Type::Measured { base, unit } => format!("{}[unit: {}]", self.written(*base), self.unit_text(*unit)),
        }
    }

    /// Whether a value of this type lives on the heap, held by counted holders (D15).
    #[must_use]
    pub fn is_reference(&self, id: TypeId) -> bool {
        match self.get(id) {
            Type::Text
            | Type::HugeWhole
            | Type::Decimal
            | Type::HugeDecimal
            | Type::List { .. }
            | Type::Array { .. }
            | Type::Dict { .. }
            | Type::Record(_)
            | Type::Maybe(_) => true,
            Type::OneOf(one_of) => self
                .one_ofs
                .get(*one_of as usize)
                .is_some_and(|one_of| one_of.cases.iter().any(|case| !case.fields.is_empty())),
            // A number with a unit is held as its number is: a `HugeWhole` on the heap (D115).
            Type::Measured { base, .. } => self.is_reference(*base),
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
