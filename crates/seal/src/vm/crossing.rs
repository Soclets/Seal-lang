//! Values crossing between a script's heap and its host (D15, D48): always as copies.

use super::dict::{Dict, Probe};
use super::heap::{Heap, HeapError, Object, SLOT_COST};
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
        Crossing::List(_) => "a list".to_owned(),
        Crossing::Dict(..) => "a `Dict`".to_owned(),
        Crossing::Maybe { .. } => "a `Maybe`".to_owned(),
        Crossing::Record { name, .. } | Crossing::OneOf { name, .. } => format!("a `{name}`"),
        Crossing::Itself { name } => format!("a {name}"),
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
/// Fails when the value isn't of the crossing's type, or the script would pass its memory cap.
pub fn to_slot(value: &Value, crossing: &Crossing, heap: &mut Heap, layouts: &[Layout]) -> Result<u64, CrossingError> {
    match (crossing, value) {
        (Crossing::Nothing, Value::Nothing) => Ok(0),
        (Crossing::Whole, Value::Whole(whole)) => Ok(whole.cast_unsigned()),
        (Crossing::Float, Value::Float(float)) if float.is_finite() => Ok(float.to_bits()),
        (Crossing::Bool, Value::Bool(flag)) => Ok(u64::from(*flag)),
        (Crossing::Text, Value::Text(text)) => {
            heap.work = heap.work.saturating_add(text.len() as u64);
            Ok(heap.new_text(text, text.len())?)
        }
        (Crossing::List(item), Value::List(values)) => {
            heap.charge(values.len().saturating_mul(SLOT_COST))?;
            heap.refund(values.len().saturating_mul(SLOT_COST));
            let mut items = Vec::new();
            items.reserve_exact(values.len());
            for value in values {
                match to_slot(value, item, heap, layouts) {
                    Ok(slot) => items.push(slot),
                    Err(error) => {
                        release_all(&items, item.is_reference(), heap, layouts);
                        return Err(error);
                    }
                }
            }
            heap.work = heap.work.saturating_add(values.len() as u64);
            let size = items.len().saturating_mul(SLOT_COST);
            let references = item.is_reference();
            match heap.allocate(Object::List { items, references }, size) {
                Ok(slot) => Ok(slot),
                Err(error) => Err(error.into()),
            }
        }
        (Crossing::Dict(key_crossing, value_crossing), Value::Dict(pairs)) => {
            let text_keys = matches!(**key_crossing, Crossing::Text);
            let mut dict = Dict::new(text_keys, value_crossing.is_reference());
            for (key, value) in pairs {
                // The same key twice keeps the later value, as setting a key twice does (D32).
                let probe = match key {
                    Value::Text(text) => Probe::of_text(text),
                    Value::Whole(whole) => Probe::of_bits(whole.cast_unsigned()),
                    Value::Bool(flag) => Probe::of_bits(u64::from(*flag)),
                    _ => return Err(wrong(key_crossing, key)),
                };
                let found = match key {
                    Value::Text(text) => {
                        dict.find(&probe, |stored| heap.text(stored).is_ok_and(|stored| stored == text)).0
                    }
                    _ => dict.find(&probe, |_| true).0,
                };
                let value_slot = to_slot(value, value_crossing, heap, layouts)?;
                if let Some(number) = found {
                    if let Some(old) = dict.replace_value(number, value_slot)
                        && dict.reference_values
                    {
                        heap.release(old, layouts);
                    }
                } else {
                    let key_slot = to_slot(key, key_crossing, heap, layouts)?;
                    heap.charge(dict.growth_for_one_more())?;
                    dict.add(&probe, key_slot, value_slot);
                }
            }
            heap.work = heap.work.saturating_add(pairs.len() as u64);
            // The dictionary's own size was counted as it grew; only the value's fixed cost is added here.
            Ok(heap.allocate(Object::Dict(Box::new(dict)), 0)?)
        }
        (Crossing::Maybe { .. }, Value::Maybe(None)) => Ok(PLAIN_CASE),
        (Crossing::Maybe { inner, layout }, Value::Maybe(Some(value))) => {
            let slot = to_slot(value, inner, heap, layouts)?;
            Ok(heap.allocate(Object::Case { fields: vec![slot], layout: *layout }, SLOT_COST)?)
        }
        (Crossing::Record { layout, fields, .. }, Value::Record(values)) if fields.len() == values.len() => {
            let mut slots = Vec::new();
            slots.reserve_exact(values.len());
            for (value, (_, field)) in values.iter().zip(fields) {
                slots.push(to_slot(value, field, heap, layouts)?);
            }
            let size = slots.len().saturating_mul(SLOT_COST);
            Ok(heap.allocate(Object::Record { fields: slots, layout: *layout }, size)?)
        }
        (Crossing::OneOf { cases, .. }, Value::Case { case, fields: values }) => {
            let Some(declared) = cases.get(*case as usize).filter(|declared| declared.fields.len() == values.len())
            else {
                return Err(wrong(crossing, value));
            };
            if !crossing.is_reference() {
                return Ok(u64::from(*case));
            }
            if values.is_empty() {
                return Ok(PLAIN_CASE | u64::from(*case));
            }
            let mut slots = Vec::new();
            slots.reserve_exact(values.len());
            for (value, (_, field)) in values.iter().zip(&declared.fields) {
                slots.push(to_slot(value, field, heap, layouts)?);
            }
            let size = slots.len().saturating_mul(SLOT_COST);
            Ok(heap.allocate(Object::Case { fields: slots, layout: declared.layout }, size)?)
        }
        _ => Err(wrong(crossing, value)),
    }
}

fn release_all(slots: &[u64], references: bool, heap: &mut Heap, layouts: &[Layout]) {
    if references {
        for &slot in slots {
            heap.release(slot, layouts);
        }
    }
}

/// Copies a script's value out to the host.
///
/// # Errors
///
/// Fails when a handle doesn't lead where its type says, which is a bug in Seal.
pub fn to_value(slot: u64, crossing: &Crossing, heap: &Heap, layouts: &[Layout]) -> Result<Value, HeapError> {
    Ok(match crossing {
        // A call whose types hold themselves is turned away before anything crosses (D72).
        Crossing::Itself { .. } => return Err(HeapError::Broken("a value of a type that holds itself, at a door")),
        Crossing::Nothing => Value::Nothing,
        Crossing::Whole => Value::Whole(slot.cast_signed()),
        Crossing::Float => Value::Float(f64::from_bits(slot)),
        Crossing::Bool => Value::Bool(slot != 0),
        Crossing::Text => Value::Text(heap.text(slot)?.to_owned()),
        Crossing::List(item) => match heap.get(slot)? {
            Object::List { items, .. } => {
                let mut values = Vec::with_capacity(items.len());
                for &item_slot in items {
                    values.push(to_value(item_slot, item, heap, layouts)?);
                }
                Value::List(values)
            }
            _ => return Err(HeapError::Broken("a handle that should lead to a list")),
        },
        Crossing::Dict(key, value) => match heap.get(slot)? {
            Object::Dict(dict) => {
                let mut pairs = Vec::with_capacity(dict.len());
                for (key_slot, value_slot) in dict.pairs() {
                    pairs.push((to_value(key_slot, key, heap, layouts)?, to_value(value_slot, value, heap, layouts)?));
                }
                Value::Dict(pairs)
            }
            _ => return Err(HeapError::Broken("a handle that should lead to a dictionary")),
        },
        Crossing::Maybe { inner, .. } => {
            if slot & PLAIN_CASE != 0 {
                Value::Maybe(None)
            } else {
                match heap.get(slot)? {
                    Object::Case { fields, .. } => {
                        let inner_slot = fields.first().copied().ok_or(HeapError::Broken("a `some` with no value"))?;
                        Value::Maybe(Some(Box::new(to_value(inner_slot, inner, heap, layouts)?)))
                    }
                    _ => return Err(HeapError::Broken("a handle that should lead to a `Maybe`")),
                }
            }
        }
        Crossing::Record { fields, .. } => match heap.get(slot)? {
            Object::Record { fields: slots, .. } => {
                let mut values = Vec::with_capacity(slots.len());
                for (&field_slot, (_, field)) in slots.iter().zip(fields) {
                    values.push(to_value(field_slot, field, heap, layouts)?);
                }
                Value::Record(values)
            }
            _ => return Err(HeapError::Broken("a handle that should lead to a record")),
        },
        Crossing::OneOf { cases, .. } => {
            if !crossing.is_reference() {
                return Ok(Value::Case { case: u32::try_from(slot).unwrap_or(u32::MAX), fields: Vec::new() });
            }
            if slot & PLAIN_CASE != 0 {
                return Ok(Value::Case {
                    case: u32::try_from(slot & !PLAIN_CASE).unwrap_or(u32::MAX),
                    fields: Vec::new(),
                });
            }
            match heap.get(slot)? {
                Object::Case { fields: slots, layout } => {
                    let case = layouts.get(usize::from(*layout)).map_or(0, |layout| layout.case);
                    let declared = cases.get(case as usize).ok_or(HeapError::Broken("a case its type doesn't have"))?;
                    let mut values = Vec::with_capacity(slots.len());
                    for (&field_slot, (_, field)) in slots.iter().zip(&declared.fields) {
                        values.push(to_value(field_slot, field, heap, layouts)?);
                    }
                    Value::Case { case, fields: values }
                }
                _ => return Err(HeapError::Broken("a handle that should lead to a case")),
            }
        }
    })
}
