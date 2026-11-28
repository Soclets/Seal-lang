//! Values crossing between a script's heap and its host (D15, D48): always as copies.

use super::dict::{Dict, Probe};
use super::heap::{Heap, HeapError, Number, NumberRef, Object, SLOT_COST};
use crate::compile::bytecode::{Crossing, Layout, PLAIN_CASE};
use crate::host::Value;

/// Why a host's value couldn't go into a script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrossingError {
    Heap(HeapError),
    /// The value isn't of the type the crossing is for: what was wanted and what came.
    WrongType {
        wanted: String,
        found: &'static str,
    },
    /// The value is nested deeper than [`DEEPEST`] (D72).
    TooDeep,
}

/// trace: D72, Building rule 1. How deep a value may be nested to cross between a script and its host, counting
/// every list, `Maybe`, record and case on the way down. A type that holds itself can make a value as deep as memory
/// allows. Wax copies it with no recursion, but the host's own code drops, copies, compares, prints and converts a
/// [`Value`] one level at a time on its stack: a debug build takes up to about 2 KB a level, so 256 levels stay well
/// inside a thread's usual 2 MB.
pub const DEEPEST: usize = 256;

/// The full description of each record and `one_of` a crossing reaches, for a type met again inside itself (D72),
/// found by going through a list of what's left rather than by recursion.
fn described(root: &Crossing) -> Vec<&Crossing> {
    let mut found = Vec::new();
    let mut left = vec![root];
    while let Some(crossing) = left.pop() {
        match crossing {
            Crossing::List(inner) | Crossing::Maybe { inner, .. } => left.push(inner),
            Crossing::Dict(key, value) => {
                left.push(key);
                left.push(value);
            }
            Crossing::Record { fields, .. } => {
                found.push(crossing);
                left.extend(fields.iter().map(|(_, field)| field));
            }
            Crossing::OneOf { cases, .. } => {
                found.push(crossing);
                left.extend(cases.iter().flat_map(|case| case.fields.iter().map(|(_, field)| field)));
            }
            _ => {}
        }
    }
    found
}

/// What a crossing stands for: itself, or for a type met again inside itself, the record or `one_of` of that name,
/// whose descriptions are found the first time one is needed.
fn resolved<'c>(
    crossing: &'c Crossing,
    root: &'c Crossing,
    named: &mut Option<Vec<&'c Crossing>>,
) -> Result<&'c Crossing, HeapError> {
    let Crossing::Itself { name } = crossing else { return Ok(crossing) };
    named
        .get_or_insert_with(|| described(root))
        .iter()
        .copied()
        .find(|found| {
            matches!(found, Crossing::Record { name: known, .. } | Crossing::OneOf { name: known, .. } if known == name)
        })
        .ok_or(HeapError::Broken("a type met again inside itself that nothing describes"))
}

impl From<HeapError> for CrossingError {
    fn from(error: HeapError) -> Self {
        CrossingError::Heap(error)
    }
}

/// How a crossing's type reads in a message.
#[must_use]
pub fn describe(crossing: &Crossing) -> String {
    match crossing {
        Crossing::Nothing => "nothing".to_owned(),
        Crossing::Whole => "a `Whole`".to_owned(),
        Crossing::Float => "a `Float`".to_owned(),
        Crossing::Bool => "a `Bool`".to_owned(),
        Crossing::Text => "a `Text`".to_owned(),
        Crossing::HugeWhole => "a `HugeWhole`".to_owned(),
        Crossing::Decimal => "a `Decimal`".to_owned(),
        Crossing::HugeDecimal => "a `HugeDecimal`".to_owned(),
        Crossing::List(_) => "a list".to_owned(),
        Crossing::Dict(..) => "a `Dict`".to_owned(),
        Crossing::Maybe { .. } => "a `Maybe`".to_owned(),
        Crossing::Record { name, .. } | Crossing::OneOf { name, .. } | Crossing::Itself { name } => {
            format!("a `{name}`")
        }
    }
}

fn wrong(crossing: &Crossing, value: &Value) -> CrossingError {
    CrossingError::WrongType { wanted: describe(crossing), found: value.describe() }
}

/// A plain number, `Bool` or nothing going into a script: no heap is needed, so it's done on the spot.
#[must_use]
#[inline]
pub fn plain_slot(value: &Value, crossing: &Crossing) -> Option<u64> {
    match (crossing, value) {
        (Crossing::Nothing, Value::Nothing) => Some(0),
        (Crossing::Whole, Value::Whole(whole)) => Some(whole.cast_unsigned()),
        (Crossing::Float, Value::Float(float)) if float.is_finite() => Some(float.to_bits()),
        (Crossing::Bool, Value::Bool(flag)) => Some(u64::from(*flag)),
        _ => None,
    }
}

/// A plain number or `Bool` coming out of a script, done on the spot.
#[must_use]
#[inline]
pub fn plain_value(slot: u64, crossing: &Crossing) -> Option<Value> {
    match crossing {
        Crossing::Nothing => Some(Value::Nothing),
        Crossing::Whole => Some(Value::Whole(slot.cast_signed())),
        Crossing::Float => Some(Value::Float(f64::from_bits(slot))),
        Crossing::Bool => Some(Value::Bool(slot != 0)),
        _ => None,
    }
}

/// Copies a host's value onto a script's heap, giving the slot that holds it, with one holder.
///
/// # Errors
///
/// Fails when the value isn't of the crossing's type, is nested deeper than [`DEEPEST`], or the script would pass
/// its memory cap.
pub fn to_slot(value: &Value, crossing: &Crossing, heap: &mut Heap, layouts: &[Layout]) -> Result<u64, CrossingError> {
    let mut made = Vec::new();
    let copied = copy_in(value, crossing, heap, layouts, &mut made);
    match copied {
        Ok(()) => made.pop().map(|(slot, _)| slot).ok_or(HeapError::Broken("a copy that made nothing").into()),
        Err(error) => {
            // What was made before the mistake was found is given back.
            for (slot, reference) in made {
                if reference {
                    heap.release(slot, layouts);
                }
            }
            Err(error)
        }
    }
}

/// What's left to do while copying into a script: a value to copy, or a value to put together from the last slots
/// made, its parts having been copied.
enum Into<'v, 'c> {
    Copy(&'v Value, &'c Crossing, usize),
    List { count: usize, references: bool },
    Dict { pairs: &'v [(Value, Value)], key: &'c Crossing, value: &'c Crossing },
    Maybe { layout: u16 },
    Fields { count: usize, layout: u16, case: bool },
}

/// Takes the last `count` slots made, each with whether it holds a heap value.
fn last_made(made: &mut Vec<(u64, bool)>, count: usize) -> Vec<(u64, bool)> {
    let start = made.len().saturating_sub(count);
    made.drain(start..).collect()
}

/// Copies with a list of what's left to do rather than by recursion, so a deep value never nests on the host's
/// stack (D72). Each slot made is kept in `made` with whether it holds a heap value, until it's put inside another.
#[allow(clippy::too_many_lines)] // one case per kind of value
fn copy_in(
    root_value: &Value,
    root: &Crossing,
    heap: &mut Heap,
    layouts: &[Layout],
    made: &mut Vec<(u64, bool)>,
) -> Result<(), CrossingError> {
    let mut named = None;
    let mut left = vec![Into::Copy(root_value, root, 1)];
    while let Some(job) = left.pop() {
        match job {
            Into::Copy(value, crossing, depth) => {
                if depth > DEEPEST {
                    return Err(CrossingError::TooDeep);
                }
                let crossing = resolved(crossing, root, &mut named)?;
                let deeper = depth + 1;
                match (crossing, value) {
                    (Crossing::Nothing, Value::Nothing) => made.push((0, false)),
                    (Crossing::Whole, Value::Whole(whole)) => made.push((whole.cast_unsigned(), false)),
                    (Crossing::Float, Value::Float(float)) if float.is_finite() => made.push((float.to_bits(), false)),
                    (Crossing::Bool, Value::Bool(flag)) => made.push((u64::from(*flag), false)),
                    (Crossing::Text, Value::Text(text)) => {
                        heap.work = heap.work.saturating_add(text.len() as u64);
                        made.push((heap.new_text(text, text.len())?, true));
                    }
                    (Crossing::HugeWhole, Value::HugeWhole(value)) => {
                        made.push((heap.new_number(Number::Huge(value.clone()))?, true));
                    }
                    (Crossing::Decimal, Value::Decimal(value)) => {
                        made.push((heap.new_number(Number::Decimal(*value))?, true));
                    }
                    (Crossing::HugeDecimal, Value::HugeDecimal(value)) => {
                        made.push((heap.new_number(Number::HugeDecimal(value.clone()))?, true));
                    }
                    (Crossing::List(item), Value::List(values)) => {
                        // Too big a list is turned away before its items are copied.
                        heap.charge(values.len().saturating_mul(SLOT_COST))?;
                        heap.refund(values.len().saturating_mul(SLOT_COST));
                        heap.work = heap.work.saturating_add(values.len() as u64);
                        left.push(Into::List { count: values.len(), references: item.is_reference() });
                        left.extend(values.iter().rev().map(|value| Into::Copy(value, item, deeper)));
                    }
                    (Crossing::Dict(key, value_crossing), Value::Dict(pairs)) => {
                        left.push(Into::Dict { pairs, key, value: value_crossing });
                        for (key_value, value) in pairs.iter().rev() {
                            left.push(Into::Copy(value, value_crossing, deeper));
                            left.push(Into::Copy(key_value, key, deeper));
                        }
                    }
                    (Crossing::Maybe { .. }, Value::Maybe(None)) => made.push((PLAIN_CASE, false)),
                    (Crossing::Maybe { inner, layout }, Value::Maybe(Some(value))) => {
                        left.push(Into::Maybe { layout: *layout });
                        left.push(Into::Copy(value, inner, deeper));
                    }
                    (Crossing::Record { layout, fields, .. }, Value::Record(values))
                        if fields.len() == values.len() =>
                    {
                        left.push(Into::Fields { count: values.len(), layout: *layout, case: false });
                        left.extend(
                            values.iter().zip(fields).rev().map(|(value, (_, field))| Into::Copy(value, field, deeper)),
                        );
                    }
                    (Crossing::OneOf { cases, .. }, Value::Case { case, fields: values }) => {
                        let Some(declared) =
                            cases.get(*case as usize).filter(|declared| declared.fields.len() == values.len())
                        else {
                            return Err(wrong(crossing, value));
                        };
                        if !crossing.is_reference() {
                            made.push((u64::from(*case), false));
                        } else if values.is_empty() {
                            made.push((PLAIN_CASE | u64::from(*case), false));
                        } else {
                            left.push(Into::Fields { count: values.len(), layout: declared.layout, case: true });
                            left.extend(
                                values
                                    .iter()
                                    .zip(&declared.fields)
                                    .rev()
                                    .map(|(value, (_, field))| Into::Copy(value, field, deeper)),
                            );
                        }
                    }
                    _ => return Err(wrong(crossing, value)),
                }
            }
            Into::List { count, references } => {
                let items = last_made(made, count);
                let size = items.len().saturating_mul(SLOT_COST);
                made.push((
                    put_together(heap, layouts, &items, |items| Object::List { items, references }, size)?,
                    true,
                ));
            }
            Into::Maybe { layout } => {
                let fields = last_made(made, 1);
                made.push((
                    put_together(heap, layouts, &fields, |fields| Object::Case { fields, layout }, SLOT_COST)?,
                    true,
                ));
            }
            Into::Fields { count, layout, case } => {
                let fields = last_made(made, count);
                let size = fields.len().saturating_mul(SLOT_COST);
                let object =
                    |fields| if case { Object::Case { fields, layout } } else { Object::Record { fields, layout } };
                made.push((put_together(heap, layouts, &fields, object, size)?, true));
            }
            Into::Dict { pairs, key, value } => {
                let slots = last_made(made, pairs.len().saturating_mul(2));
                let slot = dict_together(pairs, key, value, &slots, heap, layouts)?;
                made.push((slot, true));
            }
        }
    }
    Ok(())
}

/// Allocates a list, a record or a case from slots just made, giving back the ones that hold heap values if it
/// can't.
fn put_together(
    heap: &mut Heap,
    layouts: &[Layout],
    parts: &[(u64, bool)],
    make: impl FnOnce(Vec<u64>) -> Object,
    size: usize,
) -> Result<u64, CrossingError> {
    match heap.allocate(make(parts.iter().map(|&(slot, _)| slot).collect()), size) {
        Ok(slot) => Ok(slot),
        Err(error) => {
            for &(part, reference) in parts {
                if reference {
                    heap.release(part, layouts);
                }
            }
            Err(error.into())
        }
    }
}

/// Puts a dictionary together from its keys' and values' slots, in pairs. The same key twice keeps the later value,
/// as setting a key twice does (D32).
fn dict_together(
    pairs: &[(Value, Value)],
    key_crossing: &Crossing,
    value_crossing: &Crossing,
    slots: &[(u64, bool)],
    heap: &mut Heap,
    layouts: &[Layout],
) -> Result<u64, CrossingError> {
    let text_keys = matches!(key_crossing, Crossing::Text);
    let mut dict = Dict::new(text_keys, value_crossing.is_reference());
    let mut failed = None;
    for ((key, _), slot_pair) in pairs.iter().zip(slots.chunks_exact(2)) {
        let [(key_slot, key_reference), (value_slot, value_reference)] = *slot_pair else { continue };
        let give_back = |heap: &mut Heap| {
            for (slot, reference) in [(key_slot, key_reference), (value_slot, value_reference)] {
                if reference {
                    heap.release(slot, layouts);
                }
            }
        };
        if failed.is_some() {
            // What's left after a mistake is given back.
            give_back(heap);
            continue;
        }
        let probe = match key {
            Value::Text(text) => Probe::of_text(text),
            Value::Whole(whole) => Probe::of_bits(whole.cast_unsigned()),
            Value::Bool(flag) => Probe::of_bits(u64::from(*flag)),
            _ => {
                failed = Some(wrong(key_crossing, key));
                give_back(heap);
                continue;
            }
        };
        let found = match key {
            Value::Text(text) => dict.find(&probe, |stored| heap.text(stored).is_ok_and(|stored| stored == text)).0,
            _ => dict.find(&probe, |_| true).0,
        };
        if let Some(found) = found {
            if let Some(old) = dict.replace_value(found, value_slot)
                && dict.reference_values
            {
                heap.release(old, layouts);
            }
            if key_reference {
                heap.release(key_slot, layouts);
            }
        } else if let Err(error) = heap.charge(dict.growth_for_one_more()) {
            failed = Some(error.into());
            give_back(heap);
        } else {
            dict.add(&probe, key_slot, value_slot);
        }
    }
    heap.work = heap.work.saturating_add(pairs.len() as u64);
    if let Some(error) = failed {
        let kept: Vec<(u64, u64)> = dict.pairs().collect();
        for (key_slot, value_slot) in kept {
            if text_keys {
                heap.release(key_slot, layouts);
            }
            if dict.reference_values {
                heap.release(value_slot, layouts);
            }
        }
        return Err(error);
    }
    // The dictionary's own size was counted as it grew; only the value's fixed cost is added here.
    Ok(heap.allocate(Object::Dict(Box::new(dict)), 0)?)
}

/// What's left to do while copying out of a script: a slot to copy, or a value to put together from the last values
/// made.
enum Out<'c> {
    Copy(u64, &'c Crossing, usize),
    List(usize),
    Dict(usize),
    Maybe,
    Record(usize),
    Case { case: u32, count: usize },
}

/// Takes the last `count` values made.
fn last_values(made: &mut Vec<Value>, count: usize) -> Vec<Value> {
    let start = made.len().saturating_sub(count);
    made.drain(start..).collect()
}

/// Copies a script's value out to the host, with a list of what's left to do rather than by recursion (D72).
///
/// # Errors
///
/// Fails when the value is nested deeper than [`DEEPEST`], or a handle doesn't lead where its type says, which is a
/// bug in Wax.
#[allow(clippy::too_many_lines)] // one case per kind of value
pub fn to_value(slot: u64, root: &Crossing, heap: &Heap, layouts: &[Layout]) -> Result<Value, HeapError> {
    let mut named = None;
    let mut made: Vec<Value> = Vec::new();
    let mut left = vec![Out::Copy(slot, root, 1)];
    while let Some(job) = left.pop() {
        match job {
            Out::Copy(slot, crossing, depth) => {
                if depth > DEEPEST {
                    return Err(HeapError::TooDeep);
                }
                let crossing = resolved(crossing, root, &mut named)?;
                let deeper = depth + 1;
                match crossing {
                    Crossing::Itself { .. } => return Err(HeapError::Broken("a type met again inside itself, twice")),
                    Crossing::Nothing => made.push(Value::Nothing),
                    Crossing::Whole => made.push(Value::Whole(slot.cast_signed())),
                    Crossing::Float => made.push(Value::Float(f64::from_bits(slot))),
                    Crossing::Bool => made.push(Value::Bool(slot != 0)),
                    Crossing::Text => made.push(Value::Text(heap.text(slot)?.to_owned())),
                    Crossing::HugeWhole | Crossing::Decimal | Crossing::HugeDecimal => {
                        made.push(match heap.number(slot)? {
                            NumberRef::Huge(value) => Value::HugeWhole(value.clone()),
                            NumberRef::Decimal(value) => Value::Decimal(value),
                            NumberRef::HugeDecimal(value) => Value::HugeDecimal(value.clone()),
                        });
                    }
                    Crossing::List(item) => match heap.get(slot)? {
                        Object::List { items, .. } => {
                            left.push(Out::List(items.len()));
                            left.extend(items.iter().rev().map(|&item_slot| Out::Copy(item_slot, item, deeper)));
                        }
                        _ => return Err(HeapError::Broken("a handle that should lead to a list")),
                    },
                    Crossing::Dict(key, value) => match heap.get(slot)? {
                        Object::Dict(dict) => {
                            let pairs: Vec<(u64, u64)> = dict.pairs().collect();
                            left.push(Out::Dict(pairs.len()));
                            for (key_slot, value_slot) in pairs.into_iter().rev() {
                                left.push(Out::Copy(value_slot, value, deeper));
                                left.push(Out::Copy(key_slot, key, deeper));
                            }
                        }
                        _ => return Err(HeapError::Broken("a handle that should lead to a dictionary")),
                    },
                    Crossing::Maybe { inner, .. } => {
                        if slot & PLAIN_CASE != 0 {
                            made.push(Value::Maybe(None));
                        } else {
                            match heap.get(slot)? {
                                Object::Case { fields, .. } => {
                                    let inner_slot =
                                        fields.first().copied().ok_or(HeapError::Broken("a `some` with no value"))?;
                                    left.push(Out::Maybe);
                                    left.push(Out::Copy(inner_slot, inner, deeper));
                                }
                                _ => return Err(HeapError::Broken("a handle that should lead to a `Maybe`")),
                            }
                        }
                    }
                    Crossing::Record { fields, .. } => match heap.get(slot)? {
                        Object::Record { fields: slots, .. } => {
                            left.push(Out::Record(slots.len().min(fields.len())));
                            left.extend(
                                slots
                                    .iter()
                                    .zip(fields)
                                    .rev()
                                    .map(|(&field_slot, (_, field))| Out::Copy(field_slot, field, deeper)),
                            );
                        }
                        _ => return Err(HeapError::Broken("a handle that should lead to a record")),
                    },
                    Crossing::OneOf { cases, .. } => {
                        if !crossing.is_reference() {
                            let case = u32::try_from(slot).unwrap_or(u32::MAX);
                            made.push(Value::Case { case, fields: Vec::new() });
                        } else if slot & PLAIN_CASE != 0 {
                            let case = u32::try_from(slot & !PLAIN_CASE).unwrap_or(u32::MAX);
                            made.push(Value::Case { case, fields: Vec::new() });
                        } else {
                            match heap.get(slot)? {
                                Object::Case { fields: slots, layout } => {
                                    let case = layouts.get(usize::from(*layout)).map_or(0, |layout| layout.case);
                                    let declared = cases
                                        .get(case as usize)
                                        .ok_or(HeapError::Broken("a case its type doesn't have"))?;
                                    left.push(Out::Case { case, count: slots.len().min(declared.fields.len()) });
                                    left.extend(
                                        slots
                                            .iter()
                                            .zip(&declared.fields)
                                            .rev()
                                            .map(|(&field_slot, (_, field))| Out::Copy(field_slot, field, deeper)),
                                    );
                                }
                                _ => return Err(HeapError::Broken("a handle that should lead to a case")),
                            }
                        }
                    }
                }
            }
            Out::List(count) => {
                let items = last_values(&mut made, count);
                made.push(Value::List(items));
            }
            Out::Dict(count) => {
                let mut flat = last_values(&mut made, count.saturating_mul(2)).into_iter();
                let mut pairs = Vec::with_capacity(count);
                while let (Some(key), Some(value)) = (flat.next(), flat.next()) {
                    pairs.push((key, value));
                }
                made.push(Value::Dict(pairs));
            }
            Out::Maybe => {
                let inner = made.pop().ok_or(HeapError::Broken("a `some` with no value made"))?;
                made.push(Value::Maybe(Some(Box::new(inner))));
            }
            Out::Record(count) => {
                let fields = last_values(&mut made, count);
                made.push(Value::Record(fields));
            }
            Out::Case { case, count } => {
                let fields = last_values(&mut made, count);
                made.push(Value::Case { case, fields });
            }
        }
    }
    made.pop().ok_or(HeapError::Broken("a copy that made nothing"))
}
