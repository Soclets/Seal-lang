//! Running operations and conversions on `HugeWhole`s, `Decimal`s and `HugeDecimal`s (D115, D116, D126, D127), all
//! kept out of the run loop. Each makes a new number: the ones it reads are never changed. Work and memory are
//! counted before anything big is worked out, so a script can't freeze or flood its host with one (D14).

use std::cmp::Ordering;

use super::heap::{Heap, Number, NumberRef};
use super::machine::Stop;
use super::show::{compare_numbers, number_text};
use crate::big::Big;
use crate::check::tree::{NumberConversion, NumberHow, NumberType, Rounding};
use crate::compile::bytecode::{Register, number_operation as op};
use crate::decimal::{Decimal, HugeDecimal, NumberError};
use crate::number::text_to_whole;

/// What an operation gives: a number, or a `Bool` for a comparison.
enum Answer {
    Number(Number),
    Bool(bool),
}

fn internal(detail: &str) -> Stop {
    Stop::Internal(detail.to_owned())
}

fn failed(what: &str, when: String, why: &str) -> Stop {
    Stop::CheckFailed { what: what.to_owned(), when: Some(when), why: Some(why.to_owned()) }
}

/// A number's exact value as a decimal.
fn exact(number: NumberRef<'_>) -> HugeDecimal {
    match number {
        NumberRef::Huge(value) => HugeDecimal::from_big(value.clone()),
        NumberRef::Decimal(value) => value.exact(),
        NumberRef::HugeDecimal(value) => value.clone(),
    }
}

/// How many 32-bit pieces a number has, for counting work: for a `HugeDecimal`, with its exponent's zeros, which
/// lining two up can write out.
fn pieces(number: NumberRef<'_>) -> u64 {
    (number.size() as u64 / 4).max(1)
}

/// trace: D14. Counts work about to be done, refused before it starts when the steps left can't pay for it.
fn afford(heap: &mut Heap, work: u64) -> Result<(), Stop> {
    if work > heap.work_allowance.saturating_sub(heap.work) {
        return Err(Stop::OutOfSteps);
    }
    heap.work = heap.work.saturating_add(work);
    Ok(())
}

/// trace: D18, D127. What a stop on a decimal says.
fn decimal_stop(error: NumberError, when: String) -> Stop {
    match error {
        NumberError::ByZero => failed("this divides by zero", when, "dividing by zero has no answer."),
        NumberError::TooBig => failed(
            "the result is too big for a `Decimal`",
            when,
            "a `Decimal` keeps 34 digits, up to almost 10 to the 6145; a `HugeDecimal` has no limit.",
        ),
        NumberError::Inexact => failed(
            "this has no exact decimal answer",
            when,
            "a `HugeDecimal` is exact, so `/` and `sqrt` give the exact answer or stop; a `Decimal` rounds to 34 digits.",
        ),
        NumberError::Negative => failed(
            "a negative number has no square root",
            when,
            "the square root is taken of a number that isn't negative.",
        ),
    }
}

/// Runs a `NumberOperation`, giving the slot of what it makes: a new number's handle, or a `Bool`.
///
/// # Errors
///
/// A failed check, the memory cap or the step budget being reached, or a bug in Seal.
pub fn operation(code: u8, registers: &[u64], left: Register, right: Register, heap: &mut Heap) -> Result<u64, Stop> {
    let slot = |register: Register| {
        registers.get(usize::from(register)).copied().ok_or_else(|| internal("a register outside its frame"))
    };
    let operation = code % op::TYPE;
    // Everything here is at most quadratic in the numbers' sizes, so that's what's paid before starting.
    let size =
        |register: Register| -> u64 { slot(register).ok().and_then(|slot| heap.number(slot).ok()).map_or(1, pieces) };
    let (left_size, right_size) =
        (size(left), if matches!(operation, op::POWER | op::NEGATE | op::ABS | op::SQRT) { 1 } else { size(right) });
    let mut work = left_size.saturating_add(right_size).saturating_pow(2);
    if operation == op::POWER {
        let times = slot(right)?.cast_signed().unsigned_abs().min(u64::from(u32::MAX));
        work = if code / op::TYPE == 1 {
            // A `Decimal`'s power is worked out to a few dozen digits past 34, never in full.
            4096 * (u64::from(64 - times.leading_zeros()) + 1)
        } else {
            left_size.saturating_mul(times).saturating_pow(2)
        };
    }
    afford(heap, work)?;
    let answer = {
        let heap: &Heap = heap;
        let number = |register: Register| heap.number(slot(register)?).map_err(Stop::from);
        let first = number(left)?;
        let room = heap.room_left();
        match operation {
            op::NEGATE | op::ABS | op::SQRT => one(operation, first)?,
            op::POWER => power(first, slot(right)?.cast_signed(), room)?,
            op::CLAMP | op::LERP => {
                let (second, third) = (number(right)?, number(right.wrapping_add(1))?);
                three(operation, first, second, third)?
            }
            _ => two(operation, first, number(right)?, room)?,
        }
    };
    match answer {
        Answer::Bool(flag) => Ok(u64::from(flag)),
        Answer::Number(number) => {
            heap.work = heap.work.saturating_add((number.size() as u64 / 4).max(1));
            heap.new_number(number).map_err(Stop::from)
        }
    }
}

/// An operation on one number.
fn one(operation: u8, value: NumberRef<'_>) -> Result<Answer, Stop> {
    let when = || format!("`sqrt({})`", number_text(value));
    Ok(Answer::Number(match (operation, value) {
        (op::NEGATE, NumberRef::Huge(value)) => Number::Huge(value.negated()),
        (op::NEGATE, NumberRef::Decimal(value)) => Number::Decimal(value.negated()),
        (op::NEGATE, NumberRef::HugeDecimal(value)) => Number::HugeDecimal(value.negated()),
        (op::ABS, NumberRef::Huge(value)) => Number::Huge(value.abs()),
        (op::ABS, NumberRef::Decimal(value)) => Number::Decimal(value.abs()),
        (op::ABS, NumberRef::HugeDecimal(value)) => Number::HugeDecimal(value.abs()),
        (op::SQRT, NumberRef::Decimal(value)) => {
            Number::Decimal(value.square_root().map_err(|error| decimal_stop(error, when()))?)
        }
        (op::SQRT, NumberRef::HugeDecimal(value)) => {
            Number::HugeDecimal(value.square_root().map_err(|error| decimal_stop(error, when()))?)
        }
        _ => return Err(internal("a number operation its type doesn't take")),
    }))
}

/// trace: D108, D127. A number to a whole power, with the memory it needs counted first.
fn power(base: NumberRef<'_>, power: i64, room: usize) -> Result<Answer, Stop> {
    let when = || format!("`{} ** {power}`", number_text(base));
    let room_digits = (room as u64).saturating_mul(2);
    let too_much = || Stop::OutOfMemory { needed: room.saturating_add(1), in_use: 0, cap: room };
    Ok(Answer::Number(match base {
        NumberRef::Huge(value) => {
            if power < 0 {
                return Err(failed(
                    "this raises a `HugeWhole` to a negative power",
                    when(),
                    "a whole number's power is 0 or more, since a negative one isn't a whole number.",
                ));
            }
            let times = power.unsigned_abs();
            let is_small = value.abs().compare(&Big::from_u64(1)) != Ordering::Greater;
            if !is_small && value.bits().saturating_mul(times) / 8 > room as u64 {
                return Err(too_much());
            }
            Number::Huge(if is_small && times > 2 { value.power(2 - (times % 2)) } else { value.power(times) })
        }
        NumberRef::Decimal(value) => Number::Decimal(value.power(power).map_err(|error| decimal_stop(error, when()))?),
        NumberRef::HugeDecimal(value) => match value.power(power, room_digits) {
            Ok(raised) => Number::HugeDecimal(raised),
            Err(NumberError::TooBig) => return Err(too_much()),
            Err(error) => return Err(decimal_stop(error, when())),
        },
    }))
}

/// trace: D127. An operation on two numbers of one type.
fn two(operation: u8, left: NumberRef<'_>, right: NumberRef<'_>, room: usize) -> Result<Answer, Stop> {
    let symbol = match operation {
        op::ADD => "+",
        op::SUBTRACT => "-",
        op::MULTIPLY => "*",
        op::DIVIDE => "/",
        op::DIVIDE_FLOOR => "div.floor",
        op::DIVIDE_TRUNC => "div.trunc",
        op::REMAINDER_FLOOR => "rem.floor",
        op::REMAINDER_TRUNC => "rem.trunc",
        _ => "",
    };
    let when = || format!("`{} {symbol} {}`", number_text(left), number_text(right));
    if let op::EQUAL..=op::AT_MOST | op::MIN | op::MAX = operation {
        let order = compare_numbers(left, right).ok_or_else(|| internal("two numbers of different types compared"))?;
        return Ok(match operation {
            op::EQUAL => Answer::Bool(order == Ordering::Equal),
            op::NOT_EQUAL => Answer::Bool(order != Ordering::Equal),
            op::LESS => Answer::Bool(order == Ordering::Less),
            op::AT_MOST => Answer::Bool(order != Ordering::Greater),
            // `min` and `max` give the first of two equal values, as they do for `Whole`s.
            op::MIN => Answer::Number(if order == Ordering::Greater { right.to_number() } else { left.to_number() }),
            _ => Answer::Number(if order == Ordering::Less { right.to_number() } else { left.to_number() }),
        });
    }
    if operation == op::MULTIPLY && left.size().saturating_add(right.size()) > room {
        return Err(Stop::OutOfMemory { needed: left.size().saturating_add(right.size()), in_use: 0, cap: room });
    }
    let floor = matches!(operation, op::DIVIDE_FLOOR | op::REMAINDER_FLOOR);
    let remainder = matches!(operation, op::REMAINDER_FLOOR | op::REMAINDER_TRUNC);
    let by_zero = || failed("this divides by zero", when(), "dividing by zero has no answer.");
    Ok(Answer::Number(match (left, right) {
        (NumberRef::Huge(left), NumberRef::Huge(right)) => Number::Huge(match operation {
            op::ADD => left.add(right),
            op::SUBTRACT => left.subtract(right),
            op::MULTIPLY => left.multiply(right),
            op::DIVIDE => {
                let (quotient, rest) = left.divide_truncating(right).ok_or_else(by_zero)?;
                if !rest.is_zero() {
                    // trace: Unio D70
                    return Err(failed(
                        "this division leaves a remainder",
                        when(),
                        "`/` on whole numbers gives the exact answer or stops; `div.floor` and `div.trunc` say how to round.",
                    ));
                }
                quotient
            }
            _ => {
                let divided = if floor { left.divide_floor(right) } else { left.divide_truncating(right) };
                let (quotient, rest) = divided.ok_or_else(by_zero)?;
                if remainder { rest } else { quotient }
            }
        }),
        (NumberRef::Decimal(left), NumberRef::Decimal(right)) => {
            let stop = |error| decimal_stop(error, when());
            Number::Decimal(match operation {
                op::ADD => left.add(&right).map_err(stop)?,
                op::SUBTRACT => left.subtract(&right).map_err(stop)?,
                op::MULTIPLY => left.multiply(&right).map_err(stop)?,
                op::DIVIDE => left.divide(&right).map_err(stop)?,
                _ => {
                    let (quotient, rest) = left.divide_whole(&right, floor).map_err(stop)?;
                    if remainder { rest } else { quotient }
                }
            })
        }
        (NumberRef::HugeDecimal(left), NumberRef::HugeDecimal(right)) => {
            let stop = |error| decimal_stop(error, when());
            Number::HugeDecimal(match operation {
                op::ADD => left.add(right),
                op::SUBTRACT => left.subtract(right),
                op::MULTIPLY => left.multiply(right),
                op::DIVIDE => left.divide(right).map_err(stop)?,
                _ => {
                    let (quotient, rest) = left.divide_whole(right, floor).map_err(stop)?;
                    if remainder { rest } else { HugeDecimal::from_big(quotient) }
                }
            })
        }
        _ => return Err(internal("two numbers of different types in one operation")),
    }))
}

/// `clamp` and `lerp` (D107, D114, D127).
fn three(operation: u8, first: NumberRef<'_>, second: NumberRef<'_>, third: NumberRef<'_>) -> Result<Answer, Stop> {
    if operation == op::LERP {
        let (NumberRef::Decimal(a), NumberRef::Decimal(b), NumberRef::Decimal(t)) = (first, second, third) else {
            return Err(internal("`lerp` on numbers that aren't `Decimal`s"));
        };
        let when = || format!("`lerp({}, to: {}, by: {})`", a.to_text(), b.to_text(), t.to_text());
        return Ok(Answer::Number(Number::Decimal(a.lerp(&b, &t).map_err(|error| decimal_stop(error, when()))?)));
    }
    let order = |a: NumberRef<'_>, b: NumberRef<'_>| {
        compare_numbers(a, b).ok_or_else(|| internal("numbers of different types"))
    };
    if order(second, third)? == Ordering::Greater {
        return Err(Stop::NotDesigned("a `clamp` whose `low:` is more than its `high:`".to_owned()));
    }
    let clamped = if order(first, second)? == Ordering::Less {
        second
    } else if order(first, third)? == Ordering::Greater {
        third
    } else {
        first
    };
    Ok(Answer::Number(clamped.to_number()))
}

/// What a `ConvertNumber` gives: the slot it makes, or the failure it fails with.
pub enum Converted {
    Slot(u64),
    Failed(u32),
}

/// trace: D115, D116, D126. Runs a `ConvertNumber`.
///
/// # Errors
///
/// The memory cap, or a bug in Seal.
pub fn convert(conversion: NumberConversion, failure: u16, slot: u64, heap: &mut Heap) -> Result<Converted, Stop> {
    let fail = Ok(Converted::Failed(u32::from(failure)));
    if !matches!(conversion.from, NumberType::Whole | NumberType::Float | NumberType::Text) {
        let size = heap.number(slot).map_or(1, pieces);
        afford(heap, size.saturating_pow(2))?;
    }
    let number = match conversion.from {
        NumberType::Whole => Number::HugeDecimal(HugeDecimal::from_whole(slot.cast_signed())),
        NumberType::Float => Number::Decimal(
            match conversion.how {
                NumberHow::Shown => Decimal::from_float_shown(f64::from_bits(slot)),
                _ => Decimal::from_float_nearest(f64::from_bits(slot)),
            }
            .map_err(|_| internal("a `Float` too big for a `Decimal`"))?,
        ),
        NumberType::Text => {
            // Reading digits into a whole number is quadratic in how many there are.
            let length = heap.text(slot).map_err(Stop::from)?.len() as u64;
            afford(heap, (length / 8).saturating_add(1).saturating_pow(2))?;
            let text = heap.text(slot).map_err(Stop::from)?;
            let read = match conversion.to {
                NumberType::Huge => read_whole(text).map(HugeDecimal::from_big),
                _ => HugeDecimal::from_text(text),
            };
            match read {
                Some(read) => Number::HugeDecimal(read),
                None => return fail,
            }
        }
        _ => heap.number(slot).map_err(Stop::from)?.to_number(),
    };
    heap.work = heap.work.saturating_add((number.size() as u64 / 4).max(1));
    let value = match &number {
        Number::Huge(value) => HugeDecimal::from_big(value.clone()),
        Number::Decimal(value) => value.exact(),
        Number::HugeDecimal(value) => value.clone(),
    };
    let whole = |rounding: Option<Rounding>| value.to_whole(rounding.unwrap_or(Rounding::Trunc));
    let rounding = match conversion.how {
        NumberHow::Round(rounding) => Some(rounding),
        _ => None,
    };
    let made = match conversion.to {
        NumberType::Whole => {
            return Ok(match whole(rounding).to_i64() {
                Some(whole) => Converted::Slot(whole.cast_unsigned()),
                None => Converted::Failed(u32::from(failure)),
            });
        }
        NumberType::Float => {
            let float = value.to_float();
            return Ok(if float.is_finite() {
                Converted::Slot(float.to_bits())
            } else {
                Converted::Failed(u32::from(failure))
            });
        }
        NumberType::Text => return Err(internal("a number conversion to text")),
        NumberType::Huge => Number::Huge(whole(rounding)),
        NumberType::Decimal => match number {
            Number::Decimal(decimal) => Number::Decimal(decimal),
            _ => match Decimal::rounded(&value) {
                Ok(decimal) => Number::Decimal(decimal),
                Err(_) => return fail,
            },
        },
        NumberType::HugeDecimal => Number::HugeDecimal(value),
    };
    heap.new_number(made).map(Converted::Slot).map_err(Stop::from)
}

/// A whole number of any size read from text, as `be.Whole` reads one (Unio D95, D143).
fn read_whole(text: &str) -> Option<Big> {
    if let Some(small) = text_to_whole(text) {
        return Some(Big::from_i64(small));
    }
    let trimmed = text.trim_matches(|character| character == ' ' || character == '\t');
    let (negative, digits) = match trimmed.as_bytes().first() {
        Some(b'-') => (true, trimmed.get(1..)?),
        Some(b'+') => (false, trimmed.get(1..)?),
        _ => (false, trimmed),
    };
    if digits.is_empty() || !digits.bytes().all(|digit| digit.is_ascii_digit()) {
        return None;
    }
    let value = Big::parse(digits)?;
    Some(if negative { value.negated() } else { value })
}

/// A decimal shown to `places` digits after its point, rounded to the nearest, a tie away from zero, with no minus
/// on zero (D94, D125).
#[must_use]
pub fn with_places(number: NumberRef<'_>, places: u8) -> String {
    exact(number).with_places(places)
}

/// trace: D107, D115, D128. A `HugeWhole` landing where its type has limits: stopped past a hard edge, or pulled
/// back to a `.clamp` edge as a new number, the old one let go of.
///
/// # Errors
///
/// A hard edge passed, the memory cap, or a bug in Seal.
pub fn land(
    slot: u64,
    limits: &crate::check::types::Limits,
    heap: &mut Heap,
    layouts: &[crate::compile::bytecode::Layout],
) -> Result<u64, Stop> {
    let NumberRef::Huge(value) = heap.number(slot).map_err(Stop::from)? else {
        return Err(internal("a ranged `HugeWhole` that isn't one"));
    };
    for (edge, is_min) in [(limits.min, true), (limits.max, false)] {
        let Some(edge) = edge else { continue };
        let edge_value = Big::from_i64(edge.bits.cast_signed());
        let order = value.compare(&edge_value);
        let outside = if is_min { order == Ordering::Less } else { order == Ordering::Greater };
        if !outside {
            continue;
        }
        if edge.clamp {
            let made = heap.new_number(Number::Huge(edge_value)).map_err(Stop::from)?;
            heap.release(slot, layouts);
            return Ok(made);
        }
        let (word, side) = if is_min { ("min", "below") } else { ("max", "above") };
        let edge_text = edge.bits.cast_signed().to_string();
        return Err(failed(
            &format!("this value is {side} its type's `{word}: {edge_text}`"),
            format!("`{}` lands where the type says `{word}: {edge_text}`", value.to_decimal()),
            "a ranged number's `min:` and `max:` are hard limits; `min.clamp:` and `max.clamp:` pull a value back instead.",
        ));
    }
    Ok(slot)
}
