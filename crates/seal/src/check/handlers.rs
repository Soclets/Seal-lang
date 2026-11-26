//! Handlers: named sets of `case`s, kept in handler groups, and used at a call as if their cases were written there
//! (D97, D98, D110, Unio D40 to D44).

use super::Checker;
use super::tree::{Block, Expression, ExpressionKind, FailureId, Statement, StatementKind, Variable, VariableKind};
use super::types::{ERROR, TypeId};
use crate::message::Message;
use crate::syntax::tree as syntax;

/// Why a handler is marked `*` where it's used (Unio D42, D129).
const JUMP_MARK: &str = "a handler that can `return`, `break` or `fail` leaves the code around the call, so it's marked `*` where it's used, as `?? ::*give_up`; one that can't takes no `*`.";

/// The built-ins that can fail, which `handles [ … ]` can name (D101, D129).
const FAILING_BUILT_INS: [&str; 6] =
    ["be.Whole", "be.Bool", "be.Float", "be.HugeWhole", "be.Decimal", "be.HugeDecimal"];

/// Whether a block can leave the code around it by `return` or `fail`, or by a `break` out of a loop around it
/// (D129). `loops` counts the loops entered inside it, whose `break`s stay inside.
fn can_jump(block: &[syntax::Statement], loops: u32) -> bool {
    block.iter().any(|statement| match &statement.kind {
        syntax::StatementKind::Return(_) | syntax::StatementKind::Fail(_) => true,
        syntax::StatementKind::Break => loops == 0,
        syntax::StatementKind::If { branches, otherwise } => {
            branches.iter().any(|(_, body)| can_jump(body, loops))
                || otherwise.as_ref().is_some_and(|body| can_jump(body, loops))
        }
        syntax::StatementKind::While { body, .. } | syntax::StatementKind::ForEach { body, .. } => {
            can_jump(body, loops + 1)
        }
        syntax::StatementKind::Match { cases, .. } => cases.iter().any(|case| can_jump(&case.body, loops)),
        syntax::StatementKind::Name { value, .. }
        | syntax::StatementKind::Expression(value)
        | syntax::StatementKind::Use(value)
        | syntax::StatementKind::Change { value, .. } => expression_can_jump(value, loops),
    })
}

/// Whether an `otherwise` block inside an expression can leave the code around it.
fn expression_can_jump(expression: &syntax::Expression, loops: u32) -> bool {
    match &expression.kind {
        syntax::ExpressionKind::Otherwise { call, cases, .. } => {
            expression_can_jump(call, loops) || cases.iter().any(|case| can_jump(&case.body, loops))
        }
        syntax::ExpressionKind::Handled { call, .. } | syntax::ExpressionKind::Try(call) => {
            expression_can_jump(call, loops)
        }
        _ => false,
    }
}

impl<'a> Checker<'a> {
    /// trace: D110. Each handler group, its members, and the procedures its `handles` line names.
    pub(super) fn declare_handler_groups(&mut self, script: &'a syntax::Script<'a>) {
        for item in &script.items {
            let syntax::Item::HandlerGroup(group) = item else { continue };
            let name = &group.name;
            if self.handler_groups.contains_key(name.text) {
                self.refuse_why(
                    name.place,
                    format!("the handler group `{}` is declared twice", name.text),
                    "each handler group is declared once.",
                );
                continue;
            }
            if !self.check_new_name(name) {
                continue;
            }
            if let Some(&global) = self.globals.get(name.text) {
                let made = self.variable(global).map_or(name.place, |variable| variable.place);
                self.messages.push(
                    Message::refused(
                        name.place,
                        format!(
                            "`{}` is already a top-level name, so a handler group can't be named `{}`",
                            name.text, name.text
                        ),
                    )
                    .with_because(format!("line {} makes it", made.line), made)
                    .with_why(super::NO_SHADOWING),
                );
                continue;
            }
            for (index, member) in group.members.iter().enumerate() {
                if group.members.iter().take(index).any(|earlier| earlier.name.text == member.name.text) {
                    self.refuse_why(
                        member.name.place,
                        format!("the handler `::{}` is listed twice in `{}`", member.name.text, name.text),
                        "each handler in a group has a name of its own.",
                    );
                }
            }
            for path in &group.handles {
                let full = path.iter().map(|word| word.text).collect::<Vec<_>>().join(".");
                let place = path.first().map_or(group.place, |word| word.place);
                // trace: D129. `handles` names any procedure that can fail: the script's, a door, or a built-in.
                let can_fail = match self.procedure_names.get(full.as_str()) {
                    Some(&procedure) => self.program.procedure(procedure).is_some_and(|procedure| procedure.can_fail),
                    None => match self.doors.iter().find(|door| door.name == full) {
                        Some(door) => !door.failures.is_empty(),
                        None if FAILING_BUILT_INS.contains(&full.as_str()) => true,
                        None => {
                            self.refuse(
                                place,
                                format!("there's no procedure or door named `{full}` for `{}` to handle", name.text),
                            );
                            continue;
                        }
                    },
                };
                if !can_fail {
                    self.refuse_why(
                        place,
                        format!("`{full}` can't fail, so `{}` has nothing to handle for it", name.text),
                        "`handles [ … ]` names the procedures, doors and built-ins that can fail, whose failures the group's handlers deal with.",
                    );
                }
            }
            self.handler_groups.insert(name.text, group);
        }
    }

    /// A handler's cases, checked where it's used as if written at the call, each after lines that make its
    /// parameters from its arguments (D98, D110, Unio D40, D42, D44).
    #[allow(clippy::too_many_lines)] // finding the handler, its mark, its arguments and its cases, in order
    pub(super) fn handler_cases(
        &mut self,
        call: &Expression,
        using: &syntax::HandlerUse<'a>,
        type_id: TypeId,
        gives_nothing: bool,
    ) -> Option<Vec<(FailureId, Block)>> {
        let called = match &call.kind {
            ExpressionKind::Call { procedure, .. } => {
                self.program.procedure(*procedure).map(|found| found.name.clone())
            }
            ExpressionKind::DoorCall { door, .. } => self.doors.get(*door as usize).map(|door| door.name.clone()),
            ExpressionKind::Convert { conversion: super::tree::Conversion::TextToWhole, .. } => {
                Some("be.Whole".to_owned())
            }
            ExpressionKind::Convert { conversion: super::tree::Conversion::TextToBool, .. } => {
                Some("be.Bool".to_owned())
            }
            ExpressionKind::Convert { conversion: super::tree::Conversion::Number(number), .. } => {
                let target = match number.to {
                    super::tree::NumberType::Whole => "Whole",
                    super::tree::NumberType::Float => "Float",
                    super::tree::NumberType::Text => "Text",
                    super::tree::NumberType::Huge => "HugeWhole",
                    super::tree::NumberType::Decimal => "Decimal",
                    super::tree::NumberType::HugeDecimal => "HugeDecimal",
                };
                Some(format!("be.{target}"))
            }
            _ => None,
        };
        let member = using.name.text;
        let group: &'a syntax::HandlerGroup<'a> = if let Some(word) = &using.group {
            let Some(&group) = self.handler_groups.get(word.text) else {
                self.refuse(word.place, format!("there's no handler group named `{}`", word.text));
                return None;
            };
            group
        } else {
            // trace: D110. A short `::name` reaches into the groups that handle the procedure just called.
            let mut handling: Vec<&'a syntax::HandlerGroup<'a>> = self
                .handler_groups
                .values()
                .copied()
                .filter(|group| {
                    called.as_ref().is_some_and(|called| {
                        group
                            .handles
                            .iter()
                            .any(|path| path.iter().map(|word| word.text).collect::<Vec<_>>().join(".") == *called)
                    })
                })
                .collect();
            handling.sort_by(|first, second| first.name.text.cmp(second.name.text));
            let with_member: Vec<&'a syntax::HandlerGroup<'a>> = handling
                .iter()
                .copied()
                .filter(|group| group.members.iter().any(|known| known.name.text == member))
                .collect();
            match with_member.as_slice() {
                [only] => only,
                [] => {
                    let what = match (&called, handling.is_empty()) {
                        (Some(called), true) => {
                            format!("no handler group handles `{called}`, so `::{member}` reaches nothing")
                        }
                        (Some(called), false) => format!("no handler group that handles `{called}` has `::{member}`"),
                        (None, _) => format!("`::{member}` follows something no handler group can name in `handles`"),
                    };
                    self.refuse_why(
                        using.place,
                        what,
                        "a short `::name` reaches into the groups whose `handles [ … ]` line names the procedure just called; `group::name` works anywhere.",
                    );
                    return None;
                }
                several => {
                    let names: Vec<String> =
                        several.iter().map(|group| format!("`{}::{member}`", group.name.text)).collect();
                    self.refuse_why(
                        using.place,
                        format!("`::{member}` could be {}", names.join(" or ")),
                        "when two groups that handle the call both have the handler, the group is written, as `group::name`.",
                    );
                    return None;
                }
            }
        };
        let Some(handler) = group.members.iter().find(|known| known.name.text == member) else {
            let known: Vec<String> = group.members.iter().map(|known| format!("`::{}`", known.name.text)).collect();
            self.refuse_why(
                using.name.place,
                format!("`{}` has no handler `::{member}`", group.name.text),
                &format!("`{}`'s handlers are {}.", group.name.text, known.join(", ")),
            );
            return None;
        };
        let jumps = handler.cases.iter().any(|case| can_jump(&case.body, 0));
        if jumps != using.jumps {
            // trace: Unio D42
            let what = if jumps {
                format!("`::{member}` can `return`, `break` or `fail`, so it's marked `::*{member}` where it's used")
            } else {
                format!("`::{member}` can't `return`, `break` or `fail`, so it takes no `*`")
            };
            self.refuse_why(using.place, what, JUMP_MARK);
            return None;
        }
        let key = format!("{}::{member}", group.name.text);
        if self.handlers_in_use.contains(&key) {
            self.not_designed(using.place, "a handler used inside its own cases");
            return None;
        }
        // trace: Unio D44. Its parameters, written as a procedure's are, and the arguments given where it's used.
        let parameter_types: Vec<TypeId> = handler
            .parameters
            .iter()
            .map(|parameter| parameter.written_type.as_ref().map_or(ERROR, |written| self.written_type(written)))
            .collect();
        let mut labels: Vec<Option<String>> = vec![None; handler.parameters.len()];
        for declaration in &handler.labels {
            match handler.parameters.iter().position(|parameter| parameter.name.text == declaration.parameter.text) {
                Some(position) => {
                    if let Some(slot) = labels.get_mut(position) {
                        *slot = Some(declaration.label.text.to_owned());
                    }
                }
                None => self.refuse(
                    declaration.parameter.place,
                    format!("`{}` isn't one of `::{member}`'s parameters", declaration.parameter.text),
                ),
            }
        }
        let mut defaults: Vec<Option<Expression>> = vec![None; handler.parameters.len()];
        for declaration in &handler.defaults {
            let Some(position) = labels.iter().position(|label| label.as_deref() == Some(declaration.label.text))
            else {
                self.refuse(
                    declaration.label.place,
                    format!("`{}` isn't one of `::{member}`'s labels", declaration.label.text),
                );
                continue;
            };
            let wanted = parameter_types.get(position).copied().unwrap_or(ERROR);
            if let Some(value) = self.plain_value(&declaration.value, wanted)
                && let Some(slot) = defaults.get_mut(position)
            {
                *slot = Some(value);
            }
        }
        let has_default: Vec<bool> = defaults.iter().map(Option::is_some).collect();
        let bound = self.bind_arguments(
            &format!("::{member}"),
            (labels.as_slice(), handler.labels_any_order),
            &has_default,
            &using.arguments,
            using.place,
        )?;
        let mut values: Vec<Expression> = Vec::new();
        for (position, argument) in bound.iter().enumerate() {
            let wanted = parameter_types.get(position).copied().unwrap_or(ERROR);
            match argument.and_then(|index| using.arguments.get(index)) {
                Some(entry) => {
                    let value = self.expression(&entry.value, Some(wanted));
                    self.expect(&value, wanted, &format!("`::{member}`'s argument"));
                    values.push(self.land(value, wanted));
                }
                None => {
                    values.push(defaults.get(position).cloned().flatten().unwrap_or_else(|| Self::error(using.place)));
                }
            }
        }
        for parameter in &handler.parameters {
            // trace: D71
            self.check_new_name(&parameter.name);
            if self.globals.contains_key(parameter.name.text) {
                self.refuse_top_level_spelling(&parameter.name, "a handler's parameter");
            }
        }
        // The cases see the handler's own parameters, and the top level's names as a procedure's body does.
        self.handlers_in_use.push(key);
        let saved = std::mem::take(&mut self.context.scopes);
        let around: Vec<super::names::Names<'a, super::tree::VariableId>> =
            if self.context.procedure.is_none() { saved.first().cloned().into_iter().collect() } else { Vec::new() };
        let mut read = vec![false; handler.parameters.len()];
        let mut cases: Vec<(FailureId, Block)> = Vec::new();
        for case in &handler.cases {
            let failure = self.failure(case.failure.text);
            if cases.iter().any(|(earlier, _)| *earlier == failure) {
                self.refuse_why(
                    case.failure.place,
                    format!("the failure `{}` is handled twice", case.failure.text),
                    "a handler has one `case` for each failure.",
                );
            }
            let mut scope = super::names::Names::default();
            let mut before = Vec::new();
            let mut made = Vec::new();
            for ((parameter, &parameter_type), value) in handler.parameters.iter().zip(&parameter_types).zip(&values) {
                let id = self.new_variable(Variable {
                    name: parameter.name.text.to_owned(),
                    type_id: parameter_type,
                    kind: VariableKind::Named,
                    mutable: false,
                    changed: false,
                    marked_unused: parameter.unused,
                    read: false,
                    top_level: false,
                    captured: false,
                    made_at: 0,
                    place: parameter.name.place,
                });
                scope.insert(parameter.name.text, id);
                before.push(Statement {
                    kind: StatementKind::Make { variable: id, value: value.clone() },
                    place: using.place,
                });
                made.push(id);
            }
            self.context.scopes.clone_from(&around);
            self.context.scopes.push(scope);
            let block = self.case_block(case, type_id, gives_nothing, true, before);
            self.context.scopes.clear();
            for (slot, id) in read.iter_mut().zip(made) {
                *slot |= self.variable(id).is_some_and(|variable| variable.read);
            }
            cases.push((failure, block));
        }
        self.context.scopes = saved;
        self.handlers_in_use.pop();
        for (parameter, was_read) in handler.parameters.iter().zip(read) {
            if !was_read && !parameter.unused {
                // trace: D61
                self.refuse_why(
                    parameter.name.place,
                    format!("`{}` is never used", parameter.name.text),
                    &format!(
                        "a handler's parameter that's never read on purpose is marked, as `unused {}`.",
                        parameter.name.text
                    ),
                );
            }
        }
        Some(cases)
    }
}
