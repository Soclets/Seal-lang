//! Checking expressions: literals, names, fields and facts, items, building values, operators, and dealing with
//! failures.

use super::tree::{Expression, ExpressionKind, HandleFallback};
use super::types::{BOOL, ERROR, FLOAT, NOTHING, TEXT, Type, TypeId, WHOLE};
use super::{Checker, Requirement};
use crate::message::Place;
use crate::number::read_float_literal;
use crate::syntax::lexer::cook_text;
use crate::syntax::tree::{self as syntax, BinaryOperator, UnaryOperator};

impl Checker<'_> {
    /// Checks an expression, used where a value of the type `wanted` goes, when that's known.
    pub(super) fn expression(&mut self, expression: &syntax::Expression, wanted: Option<TypeId>) -> Expression {
        let checked = self.expression_as_written(expression, wanted);
        match wanted {
            Some(wanted) => {
                let checked = self.land_number(checked, wanted);
                let checked = self.fit_limit(checked, wanted);
                self.land_unit(checked, wanted)
            }
            None => checked,
        }
    }

    /// trace: D73. A list moves freely into a `len.max` at least as big as its own, or into none. Into a smaller
    /// one, or from none into one, it might not fit: that's a failure, `full`, like a `Dict`'s missing key.
    fn fit_limit(&mut self, value: Expression, wanted: TypeId) -> Expression {
        let found = self.program.types.resolve(value.type_id);
        let wanted = self.program.types.resolve(wanted);
        let (Type::List { item: found_item, max: from }, Type::List { item: wanted_item, max: to }) =
            (self.program.types.get(found).clone(), self.program.types.get(wanted).clone())
        else {
            return value;
        };
        if from == to || !self.program.types.unify(found_item, wanted_item) {
            return value;
        }
        let place = value.place;
        match (from, to) {
            (_, None) => Expression { type_id: wanted, ..value },
            (Some(from), Some(to)) if from <= to => Expression { type_id: wanted, ..value },
            (_, Some(limit)) => Expression {
                kind: ExpressionKind::Convert {
                    conversion: super::tree::Conversion::ListToLimit(limit),
                    value: Box::new(value),
                },
                type_id: wanted,
                place,
            },
        }
    }

    #[allow(clippy::too_many_lines)] // one case per kind of expression
    fn expression_as_written(&mut self, expression: &syntax::Expression, wanted: Option<TypeId>) -> Expression {
        use syntax::ExpressionKind as Kind;
        let place = expression.place;
        match &expression.kind {
            Kind::Whole(_) | Kind::Float { .. }
                if let Some(number) = self.number_literal(&expression.kind, false, wanted, place) =>
            {
                number
            }
            Kind::Whole(digits) => {
                if let Ok(value) = digits.parse::<i64>() {
                    Expression { kind: ExpressionKind::Whole(value), type_id: WHOLE, place }
                } else {
                    self.refuse_why(
                        place,
                        format!("`{digits}` is too big for a `Whole`"),
                        "a `Whole` goes up to 9223372036854775807.",
                    );
                    Self::error(place)
                }
            }
            Kind::Float { text, approximate } => self.float_literal(text, *approximate, false, place),
            Kind::Bool(value) => Expression { kind: ExpressionKind::Bool(*value), type_id: BOOL, place },
            // The text is kept as written until here, where `\` makes its next character literal (D63), and every
            // text fills in its braces (D75).
            Kind::Text(text) => self.text_literal(text, place),
            Kind::Name(name) => self.name(name, place),
            Kind::TypeName(word) => {
                let name = &word.text;
                if self.record_names.contains_key(name) {
                    // trace: D27
                    self.refuse_why(
                        place,
                        format!("a `{name}` is built with `[ ]`, naming every field"),
                        "a record is built as `Name[field: value]`.",
                    );
                } else {
                    self.refuse(place, format!("`{name}` is a type, not a value"));
                }
                Self::error(place)
            }
            Kind::Member { receiver, name } => {
                if let Some(path) = type_path(expression) {
                    return self.build(&path, None, wanted, place);
                }
                if !self.is_value(receiver) {
                    let path = name_path(expression).unwrap_or_default().join(".");
                    let word = path.split('.').next().unwrap_or("").to_owned();
                    if self.words.contains(&word) || super::BUILT_IN_WORDS.contains(&word.as_str()) {
                        // trace: Unio D4
                        self.refuse_why(
                            place,
                            format!("`{path}` is a procedure, and a call writes `( )`"),
                            "every call writes `( )`, even with nothing inside.",
                        );
                    } else {
                        self.refuse(receiver.place, format!("there's no name `{word}` here"));
                    }
                    return Self::error(place);
                }
                // trace: D65, D66. `.len.egc`, `.len.ucp` and `.len.bytes` say which unit a text's length counts.
                if let Kind::Member { receiver: text, name: fact } = &receiver.kind
                    && fact.text == "len"
                    && matches!(name.text.as_str(), "egc" | "ucp" | "bytes")
                {
                    let text = self.expression(text, Some(TEXT));
                    if !self.program.types.unify(text.type_id, TEXT) {
                        let found = self.describe(text.type_id);
                        self.refuse_why(
                            name.place,
                            format!("`.len.{}` is a text's length, but this is {found}", name.text),
                            "only a text's length comes in graphemes, code points and bytes.",
                        );
                        return Self::error(place);
                    }
                    return Expression {
                        kind: ExpressionKind::Member { receiver: Box::new(text), name: format!("len.{}", name.text) },
                        type_id: WHOLE,
                        place,
                    };
                }
                let receiver = self.expression(receiver, None);
                let result = self.program.types.fresh();
                self.require(
                    receiver.type_id,
                    Requirement::Member { name: name.text.clone(), result },
                    name.place,
                    "a `.` after a value reads one of its fields or facts",
                );
                Expression {
                    kind: ExpressionKind::Member { receiver: Box::new(receiver), name: name.text.clone() },
                    type_id: result,
                    place,
                }
            }
            Kind::Call { callee, arguments } => self.call(expression, callee, arguments, wanted),
            Kind::Bracketed { receiver, entries } => self.bracketed(receiver, entries, wanted, place),
            Kind::List(items) => self.list_literal(items, wanted, place),
            Kind::Pairs(pairs) => self.pairs(pairs, wanted, place),
            Kind::EmptyDict => {
                if let Some(Type::Dict { .. }) = wanted.map(|wanted| self.program.types.get(wanted).clone()) {
                    Expression { kind: ExpressionKind::Dict(Vec::new()), type_id: wanted.unwrap_or(ERROR), place }
                } else {
                    // trace: D30
                    self.refuse_why(
                    place,
                    "`[:]` can't say what the dictionary holds, so the name's type has to be written",
                    "a type is written where the value can't say it, as in `name: Dict<key: Text, value: Whole> = [:]`.",
                );
                    Self::error(place)
                }
            }
            Kind::Unary { operator: UnaryOperator::Negate, operand }
                if matches!(operand.kind, Kind::Whole(_) | Kind::Float { .. })
                    && let Some(number) = self.number_literal(&operand.kind, true, wanted, place) =>
            {
                number
            }
            Kind::Unary { operator: UnaryOperator::Negate, operand } => match &operand.kind {
                // A `-` right before a number makes the number negative, so the smallest `Whole` can be written.
                Kind::Whole(digits) => {
                    if let Ok(value) = format!("-{digits}").parse::<i64>() {
                        Expression { kind: ExpressionKind::Whole(value), type_id: WHOLE, place }
                    } else {
                        self.refuse_why(
                            place,
                            format!("`-{digits}` is too small for a `Whole`"),
                            "a `Whole` goes down to -9223372036854775808.",
                        );
                        Self::error(place)
                    }
                }
                Kind::Float { text, approximate } => self.float_literal(text, *approximate, true, place),
                _ => {
                    // The negated value lands where it's used, not its operand on its own.
                    let operand = self.expression_as_written(operand, wanted);
                    self.require(
                        operand.type_id,
                        Requirement::Number,
                        operand.place,
                        "a `-` in front negates a number",
                    );
                    let type_id = operand.type_id;
                    Expression { kind: ExpressionKind::Negate(Box::new(operand)), type_id, place }
                }
            },
            Kind::Unary { operator: UnaryOperator::Not, operand } => {
                let operand = self.expression(operand, Some(BOOL));
                self.expect_operand(&operand, BOOL, "`not`");
                Expression { kind: ExpressionKind::Not(Box::new(operand)), type_id: BOOL, place }
            }
            Kind::Binary { operator, left, right, operator_place } => {
                self.binary(*operator, left, right, *operator_place, wanted, place)
            }
            Kind::Group(inner) => {
                // The brackets land as a whole where they're used, so what's inside them doesn't land twice.
                let mut checked = self.expression_as_written(inner, wanted);
                checked.place = place;
                checked
            }
            Kind::Try(inner) => {
                let checked = self.expression(inner, wanted);
                if self.context.procedure.is_none() {
                    // trace: D67
                    self.refuse_why(
                        place,
                        "`?` hands a failure to a caller, and the top level has none",
                        "a failure at the top level is handled on the spot, with `??` or `otherwise`.",
                    );
                    return Self::error(place);
                }
                if self.program.types.is_error(checked.type_id) && matches!(checked.kind, ExpressionKind::Error) {
                    return checked;
                }
                if !self.is_failable(&checked) {
                    // trace: Unio D23
                    self.refuse_why(
                        place,
                        "`?` follows something that can't fail",
                        "`?` passes a failure up, so it goes after a call that can fail.",
                    );
                    return Self::error(place);
                }
                let type_id = checked.type_id;
                Expression { kind: ExpressionKind::Try(Box::new(checked)), type_id, place }
            }
            Kind::Handled { call, fallback } => self.handled(call, fallback, wanted, place),
            Kind::Otherwise { call, cases, includes } => self.otherwise(call, cases, includes, wanted, place),
            Kind::Measured { number, unit } => self.measured_literal(number, unit, place),
            Kind::MeasureIn { value, unit } => self.measure_in(value, unit, place),
            Kind::Unit(unit) => {
                self.refuse_why(
                    unit.place,
                    "a unit stands right after its number, as `2 secs`, or in a type, as `Float[unit: secs]`",
                    "a unit says what a number counts, so it goes with one.",
                );
                Self::error(place)
            }
            Kind::Slide { start, steps } => self.slide(start, steps, wanted, place),
        }
    }

    /// A text as written: what it holds, with its braces filled in (D12, D28, D63, D75, D94, D118, Unio D30).
    fn text_literal(&mut self, text: &str, place: Place) -> Expression {
        if !text.contains('{') {
            return Expression { kind: ExpressionKind::Text(cook_text(text)), type_id: TEXT, place };
        }
        let characters: Vec<char> = text.chars().collect();
        let column_of = |index: usize| place.column + 1 + u32::try_from(index).unwrap_or(0);
        let mut parts: Vec<Expression> = Vec::new();
        let mut plain = String::new();
        let mut index = 0;
        while let Some(&character) = characters.get(index) {
            match character {
                // trace: D63, Unio D30. `\{` is a brace, `\"` a quote and `\\` a backslash.
                '\\' => {
                    plain.extend(characters.get(index + 1));
                    index += 2;
                }
                '{' => {
                    let Some(length) =
                        characters.get(index..).and_then(|rest| rest.iter().position(|&next| next == '}'))
                    else {
                        // trace: D75. A brace shown as it is, is written `\{`.
                        let mut fixed: Vec<char> = self.source_line(place.line).chars().collect();
                        let at = (column_of(index) as usize).saturating_sub(1);
                        if at <= fixed.len() {
                            fixed.insert(at, '\\');
                        }
                        self.messages.push(
                            crate::message::Message::refused(
                                Place::new(place.line, column_of(index), 1),
                                "this `{` opens braces that are never closed",
                            )
                            .with_fix(fixed.into_iter().collect::<String>())
                            .with_why("braces in text fill in a name, as `{score}`; a brace shown as it is, is written `\\{`."),
                        );
                        return Self::error(place);
                    };
                    let inside: String = characters.get(index + 1..index + length).unwrap_or(&[]).iter().collect();
                    let brace_place = Place::new(place.line, column_of(index), u32::try_from(length + 1).unwrap_or(1));
                    match inside.as_str() {
                        // trace: D118. `{newln}` and `{tab}` give a line break and a tab.
                        "newln" => plain.push('\n'),
                        "tab" => plain.push('\t'),
                        "" => {
                            // trace: D93
                            let mut fixed: Vec<char> = self.source_line(place.line).chars().collect();
                            let at = (column_of(index) as usize).saturating_sub(1);
                            if at <= fixed.len() {
                                fixed.insert(at, '\\');
                            }
                            self.messages.push(
                                crate::message::Message::refused(
                                    brace_place,
                                    "these braces are empty, so they fill in nothing",
                                )
                                .with_fix(fixed.into_iter().collect::<String>())
                                .with_why("braces in text fill in a name; a brace shown as it is, is written `\\{`."),
                            );
                        }
                        _ => {
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
                        }
                    }
                    index += length + 1;
                }
                // A `}` with no open `{` is just a character (D63, Unio D30).
                other => {
                    plain.push(other);
                    index += 1;
                }
            }
        }
        if parts.is_empty() {
            return Expression { kind: ExpressionKind::Text(plain), type_id: TEXT, place };
        }
        if !plain.is_empty() {
            parts.push(Expression { kind: ExpressionKind::Text(plain), type_id: TEXT, place });
        }
        Expression { kind: ExpressionKind::Join(parts), type_id: TEXT, place }
    }

    /// What's inside one pair of braces: a name, which may be followed by its fields and facts, and for a `Float`,
    /// `places:` (D28, D75, D94, D100).
    fn brace(&mut self, inside: &str, place: Place) -> Option<Expression> {
        let is_name = |word: &str| {
            !word.is_empty()
                && word.starts_with(|first: char| first.is_ascii_lowercase())
                && word
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
        };
        let (path, places) = match inside.split_once(", places: ") {
            Some((path, places)) => {
                let Ok(places) = places.parse::<u8>() else {
                    self.refuse_why(
                        place,
                        format!("`places:` takes a whole number written out, but this is `{places}`"),
                        "`places:` says how many digits a `Float` shows after its point, as `{speed, places: 2}`.",
                    );
                    return None;
                };
                (path, Some(places))
            }
            None => (inside, None),
        };
        // trace: D122. `{total in secs}` measures the name in a unit, as D122's own example writes it.
        let (path, measured) = match path.split_once(" in ") {
            Some((path, unit)) => (path, Some(unit)),
            None => (path, None),
        };
        let measured_unit = match measured {
            None => None,
            Some(unit) => {
                let unit_column = place.column + 1 + u32::try_from(path.len()).unwrap_or(0) + 4;
                let Some(written) = brace_unit(unit, place.line, unit_column) else {
                    self.refuse_why(
                        place,
                        format!("`{{{inside}}}` measures in `{unit}`, which isn't a unit's name"),
                        "`in` in braces measures a name in one of its group's units, as `{total in secs}`.",
                    );
                    return None;
                };
                Some(written)
            }
        };
        if !path.split('.').all(is_name) {
            // trace: D12, D28, D75
            self.refuse_why(
                place,
                format!("`{{{inside}}}` isn't a name with its fields and facts"),
                "braces in text hold a name, which may be followed by its fields and facts, never a calculation or a call; a `Float` can add `, places: 2`.",
            );
            return None;
        }
        let built = brace_path(path, place)?;
        // `pi` is a built-in value, so braces can hold it as they hold any name (D109).
        if !self.is_value(&built) && path != "pi" {
            let first = path.split('.').next().unwrap_or(path);
            self.refuse_why(
                place,
                format!("there's no name `{first}` here"),
                "a name in braces is checked when the script loads; a brace shown as it is, is written `\\{`.",
            );
            return None;
        }
        let built = match measured_unit {
            Some(unit) => {
                let whole = built.place.to(unit.place);
                syntax::Expression {
                    id: u32::MAX,
                    kind: syntax::ExpressionKind::MeasureIn { value: Box::new(built), unit: Box::new(unit) },
                    place: whole,
                }
            }
            None => built,
        };
        let checked = self.expression(&built, None);
        let Some(places) = places else { return Some(checked) };
        // trace: D94, D125, Unio D175. `places:` rounds a `Float`, a `Decimal` or a `HugeDecimal` for showing.
        if !self.takes_decimal(checked.type_id, false) && !self.program.types.unify(checked.type_id, FLOAT) {
            let found = self.describe(checked.type_id);
            self.refuse_why(
                place,
                format!("`places:` shows a `Float` to so many digits, but this is {found}"),
                "`places:` is only for a `Float`, which otherwise shows every digit it needs.",
            );
            return None;
        }
        Some(Expression { kind: ExpressionKind::ShowPlaces { value: Box::new(checked), places }, type_id: TEXT, place })
    }

    /// The script's line as written, by its number.
    pub(super) fn source_line(&self, number: u32) -> String {
        self.script.line(number).to_owned()
    }

    /// trace: Unio D155, D180
    fn float_literal(&mut self, text: &str, approximate: bool, negative: bool, place: Place) -> Expression {
        let Some((value, exact)) = read_float_literal(text) else {
            self.refuse_why(
                place,
                format!("`{text}` is too big for a `Float`"),
                "a `Float` goes up to about 10 to the 308.",
            );
            return Self::error(place);
        };
        let sign = if negative { "-" } else { "" };
        if !exact && !approximate {
            self.refuse_why(
                place,
                format!(
                    "`{sign}{text}` can't be stored exactly as a `Float`; the inexact number is written `{sign}~{text}`"
                ),
                "a `~` marks a `Float` that's only close to what's written.",
            );
            return Self::error(place);
        }
        if exact && approximate {
            self.refuse_why(
                place,
                format!("`{text}` is stored exactly as a `Float`, so it takes no `~`"),
                "a `~` marks a `Float` that's only close to what's written.",
            );
            return Self::error(place);
        }
        let value = if negative { -value } else { value };
        Expression { kind: ExpressionKind::Float(value), type_id: FLOAT, place }
    }

    fn name(&mut self, name: &str, place: Place) -> Expression {
        if let Some(id) = self.lookup(name, place) {
            let marked_unused = self.variable(id).is_some_and(|variable| variable.marked_unused);
            if marked_unused {
                // trace: D3
                self.refuse_why(
                    place,
                    format!("`{name}` is marked `unused`, so it can't be read"),
                    "`unused` marks a name that's never read on purpose.",
                );
                return Self::error(place);
            }
            self.mark_read(id);
            return Expression { kind: ExpressionKind::Variable(id), type_id: self.variable_type(id), place };
        }
        if name == "pi" {
            // trace: D109. `pi` is the `Float` nearest π.
            return Expression { kind: ExpressionKind::Float(std::f64::consts::PI), type_id: FLOAT, place };
        }
        if matches!(name, "newln" | "tab") {
            // trace: D129. `newln` and `tab` exist only inside braces.
            self.refuse_why(
                place,
                format!("`{name}` fills in braces in text, so it's written `\"{{{name}}}\"`"),
                "`{newln}` and `{tab}` give a line break and a tab inside text; the text holding just one is written `\"{newln}\"`.",
            );
            return Self::error(place);
        }
        if self.words.contains(name) || super::BUILT_IN_WORDS.contains(&name) {
            // trace: Unio D4
            self.refuse_why(
                place,
                format!("`{name}` is a procedure, and a call writes `( )`"),
                "every call writes `( )`, even with nothing inside.",
            );
        } else {
            self.refuse(place, format!("there's no name `{name}` here"));
        }
        Self::error(place)
    }

    /// Whether an expression is a value, as opposed to a procedure's name with its narrowings.
    pub(super) fn is_value(&self, expression: &syntax::Expression) -> bool {
        match &expression.kind {
            syntax::ExpressionKind::Name(name) => self.visible(name).is_some(),
            syntax::ExpressionKind::Member { receiver, .. } => self.is_value(receiver),
            syntax::ExpressionKind::TypeName(_) => false,
            _ => true,
        }
    }

    /// A name that exists here, found without counting as a use of it.
    pub(super) fn visible(&self, name: &str) -> Option<super::tree::VariableId> {
        self.context
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .or_else(|| if self.context.procedure.is_some() { self.globals.get(name).copied() } else { None })
    }

    fn expect_operand(&mut self, operand: &Expression, wanted: TypeId, operator: &str) -> bool {
        if self.program.types.unify(operand.type_id, wanted) {
            return true;
        }
        let found = self.describe(operand.type_id);
        let wanted = self.describe(wanted);
        self.refuse(operand.place, format!("{operator} takes {wanted}, but this is {found}"));
        false
    }

    /// trace: D16, D22, D29, D108, Unio D17, D27, D34, D70, D71, D162
    #[allow(clippy::too_many_lines)] // one case per kind of operator, with `**`'s own refusal
    fn binary(
        &mut self,
        operator: BinaryOperator,
        left: &syntax::Expression,
        right: &syntax::Expression,
        operator_place: Place,
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let quoted = format!("`{}`", operator.text());
        match operator {
            BinaryOperator::Join => {
                let mut parts = Vec::new();
                self.join_parts(left, &mut parts);
                self.join_parts(right, &mut parts);
                Expression { kind: ExpressionKind::Join(parts), type_id: TEXT, place }
            }
            BinaryOperator::And | BinaryOperator::Or => {
                let left = self.expression(left, Some(BOOL));
                let right = self.expression(right, Some(BOOL));
                self.expect_operand(&left, BOOL, &quoted);
                self.expect_operand(&right, BOOL, &quoted);
                Expression {
                    kind: ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right) },
                    type_id: BOOL,
                    place,
                }
            }
            BinaryOperator::DivFloor
            | BinaryOperator::DivTrunc
            | BinaryOperator::RemFloor
            | BinaryOperator::RemTrunc => {
                // trace: D119, D127. A number written beside a bigger number type takes that type.
                let (left, right) = if super::numbers::is_literal(left) && !super::numbers::is_literal(right) {
                    let right = self.expression_as_written(right, None);
                    (self.expression_as_written(left, Some(right.type_id)), right)
                } else {
                    let left = self.expression_as_written(left, None);
                    let right = self.expression_as_written(right, Some(left.type_id));
                    (left, right)
                };
                if self.big_number(left.type_id).is_some() || self.big_number(right.type_id).is_some() {
                    return self.number_binary(operator, left, right, operator_place, place);
                }
                self.expect_operand(&left, WHOLE, &quoted);
                self.expect_operand(&right, WHOLE, &quoted);
                Expression {
                    kind: ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right) },
                    type_id: WHOLE,
                    place,
                }
            }
            _ => {
                let operand_wanted = if operator.is_comparison() { None } else { wanted };
                let operand_wanted = operand_wanted.filter(|wanted| self.program.types.measured(*wanted).is_none());
                let (left_syntax, right_syntax) = (left, right);
                // What's wanted only guides the sides' types: a side doesn't land there, the whole does, so
                // `7.0 secs / 1.1 secs` is a plain `Float` even where one is wanted (D120).
                // trace: D119. A number written beside another value takes its type, on either side.
                let (left, right) = if super::numbers::is_literal(left) && !super::numbers::is_literal(right) {
                    let right = self.expression_as_written(right, operand_wanted);
                    let right_measured = self.program.types.measured(right.type_id).is_some();
                    (self.expression_as_written(left, (!right_measured).then_some(right.type_id)), right)
                } else {
                    let left = self.expression_as_written(left, operand_wanted);
                    if operator == BinaryOperator::Power && self.big_number(left.type_id).is_some() {
                        return self.number_power(left, right, operator_place, place);
                    }
                    // A value with a unit says its own; the other side works its out (D120).
                    let left_measured = self.program.types.measured(left.type_id).is_some();
                    let right = self.expression_as_written(right, (!left_measured).then_some(left.type_id));
                    (left, right)
                };
                if self.program.types.measured(left.type_id).is_some()
                    || self.program.types.measured(right.type_id).is_some()
                {
                    return self.measured_binary(operator, left, right, operator_place, place);
                }
                if self.big_number(left.type_id).is_some() || self.big_number(right.type_id).is_some() {
                    if operator == BinaryOperator::Power {
                        return self.number_power(left, right_syntax, operator_place, place);
                    }
                    return self.number_binary(operator, left, right, operator_place, place);
                }
                if !self.program.types.unify(left.type_id, right.type_id) {
                    let left_type = self.program.types.resolve(left.type_id);
                    let right_type = self.program.types.resolve(right.type_id);
                    if (left_type == WHOLE && right_type == FLOAT) || (left_type == FLOAT && right_type == WHOLE) {
                        // trace: D16, Unio D162, D180
                        self.refuse_why(
                            operator_place,
                            format!("{quoted} can't mix a `Whole` and a `Float`"),
                            "`Whole` and `Float` don't mix quietly: `be.Float(…)` turns a `Whole` into a `Float`, and a \
                             number written with a point, as `2.0`, is a `Float`.",
                        );
                    } else {
                        let left_text = self.describe(left.type_id);
                        let right_text = self.describe(right.type_id);
                        self.refuse(
                            operator_place,
                            format!("{quoted} has {left_text} on its left, but {right_text} on its right"),
                        );
                    }
                    return Self::error(place);
                }
                if operator == BinaryOperator::Add && self.program.types.resolve(left.type_id) == TEXT {
                    // trace: D77. Text is joined only by filling it in.
                    let mut message = crate::message::Message::refused(operator_place, "`+` doesn't join text; text is joined by filling it in with braces")
                        .with_why("`+` is only for numbers; text is joined by filling it in with braces, as in `\"{first} {last}\"`.");
                    if let (Some(first), Some(second)) = (filled(left_syntax), filled(right_syntax))
                        && left_syntax.place.line == right_syntax.place.line
                    {
                        let line: Vec<char> = self.source_line(left_syntax.place.line).chars().collect();
                        let from = left_syntax.place.column.saturating_sub(1) as usize;
                        let to = (right_syntax.place.column + right_syntax.place.width).saturating_sub(1) as usize;
                        if from <= to && to <= line.len() {
                            let fixed = line.get(..from).unwrap_or(&[]).iter().collect::<String>()
                                + "\""
                                + &first
                                + &second
                                + "\""
                                + &line.get(to..).unwrap_or(&[]).iter().collect::<String>();
                            message = message.with_fix(fixed);
                        }
                    }
                    self.messages.push(message);
                    return Self::error(place);
                }
                let (requirement, purpose) = match operator {
                    BinaryOperator::Equal | BinaryOperator::NotEqual => {
                        (Requirement::Comparable, format!("{quoted} compares two values"))
                    }
                    _ if operator.is_comparison() => (Requirement::Ordered, format!("{quoted} compares two numbers")),
                    _ => (Requirement::Number, format!("{quoted} works on two numbers")),
                };
                self.require(left.type_id, requirement, left.place, &purpose);
                // trace: D108. A negative power of a `Whole` isn't whole.
                if operator == BinaryOperator::Power
                    && self.program.types.resolve(left.type_id) == WHOLE
                    && matches!(right.kind, ExpressionKind::Whole(power) if power < 0)
                {
                    self.refuse_why(
                        right.place,
                        "a `Whole` raised to a negative power isn't a whole number",
                        "a `Whole`'s power is 0 or more; `be.Float(…)` gives `Float`s, whose powers can be negative.",
                    );
                    return Self::error(place);
                }
                let type_id = if operator.is_comparison() { BOOL } else { left.type_id };
                Expression {
                    kind: ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right) },
                    type_id,
                    place,
                }
            }
        }
    }

    /// Collects the pieces of a join, left to right. The reader refuses `&` (D77), so only Seal's own joins get here.
    fn join_parts(&mut self, expression: &syntax::Expression, parts: &mut Vec<Expression>) {
        if let syntax::ExpressionKind::Binary { operator: BinaryOperator::Join, left, right, .. } = &expression.kind {
            self.join_parts(left, parts);
            self.join_parts(right, parts);
            return;
        }
        let checked = self.expression(expression, None);
        parts.push(checked);
    }

    /// `receiver[entries]`: reaching an item, or building a record or a case.
    fn bracketed(
        &mut self,
        receiver: &syntax::Expression,
        entries: &[syntax::Entry],
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        if let Some(path) = type_path(receiver) {
            return self.build(&path, Some(entries), wanted, place);
        }
        if matches!(&receiver.kind, syntax::ExpressionKind::Name(name) if name == "range")
            && self.visible("range").is_none()
        {
            // trace: D99. A `range` is a `List<Whole>`, usable anywhere a list is.
            let Some((from, end, inclusive, step)) = self.range_parts(entries, place) else {
                return Self::error(place);
            };
            let type_id = self.program.types.list(WHOLE);
            return Expression {
                kind: ExpressionKind::Range {
                    from: Box::new(from),
                    end: Box::new(end),
                    inclusive,
                    step: step.map(Box::new),
                },
                type_id,
                place,
            };
        }
        if !self.is_value(receiver) {
            let path = name_path(receiver).unwrap_or_default().join(".");
            self.refuse_why(
                place,
                format!("`{path}` is a procedure, and a call writes `( )`"),
                "`( )` calls a procedure; `[ ]` holds or describes values.",
            );
            return Self::error(place);
        }
        let checked = self.expression(receiver, None);
        let [entry] = entries else {
            self.refuse(place, "an item is reached with one of `pos:`, `offset:` or `key:`");
            return Self::error(place);
        };
        match entry.label.as_ref().map(|label| label.text.as_str()) {
            Some(label @ ("pos" | "offset")) => {
                let index = self.expression(&entry.value, Some(WHOLE));
                self.expect(&index, WHOLE, &format!("`{label}:`"));
                let item = self.program.types.fresh();
                self.require(
                    checked.type_id,
                    Requirement::Indexable { item },
                    receiver.place,
                    "`pos:` and `offset:` reach an item of a list or an array",
                );
                Expression {
                    kind: ExpressionKind::Item {
                        receiver: Box::new(checked),
                        index: Box::new(index),
                        zero_based: label == "offset",
                    },
                    type_id: item,
                    place,
                }
            }
            Some("key") => {
                let key = self.program.types.fresh();
                let value = self.program.types.fresh();
                let dict = self.program.types.dict(key, value);
                if !self.program.types.unify(checked.type_id, dict) {
                    let found = self.describe(checked.type_id);
                    self.refuse(receiver.place, format!("`key:` reaches a value of a `Dict`, but this is {found}"));
                    return Self::error(place);
                }
                let checked_key = self.expression(&entry.value, Some(key));
                self.expect(&checked_key, key, "the dictionary's key");
                Expression {
                    kind: ExpressionKind::Lookup { dict: Box::new(checked), key: Box::new(checked_key) },
                    type_id: value,
                    place,
                }
            }
            _ => {
                // trace: Unio D75
                self.refuse_why(
                    entry.value.place,
                    "an item is reached with `pos:`, `offset:` or `key:`",
                    "`pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.",
                );
                Self::error(place)
            }
        }
    }

    /// Builds a record or a case: `Enemy[…]`, `Event.hurt[…]`, `Event.idle`, `Maybe.some[value: …]`, `Maybe.none`
    /// (D27, D35, D38).
    #[allow(clippy::too_many_lines)] // records, a script's cases and `Maybe`'s are built the same way, checked here
    fn build(
        &mut self,
        path: &[&syntax::Word],
        entries: Option<&[syntax::Entry]>,
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let Some(type_word) = path.first() else { return Self::error(place) };
        let type_name = type_word.text.as_str();
        // What's being built: its type, its fields, and its case's number when it's a case.
        let (type_id, fields, case): (TypeId, Vec<(String, TypeId)>, Option<u32>) = if type_name == "Maybe" {
            let inner = match wanted.map(|wanted| self.program.types.get(wanted).clone()) {
                Some(Type::Maybe(inner)) => inner,
                _ => self.program.types.fresh(),
            };
            let maybe = self.program.types.maybe(inner);
            match path.get(1).map(|word| word.text.as_str()) {
                Some("none") if path.len() == 2 => (maybe, Vec::new(), Some(0)),
                Some("some") if path.len() == 2 => (maybe, vec![("value".to_owned(), inner)], Some(1)),
                _ => {
                    self.refuse_why(
                        place,
                        "a `Maybe` is written `Maybe.none` or `Maybe.some[value: …]`",
                        "`Maybe` has the cases `none` and `some[value]`.",
                    );
                    return Self::error(place);
                }
            }
        } else if let Some(&record) = self.record_names.get(type_name) {
            if let Some(extra) = path.get(1) {
                self.refuse(extra.place, format!("`{type_name}` is a record, which has no cases"));
                return Self::error(place);
            }
            let fields =
                self.program.types.records.get(record as usize).map(|record| record.fields.clone()).unwrap_or_default();
            (self.program.types.intern(Type::Record(record)), fields, None)
        } else if let Some(&one_of) = self.one_of_names.get(type_name) {
            let (Some(case_word), 2) = (path.get(1), path.len()) else {
                // trace: D35
                self.refuse_why(
                    place,
                    format!("a `{type_name}` is written as one of its cases, as in `{type_name}.case`"),
                    "a case is written as a narrowing of its type.",
                );
                return Self::error(place);
            };
            let cases =
                self.program.types.one_ofs.get(one_of as usize).map(|one_of| one_of.cases.clone()).unwrap_or_default();
            let Some(index) = cases.iter().position(|case| case.name == case_word.text) else {
                let real: Vec<String> = cases.iter().map(|case| format!("`{}`", case.name)).collect();
                self.refuse_why(
                    case_word.place,
                    format!("`{type_name}` has no case `{}`", case_word.text),
                    &format!("`{type_name}`'s cases are {}.", real.join(", ")),
                );
                return Self::error(place);
            };
            let fields = cases.get(index).map(|case| case.fields.clone()).unwrap_or_default();
            (self.program.types.intern(Type::OneOf(one_of)), fields, Some(u32::try_from(index).unwrap_or(0)))
        } else {
            if super::BUILT_IN_TYPES.contains(&type_name) {
                self.refuse(type_word.place, format!("`{type_name}` is a type, not a value"));
            } else {
                self.refuse(type_word.place, format!("there's no type named `{type_name}`"));
            }
            return Self::error(place);
        };
        let what = path.iter().map(|word| word.text.as_str()).collect::<Vec<_>>().join(".");
        let entries = match (entries, fields.is_empty()) {
            (None, true) => &[][..],
            (None, false) => {
                let names: Vec<String> = fields.iter().map(|(name, _)| format!("{name}: …")).collect();
                self.refuse_why(
                    place,
                    format!("`{what}` carries values, so it's built with `[ ]`, as in `{what}[{}]`", names.join(", ")),
                    "a case's values are named in `[ ]`, as a record's fields are.",
                );
                return Self::error(place);
            }
            (Some(entries), true) => {
                if case.is_some() || !entries.is_empty() {
                    self.refuse(place, format!("`{what}` carries no values, so it has no `[ ]`"));
                    return Self::error(place);
                }
                entries
            }
            (Some(entries), false) => entries,
        };
        let mut built: Vec<(u32, Expression)> = Vec::new();
        for entry in entries {
            let Some(label) = &entry.label else {
                // trace: D27
                self.refuse_why(
                    entry.value.place,
                    format!("each value of `{what}` is named, as in `field: value`"),
                    "a record is built naming every field, so which value is which can be seen.",
                );
                continue;
            };
            let Some(index) = fields.iter().position(|(name, _)| *name == label.text) else {
                self.refuse(label.place, format!("`{what}` has no field `{}`", label.text));
                continue;
            };
            let index_number = u32::try_from(index).unwrap_or(0);
            if built.iter().any(|(earlier, _)| *earlier == index_number) {
                self.refuse(label.place, format!("`{}` is given twice", label.text));
                continue;
            }
            let field_type = fields.get(index).map_or(ERROR, |(_, field_type)| *field_type);
            let value = self.expression(&entry.value, Some(field_type));
            self.expect(&value, field_type, &format!("`{}`", label.text));
            let value = self.land(value, field_type);
            built.push((index_number, value));
        }
        let missing: Vec<String> = fields
            .iter()
            .enumerate()
            .filter(|(index, _)| !built.iter().any(|(given, _)| *given as usize == *index))
            .map(|(_, (name, _))| format!("`{name}`"))
            .collect();
        if !missing.is_empty() && entries.iter().all(|entry| entry.label.is_some()) {
            // trace: D27
            self.refuse_why(
                place,
                format!("`{what}` is built without {}", missing.join(", ")),
                "a record is built naming every field; one left out is refused.",
            );
            return Self::error(place);
        }
        let kind = match case {
            Some(case) => ExpressionKind::BuildCase { case, fields: built },
            None => match self.program.types.get(type_id) {
                Type::Record(record) => ExpressionKind::BuildRecord { record: *record, fields: built },
                _ => ExpressionKind::Error,
            },
        };
        Expression { kind, type_id, place }
    }

    /// trace: D30, Unio D75, D76, D167
    fn list_literal(&mut self, items: &[syntax::Expression], wanted: Option<TypeId>, place: Place) -> Expression {
        let wanted_kind = wanted.map(|wanted| self.program.types.get(wanted).clone());
        if wanted_kind == Some(Type::Error) {
            // A written type that was refused has had its message already.
            for item in items {
                self.expression(item, None);
            }
            return Self::error(place);
        }
        match wanted_kind {
            Some(Type::List { item, max }) => {
                let checked: Vec<Expression> = items.iter().map(|value| self.item(value, item)).collect();
                if let Some(max) = max.filter(|&max| i64::try_from(checked.len()).unwrap_or(i64::MAX) > max) {
                    // trace: D73
                    self.refuse_why(
                        place,
                        format!("this list has {} items, but its `len.max` is {max}", checked.len()),
                        "a list never holds more than its `len.max`.",
                    );
                }
                Expression { kind: ExpressionKind::List(checked), type_id: wanted.unwrap_or(ERROR), place }
            }
            Some(Type::Array { item, length }) => {
                let checked: Vec<Expression> = items.iter().map(|value| self.item(value, item)).collect();
                if i64::try_from(checked.len()).unwrap_or(i64::MAX) != length {
                    // trace: Unio D167
                    self.refuse_why(
                        place,
                        format!("this has {} items, but the array holds exactly {length}", checked.len()),
                        "an array's length is part of its type.",
                    );
                }
                Expression { kind: ExpressionKind::List(checked), type_id: wanted.unwrap_or(ERROR), place }
            }
            _ => {
                let Some(first) = items.first() else {
                    // trace: D30
                    self.refuse_why(
                        place,
                        "`[]` can't say what the list holds, so the name's type has to be written",
                        "a type is written where the value can't say it, as in `name: List<Text> = []`.",
                    );
                    return Self::error(place);
                };
                let first = self.expression(first, None);
                let item = first.type_id;
                let mut checked = vec![first];
                for value in items.iter().skip(1) {
                    checked.push(self.item(value, item));
                }
                let type_id = self.program.types.list(item);
                Expression { kind: ExpressionKind::List(checked), type_id, place }
            }
        }
    }

    fn item(&mut self, value: &syntax::Expression, item: TypeId) -> Expression {
        let checked = self.expression(value, Some(item));
        self.expect(&checked, item, "the list's item");
        self.land(checked, item)
    }

    /// `[key: value, …]` (D32) and `[all: value]` (Unio D169).
    fn pairs(
        &mut self,
        pairs: &[(syntax::Expression, syntax::Expression)],
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let wanted_kind = wanted.map(|wanted| self.program.types.get(wanted).clone());
        if let [(key, value)] = pairs
            && matches!(&key.kind, syntax::ExpressionKind::Name(name) if name == "all")
        {
            let Some(Type::Array { item, length }) = wanted_kind else {
                // trace: Unio D169
                self.refuse_why(
                    place,
                    "`[all: value]` is only allowed where the length is known, as for an `Array`",
                    "`[all: value]` takes its length from the type, as in `board: Array<Whole>[len: 64] = [all: 0]`.",
                );
                return Self::error(place);
            };
            let checked = self.item(value, item);
            return Expression {
                kind: ExpressionKind::Repeat { value: Box::new(checked), count: length },
                type_id: wanted.unwrap_or(ERROR),
                place,
            };
        }
        let (key_type, value_type, type_id) = if let Some(Type::Dict { key, value }) = wanted_kind {
            (key, value, wanted.unwrap_or(ERROR))
        } else {
            let key = self.program.types.fresh();
            let value = self.program.types.fresh();
            (key, value, self.program.types.dict(key, value))
        };
        let mut checked = Vec::new();
        let mut plain_keys: Vec<String> = Vec::new();
        for (key, value) in pairs {
            let checked_key = self.expression(key, Some(key_type));
            self.expect(&checked_key, key_type, "the dictionary's key");
            let checked_value = self.expression(value, Some(value_type));
            self.expect(&checked_value, value_type, "the dictionary's value");
            let checked_value = self.land(checked_value, value_type);
            let plain = match &checked_key.kind {
                ExpressionKind::Whole(number) => Some(number.to_string()),
                ExpressionKind::Text(text) => Some(format!("\"{text}\"")),
                ExpressionKind::Bool(flag) => Some(flag.to_string()),
                _ => None,
            };
            if let Some(plain) = plain {
                if plain_keys.contains(&plain) {
                    // trace: D73
                    self.refuse_why(
                        key.place,
                        format!("the key {plain} is written twice in this dictionary"),
                        "a dictionary holds one value for each key, so one of the two would vanish.",
                    );
                }
                plain_keys.push(plain);
            }
            checked.push((checked_key, checked_value));
        }
        let key_place = pairs.first().map_or(place, |(key, _)| key.place);
        self.check_key_type(key_type, key_place);
        Expression { kind: ExpressionKind::Dict(checked), type_id, place }
    }

    /// Whether an expression can fail, so that it must be dealt with (Unio D23).
    pub(super) fn is_failable(&self, expression: &Expression) -> bool {
        match &expression.kind {
            ExpressionKind::Call { procedure, .. } => {
                self.program.procedure(*procedure).is_some_and(|procedure| procedure.can_fail)
            }
            ExpressionKind::DoorCall { door, .. } => {
                self.doors.get(*door as usize).is_some_and(|door| !door.failures.is_empty())
            }
            // A change to a part can fail as the change itself can (D62).
            ExpressionKind::Inside { change, .. } => self.is_failable(change),
            ExpressionKind::Lookup { .. } | ExpressionKind::DictRemove { .. } => true,
            ExpressionKind::Convert { conversion, .. } => {
                matches!(
                    conversion,
                    super::tree::Conversion::TextToWhole
                        | super::tree::Conversion::TextToBool
                        | super::tree::Conversion::ListToLimit(_)
                ) || matches!(conversion, super::tree::Conversion::Number(number) if number.failure().is_some())
            }
            ExpressionKind::ListAdd { variable, .. } => {
                matches!(self.program.types.get(self.variable_type(*variable)), Type::List { max: Some(_), .. })
            }
            _ => false,
        }
    }

    /// `call ?? fallback` (Unio D23, D81, D121), and `maybe ?? fallback` (D38).
    #[allow(clippy::too_many_lines)] // a value, `ignore`, `crash` and a handler, on a call or a `Maybe`
    fn handled(
        &mut self,
        call: &syntax::Expression,
        fallback: &syntax::Fallback,
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let checked = self.expression(call, None);
        // trace: D73. A list moved into a `len.max` it might not fit can fail with `full`, which this deals with.
        let checked = match wanted {
            Some(wanted) if !matches!(self.program.types.get(checked.type_id), Type::Maybe(_)) => {
                self.fit_limit(checked, wanted)
            }
            _ => checked,
        };
        if matches!(checked.kind, ExpressionKind::Error) {
            if let syntax::Fallback::Value(value) = fallback {
                self.expression(value, wanted);
            }
            return Self::error(place);
        }
        if self.is_failable(&checked) {
            let type_id = checked.type_id;
            let fallback = match fallback {
                syntax::Fallback::Value(value) => {
                    if self.program.types.resolve(type_id) == NOTHING {
                        self.refuse_why(
                            value.place,
                            "this call gives nothing, so there's no value to stand in for",
                            "after a call that gives nothing, a failure is dealt with by `?? ignore`.",
                        );
                    }
                    let value = self.expression(value, Some(type_id));
                    self.expect(&value, type_id, "the call it stands in for");
                    HandleFallback::Value(Box::new(value))
                }
                syntax::Fallback::Ignore => {
                    if !self.program.types.unify(type_id, NOTHING) {
                        // trace: Unio D121
                        self.refuse_why(
                            place,
                            "`?? ignore` follows a call that gives a value",
                            "with the failure ignored there'd be no value to carry on with; `?? ignore` goes after a \
                             call that gives nothing.",
                        );
                    }
                    HandleFallback::Ignore
                }
                syntax::Fallback::Crash => HandleFallback::Crash,
                syntax::Fallback::Handler(using) => {
                    // trace: D98, D110. A handler is used as if its cases were written at the call.
                    let gives_nothing = self.program.types.resolve(type_id) == NOTHING;
                    let Some(cases) = self.handler_cases(&checked, using, type_id, gives_nothing) else {
                        return Self::error(place);
                    };
                    let shared = cases.iter().map(|(failure, _)| *failure).collect();
                    return Expression {
                        kind: ExpressionKind::Otherwise { call: Box::new(checked), cases, shared },
                        type_id,
                        place,
                    };
                }
            };
            return Expression { kind: ExpressionKind::Handle { call: Box::new(checked), fallback }, type_id, place };
        }
        // trace: D5. A value that isn't a call can only be a `Maybe` here, so that's what its type is worked out as.
        if matches!(self.program.types.get(checked.type_id), Type::Variable(_)) {
            let inner = self.program.types.fresh();
            let maybe = self.program.types.maybe(inner);
            self.program.types.unify(checked.type_id, maybe);
        }
        match self.program.types.get(checked.type_id).clone() {
            Type::Maybe(_) if matches!(fallback, syntax::Fallback::Handler(_)) => {
                self.refuse_why(
                    place,
                    "a handler deals with a call's failures, and a `Maybe` isn't a failure",
                    "a `Maybe` is given a value to use when it's `none`, as `?? 0`, or `?? crash` when it can't be `none`.",
                );
                Self::error(place)
            }
            Type::Maybe(inner) => {
                if let syntax::Fallback::Value(value) = fallback {
                    let value = self.expression(value, Some(inner));
                    self.expect(&value, inner, "the value inside the `Maybe`");
                    Expression {
                        kind: ExpressionKind::MaybeOr { maybe: Box::new(checked), fallback: Box::new(value) },
                        type_id: inner,
                        place,
                    }
                } else if matches!(fallback, syntax::Fallback::Crash) {
                    // trace: D79. `?? crash` says the `Maybe` can't be `none` here.
                    Expression { kind: ExpressionKind::MaybeCrash(Box::new(checked)), type_id: inner, place }
                } else {
                    // trace: D79. The name would be left with nothing in it, the `nil` D38 rules out.
                    self.refuse_why(
                        place,
                        "`?? ignore` would leave nothing in place of a `Maybe` that's `none`",
                        "a `Maybe` is given a value to use when it's `none`, as `?? 0`, or `?? crash` when it can't be `none`; `?? ignore` goes after a call that gives nothing.",
                    );
                    Self::error(place)
                }
            }
            Type::Variable(_) => {
                self.not_built(place, "working out a type from a later line (D5), which this `??` needs, isn't");
                Self::error(place)
            }
            _ => {
                // trace: Unio D23, D38
                self.refuse_why(
                    place,
                    "`??` follows something that can't fail and isn't a `Maybe`",
                    "`??` deals with a failure, or gives a fallback when a `Maybe` is `none`.",
                );
                Self::error(place)
            }
        }
    }

    /// `call otherwise` and its cases, with any handlers it brings in with `include` (D37, Unio D40, D43, D186).
    fn otherwise(
        &mut self,
        call: &syntax::Expression,
        cases: &[syntax::OtherwiseCase],
        includes: &[syntax::HandlerUse],
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let checked = self.expression(call, wanted);
        let failable = self.is_failable(&checked);
        if !failable && !matches!(checked.kind, ExpressionKind::Error) {
            self.refuse_why(
                place,
                "`otherwise` follows something that can't fail",
                "`otherwise` handles each failure of the call before it.",
            );
        }
        let type_id = checked.type_id;
        let gives_nothing = self.program.types.resolve(type_id) == NOTHING;
        let mut checked_cases: Vec<(super::tree::FailureId, super::tree::Block)> = Vec::new();
        let mut shared = Vec::new();
        for using in includes {
            let Some(brought_in) = self.handler_cases(&checked, using, type_id, gives_nothing) else { continue };
            for (failure, body) in brought_in {
                if checked_cases.iter().any(|(earlier, _)| *earlier == failure) {
                    let name = self.program.failure_name(failure).to_owned();
                    // trace: Unio D43. Together the cases cover every failure, none twice.
                    self.refuse_why(
                        using.place,
                        format!("the failure `{name}` is handled twice"),
                        "an `otherwise` handles each failure once, with its own `case` or an included handler's.",
                    );
                }
                shared.push(failure);
                checked_cases.push((failure, body));
            }
        }
        for case in cases {
            let failure = self.failure(&case.failure.text);
            if checked_cases.iter().any(|(earlier, _)| *earlier == failure) {
                self.refuse_why(
                    case.failure.place,
                    format!("the failure `{}` is handled twice", case.failure.text),
                    "an `otherwise` has one `case` for each failure.",
                );
            }
            let body = self.case_block(case, type_id, gives_nothing, failable, Vec::new());
            checked_cases.push((failure, body));
        }
        if !failable {
            return Self::error(place);
        }
        Expression {
            kind: ExpressionKind::Otherwise { call: Box::new(checked), cases: checked_cases, shared },
            type_id,
            place,
        }
    }

    /// One `case` of an `otherwise` or a handler, checked where it's used: it ends in `instead`, `return`, `break` or
    /// `fail`, or simply ends after a call that gives nothing (Unio D40, D186). `before` are the lines that make the
    /// handler's parameters from its arguments, run before the case's own.
    pub(super) fn case_block(
        &mut self,
        case: &syntax::OtherwiseCase,
        type_id: TypeId,
        gives_nothing: bool,
        failable: bool,
        before: Vec<super::tree::Statement>,
    ) -> super::tree::Block {
        self.context.instead_types.push(type_id);
        let body = self.block(&case.body);
        self.context.instead_types.pop();
        let ends_with_instead =
            matches!(body.last().map(|last| &last.kind), Some(super::tree::StatementKind::Instead(_)));
        if gives_nothing && ends_with_instead {
            self.refuse_why(
                body.last().map_or(case.failure.place, |last| last.place),
                "this call gives nothing, so there's no value to give `instead`",
                "after a call that gives nothing, a case may just end.",
            );
        } else if !gives_nothing && !ends_with_instead && !Self::block_leaves(&body) && failable {
            // trace: Unio D40
            self.refuse_why(
                case.failure.place,
                format!("the case `{}` can reach its end without giving a value", case.failure.text),
                "each case ends with `instead` and a value, `return`, `break` or `fail`.",
            );
        }
        let mut whole = before;
        whole.extend(body);
        whole
    }

    /// A slide: `start |> step |> step |> step`, the same as the calls nested (Unio D5, D128).
    fn slide(
        &mut self,
        start: &syntax::Expression,
        steps: &[syntax::SlideStep],
        wanted: Option<TypeId>,
        place: Place,
    ) -> Expression {
        let start_is_call =
            matches!(&start.kind, syntax::ExpressionKind::Call { callee, .. } if !self.is_method(callee));
        let calls = steps.len() + usize::from(start_is_call);
        if calls < 3 {
            // trace: Unio D128
            self.refuse_why(
                place,
                format!("this slide is a chain of {calls}, and one or two calls are written as calls"),
                "a chain of three or more calls is a slide; a shorter one is written with `( )`, as in `show_msg(be.Text(n))`.",
            );
            return Self::error(place);
        }
        let mut nested = start.clone();
        for step in steps {
            let mut arguments = vec![syntax::Entry { label: None, value: nested }];
            arguments.extend(step.arguments.clone().unwrap_or_default());
            nested = syntax::Expression {
                id: u32::MAX,
                kind: syntax::ExpressionKind::Call { callee: Box::new(step.target.clone()), arguments },
                place: step.place,
            };
        }
        self.slide_depth += 1;
        let mut checked = self.expression(&nested, wanted);
        self.slide_depth -= 1;
        checked.place = place;
        checked
    }

    /// Whether a call's callee is a method on a value, as opposed to a procedure's name.
    pub(super) fn is_method(&self, callee: &syntax::Expression) -> bool {
        matches!(&callee.kind, syntax::ExpressionKind::Member { receiver, .. } if self.is_value(receiver))
    }
}

/// How a piece of joined text is written inside text: a text as it's written, or a name with its fields and facts in
/// braces (D75, D77).
fn filled(expression: &syntax::Expression) -> Option<String> {
    fn path(expression: &syntax::Expression) -> Option<String> {
        match &expression.kind {
            syntax::ExpressionKind::Name(name) => Some(name.clone()),
            syntax::ExpressionKind::Member { receiver, name } => Some(format!("{}.{}", path(receiver)?, name.text)),
            _ => None,
        }
    }
    match &expression.kind {
        syntax::ExpressionKind::Text(text) => Some(text.clone()),
        syntax::ExpressionKind::Group(inner) => filled(inner),
        _ => Some(format!("{{{}}}", path(expression)?)),
    }
}

/// A capitalised path, as `Event.hurt` or `Maybe.some`: a type and what's narrowed from it.
pub(super) fn type_path(expression: &syntax::Expression) -> Option<Vec<&syntax::Word>> {
    // The first word is rebuilt as a `Word` by its caller's lifetime, so the type's own name is found by walking.
    fn walk<'e>(expression: &'e syntax::Expression, path: &mut Vec<&'e syntax::Word>) -> bool {
        match &expression.kind {
            syntax::ExpressionKind::Member { receiver, name } => {
                if walk(receiver, path) {
                    path.push(name);
                    true
                } else {
                    false
                }
            }
            syntax::ExpressionKind::TypeName(_) => true,
            _ => false,
        }
    }
    let mut rest = Vec::new();
    if !walk(expression, &mut rest) {
        return None;
    }
    let mut root = expression;
    while let syntax::ExpressionKind::Member { receiver, .. } = &root.kind {
        root = receiver;
    }
    let syntax::ExpressionKind::TypeName(_) = &root.kind else { return None };
    let mut path = vec![type_word(root)?];
    path.extend(rest);
    Some(path)
}

/// The type's own name as a word. A `TypeName` expression holds its text and place, the same as a [`syntax::Word`].
fn type_word(expression: &syntax::Expression) -> Option<&syntax::Word> {
    match &expression.kind {
        syntax::ExpressionKind::TypeName(word) => Some(word),
        _ => None,
    }
}

/// A procedure's name with its narrowings, as `show_msg` and `same_line`.
pub(super) fn name_path(expression: &syntax::Expression) -> Option<Vec<String>> {
    match &expression.kind {
        syntax::ExpressionKind::Name(name) => Some(vec![name.clone()]),
        syntax::ExpressionKind::Member { receiver, name } => {
            let mut path = name_path(receiver)?;
            path.push(name.text.clone());
            Some(path)
        }
        _ => None,
    }
}

/// The unit after `in` in text's braces, as `secs`, `time::secs` or `px per secs`, placed where it's written (D122).
fn brace_unit(unit: &str, line: u32, column: u32) -> Option<syntax::UnitWritten> {
    let name = |text: &str, column: u32| -> Option<syntax::UnitName> {
        let word = |text: &str, column: u32| {
            let is_name = !text.is_empty()
                && text.starts_with(|first: char| first.is_ascii_lowercase())
                && text
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_');
            is_name.then(|| syntax::Word {
                text: text.to_owned(),
                place: Place::new(line, column, u32::try_from(text.len()).unwrap_or(1)),
            })
        };
        match text.split_once("::") {
            Some((group, name)) => Some(syntax::UnitName {
                group: Some(word(group, column)?),
                name: word(name, column + u32::try_from(group.len()).unwrap_or(0) + 2)?,
            }),
            None => Some(syntax::UnitName { group: None, name: word(text, column)? }),
        }
    };
    let place = Place::new(line, column, u32::try_from(unit.len()).unwrap_or(1));
    let (top, per) = match unit.split_once(" per ") {
        Some((top, per)) => {
            let per_column = column + u32::try_from(top.len()).unwrap_or(0) + 5;
            (name(top, column)?, Some(name(per, per_column)?))
        }
        None => (name(unit, column)?, None),
    };
    Some(syntax::UnitWritten { top, point: false, per, place })
}

/// A name and its fields and facts in text's braces, as `hero.hp`, placed where it's written.
fn brace_path(path: &str, place: Place) -> Option<syntax::Expression> {
    let mut column = place.column + 1;
    let mut built: Option<syntax::Expression> = None;
    for word in path.split('.') {
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
    built
}
