//! The loop that runs bytecode (D46).
//!
//! Nothing here can crash the host (D14): every register, instruction and heap value is reached through a check,
//! and anything that doesn't add up stops the script and says so, as a bug in Seal to report.

use super::crossing::{CrossingError, to_slot, to_value};
use super::dict::{Dict, hash_bits, hash_text};
use super::heap::{Heap, HeapError, Object, SLOT_COST};
use crate::check::tree::Rounding;
use crate::compile::bytecode::{BULK_UNIT, Compiled, FAR, Instruction, Layout, PLAIN_CASE};
use crate::host::Value;
use crate::number::{float_to_text, float_to_whole, text_to_whole};

/// What a stack slot costs against the memory cap: the slot itself, and its share of the record of calls.
const STACK_SLOT_COST: usize = 2 * SLOT_COST;

/// What a door answers when the VM calls it.
#[derive(Clone, Debug, PartialEq)]
pub enum DoorAnswer {
    Value(Value),
    /// The door failed with one of the failures its host declared (D44).
    Failure(String),
    /// The door pauses the script; the host resumes it later with the door's value (D49).
    Pause,
    /// The door couldn't be called as declared: a mistake in the host's code, or in Seal.
    Mistake(String),
}

/// The host's doors, as the VM calls them.
pub trait DoorCaller {
    fn call_door(&self, door: u32, arguments: Vec<Value>) -> DoorAnswer;
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
    text.push_str(std::str::from_utf8(digits.get(place..).unwrap_or(&[])).unwrap_or(""));
}

/// Grows a task's stack to hold `needed` slots, counting it against the memory cap (D14).
fn grow_stack(stack: &mut Vec<u64>, heap: &mut Heap, needed: usize) -> Result<(), HeapError> {
    let new_length = needed.max(stack.len().saturating_mul(2)).max(32);
    heap.charge((new_length - stack.len()).saturating_mul(STACK_SLOT_COST))?;
    stack.resize(new_length, 0);
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
        self.stack = Vec::new();
        self.frames = Vec::new();
        self.waiting = None;
    }
}

/// Appends to the text in `target`, in place. `write` adds to the text and the cap is counted for any growth.
fn append_with(heap: &mut Heap, target: u64, extra: usize, write: impl FnOnce(&mut String)) -> Result<(), HeapError> {
    let Object::Text(text) = heap.get(target)? else {
        return Err(HeapError::Broken("a handle that should lead to text"));
    };
    let (length, capacity) = (text.len(), text.capacity());
    let needed = length.saturating_add(extra);
    let mut reserve = None;
    if needed > capacity {
        // Doubling, so joining onto a text many times costs a fixed amount per byte on average.
        let new_capacity = needed.max(capacity.saturating_mul(2)).max(16);
        heap.charge(new_capacity - capacity)?;
        reserve = Some(new_capacity - length);
    }
    heap.work = heap.work.saturating_add(extra as u64);
    if let Object::Text(text) = heap.get_mut(target)? {
        if let Some(reserve) = reserve {
            text.reserve_exact(reserve);
        }
        write(text);
    }
    Ok(())
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
    let Machine { heap, globals, texts } = machine;
    let code: &[Instruction] = &compiled.code;
    let layouts: &[Layout] = &compiled.layouts;
    let constants: &[u64] = &compiled.constants;
    let [missing, not_a_whole, full] = failures;
    let mut pc = task.pc;
    let mut base = task.base;
    let mut steps_left = step_budget;
    // The failure the last call or instruction gave, or 0. It's dealt with by the very next instruction.
    let mut failure: u32 = 0;
    let stack = &mut task.stack;
    let frames = &mut task.frames;

    macro_rules! get {
        ($register:expr) => {
            match stack.get(base + usize::from($register)) {
                Some(value) => *value,
                None => return Err(broken("a register outside its frame", pc)),
            }
        };
    }
    macro_rules! set {
        ($register:expr, $value:expr) => {{
            let value = $value;
            match stack.get_mut(base + usize::from($register)) {
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
            if !result.is_finite() {
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
            let own = heap_try!(heap.make_own(handle, layouts));
            if own != handle {
                set!($register, own);
            }
            own
        }};
    }
    macro_rules! key_hash {
        ($dict:expr, $key:expr) => {
            match heap.get($dict) {
                Ok(Object::Dict(dict)) if dict.text_keys => hash_text(heap_try!(heap.text($key))),
                Ok(Object::Dict(_)) => hash_bits($key),
                Ok(_) => return Err(broken("a handle that should lead to a dictionary", pc)),
                Err(error) => return Err((Stop::from(error), pc)),
            }
        };
    }
    macro_rules! find_key {
        ($dict:expr, $key:expr, $hash:expr) => {
            match heap.get($dict) {
                Ok(Object::Dict(dict)) => {
                    let (found, looked) = if dict.text_keys {
                        let wanted = heap_try!(heap.text($key));
                        dict.find($hash, |stored| heap.text(stored).is_ok_and(|stored| stored == wanted))
                    } else {
                        dict.find($hash, |stored| stored == $key)
                    };
                    (found, looked, dict.value_at(found.unwrap_or(usize::MAX)))
                }
                Ok(_) => return Err(broken("a handle that should lead to a dictionary", pc)),
                Err(error) => return Err((Stop::from(error), pc)),
            }
        };
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
            Instruction::DivideFloat { to, left, right } => {
                let (left, right) = (float!(left), float!(right));
                if right == 0.0 {
                    return Err(divided_by_zero(&float_to_text(left), "/", pc));
                }
                let result = left / right;
                if !result.is_finite() {
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
                let Some(callee) = compiled.procedures.get(procedure as usize) else {
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
            }
            Instruction::Return { from } => {
                let value = get!(from);
                set!(0u16, value);
                if let Some(frame) = frames.pop() {
                    pc = frame.return_pc as usize;
                    base = frame.base as usize;
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
                let mut arguments = Vec::with_capacity(door_site.arguments.len());
                for (register, crossing) in (call_base + 1..).zip(&door_site.arguments) {
                    arguments.push(heap_try!(to_value(get!(register), crossing, heap, layouts)));
                }
                match doors.call_door(door_site.door, arguments) {
                    DoorAnswer::Value(value) => match to_slot(&value, &door_site.result, heap, layouts) {
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
                    }
                    None => return Ok(Outcome::Failed(failure)),
                }
            }
            Instruction::PassFailureUp => match frames.pop() {
                Some(frame) => {
                    pc = frame.return_pc as usize;
                    base = frame.base as usize;
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

            Instruction::NewText { to } => set!(to, heap_try!(heap.new_text("", 0))),
            Instruction::Append { to, from } => {
                let target = own!(to);
                let source = get!(from);
                if target == source {
                    // A text joined onto itself: its own bytes are copied before it grows.
                    let copy = heap_try!(heap.text(source)).to_owned();
                    heap_try!(append_with(heap, target, copy.len(), |text| text.push_str(&copy)));
                } else {
                    let extra = heap_try!(heap.text(source)).len();
                    // The target is taken out so the source can be read while it grows.
                    let needed = match heap.get(target) {
                        Ok(Object::Text(text)) => (text.len(), text.capacity()),
                        Ok(_) => return Err(broken("a handle that should lead to text", pc)),
                        Err(error) => return Err((Stop::from(error), pc)),
                    };
                    let (length, capacity) = needed;
                    let wanted = length.saturating_add(extra);
                    let mut reserve = 0;
                    if wanted > capacity {
                        let new_capacity = wanted.max(capacity.saturating_mul(2)).max(16);
                        heap_try!(heap.charge(new_capacity - capacity));
                        reserve = new_capacity - length;
                    }
                    heap.work = heap.work.saturating_add(extra as u64);
                    let mut taken = heap_try!(heap.take(target));
                    if let Object::Text(text) = &mut taken {
                        if reserve > 0 {
                            text.reserve_exact(reserve);
                        }
                        match heap.text(source) {
                            Ok(source) => text.push_str(source),
                            Err(error) => {
                                heap.put_back(target, taken);
                                return Err((Stop::from(error), pc));
                            }
                        }
                    }
                    heap.put_back(target, taken);
                }
                count_work!();
            }
            Instruction::AppendWhole { to, from } => {
                let target = own!(to);
                let value = whole!(from);
                heap_try!(append_with(heap, target, 20, |text| append_whole(text, value)));
            }
            Instruction::AppendBool { to, from } => {
                let target = own!(to);
                let word = if get!(from) == 0 { "false" } else { "true" };
                heap_try!(append_with(heap, target, word.len(), |text| text.push_str(word)));
            }
            Instruction::AppendFloat { to, from } => {
                let target = own!(to);
                let shown = float_to_text(float!(from));
                heap_try!(append_with(heap, target, shown.len(), |text| text.push_str(&shown)));
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
                let start = base + usize::from(first);
                let Some(values) = stack.get(start..start + usize::from(count)) else {
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
                let hash = key_hash!(dict, key);
                let (_, looked, value) = find_key!(dict, key, hash);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                match value {
                    Some(value) => set!(to, value),
                    None => failure = missing,
                }
                count_work!();
            }
            Instruction::HasKey { to, dict, key } => {
                let (dict, key) = (get!(dict), get!(key));
                let hash = key_hash!(dict, key);
                let (found, looked, _) = find_key!(dict, key, hash);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                set!(to, u64::from(found.is_some()));
                count_work!();
            }
            Instruction::SetKey { dict, key, value } => {
                let (key, value) = (get!(key), get!(value));
                let handle = own!(dict);
                let hash = key_hash!(handle, key);
                let (found, looked, _) = find_key!(handle, key, hash);
                heap.work = heap.work.saturating_add(looked.saturating_sub(4));
                let (growth, text_keys) = match heap.get(handle) {
                    Ok(Object::Dict(dict)) => {
                        (if found.is_some() { 0 } else { dict.growth_for_one_more() }, dict.text_keys)
                    }
                    _ => (0, false),
                };
                heap_try!(heap.charge(growth));
                if found.is_none() && text_keys {
                    // The dictionary becomes one more holder of a key it keeps.
                    heap.retain(key);
                }
                let old = match heap.get_mut(handle) {
                    Ok(Object::Dict(dict)) => {
                        if let Some(number) = found {
                            dict.replace_value(number, value).filter(|_| dict.reference_values)
                        } else {
                            dict.add(hash, key, value);
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(old) = old {
                    heap.release(old, layouts);
                }
                count_work!();
            }
            Instruction::RemoveKey { dict, key } => {
                let key = get!(key);
                let handle = own!(dict);
                let hash = key_hash!(handle, key);
                let (found, looked, _) = find_key!(handle, key, hash);
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
                let start = base + usize::from(first);
                let Some(values) = stack.get(start..start + count) else {
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

            Instruction::Unreachable => return Err(broken("a place the compiler took for unreachable", pc)),
        }
    }
}
