//! The syntax tree: a script as the parser read it, before any checking.

use crate::message::Place;

/// A whole script: its top level, in order (D4).
#[derive(Clone, Debug, Default)]
pub struct Script<'a> {
    pub items: Vec<Item<'a>>,
    /// How many expressions the script has; each has an `id` below this.
    pub expression_count: u32,
    /// The script as written, for fixes that show a whole line put right.
    pub source: &'a str,
}

#[derive(Clone, Debug)]
pub enum Item<'a> {
    Statement(Statement<'a>),
    Procedure(Procedure<'a>),
    Family(Family<'a>),
    Record(RecordDeclaration<'a>),
    OneOf(OneOfDeclaration<'a>),
    HandlerGroup(HandlerGroup<'a>),
    UnitGroup(UnitGroup<'a>),
}

/// `group.unit name` and its members, each `::name base` or `::name = amount unit` (D123).
#[derive(Clone, Debug)]
pub struct UnitGroup<'a> {
    pub name: Word<'a>,
    pub members: Vec<UnitMember<'a>>,
    pub place: Place,
}

/// One unit of a group a script declares: its name, and how many of an earlier unit it is, or `base` (D123).
#[derive(Clone, Debug)]
pub struct UnitMember<'a> {
    pub name: Word<'a>,
    /// `None` for the base; otherwise the amount, as written, and the unit it counts.
    pub size: Option<(Expression<'a>, UnitName<'a>)>,
}

/// A unit's name, with its group when it's written to settle a clash, as `time::secs` (D112).
#[derive(Clone, Debug)]
pub struct UnitName<'a> {
    pub group: Option<Word<'a>>,
    pub name: Word<'a>,
}

/// A unit as written after a number or in a type: `secs`, `px.point`, or a rate as `px per secs` (D112, D120,
/// D121).
#[derive(Clone, Debug)]
pub struct UnitWritten<'a> {
    pub top: UnitName<'a>,
    pub point: bool,
    pub per: Option<UnitName<'a>>,
    pub place: Place,
}

/// `group.handler name`, with its `handles [ … ]` line and its members (D110).
#[derive(Clone, Debug)]
pub struct HandlerGroup<'a> {
    pub name: Word<'a>,
    /// The procedures and doors it's for, by their full names (D110).
    pub handles: Vec<Vec<Word<'a>>>,
    pub members: Vec<Handler<'a>>,
    pub place: Place,
}

/// `::name[parameters]` in a handler group, and its cases (D97, D110, Unio D42, D44).
#[derive(Clone, Debug)]
pub struct Handler<'a> {
    pub name: Word<'a>,
    pub parameters: Vec<Parameter<'a>>,
    pub labels: Vec<LabelDeclaration<'a>>,
    pub labels_any_order: bool,
    pub defaults: Vec<DefaultDeclaration<'a>>,
    pub cases: Vec<OtherwiseCase<'a>>,
    pub place: Place,
}

/// A handler where it's used: `::name`, `group::name`, with `*` when it can jump, and its arguments (D98, D110, Unio
/// D42, D44).
#[derive(Clone, Debug)]
pub struct HandlerUse<'a> {
    pub group: Option<Word<'a>>,
    pub name: Word<'a>,
    pub jumps: bool,
    pub arguments: Vec<Entry<'a>>,
    pub place: Place,
}

/// A name with the place it's written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word<'a> {
    pub text: &'a str,
    pub place: Place,
}

/// `proc name[parameters] -> Type ! [failures]` and its body (D21, D45).
#[derive(Clone, Debug)]
pub struct Procedure<'a> {
    /// The name with its narrowings, as `spawn` and `boss` for `spawn.boss` (D43).
    pub path: Vec<Word<'a>>,
    pub parameters: Vec<Parameter<'a>>,
    pub result: Option<TypeExpression<'a>>,
    pub failures: Option<Vec<Word<'a>>>,
    pub labels: Vec<LabelDeclaration<'a>>,
    /// Written `labels.any_order`: the labelled arguments go in any order (D87).
    pub labels_any_order: bool,
    pub defaults: Vec<DefaultDeclaration<'a>>,
    pub body: Block<'a>,
    pub place: Place,
    pub end_place: Place,
}

impl Script<'_> {
    /// The script's line as written, by its number, counting from 1.
    #[must_use]
    pub fn line(&self, number: u32) -> &str {
        let line = self.source.split('\n').nth(number.saturating_sub(1) as usize).unwrap_or("");
        line.strip_suffix('\r').unwrap_or(line)
    }
}

impl<'a> Procedure<'a> {
    /// The full name, with its narrowings joined by `.`, borrowed from the script when it's one word.
    #[must_use]
    pub fn name(&self) -> std::borrow::Cow<'a, str> {
        match self.path.as_slice() {
            [word] => std::borrow::Cow::Borrowed(word.text),
            path => std::borrow::Cow::Owned(path.iter().map(|word| word.text).collect::<Vec<_>>().join(".")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Parameter<'a> {
    pub name: Word<'a>,
    pub written_type: Option<TypeExpression<'a>>,
    /// Marked `unused`: never read, on purpose (D61).
    pub unused: bool,
}

/// `parameter as label` on a `labels` line (D41, D42).
#[derive(Clone, Debug)]
pub struct LabelDeclaration<'a> {
    pub parameter: Word<'a>,
    pub label: Word<'a>,
}

/// `label: value` on a `defaults` line (D42).
#[derive(Clone, Debug)]
pub struct DefaultDeclaration<'a> {
    pub label: Word<'a>,
    pub value: Expression<'a>,
}

/// `fam.proc word` and its members (D43).
#[derive(Clone, Debug)]
pub struct Family<'a> {
    pub word: Word<'a>,
    pub members: Vec<Procedure<'a>>,
    pub place: Place,
}

/// `record Name` and its fields (D27).
#[derive(Clone, Debug)]
pub struct RecordDeclaration<'a> {
    pub name: Word<'a>,
    pub fields: Vec<FieldDeclaration<'a>>,
}

#[derive(Clone, Debug)]
pub struct FieldDeclaration<'a> {
    pub name: Word<'a>,
    pub written_type: TypeExpression<'a>,
}

/// `one_of Name` and its cases (D35, D36).
#[derive(Clone, Debug)]
pub struct OneOfDeclaration<'a> {
    pub name: Word<'a>,
    pub cases: Vec<CaseDeclaration<'a>>,
}

#[derive(Clone, Debug)]
pub struct CaseDeclaration<'a> {
    pub name: Word<'a>,
    pub fields: Vec<FieldDeclaration<'a>>,
}

/// A type as written, as `List<Text>[len.max: 10]` (D30, D31, Unio D167).
#[derive(Clone, Debug)]
pub struct TypeExpression<'a> {
    pub name: Word<'a>,
    /// What's inside `< >`, each with its label when it has one (D31).
    pub blanks: Vec<(Option<Word<'a>>, TypeExpression<'a>)>,
    /// What's inside `[ ]` after the type: its settings, as `len.max: 10`.
    pub settings: Vec<(Vec<Word<'a>>, Expression<'a>)>,
    pub place: Place,
}

pub type Block<'a> = Vec<Statement<'a>>;

#[derive(Clone, Debug)]
pub struct Statement<'a> {
    pub kind: StatementKind<'a>,
    pub place: Place,
}

/// The word in front of a new name (D2, D3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    None,
    Mut,
    Unused,
}

#[derive(Clone, Debug)]
pub enum StatementKind<'a> {
    /// `name = value`, `mut name = value`, `unused name = value`, with the type optional (D2, D3, D30).
    /// With no marker and no type, it changes `name` when that already exists (D2).
    Name {
        marker: Marker,
        name: Word<'a>,
        written_type: Option<Box<TypeExpression<'a>>>,
        value: Expression<'a>,
    },
    /// `goblin.hp = value` or `bag[pos: 1] = value`: changing part of a name's value (D27, D32, Unio D80).
    Change {
        place: Expression<'a>,
        value: Expression<'a>,
    },
    /// A call standing alone.
    Expression(Expression<'a>),
    If {
        branches: Vec<(Expression<'a>, Block<'a>)>,
        otherwise: Option<Block<'a>>,
    },
    While {
        condition: Expression<'a>,
        body: Block<'a>,
    },
    ForEach {
        item: Word<'a>,
        /// The item is marked `unused`: never read, on purpose (D61).
        item_unused: bool,
        position: Option<(PositionKind, Word<'a>)>,
        /// The `at` name is marked `unused` (D86).
        position_unused: bool,
        source: Expression<'a>,
        body: Block<'a>,
    },
    Match {
        subject: Expression<'a>,
        cases: Vec<MatchCase<'a>>,
    },
    Break,
    Return(Option<Expression<'a>>),
    Fail(Word<'a>),
    /// `instead value`, ending an `otherwise` case (Unio D40).
    Instead(Expression<'a>),
}

/// What `at …` in a `for each` gives (D33, Unio D174).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionKind {
    Pos,
    Offset,
    Key,
}

/// `case name[values]` in a `match`, and its body (D37).
#[derive(Clone, Debug)]
pub struct MatchCase<'a> {
    pub name: Word<'a>,
    pub values: Vec<Word<'a>>,
    pub body: Block<'a>,
}

/// `case failure` in an `otherwise`, and its body (Unio D40).
#[derive(Clone, Debug)]
pub struct OtherwiseCase<'a> {
    pub failure: Word<'a>,
    pub body: Block<'a>,
}

#[derive(Clone, Debug)]
pub struct Expression<'a> {
    /// A number of its own within the script, for what checking works out about it.
    pub id: u32,
    pub kind: ExpressionKind<'a>,
    pub place: Place,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    /// `**` (D108).
    Power,
    DivFloor,
    DivTrunc,
    RemFloor,
    RemTrunc,
    Join,
    Equal,
    NotEqual,
    Less,
    Greater,
    AtLeast,
    AtMost,
    And,
    Or,
}

impl<'a> ExpressionKind<'a> {
    /// A call's callee and arguments.
    #[must_use]
    pub fn as_call(&self) -> Option<(&Expression<'a>, &[Entry<'a>])> {
        match self {
            ExpressionKind::Call { parts } => {
                let (callee, arguments) = parts.split_first()?;
                Some((&callee.value, arguments))
            }
            _ => None,
        }
    }
}

impl BinaryOperator {
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Power => "**",
            BinaryOperator::DivFloor => "div.floor",
            BinaryOperator::DivTrunc => "div.trunc",
            BinaryOperator::RemFloor => "rem.floor",
            BinaryOperator::RemTrunc => "rem.trunc",
            BinaryOperator::Join => "&",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::Greater => ">",
            BinaryOperator::AtLeast => "at_least",
            BinaryOperator::AtMost => "at_most",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
        }
    }

    #[must_use]
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::Less
                | BinaryOperator::Greater
                | BinaryOperator::AtLeast
                | BinaryOperator::AtMost
        )
    }

    #[must_use]
    pub fn is_maths(self) -> bool {
        matches!(
            self,
            BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Power
                | BinaryOperator::DivFloor
                | BinaryOperator::DivTrunc
                | BinaryOperator::RemFloor
                | BinaryOperator::RemTrunc
        )
    }
}

/// One thing between a call's `( )` or inside `[ ]`: a value, with its label when it has one.
#[derive(Clone, Debug)]
pub struct Entry<'a> {
    /// A plain label, as `by` in `by: 10`.
    pub label: Option<Word<'a>>,
    pub value: Expression<'a>,
}

/// What comes after `??` (Unio D23).
#[derive(Clone, Debug)]
pub enum Fallback<'a> {
    Value(Box<Expression<'a>>),
    Ignore,
    Crash,
    /// A handler, on one line (D98).
    Handler(Box<HandlerUse<'a>>),
}

/// One step of a slide: a procedure, with any arguments beyond the value slid in (Unio D5).
#[derive(Clone, Debug)]
pub struct SlideStep<'a> {
    pub target: Expression<'a>,
    pub arguments: Option<Vec<Entry<'a>>>,
    pub place: Place,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind<'a> {
    /// A number with no point, as its digits (Unio D180).
    Whole(&'a str),
    /// A number with a point: its text, and whether `~` was written (Unio D155).
    Float {
        text: &'a str,
        approximate: bool,
    },
    Bool(bool),
    Text(&'a str),
    Name(&'a str),
    /// A capitalised word standing as a value's start, as `Event` in `Event.idle`.
    TypeName(Word<'a>),
    /// `receiver.name`: a field, a fact, a narrowing or a case, sorted out by checking.
    Member {
        receiver: Box<Expression<'a>>,
        name: Word<'a>,
    },
    /// `callee(arguments)`: the callee first, with no label, then the arguments, in one list so a call is one
    /// allocation (D140). Read with [`ExpressionKind::as_call`].
    Call {
        parts: Vec<Entry<'a>>,
    },
    /// `receiver[entries]`, all labelled: reaching an item, building a record or a case, or a `range`.
    Bracketed {
        receiver: Box<Expression<'a>>,
        entries: Vec<Entry<'a>>,
    },
    /// A number written with its unit, as `2 secs` (D112).
    Measured {
        number: Box<Expression<'a>>,
        unit: Box<UnitWritten<'a>>,
    },
    /// A unit standing alone, as a type's `unit:` setting (D112).
    Unit(Box<UnitWritten<'a>>),
    /// `value in unit`: the plain number a value is when measured in a unit of its group (D122).
    MeasureIn {
        value: Box<Expression<'a>>,
        unit: Box<UnitWritten<'a>>,
    },
    /// `[a, b, c]` (Unio D75); empty for `[]`.
    List(Vec<Expression<'a>>),
    /// `[key: value, …]` (D32), and `[all: value]` (Unio D169), which checking tells apart.
    Pairs(Vec<(Expression<'a>, Expression<'a>)>),
    /// `[:]` (D32).
    EmptyDict,
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression<'a>>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression<'a>>,
        right: Box<Expression<'a>>,
        operator_place: Place,
    },
    /// `( … )`: grouping in maths (Unio D165).
    Group(Box<Expression<'a>>),
    /// `call?` (Unio D23).
    Try(Box<Expression<'a>>),
    /// `call ?? fallback` (Unio D23).
    Handled {
        call: Box<Expression<'a>>,
        fallback: Fallback<'a>,
    },
    /// `call otherwise` and its cases (Unio D40).
    Otherwise {
        call: Box<Expression<'a>>,
        cases: Vec<OtherwiseCase<'a>>,
        /// The handlers brought in with `include` (Unio D43).
        includes: Vec<HandlerUse<'a>>,
    },
    /// `start |> step |> step |> step` (Unio D5, D128).
    Slide {
        start: Box<Expression<'a>>,
        steps: Vec<SlideStep<'a>>,
    },
}
