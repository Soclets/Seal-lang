//! Checking calls: the script's procedures, the host's doors, the built-ins, and methods on values.

use super::expressions::name_path;
use super::tree::{Conversion, Expression, ExpressionKind, ProcedureId, Rounding};
use super::types::{BOOL, ERROR, FLOAT, NOTHING, TEXT, Type, TypeId, WHOLE};
use super::{Checker, DeferredArgument, Requirement};
use crate::message::Place;
use crate::syntax::tree as syntax;

/// How many calls are nested here, each the first argument of the next (Unio D128).
fn chain_length(expression: &syntax::Expression) -> usize {
    match &expression.kind {
        syntax::ExpressionKind::Call { callee, arguments } if name_path(callee).is_some() => {
            let inner =
                arguments.first().filter(|entry| entry.label.is_none()).map_or(0, |entry| chain_length(&entry.value));
            1 + inner
        }
        _ => 0,
    }
}

impl Checker<'_> {
    pub(super) fn call(
        &mut self,
        expression: &syntax::Expression,
        callee: &syntax::Expression,
        arguments: &[syntax::Entry],
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
                while let syntax::ExpressionKind::Call { arguments, .. } = &inner.kind {
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
        let word = path.first().map(String::as_str).unwrap_or_default();
        if path.len() == 1 && self.visible(word).is_some() {
            self.refuse_why(
                callee.place,
                format!("`{word}` is a name, not a procedure, so it can't be called"),
                "`( )` calls a procedure.",
            );
            return Self::error(place);
        }
        match word {
            "fmt" => self.fmt(&path, arguments, place),
            "sqrt" => {
                let Some(value) = self.single_argument("sqrt", &path, 1, arguments, place) else {
                    return Self::error(place);
                };
                let checked = self.expression(value, Some(FLOAT));
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
                let full = path.join(".");
                if let Some(&procedure) = self.procedure_names.get(&full) {
                    return self.procedure_call(procedure, &full, arguments, place);
                }
                if let Some(door) = self.doors.iter().position(|door| door.name == full) {
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

    /// An unknown narrowing is refused, with the word's real narrowings listed (D13, Unio D102).
    fn refuse_unknown_narrowing(&mut self, path: &[String], place: Place) {
        let word = path.first().map(String::as_str).unwrap_or_default();
        let prefix = format!("{word}.");
        let mut real: Vec<String> = self
            .procedure_names
            .keys()
            .map(String::as_str)
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
    fn single_argument<'e>(
        &mut self,
        name: &str,
        path: &[String],
        path_length: usize,
        arguments: &'e [syntax::Entry],
        place: Place,
    ) -> Option<&'e syntax::Expression> {
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
    fn bind_arguments(
        &mut self,
        name: &str,
        labels: &[Option<String>],
        has_default: &[bool],
        arguments: &[syntax::Entry],
        place: Place,
    ) -> Option<Vec<Option<usize>>> {
        let mut bound: Vec<Option<usize>> = vec![None; labels.len()];
        let mut last: Option<usize> = None;
        let mut failed = false;
        for (index, entry) in arguments.iter().enumerate() {
            let parameter = if let Some(label) = &entry.label {
                let found = labels.iter().position(|known| known.as_deref() == Some(label.text.as_str()));
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
                let found = (start..labels.len()).find(|&candidate| {
                    labels.get(candidate).is_some_and(Option::is_none)
                        && bound.get(candidate).is_some_and(Option::is_none)
                });
                if found.is_none() {
                    let next_label = (0..labels.len())
                        .find(|&candidate| bound.get(candidate).is_some_and(Option::is_none))
                        .and_then(|candidate| labels.get(candidate).cloned().flatten());
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
            if last.is_some_and(|last| parameter < last) {
                self.not_designed(entry.value.place, "arguments written in another order than the parameters");
                failed = true;
            }
            if let Some(slot) = bound.get_mut(parameter) {
                *slot = Some(index);
            }
            last = Some(parameter);
        }
        if failed {
            return None;
        }
        let missing: Vec<usize> = (0..labels.len())
            .filter(|&parameter| {
                bound.get(parameter).is_some_and(Option::is_none)
                    && !has_default.get(parameter).copied().unwrap_or(false)
            })
            .collect();
        if let Some(&first) = missing.first() {
            match labels.get(first).cloned().flatten() {
                Some(label) => self.refuse_why(
                    place,
                    format!("this call leaves out `{label}:`"),
                    "a call writes every label its procedure declares, unless the label has a default.",
                ),
                None => self.refuse(
                    place,
                    format!("`{name}` takes {} arguments, but this call gives {}", labels.len(), arguments.len()),
                ),
            }
            return None;
        }
        Some(bound)
    }

    fn procedure_call(&mut self, id: ProcedureId, name: &str, arguments: &[syntax::Entry], place: Place) -> Expression {
        self.ensure_checked(id);
        let Some(procedure) = self.program.procedure(id) else { return Self::error(place) };
        let labels = procedure.labels.clone();
        let defaults = procedure.defaults.clone();
        let parameters = procedure.parameters.clone();
        let result = procedure.result;
        let has_default: Vec<bool> = defaults.iter().map(Option::is_some).collect();
        let Some(bound) = self.bind_arguments(name, &labels, &has_default, arguments, place) else {
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
        arguments: &[syntax::Entry],
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let _ = wanted;
        let Some(signature) = self.doors.get(door as usize) else { return Self::error(place) };
        let name = signature.name.clone();
        let labels: Vec<Option<String>> =
            signature.parameters.iter().map(|parameter| parameter.label.clone()).collect();
        let (parameter_types, result) = if let Some(types) = self.door_types.get(&door) {
            types.clone()
        } else {
            let before = self.messages.len();
            let parameters: Vec<TypeId> =
                signature.parameters.iter().map(|parameter| self.host_type(&parameter.host_type, place)).collect();
            let result = self.host_type(&signature.result, place);
            if self.messages.len() == before {
                self.door_types.insert(door, (parameters.clone(), result));
            }
            (parameters, result)
        };
        let has_default = vec![false; labels.len()];
        let Some(bound) = self.bind_arguments(&name, &labels, &has_default, arguments, place) else {
            for entry in arguments {
                self.expression(&entry.value, None);
            }
            return Expression { kind: ExpressionKind::Error, type_id: result, place };
        };
        let mut checked = Vec::new();
        for (position, argument) in bound.iter().enumerate() {
            let parameter_type = parameter_types.get(position).copied().unwrap_or(ERROR);
            let Some(entry) = argument.and_then(|index| arguments.get(index)) else { continue };
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
            if !crossing {
                self.expect(&value, parameter_type, &format!("`{name}`'s argument"));
            }
            checked.push(value);
        }
        Expression { kind: ExpressionKind::DoorCall { door, arguments: checked }, type_id: result, place }
    }

    /// `fmt("…")` fills in the names in its braces and gives a `Text` (D12, D28).
    fn fmt(&mut self, path: &[String], arguments: &[syntax::Entry], place: Place) -> Expression {
        let Some(argument) = self.single_argument("fmt", path, 1, arguments, place) else { return Self::error(place) };
        let syntax::ExpressionKind::Text(text) = &argument.kind else {
            self.refuse_why(
                argument.place,
                "`fmt` takes its text written out in quotes",
                "the names in `fmt`'s braces are checked when the script loads, so the text has to be there to read.",
            );
            self.expression(argument, None);
            return Self::error(place);
        };
        let characters: Vec<char> = text.chars().collect();
        let column_of = |index: usize| argument.place.column + 1 + u32::try_from(index).unwrap_or(0);
        let mut parts: Vec<Expression> = Vec::new();
        let mut plain = String::new();
        let mut index = 0;
        while let Some(&character) = characters.get(index) {
            match character {
                '{' => {
                    let Some(length) =
                        characters.get(index..).and_then(|rest| rest.iter().position(|&next| next == '}'))
                    else {
                        self.not_designed(
                            Place::new(argument.place.line, column_of(index), 1),
                            "a `{` with no `}` after it in `fmt` text",
                        );
                        return Self::error(place);
                    };
                    let inside: String = characters.get(index + 1..index + length).unwrap_or(&[]).iter().collect();
                    let brace_place =
                        Place::new(argument.place.line, column_of(index), u32::try_from(length + 1).unwrap_or(1));
                    if !plain.is_empty() {
                        parts.push(Expression {
                            kind: ExpressionKind::Text(std::mem::take(&mut plain)),
                            type_id: TEXT,
                            place,
                        });
                    }
                    if let Some(part) = self.brace(&inside, brace_place) {
                        parts.push(part);
                    }
                    index += length + 1;
                }
                '}' => {
                    self.not_designed(
                        Place::new(argument.place.line, column_of(index), 1),
                        "a `}` with no `{` before it in `fmt` text",
                    );
                    return Self::error(place);
                }
                other => {
                    plain.push(other);
                    index += 1;
                }
            }
        }
        if !plain.is_empty() || parts.is_empty() {
            parts.push(Expression { kind: ExpressionKind::Text(plain), type_id: TEXT, place });
        }
        Expression { kind: ExpressionKind::Join(parts), type_id: TEXT, place }
    }

    /// What's inside one pair of `fmt`'s braces: a name, which may be followed by its fields and facts (D28).
    fn brace(&mut self, inside: &str, place: Place) -> Option<Expression> {
        let is_name = |word: &str| {
            !word.is_empty()
                && word.starts_with(|first: char| first.is_ascii_lowercase())
                && word
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
        };
        if inside.is_empty() {
            self.not_designed(place, "empty braces in `fmt` text");
            return None;
        }
        if inside.contains("places:") {
            self.not_designed(place, "`places:` in `fmt`'s braces");
            return None;
        }
        if !inside.split('.').all(is_name) {
            // trace: D12, D28
            self.refuse_why(
                place,
                format!("`{{{inside}}}` isn't a name with its fields and facts"),
                "`fmt`'s braces hold a name, which may be followed by its fields and facts, never a calculation or a call.",
            );
            return None;
        }
        let mut column = place.column + 1;
        let mut built: Option<syntax::Expression> = None;
        for word in inside.split('.') {
            let width = u32::try_from(word.len()).unwrap_or(1);
            let word_place = Place::new(place.line, column, width);
            built = Some(match built {
                None => syntax::Expression {
                    id: u32::MAX,
                    kind: syntax::ExpressionKind::Name(word.to_owned()),
                    place: word_place,
                },
                Some(receiver) => {
                    let whole = receiver.place.to(word_place);
                    syntax::Expression {
                        id: u32::MAX,
                        kind: syntax::ExpressionKind::Member {
                            receiver: Box::new(receiver),
                            name: syntax::Word { text: word.to_owned(), place: word_place },
                        },
                        place: whole,
                    }
                }
            });
            column += width + 1;
        }
        let built = built?;
        if !self.is_value(&built) {
            let first = inside.split('.').next().unwrap_or(inside);
            self.refuse_why(
                place,
                format!("there's no name `{first}` here"),
                "a name in `fmt`'s braces is checked when the script loads.",
            );
            return None;
        }
        let checked = self.expression(&built, None);
        self.require(checked.type_id, Requirement::Showable, place, "`fmt`'s braces show a value as text");
        Some(checked)
    }

    /// `be.Text`, `be.Whole` and its narrowings, and `be.Float` (D22, Unio D22, D153, D178).
    #[allow(clippy::too_many_lines)] // one case per conversion, each with its own refusals
    fn conversion(&mut self, path: &[String], arguments: &[syntax::Entry], place: Place) -> Expression {
        let narrowing: Vec<&str> = path.iter().skip(1).map(String::as_str).collect();
        let full = path.join(".");
        let rounding = match narrowing.as_slice() {
            ["Text" | "Whole" | "Float"] => None,
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
            [target, ..]
                if target.starts_with(|first: char| first.is_ascii_uppercase())
                    && !["Text", "Whole", "Float"].contains(target) =>
            {
                self.not_designed(place, format!("converting to `{target}` with `be`"));
                return Self::error(place);
            }
            _ => {
                self.refuse_why(
                    place,
                    format!("there's no `{full}`"),
                    "`be`'s narrowings are `be.Text`, `be.Float`, `be.Whole`, `be.Whole.floor`, `be.Whole.trunc`, \
                     `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.",
                );
                return Self::error(place);
            }
        };
        let Some(argument) = self.single_argument(&full, path, path.len(), arguments, place) else {
            return Self::error(place);
        };
        let value = self.expression(argument, None);
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
            (Some("Float"), _) => {
                if found == FLOAT {
                    return same_type(self, "Float");
                }
                if !self.program.types.is_open(found) && found != WHOLE && found != ERROR {
                    let written = self.describe(found);
                    self.not_designed(argument.place, format!("converting {written} to a `Float`"));
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
                    let written = self.describe(found);
                    self.not_designed(argument.place, format!("converting {written} to a `Whole`"));
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
        receiver: &syntax::Expression,
        method: &syntax::Word,
        arguments: &[syntax::Entry],
        place: Place,
    ) -> Expression {
        let label_of = |entry: &syntax::Entry| entry.label.as_ref().map(|label| label.text.clone());
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
        if !matches!(method.text.as_str(), "add" | "remove") {
            let checked = self.expression(receiver, None);
            if !self.program.types.is_error(checked.type_id) {
                let found = self.describe(checked.type_id);
                self.refuse(method.place, format!("{found} has no method `{}`", method.text));
            }
            return Self::error(place);
        }
        // `.add` and `.remove` change their value, so they're called on a `mut` name (Unio D77).
        let syntax::ExpressionKind::Name(name) = &receiver.kind else {
            self.not_designed(receiver.place, "changing a part of a part, more than one step inside a name,");
            return Self::error(place);
        };
        let Some(variable) = self.lookup(name, receiver.place) else { return Self::error(place) };
        let changeable = self.check_changeable(variable, name, receiver.place);
        let variable_type = self.program.types.resolve(self.variable_type(variable));
        let result = match (self.program.types.get(variable_type).clone(), method.text.as_str()) {
            (Type::List { item, .. }, "add") => {
                let mut value = None;
                let mut position = None;
                for entry in arguments {
                    match label_of(entry).as_deref() {
                        None if value.is_none() => {
                            let checked = self.expression(&entry.value, Some(item));
                            self.expect(&checked, item, "the list's item");
                            value = Some(checked);
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
        Expression { kind: result, type_id: NOTHING, place }
    }
}
