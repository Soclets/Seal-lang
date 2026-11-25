//! The checked tree: a script after checking, with every name sorted out and every type known (D46).
//!
//! The compiler reads this, never the syntax tree. Types may still hold variables while a script is being checked;
//! by the time checking ends with no messages, every one is settled.

use super::types::{TypeId, TypeTable};
use crate::message::Place;
use crate::syntax::tree::{BinaryOperator, PositionKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VariableId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProcedureId(pub u32);

/// A failure's number: its place in [`Program::failure_names`], counting from 1. 0 means no failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FailureId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableKind {
    /// Made with `name = value` (D2).
    Named,
    /// A procedure's parameter: read-only (Unio D147).
    Parameter,
    /// A `for each`'s item (Unio D171).
    LoopItem,
    /// A `for each`'s position or key (D33, Unio D174).
    LoopPosition,
    /// A value taken out of a case by `match` (D37).
    CaseValue,
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)] // each is a separate fact about a name, set at a different time
pub struct Variable {
    pub name: String,
    pub type_id: TypeId,
    pub kind: VariableKind,
    /// Made with `mut` (D2).
    pub mutable: bool,
    /// Changed somewhere, which a name made with `mut` must be (D135).
    pub changed: bool,
    /// Made with `unused` (D3).
    pub marked_unused: bool,
    pub read: bool,
    /// Made at the top level's outermost block.
    pub top_level: bool,
    /// A top-level name a procedure uses, so it outlives the top level's run and is shared by tasks (D51).
    pub captured: bool,
    /// The top-level statement that makes it, counting from 0, when it's a top-level name.
    pub made_at: usize,
    pub place: Place,
}

#[derive(Clone, Debug)]
pub struct Procedure {
    /// The full name, narrowings included, as `spawn.boss`.
    pub name: String,
    pub parameters: Vec<VariableId>,
    /// Each parameter's label, when it has one (D41).
    pub labels: Vec<Option<String>>,
    /// Written `labels.any_order`: labelled arguments go in any order (D87).
    pub labels_any_order: bool,
    /// Each parameter's default, when it has one (D42).
    pub defaults: Vec<Option<Expression>>,
    pub result: TypeId,
    /// Whether the procedure can fail at all: its body has a `fail` or a `?` (D44).
    pub can_fail: bool,
    /// The failures it can give, worked out from its body (D44).
    pub failures: Vec<FailureId>,
    /// Whether it can pause, worked out from its body (D52).
    pub can_pause: bool,
    pub body: Block,
    pub place: Place,
    pub end_place: Place,
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub types: TypeTable,
    pub variables: Vec<Variable>,
    pub procedures: Vec<Procedure>,
    pub top_level: Block,
    /// Whether the top level can pause (D52).
    pub top_level_can_pause: bool,
    /// Every failure's name; a [`FailureId`] is a place in this list, counting from 1.
    pub failure_names: Vec<std::borrow::Cow<'static, str>>,
}

impl Program {
    #[must_use]
    pub fn failure_name(&self, failure: FailureId) -> &str {
        self.failure_names.get(usize::from(failure.0).wrapping_sub(1)).map_or("?", AsRef::as_ref)
    }

    #[must_use]
    pub fn variable(&self, variable: VariableId) -> Option<&Variable> {
        self.variables.get(variable.0 as usize)
    }

    #[must_use]
    pub fn procedure(&self, procedure: ProcedureId) -> Option<&Procedure> {
        self.procedures.get(procedure.0 as usize)
    }

    #[must_use]
    pub fn procedure_named(&self, name: &str) -> Option<ProcedureId> {
        let index = self.procedures.iter().position(|procedure| procedure.name == name)?;
        Some(ProcedureId(u32::try_from(index).ok()?))
    }
}

pub type Block = Vec<Statement>;

#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub place: Place,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    /// The case's number within its type.
    pub case: u32,
    /// The values taken out: each one's place among the case's values, and the name it gets.
    pub values: Vec<(u32, VariableId)>,
    pub body: Block,
}

/// One step from a value to a part of it (D62).
#[derive(Clone, Debug)]
pub enum PathStep {
    Field(String),
    Item { index: Expression, zero_based: bool },
}

/// The way from a `mut` name to the part of it a change is made in, as `bodies[pos: 1]` in
/// `bodies[pos: 1].vx = 2.0` (D62).
#[derive(Clone, Debug)]
pub struct Inside {
    /// What the change needs worked out, each under a hidden name, before the way in is taken: nothing else may
    /// run between reaching the part and changing it.
    pub lets: Vec<(VariableId, Expression)>,
    pub root: VariableId,
    pub steps: Vec<PathStep>,
    /// The type each step goes into, in order.
    pub types: Vec<TypeId>,
    /// The hidden name the part goes by while it's changed.
    pub part: VariableId,
}

#[derive(Clone, Debug)]
pub enum StatementKind {
    /// A change to a part of a part of a `mut` name: `change` is made to `path.part` (D62).
    Inside {
        path: Inside,
        change: Box<Statement>,
    },
    /// Makes a new name (D2).
    Make {
        variable: VariableId,
        value: Expression,
    },
    /// Gives a `mut` name a new value (D2).
    Set {
        variable: VariableId,
        value: Expression,
    },
    /// `name.field = value` (D27).
    SetField {
        variable: VariableId,
        field: String,
        value: Expression,
    },
    /// `name[pos: n] = value` or `name[offset: n] = value` (Unio D80).
    SetItem {
        variable: VariableId,
        index: Expression,
        zero_based: bool,
        value: Expression,
    },
    /// `name[key: k] = value` (D32).
    SetKey {
        variable: VariableId,
        key: Expression,
        value: Expression,
    },
    /// A call standing alone, giving nothing.
    Evaluate(Expression),
    If {
        branches: Vec<(Expression, Block)>,
        otherwise: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    /// `for each` over a `range[…]` (Unio D170, D172).
    ForRange {
        item: VariableId,
        position: Option<(VariableId, bool)>,
        from: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
        step: Option<Box<Expression>>,
        body: Block,
    },
    /// `for each` over a list, an array or a dictionary; which one is the source's type (D33, Unio D171).
    ForEach {
        item: VariableId,
        position: Option<(VariableId, PositionKind)>,
        source: Expression,
        body: Block,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
    },
    Break,
    Return(Option<Expression>),
    Fail(FailureId),
    /// `instead value`: the value an `otherwise` case carries on with (Unio D40).
    Instead(Expression),
}

#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub type_id: TypeId,
    pub place: Place,
}

/// How `be.Whole` takes a `Float`'s fraction (Unio D178).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    Floor,
    Trunc,
    Ceil,
    Nearest,
    NearestEven,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conversion {
    /// `be.Text` on a `Whole`, a `Float` or a `Bool`; which one is the value's type (Unio D22, D153).
    ToText,
    WholeToFloat,
    /// `be.Whole` on text, which fails with `not_a_whole` (Unio D95, D156).
    TextToWhole,
    /// `be.Bool` on text, which fails with `not_a_bool` (D101).
    TextToBool,
    /// A list moved into a name whose `len.max` it might not fit, which fails with `full` (D73).
    ListToLimit(i64),
    FloatToWhole(Rounding),
    /// A value moved into another unit of its group: multiplied by `size`, exactly for a `Whole` and rounded once
    /// for a `Float` (D120, D122).
    Scale {
        size: super::types::Size,
        from_whole: bool,
        to_float: bool,
    },
    /// Into or out of `HugeWhole`, `Decimal` or `HugeDecimal`: by itself where nothing can be lost, or with `be.`
    /// (D115, D116, D126).
    Number(NumberConversion),
}

/// One conversion between number types, or from text into one (D115, D116, D126).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberConversion {
    pub from: NumberType,
    pub to: NumberType,
    pub how: NumberHow,
}

/// What a number conversion converts from or to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberType {
    Whole,
    Float,
    Text,
    Huge,
    Decimal,
    HugeDecimal,
}

/// How a number conversion takes what doesn't fit exactly (D126).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberHow {
    /// Exactly, or to the nearest the type holds, a tie to even; or failing with `too_big`.
    Exact,
    /// Into a whole number, rounding a fraction as `be.Whole.floor` and the rest do (Unio D178).
    Round(Rounding),
    /// `be.Decimal.shown`: the number a `Float` shows.
    Shown,
    /// `be.Decimal.nearest`: the nearest `Decimal` to a `Float`'s value.
    Nearest,
}

impl NumberConversion {
    /// The failure it can give: `not_a_number` from text, `too_big` into a type that might not hold the value.
    #[must_use]
    pub fn failure(&self) -> Option<&'static str> {
        match (self.from, self.to) {
            (NumberType::Text, _) => Some("not_a_number"),
            (
                NumberType::Huge | NumberType::Decimal | NumberType::HugeDecimal,
                NumberType::Whole | NumberType::Float,
            )
            | (NumberType::Huge | NumberType::HugeDecimal, NumberType::Decimal) => Some("too_big"),
            _ => None,
        }
    }
}

/// The maths built-ins beyond `sqrt` (D107, D109, D114). `abs`, `min`, `max` and `clamp` work on a `Whole` or a
/// `Float`; the rest on `Float`s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathsFunction {
    Abs,
    Min,
    Max,
    Clamp,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan2,
    Lerp,
}

/// What comes after `??` on something that can fail (Unio D23).
#[derive(Clone, Debug)]
pub enum HandleFallback {
    Value(Box<Expression>),
    Ignore,
    Crash,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    /// A refused expression (Unio D55).
    Error,
    Whole(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    Variable(VariableId),
    /// `receiver.name`: a record's field, or the fact `.len`; which one is the receiver's type (D27, Unio D79).
    Member {
        receiver: Box<Expression>,
        name: String,
    },
    /// `list[pos: n]` or `list[offset: n]` (Unio D75).
    Item {
        receiver: Box<Expression>,
        index: Box<Expression>,
        zero_based: bool,
    },
    /// `dict[key: k]`, which fails with `missing` (D32).
    Lookup {
        dict: Box<Expression>,
        key: Box<Expression>,
    },
    /// `dict.has(key: k)` (D34).
    Has {
        dict: Box<Expression>,
        key: Box<Expression>,
    },
    /// `.add` or `.remove` on a part of a part of a `mut` name: `change` is made to `path.part` (D62).
    Inside {
        path: Box<Inside>,
        change: Box<Expression>,
    },
    /// `text.split(by: t)`, and `text.split.no_empty(by: t)`, which drops the empty pieces (D65).
    Split {
        text: Box<Expression>,
        by: Box<Expression>,
        keep_empty: bool,
    },
    /// A call to one of the script's procedures, its arguments in the parameters' order.
    Call {
        procedure: ProcedureId,
        arguments: Vec<Expression>,
    },
    /// A call to a door, by its place among the host's doors (D11).
    DoorCall {
        door: u32,
        arguments: Vec<Expression>,
    },
    Convert {
        conversion: Conversion,
        value: Box<Expression>,
    },
    Sqrt(Box<Expression>),
    /// A `HugeWhole`, `Decimal` or `HugeDecimal` written out (D115, D116, D119).
    Number(crate::decimal::Number),
    /// One of the maths built-ins, its arguments in their declared order (D107, D109, D114).
    Maths {
        function: MathsFunction,
        arguments: Vec<Expression>,
        /// For `sin`, `cos` and `tan`: 0 for radians, 1 for `degs`, 2 for `turns` (D113).
        unit: u8,
    },
    Negate(Box<Expression>),
    Not(Box<Expression>),
    /// Maths, comparing, `and` and `or`. Never `&`, which is [`ExpressionKind::Join`].
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// Pieces of text joined into one, as braces fill a text in (D75, D100).
    Join(Vec<Expression>),
    /// A `Float` shown to so many digits after its point, as `{speed, places: 2}` fills it in (D94, Unio D175).
    ShowPlaces {
        value: Box<Expression>,
        places: u8,
    },
    /// A record built from its fields, in the order written: each field's number and its value (D27).
    BuildRecord {
        record: u32,
        fields: Vec<(u32, Expression)>,
    },
    /// A case built from its values, in the order written (D35). The expression's type says which `one_of`.
    BuildCase {
        case: u32,
        fields: Vec<(u32, Expression)>,
    },
    List(Vec<Expression>),
    /// `range[…]` kept as a list (D99); a `for each` over one is [`StatementKind::ForRange`], which builds no list.
    Range {
        from: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
        step: Option<Box<Expression>>,
    },
    /// `[all: value]` for an array of `count` items (Unio D169).
    Repeat {
        value: Box<Expression>,
        count: i64,
    },
    Dict(Vec<(Expression, Expression)>),
    /// `call?` (Unio D23).
    Try(Box<Expression>),
    /// `call ?? fallback` on something that can fail (Unio D23).
    Handle {
        call: Box<Expression>,
        fallback: HandleFallback,
    },
    /// A value landing where a ranged number's type is written, checked or pulled back as it lands (D107, D128).
    Land {
        value: Box<Expression>,
        /// The ranged type as written, with its limits.
        target: TypeId,
    },
    /// `maybe ?? crash`: the value inside, or a failed check when it's `none` (D79).
    MaybeCrash(Box<Expression>),
    /// `maybe ?? fallback` on a `Maybe` (D38).
    MaybeOr {
        maybe: Box<Expression>,
        fallback: Box<Expression>,
    },
    /// `call otherwise` and its cases (Unio D40), and the failures whose case came from a handler, which may be
    /// ones the call can't give (D131).
    Otherwise {
        call: Box<Expression>,
        cases: Vec<(FailureId, Block)>,
        shared: Vec<FailureId>,
    },
    /// `list.add(value)`, with a position when one is given. Fails with `full` when the list has `len.max`
    /// (Unio D80, D168).
    ListAdd {
        variable: VariableId,
        value: Box<Expression>,
        position: Option<(Box<Expression>, bool)>,
    },
    /// `list.remove(pos: n)` (Unio D80).
    ListRemove {
        variable: VariableId,
        position: Box<Expression>,
        zero_based: bool,
    },
    /// `dict.remove(key: k)`, which fails with `missing` (D34).
    DictRemove {
        variable: VariableId,
        key: Box<Expression>,
    },
}
