//! Checking a script at load time: names, `mut` and `unused`, types worked out, failures and pauses worked out, and
//! every refusal (D17). The result is the checked tree the compiler reads.

mod calls;
mod expressions;
mod finish;
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

/// The words Seal provides itself (D11, D12, D22). A script's own names can't take them.
const BUILT_IN_WORDS: [&str; 4] = ["fmt", "range", "sqrt", "be"];

/// The types Seal provides itself (D16, D32, D38).
const BUILT_IN_TYPES: [&str; 8] = ["Whole", "Float", "Bool", "Text", "List", "Array", "Dict", "Maybe"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProcedureState {
    Unchecked,
    InProgress,
    Checked,
}

/// Something a type must allow, checked once the type is worked out (D5).
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
    /// What `fmt`'s braces show: `Whole`, `Float`, `Bool` or `Text` (D12).
    Showable,
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
pub fn check(script: &syntax::Script, doors: &[DoorSignature]) -> Result<Program, Vec<Message>> {
    let mut checker = Checker {
        doors,
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
        self.declare_types(script);
        self.declare_procedures(script);
        self.declare_globals(script);
        self.context.scopes.push(HashMap::new());
        let mut top_level = Vec::new();
        let mut left = false;
        for item in &script.items {
            if let syntax::Item::Statement(statement) = item {
                if left {
                    self.not_designed(statement.place, "a line after a `return`, `break` or `fail` in the same block");
                    left = false;
                }
                self.top_level_line = Some(statement.place);
                if let Some(checked) = self.statement(statement) {
                    left = Self::leaves(&checked);
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

    fn declare_types(&mut self, script: &'a syntax::Script) {
        let mut records = Vec::new();
        let mut one_ofs = Vec::new();
        for item in &script.items {
            let name = match item {
                syntax::Item::Record(record) => &record.name,
                syntax::Item::OneOf(one_of) => &one_of.name,
                _ => continue,
            };
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
            let fields = self.declare_fields(&record.fields);
            for (field, declaration) in fields.iter().zip(&record.fields) {
                if matches!(self.program.types.get(field.1), Type::Record(_)) {
                    self.not_designed(declaration.written_type.place, "a record inside a record");
                }
            }
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
                self.not_designed(one_of.name.place, "a `one_of` with no cases");
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

    /// Whether a value of type `from` can hold a value of the declared type `target` inside it.
    fn type_reaches(&self, from: TypeId, target: &Type, seen: &mut HashSet<TypeId>) -> bool {
        let from = self.program.types.resolve(from);
        if !seen.insert(from) {
            return false;
        }
        let kind = self.program.types.get(from).clone();
        if kind == *target {
            return true;
        }
        match kind {
            Type::List { item, .. } | Type::Array { item, .. } | Type::Maybe(item) => {
                self.type_reaches(item, target, seen)
            }
            Type::Dict { key, value } => self.type_reaches(key, target, seen) || self.type_reaches(value, target, seen),
            Type::Record(record) => {
                let fields: Vec<TypeId> = self
                    .program
                    .types
                    .records
                    .get(record as usize)
                    .map(|record| record.fields.iter().map(|field| field.1).collect())
                    .unwrap_or_default();
                fields.into_iter().any(|field| self.type_reaches(field, target, seen))
            }
            Type::OneOf(one_of) => {
                let fields: Vec<TypeId> = self
                    .program
                    .types
                    .one_ofs
                    .get(one_of as usize)
                    .map(|one_of| {
                        one_of.cases.iter().flat_map(|case| case.fields.iter().map(|field| field.1)).collect()
                    })
                    .unwrap_or_default();
                fields.into_iter().any(|field| self.type_reaches(field, target, seen))
            }
            _ => false,
        }
    }

    fn refuse_types_holding_themselves(&mut self, script: &syntax::Script) {
        for item in &script.items {
            let (name, target, fields): (&syntax::Word, Type, Vec<TypeId>) = match item {
                syntax::Item::Record(record) => {
                    let Some(&index) = self.record_names.get(&record.name.text) else { continue };
                    let fields = self
                        .program
                        .types
                        .records
                        .get(index as usize)
                        .map(|record| record.fields.iter().map(|field| field.1).collect())
                        .unwrap_or_default();
                    (&record.name, Type::Record(index), fields)
                }
                syntax::Item::OneOf(one_of) => {
                    let Some(&index) = self.one_of_names.get(&one_of.name.text) else { continue };
                    let fields = self
                        .program
                        .types
                        .one_ofs
                        .get(index as usize)
                        .map(|one_of| {
                            one_of.cases.iter().flat_map(|case| case.fields.iter().map(|field| field.1)).collect()
                        })
                        .unwrap_or_default();
                    (&one_of.name, Type::OneOf(index), fields)
                }
                _ => continue,
            };
            let holds_itself = fields.into_iter().any(|field| self.type_reaches(field, &target, &mut HashSet::new()));
            if holds_itself {
                self.not_designed(name.place, format!("a type that holds itself, as `{}` does,", name.text));
            }
        }
    }

    /// A whole number written plainly, as a type's setting needs.
    fn plain_whole(&mut self, expression: &syntax::Expression) -> Option<i64> {
        if let syntax::ExpressionKind::Whole(digits) = &expression.kind {
            if let Ok(value) = digits.parse::<i64>() {
                Some(value)
            } else {
                self.refuse(expression.place, format!("`{digits}` is too big for a `Whole`"));
                None
            }
        } else {
            self.not_designed(expression.place, "a type's setting that isn't a plain whole number");
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
        if written.blanks.len() != blank_count {
            let what = match blank_count {
                0 => format!("`{name}` has no `< >`"),
                1 => format!("`{name}` takes one type in `< >`, as in `{name}<Text>`"),
                _ => "`Dict` takes a key and a value, as in `Dict<key: Text, value: Whole>`".to_owned(),
            };
            self.refuse(written.place, what);
            return ERROR;
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
                let item = self.written_type(inner);
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
        if BUILT_IN_WORDS.contains(&word.as_str()) {
            self.not_designed(name_place, format!("a procedure named after the built-in `{word}`"));
            return;
        }
        if self.doors.iter().any(|door| door.name.split('.').next() == Some(word.as_str())) {
            self.not_designed(name_place, format!("a procedure named after the host's door `{word}`"));
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
            let type_id = match &parameter.written_type {
                Some(written) => self.written_type(written),
                None => self.program.types.fresh(),
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
        let result = match &procedure.result {
            Some(written) => self.written_type(written),
            None => self.program.types.fresh(),
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
            let Some(value) = self.plain_value(&declaration.value) else { continue };
            let parameter_type = parameters.get(position).map_or(ERROR, |&parameter| self.variable_type(parameter));
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
        let id = ProcedureId(u32::try_from(self.program.procedures.len()).unwrap_or(u32::MAX));
        self.program.procedures.push(Procedure {
            name: name.clone(),
            parameters,
            labels,
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

    /// A plain value, as a default must be: never a calculation or a call (D42).
    fn plain_value(&mut self, expression: &syntax::Expression) -> Option<Expression> {
        let plain = match &expression.kind {
            syntax::ExpressionKind::Whole(_)
            | syntax::ExpressionKind::Float { .. }
            | syntax::ExpressionKind::Bool(_)
            | syntax::ExpressionKind::Text(_) => true,
            syntax::ExpressionKind::Unary { operator: syntax::UnaryOperator::Negate, operand } => {
                matches!(operand.kind, syntax::ExpressionKind::Whole(_) | syntax::ExpressionKind::Float { .. })
            }
            _ => false,
        };
        if plain {
            Some(self.expression(expression, None))
        } else if matches!(
            expression.kind,
            syntax::ExpressionKind::Call { .. }
                | syntax::ExpressionKind::Binary { .. }
                | syntax::ExpressionKind::Slide { .. }
        ) {
            // trace: D42
            self.refuse_why(
                expression.place,
                "a default is a plain value, never a calculation or a call",
                "a default is worked out when the script loads.",
            );
            None
        } else {
            self.not_designed(expression.place, "a default that isn't a number, a `Bool` or text");
            None
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

    /// Whether a new name may be spelt `name`: it can't be a word that's already a procedure, a door or a built-in.
    fn check_new_name(&mut self, name: &syntax::Word) -> bool {
        if BUILT_IN_WORDS.contains(&name.text.as_str()) {
            self.not_designed(name.place, format!("a name spelt like the built-in `{}`", name.text));
            return false;
        }
        if self.words.contains(&name.text) {
            self.not_designed(name.place, format!("a name spelt like the procedure or door `{}`", name.text));
            return false;
        }
        true
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
            self.not_designed(case.name.place, "taking out a case's values in another order than they're declared");
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
            self.not_designed(
                name.place,
                format!(
                    "a name made inside a procedure by `for each` or `case`, spelt like the top-level name `{}`,",
                    name.text
                ),
            );
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
                    "a `for each` that doesn't read where it is leaves its `at` part out.",
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
            // A parameter may share a top-level name's spelling, and stands in front of it inside its procedure
            // (tests/d038_maybe).
            if let Some(variable) = self.variable(parameter) {
                let word = syntax::Word { text: variable.name.clone(), place: variable.place };
                self.check_new_name(&word);
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
                self.not_designed(
                    procedure.end_place,
                    format!(
                        "a procedure that gives a value but can reach its `end` without a `return`, as `{}` can,",
                        procedure.name()
                    ),
                );
            }
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
        let mut left = false;
        for statement in statements {
            if left {
                self.not_designed(statement.place, "a line after a `return`, `break` or `fail` in the same block");
                left = false;
            }
            if let Some(checked) = self.statement(statement) {
                left = Self::leaves(&checked);
                block.push(checked);
            }
        }
        let scope = self.context.scopes.pop().unwrap_or_default();
        self.check_unused(&scope);
        block
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

    fn block_leaves(block: &Block) -> bool {
        block.last().is_some_and(Self::leaves)
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
                    if !is_call {
                        self.not_designed(expression.place, "a line that only works out a value");
                    } else if !self.program.types.unify(checked.type_id, NOTHING) {
                        self.not_designed(
                            expression.place,
                            "a call that gives a value standing alone, its value unused,",
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
            syntax::StatementKind::ForEach { item, item_unused, position, source, body } => {
                self.for_each(item, *item_unused, position.as_ref(), source, body)?
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
        let wanted = written_type.map(|written| self.written_type(written));
        let checked = self.expression(value, wanted);
        let type_id = match wanted {
            Some(wanted) => {
                self.expect(&checked, wanted, &format!("`{}`", name.text));
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
                if let Some(variable) = self.program.variables.get_mut(global.0 as usize) {
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
        changeable.then_some(StatementKind::Set { variable, value: checked })
    }

    /// `name.field = value`, `name[pos: n] = value` and `name[key: k] = value` (D27, D32, Unio D80).
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
                        StatementKind::SetItem { variable, index, zero_based: label == "offset", value: checked }
                    }
                    (Type::Dict { key, value: value_type }, Some("key")) => {
                        let checked_key = self.expression(&entry.value, Some(key));
                        self.expect(&checked_key, key, "the dictionary's key");
                        let checked = self.expression(value, Some(value_type));
                        self.expect(&checked, value_type, "the dictionary's value");
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
                            self.not_designed(
                                place,
                                "a `Dict`'s key partway along a change, with more steps after it,",
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
        if let Some(value) = value {
            if self.context.returned_value == Some(false) {
                self.refuse_why(value.place, "this `return` gives a value, but an earlier one gives none", disagree);
                return None;
            }
            self.context.returned_value = Some(true);
            let checked = self.expression(value, Some(result));
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
        item_unused: bool,
        position: Option<&(syntax::PositionKind, syntax::Word)>,
        source: &syntax::Expression,
        body: &[syntax::Statement],
    ) -> Option<StatementKind> {
        // A range is written with `[ ]`, since nothing runs (Unio D165, D170).
        if let syntax::ExpressionKind::Bracketed { receiver, entries } = &source.kind
            && matches!(&receiver.kind, syntax::ExpressionKind::Name(name) if name == "range")
            && self.lookup("range", receiver.place).is_none()
        {
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
                    source.place,
                    "a `range` takes `from:`, one of `to:` or `before:`, and an optional `step_by:`",
                    "`to:` includes its number and `before:` stops short of it.",
                );
                return None;
            };
            self.refuse_plain_range(&from, &end, inclusive, step.as_ref(), source.place);
            self.context.scopes.push(HashMap::new());
            let item_id = self.make_local(item, WHOLE, VariableKind::LoopItem);
            self.mark_unused(item_id, item_unused);
            let position_id = match position {
                Some((syntax::PositionKind::Key, word)) => {
                    self.refuse(word.place, "`at key` is for going through a `Dict`");
                    None
                }
                Some((kind, word)) => Some((
                    self.make_local(word, WHOLE, VariableKind::LoopPosition),
                    *kind == syntax::PositionKind::Offset,
                )),
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
            (Type::Dict { .. }, Some(_)) => {
                self.not_designed(source.place, "`at pos` or `at offset` when going through a `Dict`");
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
        let position_id =
            position.map(|(kind, word)| (self.make_local(word, position_type, VariableKind::LoopPosition), *kind));
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
    fn host_type(&mut self, host: &HostType, place: Place) -> TypeId {
        match host {
            HostType::Nothing => NOTHING,
            HostType::Whole => WHOLE,
            HostType::Float => FLOAT,
            HostType::Bool => BOOL,
            HostType::Text => TEXT,
            HostType::List(item) => {
                let item = self.host_type(item, place);
                self.program.types.list(item)
            }
            HostType::Dict(key, value) => {
                let key = self.host_type(key, place);
                let value = self.host_type(value, place);
                self.program.types.dict(key, value)
            }
            HostType::Maybe(inner) => {
                let inner = self.host_type(inner, place);
                self.program.types.maybe(inner)
            }
            HostType::Record { name, fields } => {
                let Some(&record) = self.record_names.get(name) else {
                    self.not_designed(
                        place,
                        format!("using the host's record `{name}` without declaring it in the script"),
                    );
                    return ERROR;
                };
                let host_fields: Vec<(String, TypeId)> =
                    fields.iter().map(|(field, host)| (field.clone(), self.host_type(host, place))).collect();
                let declared = self.program.types.records.get(record as usize).map(|record| record.fields.clone());
                if !self.same_fields(declared.as_deref().unwrap_or(&[]), &host_fields) {
                    self.refuse_why(
                        place,
                        format!("the script's record `{name}` doesn't have the fields the host's `{name}` has"),
                        "a record crosses a door as the host's struct, field for field.",
                    );
                    return ERROR;
                }
                self.program.types.intern(Type::Record(record))
            }
            HostType::OneOf { name, cases } => {
                let Some(&one_of) = self.one_of_names.get(name) else {
                    self.not_designed(
                        place,
                        format!("using the host's `one_of` `{name}` without declaring it in the script"),
                    );
                    return ERROR;
                };
                let declared = self
                    .program
                    .types
                    .one_ofs
                    .get(one_of as usize)
                    .map(|one_of| one_of.cases.clone())
                    .unwrap_or_default();
                let mut same = declared.len() == cases.len();
                for (declared_case, (case_name, fields)) in declared.iter().zip(cases) {
                    let host_fields: Vec<(String, TypeId)> =
                        fields.iter().map(|(field, host)| (field.clone(), self.host_type(host, place))).collect();
                    same &= declared_case.name == *case_name && self.same_fields(&declared_case.fields, &host_fields);
                }
                if !same {
                    self.refuse_why(
                        place,
                        format!("the script's `one_of` `{name}` doesn't have the cases the host's `{name}` has"),
                        "a `one_of` crosses a door as the host's enum, case for case.",
                    );
                    return ERROR;
                }
                self.program.types.intern(Type::OneOf(one_of))
            }
        }
    }

    fn same_fields(&mut self, declared: &[(String, TypeId)], host: &[(String, TypeId)]) -> bool {
        declared.len() == host.len()
            && declared.iter().zip(host).all(|((declared_name, declared_type), (host_name, host_type))| {
                declared_name == host_name
                    && self.program.types.resolve(*declared_type) == self.program.types.resolve(*host_type)
            })
    }
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
        Kind::Otherwise { call, cases } => {
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
        check(&script, &test_doors())
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
        assert_eq!(whats("proc tick[amount]\n    show_msg(\"x\")\nend\n"), vec!["refused: `amount` is never used"]);
        assert!(whats("proc tick[unused amount]\n    show_msg(\"x\")\nend\n").is_empty());
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

    // trace: D64
    #[test]
    fn d64_two_texts_compare_with_equals_and_nothing_else() {
        assert!(whats("answer = \"yes\"\nif answer == \"yes\"\n    show_msg(answer)\nend\n").is_empty());
        assert!(whats("answer = \"yes\"\nif answer != \"\"\n    show_msg(answer)\nend\n").is_empty());
        assert_eq!(
            whats("answer = \"yes\"\nif answer < \"zebra\"\n    show_msg(answer)\nend\n"),
            vec!["NotDesignedYet: putting two texts in order, as `<` and `>` would, isn't decided yet"]
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
            whats("lives = 3\nlives = 4\nshow_msg(\"\" & lives)\n"),
            vec!["refused: `lives` can't be given a new value"]
        );
    }

    // trace: D5
    #[test]
    fn d05_parameter_types_are_worked_out_from_the_body() {
        assert_eq!(
            whats("mut score = 0\nproc on_hit[points]\n    score = score + points\nend\nshow_msg(\"\" & score)\n"),
            Vec::<String>::new()
        );
        assert_eq!(
            whats("proc add[a, b]\n    return a + b\nend\n"),
            vec!["refused: `a` and `b` could be a `Whole` or a `Float`, and `add`'s body doesn't say which"]
        );
    }

    // trace: D6
    #[test]
    fn d06_every_return_must_agree() {
        let messages =
            whats("proc pick[n: Whole]\n    if n > 0\n        return 1\n    end\n    return \"none\"\nend\n");
        assert_eq!(messages, vec!["refused: this `return` gives `Text`, but the procedure gives `Whole`"]);
    }

    // trace: D44, Unio D23
    #[test]
    fn d44_a_failure_must_be_dealt_with() {
        let source = "proc risky[n: Whole]\n    if n > 1\n        fail too_big\n    end\n    return n\nend\nx = risky(2)\nshow_msg(\"\" & x)\n";
        assert_eq!(whats(source), vec!["refused: this can fail with `too_big`, and nothing deals with the failure"]);
    }
}
