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
    /// A generic procedure's type parameter, as `?Item`, by its number among the script's (D161). It's the same only
    /// as itself while its procedure is checked; each call works out what it stands for.
    Parameter(u32),
    /// A `Whole` or a `Float` with limits, as `Whole[min: 0]` (D107). It's its plain type plus a check wherever a
    /// value lands (D128), so resolving it gives the plain type, and only [`TypeTable::limits`] sees the limits.
    Ranged {
        base: TypeId,
        limits: Limits,
    },
}

/// One unit: a member of a unit group, by their numbers in [`TypeTable::unit_groups`] (D111, D113, D123).
/// The built-in requirement `Number` (D166), beside the script's own, which are numbered from 0 (D168).
pub const NUMBER: u32 = u32::MAX;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitGroupType {
    pub name: String,
    pub members: Vec<UnitMemberType>,
}

/// A unit in its group, with its size in the group's base unit, when it has a fixed one (D113, D120, D123).
/// `frames` and `tiles` have none: they convert only through a rate.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// A `HugeWhole`'s edge past a `Whole`'s range (D115): `bits` is then its place among the table's big edges.
    pub big: bool,
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
    /// The `HugeWhole` edges past a `Whole`'s range, each once, so equal limits make the same type (D115).
    big_edges: Vec<crate::big::Big>,
    /// Each generic procedure's type parameters, as written, `?` and all, each a [`Type::Parameter`] (D161).
    pub parameters: Vec<String>,
    /// What each type parameter requires: [`NUMBER`], or one of the script's requirements by its number (D166).
    pub required: Vec<Option<u32>>,
    /// The script's requirements' names, by their numbers (D168).
    pub requirement_names: Vec<String>,
    /// Which of the script's types meets which requirement (D167).
    pub meets: Vec<(TypeId, u32)>,
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
            big_edges: Vec::new(),
            parameters: Vec::new(),
            required: Vec::new(),
            requirement_names: Vec::new(),
            meets: Vec::new(),
        }
    }
}

impl TypeTable {
    /// A `HugeWhole`'s edge with this value (D115): a `Whole`'s bits when it's in a `Whole`'s range, and otherwise
    /// its place among the big edges.
    pub fn huge_edge(&mut self, value: &crate::big::Big, clamp: bool) -> Edge {
        if let Some(small) = value.to_i64() {
            return Edge { bits: small.cast_unsigned(), clamp, big: false };
        }
        let known = self.big_edges.iter().position(|known| known.compare(value) == std::cmp::Ordering::Equal);
        let place = known.unwrap_or_else(|| {
            self.big_edges.push(value.clone());
            self.big_edges.len() - 1
        });
        Edge { bits: u64::try_from(place).unwrap_or(u64::MAX), clamp, big: true }
    }

    /// A `HugeWhole` edge's value (D115).
    #[must_use]
    pub fn edge_value(&self, edge: Edge) -> crate::big::Big {
        if edge.big {
            usize::try_from(edge.bits)
                .ok()
                .and_then(|place| self.big_edges.get(place))
                .cloned()
                .unwrap_or_else(|| crate::big::Big::from_i64(0))
        } else {
            crate::big::Big::from_i64(edge.bits.cast_signed())
        }
    }

    #[must_use]
    pub fn new() -> Self {
        // Built once, then copied for each script: Seal's own types and unit groups are the same every time.
        static BASE: std::sync::OnceLock<TypeTable> = std::sync::OnceLock::new();
        BASE.get_or_init(TypeTable::base).clone()
    }

    /// Seal's own unit groups, shared, which every table starts with (D113).
    fn seal_groups() -> std::sync::Arc<Vec<UnitGroupType>> {
        static BASE: std::sync::OnceLock<TypeTable> = std::sync::OnceLock::new();
        BASE.get_or_init(TypeTable::base).unit_groups.clone()
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

    /// trace: D161. A new type parameter, as `?Item`, of one generic procedure.
    pub fn new_parameter(&mut self, name: &str, required: Option<u32>) -> TypeId {
        let number = u32::try_from(self.parameters.len()).unwrap_or(u32::MAX);
        self.parameters.push(name.to_owned());
        self.required.push(required);
        self.intern(Type::Parameter(number))
    }

    /// trace: D166, D168. What a type parameter requires, when the type is one that requires something.
    #[must_use]
    pub fn requirement_of(&self, id: TypeId) -> Option<u32> {
        match self.get(id) {
            Type::Parameter(parameter) => self.required.get(*parameter as usize).copied().flatten(),
            _ => None,
        }
    }

    /// trace: D166. Whether a type is a type parameter that requires a `Number`.
    #[must_use]
    pub fn is_number_parameter(&self, id: TypeId) -> bool {
        self.requirement_of(id) == Some(NUMBER)
    }

    /// trace: D166, D167. Whether a type meets a requirement: a number type meets `Number`, a script's type meets what
    /// its `meets [ … ]` line names, and a type parameter meets what it requires.
    #[must_use]
    pub fn meets_requirement(&self, id: TypeId, requirement: u32) -> bool {
        if self.requirement_of(id) == Some(requirement) {
            return true;
        }
        if requirement == NUMBER {
            return matches!(
                self.get(id),
                Type::Whole | Type::Float | Type::HugeWhole | Type::Decimal | Type::HugeDecimal
            );
        }
        let id = self.resolve(id);
        self.meets.iter().any(|&(meets, what)| self.resolve(meets) == id && what == requirement)
    }

    /// A requirement's name, `Number` included.
    #[must_use]
    pub fn requirement_name(&self, requirement: u32) -> &str {
        if requirement == NUMBER {
            return "Number";
        }
        self.requirement_names.get(requirement as usize).map_or("?", String::as_str)
    }

    /// trace: D161. The type with each type parameter in `map` put in place of what it stands for, all the way down.
    pub fn substitute(&mut self, id: TypeId, map: &[(u32, TypeId)]) -> TypeId {
        let id = self.resolve_variables(id);
        match self.raw(id).clone() {
            Type::Parameter(parameter) => map.iter().find(|(from, _)| *from == parameter).map_or(id, |(_, to)| *to),
            Type::List { item, max } => {
                let item = self.substitute(item, map);
                self.intern(Type::List { item, max })
            }
            Type::Array { item, length } => {
                let item = self.substitute(item, map);
                self.intern(Type::Array { item, length })
            }
            Type::Dict { key, value } => {
                let (key, value) = (self.substitute(key, map), self.substitute(value, map));
                self.intern(Type::Dict { key, value })
            }
            Type::Maybe(inner) => {
                let inner = self.substitute(inner, map);
                self.intern(Type::Maybe(inner))
            }
            _ => id,
        }
    }

    /// How deeply a type nests, counted without nesting in the host's stack: `Whole` is 0, `List<Whole>` 1.
    #[must_use]
    pub fn depth(&self, id: TypeId) -> u32 {
        let mut deepest = 0;
        let mut waiting = vec![(id, 0u32)];
        while let Some((id, depth)) = waiting.pop() {
            deepest = deepest.max(depth);
            match self.get(id) {
                Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => {
                    waiting.push((*item, depth + 1));
                }
                Type::Dict { key, value } => {
                    waiting.push((*key, depth + 1));
                    waiting.push((*value, depth + 1));
                }
                _ => {}
            }
        }
        deepest
    }

    /// trace: D161. Whether one type parameter is anywhere in the type.
    #[must_use]
    pub fn mentions_parameter(&self, id: TypeId, parameter: TypeId) -> bool {
        let (id, parameter) = (self.resolve(id), self.resolve(parameter));
        if id == parameter {
            return true;
        }
        match self.get(id) {
            Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => {
                self.mentions_parameter(*item, parameter)
            }
            Type::Dict { key, value } => {
                self.mentions_parameter(*key, parameter) || self.mentions_parameter(*value, parameter)
            }
            _ => false,
        }
    }

    /// trace: D161. The first type parameter in a type, the type itself when it is one.
    #[must_use]
    pub fn first_parameter(&self, id: TypeId) -> Option<TypeId> {
        match self.get(id) {
            Type::Parameter(_) => Some(id),
            Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => self.first_parameter(*item),
            Type::Dict { key, value } => self.first_parameter(*key).or_else(|| self.first_parameter(*value)),
            _ => None,
        }
    }

    /// trace: D161. Whether a type parameter is anywhere in the type.
    #[must_use]
    pub fn has_parameter(&self, id: TypeId) -> bool {
        match self.get(id) {
            Type::Parameter(_) => true,
            Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => self.has_parameter(*item),
            Type::Dict { key, value } => self.has_parameter(*key) || self.has_parameter(*value),
            _ => false,
        }
    }

    /// The type with every variable bound so far followed all the way down, so nothing in it changes later; one
    /// still open stays open.
    pub fn settled_copy(&mut self, id: TypeId) -> TypeId {
        let id = self.resolve_variables(id);
        match self.raw(id).clone() {
            Type::List { item, max } => {
                let item = self.settled_copy(item);
                self.intern(Type::List { item, max })
            }
            Type::Array { item, length } => {
                let item = self.settled_copy(item);
                self.intern(Type::Array { item, length })
            }
            Type::Dict { key, value } => {
                let (key, value) = (self.settled_copy(key), self.settled_copy(value));
                self.intern(Type::Dict { key, value })
            }
            Type::Maybe(inner) => {
                let inner = self.settled_copy(inner);
                self.intern(Type::Maybe(inner))
            }
            _ => id,
        }
    }

    /// Follows bound variables, but not a ranged number to its plain type.
    fn resolve_variables(&self, mut id: TypeId) -> TypeId {
        for _ in 0..=self.bindings.len() + 1 {
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
            Type::Parameter(parameter) => self.parameters.get(*parameter as usize).cloned().unwrap_or_default(),
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

// ---- in a `.sealc` file (D141) ----

/// The lists [`TypeTable::verify`] works in: each type's parts, how deep each is, which are being walked, and the
/// walk itself.
#[derive(Default)]
struct VerifyLists {
    inside: Vec<([TypeId; 2], u8)>,
    depth: Vec<Option<u32>>,
    walking: Vec<bool>,
    stack: Vec<(usize, usize)>,
}

use crate::compile::file::{FileError, Reader, Writer};

/// How deeply a file's types may nest, so walking one can't overrun the host's stack.
const DEEPEST_IN_FILE: u32 = 256;

impl TypeTable {
    /// How many types it has.
    #[must_use]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Writes the table into a `.sealc` file.
    #[allow(clippy::too_many_lines)] // one case per kind of type, then each list in turn
    pub fn write_file(&self, out: &mut Writer) {
        let unit = |unit: UnitRef, out: &mut Writer| {
            out.u16(unit.group);
            out.u16(unit.member);
        };
        out.count(self.types.len());
        for kind in &self.types {
            match kind {
                Type::Variable(variable) => {
                    out.u8(0);
                    out.u32(*variable);
                }
                Type::Error => out.u8(1),
                Type::Nothing => out.u8(2),
                Type::Whole => out.u8(3),
                Type::Float => out.u8(4),
                Type::Bool => out.u8(5),
                Type::Text => out.u8(6),
                Type::HugeWhole => out.u8(7),
                Type::Decimal => out.u8(8),
                Type::HugeDecimal => out.u8(9),
                Type::List { item, max } => {
                    out.u8(10);
                    out.u32(item.0);
                    out.flag(max.is_some());
                    out.u64(max.unwrap_or(0).cast_unsigned());
                }
                Type::Array { item, length } => {
                    out.u8(11);
                    out.u32(item.0);
                    out.u64(length.cast_unsigned());
                }
                Type::Dict { key, value } => {
                    out.u8(12);
                    out.u32(key.0);
                    out.u32(value.0);
                }
                Type::Record(record) => {
                    out.u8(13);
                    out.u32(*record);
                }
                Type::OneOf(one_of) => {
                    out.u8(14);
                    out.u32(*one_of);
                }
                Type::Maybe(inner) => {
                    out.u8(15);
                    out.u32(inner.0);
                }
                Type::Measured { base, unit: measured } => {
                    out.u8(16);
                    out.u32(base.0);
                    unit(measured.top, out);
                    out.flag(measured.per.is_some());
                    unit(measured.per.unwrap_or(measured.top), out);
                    out.flag(measured.point);
                }
                Type::Parameter(parameter) => {
                    out.u8(17);
                    out.u32(*parameter);
                }
                Type::Ranged { base, limits } => {
                    out.u8(18);
                    out.u32(base.0);
                    crate::compile::file::write_limits(*limits, out);
                }
            }
        }
        out.count(self.bindings.len());
        for binding in &self.bindings {
            out.flag(binding.is_some());
            out.u32(binding.map_or(0, |binding| binding.0));
        }
        let fields = |fields: &[(String, TypeId)], out: &mut Writer| {
            out.count(fields.len());
            for (name, type_id) in fields {
                out.text(name);
                out.u32(type_id.0);
            }
        };
        out.count(self.records.len());
        for record in &self.records {
            out.text(&record.name);
            fields(&record.fields, out);
        }
        out.count(self.one_ofs.len());
        for one_of in &self.one_ofs {
            out.text(&one_of.name);
            out.count(one_of.cases.len());
            for case in &one_of.cases {
                out.text(&case.name);
                fields(&case.fields, out);
            }
        }
        // Seal's own groups, which every table starts with, are one flag (D171); only the rest are written.
        let own = Self::seal_groups();
        let starts_with_own = self.unit_groups.get(..own.len()) == Some(own.as_slice());
        out.flag(starts_with_own);
        let written =
            if starts_with_own { self.unit_groups.get(own.len()..).unwrap_or(&[]) } else { &self.unit_groups[..] };
        out.count(written.len());
        for group in written {
            out.text(&group.name);
            out.count(group.members.len());
            for member in &group.members {
                out.text(&member.name);
                out.flag(member.size.is_some());
                crate::compile::file::write_size(member.size.unwrap_or(Size::ONE), out);
            }
        }
        out.count(self.big_edges.len());
        for edge in &self.big_edges {
            edge.write_file(out);
        }
        out.count(self.parameters.len());
        for (parameter, &required) in self.parameters.iter().zip(&self.required) {
            out.text(parameter);
            out.flag(required.is_some());
            out.u32(required.unwrap_or(0));
        }
        out.count(self.requirement_names.len());
        for name in &self.requirement_names {
            out.text(name);
        }
        out.count(self.meets.len());
        for &(meets, requirement) in &self.meets {
            out.u32(meets.0);
            out.u32(requirement);
        }
    }

    /// Reads a table back from a `.sealc` file, for [`TypeTable::verify`] to check.
    ///
    /// # Errors
    ///
    /// When the file ends, or a part of it isn't a type's.
    #[allow(clippy::too_many_lines)] // one case per kind of type, then each list in turn
    pub fn read_file(input: &mut Reader) -> Result<TypeTable, FileError> {
        let unit = |input: &mut Reader| -> Result<UnitRef, FileError> {
            Ok(UnitRef { group: input.u16()?, member: input.u16()? })
        };
        let count = input.count(1)?;
        let mut types = Vec::with_capacity(count);
        for _ in 0..count {
            types.push(match input.u8()? {
                0 => Type::Variable(input.u32()?),
                1 => Type::Error,
                2 => Type::Nothing,
                3 => Type::Whole,
                4 => Type::Float,
                5 => Type::Bool,
                6 => Type::Text,
                7 => Type::HugeWhole,
                8 => Type::Decimal,
                9 => Type::HugeDecimal,
                10 => {
                    let item = TypeId(input.u32()?);
                    let has_max = input.flag()?;
                    let max = input.u64()?.cast_signed();
                    Type::List { item, max: has_max.then_some(max) }
                }
                11 => Type::Array { item: TypeId(input.u32()?), length: input.u64()?.cast_signed() },
                12 => Type::Dict { key: TypeId(input.u32()?), value: TypeId(input.u32()?) },
                13 => Type::Record(input.u32()?),
                14 => Type::OneOf(input.u32()?),
                15 => Type::Maybe(TypeId(input.u32()?)),
                16 => {
                    let base = TypeId(input.u32()?);
                    let top = unit(input)?;
                    let has_per = input.flag()?;
                    let per = unit(input)?;
                    Type::Measured { base, unit: Unit { top, per: has_per.then_some(per), point: input.flag()? } }
                }
                17 => Type::Parameter(input.u32()?),
                18 => Type::Ranged { base: TypeId(input.u32()?), limits: crate::compile::file::read_limits(input)? },
                _ => return Err(FileError::Damaged("a type that isn't one")),
            });
        }
        let count = input.count(5)?;
        let mut bindings = Vec::with_capacity(count);
        for _ in 0..count {
            let bound = input.flag()?;
            let to = TypeId(input.u32()?);
            bindings.push(bound.then_some(to));
        }
        let fields = |input: &mut Reader| -> Result<Vec<(String, TypeId)>, FileError> {
            let count = input.count(8)?;
            let mut fields = Vec::with_capacity(count);
            for _ in 0..count {
                fields.push((input.text()?, TypeId(input.u32()?)));
            }
            Ok(fields)
        };
        let count = input.count(8)?;
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            records.push(RecordType { name: input.text()?, fields: fields(input)? });
        }
        let count = input.count(8)?;
        let mut one_ofs = Vec::with_capacity(count);
        for _ in 0..count {
            let name = input.text()?;
            let cases = input.count(8)?;
            let mut list = Vec::with_capacity(cases);
            for _ in 0..cases {
                list.push(CaseType { name: input.text()?, fields: fields(input)? });
            }
            one_ofs.push(OneOfType { name, cases: list });
        }
        let starts_with_own = input.flag()?;
        let count = input.count(8)?;
        let only_own = starts_with_own && count == 0;
        let own = Self::seal_groups();
        let mut unit_groups =
            if starts_with_own && count > 0 { own.as_ref().clone() } else { Vec::with_capacity(count) };
        for _ in 0..count {
            let name = input.text()?;
            let members = input.count(5)?;
            let mut list = Vec::with_capacity(members);
            for _ in 0..members {
                let name = input.text()?;
                let has_size = input.flag()?;
                let size = crate::compile::file::read_size(input)?;
                list.push(UnitMemberType { name, size: has_size.then_some(size) });
            }
            unit_groups.push(UnitGroupType { name, members: list });
        }
        let count = input.count(5)?;
        let mut big_edges = Vec::with_capacity(count);
        for _ in 0..count {
            big_edges.push(crate::big::Big::read_file(input)?);
        }
        let count = input.count(4)?;
        let mut parameters = Vec::with_capacity(count);
        let mut required = Vec::with_capacity(count);
        for _ in 0..count {
            parameters.push(input.text()?);
            let has = input.flag()?;
            let requirement = input.u32()?;
            required.push(has.then_some(requirement));
        }
        let count = input.count(4)?;
        let mut requirement_names = Vec::with_capacity(count);
        for _ in 0..count {
            requirement_names.push(input.text()?);
        }
        let count = input.count(8)?;
        let mut meets = Vec::with_capacity(count);
        for _ in 0..count {
            meets.push((TypeId(input.u32()?), input.u32()?));
        }
        // Only checking looks types up by their kind, and a script from a file is checked already, so its table
        // has nothing to look up with: anything made from it later is simply a new type.
        Ok(TypeTable {
            types,
            interned: HashMap::new(),
            bindings,
            records,
            one_ofs,
            // Seal's own, and no others, are shared as a table made from text shares them (D140).
            unit_groups: if only_own { own } else { std::sync::Arc::new(unit_groups) },
            big_edges,
            parameters,
            required,
            requirement_names,
            meets,
        })
    }

    /// trace: D141. Checks a table read from a file: every type it names is in it, every record, `one_of`, unit,
    /// variable and parameter one it has, and nothing inside a type leads back to it or nests too deep, so walking
    /// any type ends.
    ///
    /// # Errors
    ///
    /// What doesn't fit.
    pub fn verify(&self) -> Result<(), String> {
        // The lists checking works in, kept on the thread between files, so checking one makes nothing (D171).
        thread_local! {
            static KEPT: std::cell::Cell<VerifyLists> = std::cell::Cell::new(VerifyLists::default());
        }
        let mut lists = KEPT.take();
        let checked = self.verify_in(&mut lists);
        KEPT.set(lists);
        checked
    }

    #[allow(clippy::too_many_lines)] // what each kind of type names is checked, then the walk through them
    fn verify_in(&self, lists: &mut VerifyLists) -> Result<(), String> {
        let VerifyLists { inside, depth, walking, stack } = lists;
        inside.clear();
        depth.clear();
        walking.clear();
        let count = self.types.len();
        let in_table = |id: TypeId| (id.0 as usize) < count;
        let unit_known = |unit: UnitRef| {
            self.unit_groups
                .get(usize::from(unit.group))
                .is_some_and(|group| usize::from(unit.member) < group.members.len())
        };
        // Seal's own plain types have fixed numbers (D140), so a table always starts with them.
        let fixed = [
            Type::Error,
            Type::Nothing,
            Type::Whole,
            Type::Float,
            Type::Bool,
            Type::Text,
            Type::HugeWhole,
            Type::Decimal,
            Type::HugeDecimal,
        ];
        for (number, kind) in fixed.iter().enumerate() {
            if self.types.get(number) != Some(kind) {
                return Err("its types don't start with Seal's own".to_owned());
            }
        }
        // What's inside each type, followed when walking it: its parts, and what a variable is bound to.
        // At most two parts each, kept side by side, so checking makes one list for all of them (D171).
        inside.reserve(count);
        for kind in &self.types {
            let parts = match kind {
                Type::Variable(variable) => match self.bindings.get(*variable as usize) {
                    Some(Some(binding)) => ([*binding, TypeId(0)], 1),
                    Some(None) => ([TypeId(0); 2], 0),
                    None => return Err("a type to be worked out with no binding".to_owned()),
                },
                Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => ([*item, TypeId(0)], 1),
                Type::Dict { key, value } => ([*key, *value], 2),
                Type::Measured { base, unit } => {
                    if !unit_known(unit.top) || unit.per.is_some_and(|per| !unit_known(per)) {
                        return Err("a unit that isn't one of the script's".to_owned());
                    }
                    ([*base, TypeId(0)], 1)
                }
                Type::Ranged { base, limits } => {
                    for edge in [limits.min, limits.max].into_iter().flatten() {
                        if edge.big && usize::try_from(edge.bits).unwrap_or(usize::MAX) >= self.big_edges.len() {
                            return Err("a limit that isn't one of the script's".to_owned());
                        }
                    }
                    ([*base, TypeId(0)], 1)
                }
                Type::Record(record) if *record as usize >= self.records.len() => {
                    return Err("a record that isn't one of the script's".to_owned());
                }
                Type::OneOf(one_of) if *one_of as usize >= self.one_ofs.len() => {
                    return Err("a `one_of` that isn't one of the script's".to_owned());
                }
                Type::Parameter(parameter) if *parameter as usize >= self.parameters.len() => {
                    return Err("a type parameter that isn't one of the script's".to_owned());
                }
                _ => ([TypeId(0); 2], 0),
            };
            if !parts.0.iter().take(usize::from(parts.1)).all(|&part| in_table(part)) {
                return Err("a type inside another that isn't in the table".to_owned());
            }
            inside.push(parts);
        }
        let fields_known = |fields: &[(String, TypeId)]| fields.iter().all(|&(_, type_id)| in_table(type_id));
        if !self.records.iter().all(|record| fields_known(&record.fields))
            || !self.one_ofs.iter().all(|one_of| one_of.cases.iter().all(|case| fields_known(&case.fields)))
        {
            return Err("a field whose type isn't in the table".to_owned());
        }
        // Depth first, without nesting in the host's stack: a type met again while it's being walked is a loop.
        depth.resize(count, None);
        walking.resize(count, false);
        for start in 0..count {
            if depth.get(start).copied().flatten().is_some() {
                continue;
            }
            // A type with nothing inside is as deep as it gets at once, with no walk.
            if inside.get(start).is_some_and(|(_, parts)| *parts == 0) {
                if let Some(slot) = depth.get_mut(start) {
                    *slot = Some(0);
                }
                continue;
            }
            stack.clear();
            stack.push((start, 0));
            while let Some(&mut (at, ref mut next)) = stack.last_mut() {
                let parts =
                    inside.get(at).map_or(&[][..], |(parts, count)| parts.get(..usize::from(*count)).unwrap_or(&[]));
                if let Some(walking_now) = walking.get_mut(at) {
                    *walking_now = true;
                }
                if let Some(&part) = parts.get(*next) {
                    *next += 1;
                    let part = part.0 as usize;
                    if walking.get(part) == Some(&true) {
                        return Err("a type that holds itself through what's inside it".to_owned());
                    }
                    if depth.get(part).copied().flatten().is_none() {
                        stack.push((part, 0));
                    }
                    continue;
                }
                let deepest = parts.iter().filter_map(|part| depth.get(part.0 as usize).copied().flatten()).max();
                let own = deepest.map_or(0, |deepest| deepest + 1);
                if own > DEEPEST_IN_FILE {
                    return Err("a type nested too deep".to_owned());
                }
                if let Some(slot) = depth.get_mut(at) {
                    *slot = Some(own);
                }
                if let Some(walking_now) = walking.get_mut(at) {
                    *walking_now = false;
                }
                stack.pop();
            }
        }
        Ok(())
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
