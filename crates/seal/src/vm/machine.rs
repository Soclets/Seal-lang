//! The loop that runs bytecode (D46).
//!
//! Nothing here can crash the host (D14): every register, instruction and heap value is reached through a check,
//! and anything that doesn't add up stops the script and says so, as a bug in Seal to report.

use super::crossing::{CrossingError, plain_slot, plain_value, to_slot, to_value};
use super::dict::{Dict, Probe};
use super::heap::{Heap, HeapError, Object, SLOT_COST};
use super::show::{equal, show};
use crate::check::tree::Rounding;
use crate::check::types::{Limits, TypeId};
use crate::compile::bytecode::{
    BULK_UNIT, Compiled, Crossing, FAR, Instruction, JoinPiece, Layout, PLAIN_CASE, ProcedureEntry, Register,
};
use crate::host::Value;
use crate::maths;
use crate::number::{float_to_text, float_to_whole, float_with_places, text_to_whole};
use crate::text::{grapheme_at, grapheme_count, graphemes};

/// What a stack slot costs against the memory cap: the slot itself, and its share of the record of calls.
const STACK_SLOT_COST: usize = 2 * SLOT_COST;

/// What a door answers when the VM calls it.
#[derive(Clone, Debug, PartialEq)]
pub enum DoorAnswer {
    Value(Value),
    /// The door failed with one of the failures its host declared (D44).
    Failure(String),
    /// A plain number, a `Bool` or nothing, as the bare bits a script holds it in.
    Plain(u64),
    /// The door pauses the script; the host resumes it later with the door's value (D49).
    Pause,
    /// The door couldn't be called as declared: a mistake in the host's code, or in Seal.
    Mistake(String),
}

/// The host's doors, as the VM calls them.
pub trait DoorCaller {
    /// Calls a door. `bits` has every argument's bare bits; `values` has, in the same places, the values of the
    /// arguments that are more than their bits, which the door takes.
    fn call_door(&self, door: u32, bits: &[u64], values: &mut Vec<Value>) -> DoorAnswer;
}

/// Why a script stopped (D14, D17).
#[derive(Clone, Debug, PartialEq)]
pub enum Stop {
    /// A check failed while running: what, and the values that broke it.
    CheckFailed { what: String, when: Option<String>, why: Option<String> },
    /// The step budget ran out (D14).
    OutOfSteps,
    /// The memory cap was reached (D14).
    OutOfMemory { needed: usize, in_use: usize, cap: usize },
    /// `?? crash` met a failure (Unio D152).
    Crashed { failure: u32 },
    /// The script reached something no decision covers, which only shows while running.
    NotDesigned(String),
    /// A door gave something its host didn't declare: a mistake in the host's code.
    HostMistake(String),
    /// Seal itself went wrong.
    Internal(String),
}

impl From<HeapError> for Stop {
    fn from(error: HeapError) -> Self {
        match error {
            HeapError::OutOfMemory { needed, in_use, cap } => Stop::OutOfMemory { needed, in_use, cap },
            HeapError::Broken(detail) => Stop::Internal(format!("the heap met {detail}")),
        }
    }
}

/// One call still running: where to go back to.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    return_pc: u32,
    base: u32,
}

/// A paused task's door call, waiting for its value (D49).
#[derive(Clone, Copy, Debug)]
pub struct Waiting {
    /// The stack slot the door's value goes into.
    pub slot: usize,
    /// Which call site it was, for how the value crosses.
    pub site: u32,
}

/// One task: a call into the script, with its own stack (D51).
#[derive(Debug, Default)]
pub struct Task {
    pub stack: Vec<u64>,
    pub frames: Vec<Frame>,
    pub pc: usize,
    pub base: usize,
    pub waiting: Option<Waiting>,
}

/// What every task of a script shares (D51).
#[derive(Debug, Default)]
pub struct Machine {
    pub heap: Heap,
    pub globals: Vec<u64>,
    /// The handles of the script's own texts, which live as long as it does.
    pub texts: Vec<u64>,
    /// Where a door's arguments are put for the call, kept from one call to the next.
    pub door_arguments: Vec<Value>,
}

/// How a run of a task ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The task's procedure returned; its result is in the task's first stack slot.
    Finished,
    /// A door paused the task (D49).
    Paused,
    /// The task's procedure failed, with this failure (D44).
    Failed(u32),
}

/// A stop, with the instruction it happened at.
pub type Stopped = (Stop, usize);

#[cold]
fn broken(detail: &str, pc: usize) -> Stopped {
    (Stop::Internal(detail.to_owned()), pc)
}

#[cold]
fn check_failed(what: &str, when: String, why: &str, pc: usize) -> Stopped {
    (Stop::CheckFailed { what: what.to_owned(), when: Some(when), why: Some(why.to_owned()) }, pc)
}

/// Whether a float is checked after every operation (D18). Always, except in a build made only to measure what the
/// check costs (D58): a script never runs without it (D17).
const FLOAT_CHECKS: bool = !cfg!(seal_measure_without_float_checks);

const WHOLE_RANGE: &str = "a `Whole` holds whole numbers from -9223372036854775808 to 9223372036854775807.";
const FLOAT_RANGE: &str = "a `Float` is always a real number, never infinite and never not-a-number.";

#[cold]
fn overflow(left: i64, operator: &str, right: i64, pc: usize) -> Stopped {
    // trace: D16, Unio D46, D92
    check_failed("the result is too big for a `Whole`", format!("`{left} {operator} {right}`"), WHOLE_RANGE, pc)
}

#[cold]
fn divided_by_zero(left: &str, operator: &str, pc: usize) -> Stopped {
    // trace: Unio D46, D72
    check_failed("this divides by zero", format!("`{left} {operator} 0`"), "nothing can be divided by zero.", pc)
}

#[cold]
fn float_not_real(left: f64, operator: &str, right: f64, pc: usize) -> Stopped {
    // trace: D18, Unio D177
    check_failed(
        "the result isn't a real number a `Float` can hold",
        format!("`{} {operator} {}`", float_to_text(left), float_to_text(right)),
        FLOAT_RANGE,
        pc,
    )
}

#[cold]
fn bad_position(label: &str, index: i64, length: usize, pc: usize) -> Stopped {
    // trace: Unio D46, D75
    let range = match (label, length) {
        (_, 0) => "the list is empty".to_owned(),
        ("pos", 1) => "the list has 1 item, at `pos:` 1".to_owned(),
        (_, 1) => "the list has 1 item, at `offset:` 0".to_owned(),
        ("pos", _) => format!("the list has {length} items, at `pos:` 1 to {length}"),
        _ => format!("the list has {length} items, at `offset:` 0 to {}", length - 1),
    };
    check_failed(
        "there's no item at this place",
        format!("`{label}: {index}`, and {range}"),
        "an item is reached at a place the list has.",
        pc,
    )
}

#[cold]
fn bad_text_position(label: &str, index: i64, length: usize, pc: usize) -> Stopped {
    // trace: D66, Unio D46
    let range = match (label, length) {
        (_, 0) => "the text is empty".to_owned(),
        ("pos", 1) => "the text has 1 character, at `pos:` 1".to_owned(),
        (_, 1) => "the text has 1 character, at `offset:` 0".to_owned(),
        ("pos", _) => format!("the text has {length} characters, at `pos:` 1 to {length}"),
        _ => format!("the text has {length} characters, at `offset:` 0 to {}", length - 1),
    };
    check_failed(
        "there's no character at this place",
        format!("`{label}: {index}`, and {range}"),
        "a text is reached at a place it has, counting what a person sees as one character.",
        pc,
    )
}

/// Makes a list of new texts, counted against the memory cap before any of it is made (D14).
fn text_list(heap: &mut Heap, pieces: &[String], work: usize, layouts: &[Layout]) -> Result<u64, HeapError> {
    heap.work = heap.work.saturating_add(work as u64).saturating_add(pieces.len() as u64);
    let size = pieces.len().saturating_mul(SLOT_COST);
    heap.charge(size)?;
    heap.refund(size);
    let mut items = Vec::new();
    items.reserve_exact(pieces.len());
    for piece in pieces {
        match heap.new_text(piece, piece.len()) {
            Ok(handle) => items.push(handle),
            Err(error) => {
                // What was made so far is given back, so a script stopped at its cap holds nothing stray.
                for handle in items {
                    heap.release(handle, layouts);
                }
                return Err(error);
            }
        }
    }
    heap.allocate(Object::List { items, references: true }, size)
}

/// Joins a text from its pieces into register `to`, or onto the text already there when `appending` (D75, D94,
/// D100). It's kept apart from [`run`]'s loop, whose speed depends on how little code sits in it.
#[inline]
fn join_pieces(
    registers: &mut [u64],
    heap: &mut Heap,
    compiled: &Compiled,
    to: Register,
    site: u32,
    appending: bool,
) -> Result<(), Stop> {
    let broken = |detail: &'static str| Stop::Internal(detail.to_owned());
    let get = |registers: &[u64], register: Register| {
        registers.get(usize::from(register)).copied().ok_or_else(|| broken("a register outside its frame"))
    };
    let Some(pieces) = compiled.joins.get(site as usize) else {
        return Err(broken("a join that isn't there"));
    };
    // How long the pieces are together, so the text is given its room once.
    let mut extra = 0usize;
    for piece in pieces {
        extra = extra.saturating_add(match *piece {
            JoinPiece::Constant(text) => compiled.texts.get(text as usize).map_or(0, String::len),
            JoinPiece::Text(register) => heap.text(get(registers, register)?)?.len(),
            JoinPiece::Whole(register) => {
                let value = get(registers, register)?.cast_signed();
                value.unsigned_abs().checked_ilog10().map_or(1, |digits| digits as usize + 1) + usize::from(value < 0)
            }
            JoinPiece::Bool(_) => 5,
            JoinPiece::Float(_) => 24,
            JoinPiece::Places(_, places) => 24 + usize::from(places),
            JoinPiece::NumberPlaces(register, places) => {
                heap.number(get(registers, register)?)?.size().saturating_mul(3).saturating_add(usize::from(places) + 2)
            }
            JoinPiece::Value(..) => 16,
        });
    }
    // The text being built is held here, outside the heap, while the pieces are read from the heap.
    let (target, mut built, old_room) = if appending {
        // Made this register's own first, so it can change (copy-on-write, D15).
        let mut target = get(registers, to)?;
        if !heap.has_one_holder(target) {
            target = heap.make_own(target, &compiled.layouts)?;
            if let Some(slot) = registers.get_mut(usize::from(to)) {
                *slot = target;
            }
        }
        match heap.take(target)? {
            Object::Text(text) => {
                let room = text.capacity();
                (target, text, room)
            }
            other => {
                heap.put_back(target, other);
                return Err(broken("a handle that should lead to text"));
            }
        }
    } else {
        (0, heap.spare_text(extra), 0)
    };
    let original = built.len();
    let needed = original.saturating_add(extra);
    if appending && needed > built.capacity() {
        // Doubling, so joining onto a text many times costs a fixed amount per byte on average.
        let room = needed.max(built.capacity().saturating_mul(2)).max(16);
        if let Err(error) = heap.charge(room - old_room) {
            heap.put_back(target, Object::Text(built));
            return Err(Stop::from(error));
        }
        built.reserve_exact(room - original);
    }
    let written = write_pieces(pieces, registers, heap, compiled, appending.then_some((target, original)), &mut built);
    if let Err(stop) = written {
        if appending {
            heap.put_back(target, Object::Text(built));
        }
        return Err(stop);
    }
    heap.work = heap.work.saturating_add(built.len().saturating_sub(original) as u64);
    if appending {
        heap.put_back(target, Object::Text(built));
    } else {
        let room = built.capacity();
        let handle = heap.allocate(Object::Text(built), room)?;
        match registers.get_mut(usize::from(to)) {
            Some(slot) => *slot = handle,
            None => return Err(broken("a register outside its frame")),
        }
    }
    Ok(())
}

/// The instructions [`run`] hands over to work outside its loop: each changes its registers as the loop would, and
/// gives the failure it ends with, when it has one.
#[inline(never)]
#[allow(clippy::too_many_lines)] // one case per instruction handed over
fn rare_instruction(
    instruction: Instruction,
    registers: &mut [u64],
    heap: &mut Heap,
    compiled: &Compiled,
    pc: usize,
    steps_left: u64,
) -> Result<Option<u32>, Stopped> {
    // trace: D14. What's left to pay for big numbers' work with.
    heap.work_allowance = steps_left.saturating_mul(BULK_UNIT);
    let get = |registers: &[u64], register: Register| {
        registers.get(usize::from(register)).copied().ok_or_else(|| broken("a register outside its frame", pc))
    };
    let set = |registers: &mut [u64], register: Register, value: u64| match registers.get_mut(usize::from(register)) {
        Some(slot) => {
            *slot = value;
            Ok(())
        }
        None => Err(broken("a register outside its frame", pc)),
    };
    let stop = |stop: Stop| (stop, pc);
    match instruction {
        Instruction::MakeRange { to, base, inclusive } => {
            let numbers = [
                get(registers, base)?.cast_signed(),
                get(registers, base.wrapping_add(1))?.cast_signed(),
                get(registers, base.wrapping_add(2))?.cast_signed(),
            ];
            let list = make_range(numbers, inclusive, heap).map_err(stop)?;
            set(registers, to, list)?;
        }
        Instruction::EqualValues { to, left, site } => {
            let right = left.wrapping_add(1);
            let same =
                equal_values(get(registers, left)?, get(registers, right)?, site, heap, compiled).map_err(stop)?;
            set(registers, to, u64::from(same))?;
        }
        Instruction::PowerWhole { to, left, right } => {
            let value = power_whole(get(registers, left)?.cast_signed(), get(registers, right)?.cast_signed(), pc)?;
            set(registers, to, value.cast_unsigned())?;
        }
        Instruction::PowerFloat { to, left, right } => {
            let (left, right) = (f64::from_bits(get(registers, left)?), f64::from_bits(get(registers, right)?));
            let result = maths::power(left, right);
            // trace: D18
            if !result.is_finite() {
                return Err(float_not_real(left, "**", right, pc));
            }
            set(registers, to, result.to_bits())?;
        }
        Instruction::Maths { to, base, function } => {
            let count = usize::from(MATHS_ARGUMENTS.get(usize::from(function)).copied().unwrap_or(1));
            let Some(arguments) = registers.get(usize::from(base)..usize::from(base) + count) else {
                return Err(broken("a register outside its frame", pc));
            };
            let value = maths_function(function, arguments, pc)?;
            set(registers, to, value)?;
        }
        Instruction::CheckLimits { register, site } => {
            let Some(limits) = compiled.limits.get(usize::from(site)) else {
                return Err(broken("a ranged number's check that isn't there", pc));
            };
            let held = get(registers, register)?;
            if limits.huge {
                // trace: D115
                let value = super::numbers::land(held, limits, heap, &compiled.layouts).map_err(stop)?;
                set(registers, register, value)?;
            } else {
                let value = land(held, limits, pc)?;
                set(registers, register, value)?;
            }
        }
        Instruction::Scale { to, from, site } => {
            let Some(&(size, from_whole)) = compiled.scales.get(usize::from(site)) else {
                return Err(broken("a unit conversion that isn't there", pc));
            };
            let bits = get(registers, from)?;
            let value = if from_whole {
                maths::scale_whole(bits.cast_signed(), size.numerator, size.denominator, size.pi)
            } else {
                maths::scale_float(f64::from_bits(bits), size.numerator, size.denominator, size.pi)
            };
            // trace: D18
            if !value.is_finite() {
                return Err(check_failed(
                    "the result isn't a real number a `Float` can hold",
                    "a unit conversion".to_owned(),
                    FLOAT_RANGE,
                    pc,
                ));
            }
            set(registers, to, value.to_bits())?;
        }
        Instruction::CrashIfNone { number } => {
            if get(registers, number)? == 0 {
                // trace: D79
                return Err(check_failed(
                    "this `Maybe` is `none`",
                    "it's followed by `?? crash`".to_owned(),
                    "`?? crash` says the `Maybe` can't be `none` here, so the script stops when it is.",
                    pc,
                ));
            }
        }
        // trace: D101. Exactly as a `Bool` shows, and nothing else.
        Instruction::TextToBool { to, from, failure } => {
            match heap.text(get(registers, from)?).map_err(|error| stop(Stop::from(error)))? {
                "true" => set(registers, to, 1)?,
                "false" => set(registers, to, 0)?,
                _ => return Ok(Some(u32::from(failure))),
            }
        }
        Instruction::JoinPieces { to, site } | Instruction::AppendPieces { to, site } => {
            let appending = matches!(instruction, Instruction::AppendPieces { .. });
            join_pieces(registers, heap, compiled, to, site, appending).map_err(stop)?;
        }
        // trace: D115, D116
        Instruction::LoadNumber { to, site } => {
            let Some(number) = compiled.numbers.get(usize::from(site)) else {
                return Err(broken("a number written out that isn't there", pc));
            };
            let handle = heap.new_number(number.clone()).map_err(|error| stop(Stop::from(error)))?;
            set(registers, to, handle)?;
        }
        Instruction::NumberOperation { to, left, right, operation } => {
            let made = super::numbers::operation(operation, registers, left, right, heap).map_err(stop)?;
            set(registers, to, made)?;
        }
        Instruction::ConvertNumber { to, from, site } => {
            let Some(&(conversion, failure)) = compiled.number_conversions.get(usize::from(site)) else {
                return Err(broken("a number conversion that isn't there", pc));
            };
            match super::numbers::convert(conversion, failure, get(registers, from)?, heap).map_err(stop)? {
                super::numbers::Converted::Slot(made) => set(registers, to, made)?,
                super::numbers::Converted::Failed(failure) => return Ok(Some(failure)),
            }
        }
        _ => return Err(broken("an instruction handed over that isn't one of these", pc)),
    }
    Ok(None)
}

/// A `range[…]` kept as a list (D99), checked as a `for each` checks one (Unio D172), with its memory counted before
/// it's built, so a range of billions stops at the cap instead of being built. Kept out of [`run`]'s loop.
#[inline(never)]
fn make_range([from, end, step]: [i64; 3], inclusive: bool, heap: &mut Heap) -> Result<u64, Stop> {
    let label = if inclusive { "to" } else { "before" };
    let when = || Some(format!("`range[from: {from}, {label}: {end}, step_by: {step}]`"));
    if step == 0 {
        return Err(Stop::CheckFailed {
            what: "this range's step is 0".to_owned(),
            when: when(),
            why: Some("a range with a step of 0 never reaches its end.".to_owned()),
        });
    }
    if if step > 0 { from > end } else { from < end } {
        return Err(Stop::CheckFailed {
            what: "this range's step points away from its end".to_owned(),
            when: when(),
            why: Some("a range's step points towards its end; a negative `step_by:` counts down.".to_owned()),
        });
    }
    let distance = (i128::from(end) - i128::from(from)).unsigned_abs();
    let stride = i128::from(step).unsigned_abs();
    let count = if inclusive { distance / stride + 1 } else { distance.div_ceil(stride) };
    let needed = usize::try_from(count).ok().and_then(|count| count.checked_mul(SLOT_COST)).unwrap_or(usize::MAX);
    if needed > heap.room_left() {
        return Err(Stop::from(HeapError::OutOfMemory { needed, in_use: heap.in_use(), cap: heap.cap() }));
    }
    let count = usize::try_from(count).unwrap_or(0);
    let mut items = Vec::with_capacity(count);
    let mut value = i128::from(from);
    for _ in 0..count {
        items.push(i64::try_from(value).unwrap_or(0).cast_unsigned());
        value += i128::from(step);
    }
    heap.work = heap.work.saturating_add(count as u64);
    Ok(heap.allocate(Object::List { items, references: false }, needed)?)
}

/// A number landing in a ranged type (D107, D128): past a hard edge it stops the script; past a `.clamp` edge it's
/// pulled back to it. Kept out of [`run`]'s loop.
#[inline(never)]
fn land(bits: u64, limits: &Limits, pc: usize) -> Result<u64, Stopped> {
    let shown = |bits: u64| {
        if limits.float { float_to_text(f64::from_bits(bits)) } else { bits.cast_signed().to_string() }
    };
    let below = |value: u64, edge: u64| {
        if limits.float {
            f64::from_bits(value) < f64::from_bits(edge)
        } else {
            value.cast_signed() < edge.cast_signed()
        }
    };
    for (edge, is_min) in [(limits.min, true), (limits.max, false)] {
        let Some(edge) = edge else { continue };
        let outside = if is_min { below(bits, edge.bits) } else { below(edge.bits, bits) };
        if !outside {
            continue;
        }
        if edge.clamp {
            return Ok(edge.bits);
        }
        let (word, side) = if is_min { ("min", "below") } else { ("max", "above") };
        return Err(check_failed(
            &format!("this value is {side} its type's `{word}: {}`", shown(edge.bits)),
            format!("`{}` lands where the type says `{word}: {}`", shown(bits), shown(edge.bits)),
            "a ranged number's `min:` and `max:` are hard limits; `min.clamp:` and `max.clamp:` pull a value back instead.",
            pc,
        ));
    }
    Ok(bits)
}

/// How many arguments each of the maths built-ins takes, by its number in [`maths_function`].
const MATHS_ARGUMENTS: [u8; 21] = [1, 1, 2, 2, 2, 2, 3, 3, 1, 1, 1, 1, 1, 2, 3, 1, 1, 1, 1, 1, 1];

/// `left ** right` on two `Whole`s (D108). Kept out of [`run`]'s loop.
#[inline(never)]
fn power_whole(left: i64, right: i64, pc: usize) -> Result<i64, Stopped> {
    if right < 0 {
        // trace: D108. A negative power of a `Whole` isn't whole; one written out is refused while loading.
        return Err(check_failed(
            "this raises a `Whole` to a negative power",
            format!("`{left} ** {right}`"),
            "a `Whole`'s power is 0 or more, since a negative one isn't a whole number; `be.Float(…)` gives `Float`s.",
            pc,
        ));
    }
    let exponent = u32::try_from(right).unwrap_or(u32::MAX);
    match left.checked_pow(exponent) {
        Some(value) => Ok(value),
        // 0, 1 and -1 never overflow, whatever the power.
        None if left == 0 || left == 1 => Ok(left),
        None if left == -1 => Ok(if right % 2 == 0 { 1 } else { -1 }),
        None => Err(overflow(left, "**", right, pc)),
    }
}

/// One of the maths built-ins on its arguments' bits, by its number: `abs`, `min`, `max` and `clamp` on `Whole`s
/// (0, 2, 4, 6) and `Float`s (1, 3, 5, 7), then `sin`, `cos`, `tan`, `asin`, `acos`, `atan2` and `lerp` (8 to 14),
/// and `sin`, `cos` and `tan` in `degs` (15 to 17) and `turns` (18 to 20) (D107, D109, D113, D114). Kept out of
/// [`run`]'s loop.
#[inline(never)]
fn maths_function(function: u8, arguments: &[u64], pc: usize) -> Result<u64, Stopped> {
    let whole = |index: usize| arguments.get(index).copied().unwrap_or(0).cast_signed();
    let float = |index: usize| f64::from_bits(arguments.get(index).copied().unwrap_or(0));
    let floats = |value: f64| Ok(value.to_bits());
    let not_real = |text: String, value: f64| {
        if value.is_finite() {
            Ok(value.to_bits())
        } else {
            Err(check_failed("the result isn't a real number a `Float` can hold", text, FLOAT_RANGE, pc))
        }
    };
    let crossed = || (Stop::NotDesigned("a `clamp` whose `low:` is more than its `high:`".to_owned()), pc);
    match function {
        0 => match whole(0).checked_abs() {
            Some(value) => Ok(value.cast_unsigned()),
            None => Err(check_failed(
                "the result is too big for a `Whole`",
                format!("`abs({})`", whole(0)),
                WHOLE_RANGE,
                pc,
            )),
        },
        1 => floats(float(0).abs()),
        2 => Ok(whole(0).min(whole(1)).cast_unsigned()),
        // Two equal `Float`s, as 0.0 and -0.0, give the first, so the answer is the same everywhere (D50).
        3 => floats(if float(1) < float(0) { float(1) } else { float(0) }),
        4 => Ok(whole(0).max(whole(1)).cast_unsigned()),
        5 => floats(if float(1) > float(0) { float(1) } else { float(0) }),
        6 => {
            let (value, low, high) = (whole(0), whole(1), whole(2));
            if low > high {
                return Err(crossed());
            }
            Ok(value.clamp(low, high).cast_unsigned())
        }
        7 => {
            let (value, low, high) = (float(0), float(1), float(2));
            if low > high {
                return Err(crossed());
            }
            floats(if value < low {
                low
            } else if value > high {
                high
            } else {
                value
            })
        }
        8 => floats(maths::sin(float(0))),
        9 => floats(maths::cos(float(0))),
        10 => floats(maths::tan(float(0))),
        11 | 12 => {
            let name = if function == 11 { "asin" } else { "acos" };
            let answer = if function == 11 { maths::asin(float(0)) } else { maths::acos(float(0)) };
            // trace: D109. `asin` and `acos` take a value from -1 to 1.
            answer.map(f64::to_bits).ok_or_else(|| {
                check_failed(
                    &format!("`{name}` takes a value from -1 to 1"),
                    format!("`{name}({})`", float_to_text(float(0))),
                    "no angle has a sine or cosine past 1 or below -1.",
                    pc,
                )
            })
        }
        15..=20 => {
            let per_turn = if function < 18 { 360 } else { 1 };
            let value = maths::angle_function((function - 15) % 3, float(0), per_turn);
            let name = ["sin", "cos", "tan"][usize::from((function - 15) % 3)];
            let unit = if per_turn == 360 { "degs" } else { "turns" };
            not_real(format!("`{name}({} {unit})`", float_to_text(float(0))), value)
        }
        13 => floats(maths::atan2(float(0), float(1))),
        _ => {
            let (a, b, t) = (float(0), float(1), float(2));
            let text = format!("`lerp({}, to: {}, by: {})`", float_to_text(a), float_to_text(b), float_to_text(t));
            not_real(text, maths::lerp(a, b, t))
        }
    }
}

/// Whether two bigger values hold the same contents (D95), with the work it took counted as steps (D14). Kept out of
/// [`run`]'s loop, as joining is.
#[inline(never)]
fn equal_values(left: u64, right: u64, site: u16, heap: &mut Heap, compiled: &Compiled) -> Result<bool, Stop> {
    let Some(&type_id) = compiled.compared_types.get(usize::from(site)) else {
        return Err(Stop::Internal("a comparison that isn't there".to_owned()));
    };
    let mut work = 0;
    let same = equal(left, right, TypeId(type_id), &compiled.types, heap, &compiled.layouts, &mut work)?;
    heap.work = heap.work.saturating_add(work);
    Ok(same)
}

/// Writes each piece of a join onto `built`. When joining onto a text in place, `onto` is that text's handle and how
/// long it was, since its own start is what's joined if it's a piece too.
fn write_pieces(
    pieces: &[JoinPiece],
    registers: &[u64],
    heap: &Heap,
    compiled: &Compiled,
    onto: Option<(u64, usize)>,
    built: &mut String,
) -> Result<(), Stop> {
    let get = |register: Register| {
        registers
            .get(usize::from(register))
            .copied()
            .ok_or_else(|| Stop::Internal("a register outside its frame".to_owned()))
    };
    for piece in pieces {
        match *piece {
            JoinPiece::Constant(text) => built.push_str(compiled.texts.get(text as usize).map_or("", String::as_str)),
            JoinPiece::Text(register) => {
                let source = get(register)?;
                match onto {
                    // A text joined onto itself: its own start is what's joined.
                    Some((target, original)) if source == target => {
                        let own = built.get(..original).unwrap_or("").to_owned();
                        built.push_str(&own);
                    }
                    _ => built.push_str(heap.text(source)?),
                }
            }
            JoinPiece::Whole(register) => append_whole(built, get(register)?.cast_signed()),
            JoinPiece::Bool(register) => built.push_str(if get(register)? == 0 { "false" } else { "true" }),
            JoinPiece::Float(register) => built.push_str(&float_to_text(f64::from_bits(get(register)?))),
            JoinPiece::Places(register, places) => {
                built.push_str(&float_with_places(f64::from_bits(get(register)?), places));
            }
            JoinPiece::NumberPlaces(register, places) => {
                built.push_str(&super::numbers::with_places(heap.number(get(register)?)?, places));
            }
            JoinPiece::Value(register, type_id) => {
                let room = heap.room_left();
                show(get(register)?, TypeId(type_id), &compiled.types, heap, &compiled.layouts, room, built)?;
            }
        }
    }
    Ok(())
}

/// The place in a list a script's `pos:` or `offset:` means, when the list has it.
#[inline]
fn item_place(index: i64, zero_based: bool, length: usize) -> Option<usize> {
    let offset = if zero_based { index } else { index.checked_sub(1)? };
    usize::try_from(offset).ok().filter(|&offset| offset < length)
}

fn append_whole(text: &mut String, value: i64) {
    let mut digits = [0u8; 20];
    let mut place = digits.len();
    let mut left = value.unsigned_abs();
    loop {
        place -= 1;
        if let Some(digit) = digits.get_mut(place) {
            // `left % 10` is a single digit.
            #[allow(clippy::cast_possible_truncation)]
            {
                *digit = b'0' + (left % 10) as u8;
            }
        }
        left /= 10;
        if left == 0 || place == 0 {
            break;
        }
    }
    if value < 0 {
        text.push('-');
    }
    for digit in digits.get(place..).unwrap_or(&[]) {
        text.push(char::from(*digit));
    }
}

/// Grows a task's stack to hold `needed` slots, counting it against the memory cap (D14).
fn grow_stack(stack: &mut Vec<u64>, heap: &mut Heap, needed: usize) -> Result<(), HeapError> {
    let new_length = needed.max(stack.len().saturating_mul(2)).max(32);
    heap.charge((new_length - stack.len()).saturating_mul(STACK_SLOT_COST))?;
    if stack.capacity() == 0 {
        // A new stack comes from the system already zeroed, instead of being written over.
        *stack = vec![0; new_length];
    } else {
        stack.resize(new_length, 0);
    }
    Ok(())
}

impl Task {
    /// A task about to run a procedure whose frame needs `frame_size` slots.
    ///
    /// # Errors
    ///
    /// Fails when the stack would pass the memory cap.
    pub fn start(&mut self, heap: &mut Heap, entry: u32, frame_size: u32) -> Result<(), HeapError> {
        self.frames.clear();
        self.pc = entry as usize;
        self.base = 0;
        self.waiting = None;
        if self.stack.len() < frame_size as usize {
            grow_stack(&mut self.stack, heap, frame_size as usize)?;
        }
        Ok(())
    }

    /// The heap values the task's frames hold where they're paused, for freeing them when the task is cancelled
    /// (D54): each call that can pause has a list of the registers holding one (D52).
    #[must_use]
    pub fn held_slots(&self, compiled: &Compiled) -> Vec<u64> {
        let mut places = vec![(self.pc, self.base)];
        places.extend(self.frames.iter().map(|frame| (frame.return_pc as usize, frame.base as usize)));
        let mut slots = Vec::new();
        for (pc, base) in places {
            let Some(registers) = u32::try_from(pc).ok().and_then(|pc| compiled.pause_maps.get(&pc)) else { continue };
            for &register in registers {
                if let Some(slot) = self.stack.get(base + usize::from(register)) {
                    slots.push(*slot);
                }
            }
        }
        slots
    }

    /// Gives back what the task's stack was counted as, when the task is over.
    pub fn finish(&mut self, heap: &mut Heap) {
        heap.refund(self.stack.len().saturating_mul(STACK_SLOT_COST));
        // The room itself is kept, emptied, for the next call to run in.
        self.stack.clear();
        self.frames.clear();
        self.waiting = None;
    }
}

/// Runs a task until it finishes, pauses or stops.
///
/// # Errors
///
/// Returns why the script stopped, with the instruction it stopped at.
#[allow(clippy::too_many_lines)] // the VM's loop: one case per instruction, kept in one place so it stays fast
#[allow(clippy::float_cmp)] // comparisons on two `Float`s are exact (Unio D184)
pub fn run(
    machine: &mut Machine,
    task: &mut Task,
    compiled: &Compiled,
    doors: &dyn DoorCaller,
    step_budget: u64,
    failures: [u32; 3],
) -> Result<Outcome, Stopped> {
    let Machine { heap, globals, texts, door_arguments } = machine;
    let code: &[Instruction] = &compiled.code;
    let layouts: &[Layout] = &compiled.layouts;
    let constants: &[u64] = &compiled.constants;
    let entries: &[ProcedureEntry] = &compiled.entries;
    let [missing, not_a_whole, full] = failures;
    let mut pc = task.pc;
    let mut base = task.base;
    let mut steps_left = step_budget;
    // The failure the last call or instruction gave, or 0. It's dealt with by the very next instruction.
    let mut failure: u32 = 0;
    let stack = &mut task.stack;
    let frames = &mut task.frames;
    // The running procedure's registers: the stack from its frame's start on. Kept as a slice of its own, so that
    // reaching a register is one check and one read.
    let mut registers: &mut [u64] = match stack.get_mut(base..) {
        Some(registers) => registers,
        None => return Err(broken("a frame outside its stack", pc)),
    };
    macro_rules! reframe {
        () => {
            registers = match stack.get_mut(base..) {
                Some(registers) => registers,
                None => return Err(broken("a frame outside its stack", pc)),
            }
        };
    }

    macro_rules! get {
        ($register:expr) => {
            match registers.get(usize::from($register)) {
                Some(value) => *value,
                None => return Err(broken("a register outside its frame", pc)),
            }
        };
    }
    macro_rules! set {
        ($register:expr, $value:expr) => {{
            let value = $value;
            match registers.get_mut(usize::from($register)) {
                Some(slot) => *slot = value,
                None => return Err(broken("a register outside its frame", pc)),
            }
        }};
    }
    macro_rules! step {
        () => {
            if steps_left == 0 {
                return Err((Stop::OutOfSteps, pc));
            }
            steps_left -= 1;
        };
    }
    // Counts bulk work, such as items copied, as steps (D14).
    macro_rules! count_work {
        () => {
            if heap.work >= BULK_UNIT {
                let steps = heap.work / BULK_UNIT;
                heap.work %= BULK_UNIT;
                if steps_left < steps {
                    return Err((Stop::OutOfSteps, pc));
                }
                steps_left -= steps;
            }
        };
    }
    macro_rules! heap_try {
        ($result:expr) => {
            match $result {
                Ok(value) => value,
                Err(error) => return Err((Stop::from(error), pc)),
            }
        };
    }
    macro_rules! jump {
        ($offset:expr) => {{
            step!();
            pc = pc.wrapping_add_signed($offset as isize);
        }};
    }
    macro_rules! short_jump {
        ($offset:expr) => {{
            step!();
            let offset = if $offset == FAR {
                match compiled.far_jumps.get(&u32::try_from(pc.wrapping_sub(1)).unwrap_or(u32::MAX)) {
                    Some(far) => *far as isize,
                    None => return Err(broken("a far jump with nowhere to go", pc)),
                }
            } else {
                $offset as isize
            };
            pc = pc.wrapping_add_signed(offset);
        }};
    }
    macro_rules! whole {
        ($register:expr) => {
            get!($register).cast_signed()
        };
    }
    macro_rules! float {
        ($register:expr) => {
            f64::from_bits(get!($register))
        };
    }
    macro_rules! whole_maths {
        ($to:expr, $left:expr, $right:expr, $method:ident, $operator:expr) => {{
            let (left, right) = (whole!($left), whole!($right));
            match left.$method(right) {
                Some(result) => set!($to, result.cast_unsigned()),
                None => return Err(overflow(left, $operator, right, pc)),
            }
        }};
    }
    macro_rules! float_maths {
        ($to:expr, $left:expr, $right:expr, $operator:tt, $text:expr) => {{
            let (left, right) = (float!($left), float!($right));
            let result = left $operator right;
            // trace: D18: a float is checked after every operation.
            if FLOAT_CHECKS && !result.is_finite() {
                return Err(float_not_real(left, $text, right, pc));
            }
            set!($to, result.to_bits());
        }};
    }
    macro_rules! compare_jump {
        ($left:expr, $right:expr, $operator:tt, $offset:expr) => {{
            if $left $operator $right {
                short_jump!($offset);
            }
        }};
    }
    // A list's items, read.
    macro_rules! items {
        ($handle:expr) => {
            match heap.get($handle) {
                Ok(Object::List { items, .. }) => items,
                Ok(_) => return Err(broken("a handle that should lead to a list", pc)),
                Err(error) => return Err((Stop::from(error), pc)),
            }
        };
    }
    // A list in a register, made this register's own so it can change (copy-on-write, D15).
    macro_rules! own {
        ($register:expr) => {{
            let handle = get!($register);
            // The usual case first: the register is the value's only holder.
            if heap.has_one_holder(handle) {
                handle
            } else {
                let own = heap_try!(heap.make_own(handle, layouts));
                set!($register, own);
                own
            }
        }};
    }
    // Looks a key up in a dictionary: how it was looked up, which pair it is when it's there, how many places were
    // looked at, and its value.
    macro_rules! lookup {
        ($dict:expr, $key:expr) => {
            match heap.get($dict) {
                Ok(Object::Dict(dict)) => {
                    let probe =
                        if dict.text_keys { Probe::of_text(heap_try!(heap.text($key))) } else { Probe::of_bits($key) };
                    let (found, looked) = dict.find(&probe, |stored| {
                        heap.text(stored).is_ok_and(|stored| heap.text($key).is_ok_and(|wanted| stored == wanted))
                    });
                    (probe, found, looked, found.and_then(|number| dict.value_at(number)))
                }
                Ok(_) => return Err(broken("a handle that should lead to a dictionary", pc)),
                Err(error) => return Err((Stop::from(error), pc)),
            }
        };
    }
    // Adds a key that isn't in a dictionary yet, with the memory it takes counted first.
    macro_rules! add_key {
        ($handle:expr, $probe:expr, $key:expr, $value:expr) => {{
            let (growth, text_keys) = match heap.get($handle) {
                Ok(Object::Dict(dict)) => (dict.growth_for_one_more(), dict.text_keys),
                Ok(_) => return Err(broken("a handle that should lead to a dictionary", pc)),
                Err(error) => return Err((Stop::from(error), pc)),
            };
            heap_try!(heap.charge(growth));
            if text_keys {
                // The dictionary becomes one more holder of a key it keeps.
                heap.retain($key);
            }
            if let Ok(Object::Dict(dict)) = heap.get_mut($handle) {
                dict.add(&$probe, $key, $value);
            }
        }};
    }
    // Gives a pair that's already there a new value, freeing the old one when it was a heap value.
    macro_rules! replace_value {
        ($handle:expr, $number:expr, $value:expr) => {{
            let old = match heap.get_mut($handle) {
                Ok(Object::Dict(dict)) => dict.replace_value($number, $value).filter(|_| dict.reference_values),
                _ => None,
            };
            if let Some(old) = old {
                heap.release(old, layouts);
            }
        }};
    }

    loop {
        let Some(&instruction) = code.get(pc) else {
            return Err(broken("running off the end of the script's instructions", pc));
        };
        pc += 1;
        match instruction {
            Instruction::Move { to, from } => set!(to, get!(from)),
            Instruction::LoadSmall { to, value } => set!(to, i64::from(value).cast_unsigned()),
            Instruction::LoadConstant { to, constant } => match constants.get(constant as usize) {
                Some(value) => set!(to, *value),
                None => return Err(broken("a constant that isn't there", pc)),
            },
            Instruction::LoadText { to, text } => match texts.get(text as usize) {
                Some(handle) => set!(to, *handle),
                None => return Err(broken("a text that isn't there", pc)),
            },
            Instruction::LoadPlainCase { to, case } => set!(to, PLAIN_CASE | u64::from(case)),
            Instruction::GetGlobal { to, global } => match globals.get(usize::from(global)) {
                Some(value) => set!(to, *value),
                None => return Err(broken("a top-level name that isn't there", pc)),
            },
            Instruction::SetGlobal { global, from } => {
                let value = get!(from);
                match globals.get_mut(usize::from(global)) {
                    Some(slot) => *slot = value,
                    None => return Err(broken("a top-level name that isn't there", pc)),
                }
            }
            Instruction::Retain { register } => heap.retain(get!(register)),
            Instruction::ReleaseText { register } => heap.release_text(get!(register)),
            Instruction::Release { register } => {
                heap.release(get!(register), layouts);
                count_work!();
            }

            Instruction::AddWhole { to, left, right } => whole_maths!(to, left, right, checked_add, "+"),
            Instruction::SubtractWhole { to, left, right } => whole_maths!(to, left, right, checked_sub, "-"),
            Instruction::MultiplyWhole { to, left, right } => whole_maths!(to, left, right, checked_mul, "*"),
            Instruction::AddWholeSmall { to, left, right } => {
                let left = whole!(left);
                match left.checked_add(i64::from(right)) {
                    Some(result) => set!(to, result.cast_unsigned()),
                    None if right < 0 => return Err(overflow(left, "-", -i64::from(right), pc)),
                    None => return Err(overflow(left, "+", i64::from(right), pc)),
                }
            }
            Instruction::AddWholeConstant { to, left, constant }
            | Instruction::MultiplyWholeConstant { to, left, constant } => {
                let Some(right) = constants.get(usize::from(constant)).map(|bits| bits.cast_signed()) else {
                    return Err(broken("a constant that isn't there", pc));
                };
                let left = whole!(left);
                let adding = matches!(instruction, Instruction::AddWholeConstant { .. });
                match if adding { left.checked_add(right) } else { left.checked_mul(right) } {
                    Some(result) => set!(to, result.cast_unsigned()),
                    None => return Err(overflow(left, if adding { "+" } else { "*" }, right, pc)),
                }
            }
            Instruction::DivideFloorConstant { to, left, constant }
            | Instruction::RemainderFloorConstant { to, left, constant } => {
                let Some(right) = constants.get(usize::from(constant)).map(|bits| bits.cast_signed()) else {
                    return Err(broken("a constant that isn't there", pc));
                };
                let left = whole!(left);
                // The compiler only uses these for a constant that's neither 0 nor -1, so nothing here can fail.
                let remainder = left.wrapping_rem(right);
                let uneven = remainder != 0 && ((remainder < 0) != (right < 0));
                let result = if matches!(instruction, Instruction::DivideFloorConstant { .. }) {
                    // The quotient comes from the remainder already in hand, with no second division.
                    let quotient = left.wrapping_sub(remainder).wrapping_div(right);
                    if uneven { quotient.wrapping_sub(1) } else { quotient }
                } else if uneven {
                    remainder.wrapping_add(right)
                } else {
                    remainder
                };
                set!(to, result.cast_unsigned());
            }
            Instruction::DivideFloorByPowerOfTwo { to, left, shift } => {
                set!(to, whole!(left).wrapping_shr(u32::from(shift)).cast_unsigned());
            }
            Instruction::RemainderFloorByPowerOfTwo { to, left, bits } => {
                let mask = 1u64.wrapping_shl(u32::from(bits)).wrapping_sub(1);
                set!(to, get!(left) & mask);
            }
            Instruction::NegateWhole { to, from } => {
                let value = whole!(from);
                match value.checked_neg() {
                    Some(result) => set!(to, result.cast_unsigned()),
                    None => {
                        return Err(check_failed(
                            "the result is too big for a `Whole`",
                            format!("`-({value})`"),
                            WHOLE_RANGE,
                            pc,
                        ));
                    }
                }
            }
            Instruction::DivideWhole { to, left, right } => {
                let (left, right) = (whole!(left), whole!(right));
                if right == 0 {
                    return Err(divided_by_zero(&left.to_string(), "/", pc));
                }
                if left.wrapping_rem(right) != 0 {
                    // trace: Unio D70
                    return Err(check_failed(
                        "this division leaves a remainder",
                        format!("`{left} / {right}`"),
                        "`/` on two `Whole`s divides exactly; `div.floor` and `div.trunc` say how a fraction goes.",
                        pc,
                    ));
                }
                match left.checked_div(right) {
                    Some(result) => set!(to, result.cast_unsigned()),
                    None => return Err(overflow(left, "/", right, pc)),
                }
            }
            Instruction::DivideFloor { to, left, right } | Instruction::DivideTrunc { to, left, right } => {
                let floor = matches!(instruction, Instruction::DivideFloor { .. });
                let name = if floor { "div.floor" } else { "div.trunc" };
                let (left, right) = (whole!(left), whole!(right));
                if right == 0 {
                    return Err(divided_by_zero(&left.to_string(), name, pc));
                }
                let Some(quotient) = left.checked_div(right) else {
                    return Err(overflow(left, name, right, pc));
                };
                let remainder = left.wrapping_rem(right);
                // Rust divides towards zero; rounding down takes one more off when the signs differ (Unio D70).
                let down = floor && remainder != 0 && ((remainder < 0) != (right < 0));
                set!(to, (if down { quotient - 1 } else { quotient }).cast_unsigned());
            }
            Instruction::RemainderFloor { to, left, right } | Instruction::RemainderTrunc { to, left, right } => {
                let floor = matches!(instruction, Instruction::RemainderFloor { .. });
                let (left, right) = (whole!(left), whole!(right));
                if right == 0 {
                    return Err(divided_by_zero(&left.to_string(), if floor { "rem.floor" } else { "rem.trunc" }, pc));
                }
                let remainder = left.wrapping_rem(right);
                // `rem.floor` takes the sign of the number divided by, matching `div.floor` (Unio D71).
                let adjust = floor && remainder != 0 && ((remainder < 0) != (right < 0));
                set!(to, (if adjust { remainder + right } else { remainder }).cast_unsigned());
            }

            Instruction::AddFloat { to, left, right } => float_maths!(to, left, right, +, "+"),
            Instruction::SubtractFloat { to, left, right } => float_maths!(to, left, right, -, "-"),
            Instruction::MultiplyFloat { to, left, right } => float_maths!(to, left, right, *, "*"),
            Instruction::FloatFused { shape, ops, to, a, b, c, d } => {
                // One operation: worked out and checked exactly as its own instruction would. `$at` finds its place.
                macro_rules! one {
                    ($code:expr, $left:expr, $right:expr, $at:expr) => {{
                        let (left, right): (f64, f64) = ($left, $right);
                        let (result, operator) = match $code {
                            0 => (left + right, "+"),
                            1 => (left - right, "-"),
                            2 => (left * right, "*"),
                            _ => {
                                if right == 0.0 {
                                    return Err(divided_by_zero(&float_to_text(left), "/", $at));
                                }
                                (left / right, "/")
                            }
                        };
                        if FLOAT_CHECKS && !result.is_finite() {
                            return Err(float_not_real(left, operator, right, $at));
                        }
                        result
                    }};
                }
                let (first, second, third) = (ops & 3, (ops >> 2) & 3, (ops >> 4) & 3);
                let (a, b, c) = (float!(a), float!(b), float!(c));
                // The last operation's place is this instruction's own; the earlier ones' follow it.
                let result = match shape {
                    0 => {
                        let inner = one!(first, a, b, pc + 1);
                        pc += 1;
                        one!(second, inner, c, pc - 1)
                    }
                    1 => {
                        let inner = one!(first, b, c, pc + 1);
                        pc += 1;
                        one!(second, a, inner, pc - 1)
                    }
                    _ => {
                        let d = float!(d);
                        let (at_first, at_second, at_last) = (pc + 1, pc + 2, pc);
                        pc += 2;
                        match shape {
                            2 => {
                                let inner = one!(first, a, b, at_first);
                                let middle = one!(second, inner, c, at_second);
                                one!(third, middle, d, at_last)
                            }
                            3 => {
                                let inner = one!(first, b, c, at_first);
                                let middle = one!(second, a, inner, at_second);
                                one!(third, middle, d, at_last)
                            }
                            4 => {
                                let left = one!(first, a, b, at_first);
                                let right = one!(second, c, d, at_second);
                                one!(third, left, right, at_last)
                            }
                            5 => {
                                let inner = one!(first, b, c, at_first);
                                let middle = one!(second, inner, d, at_second);
                                one!(third, a, middle, at_last)
                            }
                            _ => {
                                let inner = one!(first, c, d, at_first);
                                let middle = one!(second, b, inner, at_second);
                                one!(third, a, middle, at_last)
                            }
                        }
                    }
                };
                set!(to, result.to_bits());
            }
            Instruction::DivideFloat { to, left, right } => {
                let (left, right) = (float!(left), float!(right));
                if right == 0.0 {
                    return Err(divided_by_zero(&float_to_text(left), "/", pc));
                }
                let result = left / right;
                if FLOAT_CHECKS && !result.is_finite() {
                    return Err(float_not_real(left, "/", right, pc));
                }
                set!(to, result.to_bits());
            }
            Instruction::AddFloatConstant { to, left, constant }
            | Instruction::MultiplyFloatConstant { to, left, constant }
            | Instruction::DivideFloatConstant { to, left, constant } => {
                let Some(right) = constants.get(usize::from(constant)).map(|bits| f64::from_bits(*bits)) else {
                    return Err(broken("a constant that isn't there", pc));
                };
                let left = float!(left);
                let (result, operator) = match instruction {
                    Instruction::AddFloatConstant { .. } => (left + right, "+"),
                    Instruction::MultiplyFloatConstant { .. } => (left * right, "*"),
                    _ => (left / right, "/"),
                };
                // trace: D18
                if FLOAT_CHECKS && !result.is_finite() {
                    return Err(float_not_real(left, operator, right, pc));
                }
                set!(to, result.to_bits());
            }
            Instruction::DivideConstantByFloat { to, constant, right } => {
                let Some(left) = constants.get(usize::from(constant)).map(|bits| f64::from_bits(*bits)) else {
                    return Err(broken("a constant that isn't there", pc));
                };
                let right = float!(right);
                if right == 0.0 {
                    return Err(divided_by_zero(&float_to_text(left), "/", pc));
                }
                let result = left / right;
                if FLOAT_CHECKS && !result.is_finite() {
                    return Err(float_not_real(left, "/", right, pc));
                }
                set!(to, result.to_bits());
            }
            Instruction::NegateFloat { to, from } => set!(to, (-float!(from)).to_bits()),
            Instruction::SquareRoot { to, from } => {
                let value = float!(from);
                if value < 0.0 {
                    // trace: Unio D185
                    return Err(check_failed(
                        "a negative number has no square root",
                        format!("`sqrt({})`", float_to_text(value)),
                        "the square root of a negative number isn't a real number.",
                        pc,
                    ));
                }
                set!(to, value.sqrt().to_bits());
            }

            Instruction::EqualWhole { to, left, right } => set!(to, u64::from(get!(left) == get!(right))),
            Instruction::NotEqualWhole { to, left, right } => set!(to, u64::from(get!(left) != get!(right))),
            Instruction::LessWhole { to, left, right } => set!(to, u64::from(whole!(left) < whole!(right))),
            Instruction::AtMostWhole { to, left, right } => set!(to, u64::from(whole!(left) <= whole!(right))),
            Instruction::EqualText { to, left, right } | Instruction::NotEqualText { to, left, right } => {
                let first = heap_try!(heap.text(get!(left)));
                let second = heap_try!(heap.text(get!(right)));
                // Texts of different lengths differ at once; the same length is compared byte by byte, which is work.
                let (same, work) =
                    if first.len() == second.len() { (first == second, first.len()) } else { (false, 0) };
                heap.work = heap.work.saturating_add(work as u64);
                count_work!();
                set!(to, u64::from(same == matches!(instruction, Instruction::EqualText { .. })));
            }
            Instruction::EqualFloat { to, left, right } => set!(to, u64::from(float!(left) == float!(right))),
            Instruction::NotEqualFloat { to, left, right } => set!(to, u64::from(float!(left) != float!(right))),
            Instruction::LessFloat { to, left, right } => set!(to, u64::from(float!(left) < float!(right))),
            Instruction::AtMostFloat { to, left, right } => set!(to, u64::from(float!(left) <= float!(right))),
            Instruction::Not { to, from } => set!(to, u64::from(get!(from) == 0)),

            Instruction::Jump { offset } => jump!(offset),
            Instruction::JumpIfTrue { register, offset } => {
                if get!(register) != 0 {
                    jump!(offset);
                }
            }
            Instruction::JumpIfFalse { register, offset } => {
                if get!(register) == 0 {
                    jump!(offset);
                }
            }
            Instruction::JumpIfEqualWhole { left, right, offset } => compare_jump!(get!(left), get!(right), ==, offset),
            Instruction::JumpIfNotEqualWhole { left, right, offset } => {
                compare_jump!(get!(left), get!(right), !=, offset);
            }
            Instruction::JumpIfLessWhole { left, right, offset } => {
                compare_jump!(whole!(left), whole!(right), <, offset);
            }
            Instruction::JumpIfAtMostWhole { left, right, offset } => {
                compare_jump!(whole!(left), whole!(right), <=, offset);
            }
            Instruction::JumpIfEqualWholeSmall { left, right, offset } => {
                compare_jump!(whole!(left), i64::from(right), ==, offset);
            }
            Instruction::JumpIfNotEqualWholeSmall { left, right, offset } => {
                compare_jump!(whole!(left), i64::from(right), !=, offset);
            }
            Instruction::JumpIfLessWholeSmall { left, right, offset } => {
                compare_jump!(whole!(left), i64::from(right), <, offset);
            }
            Instruction::JumpIfAtMostWholeSmall { left, right, offset } => {
                compare_jump!(whole!(left), i64::from(right), <=, offset);
            }
            Instruction::JumpIfGreaterWholeSmall { left, right, offset } => {
                compare_jump!(whole!(left), i64::from(right), >, offset);
            }
            Instruction::JumpIfAtLeastWholeSmall { left, right, offset } => {
                compare_jump!(whole!(left), i64::from(right), >=, offset);
            }
            Instruction::JumpIfEqualFloat { left, right, offset } => {
                compare_jump!(float!(left), float!(right), ==, offset);
            }
            Instruction::JumpIfNotEqualFloat { left, right, offset } => {
                compare_jump!(float!(left), float!(right), !=, offset);
            }
            Instruction::JumpIfLessFloat { left, right, offset } => {
                compare_jump!(float!(left), float!(right), <, offset);
            }
            Instruction::JumpIfAtMostFloat { left, right, offset } => {
                compare_jump!(float!(left), float!(right), <=, offset);
            }

            Instruction::Call { base: call_base, procedure } => {
                step!();
                let Some(callee) = entries.get(procedure as usize) else {
                    return Err(broken("a call to a procedure that isn't there", pc));
                };
                let new_base = base + usize::from(call_base);
                let needed = new_base + callee.frame_size as usize;
                if needed > stack.len() {
                    heap_try!(grow_stack(stack, heap, needed));
                }
                let (Ok(return_pc), Ok(old_base)) = (u32::try_from(pc), u32::try_from(base)) else {
                    return Err(broken("a script too big to call into", pc));
                };
                frames.push(Frame { return_pc, base: old_base });
                base = new_base;
                pc = callee.entry as usize;
                reframe!();
            }
            Instruction::Return { from } => {
                let value = get!(from);
                set!(0u16, value);
                if let Some(frame) = frames.pop() {
                    pc = frame.return_pc as usize;
                    base = frame.base as usize;
                    reframe!();
                } else {
                    task.pc = pc;
                    task.base = base;
                    return Ok(Outcome::Finished);
                }
            }
            Instruction::ReturnNothing => {
                if let Some(frame) = frames.pop() {
                    pc = frame.return_pc as usize;
                    base = frame.base as usize;
                    reframe!();
                } else {
                    task.pc = pc;
                    task.base = base;
                    return Ok(Outcome::Finished);
                }
            }
            Instruction::CallDoor { base: call_base, site } => {
                step!();
                let Some(door_site) = compiled.door_sites.get(site as usize) else {
                    return Err(broken("a call to a door that isn't there", pc));
                };
                let first = usize::from(call_base) + 1;
                if !door_site.plain {
                    // Only arguments that are more than their bits are copied out as values (D15).
                    door_arguments.clear();
                    for (register, crossing) in (call_base + 1..).zip(&door_site.arguments) {
                        let slot = get!(register);
                        door_arguments.push(match plain_value(slot, crossing) {
                            Some(_) => Value::Nothing,
                            None => heap_try!(to_value(slot, crossing, heap, layouts)),
                        });
                    }
                }
                let Some(bits) = registers.get(first..first + door_site.arguments.len()) else {
                    return Err(broken("a door's arguments outside their frame", pc));
                };
                match doors.call_door(door_site.door, bits, door_arguments) {
                    DoorAnswer::Plain(bits) => {
                        let fits = match door_site.result {
                            Crossing::Nothing | Crossing::Whole | Crossing::Bool => true,
                            Crossing::Float => f64::from_bits(bits).is_finite(),
                            _ => false,
                        };
                        if !fits {
                            return Err((
                                Stop::HostMistake(
                                    "a door gave a value that isn't what it's declared to give".to_owned(),
                                ),
                                pc,
                            ));
                        }
                        set!(call_base, bits);
                    }
                    DoorAnswer::Value(value) => match plain_slot(&value, &door_site.result)
                        .map_or_else(|| to_slot(&value, &door_site.result, heap, layouts), Ok)
                    {
                        Ok(slot) => set!(call_base, slot),
                        Err(CrossingError::Heap(error)) => return Err((Stop::from(error), pc)),
                        Err(CrossingError::WrongType { wanted, found }) => {
                            return Err((
                                Stop::HostMistake(format!("a door gave {found}, but it's declared to give {wanted}")),
                                pc,
                            ));
                        }
                    },
                    DoorAnswer::Failure(name) => {
                        let found = (1u32..).zip(&compiled.failure_names).find(|(_, known)| **known == name);
                        match found {
                            Some((number, _)) => failure = number,
                            None => {
                                return Err((
                                    Stop::HostMistake(format!("a door failed with `{name}`, which it doesn't declare")),
                                    pc,
                                ));
                            }
                        }
                    }
                    DoorAnswer::Mistake(detail) => return Err((Stop::HostMistake(detail), pc)),
                    DoorAnswer::Pause => {
                        task.pc = pc;
                        task.base = base;
                        task.waiting = Some(Waiting { slot: base + usize::from(call_base), site });
                        return Ok(Outcome::Paused);
                    }
                }
                count_work!();
            }

            Instruction::Fail { failure: given } => {
                failure = u32::from(given);
                match frames.pop() {
                    Some(frame) => {
                        pc = frame.return_pc as usize;
                        base = frame.base as usize;
                        reframe!();
                    }
                    None => return Ok(Outcome::Failed(failure)),
                }
            }
            Instruction::PassFailureUp => match frames.pop() {
                Some(frame) => {
                    pc = frame.return_pc as usize;
                    base = frame.base as usize;
                    reframe!();
                }
                None => return Ok(Outcome::Failed(failure)),
            },
            Instruction::JumpUnlessFailed { offset } => {
                if failure == 0 {
                    jump!(offset);
                }
            }
            Instruction::JumpUnlessFailureIs { failure: wanted, offset } => {
                if failure != u32::from(wanted) {
                    jump!(offset);
                }
            }
            Instruction::ClearFailure => failure = 0,
            Instruction::Crash => return Err((Stop::Crashed { failure }, pc)),
            // The instructions a script meets rarely, or that do a lot at once, share one case here and are worked
            // in a function of their own, so the loop's own machine code stays small and its hot paths fast.
            Instruction::MakeRange { .. }
            | Instruction::EqualValues { .. }
            | Instruction::PowerWhole { .. }
            | Instruction::PowerFloat { .. }
            | Instruction::Maths { .. }
            | Instruction::CheckLimits { .. }
            | Instruction::CrashIfNone { .. }
            | Instruction::Scale { .. }
            | Instruction::TextToBool { .. }
            | Instruction::JoinPieces { .. }
            | Instruction::AppendPieces { .. }
            | Instruction::LoadNumber { .. }
            | Instruction::NumberOperation { .. }
            | Instruction::ConvertNumber { .. } => {
                match rare_instruction(instruction, registers, heap, compiled, pc, steps_left) {
                    Ok(Some(failed)) => failure = failed,
                    Ok(None) => {}
                    Err(stopped) => return Err(stopped),
                }
                count_work!();
            }
            Instruction::TextToWhole { to, from } => match text_to_whole(heap_try!(heap.text(get!(from)))) {
                Some(value) => set!(to, value.cast_unsigned()),
                None => failure = not_a_whole,
            },
            Instruction::WholeToFloat { to, from } => {
                // A `Whole` too long for a `Float`'s digits goes to the nearest `Float`, as every float step does.
                #[allow(clippy::cast_precision_loss)]
                let value = whole!(from) as f64;
                set!(to, value.to_bits());
            }
            Instruction::FloatToWhole { to, from, rounding } => {
                let value = float!(from);
                let (rounding, name) = match rounding {
                    0 => (Rounding::Floor, "floor"),
                    1 => (Rounding::Trunc, "trunc"),
                    2 => (Rounding::Ceil, "ceil"),
                    3 => (Rounding::Nearest, "nearest"),
                    _ => (Rounding::NearestEven, "nearest.even"),
                };
                match float_to_whole(value, rounding) {
                    Some(result) => set!(to, result.cast_unsigned()),
                    None => {
                        // trace: Unio D178
                        return Err(check_failed(
                            "the result is too big for a `Whole`",
                            format!("`be.Whole.{name}({})`", float_to_text(value)),
                            WHOLE_RANGE,
                            pc,
                        ));
                    }
                }
            }

            Instruction::NewList { to, base: first, count }
            | Instruction::NewReferenceList { to, base: first, count } => {
                let references = matches!(instruction, Instruction::NewReferenceList { .. });
                let start = usize::from(first);
                let Some(values) = registers.get(start..start + usize::from(count)) else {
                    return Err(broken("a list built from outside its frame", pc));
                };
                let size = values.len().saturating_mul(SLOT_COST);
                heap_try!(heap.charge(size));
                heap.refund(size);
                let mut items = Vec::new();
                items.reserve_exact(values.len());
                items.extend_from_slice(values);
                heap.work = heap.work.saturating_add(u64::from(count));
                set!(to, heap_try!(heap.allocate(Object::List { items, references }, size)));
                count_work!();
            }
            Instruction::RepeatList { to, value, count } | Instruction::RepeatReferenceList { to, value, count } => {
                let references = matches!(instruction, Instruction::RepeatReferenceList { .. });
                let value = get!(value);
                let Ok(count) = usize::try_from(whole!(count)) else {
                    return Err(broken("an array with a length below zero", pc));
                };
                let size = count.saturating_mul(SLOT_COST);
                heap_try!(heap.charge(size));
                heap.refund(size);
                heap.work = heap.work.saturating_add(count as u64);
                count_work!();
                let mut items = Vec::new();
                items.reserve_exact(count);
                items.resize(count, value);
                if references {
                    for _ in 0..count {
                        heap.retain(value);
                    }
                }
                set!(to, heap_try!(heap.allocate(Object::List { items, references }, size)));
            }
            Instruction::ItemAtPosition { to, list, index } | Instruction::ItemAtOffset { to, list, index } => {
                let zero_based = matches!(instruction, Instruction::ItemAtOffset { .. });
                let index = whole!(index);
                let items = items!(get!(list));
                match item_place(index, zero_based, items.len()).and_then(|place| items.get(place)) {
                    Some(item) => set!(to, *item),
                    None => {
                        return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, items.len(), pc));
                    }
                }
            }
            Instruction::SetItemAtPosition { list, index, value }
            | Instruction::SetItemAtOffset { list, index, value } => {
                let zero_based = matches!(instruction, Instruction::SetItemAtOffset { .. });
                let index = whole!(index);
                let value = get!(value);
                let handle = own!(list);
                count_work!();
                let old = match heap.get_mut(handle) {
                    Ok(Object::List { items, references }) => {
                        let length = items.len();
                        match item_place(index, zero_based, length).and_then(|place| items.get_mut(place)) {
                            Some(slot) => {
                                let old = std::mem::replace(slot, value);
                                references.then_some(old)
                            }
                            None => {
                                return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, length, pc));
                            }
                        }
                    }
                    Ok(_) => return Err(broken("a handle that should lead to a list", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                if let Some(old) = old {
                    heap.release(old, layouts);
                }
            }
            Instruction::AddItem { list, value }
            | Instruction::InsertItemAtPosition { list, value, .. }
            | Instruction::InsertItemAtOffset { list, value, .. } => {
                let value = get!(value);
                let handle = own!(list);
                let (length, capacity) = {
                    let items = items!(handle);
                    (items.len(), items.capacity())
                };
                // Where the item goes: the end, or a place from the first to one past the last (Unio D80).
                let place = match instruction {
                    Instruction::InsertItemAtPosition { index, .. } => {
                        let index = whole!(index);
                        match index
                            .checked_sub(1)
                            .and_then(|offset| usize::try_from(offset).ok())
                            .filter(|&offset| offset <= length)
                        {
                            Some(place) => place,
                            None => return Err(bad_position("pos", index, length + 1, pc)),
                        }
                    }
                    Instruction::InsertItemAtOffset { index, .. } => {
                        let index = whole!(index);
                        match usize::try_from(index).ok().filter(|&offset| offset <= length) {
                            Some(place) => place,
                            None => return Err(bad_position("offset", index, length + 1, pc)),
                        }
                    }
                    _ => length,
                };
                let reserve = heap_try!(heap.grow(length, capacity, SLOT_COST));
                heap.work = heap.work.saturating_add((length - place) as u64);
                if let Ok(Object::List { items, .. }) = heap.get_mut(handle) {
                    if let Some(reserve) = reserve {
                        items.reserve_exact(reserve);
                    }
                    items.insert(place, value);
                }
                count_work!();
            }
            // trace: D73
            Instruction::FitList { to, from, limit } => {
                let handle = get!(from);
                let length = items!(handle).len();
                if i64::try_from(length).unwrap_or(i64::MAX) > whole!(limit) {
                    failure = full;
                } else {
                    heap.retain(handle);
                    set!(to, handle);
                }
            }
            Instruction::CheckRoom { list, limit } => {
                let limit = whole!(limit);
                let length = items!(get!(list)).len();
                if i64::try_from(length).unwrap_or(i64::MAX) >= limit {
                    failure = full;
                }
            }
            Instruction::RemoveItemAtPosition { list, index } | Instruction::RemoveItemAtOffset { list, index } => {
                let zero_based = matches!(instruction, Instruction::RemoveItemAtOffset { .. });
                let index = whole!(index);
                let handle = own!(list);
                let removed = match heap.get_mut(handle) {
                    Ok(Object::List { items, references }) => {
                        let length = items.len();
                        match item_place(index, zero_based, length) {
                            Some(place) => {
                                let removed = items.remove(place);
                                (references.then_some(removed), length - place)
                            }
                            None => {
                                return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, length, pc));
                            }
                        }
                    }
                    Ok(_) => return Err(broken("a handle that should lead to a list", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                heap.work = heap.work.saturating_add(removed.1 as u64);
                if let Some(old) = removed.0 {
                    heap.release(old, layouts);
                }
                count_work!();
            }
            Instruction::ItemFieldAtPosition { to, list, index, field }
            | Instruction::ItemFieldAtOffset { to, list, index, field } => {
                let zero_based = matches!(instruction, Instruction::ItemFieldAtOffset { .. });
                let index = whole!(index);
                let items = items!(get!(list));
                let Some(&item) = item_place(index, zero_based, items.len()).and_then(|place| items.get(place)) else {
                    return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, items.len(), pc));
                };
                let value = match heap.get(item) {
                    Ok(Object::Record { fields, .. }) => fields.get(usize::from(field)).copied(),
                    Ok(_) => None,
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                match value {
                    Some(value) => set!(to, value),
                    None => return Err(broken("a field that isn't there", pc)),
                }
            }
            Instruction::SetItemFieldAtPosition { list, index, field, value }
            | Instruction::SetItemFieldAtOffset { list, index, field, value } => {
                let zero_based = matches!(instruction, Instruction::SetItemFieldAtOffset { .. });
                let index = whole!(index);
                let value = get!(value);
                let parent = own!(list);
                let items = items!(parent);
                let length = items.len();
                let place = item_place(index, zero_based, length);
                let Some((place, child)) = place.and_then(|place| items.get(place).map(|child| (place, *child))) else {
                    return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, length, pc));
                };
                let own = if heap.has_one_holder(child) {
                    child
                } else {
                    let own = heap_try!(heap.make_own(child, layouts));
                    if let Ok(Object::List { items, .. }) = heap.get_mut(parent)
                        && let Some(slot) = items.get_mut(place)
                    {
                        *slot = own;
                    }
                    own
                };
                let written = match heap.get_mut(own) {
                    Ok(Object::Record { fields, .. }) => fields.get_mut(usize::from(field)).map(|slot| *slot = value),
                    _ => None,
                };
                if written.is_none() {
                    return Err(broken("a field that isn't there", pc));
                }
                count_work!();
            }
            // trace: D62. A way into a part of a name: each step is made singly held where it sits, top down.
            Instruction::Own { register } => {
                own!(register);
                count_work!();
            }
            Instruction::OwnField { to, record, field } => {
                let parent = get!(record);
                let child = match heap.get(parent) {
                    Ok(Object::Record { fields, .. }) => fields.get(usize::from(field)).copied(),
                    Ok(_) => None,
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                let Some(child) = child else {
                    return Err(broken("a field reached in something that has none", pc));
                };
                let own = if heap.has_one_holder(child) {
                    child
                } else {
                    let own = heap_try!(heap.make_own(child, layouts));
                    if let Ok(Object::Record { fields, .. }) = heap.get_mut(parent)
                        && let Some(slot) = fields.get_mut(usize::from(field))
                    {
                        *slot = own;
                    }
                    own
                };
                count_work!();
                set!(to, own);
            }
            Instruction::OwnItemAtPosition { to, list, index } | Instruction::OwnItemAtOffset { to, list, index } => {
                let zero_based = matches!(instruction, Instruction::OwnItemAtOffset { .. });
                let index = whole!(index);
                let parent = get!(list);
                let items = items!(parent);
                let length = items.len();
                let place = item_place(index, zero_based, length);
                let Some((place, child)) = place.and_then(|place| items.get(place).map(|child| (place, *child))) else {
                    return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, length, pc));
                };
                let own = if heap.has_one_holder(child) {
                    child
                } else {
                    let own = heap_try!(heap.make_own(child, layouts));
                    if let Ok(Object::List { items, .. }) = heap.get_mut(parent)
                        && let Some(slot) = items.get_mut(place)
                    {
                        *slot = own;
                    }
                    own
                };
                count_work!();
                set!(to, own);
            }
            // trace: D65, D66
            Instruction::TextLength { to, from, unit } => {
                let text = heap_try!(heap.text(get!(from)));
                let (length, work) = match unit {
                    0 => (grapheme_count(text), text.len()),
                    1 => (text.chars().count(), text.len()),
                    _ => (text.len(), 0),
                };
                heap.work = heap.work.saturating_add(work as u64);
                count_work!();
                set!(to, length as u64);
            }
            Instruction::GraphemeAtPosition { to, text, index } | Instruction::GraphemeAtOffset { to, text, index } => {
                let zero_based = matches!(instruction, Instruction::GraphemeAtOffset { .. });
                let index = whole!(index);
                let written = heap_try!(heap.text(get!(text)));
                let offset = if zero_based { Some(index) } else { index.checked_sub(1) };
                let found = offset
                    .and_then(|offset| usize::try_from(offset).ok())
                    .and_then(|offset| grapheme_at(written, offset));
                let Some(found) = found else {
                    let length = grapheme_count(written);
                    return Err(bad_text_position(if zero_based { "offset" } else { "pos" }, index, length, pc));
                };
                let (grapheme, work) = (found.to_owned(), written.len());
                heap.work = heap.work.saturating_add(work as u64);
                count_work!();
                let handle = heap_try!(heap.new_text(&grapheme, grapheme.len()));
                set!(to, handle);
            }
            Instruction::Graphemes { to, from } => {
                let written = heap_try!(heap.text(get!(from)));
                let pieces: Vec<String> = graphemes(written).map(str::to_owned).collect();
                let work = written.len();
                let handle = heap_try!(text_list(heap, &pieces, work, layouts));
                count_work!();
                set!(to, handle);
            }
            Instruction::SplitText { to, text, by } | Instruction::SplitTextNoEmpty { to, text, by } => {
                let keep_empty = matches!(instruction, Instruction::SplitText { .. });
                let written = heap_try!(heap.text(get!(text)));
                let by = heap_try!(heap.text(get!(by)));
                if by.is_empty() {
                    // trace: D89. A separator that only turns out empty while running is a failed check.
                    return Err(check_failed(
                        "this splits a text by an empty text",
                        "the text to split at is `\"\"`".to_owned(),
                        "an empty text is never found between two pieces; `for each` goes through a text's characters.",
                        pc,
                    ));
                }
                let pieces: Vec<String> =
                    written.split(by).filter(|piece| keep_empty || !piece.is_empty()).map(str::to_owned).collect();
                let work = written.len();
                let handle = heap_try!(text_list(heap, &pieces, work, layouts));
                count_work!();
                set!(to, handle);
            }
            Instruction::Length { to, from } => {
                let length = match heap.get(get!(from)) {
                    Ok(Object::List { items, .. }) => items.len(),
                    Ok(Object::Dict(dict)) => dict.len(),
                    Ok(_) => return Err(broken("a length of something that has none", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                set!(to, length as u64);
            }
            Instruction::NextItem { to, base: loop_base, offset } => {
                let done = get!(loop_base + 1);
                let items = items!(get!(loop_base));
                if let Some(item) = usize::try_from(done).ok().and_then(|done| items.get(done)) {
                    let item = *item;
                    set!(to, item);
                    set!(loop_base + 1, done + 1);
                    short_jump!(offset);
                }
            }

            Instruction::PrepareRangeTo { base: range } | Instruction::PrepareRangeBefore { base: range } => {
                let inclusive = matches!(instruction, Instruction::PrepareRangeTo { .. });
                let (from, end, step) = (whole!(range), whole!(range + 1), whole!(range + 2));
                let label = if inclusive { "to" } else { "before" };
                let when = || format!("`range[from: {from}, {label}: {end}, step_by: {step}]`");
                if step == 0 {
                    // trace: Unio D172
                    return Err(check_failed(
                        "this range's step is 0",
                        when(),
                        "a range with a step of 0 never reaches its end.",
                        pc,
                    ));
                }
                let away = if step > 0 { from > end } else { from < end };
                if away {
                    return Err(check_failed(
                        "this range's step points away from its end",
                        when(),
                        "a range's step points towards its end; a negative `step_by:` counts down.",
                        pc,
                    ));
                }
                // Worked out once, in numbers wide enough that nothing overflows: how many passes the loop makes.
                // The counter then needs no overflow check of its own (D17).
                let distance = (i128::from(end) - i128::from(from)).unsigned_abs();
                let stride = i128::from(step).unsigned_abs();
                let passes = if inclusive { distance / stride + 1 } else { distance.div_ceil(stride) };
                set!(range + 1, u64::try_from(passes).unwrap_or(u64::MAX));
            }
            Instruction::NextInRange { to, base: range, offset } => {
                let left = get!(range + 1);
                if left != 0 {
                    let current = get!(range);
                    set!(to, current);
                    set!(range + 1, left - 1);
                    // The number after the last one is never used, so it may wrap.
                    set!(range, current.wrapping_add(get!(range + 2)));
                    short_jump!(offset);
                }
            }

            Instruction::NewDict { to, shape } => {
                let dict = Dict::new(shape & 1 != 0, shape & 2 != 0);
                set!(to, heap_try!(heap.allocate(Object::Dict(Box::new(dict)), 0)));
            }
            Instruction::GetKey { to, dict, key } => {
                let (dict, key) = (get!(dict), get!(key));
                let (_, _, looked, value) = lookup!(dict, key);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                match value {
                    Some(value) => set!(to, value),
                    None => failure = missing,
                }
                count_work!();
            }
            Instruction::GetKeyIfAny { to, dict, key } => {
                let (dict, key) = (get!(dict), get!(key));
                let (_, _, looked, value) = lookup!(dict, key);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                if let Some(value) = value {
                    set!(to, value);
                }
                count_work!();
            }
            Instruction::GetFoundIfAny { to, dict, found } => {
                let value = match heap.get(get!(dict)) {
                    Ok(Object::Dict(dict)) => {
                        usize::try_from(get!(found)).ok().and_then(|number| dict.value_at(number))
                    }
                    Ok(_) => return Err(broken("a handle that should lead to a dictionary", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                if let Some(value) = value {
                    set!(to, value);
                }
            }
            Instruction::HasKey { to, dict, key } => {
                let (dict, key) = (get!(dict), get!(key));
                let (_, found, looked, _) = lookup!(dict, key);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                set!(to, u64::from(found.is_some()));
                count_work!();
            }
            Instruction::FindKey { to, dict, key } => {
                let key = get!(key);
                let handle = own!(dict);
                let (_, found, looked, _) = lookup!(handle, key);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                set!(to, found.map_or(u64::MAX, |number| number as u64));
                count_work!();
            }
            Instruction::GetFound { to, dict, found } => {
                let value = match heap.get(get!(dict)) {
                    Ok(Object::Dict(dict)) => {
                        usize::try_from(get!(found)).ok().and_then(|number| dict.value_at(number))
                    }
                    Ok(_) => return Err(broken("a handle that should lead to a dictionary", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                match value {
                    Some(value) => set!(to, value),
                    None => failure = missing,
                }
            }
            Instruction::SetFound { dict, found, key } => {
                let (handle, key) = (get!(dict), get!(key));
                let (number, value) = (get!(found), get!(found + 1));
                match usize::try_from(number) {
                    Ok(number) if number != usize::MAX => replace_value!(handle, number, value),
                    _ => {
                        let (probe, _, _, _) = lookup!(handle, key);
                        add_key!(handle, probe, key, value);
                    }
                }
            }
            Instruction::SetKey { dict, key, value } => {
                let (key, value) = (get!(key), get!(value));
                let handle = own!(dict);
                let (probe, found, looked, _) = lookup!(handle, key);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                if let Some(number) = found {
                    replace_value!(handle, number, value);
                } else {
                    add_key!(handle, probe, key, value);
                }
                count_work!();
            }
            Instruction::RemoveKey { dict, key } => {
                let key = get!(key);
                let handle = own!(dict);
                let (_, found, looked, _) = lookup!(handle, key);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                match found {
                    None => failure = missing,
                    Some(number) => {
                        let (removed, before, after, text_keys, reference_values) = match heap.get_mut(handle) {
                            Ok(Object::Dict(dict)) => {
                                let before = dict.size();
                                let removed = dict.remove(number);
                                (removed, before, dict.size(), dict.text_keys, dict.reference_values)
                            }
                            _ => (None, 0, 0, false, false),
                        };
                        heap.refund(before.saturating_sub(after));
                        if let Some((old_key, old_value)) = removed {
                            if text_keys {
                                heap.release(old_key, layouts);
                            }
                            if reference_values {
                                heap.release(old_value, layouts);
                            }
                        }
                    }
                }
                count_work!();
            }
            Instruction::NextEntry { to, base: loop_base, offset } => {
                let position = get!(loop_base + 1);
                let next = match heap.get(get!(loop_base)) {
                    Ok(Object::Dict(dict)) => dict.next_from(usize::try_from(position).unwrap_or(usize::MAX)),
                    Ok(_) => return Err(broken("a handle that should lead to a dictionary", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                if let Some((after, key, value)) = next {
                    set!(to, value);
                    set!(to + 1, key);
                    set!(loop_base + 1, after as u64);
                    short_jump!(offset);
                }
            }

            Instruction::NewRecord { to, base: first, layout } | Instruction::NewCase { to, base: first, layout } => {
                let Some(count) = layouts.get(usize::from(layout)).map(|layout| layout.references.len()) else {
                    return Err(broken("a layout that isn't there", pc));
                };
                let start = usize::from(first);
                let Some(values) = registers.get(start..start + count) else {
                    return Err(broken("a record built from outside its frame", pc));
                };
                let fields = values.to_vec();
                let size = count.saturating_mul(SLOT_COST);
                let object = if matches!(instruction, Instruction::NewRecord { .. }) {
                    Object::Record { fields, layout }
                } else {
                    Object::Case { fields, layout }
                };
                set!(to, heap_try!(heap.allocate(object, size)));
            }
            Instruction::GetField { to, record, field } | Instruction::GetCaseValue { to, case: record, field } => {
                let value = match heap.get(get!(record)) {
                    Ok(Object::Record { fields, .. } | Object::Case { fields, .. }) => {
                        fields.get(usize::from(field)).copied()
                    }
                    Ok(_) => None,
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                match value {
                    Some(value) => set!(to, value),
                    None => return Err(broken("a field that isn't there", pc)),
                }
            }
            Instruction::SetField { record, field, value }
            | Instruction::SetReferenceField { record, field, value } => {
                let references = matches!(instruction, Instruction::SetReferenceField { .. });
                let value = get!(value);
                let handle = own!(record);
                let old = match heap.get_mut(handle) {
                    Ok(Object::Record { fields, .. }) => {
                        fields.get_mut(usize::from(field)).map(|slot| std::mem::replace(slot, value))
                    }
                    _ => None,
                };
                match old {
                    Some(old) if references => heap.release(old, layouts),
                    Some(_) => {}
                    None => return Err(broken("a field that isn't there", pc)),
                }
                count_work!();
            }
            Instruction::UnpackRecord { base: first, record } => {
                let start = usize::from(first);
                let fields = match heap.get(get!(record)) {
                    Ok(Object::Record { fields, .. }) => fields,
                    Ok(_) => return Err(broken("a handle that should lead to a record", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                match registers.get_mut(start..start + fields.len()) {
                    Some(registers) => registers.copy_from_slice(fields),
                    None => return Err(broken("a record unpacked outside its frame", pc)),
                }
            }
            Instruction::UnpackItemAtPosition { base: first, list, index }
            | Instruction::UnpackItemAtOffset { base: first, list, index } => {
                let zero_based = matches!(instruction, Instruction::UnpackItemAtOffset { .. });
                let index = whole!(index);
                let items = items!(get!(list));
                let Some(&item) = item_place(index, zero_based, items.len()).and_then(|place| items.get(place)) else {
                    return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, items.len(), pc));
                };
                let fields = match heap.get(item) {
                    Ok(Object::Record { fields, .. }) => fields,
                    Ok(_) => return Err(broken("a handle that should lead to a record", pc)),
                    Err(error) => return Err((Stop::from(error), pc)),
                };
                let start = usize::from(first);
                match registers.get_mut(start..start + fields.len()) {
                    Some(registers) => registers.copy_from_slice(fields),
                    None => return Err(broken("a record unpacked outside its frame", pc)),
                }
            }
            Instruction::PackIntoItemAtPosition { list, index, base: first }
            | Instruction::PackIntoItemAtOffset { list, index, base: first } => {
                let zero_based = matches!(instruction, Instruction::PackIntoItemAtOffset { .. });
                let index = whole!(index);
                // The usual case first: the list alone holds the record, so its fields are written where they are.
                let start = usize::from(first);
                let list_handle = get!(list);
                let written = usize::try_from(if zero_based { index } else { index.wrapping_sub(1) })
                    .ok()
                    .and_then(|place| heap.item_fields_to_write(list_handle, place))
                    .and_then(|fields| {
                        let from = registers.get(start..start + fields.len())?;
                        fields.copy_from_slice(from);
                        Some(())
                    });
                if written.is_some() {
                    continue;
                }
                let handle = own!(list);
                let (found, length) = {
                    let items = items!(handle);
                    let place = item_place(index, zero_based, items.len());
                    (place.and_then(|place| items.get(place).copied().map(|item| (place, item))), items.len())
                };
                let Some((place, item)) = found else {
                    return Err(bad_position(if zero_based { "offset" } else { "pos" }, index, length, pc));
                };
                let start = usize::from(first);
                if heap.has_one_holder(item) {
                    // The list alone holds the item, so the record's fields are written straight into it.
                    match heap.get_mut(item) {
                        Ok(Object::Record { fields, .. }) => match registers.get(start..start + fields.len()) {
                            Some(registers) => fields.copy_from_slice(registers),
                            None => return Err(broken("a record packed from outside its frame", pc)),
                        },
                        Ok(_) => return Err(broken("a handle that should lead to a record", pc)),
                        Err(error) => return Err((Stop::from(error), pc)),
                    }
                } else {
                    let (count, layout) = match heap.get(item) {
                        Ok(Object::Record { fields, layout }) => (fields.len(), *layout),
                        Ok(_) => return Err(broken("a handle that should lead to a record", pc)),
                        Err(error) => return Err((Stop::from(error), pc)),
                    };
                    let Some(registers) = registers.get(start..start + count) else {
                        return Err(broken("a record packed from outside its frame", pc));
                    };
                    let fields = registers.to_vec();
                    let size = count.saturating_mul(SLOT_COST);
                    let new = heap_try!(heap.allocate(Object::Record { fields, layout }, size));
                    if let Ok(Object::List { items, .. }) = heap.get_mut(handle)
                        && let Some(slot) = items.get_mut(place)
                    {
                        *slot = new;
                    }
                    heap.release(item, layouts);
                }
            }
            Instruction::CaseNumber { to, from } => {
                let slot = get!(from);
                let number = if slot & PLAIN_CASE != 0 {
                    slot & !PLAIN_CASE
                } else {
                    match heap.get(slot) {
                        Ok(Object::Case { layout, .. }) => {
                            u64::from(layouts.get(usize::from(*layout)).map_or(0, |layout| layout.case))
                        }
                        Ok(_) => return Err(broken("a handle that should lead to a case", pc)),
                        Err(error) => return Err((Stop::from(error), pc)),
                    }
                };
                set!(to, number);
            }

            Instruction::PlaceOnly => return Err(broken("a place that's only kept for a message", pc)),
            Instruction::Unreachable => return Err(broken("a place the compiler took for unreachable", pc)),
        }
    }
}
