//! Seal's bytecode: the instructions its register VM runs (D46).
//!
//! Every type is known before a script runs, so each instruction says what it works on, as `AddWhole` and
//! `AddFloat`, and nothing checks a type while running. A register is a bare 64-bit slot holding a `Whole`, a
//! `Float`'s bits, a `Bool`, or the handle of a value on the heap.
//!
//! A step (D14) is one jump taken or one call, plus one per [`BULK_UNIT`] of bulk work.

use std::collections::HashMap;

use crate::check::types::TypeTable;
use crate::message::Place;

/// A register: a slot in the running procedure's frame.
pub type Register = u16;

/// The bit that marks a slot of a `one_of` type as a case with no values, held in the slot itself, not on the heap.
pub const PLAIN_CASE: u64 = 1 << 63;

/// How much bulk work, such as items copied or bytes joined, counts as one step (D14).
pub const BULK_UNIT: u64 = 64;

/// The jump distance that means "look the real one up in [`Compiled::far_jumps`]".
pub const FAR: i16 = i16::MIN;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    // ---- moving values ----
    Move {
        to: Register,
        from: Register,
    },
    /// A small `Whole`, or a `Bool` as 0 or 1.
    LoadSmall {
        to: Register,
        value: i32,
    },
    /// A `Whole` or a `Float` from the constants.
    LoadConstant {
        to: Register,
        constant: u32,
    },
    /// One of the script's texts, which lives as long as the script does, so the register only borrows it.
    LoadText {
        to: Register,
        text: u32,
    },
    /// A case with no values (D35).
    LoadPlainCase {
        to: Register,
        case: u32,
    },
    GetGlobal {
        to: Register,
        global: u16,
    },
    SetGlobal {
        global: u16,
        from: Register,
    },
    /// Counts one more holder of a heap value (D15).
    Retain {
        register: Register,
    },
    /// Counts one holder fewer, freeing the value when it was the last (D15).
    Release {
        register: Register,
    },

    // ---- `Whole` maths, each checked for overflow (D17) ----
    AddWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    SubtractWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    MultiplyWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    /// `/` on two `Whole`s divides exactly; a remainder is a failed check (Unio D70).
    DivideWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    DivideFloor {
        to: Register,
        left: Register,
        right: Register,
    },
    DivideTrunc {
        to: Register,
        left: Register,
        right: Register,
    },
    RemainderFloor {
        to: Register,
        left: Register,
        right: Register,
    },
    RemainderTrunc {
        to: Register,
        left: Register,
        right: Register,
    },
    AddWholeSmall {
        to: Register,
        left: Register,
        right: i16,
    },
    NegateWhole {
        to: Register,
        from: Register,
    },

    // ---- `Float` maths, each checked for infinity and NaN (D18) ----
    AddFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    SubtractFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    MultiplyFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    DivideFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    NegateFloat {
        to: Register,
        from: Register,
    },
    SquareRoot {
        to: Register,
        from: Register,
    },

    // ---- comparing, giving a `Bool` ----
    EqualWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    NotEqualWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    LessWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    AtMostWhole {
        to: Register,
        left: Register,
        right: Register,
    },
    EqualFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    NotEqualFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    LessFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    AtMostFloat {
        to: Register,
        left: Register,
        right: Register,
    },
    Not {
        to: Register,
        from: Register,
    },

    // ---- jumps, each one step when taken ----
    Jump {
        offset: i32,
    },
    JumpIfTrue {
        register: Register,
        offset: i32,
    },
    JumpIfFalse {
        register: Register,
        offset: i32,
    },
    JumpIfEqualWhole {
        left: Register,
        right: Register,
        offset: i16,
    },
    JumpIfNotEqualWhole {
        left: Register,
        right: Register,
        offset: i16,
    },
    JumpIfLessWhole {
        left: Register,
        right: Register,
        offset: i16,
    },
    JumpIfAtMostWhole {
        left: Register,
        right: Register,
        offset: i16,
    },
    JumpIfEqualWholeSmall {
        left: Register,
        right: i16,
        offset: i16,
    },
    JumpIfNotEqualWholeSmall {
        left: Register,
        right: i16,
        offset: i16,
    },
    JumpIfLessWholeSmall {
        left: Register,
        right: i16,
        offset: i16,
    },
    JumpIfAtMostWholeSmall {
        left: Register,
        right: i16,
        offset: i16,
    },
    JumpIfGreaterWholeSmall {
        left: Register,
        right: i16,
        offset: i16,
    },
    JumpIfAtLeastWholeSmall {
        left: Register,
        right: i16,
        offset: i16,
    },
    JumpIfEqualFloat {
        left: Register,
        right: Register,
        offset: i16,
    },
    JumpIfNotEqualFloat {
        left: Register,
        right: Register,
        offset: i16,
    },
    JumpIfLessFloat {
        left: Register,
        right: Register,
        offset: i16,
    },
    JumpIfAtMostFloat {
        left: Register,
        right: Register,
        offset: i16,
    },

    // ---- calls ----
    /// Calls a procedure. Its frame starts at `base`: the result comes back in `base`, and the arguments sit in
    /// the registers after it.
    Call {
        base: Register,
        procedure: u32,
    },
    Return {
        from: Register,
    },
    ReturnNothing,
    /// Calls a door, with the same layout as [`Instruction::Call`]; `site` says which door and how values cross.
    CallDoor {
        base: Register,
        site: u32,
    },

    // ---- failures (D44, Unio D23) ----
    /// `fail name`: leaves the procedure with a failure.
    Fail {
        failure: u16,
    },
    /// `?`: leaves the procedure with the failure just given.
    PassFailureUp,
    JumpUnlessFailed {
        offset: i32,
    },
    JumpUnlessFailureIs {
        failure: u16,
        offset: i32,
    },
    ClearFailure,
    /// `?? crash`: stops the script on the failure just given.
    Crash,

    // ---- text ----
    /// A new, empty text.
    NewText {
        to: Register,
    },
    /// Joins `from` onto the end of `to`, in place when `to` has one holder (D15).
    Append {
        to: Register,
        from: Register,
    },
    AppendWhole {
        to: Register,
        from: Register,
    },
    AppendBool {
        to: Register,
        from: Register,
    },
    AppendFloat {
        to: Register,
        from: Register,
    },
    /// `be.Whole` on text, which can fail with `not_a_whole` (Unio D95, D156).
    TextToWhole {
        to: Register,
        from: Register,
    },
    WholeToFloat {
        to: Register,
        from: Register,
    },
    /// `be.Whole.floor` and the rest; `rounding` counts through [`crate::check::tree::Rounding`]'s cases.
    FloatToWhole {
        to: Register,
        from: Register,
        rounding: u16,
    },

    // ---- lists and arrays ----
    /// A list of the `count` values from `base` on, which it takes over.
    NewList {
        to: Register,
        base: Register,
        count: u16,
    },
    NewReferenceList {
        to: Register,
        base: Register,
        count: u16,
    },
    /// `[all: value]`: `count` holds how many (Unio D169).
    RepeatList {
        to: Register,
        value: Register,
        count: Register,
    },
    RepeatReferenceList {
        to: Register,
        value: Register,
        count: Register,
    },
    /// `list[pos: n]`, counting from 1 (Unio D75). A bad position is a failed check.
    ItemAtPosition {
        to: Register,
        list: Register,
        index: Register,
    },
    /// `list[offset: n]`, counting from 0.
    ItemAtOffset {
        to: Register,
        list: Register,
        index: Register,
    },
    SetItemAtPosition {
        list: Register,
        index: Register,
        value: Register,
    },
    SetItemAtOffset {
        list: Register,
        index: Register,
        value: Register,
    },
    AddItem {
        list: Register,
        value: Register,
    },
    InsertItemAtPosition {
        list: Register,
        index: Register,
        value: Register,
    },
    InsertItemAtOffset {
        list: Register,
        index: Register,
        value: Register,
    },
    /// Fails with `full` when the list already holds `limit` items (Unio D168).
    CheckRoom {
        list: Register,
        limit: Register,
    },
    RemoveItemAtPosition {
        list: Register,
        index: Register,
    },
    RemoveItemAtOffset {
        list: Register,
        index: Register,
    },
    /// `.len` of a list, an array or a dictionary (D34, Unio D79).
    Length {
        to: Register,
        from: Register,
    },
    /// One pass of `for each` over a list: with the list in `base` and how many items are done in `base + 1`,
    /// puts the next item in `to` and jumps back by `offset`, or carries on when none is left.
    NextItem {
        to: Register,
        base: Register,
        offset: i16,
    },

    // ---- ranges (Unio D170, D172) ----
    /// With `from`, the end and the step in `base`, `base + 1` and `base + 2`: checks the step, and leaves the
    /// next number in `base` and the passes left in `base + 1`. `to:` includes its end.
    PrepareRangeTo {
        base: Register,
    },
    /// The same for `before:`, which stops short of its end.
    PrepareRangeBefore {
        base: Register,
    },
    /// One pass over a range: puts the next number in `to` and jumps back by `offset`, or carries on.
    NextInRange {
        to: Register,
        base: Register,
        offset: i16,
    },

    // ---- dictionaries (D32 to D34) ----
    /// A new, empty dictionary. `shape` bit 0: the keys are text; bit 1: the values live on the heap.
    NewDict {
        to: Register,
        shape: u16,
    },
    /// `dict[key: k]`, which can fail with `missing`.
    GetKey {
        to: Register,
        dict: Register,
        key: Register,
    },
    SetKey {
        dict: Register,
        key: Register,
        value: Register,
    },
    /// `dict.remove(key: k)`, which can fail with `missing`.
    RemoveKey {
        dict: Register,
        key: Register,
    },
    HasKey {
        to: Register,
        dict: Register,
        key: Register,
    },
    /// One pass of `for each` over a dictionary: with the dictionary in `base` and the place reached in
    /// `base + 1`, puts the next value in `to` and its key in `to + 1`, and jumps back by `offset`.
    NextEntry {
        to: Register,
        base: Register,
        offset: i16,
    },

    // ---- records and cases (D27, D35) ----
    /// A record of the values from `base` on, as `layout` says, which it takes over.
    NewRecord {
        to: Register,
        base: Register,
        layout: u16,
    },
    GetField {
        to: Register,
        record: Register,
        field: u16,
    },
    SetField {
        record: Register,
        field: u16,
        value: Register,
    },
    SetReferenceField {
        record: Register,
        field: u16,
        value: Register,
    },
    /// A case carrying the values from `base` on, as `layout` says.
    NewCase {
        to: Register,
        base: Register,
        layout: u16,
    },
    /// Copies every field of a record into the registers from `base` on, for a record kept in registers.
    UnpackRecord {
        base: Register,
        record: Register,
    },
    /// `list[pos: n] = record` for a record kept in the registers from `base` on: the item is written in place
    /// when the list is its only holder, and replaced by a new record otherwise (D15).
    PackIntoItemAtPosition {
        list: Register,
        index: Register,
        base: Register,
    },
    PackIntoItemAtOffset {
        list: Register,
        index: Register,
        base: Register,
    },
    /// Which case a value is, as its number within its type.
    CaseNumber {
        to: Register,
        from: Register,
    },
    GetCaseValue {
        to: Register,
        case: Register,
        field: u16,
    },

    /// A place the compiler knows can't be reached. Reaching it is a bug in Seal.
    Unreachable,
}

/// How a record or a case is laid out on the heap: which of its values are heap values themselves.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Layout {
    /// The case's number within its `one_of`; 0 for a record.
    pub case: u32,
    pub references: Vec<bool>,
}

/// How a value crosses between a script and its host (D15, D48).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Crossing {
    Nothing,
    Whole,
    Float,
    Bool,
    Text,
    List(Box<Crossing>),
    Dict(Box<Crossing>, Box<Crossing>),
    Maybe { inner: Box<Crossing>, layout: u16 },
    Record { name: String, layout: u16, fields: Vec<(String, Crossing)> },
    OneOf { name: String, cases: Vec<CrossingCase> },
}

/// One case of a `one_of` at a door.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrossingCase {
    pub name: String,
    pub layout: u16,
    pub fields: Vec<(String, Crossing)>,
}

impl Crossing {
    /// Whether a slot of this kind holds a heap value.
    #[must_use]
    pub fn is_reference(&self) -> bool {
        match self {
            Crossing::Nothing | Crossing::Whole | Crossing::Float | Crossing::Bool => false,
            Crossing::OneOf { cases, .. } => cases.iter().any(|case| !case.fields.is_empty()),
            _ => true,
        }
    }
}

/// One place a door is called from.
#[derive(Clone, Debug)]
pub struct DoorSite {
    pub door: u32,
    pub arguments: Vec<Crossing>,
    pub result: Crossing,
}

/// A procedure, compiled.
#[derive(Clone, Debug)]
pub struct CompiledProcedure {
    pub name: String,
    /// Where its instructions start.
    pub entry: u32,
    /// How many registers its frame needs.
    pub frame_size: u32,
    pub parameters: Vec<CompiledParameter>,
    pub result: Crossing,
    /// The type the result has, as it's written in a script.
    pub result_written: String,
    pub failures: Vec<String>,
    pub can_pause: bool,
}

#[derive(Clone, Debug)]
pub struct CompiledParameter {
    pub name: String,
    pub label: Option<String>,
    pub crossing: Crossing,
    /// The parameter's type as it's written in a script, for messages.
    pub written: String,
    /// The default's value, when the parameter has one (D42).
    pub default: Option<crate::host::Value>,
}

/// What a call needs of the procedure it calls, kept small and side by side so calls stay fast.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProcedureEntry {
    /// Where the procedure's instructions start.
    pub entry: u32,
    /// How many registers its frame needs.
    pub frame_size: u32,
}

/// A whole script, compiled: what a loaded script runs.
#[derive(Clone, Debug, Default)]
pub struct Compiled {
    pub code: Vec<Instruction>,
    /// Each instruction's place in the script, for messages (D17).
    pub places: Vec<Place>,
    pub constants: Vec<u64>,
    pub texts: Vec<String>,
    pub procedures: Vec<CompiledProcedure>,
    /// One entry for each of `procedures`, in the same order.
    pub entries: Vec<ProcedureEntry>,
    pub top_level_entry: u32,
    pub top_level_frame_size: u32,
    pub global_count: u32,
    /// For each top-level name shared with procedures, whether its value lives on the heap.
    pub global_references: Vec<bool>,
    pub layouts: Vec<Layout>,
    pub door_sites: Vec<DoorSite>,
    /// The jumps too long for their instruction, by the instruction's place in `code`.
    pub far_jumps: HashMap<u32, i32>,
    /// For each call that can pause, by the place just after it: the registers holding heap values the frame
    /// owns there, to be freed if the task is cancelled (D54).
    pub pause_maps: HashMap<u32, Vec<Register>>,
    pub failure_names: Vec<String>,
    /// Each procedure's name, by the place its instructions start, for messages.
    pub types: TypeTable,
}

impl Compiled {
    #[must_use]
    pub fn failure_name(&self, failure: u32) -> &str {
        self.failure_names.get((failure as usize).wrapping_sub(1)).map_or("?", String::as_str)
    }

    /// The procedure whose instructions hold `pc`, when there is one.
    #[must_use]
    pub fn procedure_at(&self, pc: u32) -> Option<&CompiledProcedure> {
        self.procedures.iter().filter(|procedure| procedure.entry <= pc).max_by_key(|procedure| procedure.entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instruction_is_eight_bytes() {
        assert_eq!(std::mem::size_of::<Instruction>(), 8);
    }
}
