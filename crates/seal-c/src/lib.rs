//! Seal's C API (D136, D151, D152): everything a Rust host gets, through plain C functions, declared in
//! `include/seal.h`.
//!
//! Nothing a script does, and no mistake a host makes through this API, can crash the host or reach past its doors
//! (Building rule 1). Handles are numbers in tables Seal keeps for each thread, checked before every use, so a freed,
//! made-up or another thread's handle is refused, never followed. A panic never crosses into C: every function
//! catches one and gives a status with a message. This is the one crate with `unsafe` (Building rule 2): only to read
//! what the host's pointers point at, to write through the ones it gives for answers, and to call its doors, each
//! block saying why it's sound.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]
// The C names, `seal_status` and its kin, are the header's.
#![allow(non_camel_case_types)]

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};

use seal::big::Big;
use seal::decimal::{Decimal, HugeDecimal};
use seal::vm::machine::DoorAnswer;
use seal::{Argument, Called, Doors, Error, HostType, Limits, Message, MessageKind, Script, TaskId, Value};

// ---- what crosses ----

/// `seal_status`.
pub type seal_status = c_int;
pub const SEAL_OK: seal_status = 0;
pub const SEAL_PAUSED: seal_status = 1;
pub const SEAL_REFUSED: seal_status = 2;
pub const SEAL_STOPPED: seal_status = 3;
pub const SEAL_FAILED: seal_status = 4;
pub const SEAL_HOST_MISTAKE: seal_status = 5;

/// `seal_kind`, read as a number so a wrong one from C is refused rather than trusted.
pub type seal_kind = c_int;
const NOTHING: seal_kind = 0;
const WHOLE: seal_kind = 1;
const FLOAT: seal_kind = 2;
const BOOL: seal_kind = 3;
const TEXT: seal_kind = 4;
const HUGE_WHOLE: seal_kind = 5;
const DECIMAL: seal_kind = 6;
const HUGE_DECIMAL: seal_kind = 7;
const LIST: seal_kind = 8;
const MAYBE: seal_kind = 9;
const DICT: seal_kind = 10;
const RECORD: seal_kind = 11;
const CASE: seal_kind = 12;

/// `seal_arg`: a kind, then eight bytes holding a whole number, a float's bits, a `Bool` in its low four bytes, or a
/// handle, laid out as the header's union is.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct seal_arg {
    pub kind: seal_kind,
    pub bits: u64,
}

/// `seal_limits`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct seal_limits {
    pub steps: u64,
    pub memory: usize,
}

/// What the header's handle types point at: nothing, since a handle is only a number.
#[repr(C)]
pub struct seal_handle {
    _never: [u8; 0],
}
pub type seal_doors = seal_handle;
pub type seal_script = seal_handle;
pub type seal_value = seal_handle;

/// `seal_door_function`.
pub type seal_door_function =
    Option<unsafe extern "C" fn(*mut c_void, *const seal_arg, usize, *mut seal_arg) -> seal_status>;

// ---- handles ----

/// A table of things a host holds by number: each number is its place, counted from 1, and the place's generation,
/// so a number freed and made again never matches the old one.
struct Table<T> {
    slots: Vec<(u32, Option<T>)>,
    free: Vec<usize>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Table { slots: Vec::new(), free: Vec::new() }
    }
}

impl<T> Table<T> {
    fn insert(&mut self, item: T) -> usize {
        let place = if let Some(place) = self.free.pop() {
            place
        } else {
            self.slots.push((0, None));
            self.slots.len() - 1
        };
        let Some(slot) = self.slots.get_mut(place) else { return 0 };
        slot.0 = slot.0.wrapping_add(1).max(1);
        slot.1 = Some(item);
        (usize::try_from(slot.0).unwrap_or(0) << 32) | (place + 1)
    }

    fn place(&self, handle: usize) -> Option<usize> {
        let place = (handle & 0xffff_ffff).checked_sub(1)?;
        let generation = u32::try_from(handle >> 32).ok()?;
        (self.slots.get(place)?.0 == generation).then_some(place)
    }

    fn get(&self, handle: usize) -> Option<&T> {
        self.slots.get(self.place(handle)?)?.1.as_ref()
    }

    fn get_mut(&mut self, handle: usize) -> Option<&mut T> {
        let place = self.place(handle)?;
        self.slots.get_mut(place)?.1.as_mut()
    }

    fn remove(&mut self, handle: usize) -> Option<T> {
        let place = self.place(handle)?;
        let item = self.slots.get_mut(place)?.1.take()?;
        self.free.push(place);
        Some(item)
    }
}

/// A value a host holds: lent for a door's call, or its own; and its text, once read, kept for the pointer given.
struct Held {
    value: Value,
    lent: bool,
    text: Option<CString>,
}

/// A script a host holds, out of its place while a call runs, so a door can't call it again from inside.
type HeldScript = Option<Script>;

/// One message as the host reads it, whole and in parts (D151).
struct Said {
    kind: c_int,
    text: CString,
    what: Option<CString>,
    why: Option<CString>,
    fix: Option<CString>,
    when: Option<CString>,
    file: Option<CString>,
    line: u32,
    column: u32,
    width: u32,
}

thread_local! {
    static DOORS: RefCell<Table<Doors>> = RefCell::default();
    static SCRIPTS: RefCell<Table<HeldScript>> = RefCell::default();
    static VALUES: RefCell<Table<Held>> = RefCell::default();
    static BYTES: RefCell<Vec<Box<[u8]>>> = RefCell::default();
    static MESSAGES: RefCell<Vec<Said>> = RefCell::default();
}

fn number_of(handle: *const seal_handle) -> usize {
    handle.addr()
}

fn handle_of(number: usize) -> *mut seal_handle {
    std::ptr::without_provenance_mut(number)
}

// ---- messages ----

fn c_text(text: &str) -> CString {
    CString::new(text.replace('\0', " ")).unwrap_or_default()
}

fn say(message: &Message) {
    let kind = match message.kind {
        MessageKind::Refused => 0,
        MessageKind::CheckFailed => 1,
        MessageKind::LimitReached => 2,
        MessageKind::NotDesignedYet => 3,
        MessageKind::NotBuiltYet => 4,
        MessageKind::Failed => 5,
        MessageKind::Internal => 6,
    };
    let said = Said {
        kind,
        text: c_text(&message.text()),
        what: message.what.as_deref().map(c_text),
        why: message.why.as_deref().map(c_text),
        fix: message.fix.as_deref().map(c_text),
        when: message.when.as_deref().map(c_text),
        file: (!message.file.is_empty()).then(|| c_text(&message.file)),
        line: message.place.line,
        column: message.place.column,
        width: message.place.width,
    };
    MESSAGES.with(|messages| messages.borrow_mut().push(said));
}

/// The host used the API wrongly: said, naming the function and what was wrong with its call, and
/// `SEAL_HOST_MISTAKE` given. A host's programmer knows C, so it needs no `why:`.
fn mistake(what: &str) -> seal_status {
    let said = Said {
        kind: 6,
        text: c_text(&format!("mistake: {what}\n")),
        what: Some(c_text(what)),
        why: None,
        fix: None,
        when: None,
        file: None,
        line: 0,
        column: 0,
        width: 0,
    };
    MESSAGES.with(|messages| messages.borrow_mut().push(said));
    SEAL_HOST_MISTAKE
}

/// An error from Seal: its messages said, and its status given.
fn error(error: &Error) -> seal_status {
    match error {
        Error::Refused(messages) => {
            for message in &messages.list {
                say(message);
            }
            SEAL_REFUSED
        }
        Error::Stopped(message) => {
            say(message);
            SEAL_STOPPED
        }
        Error::Failed(failure) => {
            let said = Said {
                kind: 5,
                text: c_text(&format!("failed: {failure}\n")),
                what: Some(c_text(failure)),
                why: None,
                fix: None,
                when: None,
                file: None,
                line: 0,
                column: 0,
                width: 0,
            };
            MESSAGES.with(|messages| messages.borrow_mut().push(said));
            SEAL_FAILED
        }
    }
}

/// Runs an API function's body: messages from the call before cleared, and a panic caught and said, never let
/// through into C (D14).
fn guarded(body: impl FnOnce() -> seal_status) -> seal_status {
    MESSAGES.with(|messages| messages.borrow_mut().clear());
    catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|_| {
        say(&Message::internal("Seal's C API went wrong"));
        SEAL_STOPPED
    })
}

/// As [`guarded`], for a function giving a handle: null when it can't.
fn guarded_handle(body: impl FnOnce() -> Option<usize>) -> *mut seal_handle {
    MESSAGES.with(|messages| messages.borrow_mut().clear());
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(Some(number)) => handle_of(number),
        Ok(None) => std::ptr::null_mut(),
        Err(_) => {
            say(&Message::internal("Seal's C API went wrong"));
            std::ptr::null_mut()
        }
    }
}

// ---- reading what the host gives ----

/// The text a C string holds.
///
/// # Safety
///
/// `text` is null, or a NUL-ended string that stays as it is during the call, as the header asks of every
/// `const char *`.
unsafe fn read_text<'a>(text: *const c_char) -> Option<&'a str> {
    if text.is_null() {
        return None;
    }
    // SAFETY: not null, and the caller promises a NUL-ended string that outlives the call.
    unsafe { CStr::from_ptr(text) }.to_str().ok()
}

/// The items an array holds; none when it's null and empty.
///
/// # Safety
///
/// `items` is null with `count` 0, or points at `count` items that stay as they are during the call.
unsafe fn read_array<'a, T>(items: *const T, count: usize) -> Option<&'a [T]> {
    if count == 0 {
        return Some(&[]);
    }
    if items.is_null() {
        return None;
    }
    // SAFETY: not null, and the caller promises `count` items that outlive the call.
    Some(unsafe { std::slice::from_raw_parts(items, count) })
}

/// A text parameter of `function`, or the host mistake naming it.
///
/// # Safety
///
/// As [`read_text`].
unsafe fn text_in<'a>(function: &str, parameter: &str, text: *const c_char) -> Result<&'a str, seal_status> {
    if text.is_null() {
        return Err(mistake(&format!("`{function}`: `{parameter}` is null")));
    }
    // SAFETY: as the caller promises.
    unsafe { read_text(text) }.ok_or_else(|| mistake(&format!("`{function}`: `{parameter}` isn't UTF-8")))
}

/// An array parameter of `function`, or the host mistake naming it.
///
/// # Safety
///
/// As [`read_array`].
unsafe fn array_in<'a, T>(
    function: &str,
    parameter: &str,
    counted_by: &str,
    items: *const T,
    count: usize,
) -> Result<&'a [T], seal_status> {
    // SAFETY: as the caller promises.
    unsafe { read_array(items, count) }.ok_or_else(|| {
        mistake(&format!("`{function}`: `{parameter}` is null, but `{counted_by}` says there are {count}"))
    })
}

/// The host mistake of an answer's place that's null.
fn nowhere(function: &str, parameter: &str, what: &str) -> seal_status {
    mistake(&format!("`{function}`: `{parameter}` is null, so there's nowhere to write {what}"))
}

/// The host mistake of a handle that's freed or was never one.
fn not_a_handle(function: &str, what: &str) -> seal_status {
    mistake(&format!("`{function}`: a {what} handle that's freed, or was never one"))
}

/// Writes an answer where the host asked for it.
///
/// # Safety
///
/// `place` is null, or points where a `T` can be written.
unsafe fn write<T>(place: *mut T, value: T) -> bool {
    if place.is_null() {
        return false;
    }
    // SAFETY: not null, and the caller promises room for a `T`.
    unsafe { place.write(value) };
    true
}

// ---- values ----

fn plain(kind: seal_kind, bits: u64) -> seal_arg {
    seal_arg { kind, bits }
}

/// A value given to the host as an argument: a plain one in the argument itself, anything else as a new handle
/// the host holds, or lends when `lent`.
fn give(value: Value, lent: bool) -> seal_arg {
    match value {
        Value::Nothing => plain(NOTHING, 0),
        Value::Whole(whole) => plain(WHOLE, whole.cast_unsigned()),
        Value::Float(float) => plain(FLOAT, float.to_bits()),
        Value::Bool(flag) => plain(BOOL, u64::from(flag)),
        other => {
            let kind = kind_of(&other);
            let number = VALUES.with(|values| values.borrow_mut().insert(Held { value: other, lent, text: None }));
            plain(kind, number as u64)
        }
    }
}

/// A kind as the header names it, for messages.
fn kind_name(kind: seal_kind) -> &'static str {
    match kind {
        NOTHING => "`SEAL_NOTHING`",
        WHOLE => "`SEAL_WHOLE`",
        FLOAT => "`SEAL_FLOAT`",
        BOOL => "`SEAL_BOOL`",
        TEXT => "`SEAL_TEXT`",
        HUGE_WHOLE => "`SEAL_HUGE_WHOLE`",
        DECIMAL => "`SEAL_DECIMAL`",
        HUGE_DECIMAL => "`SEAL_HUGE_DECIMAL`",
        LIST => "`SEAL_LIST`",
        MAYBE => "`SEAL_MAYBE`",
        DICT => "`SEAL_DICT`",
        RECORD => "`SEAL_RECORD`",
        CASE => "`SEAL_CASE`",
        _ => "a kind that isn't one",
    }
}

fn kind_of(value: &Value) -> seal_kind {
    match value {
        Value::Nothing => NOTHING,
        Value::Whole(_) => WHOLE,
        Value::Float(_) => FLOAT,
        Value::Bool(_) => BOOL,
        Value::Text(_) => TEXT,
        Value::HugeWhole(_) => HUGE_WHOLE,
        Value::Decimal(_) => DECIMAL,
        Value::HugeDecimal(_) => HUGE_DECIMAL,
        Value::List(_) => LIST,
        Value::Maybe(_) => MAYBE,
        Value::Dict(_) => DICT,
        Value::Record(_) => RECORD,
        Value::Case { .. } => CASE,
    }
}

/// The value an argument stands for: taken out of its handle when `take`, copied otherwise.
fn receive(argument: seal_arg, take: bool) -> Result<Value, String> {
    Ok(match argument.kind {
        NOTHING => Value::Nothing,
        WHOLE => Value::Whole(argument.bits.cast_signed()),
        FLOAT => {
            let float = f64::from_bits(argument.bits);
            if !float.is_finite() {
                return Err("a `Float` that's infinite or not a number, which Seal never holds (D18)".to_owned());
            }
            Value::Float(float)
        }
        // A C `int` is the low four bytes; the others may be anything.
        BOOL => Value::Bool(argument.bits & 0xffff_ffff != 0),
        TEXT | HUGE_WHOLE | DECIMAL | HUGE_DECIMAL | LIST | MAYBE | DICT | RECORD | CASE => {
            let number = usize::try_from(argument.bits).map_err(|_| "a handle that isn't one".to_owned())?;
            let value = VALUES.with(|values| {
                let mut values = values.borrow_mut();
                if take {
                    match values.get(number) {
                        Some(held) if held.lent => Err("a value lent for a door's call, given away".to_owned()),
                        Some(_) => Ok(values.remove(number).map(|held| held.value)),
                        None => Ok(None),
                    }
                } else {
                    Ok(values.get(number).map(|held| held.value.clone()))
                }
            })?;
            let value = value.ok_or_else(|| "a value handle that's freed, or was never one".to_owned())?;
            if kind_of(&value) != argument.kind {
                return Err(format!(
                    "a handle to {}, but its `kind` says {}",
                    kind_name(kind_of(&value)),
                    kind_name(argument.kind)
                ));
            }
            value
        }
        _ => return Err(format!("an argument of the kind {}, which isn't one", argument.kind)),
    })
}

/// trace: D136. A type as a C host writes it, as a script would: Seal's own types, and lists, `Maybe`s and `Dict`s
/// of them. A host's records and `one_of`s at a C door aren't built yet.
fn read_type(written: &str) -> Result<HostType, String> {
    let written = written.trim();
    let inner = |prefix: &str| written.strip_prefix(prefix).and_then(|rest| rest.strip_suffix('>'));
    Ok(match written {
        "Nothing" => HostType::Nothing,
        "Whole" => HostType::Whole,
        "Float" => HostType::Float,
        "Bool" => HostType::Bool,
        "Text" => HostType::Text,
        "HugeWhole" => HostType::HugeWhole,
        "Decimal" => HostType::Decimal,
        "HugeDecimal" => HostType::HugeDecimal,
        _ => {
            if let Some(item) = inner("List<") {
                HostType::List(Box::new(read_type(item)?))
            } else if let Some(item) = inner("Maybe<") {
                HostType::Maybe(Box::new(read_type(item)?))
            } else if let Some(pair) = inner("Dict<") {
                let mut depth = 0i32;
                let split = pair.char_indices().find(|&(_, character)| {
                    depth += match character {
                        '<' => 1,
                        '>' => -1,
                        _ => 0,
                    };
                    character == ',' && depth == 0
                });
                let (key, value) = split.map(|(at, _)| pair.split_at(at)).ok_or("a `Dict` with no `value:`")?;
                let key = key.trim().strip_prefix("key:").ok_or("a `Dict` with no `key:`")?;
                let value =
                    value.trim_start_matches(',').trim().strip_prefix("value:").ok_or("a `Dict` with no `value:`")?;
                HostType::Dict(Box::new(read_type(key)?), Box::new(read_type(value)?))
            } else {
                return Err(format!(
                    "`{written}`, which isn't a type a C door takes: Seal's own, and lists, `Maybe`s and `Dict`s of them"
                ));
            }
        }
    })
}

/// Whether a value a C door gives back is of the type it was opened with.
fn fits(value: &Value, wanted: &HostType) -> bool {
    match (value, wanted) {
        (Value::Nothing, HostType::Nothing)
        | (Value::Whole(_), HostType::Whole)
        | (Value::Bool(_), HostType::Bool)
        | (Value::Text(_), HostType::Text)
        | (Value::HugeWhole(_), HostType::HugeWhole)
        | (Value::Decimal(_), HostType::Decimal)
        | (Value::HugeDecimal(_), HostType::HugeDecimal) => true,
        (Value::Float(float), HostType::Float) => float.is_finite(),
        (Value::List(items), HostType::List(item)) => items.iter().all(|value| fits(value, item)),
        (Value::Maybe(inner), HostType::Maybe(item)) => inner.as_deref().is_none_or(|value| fits(value, item)),
        (Value::Dict(pairs), HostType::Dict(key, item)) => {
            pairs.iter().all(|(found_key, value)| fits(found_key, key) && fits(value, item))
        }
        _ => false,
    }
}

/// The kind an argument of a type crosses a door as: a number with a unit is its plain number (D111).
fn crossing_kind(host_type: &HostType) -> seal_kind {
    match host_type {
        HostType::Nothing => NOTHING,
        HostType::Whole => WHOLE,
        HostType::Float => FLOAT,
        HostType::Bool => BOOL,
        HostType::Text => TEXT,
        HostType::HugeWhole => HUGE_WHOLE,
        HostType::Decimal => DECIMAL,
        HostType::HugeDecimal => HUGE_DECIMAL,
        HostType::List(_) => LIST,
        HostType::Dict(..) => DICT,
        HostType::Maybe(_) => MAYBE,
        HostType::Record { .. } => RECORD,
        HostType::OneOf { .. } => CASE,
        HostType::Measured { base, .. } => crossing_kind(base),
    }
}

// ---- doors ----

/// The host's data for a door, handed back to it on every call. Seal never reads or keeps what it points at.
struct HostData(*mut c_void);

// SAFETY: Seal only hands the pointer back to the host's own door, on the thread the host is calling Seal on, which
// is where the host's handles live; what it points at is the host's to keep safe, as the header says.
unsafe impl Send for HostData {}
// SAFETY: as above; Seal never reads through the pointer.
unsafe impl Sync for HostData {}

/// A C door as Seal keeps it: what the host opened it with, and each argument's kind, worked out once (D152).
struct CDoor {
    function: unsafe extern "C" fn(*mut c_void, *const seal_arg, usize, *mut seal_arg) -> seal_status,
    data: HostData,
    kinds: Vec<seal_kind>,
    /// Whether every argument crosses as its bits, so that nothing is lent and nothing taken back.
    plain_only: bool,
    result_type: HostType,
    result_kind: seal_kind,
    failures: Vec<String>,
    can_pause: bool,
}

/// How many arguments a door call holds without making room for them.
const ARGUMENTS_HELD: usize = 8;

/// Whether a kind crosses as its bits, in the argument itself.
fn crosses_plain(kind: seal_kind) -> bool {
    matches!(kind, WHOLE | FLOAT | BOOL | NOTHING)
}

/// A C door, called as Seal calls a door: plain arguments as their bits, the rest lent as handles for the call.
fn call_door(door: &CDoor, bits: &[u64], values: &mut [Value]) -> DoorAnswer {
    let mut held = [plain(NOTHING, 0); ARGUMENTS_HELD];
    let mut made = Vec::new();
    let arguments: &mut [seal_arg] = if let Some(held) = held.get_mut(..door.kinds.len()) {
        held
    } else {
        made.resize(door.kinds.len(), plain(NOTHING, 0));
        &mut made
    };
    for (place, (argument, &kind)) in arguments.iter_mut().zip(&door.kinds).enumerate() {
        *argument = if crosses_plain(kind) {
            plain(kind, bits.get(place).copied().unwrap_or(0))
        } else {
            give(values.get_mut(place).map(std::mem::take).unwrap_or_default(), true)
        };
    }
    let mut result = plain(NOTHING, 0);
    // SAFETY: the host opened this door with this function, promising it takes these arguments: the host's data,
    // the arguments, valid for the call, their count, and where to write its result.
    let status = unsafe { (door.function)(door.data.0, arguments.as_ptr(), arguments.len(), &raw mut result) };
    // What was lent is taken back, whatever the door did with it.
    if !door.plain_only {
        VALUES.with(|values| {
            let mut values = values.borrow_mut();
            for argument in arguments.iter().filter(|argument| !crosses_plain(argument.kind)) {
                values.remove(usize::try_from(argument.bits).unwrap_or(0));
            }
        });
    }
    match status {
        SEAL_OK => match door.result_kind {
            NOTHING => DoorAnswer::Plain(0),
            WHOLE => DoorAnswer::Plain(result.bits),
            FLOAT if f64::from_bits(result.bits).is_finite() => DoorAnswer::Plain(result.bits),
            FLOAT => DoorAnswer::Mistake("a C door gave a `Float` that's infinite or not a number".to_owned()),
            BOOL => DoorAnswer::Plain(u64::from(result.bits & 0xffff_ffff != 0)),
            _ => match receive(result, true) {
                Ok(value) if fits(&value, &door.result_type) => DoorAnswer::Value(value),
                Ok(_) => {
                    DoorAnswer::Mistake("a C door gave a value of another type than it was opened with".to_owned())
                }
                Err(why) => DoorAnswer::Mistake(format!("a C door gave {why}")),
            },
        },
        SEAL_PAUSED if door.can_pause => DoorAnswer::Pause,
        SEAL_FAILED => match usize::try_from(result.bits).ok().and_then(|place| door.failures.get(place)) {
            Some(failure) => DoorAnswer::Failure(failure.clone()),
            None => DoorAnswer::Mistake("a C door failed with a failure it wasn't opened with".to_owned()),
        },
        other => DoorAnswer::Mistake(format!("a C door gave the status {other}")),
    }
}

/// `seal_doors_new`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_doors_new() -> *mut seal_doors {
    guarded_handle(|| Some(DOORS.with(|doors| doors.borrow_mut().insert(Doors::new()))))
}

/// `seal_doors_free`. A handle that isn't one is ignored, as `free(NULL)` is.
///
/// # Safety
///
/// None needed: the handle is only a number.
#[unsafe(no_mangle)]
pub extern "C" fn seal_doors_free(doors: *mut seal_doors) {
    let _ = guarded(|| {
        DOORS.with(|table| table.borrow_mut().remove(number_of(doors)));
        SEAL_OK
    });
}

/// `seal_doors_add`.
///
/// # Safety
///
/// Every pointer is null or as the header says: a NUL-ended string, or an array of `count` of them.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_doors_add(
    doors: *mut seal_doors,
    name: *const c_char,
    value_types: *const *const c_char,
    count: usize,
    result_type: *const c_char,
    failures: *const *const c_char,
    failure_count: usize,
    can_pause: c_int,
    function: seal_door_function,
    host_data: *mut c_void,
) -> seal_status {
    guarded(|| {
        const ADD: &str = "seal_doors_add";
        // SAFETY: the header's rules for every pointer, which the caller promises.
        let read = unsafe {
            (
                text_in(ADD, "name", name),
                array_in(ADD, "value_types", "count", value_types, count),
                text_in(ADD, "result_type", result_type),
                array_in(ADD, "failures", "failure_count", failures, failure_count),
            )
        };
        let (name, types, result, failures) = match read {
            (Ok(name), Ok(types), Ok(result), Ok(failures)) => (name, types, result, failures),
            (Err(status), ..) | (_, Err(status), ..) | (_, _, Err(status), _) | (.., Err(status)) => return status,
        };
        let Some(function) = function else { return mistake("`seal_doors_add`: `function` is null") };
        let mut parameters = Vec::with_capacity(types.len());
        for (place, &written) in types.iter().enumerate() {
            // SAFETY: each is a NUL-ended string, as the header asks.
            let written = match unsafe { text_in(ADD, &format!("value_types[{place}]"), written) } {
                Ok(written) => written,
                Err(status) => return status,
            };
            match read_type(written) {
                Ok(host_type) => parameters.push(host_type),
                Err(why) => return mistake(&format!("`{ADD}`: `value_types[{place}]` is {why}")),
            }
        }
        let result = match read_type(result) {
            Ok(host_type) => host_type,
            Err(why) => return mistake(&format!("`{ADD}`: `result_type` is {why}")),
        };
        let mut failure_names = Vec::with_capacity(failures.len());
        for (place, &failure) in failures.iter().enumerate() {
            // SAFETY: as above.
            match unsafe { text_in(ADD, &format!("failures[{place}]"), failure) } {
                Ok(failure) => failure_names.push(failure.to_owned()),
                Err(status) => return status,
            }
        }
        let kinds: Vec<seal_kind> = parameters.iter().map(crossing_kind).collect();
        let kept = CDoor {
            function,
            data: HostData(host_data),
            plain_only: kinds.iter().all(|&kind| crosses_plain(kind)),
            kinds,
            result_kind: crossing_kind(&result),
            result_type: result.clone(),
            failures: failure_names.clone(),
            can_pause: can_pause != 0,
        };
        let shape = (parameters, result, failure_names, can_pause != 0);
        let door = move |bits: &[u64], values: &mut Vec<Value>| call_door(&kept, bits, values);
        let added = DOORS.with(|table| {
            table.borrow_mut().get_mut(number_of(doors)).map(|doors| doors.add_raw(name, shape, door).map(|_| ()))
        });
        match added {
            Some(Ok(())) => SEAL_OK,
            Some(Err(message)) => {
                say(&message);
                SEAL_REFUSED
            }
            None => not_a_handle("seal_doors_add", "doors"),
        }
    })
}

/// `seal_doors_labels`.
///
/// # Safety
///
/// As [`seal_doors_add`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_doors_labels(
    doors: *mut seal_doors,
    name: *const c_char,
    labels: *const *const c_char,
    count: usize,
) -> seal_status {
    guarded(|| {
        const LABELS: &str = "seal_doors_labels";
        // SAFETY: the header's rules for every pointer.
        let read = unsafe { (text_in(LABELS, "name", name), array_in(LABELS, "labels", "count", labels, count)) };
        let (name, labels) = match read {
            (Ok(name), Ok(labels)) => (name, labels),
            (Err(status), _) | (_, Err(status)) => return status,
        };
        let mut written = Vec::with_capacity(labels.len());
        for (place, &label) in labels.iter().enumerate() {
            // SAFETY: as above.
            match unsafe { text_in(LABELS, &format!("labels[{place}]"), label) } {
                Ok(label) => written.push(label),
                Err(status) => return status,
            }
        }
        let labelled = DOORS.with(|table| {
            let mut table = table.borrow_mut();
            let doors = table.get_mut(number_of(doors))?;
            Some(doors.options_of(name).map(|options| options.labels_of(&written).map(|_| ())))
        });
        match labelled {
            Some(Some(Ok(()))) => SEAL_OK,
            Some(Some(Err(message))) => {
                say(&message);
                SEAL_REFUSED
            }
            Some(None) => mistake(&format!("`seal_doors_labels`: there's no open door named `{name}`")),
            None => not_a_handle("seal_doors_labels", "doors"),
        }
    })
}

fn doors_of(doors: *const seal_doors) -> Option<Doors> {
    DOORS.with(|table| table.borrow().get(number_of(doors)).cloned())
}

// ---- scripts ----

fn limits_of(limits: seal_limits) -> Limits {
    Limits { steps: limits.steps, memory: limits.memory }
}

fn hold_script(function: &str, loaded: Result<Script, Error>, script: *mut *mut seal_script) -> seal_status {
    match loaded {
        Ok(loaded) => {
            let number = SCRIPTS.with(|table| table.borrow_mut().insert(Some(loaded)));
            // SAFETY: the caller promises `script` is null or room for a handle.
            if unsafe { write(script, handle_of(number)) } {
                SEAL_OK
            } else {
                SCRIPTS.with(|table| table.borrow_mut().remove(number));
                nowhere(function, "script", "the script's handle")
            }
        }
        Err(problem) => error(&problem),
    }
}

/// `seal_script_load`.
///
/// # Safety
///
/// `name` is a NUL-ended string, `source` points at `length` bytes, and `script` at room for a handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_script_load(
    name: *const c_char,
    source: *const c_char,
    length: usize,
    doors: *const seal_doors,
    limits: seal_limits,
    script: *mut *mut seal_script,
) -> seal_status {
    guarded(|| {
        const LOAD: &str = "seal_script_load";
        // SAFETY: the header's rules for every pointer.
        let read =
            unsafe { (text_in(LOAD, "name", name), array_in(LOAD, "source", "length", source.cast::<u8>(), length)) };
        let (name, bytes) = match read {
            (Ok(name), Ok(bytes)) => (name, bytes),
            (Err(status), _) | (_, Err(status)) => return status,
        };
        let Ok(source) = std::str::from_utf8(bytes) else { return mistake("`seal_script_load`: `source` isn't UTF-8") };
        let Some(doors) = doors_of(doors) else { return not_a_handle(LOAD, "doors") };
        hold_script(LOAD, Script::load(name, source, &doors, limits_of(limits)), script)
    })
}

/// `seal_script_load_compiled` (D141, D153).
///
/// # Safety
///
/// `bytes` points at `length` bytes, `name` is a NUL-ended string, and `script` points at room for a handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_script_load_compiled(
    bytes: *const u8,
    length: usize,
    name: *const c_char,
    doors: *const seal_doors,
    limits: seal_limits,
    script: *mut *mut seal_script,
) -> seal_status {
    guarded(|| {
        const LOAD: &str = "seal_script_load_compiled";
        // SAFETY: the header's rules for every pointer.
        let read = unsafe { (array_in(LOAD, "bytes", "length", bytes, length), text_in(LOAD, "name", name)) };
        let (bytes, name) = match read {
            (Ok(bytes), Ok(name)) => (bytes, name),
            (Err(status), _) | (_, Err(status)) => return status,
        };
        let Some(doors) = doors_of(doors) else { return not_a_handle(LOAD, "doors") };
        hold_script(LOAD, Script::load_compiled(bytes, name, &doors, limits_of(limits)), script)
    })
}

/// `seal_script_free`. A handle that isn't one is ignored, as `free(NULL)` is; one in use by its own door's call is
/// left alone.
#[unsafe(no_mangle)]
pub extern "C" fn seal_script_free(script: *mut seal_script) {
    let _ = guarded(|| {
        SCRIPTS.with(|table| {
            let mut table = table.borrow_mut();
            if table.get(number_of(script)).is_some_and(Option::is_some) {
                table.remove(number_of(script));
            }
        });
        SEAL_OK
    });
}

/// Runs something on a script held by number, taken out of its place meanwhile, so a door can't run it again
/// from inside the call.
fn with_script(
    function: &str,
    script: *const seal_script,
    run: impl FnOnce(&mut Script) -> seal_status,
) -> seal_status {
    let number = number_of(script);
    let taken = SCRIPTS.with(|table| table.borrow_mut().get_mut(number).map(Option::take));
    let mut held = match taken {
        Some(Some(held)) => held,
        Some(None) => {
            return mistake(&format!(
                "`{function}`: the script is running, and was called again from one of its doors"
            ));
        }
        None => return not_a_handle(function, "script"),
    };
    let status = run(&mut held);
    SCRIPTS.with(|table| {
        if let Some(slot) = table.borrow_mut().get_mut(number) {
            *slot = Some(held);
        }
    });
    status
}

/// How a call came back, written where the host asked.
fn answer(
    (function, task_name): (&str, &str),
    called: Result<Called<Value>, Error>,
    result: *mut seal_arg,
    task: *mut u64,
) -> seal_status {
    match called {
        Ok(Called::Finished(value)) => {
            let given = give(value, false);
            // SAFETY: the caller promises `result` is null or room for an argument.
            if unsafe { write(result, given) } {
                SEAL_OK
            } else {
                if !matches!(given.kind, WHOLE | FLOAT | BOOL | NOTHING) {
                    VALUES.with(|values| values.borrow_mut().remove(usize::try_from(given.bits).unwrap_or(0)));
                }
                nowhere(function, "result", "the result")
            }
        }
        Ok(Called::Paused(paused)) => {
            // SAFETY: the caller promises `task` is null or room for a task.
            if unsafe { write(task, paused.to_bits()) } {
                SEAL_PAUSED
            } else {
                nowhere(function, task_name, "the paused task")
            }
        }
        Err(problem) => error(&problem),
    }
}

/// `seal_script_call`.
///
/// # Safety
///
/// `procedure` is a NUL-ended string; `labels` is null or `count` strings or nulls; `args` points at `count`
/// arguments; `result` and `task` at room for their answers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_script_call(
    script: *mut seal_script,
    procedure: *const c_char,
    labels: *const *const c_char,
    args: *const seal_arg,
    count: usize,
    result: *mut seal_arg,
    task: *mut u64,
) -> seal_status {
    guarded(|| {
        const CALL: &str = "seal_script_call";
        // SAFETY: the header's rules for every pointer.
        let read = unsafe { (text_in(CALL, "procedure", procedure), array_in(CALL, "args", "count", args, count)) };
        let (procedure, given) = match read {
            (Ok(procedure), Ok(given)) => (procedure, given),
            (Err(status), _) | (_, Err(status)) => return status,
        };
        let label_list: &[*const c_char] = if labels.is_null() {
            &[]
        } else {
            // SAFETY: not null, so `count` labels, as the header says.
            match unsafe { array_in(CALL, "labels", "count", labels, count) } {
                Ok(list) => list,
                Err(status) => return status,
            }
        };
        let mut arguments = Vec::with_capacity(given.len());
        for (place, &argument) in given.iter().enumerate() {
            let value = match receive(argument, false) {
                Ok(value) => value,
                Err(why) => return mistake(&format!("`{CALL}`: `args[{place}]` is {why}")),
            };
            let label = match label_list.get(place).copied() {
                Some(label) if !label.is_null() => {
                    // SAFETY: a NUL-ended string, as the header says.
                    match unsafe { text_in(CALL, &format!("labels[{place}]"), label) } {
                        Ok(label) => Some(label.to_owned()),
                        Err(status) => return status,
                    }
                }
                _ => None,
            };
            arguments.push(Argument { label, value });
        }
        with_script(CALL, script, |held| answer((CALL, "task"), held.call_values(procedure, &arguments), result, task))
    })
}

/// `seal_script_resume`.
///
/// # Safety
///
/// `value` is null, for nothing, or points at an argument; `result` and `paused_again` at room for their answers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_script_resume(
    script: *mut seal_script,
    task: u64,
    value: *const seal_arg,
    result: *mut seal_arg,
    paused_again: *mut u64,
) -> seal_status {
    guarded(|| {
        let given = if value.is_null() {
            Value::Nothing
        } else {
            // SAFETY: not null, so one argument, as the header says.
            let argument = unsafe { value.read() };
            match receive(argument, false) {
                Ok(value) => value,
                Err(why) => return mistake(&format!("`seal_script_resume`: `value` is {why}")),
            }
        };
        with_script("seal_script_resume", script, |held| {
            answer(
                ("seal_script_resume", "paused_again"),
                held.resume_with_value(TaskId::from_bits(task), &given),
                result,
                paused_again,
            )
        })
    })
}

/// `seal_script_cancel`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_script_cancel(script: *mut seal_script, task: u64) -> seal_status {
    guarded(|| {
        with_script("seal_script_cancel", script, |held| match held.cancel(TaskId::from_bits(task)) {
            Ok(()) => SEAL_OK,
            Err(problem) => error(&problem),
        })
    })
}

/// `seal_script_memory_in_use`: 0 for a handle that isn't one.
#[unsafe(no_mangle)]
pub extern "C" fn seal_script_memory_in_use(script: *const seal_script) -> usize {
    catch_unwind(|| {
        SCRIPTS.with(|table| {
            table.borrow().get(number_of(script)).and_then(Option::as_ref).map_or(0, Script::memory_in_use)
        })
    })
    .unwrap_or(0)
}

// ---- compiled files ----

/// `seal_build` (D141, D153): the bytes stay Seal's until `seal_bytes_free`.
///
/// # Safety
///
/// `names` and `sources` point at `count` NUL-ended strings; `bytes` and `length` at room for the answers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_build(
    doors: *const seal_doors,
    names: *const *const c_char,
    sources: *const *const c_char,
    count: usize,
    bytes: *mut *const u8,
    length: *mut usize,
) -> seal_status {
    guarded(|| {
        const BUILD: &str = "seal_build";
        // SAFETY: the header's rules for every pointer.
        let read = unsafe {
            (array_in(BUILD, "names", "count", names, count), array_in(BUILD, "sources", "count", sources, count))
        };
        let (names, sources) = match read {
            (Ok(names), Ok(sources)) => (names, sources),
            (Err(status), _) | (_, Err(status)) => return status,
        };
        let mut scripts = Vec::with_capacity(count);
        for (place, (&name, &source)) in names.iter().zip(sources).enumerate() {
            // SAFETY: each is a NUL-ended string, as the header says.
            let read = unsafe {
                (text_in(BUILD, &format!("names[{place}]"), name), text_in(BUILD, &format!("sources[{place}]"), source))
            };
            match read {
                (Ok(name), Ok(source)) => scripts.push((name, source)),
                (Err(status), _) | (_, Err(status)) => return status,
            }
        }
        let Some(doors) = doors_of(doors) else { return not_a_handle(BUILD, "doors") };
        match seal::build(&scripts, &doors) {
            Ok(built) => {
                let built = built.into_boxed_slice();
                let (start, size) = (built.as_ptr(), built.len());
                BYTES.with(|kept| kept.borrow_mut().push(built));
                // SAFETY: the caller promises room for each answer.
                if unsafe { write(bytes, start) } && unsafe { write(length, size) } {
                    SEAL_OK
                } else {
                    BYTES.with(|kept| kept.borrow_mut().retain(|kept| kept.as_ptr() != start));
                    nowhere(BUILD, if bytes.is_null() { "bytes" } else { "length" }, "the file")
                }
            }
            Err(problem) => error(&problem),
        }
    })
}

/// `seal_bytes_free`: bytes `seal_build` gave, found by where they start; anything else is ignored.
#[unsafe(no_mangle)]
pub extern "C" fn seal_bytes_free(bytes: *const u8) {
    let _ = guarded(|| {
        BYTES.with(|kept| kept.borrow_mut().retain(|kept| kept.as_ptr() != bytes));
        SEAL_OK
    });
}

// ---- messages ----

/// `seal_message_count`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_count() -> usize {
    MESSAGES.with(|messages| messages.borrow().len())
}

fn message_part(index: usize, part: impl Fn(&Said) -> Option<&CString>) -> *const c_char {
    MESSAGES.with(|messages| messages.borrow().get(index).and_then(part).map_or(std::ptr::null(), |text| text.as_ptr()))
}

fn message_number(index: usize, part: impl Fn(&Said) -> u32) -> u32 {
    MESSAGES.with(|messages| messages.borrow().get(index).map_or(0, part))
}

/// `seal_message_kind_of`: 6, a mistake, for an index past the messages.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_kind_of(index: usize) -> c_int {
    MESSAGES.with(|messages| messages.borrow().get(index).map_or(6, |said| said.kind))
}

/// `seal_message_text`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_text(index: usize) -> *const c_char {
    message_part(index, |said| Some(&said.text))
}

/// `seal_message_what`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_what(index: usize) -> *const c_char {
    message_part(index, |said| said.what.as_ref())
}

/// `seal_message_why`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_why(index: usize) -> *const c_char {
    message_part(index, |said| said.why.as_ref())
}

/// `seal_message_fix`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_fix(index: usize) -> *const c_char {
    message_part(index, |said| said.fix.as_ref())
}

/// `seal_message_when`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_when(index: usize) -> *const c_char {
    message_part(index, |said| said.when.as_ref())
}

/// `seal_message_file`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_file(index: usize) -> *const c_char {
    message_part(index, |said| said.file.as_ref())
}

/// `seal_message_line`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_line(index: usize) -> u32 {
    message_number(index, |said| said.line)
}

/// `seal_message_column`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_column(index: usize) -> u32 {
    message_number(index, |said| said.column)
}

/// `seal_message_width`.
#[unsafe(no_mangle)]
pub extern "C" fn seal_message_width(index: usize) -> u32 {
    message_number(index, |said| said.width)
}

// ---- values ----

/// `seal_value_text`.
///
/// # Safety
///
/// `text` points at `length` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_text(text: *const c_char, length: usize) -> *mut seal_value {
    guarded_handle(|| {
        // SAFETY: the header's rule for the pointer.
        let Ok(bytes) = (unsafe { array_in("seal_value_text", "text", "length", text.cast::<u8>(), length) }) else {
            return None;
        };
        let Ok(text) = std::str::from_utf8(bytes) else {
            mistake("`seal_value_text`: `text` isn't UTF-8");
            return None;
        };
        Some(VALUES.with(|values| {
            values.borrow_mut().insert(Held { value: Value::Text(text.to_owned()), lent: false, text: None })
        }))
    })
}

/// `seal_value_number`: a `HugeWhole`, `Decimal` or `HugeDecimal` from its digits, as a script writes them.
///
/// # Safety
///
/// `written` is a NUL-ended string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_number(kind: seal_kind, written: *const c_char) -> *mut seal_value {
    guarded_handle(|| {
        // SAFETY: the header's rule for the pointer.
        let Ok(written) = (unsafe { text_in("seal_value_number", "written", written) }) else {
            return None;
        };
        let value = match kind {
            HUGE_WHOLE => Big::parse(written).map(Value::HugeWhole),
            DECIMAL => {
                HugeDecimal::from_text(written).and_then(|exact| Decimal::rounded(&exact).ok()).map(Value::Decimal)
            }
            HUGE_DECIMAL => HugeDecimal::from_text(written).map(Value::HugeDecimal),
            _ => {
                mistake(&format!(
                    "`seal_value_number`: `kind` is {kind}, but it makes only `SEAL_HUGE_WHOLE`, `SEAL_DECIMAL` or \
                     `SEAL_HUGE_DECIMAL`; a plain number goes in a `seal_arg` itself"
                ));
                return None;
            }
        };
        let Some(value) = value else {
            mistake(&format!("`seal_value_number`: `{written}` isn't a number written as a script writes one"));
            return None;
        };
        Some(VALUES.with(|values| values.borrow_mut().insert(Held { value, lent: false, text: None })))
    })
}

/// `seal_value_list`: the items' handles are taken over.
///
/// # Safety
///
/// `items` points at `count` arguments.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_list(items: *const seal_arg, count: usize) -> *mut seal_value {
    guarded_handle(|| {
        // SAFETY: the header's rule for the pointer.
        let Ok(items) = (unsafe { array_in("seal_value_list", "items", "count", items, count) }) else {
            return None;
        };
        let mut list = Vec::with_capacity(items.len());
        for &item in items {
            match receive(item, true) {
                Ok(value) => list.push(value),
                Err(why) => {
                    mistake(&format!("`seal_value_list`: `items[{}]` is {why}", list.len()));
                    return None;
                }
            }
        }
        Some(
            VALUES
                .with(|values| values.borrow_mut().insert(Held { value: Value::List(list), lent: false, text: None })),
        )
    })
}

/// `seal_value_maybe`: `none` for a null, or `some` of what it points at, whose handle is taken over.
///
/// # Safety
///
/// `inner` is null or points at an argument.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_maybe(inner: *const seal_arg) -> *mut seal_value {
    guarded_handle(|| {
        let value = if inner.is_null() {
            Value::Maybe(None)
        } else {
            // SAFETY: not null, so one argument, as the header says.
            match receive(unsafe { inner.read() }, true) {
                Ok(value) => Value::Maybe(Some(Box::new(value))),
                Err(why) => {
                    mistake(&format!("`seal_value_maybe`: `inner` is {why}"));
                    return None;
                }
            }
        };
        Some(VALUES.with(|values| values.borrow_mut().insert(Held { value, lent: false, text: None })))
    })
}

/// `seal_value_free`: a value lent for a door's call is the door's caller's to free, and is left alone.
#[unsafe(no_mangle)]
pub extern "C" fn seal_value_free(value: *mut seal_value) {
    let _ = guarded(|| {
        VALUES.with(|values| {
            let mut values = values.borrow_mut();
            if values.get(number_of(value)).is_some_and(|held| !held.lent) {
                values.remove(number_of(value));
            }
        });
        SEAL_OK
    });
}

/// `seal_value_kind`.
///
/// # Safety
///
/// `kind` points at room for a kind.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_kind(value: *const seal_value, kind: *mut seal_kind) -> seal_status {
    guarded(|| {
        let Some(found) = VALUES.with(|values| values.borrow().get(number_of(value)).map(|held| kind_of(&held.value)))
        else {
            return not_a_handle("seal_value_kind", "value");
        };
        // SAFETY: the caller promises room for a kind.
        if unsafe { write(kind, found) } { SEAL_OK } else { nowhere("seal_value_kind", "kind", "the kind") }
    })
}

/// `seal_value_get_text`: a text's, or a big number's digits as a script writes them; the pointer is good until
/// the value is freed.
///
/// # Safety
///
/// `text` and `length` point at room for their answers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_get_text(
    value: *const seal_value,
    text: *mut *const c_char,
    length: *mut usize,
) -> seal_status {
    guarded(|| {
        let found = VALUES.with(|values| {
            let mut values = values.borrow_mut();
            let held = values.get_mut(number_of(value))?;
            let written = match &held.value {
                Value::Text(text) => text.clone(),
                Value::HugeWhole(big) => big.to_decimal(),
                Value::Decimal(decimal) => decimal.to_text(),
                Value::HugeDecimal(exact) => exact.to_text(),
                other => return Some(Err(kind_of(other))),
            };
            let kept = held.text.insert(c_text(&written));
            Some(Ok((kept.as_ptr(), written.len())))
        });
        match found {
            Some(Ok((start, size))) => {
                // SAFETY: the caller promises room for each answer.
                if unsafe { write(text, start) } && unsafe { write(length, size) } {
                    SEAL_OK
                } else {
                    nowhere("seal_value_get_text", if text.is_null() { "text" } else { "length" }, "the text")
                }
            }
            Some(Err(kind)) => mistake(&format!(
                "`seal_value_get_text`: `value` is {}, which has no text; it reads a `SEAL_TEXT` or a big number",
                kind_name(kind)
            )),
            None => not_a_handle("seal_value_get_text", "value"),
        }
    })
}

/// `seal_value_count`: a list's items, a `Dict`'s pairs, a record's fields, a case's values, a `Maybe`'s one or none.
///
/// # Safety
///
/// `count` points at room for a count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_count(value: *const seal_value, count: *mut usize) -> seal_status {
    guarded(|| {
        let found = VALUES.with(|values| {
            values.borrow().get(number_of(value)).map(|held| match &held.value {
                Value::List(items) | Value::Record(items) | Value::Case { fields: items, .. } => Ok(items.len()),
                Value::Dict(pairs) => Ok(pairs.len()),
                Value::Maybe(inner) => Ok(usize::from(inner.is_some())),
                other => Err(kind_of(other)),
            })
        });
        match found {
            Some(Ok(size)) => {
                // SAFETY: the caller promises room for a count.
                if unsafe { write(count, size) } { SEAL_OK } else { nowhere("seal_value_count", "count", "the count") }
            }
            Some(Err(kind)) => {
                mistake(&format!("`seal_value_count`: `value` is {}, which has no items", kind_name(kind)))
            }
            None => not_a_handle("seal_value_count", "value"),
        }
    })
}

/// `seal_value_item`: an item, a big one as a new handle the host frees. A `Dict`'s pair is a list of its key and
/// its value.
///
/// # Safety
///
/// `item` points at room for an argument.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn seal_value_item(
    value: *const seal_value,
    position: usize,
    item: *mut seal_arg,
) -> seal_status {
    guarded(|| {
        let found = VALUES.with(|values| {
            values.borrow().get(number_of(value)).map(|held| match &held.value {
                Value::List(items) | Value::Record(items) | Value::Case { fields: items, .. } => {
                    items.get(position).cloned().ok_or(Ok(items.len()))
                }
                Value::Dict(pairs) => pairs
                    .get(position)
                    .map(|(key, value)| Value::List(vec![key.clone(), value.clone()]))
                    .ok_or(Ok(pairs.len())),
                Value::Maybe(inner) => {
                    inner.as_deref().filter(|_| position == 0).cloned().ok_or(Ok(usize::from(inner.is_some())))
                }
                other => Err(Err(kind_of(other))),
            })
        });
        match found {
            Some(Ok(found)) => {
                let given = give(found, false);
                // SAFETY: the caller promises room for an argument.
                if unsafe { write(item, given) } { SEAL_OK } else { nowhere("seal_value_item", "item", "the item") }
            }
            Some(Err(Ok(count))) => mistake(&format!(
                "`seal_value_item`: `position` is {position}, but `value` has {count} items, counted from 0"
            )),
            Some(Err(Err(kind))) => {
                mistake(&format!("`seal_value_item`: `value` is {}, which has no items", kind_name(kind)))
            }
            None => not_a_handle("seal_value_item", "value"),
        }
    })
}
