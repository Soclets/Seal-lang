//! The syntax tree: a script as the parser read it, before any checking.

use crate::message::Place;

/// A whole script: its top level, in order (D4).
#[derive(Clone, Debug, Default)]
pub struct Script {
    pub items: Vec<Item>,
    /// How many expressions the script has; each has an `id` below this.
    pub expression_count: u32,
    /// The script as written, for fixes that show a whole line put right.
    pub source: String,
}

#[derive(Clone, Debug)]
pub enum Item {
    Statement(Statement),
    Procedure(Procedure),
    Family(Family),
    Record(RecordDeclaration),
    OneOf(OneOfDeclaration),
}

/// A name with the place it's written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub place: Place,
}

/// `proc name[parameters] -> Type ! [failures]` and its body (D21, D45).
#[derive(Clone, Debug)]
pub struct Procedure {
    /// The name with its narrowings, as `spawn` and `boss` for `spawn.boss` (D43).
    pub path: Vec<Word>,
    pub parameters: Vec<Parameter>,
    pub result: Option<TypeExpression>,
    pub failures: Option<Vec<Word>>,
    pub labels: Vec<LabelDeclaration>,
    /// Written `labels.any_order`: the labelled arguments go in any order (D87).
    pub labels_any_order: bool,
    pub defaults: Vec<DefaultDeclaration>,
    pub body: Block,
    pub place: Place,
    pub end_place: Place,
}

impl Script {
    /// The script's line as written, by its number, counting from 1.
    #[must_use]
    pub fn line(&self, number: u32) -> &str {
        let line = self.source.split('\n').nth(number.saturating_sub(1) as usize).unwrap_or("");
        line.strip_suffix('\r').unwrap_or(line)
    }
}

impl Procedure {
    /// The full name, with its narrowings joined by `.`.
    #[must_use]
    pub fn name(&self) -> String {
        self.path.iter().map(|word| word.text.as_str()).collect::<Vec<_>>().join(".")
    }
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: Word,
    pub written_type: Option<TypeExpression>,
    /// Marked `unused`: never read, on purpose (D61).
    pub unused: bool,
}

/// `parameter as label` on a `labels` line (D41, D42).
#[derive(Clone, Debug)]
pub struct LabelDeclaration {
    pub parameter: Word,
    pub label: Word,
}

/// `label: value` on a `defaults` line (D42).
#[derive(Clone, Debug)]
pub struct DefaultDeclaration {
    pub label: Word,
    pub value: Expression,
}

/// `fam.proc word` and its members (D43).
#[derive(Clone, Debug)]
pub struct Family {
    pub word: Word,
    pub members: Vec<Procedure>,
    pub place: Place,
}

/// `record Name` and its fields (D27).
#[derive(Clone, Debug)]
pub struct RecordDeclaration {
    pub name: Word,
    pub fields: Vec<FieldDeclaration>,
}

#[derive(Clone, Debug)]
pub struct FieldDeclaration {
    pub name: Word,
    pub written_type: TypeExpression,
}

/// `one_of Name` and its cases (D35, D36).
#[derive(Clone, Debug)]
pub struct OneOfDeclaration {
    pub name: Word,
    pub cases: Vec<CaseDeclaration>,
}

#[derive(Clone, Debug)]
pub struct CaseDeclaration {
    pub name: Word,
    pub fields: Vec<FieldDeclaration>,
}

/// A type as written, as `List<Text>[len.max: 10]` (D30, D31, Unio D167).
#[derive(Clone, Debug)]
pub struct TypeExpression {
    pub name: Word,
    /// What's inside `< >`, each with its label when it has one (D31).
    pub blanks: Vec<(Option<Word>, TypeExpression)>,
    /// What's inside `[ ]` after the type: its settings, as `len.max: 10`.
    pub settings: Vec<(Vec<Word>, Expression)>,
    pub place: Place,
}

pub type Block = Vec<Statement>;

#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
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
pub enum StatementKind {
    /// `name = value`, `mut name = value`, `unused name = value`, with the type optional (D2, D3, D30).
    /// With no marker and no type, it changes `name` when that already exists (D2).
    Name {
        marker: Marker,
        name: Word,
        written_type: Option<TypeExpression>,
        value: Expression,
    },
    /// `goblin.hp = value` or `bag[pos: 1] = value`: changing part of a name's value (D27, D32, Unio D80).
    Change {
        place: Expression,
        value: Expression,
    },
    /// A call standing alone.
    Expression(Expression),
    If {
        branches: Vec<(Expression, Block)>,
        otherwise: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    ForEach {
        item: Word,
        /// The item is marked `unused`: never read, on purpose (D61).
        item_unused: bool,
        position: Option<(PositionKind, Word)>,
        source: Expression,
        body: Block,
    },
    Match {
        subject: Expression,
        cases: Vec<MatchCase>,
    },
    Break,
    Return(Option<Expression>),
    Fail(Word),
    /// `instead value`, ending an `otherwise` case (Unio D40).
    Instead(Expression),
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
pub struct MatchCase {
    pub name: Word,
    pub values: Vec<Word>,
    pub body: Block,
}

/// `case failure` in an `otherwise`, and its body (Unio D40).
#[derive(Clone, Debug)]
pub struct OtherwiseCase {
    pub failure: Word,
    pub body: Block,
}

#[derive(Clone, Debug)]
pub struct Expression {
    /// A number of its own within the script, for what checking works out about it.
    pub id: u32,
    pub kind: ExpressionKind,
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

impl BinaryOperator {
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
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
                | BinaryOperator::DivFloor
                | BinaryOperator::DivTrunc
                | BinaryOperator::RemFloor
                | BinaryOperator::RemTrunc
        )
    }
}

/// One thing between a call's `( )` or inside `[ ]`: a value, with its label when it has one.
#[derive(Clone, Debug)]
pub struct Entry {
    /// A plain label, as `by` in `by: 10`.
    pub label: Option<Word>,
    pub value: Expression,
}

/// What comes after `??` (Unio D23).
#[derive(Clone, Debug)]
pub enum Fallback {
    Value(Box<Expression>),
    Ignore,
    Crash,
}

/// One step of a slide: a procedure, with any arguments beyond the value slid in (Unio D5).
#[derive(Clone, Debug)]
pub struct SlideStep {
    pub target: Expression,
    pub arguments: Option<Vec<Entry>>,
    pub place: Place,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    /// A number with no point, as its digits (Unio D180).
    Whole(String),
    /// A number with a point: its text, and whether `~` was written (Unio D155).
    Float {
        text: String,
        approximate: bool,
    },
    Bool(bool),
    Text(String),
    Name(String),
    /// A capitalised word standing as a value's start, as `Event` in `Event.idle`.
    TypeName(Word),
    /// `receiver.name`: a field, a fact, a narrowing or a case, sorted out by checking.
    Member {
        receiver: Box<Expression>,
        name: Word,
    },
    /// `callee(arguments)`.
    Call {
        callee: Box<Expression>,
        arguments: Vec<Entry>,
    },
    /// `receiver[entries]`, all labelled: reaching an item, building a record or a case, or a `range`.
    Bracketed {
        receiver: Box<Expression>,
        entries: Vec<Entry>,
    },
    /// `[a, b, c]` (Unio D75); empty for `[]`.
    List(Vec<Expression>),
    /// `[key: value, …]` (D32), and `[all: value]` (Unio D169), which checking tells apart.
    Pairs(Vec<(Expression, Expression)>),
    /// `[:]` (D32).
    EmptyDict,
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
        operator_place: Place,
    },
    /// `( … )`: grouping in maths (Unio D165).
    Group(Box<Expression>),
    /// `call?` (Unio D23).
    Try(Box<Expression>),
    /// `call ?? fallback` (Unio D23).
    Handled {
        call: Box<Expression>,
        fallback: Fallback,
    },
    /// `call otherwise` and its cases (Unio D40).
    Otherwise {
        call: Box<Expression>,
        cases: Vec<OtherwiseCase>,
    },
    /// `start |> step |> step |> step` (Unio D5, D128).
    Slide {
        start: Box<Expression>,
        steps: Vec<SlideStep>,
    },
}
