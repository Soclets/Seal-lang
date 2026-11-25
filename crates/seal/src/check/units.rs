//! Units: optional, part of a number's type, written bare after a number, converting by themselves where nothing is
//! lost, and worked out through `*` and `/` (D111 to D113, D120 to D123).
//!
//! A value with a unit is a bare number in that unit when the script runs; every conversion is worked out here and
//! becomes a multiplication by an exact whole number, or a scaling rounded once.

use super::Checker;
use super::tree::{Conversion, Expression, ExpressionKind, NumberConversion, NumberHow, NumberType};
use super::types::{FLOAT, HUGE_WHOLE, Size, Type, TypeId, Unit, UnitGroupType, UnitMemberType, UnitRef, WHOLE};
use crate::big::Big;
use crate::decimal::Number;
use crate::message::{Message, Place};
use crate::syntax::tree::{self as syntax, BinaryOperator};

/// The number of `to` in one `from`, when both have a fixed size in one group.
fn ratio(groups: &[UnitGroupType], from: UnitRef, to: UnitRef) -> Option<Size> {
    if from == to {
        return Some(Size::ONE);
    }
    if from.group != to.group {
        return None;
    }
    let group = groups.get(usize::from(from.group))?;
    let from_size = group.members.get(usize::from(from.member))?.size?;
    let to_size = group.members.get(usize::from(to.member))?.size?;
    from_size.over(to_size)
}

/// Whether a unit is smaller than another of its group: the finer of the two.
fn finer(groups: &[UnitGroupType], first: UnitRef, second: UnitRef) -> bool {
    ratio(groups, first, second).is_some_and(|size| size.rough() < 1.0)
}

impl<'a> Checker<'a> {
    /// trace: D123. Each `group.unit`: exactly one member marked `base`, and each other so many of an earlier one,
    /// exactly; one written with `~` has no fixed size, so it never converts by itself.
    pub(super) fn declare_unit_groups(&mut self, script: &'a syntax::Script) {
        // trace: D111, D123. The host's groups first, checked when it declared them.
        for group in self.host_units {
            let mut members: Vec<UnitMemberType> = Vec::new();
            for (name, size) in &group.members {
                let size = match size.split_once(' ') {
                    None => Some(Size::ONE),
                    Some((amount, earlier)) => {
                        let earlier = members.iter().find(|known| known.name == earlier).and_then(|known| known.size);
                        match (earlier, Size::written(amount)) {
                            (Some(earlier), Some(amount)) => earlier.times(amount),
                            _ => None,
                        }
                    }
                };
                members.push(UnitMemberType { name: name.clone(), size });
            }
            self.program.types.unit_groups.push(UnitGroupType { name: group.name.clone(), members });
        }
        for item in &script.items {
            let syntax::Item::UnitGroup(group) = item else { continue };
            if self.program.types.unit_groups.iter().any(|known| known.name == group.name.text) {
                self.refuse_why(
                    group.name.place,
                    format!("the unit group `{}` is declared twice, or is one of Seal's own", group.name.text),
                    "each unit group is declared once; `time`, `angle` and `distance` are Seal's own.",
                );
                continue;
            }
            let bases = group.members.iter().filter(|member| member.size.is_none()).count();
            if bases != 1 {
                self.refuse_why(
                    group.place,
                    format!("`{}` has {bases} units marked `base`, where it needs exactly one", group.name.text),
                    "a unit group's base is the unit every other one is counted in, so there's exactly one.",
                );
                continue;
            }
            let mut members: Vec<UnitMemberType> = Vec::new();
            for member in &group.members {
                if members.iter().any(|known| known.name == member.name.text) {
                    self.refuse_why(
                        member.name.place,
                        format!("the unit `::{}` is listed twice in `{}`", member.name.text, group.name.text),
                        "each unit in a group has a name of its own.",
                    );
                    continue;
                }
                let size = match &member.size {
                    None => Some(Size::ONE),
                    Some((amount, unit)) => {
                        let Some(earlier) = members.iter().find(|known| known.name == unit.name.text) else {
                            self.refuse_why(
                                unit.name.place,
                                format!("`{}` isn't a unit listed above in `{}`", unit.name.text, group.name.text),
                                "each unit is counted in one listed before it, so every size is known.",
                            );
                            continue;
                        };
                        self.amount_times(amount, earlier.size)
                    }
                };
                members.push(UnitMemberType { name: member.name.text.clone(), size });
            }
            self.program.types.unit_groups.push(UnitGroupType { name: group.name.text.clone(), members });
        }
    }

    /// An amount as written in a unit's definition times an earlier unit's size: the number as written, marked
    /// inexact when it's written with `~` (D123).
    fn amount_times(&mut self, amount: &syntax::Expression, size: Option<Size>) -> Option<Size> {
        let size = size?;
        let (amount_numerator, amount_denominator, inexact) = match &amount.kind {
            syntax::ExpressionKind::Whole(digits) => (digits.replace('_', "").parse::<u128>().ok()?, 1, false),
            syntax::ExpressionKind::Float { text, approximate } => {
                let text = text.replace('_', "");
                let text = text.trim_start_matches('~');
                let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
                let digits = format!("{whole}{fraction}").parse::<u128>().ok()?;
                (digits, 10u128.checked_pow(u32::try_from(fraction.len()).ok()?)?, *approximate)
            }
            _ => {
                self.refuse_why(
                    amount.place,
                    "a unit's size is a number written out",
                    "a unit is so many of an earlier one, as `::dollars = 100 cents`.",
                );
                return None;
            }
        };
        if amount_numerator == 0 {
            self.refuse(amount.place, "a unit's size can't be 0");
            return None;
        }
        let amount = Size { numerator: amount_numerator, denominator: amount_denominator, pi: 0, inexact };
        size.times(amount.over(Size::ONE)?)
    }

    /// A unit's name as written, found among the groups: its group is needed only when two groups have a unit of
    /// the same name (D112).
    fn unit_ref(&mut self, name: &syntax::UnitName) -> Option<UnitRef> {
        let groups = &self.program.types.unit_groups;
        let mut found: Vec<UnitRef> = Vec::new();
        for (group_index, group) in groups.iter().enumerate() {
            if name.group.as_ref().is_some_and(|written| written.text != group.name) {
                continue;
            }
            if let Some(member) = group.members.iter().position(|member| member.name == name.name.text) {
                found.push(UnitRef {
                    group: u16::try_from(group_index).unwrap_or(u16::MAX),
                    member: u16::try_from(member).unwrap_or(u16::MAX),
                });
            }
        }
        match found.as_slice() {
            [only] => Some(*only),
            [] => {
                let what = match &name.group {
                    Some(group) if !groups.iter().any(|known| known.name == group.text) => {
                        format!("there's no unit group named `{}`", group.text)
                    }
                    Some(group) => format!("the unit group `{}` has no unit `{}`", group.text, name.name.text),
                    None => format!("there's no unit named `{}`", name.name.text),
                };
                self.refuse_why(
                    name.name.place,
                    what,
                    "Seal's units are `ms`, `secs`, `mins`, `frames`, `degs`, `rads`, `turns`, `px` and `tiles`; a script declares its own with `group.unit`.",
                );
                None
            }
            several => {
                let groups: Vec<String> = several
                    .iter()
                    .filter_map(|unit| self.program.types.unit_groups.get(usize::from(unit.group)))
                    .map(|group| format!("`{}::{}`", group.name, name.name.text))
                    .collect();
                // trace: D112. The group settles a clash.
                self.refuse_why(
                    name.name.place,
                    format!("`{}` could be {}", name.name.text, groups.join(" or ")),
                    "when two groups have a unit of the same name, the group is written before it, with `::`.",
                );
                None
            }
        }
    }

    /// A unit as written, found: `secs`, `px.point`, `px per secs` (D112, D120, D121).
    pub(super) fn resolve_unit(&mut self, written: &syntax::UnitWritten) -> Option<Unit> {
        let top = self.unit_ref(&written.top)?;
        let per = match &written.per {
            Some(per) => Some(self.unit_ref(per)?),
            None => None,
        };
        if written.point && per.is_some() {
            self.refuse_why(
                written.place,
                "a rate isn't a point",
                "`.point` marks a place counted from a fixed one, as `px.point`; a rate, as `px per secs`, is an amount.",
            );
            return None;
        }
        Some(Unit { top, per, point: written.point })
    }

    /// A unit a host wrote for a door's value (D111), found as a script's is; a refusal names the host's unit.
    pub(super) fn host_unit(&mut self, text: &str) -> Option<Unit> {
        let name = |text: &str| syntax::UnitName {
            group: text
                .split_once("::")
                .map(|(group, _)| syntax::Word { text: group.to_owned(), place: Place::default() }),
            name: syntax::Word {
                text: text.split_once("::").map_or(text, |(_, name)| name).to_owned(),
                place: Place::default(),
            },
        };
        let (top, per) = text.split_once(" per ").map_or((text, None), |(top, per)| (top, Some(per)));
        let (top, point) = top.strip_suffix(".point").map_or((top, false), |top| (top, true));
        let written = syntax::UnitWritten { top: name(top), point, per: per.map(name), place: Place::default() };
        let before = self.messages.len();
        let unit = self.resolve_unit(&written);
        for message in self.messages.iter_mut().skip(before) {
            message.file = format!("the host's unit `{text}`");
        }
        unit
    }

    /// The type of a `Whole` or `Float` with a unit.
    pub(super) fn measured_type(&mut self, base: TypeId, unit: Unit) -> TypeId {
        self.program.types.intern(Type::Measured { base, unit })
    }

    /// How a value in one unit becomes a value in another of the same groups: the factor to multiply by, as a
    /// fraction, when there's a fixed one (D120).
    fn factor(&self, from: Unit, to: Unit) -> Option<Size> {
        if from.point != to.point {
            return None;
        }
        let groups = &self.program.types.unit_groups;
        let top = ratio(groups, from.top, to.top)?;
        let per = match (from.per, to.per) {
            (None, None) => Size::ONE,
            // Per one `from.per` is per `ratio` of `to.per`, so the value is divided by that ratio.
            (Some(from_per), Some(to_per)) => ratio(groups, from_per, to_per)?,
            _ => return None,
        };
        top.over(per)
    }

    /// A value moved into another unit of its groups, or `Err` with why it can't be (D120): into a `Whole` only by
    /// a whole number of times, into a `Float` by any fixed fraction, rounded once.
    fn converted(&mut self, value: Expression, to: Unit, to_base: TypeId) -> Result<Expression, String> {
        let types = &self.program.types;
        let Some((from_base, from)) = types.measured(value.type_id) else {
            return Err("it has no unit".to_owned());
        };
        let target = self.measured_type(to_base, to);
        if from == to && from_base == to_base {
            return Ok(Expression { type_id: target, ..value });
        }
        if from_base == HUGE_WHOLE || to_base == HUGE_WHOLE {
            return self.converted_huge(value, (from_base, from), to, target);
        }
        let Some(size) = self.factor(from, to) else {
            let (from_text, to_text) = (self.program.types.unit_text(from), self.program.types.unit_text(to));
            return Err(format!("`{from_text}` doesn't convert to `{to_text}` by itself"));
        };
        let to_float = to_base == FLOAT;
        if from_base == FLOAT && !to_float {
            return Err("a `Float` doesn't become a `Whole` by itself".to_owned());
        }
        if from_base == WHOLE && to_base == WHOLE && !size.is_whole() {
            let (from_text, to_text) = (self.program.types.unit_text(from), self.program.types.unit_text(to));
            return Err(format!("`{from_text}` in `{to_text}` isn't always a whole number"));
        }
        if from == to && from_base != to_base {
            // A `Whole` and a `Float` of the same unit don't mix quietly, as plain ones don't (D16).
            return Err("a `Whole` doesn't become a `Float` by itself".to_owned());
        }
        let place = value.place;
        Ok(Expression {
            kind: ExpressionKind::Convert {
                conversion: Conversion::Scale { size, from_whole: from_base == WHOLE, to_float },
                value: Box::new(value),
            },
            type_id: target,
            place,
        })
    }

    /// trace: D115, D120. A value with a unit becoming a `HugeWhole` in another unit: a `Whole` widens first, and the
    /// unit changes only by a whole number of times, so nothing is lost.
    fn converted_huge(
        &mut self,
        value: Expression,
        (from_base, from): (TypeId, Unit),
        to: Unit,
        target: TypeId,
    ) -> Result<Expression, String> {
        if self.program.types.measured(target).map(|(base, _)| base) != Some(HUGE_WHOLE) {
            return Err("a `HugeWhole` becomes another type only with `be.`".to_owned());
        }
        if from_base != WHOLE && from_base != HUGE_WHOLE {
            return Err("a `Float` doesn't become a `HugeWhole` by itself".to_owned());
        }
        let value = if from_base == WHOLE { self.widen_measured(value) } else { value };
        if from == to {
            return Ok(Expression { type_id: target, ..value });
        }
        let Some(size) = self.factor(from, to) else {
            let (from_text, to_text) = (self.program.types.unit_text(from), self.program.types.unit_text(to));
            return Err(format!("`{from_text}` doesn't convert to `{to_text}` by itself"));
        };
        if !size.is_whole() {
            let (from_text, to_text) = (self.program.types.unit_text(from), self.program.types.unit_text(to));
            return Err(format!("`{from_text}` in `{to_text}` isn't always a whole number"));
        }
        let place = value.place;
        let factor = Expression {
            kind: ExpressionKind::Number(Number::Huge(Big::from_u128(size.numerator))),
            type_id: HUGE_WHOLE,
            place,
        };
        Ok(Expression {
            kind: ExpressionKind::Binary {
                operator: BinaryOperator::Multiply,
                left: Box::new(value),
                right: Box::new(factor),
            },
            type_id: target,
            place,
        })
    }

    /// A `Whole`, with a unit or without, made a `HugeWhole` with the same unit (D115).
    fn widen_measured(&mut self, value: Expression) -> Expression {
        let place = value.place;
        let type_id = match self.program.types.measured(value.type_id) {
            Some((_, unit)) => self.measured_type(HUGE_WHOLE, unit),
            None => HUGE_WHOLE,
        };
        let kind = match value.kind {
            ExpressionKind::Whole(whole) => ExpressionKind::Number(Number::Huge(Big::from_i64(whole))),
            _ => ExpressionKind::Convert {
                conversion: Conversion::Number(NumberConversion {
                    from: NumberType::Whole,
                    to: NumberType::Huge,
                    how: NumberHow::Exact,
                }),
                value: Box::new(value),
            },
        };
        Expression { kind, type_id, place }
    }

    /// trace: D111, D120. A value landing where a unit is wanted: converted when it's another unit of the group and
    /// nothing is lost; a plain number where a unit is wanted is refused, with the unit added as the fix.
    pub(super) fn land_unit(&mut self, value: Expression, wanted: TypeId) -> Expression {
        if matches!(value.kind, ExpressionKind::Error) {
            return value;
        }
        let wanted_measured = self.program.types.measured(wanted);
        let found_measured = self.program.types.measured(value.type_id);
        match (wanted_measured, found_measured) {
            (Some((base, unit)), Some(_)) => match self.converted(value.clone(), unit, base) {
                Ok(converted) => converted,
                Err(why) => {
                    if self.program.types.resolve(value.type_id) != self.program.types.resolve(wanted) {
                        let (found, wanted_text) = (self.describe(value.type_id), self.describe(wanted));
                        self.refuse_why(
                            value.place,
                            format!("this is {found}, but {wanted_text} goes here, and {why}"),
                            "units of one group convert by themselves only where nothing is lost: into a `Whole`, a finer unit; into a `Float`, any unit with a fixed size.",
                        );
                        return Self::error(value.place);
                    }
                    value
                }
            },
            (Some((base, unit)), None) => {
                let plain = self.program.types.resolve(value.type_id);
                if plain != WHOLE && plain != FLOAT && !self.program.types.is_open(plain) {
                    return value;
                }
                if self.program.types.is_open(plain) {
                    return value;
                }
                let unit_text = self.program.types.unit_text(unit);
                let mut message = Message::refused(value.place, format!("this plain number goes where `{unit_text}` is wanted"))
                    .with_why("where something takes a unit, the number says it, so seconds and milliseconds, or degrees and radians, can't be mixed up.");
                // A whole number written where a `Float` with a unit goes gets its `.0` in the fix too (D16).
                let point = if plain == WHOLE && base == FLOAT { ".0" } else { "" };
                if matches!(value.kind, ExpressionKind::Whole(_) | ExpressionKind::Float(_))
                    && (plain == base || !point.is_empty())
                {
                    let line: Vec<char> = self.source_line(value.place.line).chars().collect();
                    let end = (value.place.column + value.place.width).saturating_sub(1) as usize;
                    if end <= line.len() {
                        let fixed: String = line.get(..end).unwrap_or(&[]).iter().collect::<String>()
                            + point
                            + " "
                            + &unit_text
                            + &line.get(end..).unwrap_or(&[]).iter().collect::<String>();
                        message = message.with_fix(fixed);
                    }
                }
                self.messages.push(message);
                Self::error(value.place)
            }
            (None, Some((_, unit))) => {
                let plain = self.program.types.resolve(wanted);
                if plain == WHOLE || plain == FLOAT {
                    let unit_text = self.program.types.unit_text(unit);
                    self.refuse_why(
                        value.place,
                        format!("this has a unit, `{unit_text}`, where a plain number goes"),
                        &format!("a value with a unit gives its plain number when it's measured in one, as `value in {unit_text}`."),
                    );
                    return Self::error(value.place);
                }
                value
            }
            (None, None) => value,
        }
    }

    /// trace: D112. A number written with its unit, as `2 secs` or `120.0 px per secs`.
    pub(super) fn measured_literal(
        &mut self,
        number: &syntax::Expression,
        unit: &syntax::UnitWritten,
        place: Place,
    ) -> Expression {
        let checked = self.expression(number, None);
        let Some(unit) = self.resolve_unit(unit) else { return Self::error(place) };
        let base = self.program.types.resolve(checked.type_id);
        if base != WHOLE && base != FLOAT {
            return Self::error(place);
        }
        let type_id = self.measured_type(base, unit);
        Expression { type_id, place, ..checked }
    }

    /// trace: D122. `value in unit`: the plain number a value is, measured in a unit of its own group. It's a
    /// `Whole` when the count is always whole, and a `Float` otherwise.
    pub(super) fn measure_in(
        &mut self,
        value: &syntax::Expression,
        unit: &syntax::UnitWritten,
        place: Place,
    ) -> Expression {
        let checked = self.expression(value, None);
        let Some(target) = self.resolve_unit(unit) else { return Self::error(place) };
        let Some((base, from)) = self.program.types.measured(checked.type_id) else {
            if !self.program.types.is_error(checked.type_id) {
                let found = self.describe(checked.type_id);
                self.refuse_why(
                    checked.place,
                    format!("`in` measures a value with a unit, but this is {found}"),
                    "`value in unit` gives the plain number a value with a unit is, as `total in secs`.",
                );
            }
            return Self::error(place);
        };
        let target = Unit { point: from.point, ..target };
        let Some(size) = self.factor(from, target) else {
            let (from_text, to_text) = (self.program.types.unit_text(from), self.program.types.unit_text(target));
            self.refuse_why(
                unit.place,
                format!("a value in `{from_text}` isn't measured in `{to_text}`"),
                "only a unit of the value's own group measures it, and only one with a fixed size, which `frames` and `tiles` haven't.",
            );
            return Self::error(place);
        };
        if base == HUGE_WHOLE {
            // trace: D115. A `HugeWhole` is measured in a finer unit by a whole number of times.
            if !size.is_whole() {
                self.not_designed(unit.place, "a `HugeWhole` measured in a unit it isn't always a whole number of");
                return Self::error(place);
            }
            let factor = Expression {
                kind: ExpressionKind::Number(Number::Huge(Big::from_u128(size.numerator))),
                type_id: HUGE_WHOLE,
                place,
            };
            let value = Expression { type_id: HUGE_WHOLE, ..checked };
            return Expression {
                kind: ExpressionKind::Binary {
                    operator: BinaryOperator::Multiply,
                    left: Box::new(value),
                    right: Box::new(factor),
                },
                type_id: HUGE_WHOLE,
                place,
            };
        }
        let whole = base == WHOLE && size.is_whole();
        let plain = if whole { WHOLE } else { FLOAT };
        if size == Size::ONE && base == plain {
            return Expression { type_id: plain, place, ..checked };
        }
        Expression {
            kind: ExpressionKind::Convert {
                conversion: Conversion::Scale { size, from_whole: base == WHOLE, to_float: !whole },
                value: Box::new(checked),
            },
            type_id: plain,
            place,
        }
    }

    /// Two values with units brought to one unit for adding, subtracting or comparing: the finer of the two, and
    /// for a rate the coarser per, so `Whole`s stay exact (D120).
    fn common(
        &mut self,
        left: Expression,
        right: Expression,
        operator: &str,
    ) -> Option<(Expression, Expression, Unit)> {
        let (Some((left_base, left_unit)), Some((right_base, right_unit))) =
            (self.program.types.measured(left.type_id), self.program.types.measured(right.type_id))
        else {
            return None;
        };
        let groups = &self.program.types.unit_groups;
        let same_groups = left_unit.top.group == right_unit.top.group
            && left_unit.per.map(|unit| unit.group) == right_unit.per.map(|unit| unit.group);
        if !same_groups || left_base != right_base {
            let (left_text, right_text) = (self.describe(left.type_id), self.describe(right.type_id));
            self.refuse_why(
                right.place,
                format!("`{operator}` has {left_text} on its left, but {right_text} on its right"),
                "units of different groups, as seconds and pixels, don't go together, and a `Whole` and a `Float` don't mix quietly.",
            );
            return None;
        }
        let top = if finer(groups, right_unit.top, left_unit.top) { right_unit.top } else { left_unit.top };
        let per = match (left_unit.per, right_unit.per) {
            (Some(left_per), Some(right_per)) => {
                Some(if finer(groups, left_per, right_per) { right_per } else { left_per })
            }
            (per, _) => per,
        };
        let point = left_unit.point;
        let mut results = Vec::new();
        for value in [left, right] {
            let own = self.program.types.measured(value.type_id).map_or(left_unit, |(_, unit)| unit);
            let place = value.place;
            match self.converted(value, Unit { top, per, point: own.point }, left_base) {
                Ok(converted) => results.push(converted),
                Err(why) => {
                    self.refuse_why(
                        place,
                        format!("this can't be brought to the other side's unit for `{operator}`: {why}"),
                        "units of one group combine where they convert exactly, in the finer unit.",
                    );
                    return None;
                }
            }
        }
        let right = results.pop()?;
        let left = results.pop()?;
        Some((left, right, Unit { top, per, point }))
    }

    /// trace: D111, D120, D121. Maths and comparing with values that have units: the same group adds and compares
    /// in the finer unit; a point and an amount make a point, and two points' difference is an amount; a number
    /// scales an amount; a rate times its own unit, or an amount divided by another of a different group, works the
    /// units out. Anything whose unit has no written form stops as not designed yet.
    #[allow(clippy::too_many_lines)] // one case per operator, each with its own rules
    pub(super) fn measured_binary(
        &mut self,
        operator: BinaryOperator,
        left: Expression,
        right: Expression,
        operator_place: Place,
        place: Place,
    ) -> Expression {
        let quoted = format!("`{}`", operator.text());
        // trace: D115. A `Whole` beside a `HugeWhole` becomes one, units and all.
        let base_of = |checker: &Self, value: &Expression| {
            checker
                .program
                .types
                .measured(value.type_id)
                .map_or_else(|| checker.program.types.resolve(value.type_id), |(base, _)| base)
        };
        let (left, right) = match (base_of(self, &left), base_of(self, &right)) {
            (WHOLE, HUGE_WHOLE) => (self.widen_measured(left), right),
            (HUGE_WHOLE, WHOLE) => (left, self.widen_measured(right)),
            _ => (left, right),
        };
        let left_measured = self.program.types.measured(left.type_id);
        let right_measured = self.program.types.measured(right.type_id);
        let left_base = left_measured.map_or_else(|| self.program.types.resolve(left.type_id), |(base, _)| base);
        let right_base = right_measured.map_or_else(|| self.program.types.resolve(right.type_id), |(base, _)| base);
        let binary = |left: Expression, right: Expression, type_id: TypeId| Expression {
            kind: ExpressionKind::Binary { operator, left: Box::new(left), right: Box::new(right) },
            type_id,
            place,
        };
        if left_base != right_base
            && !self.program.types.is_error(left.type_id)
            && !self.program.types.is_error(right.type_id)
        {
            self.refuse_why(
                operator_place,
                format!("{quoted} can't mix a `Whole` and a `Float`"),
                "`Whole` and `Float` don't mix quietly, units or not.",
            );
            return Self::error(place);
        }
        let not_designed = |checker: &mut Self| {
            checker.not_designed(
                operator_place,
                format!("{quoted} on these units, whose answer's unit has no written form,"),
            );
            Self::error(place)
        };
        match operator {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::RemFloor | BinaryOperator::RemTrunc
                if left_measured.is_some() && right_measured.is_some() =>
            {
                let (left_point, right_point) = (
                    left_measured.is_some_and(|(_, unit)| unit.point),
                    right_measured.is_some_and(|(_, unit)| unit.point),
                );
                let point = match (operator, left_point, right_point) {
                    (BinaryOperator::Add, true, true) => {
                        // trace: D121. Two places added is the classic slip.
                        self.refuse_why(
                            operator_place,
                            "`+` adds two points, two places, which has no meaning",
                            "a point plus an amount is a point, and a point minus a point is the amount between them; two points aren't added.",
                        );
                        return Self::error(place);
                    }
                    (BinaryOperator::Subtract, true, true) => false,
                    (BinaryOperator::Add | BinaryOperator::Subtract, left_point, false) => left_point,
                    (BinaryOperator::Add, false, true) => true,
                    _ if left_point || right_point => return not_designed(self),
                    _ => false,
                };
                let Some((left, right, unit)) = self.common(left, right, operator.text()) else {
                    return Self::error(place);
                };
                let type_id = self.measured_type(left_base, Unit { point, ..unit });
                // The operands each keep their own point-ness: only the answer's unit changes.
                binary(left, right, type_id)
            }
            BinaryOperator::Add | BinaryOperator::Subtract => {
                // trace: D111. Where something takes a unit, a plain number is refused.
                let plain = if left_measured.is_some() { &right } else { &left };
                self.refuse_why(
                    plain.place,
                    format!("{quoted} has a value with a unit on one side and a plain number on the other"),
                    "a number with a unit is added only to another of its group, so the unit is written on both, as `2 secs + 500 ms`.",
                );
                Self::error(place)
            }
            _ if operator.is_comparison() => {
                if left_measured.is_none() || right_measured.is_none() {
                    self.refuse_why(
                        operator_place,
                        format!("{quoted} compares a value with a unit and a plain number"),
                        "a number with a unit is compared only with another of its group, so the unit is written on both.",
                    );
                    return Self::error(place);
                }
                if left_measured.map(|(_, unit)| unit.point) != right_measured.map(|(_, unit)| unit.point) {
                    return not_designed(self);
                }
                let Some((left, right, _)) = self.common(left, right, operator.text()) else {
                    return Self::error(place);
                };
                if matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
                    || [WHOLE, FLOAT, HUGE_WHOLE].contains(&self.program.types.resolve(left_base))
                {
                    return binary(left, right, super::types::BOOL);
                }
                not_designed(self)
            }
            BinaryOperator::Multiply => match (left_measured, right_measured) {
                (Some((_, unit)), None) | (None, Some((_, unit))) => {
                    if unit.point {
                        return not_designed(self);
                    }
                    let type_id = self.measured_type(left_base, unit);
                    binary(left, right, type_id)
                }
                (Some((_, first)), Some((_, second))) => {
                    // A rate times an amount of its own `per` unit: `px per secs` times `secs` is `px`.
                    let (rate, rate_on_left, amount) = if first.per.is_some() && second.per.is_none() {
                        (first, true, second)
                    } else if second.per.is_some() && first.per.is_none() {
                        (second, false, first)
                    } else {
                        return not_designed(self);
                    };
                    let Some(per) = rate.per else { return not_designed(self) };
                    if amount.point || amount.top.group != per.group {
                        return not_designed(self);
                    }
                    let (rate_value, amount_value) = if rate_on_left { (left, right) } else { (right, left) };
                    let amount_place = amount_value.place;
                    let amount_value = match self.converted(
                        amount_value,
                        Unit { top: per, per: None, point: false },
                        left_base,
                    ) {
                        Ok(converted) => converted,
                        Err(why) => {
                            self.refuse_why(
                                amount_place,
                                format!("this can't be brought to the rate's unit: {why}"),
                                "a rate times an amount of its own unit gives the rate's unit, as `px per secs` times `secs` gives `px`.",
                            );
                            return Self::error(place);
                        }
                    };
                    let type_id = self.measured_type(left_base, Unit { top: rate.top, per: None, point: false });
                    if rate_on_left {
                        binary(rate_value, amount_value, type_id)
                    } else {
                        binary(amount_value, rate_value, type_id)
                    }
                }
                (None, None) => binary(left, right, left_base),
            },
            BinaryOperator::Divide | BinaryOperator::DivFloor | BinaryOperator::DivTrunc => {
                match (left_measured, right_measured) {
                    (Some((_, unit)), None) => {
                        if unit.point {
                            return not_designed(self);
                        }
                        let type_id = self.measured_type(left_base, unit);
                        binary(left, right, type_id)
                    }
                    (Some((_, first)), Some((_, second)))
                        if first.per.is_none() && second.per.is_none() && !first.point && !second.point =>
                    {
                        if first.top.group == second.top.group {
                            // An amount divided by another of its group is a plain number.
                            let Some((left, right, _)) = self.common(left, right, operator.text()) else {
                                return Self::error(place);
                            };
                            binary(left, right, left_base)
                        } else if operator == BinaryOperator::Divide {
                            // trace: D120. An amount divided by another of a different group is a rate.
                            let type_id = self
                                .measured_type(left_base, Unit { top: first.top, per: Some(second.top), point: false });
                            binary(left, right, type_id)
                        } else {
                            not_designed(self)
                        }
                    }
                    _ => not_designed(self),
                }
            }
            _ => not_designed(self),
        }
    }

    /// The angle built-ins' unit for an angle with one: 0 for `rads`, 1 for `degs`, 2 for `turns` (D113).
    pub(super) fn angle_unit(&self, type_id: TypeId) -> Option<Result<u8, ()>> {
        let (_, unit) = self.program.types.measured(type_id)?;
        let group = self.program.types.unit_groups.get(usize::from(unit.top.group))?;
        if group.name != "angle" || unit.per.is_some() || unit.point || unit.top.group != 1 {
            return Some(Err(()));
        }
        let member = group.members.get(usize::from(unit.top.member))?;
        Some(match member.name.as_str() {
            "rads" => Ok(0),
            "degs" => Ok(1),
            "turns" => Ok(2),
            _ => Err(()),
        })
    }

    /// `rads`, the unit `asin`, `acos` and `atan2` give their angle in (D113).
    pub(super) fn radians(&mut self) -> TypeId {
        let unit = Unit { top: UnitRef { group: 1, member: 2 }, per: None, point: false };
        self.measured_type(FLOAT, unit)
    }
}
