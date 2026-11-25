//! Showing a value as it's written in Seal (D100), and comparing two values by their contents (D95).
//!
//! Both walk a value through the heap without nesting in the host's stack, since a value can be as deep as the script
//! makes it (D72, Building rule 1).

use super::heap::{Heap, HeapError, Object};
use crate::check::types::{Type, TypeId, TypeTable};
use crate::compile::bytecode::{Layout, PLAIN_CASE};
use crate::number::float_to_text;

/// What's left to write, last first.
enum Task {
    Value(u64, TypeId),
    Text(&'static str),
    Owned(String),
}

/// The case a `one_of`'s slot holds, and its values' slots.
fn case_of<'h>(slot: u64, reference: bool, heap: &'h Heap, layouts: &[Layout]) -> Result<(u32, &'h [u64]), HeapError> {
    if !reference {
        return Ok((u32::try_from(slot).unwrap_or(u32::MAX), &[]));
    }
    if slot & PLAIN_CASE != 0 {
        return Ok((u32::try_from(slot & !PLAIN_CASE).unwrap_or(u32::MAX), &[]));
    }
    match heap.get(slot)? {
        Object::Case { fields, layout } => {
            let case = layouts.get(usize::from(*layout)).map_or(0, |layout| layout.case);
            Ok((case, fields))
        }
        _ => Err(HeapError::Broken("a handle that should lead to a case")),
    }
}

/// Text as it's written inside a value: in quotes, with `"`, `\` and `{` made literal, and a line break and a tab
/// written with braces (D63, D100, D118).
pub fn write_quoted(text: &str, out: &mut String) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '\n' => out.push_str("{newln}"),
            '\t' => out.push_str("{tab}"),
            other => out.push(other),
        }
    }
    out.push('"');
}

/// Writes a value as it would be written in a script (D100). The text written may grow to at most `room` bytes,
/// what's left under the script's memory cap; past that, the script is out of memory.
///
/// # Errors
///
/// The script's memory cap, or a broken handle, which is a bug in Seal.
#[allow(clippy::too_many_lines)] // one case per kind of value
pub fn show(
    slot: u64,
    type_id: TypeId,
    types: &TypeTable,
    heap: &Heap,
    layouts: &[Layout],
    room: usize,
    out: &mut String,
) -> Result<(), HeapError> {
    let start = out.len();
    let mut tasks = vec![Task::Value(slot, type_id)];
    while let Some(task) = tasks.pop() {
        if out.len() - start > room {
            return Err(HeapError::OutOfMemory { needed: out.len() - start, in_use: heap.in_use(), cap: heap.cap() });
        }
        let (slot, type_id) = match task {
            Task::Text(text) => {
                out.push_str(text);
                continue;
            }
            Task::Owned(text) => {
                out.push_str(&text);
                continue;
            }
            Task::Value(slot, type_id) => (slot, type_id),
        };
        match types.get(type_id) {
            Type::Whole => out.push_str(&slot.cast_signed().to_string()),
            Type::Float => out.push_str(&float_to_text(f64::from_bits(slot))),
            Type::Bool => out.push_str(if slot == 0 { "false" } else { "true" }),
            Type::Text => write_quoted(heap.text(slot)?, out),
            Type::List { item, .. } | Type::Array { item, .. } => {
                let Object::List { items, .. } = heap.get(slot)? else {
                    return Err(HeapError::Broken("a handle that should lead to a list"));
                };
                out.push('[');
                tasks.push(Task::Text("]"));
                for (index, &item_slot) in items.iter().enumerate().rev() {
                    tasks.push(Task::Value(item_slot, *item));
                    if index > 0 {
                        tasks.push(Task::Text(", "));
                    }
                }
            }
            Type::Dict { key, value } => {
                let Object::Dict(dict) = heap.get(slot)? else {
                    return Err(HeapError::Broken("a handle that should lead to a dictionary"));
                };
                if dict.is_empty() {
                    out.push_str("[:]");
                    continue;
                }
                out.push('[');
                tasks.push(Task::Text("]"));
                let pairs: Vec<(u64, u64)> = dict.pairs().collect();
                for (index, &(key_slot, value_slot)) in pairs.iter().enumerate().rev() {
                    tasks.push(Task::Value(value_slot, *value));
                    tasks.push(Task::Text(": "));
                    tasks.push(Task::Value(key_slot, *key));
                    if index > 0 {
                        tasks.push(Task::Text(", "));
                    }
                }
            }
            Type::Record(record) => {
                let Object::Record { fields: slots, .. } = heap.get(slot)? else {
                    return Err(HeapError::Broken("a handle that should lead to a record"));
                };
                let declared = types.records.get(*record as usize).ok_or(HeapError::Broken("a record type"))?;
                out.push_str(&declared.name);
                out.push('[');
                tasks.push(Task::Text("]"));
                for (index, (&field_slot, (field, field_type))) in slots.iter().zip(&declared.fields).enumerate().rev()
                {
                    tasks.push(Task::Value(field_slot, *field_type));
                    tasks.push(Task::Owned(format!("{field}: ")));
                    if index > 0 {
                        tasks.push(Task::Text(", "));
                    }
                }
            }
            Type::Maybe(inner) => {
                if slot & PLAIN_CASE != 0 {
                    out.push_str("Maybe.none");
                } else {
                    let Object::Case { fields, .. } = heap.get(slot)? else {
                        return Err(HeapError::Broken("a handle that should lead to a `Maybe`"));
                    };
                    let inner_slot = fields.first().copied().ok_or(HeapError::Broken("a `some` with no value"))?;
                    out.push_str("Maybe.some[value: ");
                    tasks.push(Task::Text("]"));
                    tasks.push(Task::Value(inner_slot, *inner));
                }
            }
            Type::OneOf(one_of) => {
                let declared = types.one_ofs.get(*one_of as usize).ok_or(HeapError::Broken("a `one_of` type"))?;
                let (case, slots) = case_of(slot, types.is_reference(type_id), heap, layouts)?;
                let case_type =
                    declared.cases.get(case as usize).ok_or(HeapError::Broken("a case its type doesn't have"))?;
                out.push_str(&declared.name);
                out.push('.');
                out.push_str(&case_type.name);
                if case_type.fields.is_empty() {
                    continue;
                }
                out.push('[');
                tasks.push(Task::Text("]"));
                for (index, (&field_slot, (field, field_type))) in slots.iter().zip(&case_type.fields).enumerate().rev()
                {
                    tasks.push(Task::Value(field_slot, *field_type));
                    tasks.push(Task::Owned(format!("{field}: ")));
                    if index > 0 {
                        tasks.push(Task::Text(", "));
                    }
                }
            }
            Type::Variable(_) | Type::Error | Type::Nothing | Type::Ranged { .. } => {
                return Err(HeapError::Broken("a value of no type to show"));
            }
        }
    }
    Ok(())
}

/// Whether two values of one type hold the same contents (D95): records field by field, lists and arrays item by item
/// in order, `Dict`s by their keys, each with an equal value, whatever order they were added in, and `one_of`s by
/// their case and its values. Two `Float`s are equal as numbers (Unio D184).
///
/// Every pair of values compared counts one unit of `work`.
///
/// # Errors
///
/// A broken handle, which is a bug in Seal.
#[allow(clippy::too_many_lines)] // one case per kind of value
pub fn equal(
    left: u64,
    right: u64,
    type_id: TypeId,
    types: &TypeTable,
    heap: &Heap,
    layouts: &[Layout],
    work: &mut u64,
) -> Result<bool, HeapError> {
    let mut pending = vec![(left, right, type_id)];
    while let Some((left, right, type_id)) = pending.pop() {
        *work = work.saturating_add(1);
        let same = match types.get(type_id) {
            #[allow(clippy::float_cmp)] // `==` on floats is exact (Unio D184)
            Type::Float => f64::from_bits(left) == f64::from_bits(right),
            Type::Whole | Type::Bool => left == right,
            Type::Text => left == right || heap.text(left)? == heap.text(right)?,
            Type::List { item, .. } | Type::Array { item, .. } => {
                let (Object::List { items: left_items, .. }, Object::List { items: right_items, .. }) =
                    (heap.get(left)?, heap.get(right)?)
                else {
                    return Err(HeapError::Broken("a handle that should lead to a list"));
                };
                if left_items.len() == right_items.len() {
                    pending.extend(left_items.iter().zip(right_items).map(|(&a, &b)| (a, b, *item)));
                    true
                } else {
                    false
                }
            }
            Type::Dict { key, value } => {
                let (Object::Dict(left_dict), Object::Dict(right_dict)) = (heap.get(left)?, heap.get(right)?) else {
                    return Err(HeapError::Broken("a handle that should lead to a dictionary"));
                };
                if left_dict.len() == right_dict.len() {
                    let text_keys = matches!(types.get(*key), Type::Text);
                    let mut all_found = true;
                    for (key_slot, value_slot) in left_dict.pairs() {
                        let probe = if text_keys {
                            super::dict::Probe::of_text(heap.text(key_slot)?)
                        } else {
                            super::dict::Probe::of_bits(key_slot)
                        };
                        let key_text = if text_keys { Some(heap.text(key_slot)?) } else { None };
                        let (found, _) = right_dict.find(&probe, |other| match key_text {
                            Some(text) => heap.text(other).is_ok_and(|other| other == text),
                            None => other == key_slot,
                        });
                        if let Some(other_value) = found.and_then(|number| right_dict.value_at(number)) {
                            pending.push((value_slot, other_value, *value));
                        } else {
                            all_found = false;
                            break;
                        }
                    }
                    all_found
                } else {
                    false
                }
            }
            Type::Record(record) => {
                let (Object::Record { fields: left_fields, .. }, Object::Record { fields: right_fields, .. }) =
                    (heap.get(left)?, heap.get(right)?)
                else {
                    return Err(HeapError::Broken("a handle that should lead to a record"));
                };
                let declared = types.records.get(*record as usize).ok_or(HeapError::Broken("a record type"))?;
                pending.extend(
                    left_fields
                        .iter()
                        .zip(right_fields)
                        .zip(&declared.fields)
                        .map(|((&a, &b), (_, field))| (a, b, *field)),
                );
                true
            }
            Type::Maybe(inner) => match (left & PLAIN_CASE != 0, right & PLAIN_CASE != 0) {
                (true, true) => true,
                (false, false) => {
                    let (Object::Case { fields: left_fields, .. }, Object::Case { fields: right_fields, .. }) =
                        (heap.get(left)?, heap.get(right)?)
                    else {
                        return Err(HeapError::Broken("a handle that should lead to a `Maybe`"));
                    };
                    pending.extend(left_fields.iter().zip(right_fields).map(|(&a, &b)| (a, b, *inner)));
                    true
                }
                _ => false,
            },
            Type::OneOf(one_of) => {
                let reference = types.is_reference(type_id);
                let (left_case, left_fields) = case_of(left, reference, heap, layouts)?;
                let (right_case, right_fields) = case_of(right, reference, heap, layouts)?;
                if left_case == right_case {
                    let declared = types.one_ofs.get(*one_of as usize).ok_or(HeapError::Broken("a `one_of` type"))?;
                    let case = declared.cases.get(left_case as usize).ok_or(HeapError::Broken("a case"))?;
                    pending.extend(
                        left_fields
                            .iter()
                            .zip(right_fields)
                            .zip(&case.fields)
                            .map(|((&a, &b), (_, field))| (a, b, *field)),
                    );
                    true
                } else {
                    false
                }
            }
            Type::Variable(_) | Type::Error | Type::Nothing | Type::Ranged { .. } => {
                return Err(HeapError::Broken("a value of no type to compare"));
            }
        };
        if !same {
            return Ok(false);
        }
    }
    Ok(true)
}
