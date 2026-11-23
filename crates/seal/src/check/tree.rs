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
    pub failure_names: Vec<String>,
}

impl Program {
    #[must_use]
    pub fn failure_name(&self, failure: FailureId) -> &str {
        self.failure_names.get(usize::from(failure.0).wrapping_sub(1)).map_or("?", String::as_str)
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

#[derive(Clone, Debug)]
pub enum StatementKind {
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
        from: Expression,
        end: Expression,
        inclusive: bool,
        step: Option<Expression>,
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
    FloatToWhole(Rounding),
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
    Negate(Box<Expression>),
    Not(Box<Expression>),
    /// Maths, comparing, `and` and `or`. Never `&`, which is [`ExpressionKind::Join`].
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// Pieces of text joined into one (D29, D12).
    Join(Vec<Expression>),
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
    /// `maybe ?? fallback` on a `Maybe` (D38).
    MaybeOr {
        maybe: Box<Expression>,
        fallback: Box<Expression>,
    },
    /// `call otherwise` and its cases (Unio D40).
    Otherwise {
        call: Box<Expression>,
        cases: Vec<(FailureId, Block)>,
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
