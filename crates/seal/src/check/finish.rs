//! The checks that need a whole procedure, or the whole script: deciding the types a body leaves open (D5), and
//! working out failures (D44) and pauses (D52).

use std::collections::{BTreeSet, HashMap};

use super::tree::{
    Conversion, Expression, ExpressionKind, FailureId, HandleFallback, Inside, PathStep, ProcedureId, Program,
    Statement, StatementKind, VariableId,
};
use super::types::{ERROR, FLOAT, TEXT, Type, TypeId, WHOLE};
use super::{Checker, Pending, Requirement};
use crate::message::{Message, Place};
use crate::signature::DoorSignature;

/// Something that looks at every statement and expression of a block.
trait Visitor {
    fn statement(&mut self, _statement: &Statement) {}
    /// `handled` says whether a `?`, `??` or `otherwise` deals with this expression's failure.
    fn expression(&mut self, _expression: &Expression, _handled: bool) {}
}

/// Two visitors looking at a block in one walk, each seeing everything in the order it would alone.
struct Both<'v>(&'v mut dyn Visitor, &'v mut dyn Visitor);

impl Visitor for Both<'_> {
    fn statement(&mut self, statement: &Statement) {
        self.0.statement(statement);
        self.1.statement(statement);
    }

    fn expression(&mut self, expression: &Expression, handled: bool) {
        self.0.expression(expression, handled);
        self.1.expression(expression, handled);
    }
}

/// Whether a body's failures and pauses can depend on other procedures': only a call, or a change to a part, which
/// can be one, looks at them (D140).
#[derive(Default)]
struct Dependence {
    found: bool,
}

impl Visitor for Dependence {
    fn expression(&mut self, expression: &Expression, _handled: bool) {
        self.found |= matches!(expression.kind, ExpressionKind::Call { .. } | ExpressionKind::Inside { .. });
    }
}

/// What a body holds that the last passes look at: something that can fail, or deals with a failure, and a
/// `for_each`. A body with neither needs neither pass (D140).
struct Concerns<'p> {
    doors: &'p [DoorSignature],
    may_fail: bool,
    loops: bool,
}

impl Visitor for Concerns<'_> {
    fn statement(&mut self, statement: &Statement) {
        self.loops |= matches!(statement.kind, StatementKind::ForEach { .. });
    }

    fn expression(&mut self, expression: &Expression, _handled: bool) {
        self.may_fail |= match &expression.kind {
            ExpressionKind::DoorCall { door, .. } => {
                self.doors.get(*door as usize).is_none_or(|door| !door.failures.is_empty())
            }
            ExpressionKind::Call { .. }
            | ExpressionKind::Lookup { .. }
            | ExpressionKind::DictRemove { .. }
            | ExpressionKind::Convert { .. }
            | ExpressionKind::ListAdd { .. }
            | ExpressionKind::Inside { .. }
            | ExpressionKind::Try(_)
            | ExpressionKind::Handle { .. }
            | ExpressionKind::Otherwise { .. } => true,
            _ => false,
        };
    }
}

/// Looks at what a way into a part works out: its hidden names' values and its steps' places (D62).
fn walk_inside(path: &Inside, visitor: &mut dyn Visitor) {
    for (_, value) in &path.lets {
        walk_expression(value, false, visitor);
    }
    for step in &path.steps {
        if let PathStep::Item { index, .. } = step {
            walk_expression(index, false, visitor);
        }
    }
}

fn walk_block(block: &[Statement], visitor: &mut dyn Visitor) {
    for statement in block {
        visitor.statement(statement);
        match &statement.kind {
            StatementKind::Inside { path, change } => {
                walk_inside(path, visitor);
                walk_block(std::slice::from_ref(&**change), visitor);
            }
            StatementKind::Make { value, .. }
            | StatementKind::Set { value, .. }
            | StatementKind::SetField { value, .. }
            | StatementKind::Evaluate(value)
            | StatementKind::Use(value)
            | StatementKind::Return(Some(value)) => walk_expression(value, false, visitor),
            StatementKind::SetItem { index, value, .. } => {
                walk_expression(index, false, visitor);
                walk_expression(value, false, visitor);
            }
            StatementKind::SetKey { key, value, .. } => {
                walk_expression(key, false, visitor);
                walk_expression(value, false, visitor);
            }
            StatementKind::If { branches, otherwise } => {
                for (condition, body) in branches {
                    walk_expression(condition, false, visitor);
                    walk_block(body, visitor);
                }
                if let Some(body) = otherwise {
                    walk_block(body, visitor);
                }
            }
            StatementKind::While { condition, body } => {
                walk_expression(condition, false, visitor);
                walk_block(body, visitor);
            }
            StatementKind::ForRange { from, end, step, body, .. } => {
                walk_expression(from, false, visitor);
                walk_expression(end, false, visitor);
                if let Some(step) = step {
                    walk_expression(step, false, visitor);
                }
                walk_block(body, visitor);
            }
            StatementKind::ForEach { source, body, .. } => {
                walk_expression(source, false, visitor);
                walk_block(body, visitor);
            }
            StatementKind::Match { subject, arms } => {
                walk_expression(subject, false, visitor);
                for arm in arms {
                    walk_block(&arm.body, visitor);
                }
            }
            StatementKind::Return(None) | StatementKind::Break | StatementKind::Fail(_) => {}
        }
    }
}

fn walk_expression(expression: &Expression, handled: bool, visitor: &mut dyn Visitor) {
    visitor.expression(expression, handled);
    let mut plain = |child: &Expression| walk_expression(child, false, visitor);
    match &expression.kind {
        ExpressionKind::Error
        | ExpressionKind::Whole(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Text(_)
        | ExpressionKind::Number(_)
        | ExpressionKind::Variable(_) => {}
        ExpressionKind::Member { receiver: inner, .. }
        | ExpressionKind::Convert { value: inner, .. }
        | ExpressionKind::Sqrt(inner)
        | ExpressionKind::MaybeCrash(inner)
        | ExpressionKind::Land { value: inner, .. }
        | ExpressionKind::ShowPlaces { value: inner, .. }
        | ExpressionKind::Negate(inner)
        | ExpressionKind::Not(inner)
        | ExpressionKind::Repeat { value: inner, .. }
        | ExpressionKind::ListRemove { position: inner, .. }
        | ExpressionKind::DictRemove { key: inner, .. } => plain(inner),
        ExpressionKind::Item { receiver: first, index: second, .. }
        | ExpressionKind::Lookup { dict: first, key: second }
        | ExpressionKind::Has { dict: first, key: second }
        | ExpressionKind::Split { text: first, by: second, .. }
        | ExpressionKind::Binary { left: first, right: second, .. }
        | ExpressionKind::MaybeOr { maybe: first, fallback: second } => {
            plain(first);
            plain(second);
        }
        ExpressionKind::Call { arguments: items, .. }
        | ExpressionKind::Maths { arguments: items, .. }
        | ExpressionKind::DoorCall { arguments: items, .. }
        | ExpressionKind::Join(items)
        | ExpressionKind::List(items) => {
            for item in items {
                plain(item);
            }
        }
        ExpressionKind::BuildRecord { fields, .. } | ExpressionKind::BuildCase { fields, .. } => {
            for (_, value) in fields {
                plain(value);
            }
        }
        ExpressionKind::Dict(pairs) => {
            for (key, value) in pairs {
                plain(key);
                plain(value);
            }
        }
        ExpressionKind::Range { from, end, step, .. } => {
            plain(from);
            plain(end);
            if let Some(step) = step {
                plain(step);
            }
        }
        ExpressionKind::ListAdd { value, position, .. } => {
            plain(value);
            if let Some((position, _)) = position {
                plain(position);
            }
        }
        // The change itself was looked at with its way in: everything it needs is among the way's hidden names.
        ExpressionKind::Inside { path, .. } => walk_inside(path, visitor),
        ExpressionKind::Try(inner) => walk_expression(inner, true, visitor),
        ExpressionKind::Handle { call, fallback } => {
            walk_expression(call, true, visitor);
            if let HandleFallback::Value(value) = fallback {
                walk_expression(value, false, visitor);
            }
        }
        ExpressionKind::Otherwise { call, cases, .. } => {
            walk_expression(call, true, visitor);
            for (_, body) in cases {
                walk_block(body, visitor);
            }
        }
    }
}

/// What the finishing passes know about the script.
struct Facts<'p> {
    program: &'p Program,
    doors: &'p [DoorSignature],
    failure_ids: &'p HashMap<std::borrow::Cow<'static, str>, FailureId>,
}

impl Facts<'_> {
    fn known_failure(&self, name: &str) -> Option<FailureId> {
        super::find_failure(&self.program.failure_names, self.failure_ids, name)
    }

    /// The failures an expression can give, itself, before any handling.
    fn failures_of(&self, expression: &Expression) -> Vec<FailureId> {
        let named = |name: &str| self.known_failure(name).into_iter().collect::<Vec<_>>();
        match &expression.kind {
            // A change to a part fails as the change itself does (D62).
            ExpressionKind::Inside { change, .. } => self.failures_of(change),
            ExpressionKind::Call { procedure, .. } => {
                self.program.procedure(*procedure).map(|procedure| procedure.failures.clone()).unwrap_or_default()
            }
            ExpressionKind::DoorCall { door, .. } => self
                .doors
                .get(*door as usize)
                .map(|door| door.failures.iter().filter_map(|name| self.known_failure(name)).collect())
                .unwrap_or_default(),
            ExpressionKind::Lookup { .. } | ExpressionKind::DictRemove { .. } => named("missing"),
            ExpressionKind::Convert { conversion: Conversion::TextToWhole, .. } => named("not_a_whole"),
            ExpressionKind::Convert { conversion: Conversion::TextToBool, .. } => named("not_a_bool"),
            ExpressionKind::Convert { conversion: Conversion::ListToLimit(_), .. } => named("full"),
            ExpressionKind::Convert { conversion: Conversion::Number(conversion), .. } => {
                conversion.failure().map(named).unwrap_or_default()
            }
            ExpressionKind::ListAdd { variable, .. } => {
                let limited = self.program.variable(*variable).is_some_and(|variable| {
                    matches!(self.program.types.get(variable.type_id), Type::List { max: Some(_), .. })
                });
                if limited { named("full") } else { Vec::new() }
            }
            _ => Vec::new(),
        }
    }

    fn can_fail(&self, expression: &Expression) -> bool {
        match &expression.kind {
            ExpressionKind::Call { procedure, .. } => {
                self.program.procedure(*procedure).is_some_and(|procedure| procedure.can_fail)
            }
            _ => !self.failures_of(expression).is_empty(),
        }
    }

    fn pauses(&self, expression: &Expression) -> bool {
        match &expression.kind {
            ExpressionKind::Call { procedure, .. } => {
                self.program.procedure(*procedure).is_some_and(|procedure| procedure.can_pause)
            }
            ExpressionKind::DoorCall { door, .. } => self.doors.get(*door as usize).is_some_and(|door| door.can_pause),
            _ => false,
        }
    }

    fn names(&self, failures: &[FailureId]) -> String {
        failures
            .iter()
            .map(|&failure| format!("`{}`", self.program.failure_name(failure)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Collects the failures a body can give: every `fail`, and every failure passed up with `?` (D44).
struct FailureCollector<'p> {
    facts: &'p Facts<'p>,
    found: BTreeSet<FailureId>,
}

impl Visitor for FailureCollector<'_> {
    fn statement(&mut self, statement: &Statement) {
        if let StatementKind::Fail(failure) = statement.kind {
            self.found.insert(failure);
        }
    }

    fn expression(&mut self, expression: &Expression, _handled: bool) {
        if let ExpressionKind::Try(inner) = &expression.kind {
            self.found.extend(self.facts.failures_of(inner));
        }
    }
}

struct PauseFinder<'p> {
    facts: &'p Facts<'p>,
    found: Option<Place>,
}

impl Visitor for PauseFinder<'_> {
    fn expression(&mut self, expression: &Expression, _handled: bool) {
        if self.found.is_none() && self.facts.pauses(expression) {
            self.found = Some(expression.place);
        }
    }
}

/// Checks that every failure is dealt with, and that every `otherwise` covers its call's failures (Unio D23, D40).
struct HandlingChecker<'p> {
    facts: &'p Facts<'p>,
    messages: Vec<Message>,
}

impl Visitor for HandlingChecker<'_> {
    fn expression(&mut self, expression: &Expression, handled: bool) {
        if !handled && self.facts.can_fail(expression) {
            let failures = self.facts.failures_of(expression);
            // trace: Unio D23
            self.messages.push(
                Message::refused(
                    expression.place,
                    format!("this can fail with {}, and nothing deals with the failure", self.facts.names(&failures)),
                )
                .with_why("every call that can fail is dealt with, by `?`, `??` or `otherwise`."),
            );
        }
        if let ExpressionKind::Otherwise { call, cases, shared } = &expression.kind {
            let possible = self.facts.failures_of(call);
            let missing: Vec<FailureId> =
                possible.iter().copied().filter(|failure| !cases.iter().any(|(case, _)| case == failure)).collect();
            if !missing.is_empty() {
                // trace: Unio D40
                self.messages.push(
                    Message::refused(
                        expression.place,
                        format!("this `otherwise` has no `case` for {}", self.facts.names(&missing)),
                    )
                    .with_why("the cases cover every failure the call can give."),
                );
            }
            // trace: D131. A handler's case for a failure this call can't give is there for its other calls.
            let extra: Vec<FailureId> = cases
                .iter()
                .map(|(case, _)| *case)
                .filter(|case| !possible.contains(case) && !shared.contains(case))
                .collect();
            if !extra.is_empty() {
                self.messages.push(
                    Message::refused(
                        expression.place,
                        format!(
                            "this `otherwise` has a `case` for {}, which the call can't fail with",
                            self.facts.names(&extra)
                        ),
                    )
                    .with_why(format!("the call can fail with {}.", self.facts.names(&possible))),
                );
            }
        }
    }
}

/// Collects the top-level names a body changes itself, and the procedures it calls.
#[derive(Default)]
struct WriteCollector {
    writes: BTreeSet<VariableId>,
    calls: BTreeSet<ProcedureId>,
}

impl Visitor for WriteCollector {
    fn statement(&mut self, statement: &Statement) {
        if let StatementKind::Set { variable, .. }
        | StatementKind::SetField { variable, .. }
        | StatementKind::SetItem { variable, .. }
        | StatementKind::SetKey { variable, .. } = &statement.kind
        {
            self.writes.insert(*variable);
        }
        // A change to a part of a name changes the name (D62).
        if let StatementKind::Inside { path, .. } = &statement.kind {
            self.writes.insert(path.root);
        }
    }

    fn expression(&mut self, expression: &Expression, _handled: bool) {
        match &expression.kind {
            ExpressionKind::Inside { path, .. } => {
                self.writes.insert(path.root);
            }
            ExpressionKind::ListAdd { variable, .. }
            | ExpressionKind::ListRemove { variable, .. }
            | ExpressionKind::DictRemove { variable, .. } => {
                self.writes.insert(*variable);
            }
            ExpressionKind::Call { procedure, .. } => {
                self.calls.insert(*procedure);
            }
            _ => {}
        }
    }
}

/// Finds a `for_each` over a top-level name whose body could change that name from somewhere else: through a
/// procedure it calls, or through another task while it's paused (D51). Both are refused (D69).
struct LoopChecker<'p> {
    facts: &'p Facts<'p>,
    may_write: &'p [BTreeSet<VariableId>],
    messages: Vec<Message>,
}

struct LoopBodyChecker<'p> {
    facts: &'p Facts<'p>,
    may_write: &'p [BTreeSet<VariableId>],
    name: VariableId,
    /// Where the trouble is, and the procedure that can change the name, when it's a call rather than a pause.
    found: Option<(Place, Option<ProcedureId>)>,
}

impl Visitor for LoopBodyChecker<'_> {
    fn expression(&mut self, expression: &Expression, _handled: bool) {
        if self.found.is_some() {
            return;
        }
        if let ExpressionKind::Call { procedure, .. } = &expression.kind
            && self.may_write.get(procedure.0 as usize).is_some_and(|writes| writes.contains(&self.name))
        {
            self.found = Some((expression.place, Some(*procedure)));
        } else if self.facts.pauses(expression) {
            self.found = Some((expression.place, None));
        }
    }
}

impl Visitor for LoopChecker<'_> {
    fn statement(&mut self, statement: &Statement) {
        let StatementKind::ForEach { source, body, .. } = &statement.kind else { return };
        let Some(name) = Checker::root_variable(source) else { return };
        let Some(variable) = self.facts.program.variable(name) else { return };
        if !variable.top_level {
            return;
        }
        let mut body_checker = LoopBodyChecker { facts: self.facts, may_write: self.may_write, name, found: None };
        walk_block(body, &mut body_checker);
        // trace: D69
        let name = &variable.name;
        match body_checker.found {
            Some((place, Some(procedure))) => {
                let called = self.facts.program.procedure(procedure).map_or("this", |procedure| procedure.name.as_str());
                self.messages.push(
                    Message::refused(place, format!("`{called}` can change `{name}`, which this `for_each` is going through"))
                        .with_because(format!("line {} opens the `for_each`", statement.place.line), statement.place)
                        .with_why(format!(
                            "what a `for_each` goes through can't change inside it; copying `{name}` into a name of its own first lets the loop go through the copy."
                        )),
                );
            }
            Some((place, None)) => self.messages.push(
                Message::refused(place, format!("this can pause, while a `for_each` is going through the top-level name `{name}`"))
                    .with_because(format!("line {} opens the `for_each`", statement.place.line), statement.place)
                    .with_why(format!(
                        "another task could change `{name}` during the pause; copying it into a name of its own first lets the loop go through the copy."
                    )),
            ),
            None => {}
        }
    }
}

impl Checker<'_> {
    /// Says that a type must allow something: checked now when the type is known, or once it's worked out.
    pub(super) fn require(&mut self, subject: TypeId, requirement: Requirement, place: Place, purpose: &str) {
        let pending = Pending { subject, requirement, place, purpose: purpose.to_owned() };
        if self.program.types.is_open(subject) {
            self.pending.push(pending);
        } else {
            self.discharge(&pending);
        }
    }

    #[allow(clippy::too_many_lines)] // one case per requirement
    fn discharge(&mut self, pending: &Pending) {
        let kind = self.program.types.get(pending.subject).clone();
        let found = self.describe(pending.subject);
        let place = pending.place;
        if matches!(kind, Type::Error) {
            match &pending.requirement {
                Requirement::Member { result: open, .. }
                | Requirement::Indexable { item: open }
                | Requirement::Iterable { item: open } => self.program.types.settle_as_error(*open),
                _ => {}
            }
            return;
        }
        let purpose = pending.purpose.as_str();
        match &pending.requirement {
            Requirement::Number => {
                if !matches!(
                    kind,
                    Type::Whole | Type::Float | Type::Measured { .. } | Type::HugeWhole | Type::Decimal | Type::HugeDecimal
                ) {
                    self.refuse(place, format!("{purpose}, but this is {found}"));
                }
            }
            Requirement::Ordered => match kind {
                Type::Whole | Type::Float | Type::HugeWhole | Type::Decimal | Type::HugeDecimal => {}
                // trace: D96
                Type::Text => self.refuse_why(
                    place,
                    "two texts aren't put in order with `<`, `>`, `at_least` or `at_most`",
                    "text has no one right order: by character codes `\"Zebra\"` comes before `\"apple\"`, and alphabetical order differs between languages.",
                ),
                _ => self.refuse(place, format!("{purpose}, but this is {found}")),
            },
            // trace: D64, D95. Two values are equal when their contents are.
            Requirement::Comparable => {}
            Requirement::Textable => {
                if matches!(kind, Type::Text) {
                    self.refuse_why(
                        place,
                        "this is already a `Text`, so `be.Text` does nothing",
                        "converting a value to the type it already is, is refused.",
                    );
                }
            }
            Requirement::Member { name, result } => {
                let member = match &kind {
                    Type::Record(record) => self
                        .program
                        .types
                        .records
                        .get(*record as usize)
                        .and_then(|record| record.fields.iter().find(|(field, _)| field == name))
                        .map(|(_, field_type)| *field_type),
                    // trace: D65, D66. A text's `.len` counts graphemes; `.len.ucp` and `.len.bytes` are narrowings.
                    Type::List { .. } | Type::Array { .. } | Type::Dict { .. } | Type::Text if name == "len" => {
                        Some(WHOLE)
                    }
                    _ => None,
                };
                if let Some(member) = member {
                    self.program.types.unify(*result, member);
                } else {
                    let what = match kind {
                        Type::Record(_) => format!("{found} has no field `{name}`"),
                        Type::List { .. } | Type::Array { .. } | Type::Dict { .. } | Type::Text => {
                            format!("{found} has no fact `.{name}`")
                        }
                        _ => format!("{found} has no fields or facts, so there's no `.{name}`"),
                    };
                    self.refuse(place, what);
                    self.program.types.settle_as_error(*result);
                }
            }
            Requirement::Indexable { item } => match kind {
                Type::List { item: found_item, .. } | Type::Array { item: found_item, .. } => {
                    self.program.types.unify(*item, found_item);
                }
                // trace: D66. A text is reached by graphemes, each one a `Text`.
                Type::Text => {
                    self.program.types.unify(*item, TEXT);
                }
                Type::Dict { .. } => {
                    self.refuse(place, "a `Dict`'s value is reached with `key:`");
                    self.program.types.settle_as_error(*item);
                }
                _ => {
                    self.refuse(place, format!("{purpose}, but this is {found}"));
                    self.program.types.settle_as_error(*item);
                }
            },
            Requirement::Iterable { item } => match kind {
                Type::List { item: found_item, .. } | Type::Array { item: found_item, .. } => {
                    self.program.types.unify(*item, found_item);
                }
                Type::Dict { value, .. } => {
                    self.program.types.unify(*item, value);
                }
                // trace: D66. `for_each` goes through a text by graphemes.
                Type::Text => {
                    self.program.types.unify(*item, TEXT);
                }
                _ => {
                    self.refuse(place, format!("{purpose}, but this is {found}"));
                    self.program.types.settle_as_error(*item);
                }
            },
        }
    }

    /// Checks everything whose type has been worked out since it was asked for.
    fn solve_ready(&mut self) {
        loop {
            let mut progress = false;
            for pending in std::mem::take(&mut self.pending) {
                if self.program.types.is_open(pending.subject) {
                    self.pending.push(pending);
                } else {
                    self.discharge(&pending);
                    progress = true;
                }
            }
            for deferred in std::mem::take(&mut self.deferred) {
                if self.program.types.is_settled(deferred.parameter) {
                    if !self.program.types.unify(deferred.argument, deferred.parameter) {
                        let found = self.describe(deferred.argument);
                        let wanted = self.describe(deferred.parameter);
                        self.refuse(deferred.place, format!("this is {found}, but {} is {wanted}", deferred.purpose));
                    }
                    progress = true;
                } else {
                    self.deferred.push(deferred);
                }
            }
            if !progress {
                break;
            }
        }
    }

    /// The types that would satisfy a requirement, when they can be listed.
    fn candidates(&mut self, requirement: &Requirement) -> Option<Vec<TypeId>> {
        match requirement {
            Requirement::Number | Requirement::Ordered => Some(vec![WHOLE, FLOAT]),
            Requirement::Member { name, .. } => {
                if name == "len" {
                    return None;
                }
                let records: Vec<u32> = (0u32..)
                    .zip(&self.program.types.records)
                    .filter(|(_, record)| record.fields.iter().any(|(field, _)| field == name))
                    .map(|(index, _)| index)
                    .collect();
                Some(records.into_iter().map(|record| self.program.types.intern(Type::Record(record))).collect())
            }
            Requirement::Comparable
            | Requirement::Textable
            | Requirement::Indexable { .. }
            | Requirement::Iterable { .. } => None,
        }
    }

    /// The types each open type could still be, from everything asked of it.
    fn open_candidates(&mut self) -> HashMap<TypeId, Option<Vec<TypeId>>> {
        let mut by_type: HashMap<TypeId, Option<Vec<TypeId>>> = HashMap::new();
        for pending in self.pending.clone() {
            let root = self.program.types.resolve(pending.subject);
            let candidates = self.candidates(&pending.requirement);
            match (by_type.get_mut(&root), candidates) {
                (None, candidates) => {
                    by_type.insert(root, candidates);
                }
                (Some(slot), None) => *slot = None,
                (Some(Some(known)), Some(candidates)) => known.retain(|candidate| candidates.contains(candidate)),
                (Some(None), Some(_)) => {}
            }
        }
        by_type
    }

    /// Works types out as far as they go. When `last` is set and no procedure is still being checked, what the
    /// bodies leave open is decided where only one type fits, and refused otherwise (D5).
    #[allow(clippy::too_many_lines)] // deciding what's left open, then refusing the rest, in one pass
    pub(super) fn solve(&mut self, last: bool) {
        self.solve_ready();
        if !last || self.demand_depth > 0 {
            return;
        }
        loop {
            let single = self
                .open_candidates()
                .into_iter()
                .filter_map(|(open, candidates)| match candidates.as_deref() {
                    Some([only]) => Some((open, *only)),
                    _ => None,
                })
                .min_by_key(|(open, _)| open.0);
            let Some((open, only)) = single else { break };
            self.program.types.unify(open, only);
            self.solve_ready();
        }
        let candidates = self.open_candidates();
        let why = "a parameter's type is worked out from its own procedure's body; where the body can't decide, the \
                   type is written after the name, as in `name: Type`.";
        let mut batch = std::mem::take(&mut self.batch);
        for &id in &batch {
            let Some(procedure) = self.program.procedure(id) else { continue };
            // Usually every parameter and the result are settled (D81), and there's nothing to say.
            let settled = |types: &super::types::TypeTable, type_id| types.is_settled(type_id);
            if settled(&self.program.types, procedure.result)
                && procedure.parameters.iter().all(|&parameter| {
                    self.program
                        .variables
                        .get(parameter.0 as usize)
                        .is_none_or(|variable| settled(&self.program.types, variable.type_id))
                })
            {
                continue;
            }
            let name = procedure.name.clone();
            let parameters = procedure.parameters.clone();
            let result = procedure.result;
            let procedure_place = procedure.place;
            // Parameters left open together, as `a` and `b` in `a + b`, are refused together.
            let mut groups: Vec<(TypeId, Vec<String>, Place, bool)> = Vec::new();
            for parameter in parameters {
                let Some(variable) = self.variable(parameter) else { continue };
                if self.program.types.is_settled(variable.type_id) {
                    continue;
                }
                let root = self.program.types.resolve(variable.type_id);
                let quoted = format!("`{}`", variable.name);
                match groups.iter_mut().find(|(known, _, _, _)| *known == root) {
                    Some((_, names, _, read)) => {
                        names.push(quoted);
                        *read |= variable.read;
                    }
                    None => groups.push((root, vec![quoted], variable.place, variable.read)),
                }
            }
            for (type_id, mut names, place, read) in groups {
                let root = self.program.types.resolve(type_id);
                let last_name = names.pop().unwrap_or_default();
                let parameter_names =
                    if names.is_empty() { last_name } else { format!("{} and {last_name}", names.join(", ")) };
                let what = match candidates.get(&root) {
                    Some(Some(fitting)) if fitting.len() > 1 => {
                        let mut listed: Vec<String> =
                            fitting.iter().map(|&candidate| format!("a {}", self.describe(candidate))).collect();
                        let last_listed = listed.pop().unwrap_or_default();
                        // trace: D5: the message names the types that would fit, and suggests none.
                        format!(
                            "{parameter_names} could be {} or {last_listed}, and `{name}`'s body doesn't say which",
                            listed.join(", ")
                        )
                    }
                    Some(Some(fitting)) if fitting.is_empty() => {
                        format!("no type fits everything `{name}`'s body does with {parameter_names}")
                    }
                    Some(_) => format!("`{name}`'s body doesn't say enough to work out the type of {parameter_names}"),
                    None => format!("nothing in `{name}`'s body says what type {parameter_names} is"),
                };
                // A parameter that's never read is its own stop; it needs no second message.
                if read {
                    self.refuse_why(place, what, why);
                }
                self.program.types.settle_as_error(type_id);
            }
            if !self.program.types.is_settled(result) {
                if !self.has_refusals() {
                    self.refuse_why(
                        procedure_place,
                        format!("what `{name}` gives back can't be worked out from its body"),
                        "a procedure's result comes from its `ret`s; where they can't say, it's written with `-> Type`.",
                    );
                }
                self.program.types.settle_as_error(result);
            }
        }
        // The emptied list is kept for the next batch, unless one was started meanwhile.
        batch.clear();
        if self.batch.is_empty() {
            self.batch = batch;
        }
        self.solve_ready();
        for pending in std::mem::take(&mut self.pending) {
            if self.program.types.is_open(pending.subject) {
                if !self.has_refusals() {
                    self.refuse(pending.place, "this value's type can't be worked out");
                }
                self.program.types.settle_as_error(pending.subject);
            }
            self.discharge(&pending);
        }
        self.deferred.clear();
        // Once every procedure has been checked, anything still open can never be worked out.
        if !self.every_procedure_checked {
            return;
        }
        let open: Vec<(TypeId, Place, String)> = (0u32..)
            .zip(&self.program.variables)
            .filter(|(_, variable)| !self.program.types.is_settled(variable.type_id))
            .filter(|(index, variable)| !variable.top_level || self.made.contains(&VariableId(*index)))
            .map(|(_, variable)| (variable.type_id, variable.place, variable.name.clone()))
            .collect();
        for (type_id, place, name) in open {
            if self.messages.is_empty() {
                self.refuse(place, format!("`{name}`'s type can't be worked out"));
            }
            self.program.types.settle_as_error(type_id);
        }
    }

    /// The whole-script passes, once everything else is clean: failures (D44), their handling (Unio D23, D40),
    /// pauses (D52), and what isn't decided about loops over shared names.
    #[allow(clippy::too_many_lines)] // the whole-script passes, run one after another over the same facts
    pub(super) fn finish(&mut self) {
        for name in super::BUILT_IN_FAILURES {
            self.failure(name);
        }
        let door_failures: Vec<String> = self.doors.iter().flat_map(|door| door.failures.clone()).collect();
        for name in door_failures {
            self.failure(&name);
        }
        let count = self.program.procedures.len();

        // Failures and pauses: repeated until nothing changes, since procedures call each other. The first look also
        // collects which top-level names each procedure changes itself, and what it calls, which don't change. A body
        // that calls nothing is settled by the first look, and isn't looked at again.
        let mut may_write: Vec<BTreeSet<VariableId>> = Vec::with_capacity(count);
        let mut calls: Vec<BTreeSet<ProcedureId>> = Vec::with_capacity(count);
        let mut settled = vec![false; count];
        let mut first = true;
        let mut concerns = Concerns { doors: self.doors, may_fail: false, loops: false };
        loop {
            let mut changed = false;
            for index in 0..count {
                if settled.get(index) == Some(&true) {
                    continue;
                }
                let facts = Facts { program: &self.program, doors: self.doors, failure_ids: &self.failure_ids };
                let Some(procedure) = self.program.procedures.get(index) else { continue };
                let mut collector = FailureCollector { facts: &facts, found: BTreeSet::new() };
                let mut finder = PauseFinder { facts: &facts, found: None };
                let mut dependence = Dependence::default();
                if first {
                    let mut writes = WriteCollector::default();
                    let mut pauses_and_dependence = Both(&mut finder, &mut dependence);
                    let mut found = Both(&mut collector, &mut pauses_and_dependence);
                    let mut writes_and_concerns = Both(&mut writes, &mut concerns);
                    walk_block(&procedure.body, &mut Both(&mut found, &mut writes_and_concerns));
                    may_write.push(writes.writes);
                    calls.push(writes.calls);
                } else {
                    let mut pauses_and_dependence = Both(&mut finder, &mut dependence);
                    walk_block(&procedure.body, &mut Both(&mut collector, &mut pauses_and_dependence));
                }
                if !dependence.found
                    && let Some(settled) = settled.get_mut(index)
                {
                    *settled = true;
                }
                let failures: Vec<FailureId> = collector.found.into_iter().collect();
                let can_pause = finder.found.is_some();
                if failures != procedure.failures || can_pause != procedure.can_pause {
                    changed = true;
                    if let Some(procedure) = self.program.procedures.get_mut(index) {
                        procedure.failures = failures;
                        procedure.can_pause = can_pause;
                    }
                }
            }
            first = false;
            if !changed {
                break;
            }
        }

        // Which top-level names each procedure may change, itself or through what it calls.
        loop {
            let mut changed = false;
            for index in 0..count {
                let callees: Vec<ProcedureId> =
                    calls.get(index).map(|calls| calls.iter().copied().collect()).unwrap_or_default();
                for callee in callees {
                    let extra: Vec<VariableId> = may_write
                        .get(callee.0 as usize)
                        .map(|writes| writes.iter().copied().collect())
                        .unwrap_or_default();
                    if let Some(writes) = may_write.get_mut(index) {
                        for variable in extra {
                            changed |= writes.insert(variable);
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }

        let facts = Facts { program: &self.program, doors: self.doors, failure_ids: &self.failure_ids };
        let mut messages = Vec::new();
        let mut handling = HandlingChecker { facts: &facts, messages: Vec::new() };
        let mut loops = LoopChecker { facts: &facts, may_write: &may_write, messages: Vec::new() };
        // The procedures' bodies are looked at only for what one of them holds; the top level always is.
        let bodies = self.program.procedures.iter().map(|procedure| &procedure.body);
        for block in bodies.filter(|_| concerns.may_fail || concerns.loops).chain([&self.program.top_level]) {
            walk_block(block, &mut Both(&mut handling, &mut loops));
        }
        messages.append(&mut handling.messages);
        messages.append(&mut loops.messages);

        // trace: D84. A procedure that can fail writes `! [ … ]` in its header, matching what the body does exactly.
        for (procedure, written) in self.program.procedures.iter().zip(&self.written_failures) {
            // The header put right, only worked out when it's shown.
            let header = || {
                let list = procedure
                    .failures
                    .iter()
                    .map(|&failure| facts.program.failure_name(failure).to_owned())
                    .collect::<Vec<_>>()
                    .join(", ");
                super::header_with(&self.source_line(procedure.place.line), "", Some(&format!(" ! [{list}]")))
            };
            let Some(written) = written else {
                if !procedure.failures.is_empty() {
                    messages.push(
                        Message::refused(
                            procedure.place,
                            format!("`{}` can fail with {}, but its header doesn't say so", procedure.name, facts.names(&procedure.failures)),
                        )
                        .with_fix(header())
                        .with_why("a procedure that can fail lists its failures in its header, after `!`, so every caller knows what can come back."),
                    );
                }
                continue;
            };
            let mut written_ids: Vec<FailureId> =
                written.iter().filter_map(|word| facts.known_failure(word.text)).collect();
            written_ids.sort();
            written_ids.dedup();
            if written_ids != procedure.failures || written_ids.len() != written.len() {
                let place = written.first().map_or(procedure.place, |word| word.place);
                let body = if procedure.failures.is_empty() {
                    "its body can't fail".to_owned()
                } else {
                    format!("its body can fail with {}", facts.names(&procedure.failures))
                };
                let fix = if procedure.failures.is_empty() {
                    super::header_with(&self.source_line(procedure.place.line), "", Some(""))
                } else {
                    header()
                };
                messages.push(
                    Message::refused(place, format!("`{}`'s written failures don't match: {body}", procedure.name))
                        .with_fix(fix)
                        .with_why("the failures after `!` are exactly what the body can fail with: every `fail`, and every failure passed up with `?`."),
                );
            }
        }

        let mut finder = PauseFinder { facts: &facts, found: None };
        walk_block(&self.program.top_level, &mut finder);
        let top_level_pause = finder.found;
        if let Some(place) = top_level_pause {
            // trace: D67
            messages.push(
                Message::refused(place, "this pauses, and the top level can't pause")
                    .with_why("the host waits for loading to finish; a pause belongs in a procedure the host calls."),
            );
        }
        self.program.top_level_can_pause = top_level_pause.is_some();
        self.messages.append(&mut messages);
        let _ = ERROR;
    }
}
