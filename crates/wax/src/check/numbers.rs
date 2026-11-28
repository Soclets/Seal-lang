//! `HugeWhole`, `Decimal` and `HugeDecimal` in a script (D115, D116, D119, D125 to D127): where a number becomes a
//! bigger type by itself, what a number written out becomes beside one, the operations each takes, and `be.`'s
//! conversions between them.

use super::Checker;
use super::tree::{Conversion, Expression, ExpressionKind, NumberConversion, NumberHow, NumberType, Rounding};
use super::types::{BOOL, DECIMAL, FLOAT, HUGE_DECIMAL, HUGE_WHOLE, TEXT, TypeId, WHOLE};
use crate::decimal::{Decimal, HugeDecimal, Number};
use crate::message::Place;
use crate::syntax::tree::{self as syntax, BinaryOperator, UnaryOperator};

/// The type a number type is.
fn type_of(number: NumberType) -> TypeId {
    match number {
        NumberType::Whole => WHOLE,
        NumberType::Float => FLOAT,
        NumberType::Text => TEXT,
        NumberType::Huge => HUGE_WHOLE,
        NumberType::Decimal => DECIMAL,
        NumberType::HugeDecimal => HUGE_DECIMAL,
    }
}

/// How a number type is written.
fn written(number: NumberType) -> &'static str {
    match number {
        NumberType::Whole => "Whole",
        NumberType::Float => "Float",
        NumberType::Text => "Text",
        NumberType::Huge => "HugeWhole",
        NumberType::Decimal => "Decimal",
        NumberType::HugeDecimal => "HugeDecimal",
    }
}

/// trace: D115, D116. Whether a number of one type always fits exactly in another, so it becomes one by itself.
fn widens(from: NumberType, to: NumberType) -> bool {
    matches!(
        (from, to),
        (NumberType::Whole, NumberType::Huge | NumberType::Decimal | NumberType::HugeDecimal)
            | (NumberType::Huge | NumberType::Decimal, NumberType::HugeDecimal)
    )
}

/// Whether an expression is a number written out, or one with a `-` in front.
pub(super) fn is_literal(expression: &syntax::Expression) -> bool {
    match &expression.kind {
        syntax::ExpressionKind::Whole(_) | syntax::ExpressionKind::Float { .. } => true,
        syntax::ExpressionKind::Unary { operator: UnaryOperator::Negate, operand } => {
            matches!(operand.kind, syntax::ExpressionKind::Whole(_) | syntax::ExpressionKind::Float { .. })
        }
        _ => false,
    }
}

/// A constant widened to a bigger type.
fn widened_constant(value: &ExpressionKind, to: NumberType) -> Option<Number> {
    let exact = match value {
        ExpressionKind::Whole(whole) => HugeDecimal::from_whole(*whole),
        ExpressionKind::Number(Number::Huge(big)) => HugeDecimal::from_big(big.clone()),
        ExpressionKind::Number(Number::Decimal(decimal)) => decimal.exact(),
        _ => return None,
    };
    Some(match to {
        NumberType::Huge => Number::Huge(exact.coefficient),
        NumberType::Decimal => Number::Decimal(Decimal::rounded(&exact).ok()?),
        NumberType::HugeDecimal => Number::HugeDecimal(exact),
        _ => return None,
    })
}

impl<'a> Checker<'a> {
    /// The number type a value's type is, when it's one of them, or text.
    pub(super) fn number_type(&mut self, type_id: TypeId) -> Option<NumberType> {
        match self.program.types.resolve(type_id) {
            WHOLE => Some(NumberType::Whole),
            FLOAT => Some(NumberType::Float),
            TEXT => Some(NumberType::Text),
            HUGE_WHOLE => Some(NumberType::Huge),
            DECIMAL => Some(NumberType::Decimal),
            HUGE_DECIMAL => Some(NumberType::HugeDecimal),
            _ => None,
        }
    }

    /// Which of the three new number types a type is, when it's one.
    pub(super) fn big_number(&mut self, type_id: TypeId) -> Option<NumberType> {
        self.number_type(type_id)
            .filter(|number| matches!(number, NumberType::Huge | NumberType::Decimal | NumberType::HugeDecimal))
    }

    /// A value made the bigger number type it fits in exactly (D115, D116).
    fn widen(value: Expression, from: NumberType, to: NumberType) -> Expression {
        let place = value.place;
        if let Some(constant) = widened_constant(&value.kind, to) {
            return Expression { kind: ExpressionKind::Number(constant), type_id: type_of(to), place };
        }
        Expression {
            kind: ExpressionKind::Convert {
                conversion: Conversion::Number(NumberConversion { from, to, how: NumberHow::Exact }),
                value: Box::new(value),
            },
            type_id: type_of(to),
            place,
        }
    }

    /// trace: D115, D116. A value landing where a bigger number type is wanted becomes one, where it always fits.
    pub(super) fn land_number(&mut self, value: Expression, wanted: TypeId) -> Expression {
        let Some(to) = self.big_number(wanted) else { return value };
        let Some(from) = self.number_type(value.type_id) else { return value };
        if widens(from, to) { Self::widen(value, from, to) } else { value }
    }

    /// trace: D115, D116, D119. A number written out where a `HugeWhole`, `Decimal` or `HugeDecimal` is wanted is
    /// one, exactly as written; nothing when none is wanted, or a `HugeWhole` is wanted and it has a point.
    pub(super) fn number_literal(
        &mut self,
        literal: &syntax::ExpressionKind<'a>,
        negative: bool,
        wanted: Option<TypeId>,
        place: Place,
    ) -> Option<Expression> {
        let to = self.big_number(wanted?)?;
        let (text, approximate) = match literal {
            syntax::ExpressionKind::Whole(digits) => (*digits, false),
            syntax::ExpressionKind::Float { .. } if to == NumberType::Huge => return None,
            syntax::ExpressionKind::Float { text, approximate } => (*text, *approximate),
            _ => return None,
        };
        let name = written(to);
        if approximate {
            self.refuse_why(
                place,
                format!("a `{name}` holds exactly what's written, so its number takes no `~`"),
                "a `~` marks a `Float` that's only close to what's written; a decimal counts in tens, so `0.1` is exact.",
            );
            return Some(Self::error(place));
        }
        let Some(exact) = HugeDecimal::read(text) else { return Some(Self::error(place)) };
        let exact = if negative { exact.negated() } else { exact };
        let number = match to {
            NumberType::Huge => Number::Huge(exact.coefficient),
            NumberType::Decimal => match Decimal::rounded(&exact) {
                Ok(decimal) if decimal.exact().compare(&exact) == std::cmp::Ordering::Equal => Number::Decimal(decimal),
                _ => {
                    self.refuse_why(
                        place,
                        format!("`{text}` has more digits than a `Decimal` keeps"),
                        "a `Decimal` keeps 34 digits; a `HugeDecimal` keeps any number of them.",
                    );
                    return Some(Self::error(place));
                }
            },
            _ => Number::HugeDecimal(exact),
        };
        Some(Expression { kind: ExpressionKind::Number(number), type_id: type_of(to), place })
    }

    /// trace: D115, D116, D127. Maths and comparisons where one side is a `HugeWhole`, `Decimal` or `HugeDecimal`:
    /// the other side becomes its type when it fits exactly, and each operation is one the type takes.
    pub(super) fn number_binary(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: Expression,
        operator_place: Place,
        place: Place,
    ) -> Expression {
        let quoted = format!("`{}`", operator.text());
        let (Some(left_number), Some(right_number)) = (self.number_type(left.type_id), self.number_type(right.type_id))
        else {
            let (left_type, right_type) = (self.describe(left.type_id), self.describe(right.type_id));
            self.refuse(operator_place, format!("{quoted} has {left_type} on its left, but {right_type} on its right"));
            return Self::error(place);
        };
        let target = if left_number == right_number || widens(right_number, left_number) {
            left_number
        } else if widens(left_number, right_number) {
            right_number
        } else {
            let (left_type, right_type) = (written(left_number), written(right_number));
            // trace: D116, Unio D162
            self.refuse_why(
                operator_place,
                format!("{quoted} can't mix a `{left_type}` and a `{right_type}`"),
                "a number becomes a bigger type by itself only where it always fits; `be.` converts the other ways, and a `Float` mixes with none of these.",
            );
            return Self::error(place);
        };
        let left = if left_number == target { left } else { Self::widen(left, left_number, target) };
        let right = if right_number == target { right } else { Self::widen(right, right_number, target) };
        let type_id = if operator.is_comparison() { BOOL } else { type_of(target) };
        Expression {
            kind: ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right) },
            type_id,
            place,
        }
    }

    /// trace: D127. `**` on a `HugeWhole`, `Decimal` or `HugeDecimal`: a whole power.
    pub(super) fn number_power(
        &mut self,
        left: Expression,
        right: &syntax::Expression<'a>,
        operator_place: Place,
        place: Place,
    ) -> Expression {
        let right = self.expression(right, Some(WHOLE));
        if !self.program.types.unify(right.type_id, WHOLE) {
            let found = self.describe(right.type_id);
            self.refuse_why(
                operator_place,
                format!("`**` on a `{}` takes a whole power, but this is {found}", self.describe(left.type_id)),
                "a `HugeWhole`, `Decimal` or `HugeDecimal` is raised to a `Whole` power.",
            );
            return Self::error(place);
        }
        if let ExpressionKind::Whole(power) = right.kind
            && power < 0
            && self.big_number(left.type_id) == Some(NumberType::Huge)
        {
            // trace: D108, D127. A negative power of a whole number isn't whole.
            self.refuse_why(
                right.place,
                "a `HugeWhole` to a negative power isn't a whole number",
                "a negative power is one over the number, as `2 ** -3` is `1 / 8`; a `HugeDecimal` holds it when its digits end, as `0.125` does.",
            );
            return Self::error(place);
        }
        let type_id = left.type_id;
        Expression {
            kind: ExpressionKind::Binary {
                operator: BinaryOperator::Power,
                left: Box::new(left),
                right: Box::new(right),
            },
            type_id,
            place,
        }
    }

    /// trace: D115, D116, D126. `be.HugeWhole`, `be.Decimal` and `be.HugeDecimal`, with their narrowings.
    #[allow(clippy::too_many_lines)] // one case per pair of types
    pub(super) fn convert_to_number(
        &mut self,
        path: &[&str],
        arguments: &[syntax::Entry<'a>],
        place: Place,
    ) -> Expression {
        let full = path.join(".");
        let narrowing: Vec<&str> = path.iter().skip(1).copied().collect();
        let to = match narrowing.first().copied() {
            Some("HugeWhole") => NumberType::Huge,
            Some("Decimal") => NumberType::Decimal,
            _ => NumberType::HugeDecimal,
        };
        let name = written(to);
        let how = match (to, narrowing.get(1..).unwrap_or(&[])) {
            (_, []) => NumberHow::Exact,
            (NumberType::Decimal, ["shown"]) => NumberHow::Shown,
            (NumberType::Decimal, ["nearest"]) => NumberHow::Nearest,
            (NumberType::Huge, ["floor"]) => NumberHow::Round(Rounding::Floor),
            (NumberType::Huge, ["trunc"]) => NumberHow::Round(Rounding::Trunc),
            (NumberType::Huge, ["ceil"]) => NumberHow::Round(Rounding::Ceil),
            (NumberType::Huge, ["nearest"]) => NumberHow::Round(Rounding::Nearest),
            (NumberType::Huge, ["nearest", "even"]) => NumberHow::Round(Rounding::NearestEven),
            _ => {
                self.refuse_why(
                    place,
                    format!("there's no `{full}`"),
                    "`be.HugeWhole` takes `.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` for a decimal, and `be.Decimal` takes `.shown` and `.nearest` for a `Float`.",
                );
                return Self::error(place);
            }
        };
        let Some(argument) = self.single_argument(&full, path, path.len(), arguments, place) else {
            return Self::error(place);
        };
        let value = self.expression(argument, None);
        if matches!(value.kind, ExpressionKind::Error) {
            return Self::error(place);
        }
        let Some(from) = self.number_type(value.type_id) else {
            if self.program.types.is_open(value.type_id) {
                self.program.types.unify(value.type_id, TEXT);
                return self.number_conversion(value, NumberType::Text, to, how, place);
            }
            let found = self.describe(value.type_id);
            self.refuse_why(
                argument.place,
                format!("there's no conversion from {found} to a `{name}`"),
                "a number type converts from text and from the other number types.",
            );
            return Self::error(place);
        };
        if from == to {
            // trace: Unio D22
            self.refuse_why(
                argument.place,
                format!("this is already a `{name}`, so `be.{name}` does nothing"),
                "it gives back the same value, so the `be.` can go.",
            );
            return Self::error(place);
        }
        let fraction = matches!(from, NumberType::Decimal | NumberType::HugeDecimal | NumberType::Float);
        match (from, to, how) {
            (NumberType::Float, NumberType::Huge | NumberType::HugeDecimal, _) => {
                self.not_designed(place, format!("converting a `Float` to a `{name}`"));
                Self::error(place)
            }
            (NumberType::Float, NumberType::Decimal, NumberHow::Exact) => {
                // trace: D126, Unio D178. Which `Decimal` a `Float` becomes is said.
                self.refuse_why(
                    place,
                    "`be.Decimal` on a `Float` needs to say which `Decimal` it becomes",
                    "`be.Decimal.shown` takes the number the `Float` shows, so `~0.1` gives `0.1`; `be.Decimal.nearest` takes the `Decimal` nearest the value it holds.",
                );
                Self::error(place)
            }
            (NumberType::Float, NumberType::Decimal, _) => self.number_conversion(value, from, to, how, place),
            (_, _, NumberHow::Shown | NumberHow::Nearest) => {
                self.refuse_why(
                    place,
                    format!("`{full}` takes a `Float`, but this is a `{}`", written(from)),
                    "`.shown` and `.nearest` say which `Decimal` a `Float` becomes.",
                );
                Self::error(place)
            }
            (NumberType::Decimal | NumberType::HugeDecimal, NumberType::Huge, NumberHow::Exact) => {
                // trace: D126
                self.refuse_why(
                    place,
                    format!("`be.HugeWhole` on a `{}` needs to say how the fraction goes", written(from)),
                    "the ways are `be.HugeWhole.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even`.",
                );
                Self::error(place)
            }
            (_, NumberType::Huge, NumberHow::Round(_)) if !fraction => {
                self.refuse_why(
                    place,
                    format!("`{full}` rounds a fraction, and a `{}` has none", written(from)),
                    "`be.HugeWhole` converts a whole number or text with nothing to round.",
                );
                Self::error(place)
            }
            _ => self.number_conversion(value, from, to, how, place),
        }
    }

    /// trace: D126. `be.Whole`, `be.Float` and `be.Text` on a `HugeWhole`, `Decimal` or `HugeDecimal`.
    pub(super) fn convert_from_number(
        &mut self,
        value: Expression,
        from: NumberType,
        full: &str,
        to: NumberType,
        rounding: Option<Rounding>,
        place: Place,
    ) -> Expression {
        let source = written(from);
        match (to, rounding) {
            (NumberType::Text, _) => Expression {
                kind: ExpressionKind::Convert { conversion: Conversion::ToText, value: Box::new(value) },
                type_id: TEXT,
                place,
            },
            (NumberType::Float, _) => self.number_conversion(value, from, to, NumberHow::Exact, place),
            (_, None) if from != NumberType::Huge => {
                self.refuse_why(
                    place,
                    format!("`be.Whole` on a `{source}` needs to say how the fraction goes"),
                    "the ways are `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.",
                );
                Self::error(place)
            }
            (_, Some(_)) if from == NumberType::Huge => {
                self.refuse_why(
                    place,
                    format!("`{full}` rounds a fraction, and a `HugeWhole` has none"),
                    "`be.Whole` converts a `HugeWhole`, failing with `too_big` when it doesn't fit.",
                );
                Self::error(place)
            }
            (_, rounding) => {
                let how = rounding.map_or(NumberHow::Exact, NumberHow::Round);
                self.number_conversion(value, from, NumberType::Whole, how, place)
            }
        }
    }

    fn number_conversion(
        &mut self,
        value: Expression,
        from: NumberType,
        to: NumberType,
        how: NumberHow,
        place: Place,
    ) -> Expression {
        let conversion = NumberConversion { from, to, how };
        if let Some(failure) = conversion.failure() {
            self.failure(failure);
        }
        Expression {
            kind: ExpressionKind::Convert { conversion: Conversion::Number(conversion), value: Box::new(value) },
            type_id: type_of(to),
            place,
        }
    }

    /// trace: D127. Whether `sqrt` and `lerp` take a value of this number type.
    pub(super) fn takes_decimal(&mut self, type_id: TypeId, lerp: bool) -> bool {
        match self.big_number(type_id) {
            Some(NumberType::Decimal) => true,
            Some(NumberType::HugeDecimal) => !lerp,
            _ => false,
        }
    }
}
