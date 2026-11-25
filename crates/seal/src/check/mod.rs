//! Checking a script at load time: names, `mut` and `unused`, types worked out, failures and pauses worked out, and
//! every refusal (D17). The result is the checked tree the compiler reads.

mod calls;
mod expressions;
mod finish;
mod handlers;
pub mod tree;
pub mod types;

use std::collections::{HashMap, HashSet};

use crate::message::{Message, MessageKind, Place};
use crate::signature::{DoorSignature, HostType};
use crate::syntax::tree as syntax;
use tree::{
    Block, Expression, ExpressionKind, FailureId, Inside, MatchArm, PathStep, Procedure, ProcedureId, Program,
    Statement, StatementKind, Variable, VariableId, VariableKind,
};
use types::{BOOL, CaseType, ERROR, FLOAT, NOTHING, OneOfType, RecordType, TEXT, Type, TypeId, WHOLE};

/// The words Seal provides itself (D11, D22, D107, D109, D114, D118). A script's own names can't take them.
const BUILT_IN_WORDS: [&str; 17] = [
    "range", "sqrt", "be", "newln", "tab", "abs", "min", "max", "clamp", "sin", "cos", "tan", "asin", "acos", "atan2",
    "pi", "lerp",
];

/// The types Seal provides itself (D16, D32, D38).
const BUILT_IN_TYPES: [&str; 8] = ["Whole", "Float", "Bool", "Text", "List", "Array", "Dict", "Maybe"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProcedureState {
    Unchecked,
    InProgress,
    Checked,
}

/// Something a type must allow, checked once the type is worked out (D5).
/// How a statement can leave the block it's in (D68).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Leaving {
    /// It always leaves: a `return`, a `break`, a `fail`, or an `if` or `match` whose every way does.
    Always,
    /// Some way through an `if` leaves (Unio D9).
    SomeWayOfIf,
    /// Some case of a `match` leaves (D86).
    SomeWayOfMatch,
    /// An `instead`, which ends its case (D80).
    Instead,
}

/// Why a spelling can't be taken twice (D71).
const NO_SHADOWING: &str = "one spelling means one thing wherever it's seen.";

/// Why the top level can't hand anything to a caller (D67).
pub(super) const TOP_LEVEL: &str = "the top level runs once, when the script loads, with nobody calling it.";

#[derive(Clone, Debug)]
enum Requirement {
    /// `Whole` or `Float`: maths and ordering.
    Number,
    /// `Whole` or `Float`: `<`, `>`, `<=` and `>=`. Ordering two texts isn't decided (D64).
    Ordered,
    /// `Whole`, `Float`, `Bool` or `Text`: `==` and `!=` (D64).
    Comparable,
    /// What `&` joins: `Text`, `Whole` or `Bool` (D29).
    Joinable,
    /// What `be.Text` converts: `Whole`, `Float` or `Bool`.
    Textable,
    /// Has the field or fact `name`, of type `result` (D27, Unio D79).
    Member { name: String, result: TypeId },
    /// Reached with `pos:` or `offset:`, giving `item` (Unio D75).
    Indexable { item: TypeId },
    /// Gone through by `for each`, giving `item` (D33, Unio D171).
    Iterable { item: TypeId },
}

#[derive(Clone, Debug)]
struct Pending {
    subject: TypeId,
    requirement: Requirement,
    place: Place,
    /// What the value is for, as "`+` adds numbers", for the refusal.
    purpose: String,
}

/// An argument whose parameter's type wasn't worked out yet at the call (D5).
#[derive(Clone, Debug)]
struct DeferredArgument {
    argument: TypeId,
    parameter: TypeId,
    place: Place,
    purpose: String,
}

#[derive(Default)]
struct Context {
    procedure: Option<ProcedureId>,
    scopes: Vec<HashMap<String, VariableId>>,
    loop_depth: u32,
    /// The names being gone through by a `for each`, which can't change inside it (Unio D173).
    iterating: Vec<VariableId>,
    /// Whether a `return` with a value, or one without, has been seen.
    returned_value: Option<bool>,
    /// The type an `instead` must give, innermost `otherwise` last.
    instead_types: Vec<TypeId>,
}

pub(crate) struct Checker<'a> {
    doors: &'a [DoorSignature],
    /// The records and `one_of`s the host opens, used with nothing declared (D85).
    host_types: &'a [HostType],
    /// Top-level names made with no `mut` from a value written out plainly, which are known while loading (D103,
    /// D104), and the top-level names made with `mut`, which aren't.
    plain_names: HashMap<String, &'a syntax::Expression>,
    changing_names: HashSet<String>,
    /// The handler groups, by name (D110).
    handler_groups: HashMap<String, &'a syntax::HandlerGroup>,
    /// The handlers whose cases are being checked where they're used, so that one can't use itself.
    handlers_in_use: Vec<String>,
    /// Their names.
    host_type_names: HashSet<String>,
    /// Whether a type being read may leave out the parts its value shows: a name's written type (D82).
    partial_types: bool,
    /// The script as it's written, for fixes that show a whole line put right.
    script: &'a syntax::Script,
    /// Each door's types in the script's terms, worked out the first time the door is used.
    door_types: HashMap<u32, (Vec<TypeId>, TypeId)>,
    messages: Vec<Message>,
    program: Program,
    procedure_syntax: Vec<&'a syntax::Procedure>,
    procedure_state: Vec<ProcedureState>,
    procedure_names: HashMap<String, ProcedureId>,
    /// The first word of every procedure and door, for telling an unknown narrowing from an unknown word.
    words: HashSet<String>,
    record_names: HashMap<String, u32>,
    one_of_names: HashMap<String, u32>,
    failure_ids: HashMap<String, FailureId>,
    /// The top level's own names, known before they're made so that procedures can use them (D4).
    globals: HashMap<String, VariableId>,
    made: HashSet<VariableId>,
    context: Context,
    pending: Vec<Pending>,
    deferred: Vec<DeferredArgument>,
    demand_depth: u32,
    batch: Vec<ProcedureId>,
    top_level_position: usize,
    /// Each procedure's written failures, to compare with what its body does (D44).
    written_failures: Vec<Option<Vec<syntax::Word>>>,
    /// The first place of each top-level name that isn't decided for use before it's made.
    reported_early_use: HashSet<VariableId>,
    /// The top-level line being checked, for saying which line reached a procedure (D67).
    top_level_line: Option<Place>,
    /// Set once the top level and every procedure have been checked.
    every_procedure_checked: bool,
    /// How many slides are being checked as nested calls, which Unio D128's refusal of nested calls leaves alone.
    slide_depth: u32,
    /// The calls already judged as part of a chain (Unio D128), by their expressions' numbers.
    judged_chains: HashSet<u32>,
}

/// Checks a script against the doors its host opened.
///
/// # Errors
///
/// Returns every message, when the script is refused or uses something not designed or not built yet.
pub fn check(
    script: &syntax::Script,
    doors: &[DoorSignature],
    host_types: &[HostType],
) -> Result<Program, Vec<Message>> {
    let mut checker = Checker {
        doors,
        host_types,
        host_type_names: HashSet::new(),
        partial_types: false,
        plain_names: HashMap::new(),
        changing_names: HashSet::new(),
        handler_groups: HashMap::new(),
        handlers_in_use: Vec::new(),
        script,
        door_types: HashMap::new(),
        messages: Vec::new(),
        program: Program { types: types::TypeTable::new(), ..Program::default() },
        procedure_syntax: Vec::new(),
        procedure_state: Vec::new(),
        procedure_names: HashMap::new(),
        words: HashSet::new(),
        record_names: HashMap::new(),
        one_of_names: HashMap::new(),
        failure_ids: HashMap::new(),
        globals: HashMap::new(),
        made: HashSet::new(),
        context: Context::default(),
        pending: Vec::new(),
        deferred: Vec::new(),
        demand_depth: 0,
        batch: Vec::new(),
        top_level_position: 0,
        written_failures: Vec::new(),
        reported_early_use: HashSet::new(),
        top_level_line: None,
        every_procedure_checked: false,
        slide_depth: 0,
        judged_chains: HashSet::new(),
    };
    checker.run(script);
    if checker.messages.is_empty() {
        Ok(checker.program)
    } else {
        // Refusals first, in the script's order; one mistake's knock-on stops are still shown (Unio D54).
        checker.messages.sort_by_key(|message| (message.place.line, message.place.column));
        checker.messages.dedup();
        Err(checker.messages)
    }
}

impl<'a> Checker<'a> {
    // ---- messages ----

    fn refuse(&mut self, place: Place, what: impl Into<String>) {
        self.messages.push(Message::refused(place, what));
    }

    fn refuse_why(&mut self, place: Place, what: impl Into<String>, why: &str) {
        self.messages.push(Message::refused(place, what).with_why(why));
    }

    fn not_designed(&mut self, place: Place, what: impl AsRef<str>) {
        self.messages.push(Message::not_designed_yet(place, what));
    }

    fn not_built(&mut self, place: Place, what: impl AsRef<str>) {
        self.messages.push(Message::not_built_yet(place, what));
    }

    fn has_refusals(&self) -> bool {
        self.messages.iter().any(|message| message.kind == MessageKind::Refused)
    }

    fn error(place: Place) -> Expression {
        Expression { kind: ExpressionKind::Error, type_id: ERROR, place }
    }

    fn describe(&self, type_id: TypeId) -> String {
        self.program.types.describe(type_id)
    }

    // ---- the whole script ----

    fn run(&mut self, script: &'a syntax::Script) {
        self.collect_plain_names(script);
        self.declare_globals(script);
        self.declare_types(script);
        self.declare_procedures(script);
        self.declare_handler_groups(script);
        self.context.scopes.push(HashMap::new());
        let mut top_level = Vec::new();
        let mut left: Option<(Place, Leaving)> = None;
        for item in &script.items {
            if let syntax::Item::Statement(statement) = item {
                if let Some((place, leaving)) = left.take() {
                    self.refuse_after_leaving(statement.place, place, leaving);
                }
                self.top_level_line = Some(statement.place);
                if let Some(checked) = self.statement(statement) {
                    left = Self::leaving(&checked).map(|leaving| (checked.place, leaving));
                    top_level.push(checked);
                }
                self.top_level_line = None;
                self.top_level_position += 1;
            }
        }
        // Procedures the top level never calls are still checked: the host may call them (D4).
        for index in 0..self.procedure_state.len() {
            self.ensure_checked(ProcedureId(u32::try_from(index).unwrap_or(u32::MAX)));
        }
        self.every_procedure_checked = true;
        self.solve(true);
        let scope = self.context.scopes.pop().unwrap_or_default();
        self.check_unused(&scope);
        self.program.top_level = top_level;
        if self.messages.is_empty() {
            self.finish();
        }
    }

    // ---- declarations ----

    /// trace: D85. The host's records and `one_of`s are there before the script's own, named as the host names them.
    fn declare_host_types(&mut self) {
        for host in self.host_types {
            match host {
                HostType::Record { name, .. } => {
                    let index = u32::try_from(self.program.types.records.len()).unwrap_or(u32::MAX);
                    self.program.types.records.push(RecordType { name: name.clone(), fields: Vec::new() });
                    self.record_names.insert(name.clone(), index);
                }
                HostType::OneOf { name, .. } => {
                    let index = u32::try_from(self.program.types.one_ofs.len()).unwrap_or(u32::MAX);
                    self.program.types.one_ofs.push(OneOfType { name: name.clone(), cases: Vec::new() });
                    self.one_of_names.insert(name.clone(), index);
                }
                _ => continue,
            }
            if let HostType::Record { name, .. } | HostType::OneOf { name, .. } = host {
                self.host_type_names.insert(name.clone());
            }
        }
        for host in self.host_types {
            match host {
                HostType::Record { name, fields } => {
                    let fields: Vec<(String, TypeId)> =
                        fields.iter().map(|(field, inner)| (field.clone(), self.host_type(inner))).collect();
                    if let Some(&index) = self.record_names.get(name)
                        && let Some(record) = self.program.types.records.get_mut(index as usize)
                    {
                        record.fields = fields;
                    }
                }
                HostType::OneOf { name, cases } => {
                    let cases: Vec<CaseType> = cases
                        .iter()
                        .map(|(case, fields)| CaseType {
                            name: case.clone(),
                            fields: fields
                                .iter()
                                .map(|(field, inner)| (field.clone(), self.host_type(inner)))
                                .collect(),
                        })
                        .collect();
                    if let Some(&index) = self.one_of_names.get(name)
                        && let Some(one_of) = self.program.types.one_ofs.get_mut(index as usize)
                    {
                        one_of.cases = cases;
                    }
                }
                _ => {}
            }
        }
    }

    fn declare_types(&mut self, script: &'a syntax::Script) {
        self.declare_host_types();
        let mut records = Vec::new();
        let mut one_ofs = Vec::new();
        for item in &script.items {
            let name = match item {
                syntax::Item::Record(record) => &record.name,
                syntax::Item::OneOf(one_of) => &one_of.name,
                _ => continue,
            };
            if self.host_type_names.contains(&name.text) {
                // trace: D71, D85
                self.refuse_why(
                    name.place,
                    format!("`{}` is one of the host's types, so the script can't declare it", name.text),
                    "the host opens its own types for a script, which uses them by name, as it uses a door.",
                );
                continue;
            }
            if BUILT_IN_TYPES.contains(&name.text.as_str()) {
                self.refuse_why(
                    name.place,
                    format!("`{}` is one of Seal's own types, so a script can't declare it", name.text),
                    "each type has one meaning.",
                );
                continue;
            }
            if self.record_names.contains_key(&name.text) || self.one_of_names.contains_key(&name.text) {
                self.refuse_why(
                    name.place,
                    format!("the type `{}` is declared twice", name.text),
                    "each type is declared once.",
                );
                continue;
            }
            match item {
                syntax::Item::Record(record) => {
                    let index = u32::try_from(self.program.types.records.len()).unwrap_or(u32::MAX);
                    self.program.types.records.push(RecordType { name: name.text.clone(), fields: Vec::new() });
                    self.record_names.insert(name.text.clone(), index);
                    records.push((index, record));
                }
                syntax::Item::OneOf(one_of) => {
                    let index = u32::try_from(self.program.types.one_ofs.len()).unwrap_or(u32::MAX);
                    self.program.types.one_ofs.push(OneOfType { name: name.text.clone(), cases: Vec::new() });
                    self.one_of_names.insert(name.text.clone(), index);
                    one_ofs.push((index, one_of));
                }
                _ => {}
            }
        }
        for (index, record) in records {
            // trace: D72. A record can hold a record: it's one bigger value.
            let fields = self.declare_fields(&record.fields);
            if let Some(slot) = self.program.types.records.get_mut(index as usize) {
                slot.fields = fields;
            }
        }
        for (index, one_of) in one_ofs {
            let mut cases: Vec<CaseType> = Vec::new();
            for case in &one_of.cases {
                if cases.iter().any(|earlier| earlier.name == case.name.text) {
                    self.refuse_why(
                        case.name.place,
                        format!("the case `{}` is listed twice", case.name.text),
                        "each case is listed once.",
                    );
                    continue;
                }
                let fields = self.declare_fields(&case.fields);
                cases.push(CaseType { name: case.name.text.clone(), fields });
            }
            if cases.is_empty() {
                // trace: D93
                self.refuse_why(
                    one_of.name.place,
                    format!("`{}` has no cases, so no value of it could exist", one_of.name.text),
                    "a `one_of`'s value is exactly one of its cases, listed one per line.",
                );
            }
            if let Some(slot) = self.program.types.one_ofs.get_mut(index as usize) {
                slot.cases = cases;
            }
        }
        self.refuse_types_holding_themselves(script);
    }

    fn declare_fields(&mut self, declarations: &[syntax::FieldDeclaration]) -> Vec<(String, TypeId)> {
        let mut fields: Vec<(String, TypeId)> = Vec::new();
        for declaration in declarations {
            if fields.iter().any(|(name, _)| *name == declaration.name.text) {
                self.refuse_why(
                    declaration.name.place,
                    format!("the field `{}` is listed twice", declaration.name.text),
                    "each field is listed once.",
                );
                continue;
            }
            let type_id = self.written_type(&declaration.written_type);
            fields.push((declaration.name.text.clone(), type_id));
        }
        fields
    }

    /// Whether a value of a type can end, given which records and `one_of`s are known to: a `List`, a `Dict` and a
    /// `Maybe` can be empty, and a plain value holds nothing more (D72).
    fn type_can_end(&self, type_id: TypeId, records: &[bool], one_ofs: &[bool]) -> bool {
        match self.program.types.get(self.program.types.resolve(type_id)) {
            Type::Record(record) => records.get(*record as usize).copied().unwrap_or(true),
            Type::OneOf(one_of) => one_ofs.get(*one_of as usize).copied().unwrap_or(true),
            Type::Array { item, length } => *length == 0 || self.type_can_end(*item, records, one_ofs),
            _ => true,
        }
    }

    /// trace: D72. A type can hold itself where a value can end: through a `one_of` case beside one that ends, or
    /// through a `List`, a `Dict` or a `Maybe`. A type whose every value would go on for ever is refused.
    fn refuse_types_holding_themselves(&mut self, script: &syntax::Script) {
        let mut records = vec![false; self.program.types.records.len()];
        let mut one_ofs = vec![false; self.program.types.one_ofs.len()];
        // What can end is found from the ground up: a type can end once everything it must hold can.
        loop {
            let mut changed = false;
            for (index, record) in self.program.types.records.iter().enumerate() {
                let ends = record.fields.iter().all(|(_, field)| self.type_can_end(*field, &records, &one_ofs));
                if ends && records.get(index) == Some(&false) {
                    if let Some(slot) = records.get_mut(index) {
                        *slot = true;
                    }
                    changed = true;
                }
            }
            for (index, one_of) in self.program.types.one_ofs.iter().enumerate() {
                let ends = one_of
                    .cases
                    .iter()
                    .any(|case| case.fields.iter().all(|(_, field)| self.type_can_end(*field, &records, &one_ofs)));
                if ends && one_ofs.get(index) == Some(&false) {
                    if let Some(slot) = one_ofs.get_mut(index) {
                        *slot = true;
                    }
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let why = "a type can hold itself only where a value can end: in a `one_of` beside a case that ends, or in a `List`, a `Dict` or a `Maybe`, as `next: Maybe<Node>`.";
        for item in &script.items {
            match item {
                syntax::Item::Record(record) => {
                    let Some(&index) = self.record_names.get(&record.name.text) else { continue };
                    if records.get(index as usize) != Some(&false) {
                        continue;
                    }
                    let fields = self
                        .program
                        .types
                        .records
                        .get(index as usize)
                        .map(|record| record.fields.clone())
                        .unwrap_or_default();
                    let endless = fields
                        .iter()
                        .zip(&record.fields)
                        .find(|((_, field), _)| !self.type_can_end(*field, &records, &one_ofs));
                    if let Some(((field, field_type), declaration)) = endless {
                        let found = self.describe(*field_type);
                        self.refuse_why(
                            declaration.written_type.place,
                            format!(
                                "`{}`'s field `{field}` holds {found}, with no way for a value to end",
                                record.name.text
                            ),
                            why,
                        );
                    }
                }
                syntax::Item::OneOf(one_of) => {
                    let Some(&index) = self.one_of_names.get(&one_of.name.text) else { continue };
                    if one_ofs.get(index as usize) == Some(&false) {
                        self.refuse_why(
                            one_of.name.place,
                            format!(
                                "every case of `{}` holds more that can't end, so no value of it could end",
                                one_of.name.text
                            ),
                            why,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// `Whole[…]` or `Float[…]` with its limits, `min:` and `max:` stopping the script past them and `min.clamp:` and
    /// `max.clamp:` pulling a value back, one of each kind per edge (D107).
    fn ranged_type(&mut self, written: &syntax::TypeExpression, float: bool) -> TypeId {
        let base = if float { FLOAT } else { WHOLE };
        let mut limits = types::Limits { min: None, max: None, float };
        for (path, value) in &written.settings {
            let setting = path.iter().map(|word| word.text.as_str()).collect::<Vec<_>>().join(".");
            let place = path.first().map_or(written.place, |word| word.place);
            let (is_min, clamp) = match setting.as_str() {
                "min" => (true, false),
                "max" => (false, false),
                "min.clamp" => (true, true),
                "max.clamp" => (false, true),
                _ => {
                    self.refuse_why(
                        place,
                        format!("`{}` has no setting `{setting}`", written.name.text),
                        "a number's settings are its limits: `min:`, `max:`, `min.clamp:` and `max.clamp:`.",
                    );
                    return ERROR;
                }
            };
            let Some(bits) = self.plain_number(value, float) else { return ERROR };
            let slot = if is_min { &mut limits.min } else { &mut limits.max };
            if slot.is_some() {
                let edge = if is_min { "lowest" } else { "highest" };
                self.refuse_why(
                    place,
                    format!("the {edge} edge is written twice"),
                    "each edge is written once, with `min:` or `min.clamp:`, and `max:` or `max.clamp:`.",
                );
                return ERROR;
            }
            *slot = Some(types::Edge { bits, clamp });
        }
        if let (Some(min), Some(max)) = (limits.min, limits.max) {
            let crossed = if float {
                f64::from_bits(min.bits) > f64::from_bits(max.bits)
            } else {
                min.bits.cast_signed() > max.bits.cast_signed()
            };
            if crossed {
                self.refuse_why(
                    written.place,
                    "this number's lowest edge is above its highest",
                    "a ranged number's `min` is at most its `max`, so some value fits.",
                );
                return ERROR;
            }
        }
        self.program.types.intern(Type::Ranged { base, limits })
    }

    /// A ranged number's edge: a number written there, `-` allowed, or a top-level name made from one with no `mut`,
    /// a `Whole` or a `Float` as the type is (D103, D107). Gives its bits.
    fn plain_number(&mut self, expression: &syntax::Expression, float: bool) -> Option<u64> {
        let written = match &expression.kind {
            syntax::ExpressionKind::Name(name) => {
                if let Some(&global) = self.globals.get(name) {
                    self.mark_read(global);
                }
                let Some(value) = self.plain_names.get(name).copied() else {
                    self.refuse_why(
                        expression.place,
                        format!("`{name}` isn't a top-level name made from a number written out, with no `mut`"),
                        "a number's limits are known while loading: a number written there, or a name made at the top level from one, with no `mut`.",
                    );
                    return None;
                };
                value
            }
            _ => expression,
        };
        let checked = self.expression(written, Some(if float { FLOAT } else { WHOLE }));
        match (&checked.kind, float) {
            (ExpressionKind::Whole(value), false) => Some(value.cast_unsigned()),
            (ExpressionKind::Float(value), true) => Some(value.to_bits()),
            (ExpressionKind::Error, _) => None,
            _ => {
                let wanted = if float { "a `Float`" } else { "a `Whole`" };
                self.refuse_why(
                    expression.place,
                    format!("this number's limit is {wanted} written out"),
                    "a number's limits are known while loading, and are of its own type.",
                );
                None
            }
        }
    }

    /// trace: D107, D128. A value landing where a ranged number's type is written: in a name, a field, an item, a
    /// parameter or a `return`. One Seal can see past a hard edge is refused; any other is checked, or pulled back
    /// with `.clamp`, as it lands.
    pub(super) fn land(&mut self, value: Expression, target: TypeId) -> Expression {
        let Some(limits) = self.program.types.limits(target) else { return value };
        let past = |edge: Option<types::Edge>, below: bool| {
            let edge = edge.filter(|edge| !edge.clamp)?;
            let outside = match value.kind {
                ExpressionKind::Whole(number) => {
                    let limit = edge.bits.cast_signed();
                    if below { number < limit } else { number > limit }
                }
                ExpressionKind::Float(number) => {
                    let limit = f64::from_bits(edge.bits);
                    if below { number < limit } else { number > limit }
                }
                _ => false,
            };
            outside.then_some(edge)
        };
        let shown = |limits: &types::Limits, edge: types::Edge| {
            if limits.float {
                crate::number::float_to_text(f64::from_bits(edge.bits))
            } else {
                edge.bits.cast_signed().to_string()
            }
        };
        for (edge, below) in [(limits.min, true), (limits.max, false)] {
            if let Some(edge) = past(edge, below) {
                let (word, side) = if below { ("min", "below") } else { ("max", "above") };
                self.refuse_why(
                    value.place,
                    format!("this is {side} its type's `{word}: {}`", shown(&limits, edge)),
                    "a ranged number's `min:` and `max:` are hard limits, and one written past them never fits; `min.clamp:` and `max.clamp:` pull a value back instead.",
                );
                return Self::error(value.place);
            }
        }
        let (type_id, place) = (value.type_id, value.place);
        Expression { kind: ExpressionKind::Land { value: Box::new(value), target }, type_id, place }
    }

    /// A type's setting: a whole number written there, or a top-level name made from one with no `mut` (D103).
    fn plain_whole(&mut self, expression: &syntax::Expression) -> Option<i64> {
        let written = match &expression.kind {
            syntax::ExpressionKind::Name(name) => {
                let value = self.plain_names.get(name).copied();
                if let Some(value) = value.filter(|value| matches!(value.kind, syntax::ExpressionKind::Whole(_))) {
                    if let Some(&global) = self.globals.get(name) {
                        self.mark_read(global);
                    }
                    value
                } else {
                    if let Some(&global) = self.globals.get(name) {
                        self.mark_read(global);
                    }
                    // trace: D103
                    let what = if self.changing_names.contains(name) {
                        format!("`{name}` is made with `mut`, so its value isn't known while loading")
                    } else {
                        format!("`{name}` isn't a top-level name made from a whole number written out")
                    };
                    self.refuse_why(
                        expression.place,
                        what,
                        "a type's setting is a whole number written there, or a name made at the top level from one, with no `mut`, as `max_items = 10`.",
                    );
                    return None;
                }
            }
            _ => expression,
        };
        if let syntax::ExpressionKind::Whole(digits) = &written.kind {
            if let Ok(value) = digits.parse::<i64>() {
                Some(value)
            } else {
                self.refuse(expression.place, format!("`{digits}` is too big for a `Whole`"));
                None
            }
        } else {
            // trace: D103
            self.refuse_why(
                expression.place,
                "a type's setting is a whole number known while loading, never a calculation or a call",
                "a type's setting is a whole number written there, or a name made at the top level from one, with no `mut`, as `max_items = 10`.",
            );
            None
        }
    }

    /// A type as written, in the script's terms (D30, D31).
    #[allow(clippy::too_many_lines)] // one case per type
    fn written_type(&mut self, written: &syntax::TypeExpression) -> TypeId {
        let name = written.name.text.as_str();
        let blank_count = match name {
            "List" | "Array" | "Maybe" => 1,
            "Dict" => 2,
            _ => 0,
        };
        // trace: D82. A name's written type may leave out the parts its value shows: they're worked out from it.
        let left_out = written.blanks.is_empty() && blank_count > 0 && self.partial_types;
        if written.blanks.len() != blank_count && !left_out {
            let what = match blank_count {
                0 => format!("`{name}` has no `< >`"),
                1 => format!("`{name}` takes one type in `< >`, as in `{name}<Text>`"),
                _ => "`Dict` takes a key and a value, as in `Dict<key: Text, value: Whole>`".to_owned(),
            };
            self.refuse(written.place, what);
            return ERROR;
        }
        if matches!(name, "Whole" | "Float") && !written.settings.is_empty() {
            return self.ranged_type(written, name == "Float");
        }
        let mut settings: HashMap<String, i64> = HashMap::new();
        for (path, value) in &written.settings {
            let setting = path.iter().map(|word| word.text.as_str()).collect::<Vec<_>>().join(".");
            let allowed = matches!((name, setting.as_str()), ("List", "len.max") | ("Array", "len"));
            let place = path.first().map_or(written.place, |word| word.place);
            if !allowed {
                self.refuse(place, format!("`{name}` has no setting `{setting}`"));
                return ERROR;
            }
            let Some(number) = self.plain_whole(value) else { return ERROR };
            if number < 0 {
                self.refuse(value.place, format!("`{setting}` can't be less than 0"));
                return ERROR;
            }
            if settings.insert(setting.clone(), number).is_some() {
                self.refuse(place, format!("`{setting}` is written twice"));
                return ERROR;
            }
        }
        match name {
            "Whole" => WHOLE,
            "Float" => FLOAT,
            "Bool" => BOOL,
            "Text" => TEXT,
            "List" | "Array" | "Maybe" => {
                let item = if left_out {
                    self.program.types.fresh()
                } else {
                    let Some((label, inner)) = written.blanks.first() else { return ERROR };
                    if let Some(label) = label {
                        // trace: D31
                        self.refuse_why(
                            label.place,
                            format!("`{name}` has one blank, so it takes no label"),
                            "a type with a single blank leaves it unlabelled, since nothing can be mixed up.",
                        );
                        return ERROR;
                    }
                    self.written_type(inner)
                };
                match name {
                    "List" => self.program.types.intern(Type::List { item, max: settings.get("len.max").copied() }),
                    "Maybe" => self.program.types.maybe(item),
                    _ => {
                        if let Some(&length) = settings.get("len") {
                            self.program.types.intern(Type::Array { item, length })
                        } else {
                            // trace: Unio D167
                            self.refuse_why(
                                written.place,
                                "an `Array` says how many items it holds, as in `Array<Whole>[len: 64]`",
                                "an array's length is part of its type.",
                            );
                            ERROR
                        }
                    }
                }
            }
            "Dict" if left_out => {
                let (key, value) = (self.program.types.fresh(), self.program.types.fresh());
                self.program.types.dict(key, value)
            }
            "Dict" => {
                let mut key = None;
                let mut value = None;
                for (label, inner) in &written.blanks {
                    let inner_type = self.written_type(inner);
                    match label.as_ref().map(|label| label.text.as_str()) {
                        Some("key") if key.is_none() => key = Some((inner_type, inner.place)),
                        Some("value") if value.is_none() => value = Some(inner_type),
                        _ => {
                            // trace: D31
                            self.refuse_why(
                                inner.place,
                                "a `Dict`'s blanks are labelled `key:` and `value:`, as in `Dict<key: Text, value: Whole>`",
                                "a type with more than one blank labels each one.",
                            );
                            return ERROR;
                        }
                    }
                }
                let (Some((key, key_place)), Some(value)) = (key, value) else { return ERROR };
                if !self.check_key_type(key, key_place) {
                    return ERROR;
                }
                self.program.types.dict(key, value)
            }
            _ => {
                if let Some(&record) = self.record_names.get(name) {
                    self.program.types.intern(Type::Record(record))
                } else if let Some(&one_of) = self.one_of_names.get(name) {
                    self.program.types.intern(Type::OneOf(one_of))
                } else {
                    self.refuse(written.name.place, format!("there's no type named `{name}`"));
                    ERROR
                }
            }
        }
    }

    /// trace: D34
    fn check_key_type(&mut self, key: TypeId, place: Place) -> bool {
        match self.program.types.get(key) {
            Type::Whole | Type::Text | Type::Bool | Type::Error | Type::Variable(_) => true,
            Type::Float => {
                self.refuse_why(
                    place,
                    "a `Float` can't be a `Dict`'s key",
                    "two neighbouring floats that print alike would be different keys.",
                );
                false
            }
            _ => {
                let written = self.describe(key);
                self.not_designed(place, format!("{written} as a `Dict`'s key"));
                false
            }
        }
    }

    fn failure(&mut self, name: &str) -> FailureId {
        if let Some(&id) = self.failure_ids.get(name) {
            return id;
        }
        self.program.failure_names.push(name.to_owned());
        let id = FailureId(u16::try_from(self.program.failure_names.len()).unwrap_or(u16::MAX));
        self.failure_ids.insert(name.to_owned(), id);
        id
    }

    fn new_variable(&mut self, variable: Variable) -> VariableId {
        let id = VariableId(u32::try_from(self.program.variables.len()).unwrap_or(u32::MAX));
        self.program.variables.push(variable);
        id
    }

    fn variable(&self, id: VariableId) -> Option<&Variable> {
        self.program.variables.get(id.0 as usize)
    }

    fn variable_type(&self, id: VariableId) -> TypeId {
        self.variable(id).map_or(ERROR, |variable| variable.type_id)
    }

    fn declare_procedures(&mut self, script: &'a syntax::Script) {
        for door in self.doors {
            self.words.insert(door.name.split('.').next().unwrap_or("").to_owned());
        }
        let mut list: Vec<&'a syntax::Procedure> = Vec::new();
        for item in &script.items {
            match item {
                syntax::Item::Procedure(procedure) => {
                    if let Some(narrowing) = procedure.path.get(1) {
                        // trace: D43
                        self.refuse_why(
                            narrowing.place,
                            format!(
                                "`{}` is a narrowing, so it's declared inside a `fam.proc` block",
                                procedure.name()
                            ),
                            "a word and its kinds are declared together.",
                        );
                    }
                    list.push(procedure);
                }
                syntax::Item::Family(family) => {
                    for member in &family.members {
                        if member.path.first().map(|word| word.text.as_str()) != Some(family.word.text.as_str()) {
                            // trace: D43
                            let place = member.path.first().map_or(member.place, |word| word.place);
                            self.refuse_why(
                                place,
                                format!("`{}` isn't `{}` or a kind of it", member.name(), family.word.text),
                                "each member of a family is the word itself, or reads as \"a Y kind of X\" for its name `X.Y`.",
                            );
                        }
                        list.push(member);
                    }
                }
                _ => {}
            }
        }
        for procedure in list {
            self.declare_procedure(procedure);
        }
    }

    #[allow(clippy::too_many_lines)] // a procedure's first line, its labels and its defaults, in order
    fn declare_procedure(&mut self, procedure: &'a syntax::Procedure) {
        let name = procedure.name();
        let name_place = procedure.path.first().map_or(procedure.place, |word| word.place);
        let word = procedure.path.first().map(|word| word.text.clone()).unwrap_or_default();
        // trace: D71, D93. One spelling means one thing, so a procedure can't take a built-in's or a door's name.
        if BUILT_IN_WORDS.contains(&word.as_str()) {
            self.refuse_why(
                name_place,
                format!("`{word}` is one of Seal's own words, so a procedure can't be named `{word}`"),
                NO_SHADOWING,
            );
            return;
        }
        if self.doors.iter().any(|door| door.name.split('.').next() == Some(word.as_str())) {
            self.refuse_why(
                name_place,
                format!("`{word}` is one of the host's doors, so a procedure can't be named `{word}`"),
                NO_SHADOWING,
            );
            return;
        }
        if let Some(&earlier) = self.procedure_names.get(&name) {
            let earlier_place = self.program.procedure(earlier).map_or(Place::default(), |earlier| earlier.place);
            self.messages.push(
                Message::refused(name_place, format!("the procedure `{name}` is declared twice"))
                    .with_because(format!("line {} declares it first", earlier_place.line), earlier_place)
                    .with_why("each procedure is declared once."),
            );
            return;
        }
        let mut parameters: Vec<VariableId> = Vec::new();
        for parameter in &procedure.parameters {
            let duplicate = parameters
                .iter()
                .any(|&earlier| self.variable(earlier).is_some_and(|earlier| earlier.name == parameter.name.text));
            if duplicate {
                // trace: Unio D102
                self.refuse_why(
                    parameter.name.place,
                    format!("two parameters are named `{}`", parameter.name.text),
                    "each parameter has a name of its own.",
                );
            }
            // trace: D81. Every parameter writes its type; the parser refuses one that doesn't.
            let type_id = match &parameter.written_type {
                Some(written) => self.written_type(written),
                None => ERROR,
            };
            parameters.push(self.new_variable(Variable {
                name: parameter.name.text.clone(),
                type_id,
                kind: VariableKind::Parameter,
                mutable: false,
                marked_unused: parameter.unused,
                read: false,
                top_level: false,
                captured: false,
                made_at: 0,
                place: parameter.name.place,
            }));
        }
        // trace: D81. A procedure that gives back a value writes `-> Type`; one that doesn't gives nothing.
        let result = match &procedure.result {
            Some(written) => self.written_type(written),
            None => NOTHING,
        };
        let mut labels: Vec<Option<String>> = vec![None; parameters.len()];
        for declaration in &procedure.labels {
            let position =
                procedure.parameters.iter().position(|parameter| parameter.name.text == declaration.parameter.text);
            let Some(position) = position else {
                self.refuse(
                    declaration.parameter.place,
                    format!("`{}` isn't one of `{name}`'s parameters", declaration.parameter.text),
                );
                continue;
            };
            if labels.iter().flatten().any(|label| *label == declaration.label.text) {
                self.refuse_why(
                    declaration.label.place,
                    format!("two parameters have the label `{}`", declaration.label.text),
                    "each label names one argument.",
                );
                continue;
            }
            match labels.get_mut(position) {
                Some(slot) if slot.is_none() => *slot = Some(declaration.label.text.clone()),
                _ => self.refuse(
                    declaration.parameter.place,
                    format!("`{}` is given two labels", declaration.parameter.text),
                ),
            }
        }
        let mut defaults: Vec<Option<Expression>> = vec![None; parameters.len()];
        for declaration in &procedure.defaults {
            let position = labels.iter().position(|label| label.as_deref() == Some(declaration.label.text.as_str()));
            let Some(position) = position else {
                // trace: D42
                self.refuse_why(
                    declaration.label.place,
                    format!("`{}` isn't one of `{name}`'s labels", declaration.label.text),
                    "only a labelled parameter can have a default.",
                );
                continue;
            };
            let parameter_type = parameters.get(position).map_or(ERROR, |&parameter| self.variable_type(parameter));
            let Some(value) = self.plain_value(&declaration.value, parameter_type) else { continue };
            if !self.program.types.unify(value.type_id, parameter_type) {
                let found = self.describe(value.type_id);
                let wanted = self.describe(parameter_type);
                self.refuse(
                    declaration.value.place,
                    format!("this default is {found}, but `{}` is {wanted}", declaration.label.text),
                );
            }
            match defaults.get_mut(position) {
                Some(slot) if slot.is_none() => *slot = Some(value),
                _ => self.refuse(declaration.label.place, format!("`{}` has two defaults", declaration.label.text)),
            }
        }
        let can_fail = procedure.failures.is_some() || block_can_fail(&procedure.body);
        let labels_any_order = procedure.labels_any_order;
        let id = ProcedureId(u32::try_from(self.program.procedures.len()).unwrap_or(u32::MAX));
        self.program.procedures.push(Procedure {
            name: name.clone(),
            parameters,
            labels,
            labels_any_order,
            defaults,
            result,
            can_fail,
            failures: Vec::new(),
            can_pause: false,
            body: Vec::new(),
            place: procedure.place,
            end_place: procedure.end_place,
        });
        self.procedure_syntax.push(procedure);
        self.procedure_state.push(ProcedureState::Unchecked);
        self.written_failures.push(procedure.failures.clone());
        self.procedure_names.insert(name, id);
        self.words.insert(word);
    }

    /// A default: a value written out plainly, or a top-level name made from one with no `mut`, of the parameter's
    /// type, never a calculation or a call (D42, D104).
    fn plain_value(&mut self, expression: &syntax::Expression, wanted: TypeId) -> Option<Expression> {
        if !is_plain(expression, &self.plain_names) {
            // trace: D104
            self.refuse_why(
                expression.place,
                "a default is a value written out plainly, never a calculation or a call",
                "a default is the same every time: a number, text, a `Bool`, a list, a record, a case, or a top-level name made from one with no `mut`.",
            );
            return None;
        }
        let written = self.inline_plain_names(expression);
        Some(self.expression(&written, Some(wanted)))
    }

    /// A plain value with each top-level name in it replaced by the value it was made from, which is the same every
    /// time (D103, D104). Each name so used counts as read.
    fn inline_plain_names(&mut self, expression: &syntax::Expression) -> syntax::Expression {
        let mut copy = expression.clone();
        let mut waiting = vec![&mut copy];
        while let Some(current) = waiting.pop() {
            if let syntax::ExpressionKind::Name(name) = &current.kind
                && let Some(&value) = self.plain_names.get(name)
            {
                if let Some(&global) = self.globals.get(name) {
                    self.mark_read(global);
                }
                let place = current.place;
                *current = (*value).clone();
                current.place = place;
                waiting.push(current);
                continue;
            }
            match &mut current.kind {
                syntax::ExpressionKind::List(items) => waiting.extend(items.iter_mut()),
                syntax::ExpressionKind::Pairs(pairs) => {
                    for (key, value) in pairs {
                        waiting.push(key);
                        waiting.push(value);
                    }
                }
                syntax::ExpressionKind::Bracketed { entries, .. } => {
                    waiting.extend(entries.iter_mut().map(|entry| &mut entry.value));
                }
                syntax::ExpressionKind::Unary { operand, .. } | syntax::ExpressionKind::Group(operand) => {
                    waiting.push(operand);
                }
                _ => {}
            }
        }
        copy
    }

    /// trace: D103, D104. The top-level names made with no `mut` from a value written out plainly, which are known
    /// while loading, and the ones made with `mut`, which aren't.
    fn collect_plain_names(&mut self, script: &'a syntax::Script) {
        for item in &script.items {
            let syntax::Item::Statement(statement) = item else { continue };
            let syntax::StatementKind::Name { marker, name, value, .. } = &statement.kind else { continue };
            if self.plain_names.contains_key(&name.text) || self.changing_names.contains(&name.text) {
                continue;
            }
            if *marker == syntax::Marker::Mut {
                self.changing_names.insert(name.text.clone());
            } else if *marker == syntax::Marker::None && is_plain(value, &self.plain_names) {
                self.plain_names.insert(name.text.clone(), value);
            }
        }
    }

    /// Finds the top level's own names before anything is checked, so procedures can use them (D4, D56).
    fn declare_globals(&mut self, script: &syntax::Script) {
        let mut position = 0usize;
        for item in &script.items {
            let syntax::Item::Statement(statement) = item else { continue };
            if let syntax::StatementKind::Name { marker, name, .. } = &statement.kind
                && !self.globals.contains_key(&name.text)
            {
                let type_id = self.program.types.fresh();
                let id = self.new_variable(Variable {
                    name: name.text.clone(),
                    type_id,
                    kind: VariableKind::Named,
                    mutable: *marker == syntax::Marker::Mut,
                    marked_unused: *marker == syntax::Marker::Unused,
                    read: false,
                    top_level: true,
                    captured: false,
                    made_at: position,
                    place: name.place,
                });
                self.globals.insert(name.text.clone(), id);
            }
            position += 1;
        }
    }

    // ---- names ----

    /// Finds a name that exists here. Inside a procedure, the top level's names are visible too (D56).
    fn lookup(&mut self, name: &str, place: Place) -> Option<VariableId> {
        for scope in self.context.scopes.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Some(id);
            }
        }
        self.context.procedure?;
        let id = *self.globals.get(name)?;
        if let Some(variable) = self.program.variables.get_mut(id.0 as usize) {
            variable.captured = true;
        }
        if !self.made.contains(&id) && self.reported_early_use.insert(id) {
            // trace: D67. The top level reaches this procedure before the line that makes the name has run.
            let made_at = self.variable(id).map_or(place, |variable| variable.place);
            let mut message = Message::refused(place, format!("`{name}` is reached here before the top level has made it"))
                .with_because(format!("line {} makes it", made_at.line), made_at)
                .with_why("the top level runs from top to bottom when the script loads, and a procedure it calls runs right then.");
            if let Some(line) = self.top_level_line {
                message = message.with_when(format!("the top level's line {} runs this", line.line));
            }
            self.messages.push(message);
        }
        Some(id)
    }

    fn mark_read(&mut self, id: VariableId) {
        if let Some(variable) = self.program.variables.get_mut(id.0 as usize) {
            variable.read = true;
        }
    }

    fn declare(&mut self, name: &str, id: VariableId) {
        if let Some(scope) = self.context.scopes.last_mut() {
            scope.insert(name.to_owned(), id);
        }
    }

    /// Whether a new name may be spelt `name`: one spelling means one thing, so it can't be a word that's already a
    /// procedure, a door or one of Seal's own words (D71).
    fn check_new_name(&mut self, name: &syntax::Word) -> bool {
        let word = name.text.as_str();
        if word == "all" {
            // trace: D83
            self.refuse_why(
                name.place,
                "a name can't be `all`",
                "`[all: value]` always means a list or array whose every item starts as `value`, so `all` can't name anything else.",
            );
            return false;
        }
        let taken_by = if BUILT_IN_WORDS.contains(&word) {
            "one of Seal's own words"
        } else if self.doors.iter().any(|door| door.name.split('.').next() == Some(word)) {
            "one of the host's doors"
        } else if self.words.contains(word) {
            "a procedure"
        } else if self.handler_groups.contains_key(word) {
            "a handler group"
        } else {
            return true;
        };
        self.refuse_why(
            name.place,
            format!("`{word}` is already {taken_by}, so nothing else can be named `{word}`"),
            NO_SHADOWING,
        );
        false
    }

    /// Refuses a name made inside a procedure with a top-level name's spelling (D71).
    fn refuse_top_level_spelling(&mut self, name: &syntax::Word, what: &str) {
        let Some(&global) = self.globals.get(&name.text) else { return };
        let made = self.variable(global).map_or(Place::default(), |variable| variable.place);
        self.messages.push(
            Message::refused(
                name.place,
                format!("`{}` is already a top-level name, so {what} can't be named `{}`", name.text, name.text),
            )
            .with_because(format!("line {} makes it", made.line), made)
            .with_why(NO_SHADOWING),
        );
    }

    /// trace: D37, D61. A case takes out only the values it reads: some of them, by name, in the order they're
    /// declared.
    fn check_case_values(&mut self, case: &syntax::MatchCase, fields: &[(String, TypeId)]) {
        let written: Vec<&str> = case.values.iter().map(|word| word.text.as_str()).collect();
        let declared: Vec<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
        if let Some(word) = case.values.iter().find(|word| !declared.contains(&word.text.as_str())) {
            self.refuse_why(
                word.place,
                format!("the case `{}` has no value `{}`", case.name.text, word.text),
                "a case's values come out by name.",
            );
            return;
        }
        let mut left = declared.iter();
        let in_order = written.iter().all(|name| left.any(|declared| declared == name));
        // A value taken out twice is refused as a name made twice.
        let twice = written.iter().enumerate().any(|(at, name)| written.iter().skip(at + 1).any(|later| later == name));
        if !in_order && !twice {
            // trace: D87. A case takes out its values in their declared order.
            let ordered: Vec<&str> = declared.iter().filter(|name| written.contains(name)).copied().collect();
            self.refuse_why(
                case.name.place,
                format!(
                    "the case `{}`'s values are taken out in the order they're declared: `{}`",
                    case.name.text,
                    ordered.join("`, `")
                ),
                "a case's values come out in the order its type declares them, skipping any that aren't read.",
            );
        }
    }

    /// Marks a name `unused`: never read, on purpose (D2, D61).
    fn mark_unused(&mut self, id: VariableId, unused: bool) {
        if let Some(variable) = self.program.variables.get_mut(id.0 as usize) {
            variable.marked_unused = unused;
        }
    }

    /// Makes a name that's new in the current block, as a loop's item or a case's value.
    fn make_local(&mut self, name: &syntax::Word, type_id: TypeId, kind: VariableKind) -> VariableId {
        let local = self.context.scopes.iter().rev().find_map(|scope| scope.get(&name.text).copied());
        if let Some(existing) = local {
            self.refuse_existing(name, existing);
        } else if self.context.procedure.is_some() && self.globals.contains_key(&name.text) {
            // trace: D71
            let what = if kind == VariableKind::CaseValue { "a `case`'s value" } else { "a `for each` name" };
            self.refuse_top_level_spelling(name, what);
        }
        self.check_new_name(name);
        let id = self.new_variable(Variable {
            name: name.text.clone(),
            type_id,
            kind,
            mutable: false,
            marked_unused: false,
            read: false,
            top_level: false,
            captured: false,
            made_at: 0,
            place: name.place,
        });
        self.declare(&name.text, id);
        id
    }

    /// trace: D56
    fn refuse_existing(&mut self, name: &syntax::Word, existing: VariableId) {
        let earlier = self.variable(existing).map_or(Place::default(), |variable| variable.place);
        self.messages.push(
            Message::refused(name.place, format!("`{}` already exists, so it can't be made again", name.text))
                .with_because(format!("line {} makes it", earlier.line), earlier)
                .with_why("while a name exists, it can't be made again."),
        );
    }

    /// trace: D2, D61
    fn check_unused(&mut self, scope: &HashMap<String, VariableId>) {
        let mut unused: Vec<(Place, String, VariableKind)> = scope
            .values()
            .filter_map(|&id| self.variable(id))
            .filter(|variable| !variable.read && !variable.marked_unused)
            .map(|variable| (variable.place, variable.name.clone(), variable.kind))
            .collect();
        unused.sort_by_key(|(place, _, _)| (place.line, place.column));
        for (place, name, kind) in unused {
            match kind {
                VariableKind::Named => self.refuse(place, format!("`{name}` is never used")),
                VariableKind::Parameter => self.refuse_why(
                    place,
                    format!("`{name}` is never used"),
                    &format!("a parameter that's never read on purpose is marked, as `unused {name}`."),
                ),
                VariableKind::LoopItem => self.refuse_why(
                    place,
                    format!("`{name}` is never used"),
                    &format!("a `for each` name that's never read on purpose is marked, as `for each unused {name}`."),
                ),
                VariableKind::LoopPosition => self.refuse_why(
                    place,
                    format!("`{name}` is never used"),
                    &format!("an `at` name that's never read is marked, as `at key unused {name}`, or the `at` part is left out."),
                ),
                VariableKind::CaseValue => self.refuse_why(
                    place,
                    format!("`{name}` is never used"),
                    "a `case` takes out only the values it reads.",
                ),
            }
        }
    }

    // ---- procedures ----

    /// Checks a procedure's body the first time it's needed: when it's called, or after the top level.
    fn ensure_checked(&mut self, id: ProcedureId) {
        let index = id.0 as usize;
        if self.procedure_state.get(index) != Some(&ProcedureState::Unchecked) {
            return;
        }
        let Some(&procedure) = self.procedure_syntax.get(index) else { return };
        if let Some(state) = self.procedure_state.get_mut(index) {
            *state = ProcedureState::InProgress;
        }
        let saved = std::mem::replace(&mut self.context, Context { procedure: Some(id), ..Context::default() });
        self.demand_depth += 1;
        let mut scope = HashMap::new();
        let parameters = self.program.procedure(id).map(|procedure| procedure.parameters.clone()).unwrap_or_default();
        for &parameter in &parameters {
            // trace: D71. A parameter can't take a spelling that already means something.
            if let Some(variable) = self.variable(parameter) {
                let word = syntax::Word { text: variable.name.clone(), place: variable.place };
                self.check_new_name(&word);
                self.refuse_top_level_spelling(&word, "a parameter");
                scope.insert(word.text, parameter);
            }
        }
        self.context.scopes.push(scope);
        let body = self.block(&procedure.body);
        let scope = self.context.scopes.pop().unwrap_or_default();
        self.check_unused(&scope);
        let result = self.program.procedure(id).map_or(ERROR, |procedure| procedure.result);
        if self.context.returned_value == Some(true) {
            if !Self::block_leaves(&body) {
                // trace: D68, Unio D57, D149
                self.refuse_why(
                    procedure.end_place,
                    format!("`{}` gives a value, but it can reach its `end` without a `return`", procedure.name()),
                    "when any `return` in a procedure gives a value, every way through ends in one; a `while` counts as able to end, whatever its condition.",
                );
            }
        } else if let Some(last) = body.last().filter(|last| matches!(last.kind, StatementKind::Return(None))) {
            // trace: D68, Unio D57
            self.refuse_why(
                last.place,
                "this `return` is the procedure's last line, where the procedure ends anyway",
                "a bare `return` leaves a procedure early.",
            );
        } else if !self.program.types.unify(result, NOTHING) {
            // trace: D6
            let wanted = self.describe(result);
            self.refuse_why(
                procedure.result.as_ref().map_or(procedure.place, |written| written.place),
                format!("`{}` says it gives {wanted}, but it has no `return` with a value", procedure.name()),
                "a procedure's result comes from its `return`s.",
            );
        }
        if let Some(slot) = self.program.procedures.get_mut(index) {
            slot.body = body;
        }
        if let Some(state) = self.procedure_state.get_mut(index) {
            *state = ProcedureState::Checked;
        }
        self.context = saved;
        self.demand_depth -= 1;
        self.batch.push(id);
        self.solve(false);
        if self.demand_depth == 0 {
            self.solve(true);
        }
    }

    // ---- blocks and statements ----

    fn block(&mut self, statements: &[syntax::Statement]) -> Block {
        self.context.scopes.push(HashMap::new());
        let mut block = Vec::new();
        // The line before, when it always leaves this block, or is an `if` or a `match` that can.
        let mut left: Option<(Place, Leaving)> = None;
        for statement in statements {
            if let Some((place, leaving)) = left.take() {
                self.refuse_after_leaving(statement.place, place, leaving);
            }
            if let Some(checked) = self.statement(statement) {
                left = Self::leaving(&checked).map(|leaving| (checked.place, leaving));
                block.push(checked);
            }
        }
        let scope = self.context.scopes.pop().unwrap_or_default();
        self.check_unused(&scope);
        block
    }

    /// trace: D68, Unio D9, D57. Nothing follows a line that always leaves, or an `if` where any way leaves.
    fn refuse_after_leaving(&mut self, place: Place, before: Place, leaving: Leaving) {
        match leaving {
            Leaving::Always => self.messages.push(
                Message::refused(place, "this line can never run")
                    .with_because(format!("line {} always leaves this block", before.line), before)
                    .with_why("nothing follows `return`, `break` or `fail` in the same block."),
            ),
            Leaving::SomeWayOfIf => self.messages.push(
                Message::refused(place, "this line follows an `if` that can leave")
                    .with_because(format!("line {} opens that `if`", before.line), before)
                    .with_why(
                        "when any way through an `if` leaves, by `return`, `break` or `fail`, nothing follows its `end`; what runs afterwards goes in the ways that don't leave.",
                    ),
            ),
            // trace: D86. D68's rule covers `match`.
            Leaving::SomeWayOfMatch => self.messages.push(
                Message::refused(place, "this line follows a `match` that can leave")
                    .with_because(format!("line {} opens that `match`", before.line), before)
                    .with_why(
                        "when any case of a `match` leaves, by `return`, `break` or `fail`, nothing follows its `end`; what runs afterwards goes in the cases that don't leave.",
                    ),
            ),
            // trace: D80, Unio D57
            Leaving::Instead => self.messages.push(
                Message::refused(place, "this line can never run")
                    .with_because(format!("line {} gives the case its value with `instead`", before.line), before)
                    .with_why("`instead` ends its case: the call has its value and the script carries on after the `otherwise`, so anything else the case does goes before it."),
            ),
        }
    }

    /// How a statement can leave the block it's in, when it can: always, or by some way through an `if` or a
    /// `match`.
    fn leaving(statement: &Statement) -> Option<Leaving> {
        if matches!(statement.kind, StatementKind::Instead(_)) {
            return Some(Leaving::Instead);
        }
        if Self::leaves(statement) {
            return Some(Leaving::Always);
        }
        match &statement.kind {
            StatementKind::If { branches, otherwise } => (branches.iter().any(|(_, block)| Self::can_leave(block, 0))
                || otherwise.as_ref().is_some_and(|block| Self::can_leave(block, 0)))
            .then_some(Leaving::SomeWayOfIf),
            StatementKind::Match { arms, .. } => {
                arms.iter().any(|arm| Self::can_leave(&arm.body, 0)).then_some(Leaving::SomeWayOfMatch)
            }
            _ => None,
        }
    }

    /// Whether some way through a block leaves it by `return` or `fail`, or by a `break` out of a loop around it.
    /// `loops` counts the loops entered inside the block: a `break` in one of those only leaves that loop.
    fn can_leave(block: &[Statement], loops: u32) -> bool {
        block.iter().any(|statement| match &statement.kind {
            StatementKind::Return(_) | StatementKind::Fail(_) => true,
            StatementKind::Break => loops == 0,
            StatementKind::If { branches, otherwise } => {
                branches.iter().any(|(_, block)| Self::can_leave(block, loops))
                    || otherwise.as_ref().is_some_and(|block| Self::can_leave(block, loops))
            }
            StatementKind::Match { arms, .. } => arms.iter().any(|arm| Self::can_leave(&arm.body, loops)),
            StatementKind::While { body, .. }
            | StatementKind::ForRange { body, .. }
            | StatementKind::ForEach { body, .. } => Self::can_leave(body, loops + 1),
            _ => false,
        })
    }

    /// Whether a statement always leaves its block, by `return`, `break` or `fail`.
    fn leaves(statement: &Statement) -> bool {
        match &statement.kind {
            StatementKind::Return(_) | StatementKind::Break | StatementKind::Fail(_) => true,
            StatementKind::If { branches, otherwise: Some(otherwise) } => {
                branches.iter().all(|(_, block)| Self::block_leaves(block)) && Self::block_leaves(otherwise)
            }
            StatementKind::Match { arms, .. } => {
                !arms.is_empty() && arms.iter().all(|arm| Self::block_leaves(&arm.body))
            }
            _ => false,
        }
    }

    /// Whether a block always leaves: one of its lines does. A line after that one has its own message.
    fn block_leaves(block: &Block) -> bool {
        block.iter().any(Self::leaves)
    }

    #[allow(clippy::too_many_lines)] // one case per kind of statement
    fn statement(&mut self, statement: &syntax::Statement) -> Option<Statement> {
        let place = statement.place;
        let kind = match &statement.kind {
            syntax::StatementKind::Name { marker, name, written_type, value } => {
                self.name_statement(*marker, name, written_type.as_ref(), value)?
            }
            syntax::StatementKind::Change { place: target, value } => self.change(target, value)?,
            syntax::StatementKind::Expression(expression) => {
                let checked = self.expression(expression, None);
                if !self.program.types.is_error(checked.type_id) {
                    let is_call = matches!(
                        Self::without_handling(&checked).kind,
                        // A call that was refused has had its message already.
                        ExpressionKind::Error
                            | ExpressionKind::Inside { .. }
                            | ExpressionKind::Call { .. }
                            | ExpressionKind::DoorCall { .. }
                            | ExpressionKind::ListAdd { .. }
                            | ExpressionKind::ListRemove { .. }
                            | ExpressionKind::DictRemove { .. }
                    );
                    // trace: D70, Unio D120, D161
                    if !is_call {
                        let found = self.describe(checked.type_id);
                        self.refuse_why(
                            expression.place,
                            format!("this line works out {found}, but nothing uses it"),
                            "a line that only works out a value does nothing; keep the value under a name, or give it to a `mut` name with `=`.",
                        );
                    } else if !self.program.types.unify(checked.type_id, NOTHING) {
                        let found = self.describe(checked.type_id);
                        self.refuse_why(
                            expression.place,
                            format!("this call gives {found}, but nothing uses it"),
                            "a call that gives a value is used: kept under a name, passed on, or compared; `unused name = …` drops it on purpose.",
                        );
                    }
                }
                StatementKind::Evaluate(checked)
            }
            syntax::StatementKind::If { branches, otherwise } => {
                let mut checked_branches = Vec::new();
                for (condition, body) in branches {
                    let condition = self.condition(condition, "if");
                    checked_branches.push((condition, self.block(body)));
                }
                let otherwise = otherwise.as_ref().map(|body| self.block(body));
                StatementKind::If { branches: checked_branches, otherwise }
            }
            syntax::StatementKind::While { condition, body } => {
                self.refuse_float_equality(condition);
                let condition = self.condition(condition, "while");
                self.context.loop_depth += 1;
                let body = self.block(body);
                self.context.loop_depth -= 1;
                StatementKind::While { condition, body }
            }
            syntax::StatementKind::ForEach { item, item_unused, position, position_unused, source, body } => {
                self.for_each(item, (*item_unused, *position_unused), position.as_ref(), source, body)?
            }
            syntax::StatementKind::Match { subject, cases } => self.match_statement(subject, cases),
            syntax::StatementKind::Break => {
                if self.context.loop_depth == 0 {
                    // trace: Unio D32
                    self.refuse_why(
                        place,
                        "this `break` isn't inside a loop",
                        "`break` leaves the innermost loop it's in.",
                    );
                    return None;
                }
                StatementKind::Break
            }
            syntax::StatementKind::Return(value) => self.return_statement(value.as_ref(), place)?,
            syntax::StatementKind::Fail(name) => {
                if self.context.procedure.is_none() {
                    // trace: D67
                    self.refuse_why(place, "`fail` hands a failure to a caller, and the top level has none", TOP_LEVEL);
                    return None;
                }
                StatementKind::Fail(self.failure(&name.text))
            }
            syntax::StatementKind::Instead(value) => {
                let Some(&wanted) = self.context.instead_types.last() else {
                    // trace: Unio D40
                    self.refuse_why(
                        place,
                        "this `instead` isn't inside an `otherwise` case",
                        "`instead` gives the value an `otherwise` case carries on with.",
                    );
                    return None;
                };
                let checked = self.expression(value, Some(wanted));
                self.expect(&checked, wanted, "the call it stands in for");
                StatementKind::Instead(checked)
            }
        };
        Some(Statement { kind, place })
    }

    /// The call inside any `?`, `??` or `otherwise` around it.
    fn without_handling(expression: &Expression) -> &Expression {
        match &expression.kind {
            ExpressionKind::Try(inner) => Self::without_handling(inner),
            ExpressionKind::Handle { call, .. } | ExpressionKind::Otherwise { call, .. } => {
                Self::without_handling(call)
            }
            _ => expression,
        }
    }

    /// Unifies an expression's type with what's wanted, refusing a mismatch.
    fn expect(&mut self, expression: &Expression, wanted: TypeId, role: &str) -> bool {
        if self.program.types.unify(expression.type_id, wanted) {
            return true;
        }
        let found = self.describe(expression.type_id);
        let wanted_text = self.describe(wanted);
        // trace: Unio D89: "but" sets what's expected against what was found.
        self.refuse(expression.place, format!("this is {found}, but {role} is {wanted_text}"));
        false
    }

    /// trace: D16, Unio D187
    fn condition(&mut self, condition: &syntax::Expression, word: &str) -> Expression {
        let checked = self.expression(condition, Some(BOOL));
        if !self.program.types.unify(checked.type_id, BOOL) {
            let found = self.describe(checked.type_id);
            let hint = match self.program.types.get(checked.type_id) {
                Type::Whole => ", as in `!= 0`",
                Type::Float => ", as in `!= 0.0`",
                _ => "",
            };
            self.refuse_why(
                condition.place,
                format!("`{word}` checks a `Bool`, but this is {found}"),
                &format!("a number or text doesn't stand for true or false; compare it{hint}."),
            );
        }
        checked
    }

    /// trace: Unio D184: `==` and `!=` between two `Float`s are refused in a `while` condition.
    fn refuse_float_equality(&mut self, condition: &syntax::Expression) {
        match &condition.kind {
            syntax::ExpressionKind::Binary { operator, left, right, operator_place } => {
                if matches!(operator, syntax::BinaryOperator::Equal | syntax::BinaryOperator::NotEqual) {
                    let saved = self.messages.len();
                    let left_type = self.peek_type(left);
                    let right_type = self.peek_type(right);
                    self.messages.truncate(saved);
                    if left_type == Some(FLOAT) && right_type == Some(FLOAT) {
                        self.refuse_why(
                            *operator_place,
                            format!("`{}` between two `Float`s in a `while` condition", operator.text()),
                            "a `Float` that changes each pass rarely lands exactly on a value, so the loop could never \
                             stop; `<`, `>`, `at_least` or `at_most` say \"until it reaches\".",
                        );
                    }
                }
                self.refuse_float_equality(left);
                self.refuse_float_equality(right);
            }
            syntax::ExpressionKind::Unary { operand, .. } => self.refuse_float_equality(operand),
            syntax::ExpressionKind::Group(inner) => self.refuse_float_equality(inner),
            _ => {}
        }
    }

    /// The type of a simple expression, when it's plain to see: a literal or an existing name.
    fn peek_type(&mut self, expression: &syntax::Expression) -> Option<TypeId> {
        match &expression.kind {
            syntax::ExpressionKind::Float { .. } => Some(FLOAT),
            syntax::ExpressionKind::Whole(_) => Some(WHOLE),
            syntax::ExpressionKind::Group(inner) => self.peek_type(inner),
            syntax::ExpressionKind::Unary { operator: syntax::UnaryOperator::Negate, operand } => {
                self.peek_type(operand)
            }
            syntax::ExpressionKind::Name(name) => {
                let id = self.context.scopes.iter().rev().find_map(|scope| scope.get(name).copied()).or_else(|| {
                    if self.context.procedure.is_some() { self.globals.get(name).copied() } else { None }
                })?;
                Some(self.program.types.resolve(self.variable_type(id)))
            }
            syntax::ExpressionKind::Binary { operator, left, .. } if operator.is_maths() => self.peek_type(left),
            _ => None,
        }
    }

    /// `name = value`, with `mut`, `unused` or a type when written (D2, D3, D30, D56).
    fn name_statement(
        &mut self,
        marker: syntax::Marker,
        name: &syntax::Word,
        written_type: Option<&syntax::TypeExpression>,
        value: &syntax::Expression,
    ) -> Option<StatementKind> {
        let existing = self.lookup(&name.text, name.place);
        if let Some(existing) = existing {
            if marker == syntax::Marker::None && written_type.is_none() {
                return self.set(existing, name, value);
            }
            self.refuse_existing(name, existing);
            return None;
        }
        let name_is_free = self.check_new_name(name);
        self.partial_types = true;
        let wanted = written_type.map(|written| self.written_type(written));
        self.partial_types = false;
        let checked = self.expression(value, wanted);
        let checked = match wanted {
            Some(wanted) => self.land(checked, wanted),
            None => checked,
        };
        let type_id = match wanted {
            Some(wanted) => {
                self.expect(&checked, wanted, &format!("`{}`", name.text));
                if let Some(written) = written_type
                    && !self.program.types.is_settled(wanted)
                {
                    // trace: D82. What the value doesn't show must be written.
                    let kind = written.name.text.as_str();
                    let example = if kind == "Dict" {
                        "Dict<key: Text, value: Whole>".to_owned()
                    } else {
                        format!("{kind}<Text>")
                    };
                    self.refuse_why(
                        written.place,
                        format!("`{kind}` leaves out what it holds, and the value doesn't show it"),
                        &format!("a written type leaves out only what its value shows, so this one writes it, as `{example}`."),
                    );
                    // The name is still made, so that its uses aren't refused as well.
                    self.program.types.settle_as_error(wanted);
                }
                wanted
            }
            None => checked.type_id,
        };
        if wanted.is_none()
            && matches!(checked.kind, ExpressionKind::BuildCase { case: 0, .. })
            && !self.program.types.is_settled(checked.type_id)
        {
            // trace: D30, D38
            self.refuse_why(
                value.place,
                "`Maybe.none` can't say what the `Maybe` holds, so the name's type has to be written",
                "a type is written where the value can't say it, as in `name: Maybe<Whole> = Maybe.none`.",
            );
            self.program.types.settle_as_error(checked.type_id);
            return None;
        }
        if self.program.types.resolve(checked.type_id) == NOTHING {
            // trace: Unio D38
            self.refuse_why(
                value.place,
                "this gives nothing, so there's no value for the name",
                "a procedure that gives nothing back can't be used as a value.",
            );
            return None;
        }
        if !name_is_free {
            return None;
        }
        let at_top_level = self.context.procedure.is_none() && self.context.scopes.len() == 1;
        let id = match self.globals.get(&name.text).copied() {
            Some(global) if at_top_level => {
                let variable_type = self.variable_type(global);
                self.program.types.unify(variable_type, type_id);
                let ranged = self.program.types.limits(type_id).is_some();
                if let Some(variable) = self.program.variables.get_mut(global.0 as usize) {
                    // trace: D128. A ranged name keeps its limits, which its plain type, worked out, doesn't carry.
                    if ranged {
                        variable.type_id = type_id;
                    }
                    variable.mutable = marker == syntax::Marker::Mut;
                    variable.marked_unused = marker == syntax::Marker::Unused;
                    variable.place = name.place;
                }
                self.made.insert(global);
                global
            }
            _ => self.new_variable(Variable {
                name: name.text.clone(),
                type_id,
                kind: VariableKind::Named,
                mutable: marker == syntax::Marker::Mut,
                marked_unused: marker == syntax::Marker::Unused,
                read: false,
                top_level: false,
                captured: false,
                made_at: 0,
                place: name.place,
            }),
        };
        self.declare(&name.text, id);
        Some(StatementKind::Make { variable: id, value: checked })
    }

    /// Whether `variable` may be changed here, refusing when it may not (D2, Unio D147, D173).
    fn check_changeable(&mut self, variable: VariableId, name: &str, place: Place) -> bool {
        let Some(found) = self.variable(variable) else { return false };
        let (kind, mutable, made_place) = (found.kind, found.mutable, found.place);
        if self.context.iterating.contains(&variable) {
            // trace: Unio D173
            self.refuse_why(
                place,
                format!("`{name}` is being gone through by a `for each`, so it can't change here"),
                "what a `for each` goes through can't change inside it.",
            );
            return false;
        }
        if mutable {
            return true;
        }
        let (because, why) = match kind {
            VariableKind::Parameter => {
                (format!("line {} makes it a parameter", made_place.line), "a parameter is read-only.")
            }
            VariableKind::Named => {
                (format!("line {} makes it without `mut`", made_place.line), "only a name made with `mut` can change.")
            }
            VariableKind::LoopItem | VariableKind::LoopPosition => (
                format!("line {} makes it for each pass of the loop", made_place.line),
                "a `for each`'s names are read-only.",
            ),
            VariableKind::CaseValue => {
                (format!("line {} takes it out of a case", made_place.line), "a case's values are read-only.")
            }
        };
        self.messages.push(
            Message::refused(place, format!("`{name}` can't be given a new value"))
                .with_because(because, made_place)
                .with_why(why),
        );
        false
    }

    /// `name = value` where `name` already exists: a change (D2).
    fn set(&mut self, variable: VariableId, name: &syntax::Word, value: &syntax::Expression) -> Option<StatementKind> {
        let changeable = self.check_changeable(variable, &name.text, name.place);
        let wanted = self.variable_type(variable);
        let checked = self.expression(value, Some(wanted));
        self.expect(&checked, wanted, &format!("`{}`", name.text));
        let checked = self.land(checked, wanted);
        changeable.then_some(StatementKind::Set { variable, value: checked })
    }

    /// `name.field = value`, `name[pos: n] = value` and `name[key: k] = value` (D27, D32, Unio D80).
    #[allow(clippy::too_many_lines)] // a field, an item and a key, each checked and landed
    fn change(&mut self, target: &syntax::Expression, value: &syntax::Expression) -> Option<StatementKind> {
        let (receiver, step) = match &target.kind {
            syntax::ExpressionKind::Member { receiver, name } => (receiver, Step::Field(name)),
            syntax::ExpressionKind::Bracketed { receiver, entries } => (receiver, Step::Reach(entries)),
            _ => return None,
        };
        // trace: D62. The change is made to a name's own value, or to a part of it reached through fields and places.
        let (variable, described, changeable, way) = if let syntax::ExpressionKind::Name(name) = &receiver.kind {
            let Some(variable) = self.lookup(name, receiver.place) else {
                self.refuse(receiver.place, format!("there's no name `{name}` here"));
                return None;
            };
            (variable, format!("`{name}`"), self.check_changeable(variable, name, receiver.place), None)
        } else {
            let (way, described, changeable) = self.way_in(receiver)?;
            (way.part, described, changeable, Some(way))
        };
        let name = described.as_str();
        let open = self.variable_type(variable);
        self.work_out_from_step(open, &step);
        let variable_type = self.program.types.resolve(self.variable_type(variable));
        let kind = self.program.types.get(variable_type).clone();
        let statement = match step {
            Step::Field(field) => {
                let Type::Record(record) = kind else {
                    if !matches!(kind, Type::Error) {
                        let found = self.describe(variable_type);
                        self.refuse(field.place, format!("{name} is {found}, which has no fields to change"));
                    }
                    return None;
                };
                let field_type = self
                    .program
                    .types
                    .records
                    .get(record as usize)
                    .and_then(|record| record.fields.iter().find(|(known, _)| *known == field.text))
                    .map(|(_, field_type)| *field_type);
                let Some(field_type) = field_type else {
                    let found = self.describe(variable_type);
                    self.refuse(field.place, format!("{found} has no field `{}`", field.text));
                    return None;
                };
                let checked = self.expression(value, Some(field_type));
                self.expect(&checked, field_type, &format!("the field `{}`", field.text));
                let checked = self.land(checked, field_type);
                StatementKind::SetField { variable, field: field.text.clone(), value: checked }
            }
            Step::Reach(entries) => {
                let [entry] = entries else {
                    self.refuse(target.place, "an item is reached with one of `pos:`, `offset:` or `key:`");
                    return None;
                };
                let label = entry.label.as_ref().map(|label| label.text.as_str());
                match (kind, label) {
                    (Type::List { item, .. } | Type::Array { item, .. }, Some(label @ ("pos" | "offset"))) => {
                        let index = self.expression(&entry.value, Some(WHOLE));
                        self.expect(&index, WHOLE, &format!("`{label}:`"));
                        let checked = self.expression(value, Some(item));
                        self.expect(&checked, item, "an item of the list");
                        let checked = self.land(checked, item);
                        StatementKind::SetItem { variable, index, zero_based: label == "offset", value: checked }
                    }
                    (Type::Dict { key, value: value_type }, Some("key")) => {
                        let checked_key = self.expression(&entry.value, Some(key));
                        self.expect(&checked_key, key, "the dictionary's key");
                        let checked = self.expression(value, Some(value_type));
                        self.expect(&checked, value_type, "the dictionary's value");
                        let checked = self.land(checked, value_type);
                        StatementKind::SetKey { variable, key: checked_key, value: checked }
                    }
                    (Type::Error, _) => return None,
                    (Type::List { .. } | Type::Array { .. }, _) => {
                        // trace: Unio D75
                        self.refuse_why(
                            target.place,
                            "a list's item is reached with `pos:` or `offset:`",
                            "`pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.",
                        );
                        return None;
                    }
                    (Type::Dict { .. }, _) => {
                        self.refuse(target.place, "a dictionary's value is reached with `key:`");
                        return None;
                    }
                    (Type::Variable(_), _) => {
                        self.not_built(
                            target.place,
                            "working out a type from a later line (D5), which this change needs, isn't",
                        );
                        return None;
                    }
                    _ => {
                        let found = self.describe(variable_type);
                        self.refuse(target.place, format!("{name} is {found}, which has no items to change"));
                        return None;
                    }
                }
            }
        };
        if !changeable {
            return None;
        }
        let Some(way) = way else { return Some(statement) };
        Some(self.change_inside(way, statement, target.place))
    }

    /// Wraps a change in the way to the part it's made in, with what it needs worked out before the way in is
    /// taken (D62).
    fn change_inside(&mut self, mut way: Inside, statement: StatementKind, place: Place) -> StatementKind {
        let change = match statement {
            StatementKind::SetField { variable, field, value } => {
                StatementKind::SetField { variable, field, value: self.hoist(value, &mut way.lets) }
            }
            StatementKind::SetItem { variable, index, zero_based, value } => {
                let index = self.hoist(index, &mut way.lets);
                StatementKind::SetItem { variable, index, zero_based, value: self.hoist(value, &mut way.lets) }
            }
            StatementKind::SetKey { variable, key, value } => {
                let key = self.hoist(key, &mut way.lets);
                StatementKind::SetKey { variable, key, value: self.hoist(value, &mut way.lets) }
            }
            other => other,
        };
        StatementKind::Inside { path: way, change: Box::new(Statement { kind: change, place }) }
    }

    /// trace: D5. With a value's type not worked out yet, a step into it can say what it must be: a field only one
    /// record has makes it that record, and a `key:` makes it a `Dict`. A place, `pos:` or `offset:`, doesn't say
    /// whether it's a list or an array, so that's left alone.
    fn work_out_from_step(&mut self, type_id: TypeId, step: &Step) {
        if !matches!(self.program.types.get(type_id), Type::Variable(_)) {
            return;
        }
        match step {
            Step::Field(field) => {
                let mut with_field = (0u32..)
                    .zip(&self.program.types.records)
                    .filter(|(_, record)| record.fields.iter().any(|(known, _)| *known == field.text))
                    .map(|(index, _)| index);
                if let (Some(only), None) = (with_field.next(), with_field.next()) {
                    let record = self.program.types.intern(Type::Record(only));
                    self.program.types.unify(type_id, record);
                }
            }
            Step::Reach(entries) => {
                if let [entry] = entries
                    && entry.label.as_ref().is_some_and(|label| label.text == "key")
                {
                    let (key, value) = (self.program.types.fresh(), self.program.types.fresh());
                    let dict = self.program.types.dict(key, value);
                    self.program.types.unify(type_id, dict);
                }
            }
        }
    }

    /// trace: D5. With a `match`'s value not worked out yet, its cases say what it must be: `some` and `none` make
    /// it a `Maybe`, and cases only one `one_of` has make it that `one_of`.
    fn work_out_from_cases(&mut self, type_id: TypeId, cases: &[syntax::MatchCase]) {
        if !matches!(self.program.types.get(type_id), Type::Variable(_)) || cases.is_empty() {
            return;
        }
        if cases.iter().all(|case| case.name.text == "some" || case.name.text == "none") {
            let inner = self.program.types.fresh();
            let maybe = self.program.types.maybe(inner);
            self.program.types.unify(type_id, maybe);
            return;
        }
        let mut with_cases = (0u32..)
            .zip(&self.program.types.one_ofs)
            .filter(|(_, one_of)| {
                cases.iter().all(|case| one_of.cases.iter().any(|known| known.name == case.name.text))
            })
            .map(|(index, _)| index);
        if let (Some(only), None) = (with_cases.next(), with_cases.next()) {
            let one_of = self.program.types.intern(Type::OneOf(only));
            self.program.types.unify(type_id, one_of);
        }
    }

    /// A hidden name, for a value worked out ahead of a change or for the part being changed (D62).
    fn hidden(&mut self, type_id: TypeId, place: Place, mutable: bool) -> VariableId {
        self.new_variable(Variable {
            name: String::new(),
            type_id,
            kind: VariableKind::Named,
            mutable,
            marked_unused: false,
            read: true,
            top_level: false,
            captured: false,
            made_at: 0,
            place,
        })
    }

    /// Puts a value under a hidden name, to be worked out ahead of a change to a part (D62). A plain value or a
    /// name stays as it is: reading it runs nothing.
    pub(super) fn hoist(&mut self, value: Expression, lets: &mut Vec<(VariableId, Expression)>) -> Expression {
        if matches!(
            value.kind,
            ExpressionKind::Error
                | ExpressionKind::Whole(_)
                | ExpressionKind::Float(_)
                | ExpressionKind::Bool(_)
                | ExpressionKind::Text(_)
                | ExpressionKind::Variable(_)
        ) {
            return value;
        }
        let (type_id, place) = (value.type_id, value.place);
        let hidden = self.hidden(type_id, place, false);
        lets.push((hidden, value));
        Expression { kind: ExpressionKind::Variable(hidden), type_id, place }
    }

    /// The name a change starts from and the steps written after it, outermost first (D62).
    fn written_way<'w>(&mut self, receiver: &'w syntax::Expression) -> Option<WrittenWay<'w>> {
        let mut written = Vec::new();
        let mut current = receiver;
        loop {
            match &current.kind {
                syntax::ExpressionKind::Name(name) => {
                    written.reverse();
                    return Some((name, current.place, written));
                }
                syntax::ExpressionKind::Member { receiver, name } => {
                    written.push((Step::Field(name), current.place));
                    current = receiver;
                }
                syntax::ExpressionKind::Bracketed { receiver, entries } => {
                    written.push((Step::Reach(entries), current.place));
                    current = receiver;
                }
                _ => {
                    self.refuse_why(
                        current.place,
                        "a change reaches into a `mut` name, and this isn't a name",
                        "a change is a name changing its own value.",
                    );
                    return None;
                }
            }
        }
    }

    /// The way from a `mut` name to a part of it, through fields and places (D62): the way, how to speak of the
    /// part in a message, and whether the name can change here.
    pub(super) fn way_in(&mut self, receiver: &syntax::Expression) -> Option<(Inside, String, bool)> {
        let (name, name_place, written) = self.written_way(receiver)?;
        let Some(root) = self.lookup(name, name_place) else {
            self.refuse(name_place, format!("there's no name `{name}` here"));
            return None;
        };
        let changeable = self.check_changeable(root, name, name_place);
        let mut way = Inside { lets: Vec::new(), root, steps: Vec::new(), types: Vec::new(), part: root };
        let mut reached = self.program.types.resolve(self.variable_type(root));
        for (step, place) in written {
            self.work_out_from_step(reached, &step);
            reached = self.program.types.resolve(reached);
            let kind = self.program.types.get(reached).clone();
            let found = self.describe(reached);
            way.types.push(reached);
            match step {
                Step::Field(field) => {
                    let Type::Record(record) = kind else {
                        if !matches!(kind, Type::Error) {
                            self.refuse(field.place, format!("this is {found}, which has no fields"));
                        }
                        return None;
                    };
                    let field_type = self
                        .program
                        .types
                        .records
                        .get(record as usize)
                        .and_then(|record| record.fields.iter().find(|(known, _)| *known == field.text))
                        .map(|(_, field_type)| *field_type);
                    let Some(field_type) = field_type else {
                        self.refuse(field.place, format!("{found} has no field `{}`", field.text));
                        return None;
                    };
                    way.steps.push(PathStep::Field(field.text.clone()));
                    reached = self.program.types.resolve(field_type);
                }
                Step::Reach(entries) => {
                    let [entry] = entries else {
                        self.refuse(place, "an item is reached with one of `pos:`, `offset:` or `key:`");
                        return None;
                    };
                    let label = entry.label.as_ref().map(|label| label.text.as_str());
                    match (kind, label) {
                        (Type::List { item, .. } | Type::Array { item, .. }, Some(label @ ("pos" | "offset"))) => {
                            let index = self.expression(&entry.value, Some(WHOLE));
                            self.expect(&index, WHOLE, &format!("`{label}:`"));
                            let index = self.hoist(index, &mut way.lets);
                            way.steps.push(PathStep::Item { index, zero_based: label == "offset" });
                            reached = self.program.types.resolve(item);
                        }
                        (Type::Dict { .. }, _) => {
                            // trace: D90. A missing key would have nowhere to be dealt with.
                            self.refuse_why(
                                place,
                                "a change can't go through a `Dict`'s key on its way, since the key might be missing",
                                "a lookup is dealt with on its own line: the value is taken out, as `mut north = shops[key: \"north\"] ?? crash`, changed, and put back, as `shops[key: \"north\"] = north`.",
                            );
                            return None;
                        }
                        (Type::Error, _) => return None,
                        (Type::List { .. } | Type::Array { .. }, _) => {
                            // trace: Unio D75
                            self.refuse_why(
                                place,
                                "a list's item is reached with `pos:` or `offset:`",
                                "`pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.",
                            );
                            return None;
                        }
                        (Type::Variable(_), _) => {
                            self.not_built(
                                place,
                                "working out a type from a later line (D5), which this change needs, isn't",
                            );
                            return None;
                        }
                        _ => {
                            self.refuse(place, format!("this is {found}, which has no items"));
                            return None;
                        }
                    }
                }
            }
        }
        way.part = self.hidden(reached, receiver.place, true);
        Some((way, format!("this part of `{name}`"), changeable))
    }

    /// trace: D6, Unio D57
    fn return_statement(&mut self, value: Option<&syntax::Expression>, place: Place) -> Option<StatementKind> {
        let Some(procedure) = self.context.procedure else {
            // trace: D67
            self.refuse_why(place, "`return` hands a value to a caller, and the top level has none", TOP_LEVEL);
            return None;
        };
        let result = self.program.procedure(procedure).map_or(ERROR, |procedure| procedure.result);
        let disagree = "every `return` in a procedure must agree.";
        let written = self.procedure_syntax.get(procedure.0 as usize).is_some_and(|syntax| syntax.result.is_some());
        if let Some(value) = value
            && !written
        {
            // trace: D81. The header says what a procedure gives back.
            let checked = self.expression(value, None);
            let found = self.program.types.written(checked.type_id);
            let header =
                self.procedure_syntax.get(procedure.0 as usize).map_or(Place::default(), |syntax| syntax.place);
            let mut message = Message::refused(
                value.place,
                format!(
                    "this `return` gives back {}, but the procedure's header has no `-> Type`",
                    self.describe(checked.type_id)
                ),
            )
            .with_because(format!("line {} is the header", header.line), header)
            .with_why("a procedure that gives back a value says so in its header, with `-> Type`.");
            if !self.program.types.is_error(checked.type_id) && self.program.types.is_settled(checked.type_id) {
                message = message.with_fix(header_with(&self.source_line(header.line), &format!(" -> {found}"), None));
            }
            self.messages.push(message);
            return None;
        }
        if let Some(value) = value {
            if self.context.returned_value == Some(false) {
                self.refuse_why(value.place, "this `return` gives a value, but an earlier one gives none", disagree);
                return None;
            }
            self.context.returned_value = Some(true);
            let checked = self.expression(value, Some(result));
            let checked = self.land(checked, result);
            if self.program.types.resolve(checked.type_id) == NOTHING {
                self.refuse_why(
                    value.place,
                    "this gives nothing, so there's nothing to return",
                    "a procedure that gives nothing back can't be used as a value.",
                );
                return None;
            }
            if !self.program.types.unify(checked.type_id, result) {
                let found = self.describe(checked.type_id);
                let wanted = self.describe(result);
                self.refuse_why(
                    value.place,
                    format!("this `return` gives {found}, but the procedure gives {wanted}"),
                    disagree,
                );
            }
            Some(StatementKind::Return(Some(checked)))
        } else {
            if self.context.returned_value == Some(true) || !self.program.types.unify(result, NOTHING) {
                self.refuse_why(place, "this `return` gives no value, but the procedure gives one", disagree);
                return None;
            }
            self.context.returned_value = Some(false);
            Some(StatementKind::Return(None))
        }
    }

    /// trace: D22, D33, Unio D170 to D174
    #[allow(clippy::too_many_lines)] // a range and a list take the same line, read here in one place
    fn for_each(
        &mut self,
        item: &syntax::Word,
        (item_unused, position_unused): (bool, bool),
        position: Option<&(syntax::PositionKind, syntax::Word)>,
        source: &syntax::Expression,
        body: &[syntax::Statement],
    ) -> Option<StatementKind> {
        // A range is written with `[ ]`, since nothing runs (Unio D165, D170).
        if let syntax::ExpressionKind::Bracketed { receiver, entries } = &source.kind
            && matches!(&receiver.kind, syntax::ExpressionKind::Name(name) if name == "range")
            && self.lookup("range", receiver.place).is_none()
        {
            let (from, end, inclusive, step) = self.range_parts(entries, source.place)?;
            self.context.scopes.push(HashMap::new());
            let item_id = self.make_local(item, WHOLE, VariableKind::LoopItem);
            self.mark_unused(item_id, item_unused);
            let position_id = match position {
                Some((syntax::PositionKind::Key, word)) => {
                    self.refuse(word.place, "`at key` is for going through a `Dict`");
                    None
                }
                Some((kind, word)) => {
                    let id = self.make_local(word, WHOLE, VariableKind::LoopPosition);
                    self.mark_unused(id, position_unused);
                    Some((id, *kind == syntax::PositionKind::Offset))
                }
                None => None,
            };
            self.context.loop_depth += 1;
            let body = self.block(body);
            self.context.loop_depth -= 1;
            let scope = self.context.scopes.pop().unwrap_or_default();
            self.check_unused(&scope);
            return Some(StatementKind::ForRange {
                item: item_id,
                position: position_id,
                from,
                end,
                inclusive,
                step,
                body,
            });
        }
        let checked = self.expression(source, None);
        let item_type = self.program.types.fresh();
        let mut position_type = WHOLE;
        match (self.program.types.get(checked.type_id).clone(), position.map(|(kind, _)| *kind)) {
            (Type::Dict { key, value }, Some(syntax::PositionKind::Key) | None) => {
                self.program.types.unify(item_type, value);
                position_type = key;
            }
            (Type::Dict { .. }, Some(kind)) => {
                // trace: D105. How something is gone through matches how it's reached.
                let word = if kind == syntax::PositionKind::Pos { "pos" } else { "offset" };
                let line = self.source_line(source.place.line);
                let fixed = line.replacen(&format!(" at {word} "), " at key ", 1);
                self.messages.push(
                    Message::refused(source.place, format!("`at {word}` is for going through a list; a `Dict` is gone through `at key`"))
                        .with_fix(fixed)
                        .with_why("a `Dict` is never reached by position, so going through one gives each key; a count for showing is kept with a `mut` name."),
                );
            }
            (Type::Variable(_), Some(syntax::PositionKind::Key)) => {
                let key = self.program.types.fresh();
                let dict = self.program.types.dict(key, item_type);
                self.program.types.unify(checked.type_id, dict);
                position_type = key;
            }
            (_, Some(syntax::PositionKind::Key)) => {
                if !self.program.types.is_error(checked.type_id) {
                    let found = self.describe(checked.type_id);
                    self.refuse(source.place, format!("`at key` is for going through a `Dict`, but this is {found}"));
                }
            }
            _ => self.require(
                checked.type_id,
                Requirement::Iterable { item: item_type },
                source.place,
                "`for each` goes through a list, an array, a dictionary or a `range[…]`",
            ),
        }
        let root = Self::root_variable(&checked);
        self.context.scopes.push(HashMap::new());
        let item_id = self.make_local(item, item_type, VariableKind::LoopItem);
        self.mark_unused(item_id, item_unused);
        let position_id = position.map(|(kind, word)| {
            let id = self.make_local(word, position_type, VariableKind::LoopPosition);
            self.mark_unused(id, position_unused);
            (id, *kind)
        });
        if let Some(root) = root {
            self.context.iterating.push(root);
        }
        self.context.loop_depth += 1;
        let body = self.block(body);
        self.context.loop_depth -= 1;
        if root.is_some() {
            self.context.iterating.pop();
        }
        let scope = self.context.scopes.pop().unwrap_or_default();
        self.check_unused(&scope);
        Some(StatementKind::ForEach { item: item_id, position: position_id, source: checked, body })
    }

    /// A `range[…]`'s numbers: where it starts, where it ends and whether the end is in it, and its step (D22, Unio
    /// D170, D172). A range of plain numbers that could never end is refused.
    pub(super) fn range_parts(
        &mut self,
        entries: &[syntax::Entry],
        place: Place,
    ) -> Option<(Expression, Expression, bool, Option<Expression>)> {
        let mut from = None;
        let mut end = None;
        let mut step = None;
        for entry in entries {
            let label = entry.label.as_ref().map(|label| label.text.as_str());
            let checked = self.expression(&entry.value, Some(WHOLE));
            self.expect(&checked, WHOLE, "a `range`'s number");
            match label {
                Some("from") if from.is_none() => from = Some(checked),
                Some("to") if end.is_none() => end = Some((checked, true)),
                Some("before") if end.is_none() => end = Some((checked, false)),
                Some("step_by") if step.is_none() => step = Some(checked),
                _ => {
                    self.refuse_why(
                        entry.value.place,
                        "a `range` takes `from:`, one of `to:` or `before:`, and an optional `step_by:`",
                        "`to:` includes its number and `before:` stops short of it.",
                    );
                    return None;
                }
            }
        }
        let (Some(from), Some((end, inclusive))) = (from, end) else {
            self.refuse_why(
                place,
                "a `range` takes `from:`, one of `to:` or `before:`, and an optional `step_by:`",
                "`to:` includes its number and `before:` stops short of it.",
            );
            return None;
        };
        self.refuse_plain_range(&from, &end, inclusive, step.as_ref(), place);
        Some((from, end, inclusive, step))
    }

    /// The name an expression reads from, when it's a name or a part of one.
    fn root_variable(expression: &Expression) -> Option<VariableId> {
        match &expression.kind {
            ExpressionKind::Variable(variable) => Some(*variable),
            ExpressionKind::Member { receiver, .. } | ExpressionKind::Item { receiver, .. } => {
                Self::root_variable(receiver)
            }
            _ => None,
        }
    }

    /// A range of plain numbers with a step of 0, or one pointing away from its end, is refused (Unio D172).
    fn refuse_plain_range(
        &mut self,
        from: &Expression,
        end: &Expression,
        inclusive: bool,
        step: Option<&Expression>,
        place: Place,
    ) {
        let plain = |expression: &Expression| match expression.kind {
            ExpressionKind::Whole(value) => Some(value),
            _ => None,
        };
        let step_value = match step {
            Some(step) => plain(step),
            None => Some(1),
        };
        if step_value == Some(0) {
            self.refuse_why(place, "this range's step is 0", "a range with a step of 0 never reaches its end.");
            return;
        }
        if let (Some(from), Some(end), Some(step)) = (plain(from), plain(end), step_value) {
            let away = if step > 0 { from > end } else { from < end };
            let _ = inclusive;
            if away {
                self.refuse_why(
                    place,
                    format!("this range counts by {step} from {from}, away from its end, {end}"),
                    "a range's step points towards its end; a negative `step_by:` counts down.",
                );
            }
        }
    }

    /// trace: D37, D38
    fn match_statement(&mut self, subject: &syntax::Expression, cases: &[syntax::MatchCase]) -> StatementKind {
        let checked = self.expression(subject, None);
        self.work_out_from_cases(checked.type_id, cases);
        let subject_type = self.program.types.resolve(checked.type_id);
        let case_types: Vec<CaseType> = match self.program.types.get(subject_type).clone() {
            Type::OneOf(one_of) => {
                self.program.types.one_ofs.get(one_of as usize).map(|one_of| one_of.cases.clone()).unwrap_or_default()
            }
            Type::Maybe(inner) => vec![
                CaseType { name: "none".to_owned(), fields: Vec::new() },
                CaseType { name: "some".to_owned(), fields: vec![("value".to_owned(), inner)] },
            ],
            Type::Error => Vec::new(),
            Type::Variable(_) => {
                self.not_built(
                    subject.place,
                    "working out a type from a later line (D5), which this `match` needs, isn't",
                );
                Vec::new()
            }
            _ => {
                let found = self.describe(subject_type);
                self.refuse_why(
                    subject.place,
                    format!("`match` reads which case a value is, but this is {found}"),
                    "only a `one_of` or a `Maybe` has cases.",
                );
                Vec::new()
            }
        };
        let known = !case_types.is_empty();
        // A `mut` name could be given a new value inside a case while that case's values are still in use, so the
        // matched name can't change inside the `match`, as what a `for each` goes through can't.
        let root = Self::root_variable(&checked);
        let mut arms: Vec<MatchArm> = Vec::new();
        for case in cases {
            let index = case_types.iter().position(|known| known.name == case.name.text);
            if known && index.is_none() {
                let found = self.describe(subject_type);
                self.refuse(case.name.place, format!("{found} has no case `{}`", case.name.text));
            }
            let case_number = u32::try_from(index.unwrap_or(0)).unwrap_or(0);
            if known && index.is_some() && arms.iter().any(|arm| arm.case == case_number) {
                self.refuse_why(
                    case.name.place,
                    format!("the case `{}` is handled twice", case.name.text),
                    "a `match` has one `case` for each case.",
                );
            }
            self.context.scopes.push(HashMap::new());
            let mut values = Vec::new();
            let fields =
                index.and_then(|index| case_types.get(index)).map(|case| case.fields.clone()).unwrap_or_default();
            if index.is_some() {
                self.check_case_values(case, &fields);
            }
            for word in &case.values {
                let field = fields.iter().position(|(name, _)| *name == word.text);
                let type_id = field.and_then(|field| fields.get(field)).map_or(ERROR, |(_, type_id)| *type_id);
                let id = self.make_local(word, type_id, VariableKind::CaseValue);
                values.push((u32::try_from(field.unwrap_or(0)).unwrap_or(0), id));
            }
            if let Some(root) = root {
                self.context.iterating.push(root);
            }
            let body = self.block(&case.body);
            if root.is_some() {
                self.context.iterating.pop();
            }
            // trace: D61. A value taken out and never read is refused.
            if let Some(scope) = self.context.scopes.pop() {
                self.check_unused(&scope);
            }
            arms.push(MatchArm { case: case_number, values, body });
        }
        if known {
            let missing: Vec<String> = case_types
                .iter()
                .enumerate()
                .filter(|(index, _)| !arms.iter().any(|arm| arm.case as usize == *index))
                .map(|(_, case)| format!("`{}`", case.name))
                .collect();
            if !missing.is_empty() {
                // trace: D37
                self.refuse_why(
                    subject.place,
                    format!("this `match` has no `case` for {}", missing.join(", ")),
                    "a `match` covers every case; there's no catch-all.",
                );
            }
        }
        StatementKind::Match { subject: checked, arms }
    }

    /// Turns a host's type into the script's. A record or a `one_of` must be declared by the script, the same way.
    fn host_type(&mut self, host: &HostType) -> TypeId {
        match host {
            HostType::Nothing => NOTHING,
            HostType::Whole => WHOLE,
            HostType::Float => FLOAT,
            HostType::Bool => BOOL,
            HostType::Text => TEXT,
            HostType::List(item) => {
                let item = self.host_type(item);
                self.program.types.list(item)
            }
            HostType::Dict(key, value) => {
                let key = self.host_type(key);
                let value = self.host_type(value);
                self.program.types.dict(key, value)
            }
            HostType::Maybe(inner) => {
                let inner = self.host_type(inner);
                self.program.types.maybe(inner)
            }
            // trace: D85. The host's types were declared from its own description before anything else.
            HostType::Record { name, .. } => match self.record_names.get(name) {
                Some(&record) if self.host_type_names.contains(name) => self.program.types.intern(Type::Record(record)),
                _ => {
                    self.messages.push(Message::internal(format!("the host's record `{name}` wasn't opened")));
                    ERROR
                }
            },
            HostType::OneOf { name, .. } => match self.one_of_names.get(name) {
                Some(&one_of) if self.host_type_names.contains(name) => self.program.types.intern(Type::OneOf(one_of)),
                _ => {
                    self.messages.push(Message::internal(format!("the host's `one_of` `{name}` wasn't opened")));
                    ERROR
                }
            },
        }
    }
}

/// Whether an expression is a value written out plainly, which is the same every time (D104): a number, text with
/// nothing filled in, a `Bool`, a list, an array's `[all: …]`, a `Dict`, a record or a case built from plain values,
/// or a top-level name made from one with no `mut`.
fn is_plain(expression: &syntax::Expression, names: &HashMap<String, &syntax::Expression>) -> bool {
    use syntax::ExpressionKind as Kind;
    match &expression.kind {
        Kind::Whole(_) | Kind::Float { .. } | Kind::Bool(_) | Kind::EmptyDict => true,
        Kind::Text(text) => !text.replace("\\{", "").contains('{'),
        Kind::Unary { operator: syntax::UnaryOperator::Negate, operand } => {
            matches!(operand.kind, Kind::Whole(_) | Kind::Float { .. })
        }
        Kind::Name(name) => names.contains_key(name),
        Kind::List(items) => items.iter().all(|item| is_plain(item, names)),
        Kind::Pairs(pairs) => pairs.iter().all(|(key, value)| {
            (matches!(&key.kind, Kind::Name(name) if name == "all") || is_plain(key, names)) && is_plain(value, names)
        }),
        Kind::Group(inner) => is_plain(inner, names),
        Kind::Member { .. } | Kind::TypeName(_) => expressions::type_path(expression).is_some(),
        Kind::Bracketed { receiver, entries } => {
            expressions::type_path(receiver).is_some() && entries.iter().all(|entry| is_plain(&entry.value, names))
        }
        _ => false,
    }
}

/// A procedure's header line with something added after its name and parameters: `-> Type` before any `! [ … ]`,
/// or `! [ … ]` in place of the one written, at the line's end (D81, D84).
pub(super) fn header_with(line: &str, result: &str, failures: Option<&str>) -> String {
    let (head, written_failures) = match line.find(" ! [") {
        Some(at) => (line.get(..at).unwrap_or(line), line.get(at..).unwrap_or("")),
        None => (line, ""),
    };
    let failures = failures.map_or_else(|| written_failures.to_owned(), str::to_owned);
    format!("{head}{result}{failures}")
}

/// The name a change starts from, where it's written, and the steps after it with their places (D62).
type WrittenWay<'a> = (&'a String, Place, Vec<(Step<'a>, Place)>);

enum Step<'a> {
    Field(&'a syntax::Word),
    Reach(&'a [syntax::Entry]),
}

/// Whether a block has a `fail` or a `?` anywhere in it: whether its procedure can fail at all (D44).
fn block_can_fail(block: &[syntax::Statement]) -> bool {
    block.iter().any(statement_can_fail)
}

fn statement_can_fail(statement: &syntax::Statement) -> bool {
    match &statement.kind {
        syntax::StatementKind::Fail(_) => true,
        syntax::StatementKind::Name { value, .. } | syntax::StatementKind::Instead(value) => expression_can_fail(value),
        syntax::StatementKind::Change { place, value } => expression_can_fail(place) || expression_can_fail(value),
        syntax::StatementKind::Expression(expression) => expression_can_fail(expression),
        syntax::StatementKind::If { branches, otherwise } => {
            branches.iter().any(|(condition, block)| expression_can_fail(condition) || block_can_fail(block))
                || otherwise.as_ref().is_some_and(|block| block_can_fail(block))
        }
        syntax::StatementKind::While { condition, body } => expression_can_fail(condition) || block_can_fail(body),
        syntax::StatementKind::ForEach { source, body, .. } => expression_can_fail(source) || block_can_fail(body),
        syntax::StatementKind::Match { subject, cases } => {
            expression_can_fail(subject) || cases.iter().any(|case| block_can_fail(&case.body))
        }
        syntax::StatementKind::Return(value) => value.as_ref().is_some_and(expression_can_fail),
        syntax::StatementKind::Break => false,
    }
}

fn expression_can_fail(expression: &syntax::Expression) -> bool {
    use syntax::ExpressionKind as Kind;
    let entries_can_fail = |entries: &[syntax::Entry]| entries.iter().any(|entry| expression_can_fail(&entry.value));
    match &expression.kind {
        Kind::Try(_) => true,
        Kind::Whole(_)
        | Kind::Float { .. }
        | Kind::Bool(_)
        | Kind::Text(_)
        | Kind::Name(_)
        | Kind::TypeName(_)
        | Kind::EmptyDict => false,
        Kind::Member { receiver, .. } => expression_can_fail(receiver),
        Kind::Call { callee, arguments } => expression_can_fail(callee) || entries_can_fail(arguments),
        Kind::Bracketed { receiver, entries } => expression_can_fail(receiver) || entries_can_fail(entries),
        Kind::List(items) => items.iter().any(expression_can_fail),
        Kind::Pairs(pairs) => pairs.iter().any(|(key, value)| expression_can_fail(key) || expression_can_fail(value)),
        Kind::Unary { operand, .. } => expression_can_fail(operand),
        Kind::Binary { left, right, .. } => expression_can_fail(left) || expression_can_fail(right),
        Kind::Group(inner) => expression_can_fail(inner),
        Kind::Handled { call, fallback } => {
            expression_can_fail(call)
                || matches!(fallback, syntax::Fallback::Value(value) if expression_can_fail(value))
        }
        Kind::Otherwise { call, cases, .. } => {
            expression_can_fail(call) || cases.iter().any(|case| block_can_fail(&case.body))
        }
        Kind::Slide { start, steps } => {
            expression_can_fail(start)
                || steps.iter().any(|step| step.arguments.as_deref().is_some_and(entries_can_fail))
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::signature::DoorParameter;

    /// The test host's doors, as `tests/README.md` describes them.
    pub(crate) fn test_doors() -> Vec<DoorSignature> {
        let door =
            |name: &str, parameters: Vec<(Option<&str>, HostType)>, result: HostType, can_pause: bool| DoorSignature {
                name: name.to_owned(),
                parameters: parameters
                    .into_iter()
                    .map(|(label, host_type)| DoorParameter { label: label.map(str::to_owned), host_type })
                    .collect(),
                result,
                failures: Vec::new(),
                can_pause,
            };
        vec![
            door("show_msg", vec![(None, HostType::Text)], HostType::Nothing, false),
            door("show_msg.same_line", vec![(None, HostType::Text)], HostType::Nothing, false),
            door("ask", vec![(None, HostType::Text)], HostType::Text, false),
            door("rand", vec![(Some("from"), HostType::Whole), (Some("to"), HostType::Whole)], HostType::Whole, false),
            door("wait", vec![(Some("seconds"), HostType::Whole)], HostType::Nothing, true),
        ]
    }

    pub(crate) fn check_source(source: &str) -> Result<Program, Vec<Message>> {
        let script = crate::syntax::parser::parse(source).map_err(|message| vec![message])?;
        check(&script, &test_doors(), &[])
    }

    fn whats(source: &str) -> Vec<String> {
        match check_source(source) {
            Ok(_) => Vec::new(),
            Err(messages) => messages
                .iter()
                .map(|message| format!("{}: {}", message.kind.word(), message.what.clone().unwrap_or_default()))
                .collect(),
        }
    }

    // trace: D60, D61
    #[test]
    fn d61_unread_parameters_and_loop_names_take_unused() {
        assert_eq!(
            whats("proc tick[amount: Whole]\n    show_msg(\"x\")\nend\n"),
            vec!["refused: `amount` is never used"]
        );
        assert!(whats("proc tick[unused amount: Whole]\n    show_msg(\"x\")\nend\n").is_empty());
        assert!(whats("proc tick\n    show_msg(\"x\")\nend\ntick()\n").is_empty());
        assert_eq!(
            whats("for each round in range[from: 1, to: 3]\n    show_msg(\"x\")\nend\n"),
            vec!["refused: `round` is never used"]
        );
        assert!(whats("for each unused round in range[from: 1, to: 3]\n    show_msg(\"x\")\nend\n").is_empty());
        let event =
            "one_of Event\n    healed[amount: Whole, by: Text]\nend\nevent = Event.healed[amount: 1, by: \"Mia\"]\n";
        assert!(whats(&format!("{event}match event\n    case healed[by]\n        show_msg(by)\nend\n")).is_empty());
        assert_eq!(
            whats(&format!("{event}match event\n    case healed[amount, by]\n        show_msg(by)\nend\n")),
            vec!["refused: `amount` is never used"]
        );
    }

    // trace: D64, D96
    #[test]
    fn d64_two_texts_compare_with_equals_and_nothing_else() {
        assert!(whats("answer = \"yes\"\nif answer == \"yes\"\n    show_msg(answer)\nend\n").is_empty());
        assert!(whats("answer = \"yes\"\nif answer != \"\"\n    show_msg(answer)\nend\n").is_empty());
        assert_eq!(
            whats("answer = \"yes\"\nif answer < \"zebra\"\n    show_msg(answer)\nend\n"),
            vec!["refused: two texts aren't put in order with `<`, `>`, `at_least` or `at_most`"]
        );
    }

    // trace: D68, Unio D9, D57, D149
    #[test]
    fn d68_returns_and_ifs_that_leave() {
        let guard = "proc pick[n: Whole] -> Whole\n    if n > 0\n        return 1\n    end\n    return 2\nend\n";
        assert_eq!(whats(guard), vec!["refused: this line follows an `if` that can leave"]);
        let after = "proc pick[n: Whole] -> Whole\n    return n\n    show_msg(\"x\")\nend\n";
        assert_eq!(whats(after), vec!["refused: this line can never run"]);
        let bare = "proc tick[n: Whole]\n    show_msg(be.Text(n))\n    return\nend\n";
        assert_eq!(
            whats(bare),
            vec!["refused: this `return` is the procedure's last line, where the procedure ends anyway"]
        );
        let forever = "proc first_big[start: Whole] -> Whole\n    mut n = start\n    while true\n        if n > 100\n            return n\n        else\n            n = n + 1\n        end\n    end\nend\n";
        assert_eq!(
            whats(forever),
            vec!["refused: `first_big` gives a value, but it can reach its `end` without a `return`"]
        );
        let fine = "proc first_big[start: Whole] -> Whole\n    mut n = start\n    while true\n        if n > 100\n            return n\n        else\n            n = n + 1\n        end\n    end\n    return n\nend\n";
        assert!(whats(fine).is_empty(), "{:?}", whats(fine));
    }

    // trace: D69
    #[test]
    fn d69_a_for_each_over_a_top_level_name() {
        let changes = "mut enemies = [1, 2]\nproc hurt[n: Whole]\n    enemies.add(n)\nend\nproc all\n    for each e in enemies\n        hurt(e)\n    end\nend\n";
        assert_eq!(
            whats(changes),
            vec!["refused: `hurt` can change `enemies`, which this `for each` is going through"]
        );
        let copied = "mut enemies = [1, 2]\nproc hurt[n: Whole]\n    enemies.add(n)\nend\nproc all\n    now = enemies\n    for each e in now\n        hurt(e)\n    end\nend\n";
        assert!(whats(copied).is_empty(), "{:?}", whats(copied));
    }

    // trace: D70, D71
    #[test]
    fn d70_dropped_values_and_d71_no_shadowing() {
        let call = "proc two -> Whole\n    return 2\nend\ntwo()\n";
        assert_eq!(whats(call), vec!["refused: this call gives `Whole`, but nothing uses it"]);
        assert!(whats("proc two -> Whole\n    return 2\nend\nunused dropped = two()\n").is_empty());
        assert_eq!(whats("score = 1\nscore + 1\n"), vec!["refused: this line works out `Whole`, but nothing uses it"]);
        let parameter = "results = [3]\nproc best[results: List<Whole>] -> Whole\n    return results.len\nend\nshow_msg(be.Text(results.len))\n";
        assert_eq!(
            whats(parameter),
            vec!["refused: `results` is already a top-level name, so a parameter can't be named `results`"]
        );
        let procedure = "proc roll -> Whole\n    return 4\nend\nroll = roll()\nshow_msg(be.Text(roll))\n";
        assert_eq!(
            whats(procedure).first().map(String::as_str),
            Some("refused: `roll` is already a procedure, so nothing else can be named `roll`")
        );
        assert_eq!(
            whats("show_msg = \"hi\"\n").first().map(String::as_str),
            Some("refused: `show_msg` is already one of the host's doors, so nothing else can be named `show_msg`")
        );
    }

    // trace: D72
    #[test]
    fn d72_records_in_records_and_types_that_hold_themselves() {
        let tree = "one_of Tree\n    leaf\n    node[left: Tree, right: Tree]\nend\n";
        assert!(whats(tree).is_empty(), "{:?}", whats(tree));
        let nested =
            "record Stats\n    hp: Whole\nend\n\nrecord Enemy\n    stats: Stats\n    friends: List<Enemy>\nend\n";
        assert!(whats(nested).is_empty(), "{:?}", whats(nested));
        assert_eq!(
            whats("record Node\n    value: Whole\n    next: Node\nend\n"),
            vec!["refused: `Node`'s field `next` holds `Node`, with no way for a value to end"]
        );
        assert!(whats("record Node\n    value: Whole\n    next: Maybe<Node>\nend\n").is_empty());
        assert_eq!(
            whats("one_of Loop\n    again[more: Loop]\nend\n"),
            vec!["refused: every case of `Loop` holds more that can't end, so no value of it could end"]
        );
    }

    // trace: D73
    #[test]
    fn d73_a_list_moves_between_limits() {
        let bags = "small_bag: List<Text>[len.max: 3] = [\"a\"]\nbig_bag = [\"a\", \"b\"]\n";
        let shown = "show_msg(be.Text(moved.len))\n";
        assert!(
            whats(&format!("{bags}moved: List<Text>[len.max: 10] = small_bag\nunused other = big_bag\n{shown}"))
                .is_empty()
        );
        assert!(whats(&format!("{bags}moved: List<Text> = small_bag\nunused other = big_bag\n{shown}")).is_empty());
        assert_eq!(
            whats(&format!("{bags}moved: List<Text>[len.max: 2] = small_bag\nunused other = big_bag\n{shown}")),
            vec!["refused: this can fail with `full`, and nothing deals with the failure"]
        );
        assert!(
            whats(&format!("{bags}moved: List<Text>[len.max: 2] = big_bag ?? []\nunused other = small_bag\n{shown}"))
                .is_empty()
        );
        assert_eq!(
            whats("few: List<Whole>[len.max: 2] = [1, 2, 3]\nshow_msg(be.Text(few.len))\n"),
            vec!["refused: this list has 3 items, but its `len.max` is 2"]
        );
        assert_eq!(
            whats("ages = [\"ann\": 1, \"ann\": 3]\nshow_msg(be.Text(ages.len))\n"),
            vec!["refused: the key \"ann\" is written twice in this dictionary"]
        );
    }

    #[test]
    fn every_test_script_passes_checking() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests");
        let mut checked = 0;
        for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
            let path = entry.path().join("program.seal");
            let Ok(source) = std::fs::read_to_string(&path) else { continue };
            assert_eq!(whats(&source), Vec::<String>::new(), "{}", path.display());
            checked += 1;
        }
        assert!(checked >= 14);
    }

    // trace: D2
    #[test]
    fn d02_an_unused_name_is_refused() {
        assert_eq!(whats("mut tries = 0\ntires = tries + 1\n"), vec!["refused: `tires` is never used"]);
        assert_eq!(
            whats("lives = 3\nlives = 4\nshow_msg(\"{lives}\")\n"),
            vec!["refused: `lives` can't be given a new value"]
        );
    }

    // trace: D81, D88
    #[test]
    fn d81_a_procedure_header_writes_its_types() {
        assert_eq!(
            whats(
                "mut score = 0\nproc on_hit[points: Whole]\n    score = score + points\nend\nshow_msg(\"{score}\")\n"
            ),
            Vec::<String>::new()
        );
        assert_eq!(whats("proc add[a, b]\n    show_msg(\"x\")\nend\n"), vec!["refused: the parameter `a` has no type"]);
        assert_eq!(
            whats("proc double[n: Whole]\n    return n * 2\nend\n"),
            vec!["refused: this `return` gives back `Whole`, but the procedure's header has no `-> Type`"]
        );
        assert!(
            whats("proc add[a, b : Whole] -> Whole\n    return a + b\nend\nshow_msg(\"{add(1, 2)}\")\n").len() == 1
        );
        assert_eq!(
            whats("proc add[a, b: Whole] -> Whole\n    return a + b\nend\n"),
            vec!["refused: this `:` gives a group of parameters their type, so it has a space on each side"]
        );
        assert!(
            whats("proc add[a, b : Whole] -> Whole\n    return a + b\nend\nsum = add(1, 2)\nshow_msg(\"{sum}\")\n")
                .is_empty()
        );
    }

    // trace: D6, D81
    #[test]
    fn d06_every_return_must_agree() {
        let messages = whats(
            "proc pick[n: Whole] -> Whole\n    if n > 0\n        return 1\n    else\n        return \"none\"\n    end\nend\n",
        );
        assert_eq!(messages, vec!["refused: this `return` gives `Text`, but the procedure gives `Whole`"]);
    }

    // trace: D44, Unio D23
    #[test]
    fn d44_a_failure_must_be_dealt_with() {
        let source = "proc risky[n: Whole] -> Whole ! [too_big]\n    if n > 1\n        fail too_big\n    else\n        return n\n    end\nend\nx = risky(2)\nshow_msg(\"{x}\")\n";
        assert_eq!(whats(source), vec!["refused: this can fail with `too_big`, and nothing deals with the failure"]);
        // trace: D84. A procedure that can fail says so in its header.
        let unwritten = "proc risky[n: Whole] -> Whole\n    if n > 1\n        fail too_big\n    else\n        return n\n    end\nend\nx = risky(2) ?? 0\nshow_msg(\"{x}\")\n";
        assert_eq!(whats(unwritten), vec!["refused: `risky` can fail with `too_big`, but its header doesn't say so"]);
    }
}
