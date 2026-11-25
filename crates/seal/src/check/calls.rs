//! Checking calls: the script's procedures, the host's doors, the built-ins, and methods on values.

use super::expressions::{is_name_path, name_path};
use super::tree::{Conversion, Expression, ExpressionKind, NumberType, ProcedureId, Rounding};
use super::types::{BOOL, ERROR, FLOAT, NOTHING, TEXT, Type, TypeId, WHOLE};
use super::{Checker, DeferredArgument, Requirement};
use crate::message::Place;
use crate::syntax::tree as syntax;

/// How many calls are nested here, each the first argument of the next (Unio D128).
/// What a call's arguments are matched against: each value's label, when it has one (D41, D51, D87).
pub(super) trait Labels {
    fn count(&self) -> usize;
    fn label(&self, index: usize) -> Option<&str>;
}

impl Labels for [Option<String>] {
    fn count(&self) -> usize {
        self.len()
    }

    fn label(&self, index: usize) -> Option<&str> {
        self.get(index).and_then(Option::as_deref)
    }
}

/// A procedure's parameters when none has a label: only how many there are.
struct Unlabelled(usize);

impl Labels for Unlabelled {
    fn count(&self) -> usize {
        self.0
    }

    fn label(&self, _index: usize) -> Option<&str> {
        None
    }
}

impl Labels for [Option<&str>] {
    fn count(&self) -> usize {
        self.len()
    }

    fn label(&self, index: usize) -> Option<&str> {
        self.get(index).copied().flatten()
    }
}

impl Labels for [crate::signature::DoorParameter] {
    fn count(&self) -> usize {
        self.len()
    }

    fn label(&self, index: usize) -> Option<&str> {
        self.get(index).and_then(|parameter| parameter.label.as_deref())
    }
}

/// Which argument each parameter gets, held in place for the usual few, so matching a call allocates nothing.
#[derive(Clone, Debug)]
pub(super) enum Bound {
    Few([Option<usize>; 8], usize),
    Many(Vec<Option<usize>>),
}

impl Bound {
    fn empty(count: usize) -> Bound {
        if count <= 8 { Bound::Few([None; 8], count) } else { Bound::Many(vec![None; count]) }
    }
}

impl std::ops::Deref for Bound {
    type Target = [Option<usize>];

    fn deref(&self) -> &[Option<usize>] {
        match self {
            Bound::Few(slots, count) => slots.get(..*count).unwrap_or(&[]),
            Bound::Many(slots) => slots,
        }
    }
}

impl std::ops::DerefMut for Bound {
    fn deref_mut(&mut self) -> &mut [Option<usize>] {
        match self {
            Bound::Few(slots, count) => slots.get_mut(..*count).unwrap_or(&mut []),
            Bound::Many(slots) => slots,
        }
    }
}

fn chain_length(expression: &syntax::Expression) -> usize {
    match expression.kind.as_call() {
        Some((callee, arguments)) if is_name_path(callee) => {
            // Brackets around the inner call don't make the chain any shorter.
            let inner = arguments.first().filter(|entry| entry.label.is_none()).map_or(0, |entry| {
                let mut value = &entry.value;
                while let syntax::ExpressionKind::Group(grouped) = &value.kind {
                    value = grouped;
                }
                chain_length(value)
            });
            1 + inner
        }
        _ => 0,
    }
}

impl<'a> Checker<'a> {
    pub(super) fn call(
        &mut self,
        expression: &syntax::Expression<'a>,
        callee: &syntax::Expression<'a>,
        arguments: &[syntax::Entry<'a>],
        wanted: Option<TypeId>,
    ) -> Expression {
        let place = expression.place;
        if let syntax::ExpressionKind::Member { receiver, name } = &callee.kind
            && self.is_value(receiver)
        {
            return self.method(receiver, name, arguments, place);
        }
        let Some(path) = name_path(callee) else {
            self.refuse_why(callee.place, "this isn't a procedure, so it can't be called", "`( )` calls a procedure.");
            return Self::error(place);
        };
        // trace: Unio D128: three or more calls in a chain are written as a slide.
        if self.slide_depth == 0 && !self.judged_chains.contains(&expression.id) {
            let length = chain_length(expression);
            if length >= 3 && path.first().is_none_or(|word| self.visible(word).is_none()) {
                let mut inner = expression;
                while let Some((_, arguments)) = inner.kind.as_call() {
                    self.judged_chains.insert(inner.id);
                    let Some(first) = arguments.first() else { break };
                    inner = &first.value;
                }
                self.refuse_why(
                    place,
                    format!("this is a chain of {length} calls, each inside the next, and three or more are written as a slide"),
                    "a slide reads in the order it runs, as in `points |> double |> be.Text |> show_msg`.",
                );
            }
        }
        let word = path.first().copied().unwrap_or_default();
        if path.len() == 1 && self.visible(word).is_some() {
            self.refuse_why(
                callee.place,
                format!("`{word}` is a name, not a procedure, so it can't be called"),
                "`( )` calls a procedure.",
            );
            return Self::error(place);
        }
        match word {
            "sqrt" => {
                let Some(value) = self.single_argument("sqrt", &path, 1, arguments, place) else {
                    return Self::error(place);
                };
                let checked = self.expression(value, None);
                // trace: D127. A `Decimal`'s root is correctly rounded, a `HugeDecimal`'s exact or a stop.
                if self.takes_decimal(checked.type_id, false) {
                    let type_id = checked.type_id;
                    return Expression { kind: ExpressionKind::Sqrt(Box::new(checked)), type_id, place };
                }
                if !self.program.types.unify(checked.type_id, FLOAT) {
                    // trace: Unio D185
                    let found = self.describe(checked.type_id);
                    self.refuse_why(
                        value.place,
                        format!("`sqrt` takes a `Float`, but this is {found}"),
                        "`sqrt` gives a `Float`'s square root; a number written with a point, as `9.0`, is a `Float`.",
                    );
                }
                Expression { kind: ExpressionKind::Sqrt(Box::new(checked)), type_id: FLOAT, place }
            }
            "be" => self.conversion(&path, arguments, place),
            "abs" | "min" | "max" | "clamp" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan2" | "lerp" => {
                self.maths_call(&path, arguments, place)
            }
            "range" => {
                // trace: Unio D165, D170
                self.refuse_why(
                    place,
                    "a `range` is written with `[ ]`, as in `range[from: 1, to: 3]`",
                    "`( )` calls a procedure, and a range runs nothing.",
                );
                Self::error(place)
            }
            _ => {
                // A one-word name is itself; only a narrowed one is joined up.
                let full: std::borrow::Cow<'_, str> = match &*path {
                    [only] => std::borrow::Cow::Borrowed(only),
                    _ => std::borrow::Cow::Owned(path.join(".")),
                };
                if let Some(&procedure) = self.procedure_names.get(full.as_ref()) {
                    return self.procedure_call(procedure, &full, arguments, place);
                }
                if let Some(door) = self.doors.iter().position(|door| door.name == *full) {
                    return self.door_call(u32::try_from(door).unwrap_or(u32::MAX), arguments, wanted, place);
                }
                for entry in arguments {
                    self.expression(&entry.value, None);
                }
                if self.words.contains(word) {
                    self.refuse_unknown_narrowing(&path, callee.place);
                } else {
                    self.refuse(callee.place, format!("there's no procedure or door named `{word}`"));
                }
                Self::error(place)
            }
        }
    }

    /// The maths built-ins (D107, D109, D113, D114): `abs`, `min`, `max` and `clamp` on two `Whole`s or two `Float`s,
    /// never mixed, with units or without; `sin`, `cos` and `tan` on an angle in `rads`, `degs` or `turns`, or a
    /// plain `Float` in radians; `asin`, `acos` and `atan2` giving `rads`; `lerp` on two values of one unit group.
    #[allow(clippy::too_many_lines)] // one case per kind of built-in, each with its units
    fn maths_call(&mut self, path: &[&str], arguments: &[syntax::Entry<'a>], place: Place) -> Expression {
        use super::tree::MathsFunction as Function;
        let word = path.first().copied().unwrap_or_default();
        let (function, labels): (Function, &[Option<&str>]) = match word {
            "abs" => (Function::Abs, &[None]),
            "min" => (Function::Min, &[None, None]),
            "max" => (Function::Max, &[None, None]),
            "clamp" => (Function::Clamp, &[None, Some("low"), Some("high")]),
            "sin" => (Function::Sin, &[None]),
            "cos" => (Function::Cos, &[None]),
            "tan" => (Function::Tan, &[None]),
            "asin" => (Function::Asin, &[None]),
            "acos" => (Function::Acos, &[None]),
            "atan2" => (Function::Atan2, &[Some("y"), Some("x")]),
            _ => (Function::Lerp, &[None, Some("to"), Some("by")]),
        };
        if let Some(narrowing) = path.get(1) {
            // trace: D113. The `.degrees` versions went; an angle's unit says it.
            self.refuse_why(
                place,
                format!("`{word}` has no narrowing `.{narrowing}`"),
                &format!("`{word}` takes an angle in `rads`, `degs` or `turns`, as `{word}(90 degs)`, or a plain `Float` in radians."),
            );
            return Self::error(place);
        }
        let Some(bound) = self.bind_arguments(word, (labels, false), &[], arguments, place) else {
            for entry in arguments {
                self.expression(&entry.value, None);
            }
            return Self::error(place);
        };
        let entries: Vec<&syntax::Entry> =
            bound.iter().filter_map(|argument| argument.and_then(|index| arguments.get(index))).collect();
        let refuse_float = |checker: &mut Self, value: &Expression| {
            let found = checker.describe(value.type_id);
            checker.refuse_why(
                value.place,
                format!("`{word}` takes a `Float`, but this is {found}"),
                "angles and `lerp` work on `Float`s; a number written with a point, as `2.0`, is a `Float`.",
            );
            Self::error(place)
        };
        let mut unit = 0u8;
        let (checked, type_id): (Vec<Expression>, TypeId) = match function {
            Function::Abs | Function::Min | Function::Max | Function::Clamp => {
                let Some(first) = entries.first() else { return Self::error(place) };
                let first = self.expression(&first.value, None);
                let first_type = first.type_id;
                let mut checked = vec![first];
                for entry in entries.iter().skip(1) {
                    // The others land in the first's type, its unit included (D120).
                    let value = self.expression(&entry.value, Some(first_type));
                    if !self.program.types.unify(value.type_id, first_type)
                        && !matches!(value.kind, ExpressionKind::Error)
                    {
                        let (left, right) = (self.describe(first_type), self.describe(value.type_id));
                        // trace: D107, Unio D162. The two never mix.
                        self.refuse_why(
                            value.place,
                            format!("`{word}` takes values of one type, but this is {right} and the first is {left}"),
                            "`Whole` and `Float` don't mix quietly: `be.Float(…)` turns a `Whole` into a `Float`.",
                        );
                        return Self::error(place);
                    }
                    checked.push(value);
                }
                self.require(
                    first_type,
                    Requirement::Number,
                    place,
                    &format!("`{word}` works on a `Whole` or a `Float`"),
                );
                (checked, first_type)
            }
            Function::Sin | Function::Cos | Function::Tan => {
                let Some(entry) = entries.first() else { return Self::error(place) };
                let value = self.expression(&entry.value, None);
                // trace: D113. An angle in `rads`, `degs` or `turns`, or a plain `Float` in radians.
                let value = match self.angle_unit(value.type_id) {
                    Some(Ok(angle)) => {
                        unit = angle;
                        let base = self.program.types.measured(value.type_id).map_or(FLOAT, |(base, _)| base);
                        if base == WHOLE {
                            let place = value.place;
                            Expression {
                                kind: ExpressionKind::Convert {
                                    conversion: Conversion::WholeToFloat,
                                    value: Box::new(value),
                                },
                                type_id: FLOAT,
                                place,
                            }
                        } else {
                            value
                        }
                    }
                    Some(Err(())) => {
                        let found = self.describe(value.type_id);
                        self.refuse_why(
                            value.place,
                            format!("`{word}` takes an angle, but this is {found}"),
                            "an angle is in `rads`, `degs` or `turns`, or a plain `Float` in radians.",
                        );
                        return Self::error(place);
                    }
                    None => {
                        if !self.program.types.unify(value.type_id, FLOAT) {
                            return refuse_float(self, &value);
                        }
                        value
                    }
                };
                (vec![value], FLOAT)
            }
            Function::Asin | Function::Acos => {
                let Some(entry) = entries.first() else { return Self::error(place) };
                let value = self.expression(&entry.value, Some(FLOAT));
                if !self.program.types.unify(value.type_id, FLOAT) {
                    return refuse_float(self, &value);
                }
                // trace: D134. They take a plain ratio, so they give a plain `Float`, in radians.
                (vec![value], FLOAT)
            }
            Function::Atan2 | Function::Lerp => {
                let (Some(first), Some(second)) = (entries.first(), entries.get(1)) else { return Self::error(place) };
                let first = self.expression(&first.value, None);
                // The second lands in the first's type, unit included, so the two are of one group (D114).
                let second = self.expression(&second.value, Some(first.type_id));
                let plain = self
                    .program
                    .types
                    .measured(first.type_id)
                    .map_or(self.program.types.resolve(first.type_id), |(base, _)| base);
                let decimal = function == Function::Lerp && self.takes_decimal(first.type_id, true);
                if plain != FLOAT && !decimal && !self.program.types.is_open(plain) {
                    return refuse_float(self, &first);
                }
                if !self.program.types.unify(second.type_id, first.type_id)
                    && !matches!(second.kind, ExpressionKind::Error)
                {
                    let (left, right) = (self.describe(first.type_id), self.describe(second.type_id));
                    self.refuse_why(
                        second.place,
                        format!("`{word}` takes values of one type, but this is {right} and the first is {left}"),
                        "the two are `Float`s of one unit group, or both plain.",
                    );
                    return Self::error(place);
                }
                let mut checked = vec![first.clone(), second];
                if function == Function::Lerp {
                    let Some(by) = entries.get(2) else { return Self::error(place) };
                    // trace: D127. A `Decimal`'s `lerp` goes by a `Decimal`.
                    let by_type = if self.takes_decimal(first.type_id, true) { first.type_id } else { FLOAT };
                    let by = self.expression(&by.value, Some(by_type));
                    if !self.program.types.unify(by.type_id, by_type) {
                        return refuse_float(self, &by);
                    }
                    checked.push(by);
                    (checked, first.type_id)
                } else if self.program.types.measured(first.type_id).is_some() {
                    // trace: D134. Two values with units give `rads`; two plain ones a plain `Float`.
                    (checked, self.radians())
                } else {
                    (checked, FLOAT)
                }
            }
        };
        // trace: D107. A `clamp` whose edges are written the wrong way round has no answer anyone decided.
        if function == Function::Clamp
            && let [_, low, high] = checked.as_slice()
        {
            let crossed = match (&low.kind, &high.kind) {
                (ExpressionKind::Whole(low), ExpressionKind::Whole(high)) => low > high,
                (ExpressionKind::Float(low), ExpressionKind::Float(high)) => low > high,
                _ => false,
            };
            if crossed {
                self.not_designed(place, "a `clamp` whose `low:` is more than its `high:`");
                return Self::error(place);
            }
        }
        Expression { kind: ExpressionKind::Maths { function, arguments: checked, unit }, type_id, place }
    }

    /// An unknown narrowing is refused, with the word's real narrowings listed (D13, Unio D102).
    fn refuse_unknown_narrowing(&mut self, path: &[&str], place: Place) {
        let word = path.first().copied().unwrap_or_default();
        let prefix = format!("{word}.");
        let mut real: Vec<String> = self
            .procedure_names
            .keys()
            .map(AsRef::as_ref)
            .chain(self.doors.iter().map(|door| door.name.as_str()))
            .filter(|name| name.starts_with(&prefix))
            .map(|name| format!("`{name}`"))
            .collect();
        real.sort();
        real.dedup();
        let full = path.join(".");
        let narrowing = path.get(1..).unwrap_or(&[]).join(".");
        let has_plain = self.procedure_names.contains_key(word) || self.doors.iter().any(|door| door.name == word);
        if path.len() == 1 && !has_plain {
            self.refuse_why(
                place,
                format!("`{word}` is only ever one of its kinds"),
                &format!("`{word}`'s narrowings are {}.", real.join(", ")),
            );
        } else if real.is_empty() {
            self.refuse_why(
                place,
                format!("`{word}` has no narrowing `.{narrowing}`"),
                &format!("`{word}` has no narrowings."),
            );
        } else {
            self.refuse_why(
                place,
                format!("`{word}` has no narrowing `.{narrowing}`, so there's no `{full}`"),
                &format!("`{word}`'s narrowings are {}.", real.join(", ")),
            );
        }
    }

    /// The one unlabelled argument a built-in takes.
    pub(super) fn single_argument<'e>(
        &mut self,
        name: &str,
        path: &[&str],
        path_length: usize,
        arguments: &'e [syntax::Entry<'a>],
        place: Place,
    ) -> Option<&'e syntax::Expression<'a>> {
        if path.len() != path_length {
            let narrowing = path.get(path_length..).unwrap_or(&[]).join(".");
            self.refuse_why(
                place,
                format!("`{name}` has no narrowing `.{narrowing}`"),
                &format!("`{name}` can't be narrowed further."),
            );
            return None;
        }
        match arguments {
            [entry] if entry.label.is_none() => Some(&entry.value),
            _ => {
                self.refuse(place, format!("`{name}` takes one value, with no label"));
                None
            }
        }
    }

    /// Matches a call's arguments to parameters by position and label (D41, D42, Unio D24). Gives, for each
    /// parameter, which argument fills it, or nothing where a default does.
    pub(super) fn bind_arguments<L: Labels + ?Sized>(
        &mut self,
        name: &str,
        (labels, any_order): (&L, bool),
        has_default: &[bool],
        arguments: &[syntax::Entry<'a>],
        place: Place,
    ) -> Option<Bound> {
        let mut out_of_order = false;
        let mut bound = Bound::empty(labels.count());
        let mut last: Option<usize> = None;
        let mut failed = false;
        for (index, entry) in arguments.iter().enumerate() {
            let parameter = if let Some(label) = &entry.label {
                let found = (0..labels.count()).position(|index| labels.label(index) == Some(label.text));
                if found.is_none() {
                    self.refuse_why(
                        label.place,
                        format!("`{name}` has no label `{}:`", label.text),
                        "a call writes the labels its procedure declares, and no others.",
                    );
                    failed = true;
                }
                found
            } else {
                let start = last.map_or(0, |last| last + 1);
                let found = (start..labels.count()).find(|&candidate| {
                    labels.label(candidate).is_none() && bound.get(candidate).is_some_and(Option::is_none)
                });
                if found.is_none() {
                    let next_label = (0..labels.count())
                        .find(|&candidate| bound.get(candidate).is_some_and(Option::is_none))
                        .and_then(|candidate| labels.label(candidate));
                    match next_label {
                        Some(label) => self.refuse_why(
                            entry.value.place,
                            format!("this argument needs its label, `{label}:`"),
                            "a call writes every label its procedure declares.",
                        ),
                        None => {
                            self.refuse(entry.value.place, format!("`{name}` doesn't take this many arguments"));
                        }
                    }
                    failed = true;
                }
                found
            };
            let Some(parameter) = parameter else { continue };
            if bound.get(parameter).is_some_and(Option::is_some) {
                self.refuse(entry.value.place, "this argument is given twice");
                failed = true;
                continue;
            }
            if last.is_some_and(|last| parameter < last) && !any_order {
                out_of_order = true;
            }
            if let Some(slot) = bound.get_mut(parameter) {
                *slot = Some(index);
            }
            last = Some(parameter);
        }
        if failed {
            return None;
        }
        if out_of_order {
            // trace: D87. A call writes its arguments in the order the procedure declares them.
            self.refuse_order(name, &bound, arguments, place);
            return None;
        }
        let missing = (0..labels.count()).find(|&parameter| {
            bound.get(parameter).is_some_and(Option::is_none) && !has_default.get(parameter).copied().unwrap_or(false)
        });
        if let Some(first) = missing {
            match labels.label(first) {
                Some(label) => self.refuse_why(
                    place,
                    format!("this call leaves out `{label}:`"),
                    "a call writes every label its procedure declares, unless the label has a default.",
                ),
                None => self.refuse(
                    place,
                    format!(
                        "`{name}` takes {} argument{}, but this call gives {}",
                        labels.count(),
                        if labels.count() == 1 { "" } else { "s" },
                        arguments.len()
                    ),
                ),
            }
            return None;
        }
        Some(bound)
    }

    /// trace: D87. Arguments in another order than declared are refused, with the call put in order as the fix when
    /// it's on one line.
    fn refuse_order(&mut self, name: &str, bound: &[Option<usize>], arguments: &[syntax::Entry<'a>], place: Place) {
        let start = |entry: &syntax::Entry| entry.label.as_ref().map_or(entry.value.place, |label| label.place);
        let mut message = crate::message::Message::refused(
            place,
            format!("this call writes `{name}`'s arguments in another order than it declares them"),
        )
        .with_why("a call writes its arguments in the order its procedure declares them, so every call to it reads alike; `labels.any_order` lets the labelled ones go in any order.");
        let one_line =
            arguments.iter().all(|entry| start(entry).line == place.line && entry.value.place.line == place.line);
        if one_line && let (Some(first), Some(last)) = (arguments.first(), arguments.last()) {
            let line: Vec<char> = self.source_line(place.line).chars().collect();
            let text = |from: Place, to: Place| -> String {
                let (from, to) =
                    (from.column.saturating_sub(1) as usize, (to.column + to.width).saturating_sub(1) as usize);
                line.get(from..to.min(line.len())).unwrap_or(&[]).iter().collect()
            };
            let ordered: Vec<String> = bound
                .iter()
                .filter_map(|argument| argument.and_then(|index| arguments.get(index)))
                .map(|entry| text(start(entry), entry.value.place))
                .collect();
            let (from, to) = (start(first), last.value.place);
            let before: String = line.get(..from.column.saturating_sub(1) as usize).unwrap_or(&[]).iter().collect();
            let after: String = line
                .get(((to.column + to.width).saturating_sub(1) as usize).min(line.len())..)
                .unwrap_or(&[])
                .iter()
                .collect();
            message = message.with_fix(format!("{before}{}{after}", ordered.join(", ")));
        }
        self.messages.push(message);
    }

    fn procedure_call(
        &mut self,
        id: ProcedureId,
        name: &str,
        arguments: &[syntax::Entry<'a>],
        place: Place,
    ) -> Expression {
        self.ensure_checked(id);
        let Some(procedure) = self.program.procedure(id) else { return Self::error(place) };
        let labels = procedure.labels.clone();
        let any_order = procedure.labels_any_order;
        let defaults = procedure.defaults.clone();
        let parameters = procedure.parameters.clone();
        let result = procedure.result;
        let has_default: Vec<bool> = defaults.iter().map(Option::is_some).collect();
        let bound = if labels.is_empty() {
            self.bind_arguments(name, (&Unlabelled(parameters.len()), any_order), &has_default, arguments, place)
        } else {
            self.bind_arguments(name, (labels.as_slice(), any_order), &has_default, arguments, place)
        };
        let Some(bound) = bound else {
            for entry in arguments {
                self.expression(&entry.value, None);
            }
            return Expression { kind: ExpressionKind::Error, type_id: result, place };
        };
        let mut checked = Vec::new();
        for (position, argument) in bound.iter().enumerate() {
            let parameter_type = parameters.get(position).map_or(ERROR, |&parameter| self.variable_type(parameter));
            match argument.and_then(|index| arguments.get(index)) {
                Some(entry) => {
                    let settled = self.program.types.is_settled(parameter_type);
                    let value = self.expression(&entry.value, settled.then_some(parameter_type));
                    // trace: D128. An argument Seal can see past a ranged parameter's hard edge is refused while
                    // loading; any other is checked as the procedure starts, where the host's arguments land too.
                    if self.program.types.limits(parameter_type).is_some()
                        && matches!(self.land(value.clone(), parameter_type).kind, ExpressionKind::Error)
                    {
                        checked.push(Self::error(place));
                        continue;
                    }
                    let purpose = format!("`{name}`'s argument");
                    if settled {
                        self.expect(&value, parameter_type, &purpose);
                    } else {
                        // The parameter's type comes from its own procedure's body, never from this call (D5), so the
                        // argument is checked once that body has decided.
                        self.deferred.push(DeferredArgument {
                            argument: value.type_id,
                            parameter: parameter_type,
                            place: value.place,
                            purpose,
                        });
                    }
                    checked.push(value);
                }
                None => match defaults.get(position).cloned().flatten() {
                    Some(default) => checked.push(default),
                    None => checked.push(Self::error(place)),
                },
            }
        }
        Expression { kind: ExpressionKind::Call { procedure: id, arguments: checked }, type_id: result, place }
    }

    fn door_call(
        &mut self,
        door: u32,
        arguments: &[syntax::Entry<'a>],
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let _ = wanted;
        // The door's own description lives as long as the checking, so nothing of it is copied here.
        let doors = self.doors;
        let Some(signature) = doors.get(door as usize) else { return Self::error(place) };
        let name = signature.name.as_str();
        // A door's types are worked out where they're used: a plain one is a constant, so nothing is kept (D140).
        let result = self.host_type(&signature.result);
        // A door's values have no defaults (D51).
        let Some(bound) = self.bind_arguments(name, (signature.parameters.as_slice(), false), &[], arguments, place)
        else {
            for entry in arguments {
                self.expression(&entry.value, None);
            }
            return Expression { kind: ExpressionKind::Error, type_id: result, place };
        };
        let mut checked = Vec::with_capacity(bound.len());
        for (position, argument) in bound.iter().enumerate() {
            let Some(entry) = argument.and_then(|index| arguments.get(index)) else { continue };
            let parameter_type =
                signature.parameters.get(position).map_or(ERROR, |parameter| self.host_type(&parameter.host_type));
            let value = self.expression(&entry.value, Some(parameter_type));
            // A list with `len.max` and an array both cross a door as a Rust `Vec` (the standard, chapter 6).
            let crossing =
                match (self.program.types.get(value.type_id).clone(), self.program.types.get(parameter_type).clone()) {
                    (
                        Type::List { item, max: Some(_) } | Type::Array { item, .. },
                        Type::List { item: wanted_item, max: None },
                    ) => self.program.types.unify(item, wanted_item),
                    _ => false,
                };
            if !crossing && !self.program.types.unify(value.type_id, parameter_type) {
                self.expect(&value, parameter_type, &format!("`{name}`'s argument"));
            }
            checked.push(value);
        }
        Expression { kind: ExpressionKind::DoorCall { door, arguments: checked }, type_id: result, place }
    }

    /// `be.Text`, `be.Whole` and its narrowings, and `be.Float` (D22, Unio D22, D153, D178).
    #[allow(clippy::too_many_lines)] // one case per conversion, each with its own refusals
    fn conversion(&mut self, path: &[&str], arguments: &[syntax::Entry<'a>], place: Place) -> Expression {
        let narrowing: Vec<&str> = path.iter().skip(1).copied().collect();
        let full = path.join(".");
        let rounding = match narrowing.as_slice() {
            ["Text" | "Whole" | "Float" | "Bool"] => None,
            ["Whole", "floor"] => Some(Rounding::Floor),
            ["Whole", "trunc"] => Some(Rounding::Trunc),
            ["Whole", "ceil"] => Some(Rounding::Ceil),
            ["Whole", "nearest"] => Some(Rounding::Nearest),
            ["Whole", "nearest", "even"] => Some(Rounding::NearestEven),
            [] => {
                // trace: Unio D22
                self.refuse_why(
                    place,
                    "`be` is narrowed by the type the value becomes, as in `be.Whole`",
                    "`be.Type(value)` converts a value to a type.",
                );
                return Self::error(place);
            }
            [target, ..] if ["HugeWhole", "Decimal", "HugeDecimal"].contains(target) => {
                return self.convert_to_number(path, arguments, place);
            }
            [target, ..]
                if target.starts_with(|first: char| first.is_ascii_uppercase())
                    && !["Text", "Whole", "Float", "Bool"].contains(target) =>
            {
                // trace: D101. Only the conversions decided are in Seal.
                self.refuse_why(
                    place,
                    format!("there's no conversion to `{target}`"),
                    "`be` converts to `Text`, `Whole`, `Float` and `Bool`; other values are built as they're written.",
                );
                return Self::error(place);
            }
            _ => {
                self.refuse_why(
                    place,
                    format!("there's no `{full}`"),
                    "`be`'s narrowings are `be.Text`, `be.Float`, `be.Whole`, `be.Bool`, `be.Whole.floor`, \
                     `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.",
                );
                return Self::error(place);
            }
        };
        let Some(argument) = self.single_argument(&full, path, path.len(), arguments, place) else {
            return Self::error(place);
        };
        let value = self.expression(argument, None);
        if let Some(from) = self.big_number(value.type_id) {
            let to = match narrowing.first().copied() {
                Some("Text") => NumberType::Text,
                Some("Float") => NumberType::Float,
                Some("Bool") => {
                    let found = self.describe(value.type_id);
                    self.refuse_why(
                        argument.place,
                        format!("there's no conversion from {found} to a `Bool`"),
                        "a number is compared, as `n != 0`, for a `Bool`.",
                    );
                    return Self::error(place);
                }
                _ => NumberType::Whole,
            };
            return self.convert_from_number(value, from, &full, to, rounding, place);
        }
        let found = self.program.types.resolve(value.type_id);
        let same_type = |checker: &mut Self, target: &str| {
            // trace: Unio D22
            checker.refuse_why(
                argument.place,
                format!("this is already a `{target}`, so `be.{target}` does nothing"),
                "converting a value to the type it already is, is refused.",
            );
            Self::error(place)
        };
        let (conversion, type_id) = match (narrowing.first().copied(), rounding) {
            (Some("Text"), _) => {
                if found == TEXT {
                    return same_type(self, "Text");
                }
                self.require(
                    value.type_id,
                    Requirement::Textable,
                    argument.place,
                    "`be.Text` turns a `Whole`, a `Float` or a `Bool` into text",
                );
                (Conversion::ToText, TEXT)
            }
            (Some("Bool"), _) => {
                if found == BOOL {
                    return same_type(self, "Bool");
                }
                if !self.program.types.is_open(found) && found != TEXT && found != ERROR {
                    // trace: D101
                    let written = self.describe(found);
                    self.refuse_why(
                        argument.place,
                        format!("there's no conversion from {written} to a `Bool`"),
                        "`be.Bool` reads `\"true\"` or `\"false\"` from text; a number is compared, as `n != 0`, for a `Bool`.",
                    );
                    return Self::error(place);
                }
                self.program.types.unify(value.type_id, TEXT);
                (Conversion::TextToBool, BOOL)
            }
            (Some("Float"), _) => {
                if found == FLOAT {
                    return same_type(self, "Float");
                }
                if !self.program.types.is_open(found) && found != WHOLE && found != ERROR {
                    // trace: D101
                    let written = self.describe(found);
                    self.refuse_why(
                        argument.place,
                        format!("there's no conversion from {written} to a `Float`"),
                        "`be.Float` turns a `Whole` into a `Float`.",
                    );
                    return Self::error(place);
                }
                self.program.types.unify(value.type_id, WHOLE);
                (Conversion::WholeToFloat, FLOAT)
            }
            (_, Some(rounding)) => {
                if !self.program.types.unify(value.type_id, FLOAT) {
                    let written = self.describe(found);
                    self.refuse_why(
                        argument.place,
                        format!("`{full}` takes a `Float`, but this is {written}"),
                        "`.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` say how a `Float`'s fraction goes.",
                    );
                    return Self::error(place);
                }
                (Conversion::FloatToWhole(rounding), WHOLE)
            }
            _ => {
                if found == WHOLE {
                    return same_type(self, "Whole");
                }
                if found == FLOAT {
                    // trace: Unio D178
                    self.refuse_why(
                        place,
                        "`be.Whole` on a `Float` says how the fraction goes",
                        "the ways are `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.",
                    );
                    return Self::error(place);
                }
                if found == BOOL || !(self.program.types.is_open(found) || found == TEXT || found == ERROR) {
                    // trace: D101
                    let written = self.describe(found);
                    self.refuse_why(
                        argument.place,
                        format!("there's no conversion from {written} to a `Whole`"),
                        "`be.Whole` reads a `Whole` from text, and turns a `Float` into one with `.floor`, `.trunc`, `.ceil`, `.nearest` or `.nearest.even`.",
                    );
                    return Self::error(place);
                }
                self.program.types.unify(value.type_id, TEXT);
                (Conversion::TextToWhole, WHOLE)
            }
        };
        Expression { kind: ExpressionKind::Convert { conversion, value: Box::new(value) }, type_id, place }
    }

    /// A method on a value: `.add` and `.remove` on a list, `.remove` and `.has` on a dictionary
    /// (D34, Unio D77, D80).
    #[allow(clippy::too_many_lines)] // each method's arguments are read here, one after another
    fn method(
        &mut self,
        receiver: &syntax::Expression<'a>,
        method: &syntax::Word<'a>,
        arguments: &[syntax::Entry<'a>],
        place: Place,
    ) -> Expression {
        let label_of = |entry: &syntax::Entry| entry.label.as_ref().map(|label| label.text.to_owned());
        if method.text == "has" {
            let dict = self.expression(receiver, None);
            let key_type = self.program.types.fresh();
            let value_type = self.program.types.fresh();
            let wanted = self.program.types.dict(key_type, value_type);
            if !self.program.types.unify(dict.type_id, wanted) {
                let found = self.describe(dict.type_id);
                self.refuse(method.place, format!("{found} has no method `has`"));
                return Self::error(place);
            }
            let [entry] = arguments else {
                self.refuse(place, "`.has` takes `key:` and its value");
                return Self::error(place);
            };
            if label_of(entry).as_deref() != Some("key") {
                self.refuse(entry.value.place, "`.has` takes `key:` and its value");
                return Self::error(place);
            }
            let key = self.expression(&entry.value, Some(key_type));
            self.expect(&key, key_type, "the dictionary's key");
            return Expression {
                kind: ExpressionKind::Has { dict: Box::new(dict), key: Box::new(key) },
                type_id: BOOL,
                place,
            };
        }
        // trace: D65. `text.split(by: t)` keeps empty pieces; `text.split.no_empty(by: t)` drops them.
        let split_of = match &receiver.kind {
            syntax::ExpressionKind::Member { receiver: text, name }
                if name.text == "split" && method.text == "no_empty" =>
            {
                Some((&**text, false))
            }
            _ if method.text == "split" => Some((receiver, true)),
            _ => None,
        };
        if let Some((text, keep_empty)) = split_of {
            let name = if keep_empty { ".split" } else { ".split.no_empty" };
            let text = self.expression(text, Some(TEXT));
            if !self.program.types.unify(text.type_id, TEXT) {
                let found = self.describe(text.type_id);
                self.refuse(method.place, format!("`{name}` splits a text, but this is {found}"));
                return Self::error(place);
            }
            let [entry] = arguments else {
                self.refuse(place, format!("`{name}` takes `by:` and the text to split at"));
                return Self::error(place);
            };
            if label_of(entry).as_deref() != Some("by") {
                self.refuse(entry.value.place, format!("`{name}` takes `by:` and the text to split at"));
                return Self::error(place);
            }
            let by = self.expression(&entry.value, Some(TEXT));
            self.expect(&by, TEXT, "the text to split at");
            // trace: D89. An empty separator the checker can see is refused while loading.
            let empty = match &by.kind {
                ExpressionKind::Text(text) => text.is_empty(),
                ExpressionKind::Variable(variable) => self.variable(*variable).is_some_and(|variable| {
                    variable.top_level
                        && self.plain_names.get(variable.name.as_str()).is_some_and(
                            |value| matches!(&value.kind, syntax::ExpressionKind::Text(text) if text.is_empty()),
                        )
                }),
                _ => false,
            };
            if empty {
                self.refuse_why(
                    entry.value.place,
                    "this splits a text by an empty text",
                    "an empty text is never found between two pieces; `for_each` goes through a text's characters, as `for_each letter in name`.",
                );
                return Self::error(place);
            }
            let type_id = self.program.types.list(TEXT);
            return Expression {
                kind: ExpressionKind::Split { text: Box::new(text), by: Box::new(by), keep_empty },
                type_id,
                place,
            };
        }
        if !matches!(method.text, "add" | "remove") {
            let checked = self.expression(receiver, None);
            if !self.program.types.is_error(checked.type_id) {
                let found = self.describe(checked.type_id);
                self.refuse(method.place, format!("{found} has no method `{}`", method.text));
            }
            return Self::error(place);
        }
        // `.add` and `.remove` change their value, so they're called on a `mut` name (Unio D77).
        // trace: D62. The name's own value changes, or a part of it reached through fields and places.
        let (variable, changeable, mut way) = if let syntax::ExpressionKind::Name(name) = &receiver.kind {
            let Some(variable) = self.lookup(name, receiver.place) else { return Self::error(place) };
            (variable, self.check_changeable(variable, name, receiver.place), None)
        } else {
            let Some((way, _, changeable)) = self.way_in(receiver) else { return Self::error(place) };
            (way.part, changeable, Some(way))
        };
        // trace: D5. With its type not worked out yet, the method says what the value must be: only a list has
        // `.add` and a `.remove` at a place, and only a `Dict` a `.remove` by key.
        if matches!(self.program.types.get(self.variable_type(variable)), Type::Variable(_)) {
            let by_key = arguments.iter().any(|entry| label_of(entry).as_deref() == Some("key"));
            let shape = if method.text == "remove" && by_key {
                let (key, value) = (self.program.types.fresh(), self.program.types.fresh());
                self.program.types.dict(key, value)
            } else {
                let item = self.program.types.fresh();
                self.program.types.list(item)
            };
            let open = self.variable_type(variable);
            self.program.types.unify(open, shape);
        }
        let variable_type = self.program.types.resolve(self.variable_type(variable));
        let result = match (self.program.types.get(variable_type).clone(), method.text) {
            (Type::List { item, .. }, "add") => {
                let mut value = None;
                let mut position = None;
                for entry in arguments {
                    match label_of(entry).as_deref() {
                        None if value.is_none() => {
                            let checked = self.expression(&entry.value, Some(item));
                            self.expect(&checked, item, "the list's item");
                            value = Some(self.land(checked, item));
                        }
                        Some(label @ ("pos" | "offset")) if position.is_none() && value.is_some() => {
                            let checked = self.expression(&entry.value, Some(WHOLE));
                            self.expect(&checked, WHOLE, &format!("`{label}:`"));
                            position = Some((Box::new(checked), label == "offset"));
                        }
                        _ => {
                            self.refuse(
                                entry.value.place,
                                "`.add` takes the item, then `pos:` or `offset:` when it goes at a place",
                            );
                            return Self::error(place);
                        }
                    }
                }
                let Some(value) = value else {
                    self.refuse(place, "`.add` takes the item to add");
                    return Self::error(place);
                };
                ExpressionKind::ListAdd { variable, value: Box::new(value), position }
            }
            (Type::List { .. }, _) => {
                let [entry] = arguments else {
                    self.refuse(place, "`.remove` takes `pos:` or `offset:` and its number");
                    return Self::error(place);
                };
                let label = match label_of(entry).as_deref() {
                    Some("pos") => "pos",
                    Some("offset") => "offset",
                    _ => {
                        self.refuse(entry.value.place, "`.remove` takes `pos:` or `offset:` and its number");
                        return Self::error(place);
                    }
                };
                let checked = self.expression(&entry.value, Some(WHOLE));
                self.expect(&checked, WHOLE, &format!("`{label}:`"));
                ExpressionKind::ListRemove { variable, position: Box::new(checked), zero_based: label == "offset" }
            }
            (Type::Dict { key, .. }, "remove") => {
                let [entry] = arguments else {
                    self.refuse(place, "`.remove` takes `key:` and its value");
                    return Self::error(place);
                };
                if label_of(entry).as_deref() != Some("key") {
                    self.refuse(entry.value.place, "`.remove` takes `key:` and its value");
                    return Self::error(place);
                }
                let checked = self.expression(&entry.value, Some(key));
                self.expect(&checked, key, "the dictionary's key");
                ExpressionKind::DictRemove { variable, key: Box::new(checked) }
            }
            (Type::Array { .. }, _) => {
                // trace: Unio D167
                self.refuse_why(
                    method.place,
                    format!("an `Array` has no `.{}`", method.text),
                    "an array holds exactly its length, so nothing is added or taken away; its items can still change.",
                );
                return Self::error(place);
            }
            (Type::Variable(_), _) => {
                self.not_built(
                    place,
                    format!("working out a type from a later line (D5), which this `.{}` needs, isn't", method.text),
                );
                return Self::error(place);
            }
            (Type::Error, _) => return Self::error(place),
            _ => {
                let found = self.describe(variable_type);
                self.refuse(method.place, format!("{found} has no method `{}`", method.text));
                return Self::error(place);
            }
        };
        if !changeable {
            return Self::error(place);
        }
        let Some(mut way) = way.take() else { return Expression { kind: result, type_id: NOTHING, place } };
        // What the change needs is worked out before the way in is taken.
        let change = match result {
            ExpressionKind::ListAdd { variable, value, position } => {
                let position =
                    position.map(|(index, zero_based)| (Box::new(self.hoist(*index, &mut way.lets)), zero_based));
                ExpressionKind::ListAdd { variable, value: Box::new(self.hoist(*value, &mut way.lets)), position }
            }
            ExpressionKind::ListRemove { variable, position, zero_based } => ExpressionKind::ListRemove {
                variable,
                position: Box::new(self.hoist(*position, &mut way.lets)),
                zero_based,
            },
            ExpressionKind::DictRemove { variable, key } => {
                ExpressionKind::DictRemove { variable, key: Box::new(self.hoist(*key, &mut way.lets)) }
            }
            other => other,
        };
        Expression {
            kind: ExpressionKind::Inside {
                path: Box::new(way),
                change: Box::new(Expression { kind: change, type_id: NOTHING, place }),
            },
            type_id: NOTHING,
            place,
        }
    }
}
