//! A script's heap: its big values, each with its holders counted (D15).
//!
//! Values live in one list the script owns, and a slot holds a value's place in it, its handle. There are no
//! pointers and no garbage collector: a value is freed the moment its last holder ends. The heap counts its own
//! size, in Wax's own sizes, the same on every machine (D50), against the script's memory cap (D14).

use super::dict::Dict;
use crate::compile::bytecode::{Layout, PLAIN_CASE};
use crate::decimal::PackedDecimal;
pub use crate::decimal::{Number, NumberRef};

/// What every heap value costs beyond its contents, in bytes of the memory cap.
pub const VALUE_COST: usize = 48;
/// The longest text, in bytes of room, whose room is kept for reuse when it's freed, and how many are kept.
const POOLED_TEXT: usize = 128;
const SPARE_TEXTS: usize = 8;

/// What one slot costs: an item of a list, a field, a register.
pub const SLOT_COST: usize = 8;

/// Why the heap couldn't do what was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeapError {
    /// The script would hold more than its cap (D14): how much was needed, how much was in use, and the cap.
    OutOfMemory { needed: usize, in_use: usize, cap: usize },
    /// A handle didn't lead to the kind of value its instruction works on: a bug in Wax.
    Broken(&'static str),
    /// A value going to the host is nested deeper than a copy may go (D72).
    TooDeep,
}

#[derive(Debug)]
pub enum Object {
    /// A freed place, waiting to be used again.
    Free,
    Text(String),
    /// A list or an array. `references` says whether its items are heap values themselves.
    List {
        items: Vec<u64>,
        references: bool,
    },
    Record {
        fields: Vec<u64>,
        layout: u16,
    },
    /// A case of a `one_of` that carries values (D35).
    Case {
        fields: Vec<u64>,
        layout: u16,
    },
    Dict(Box<Dict>),
    Number(Box<Number>),
    /// A `Decimal`, held in the entry itself (D116).
    Decimal(PackedDecimal),
    /// A value taken out for a moment while it's being changed.
    Taken,
}

#[derive(Debug)]
struct Entry {
    holders: u32,
    object: Object,
}

#[derive(Debug, Default)]
pub struct Heap {
    entries: Vec<Entry>,
    free: Vec<u32>,
    in_use: usize,
    most: usize,
    cap: usize,
    /// Bulk work done since it was last asked about, in items and bytes, to be counted as steps (D14).
    pub work: u64,
    /// How much work the steps left can pay for, set before an instruction that works out big numbers (D14).
    pub work_allowance: u64,
    /// Freed short texts, emptied, whose room the next short texts reuse.
    spare_texts: Vec<String>,
    /// Freed numbers' boxes, which the next numbers are made in, so working with numbers asks the system for less.
    #[allow(clippy::vec_box)] // the boxes themselves are what's kept, to be reused
    spare_numbers: Vec<Box<Number>>,
}

/// How many freed numbers' boxes are kept for reuse.
const SPARE_NUMBERS: usize = 64;

impl Heap {
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Heap { cap, work_allowance: u64::MAX, ..Heap::default() }
    }

    /// How many bytes of its cap the script holds.
    #[must_use]
    pub fn in_use(&self) -> usize {
        self.in_use
    }

    /// The script's memory cap (D14).
    #[must_use]
    pub fn cap(&self) -> usize {
        self.cap
    }

    /// How many bytes the script may still take before its cap.
    #[must_use]
    pub fn room_left(&self) -> usize {
        self.cap.saturating_sub(self.in_use)
    }

    /// The most the script has held at once.
    #[must_use]
    pub fn most(&self) -> usize {
        self.most
    }

    /// Counts `bytes` more against the cap.
    ///
    /// # Errors
    ///
    /// Fails when the script would hold more than its cap (D14).
    pub fn charge(&mut self, bytes: usize) -> Result<(), HeapError> {
        let total = self.in_use.saturating_add(bytes);
        if total > self.cap {
            return Err(HeapError::OutOfMemory { needed: bytes, in_use: self.in_use, cap: self.cap });
        }
        self.in_use = total;
        if total > self.most {
            self.most = total;
        }
        Ok(())
    }

    pub fn refund(&mut self, bytes: usize) {
        self.in_use = self.in_use.saturating_sub(bytes);
    }

    /// Puts a value on the heap with one holder, giving its handle. `size` is what its contents cost.
    ///
    /// # Errors
    ///
    /// Fails when the script would hold more than its cap (D14).
    pub fn allocate(&mut self, object: Object, size: usize) -> Result<u64, HeapError> {
        self.charge(VALUE_COST.saturating_add(size))?;
        let entry = Entry { holders: 1, object };
        if let Some(index) = self.free.pop()
            && let Some(slot) = self.entries.get_mut(index as usize)
        {
            // A freed place holds nothing, so what was there is set aside without being looked through.
            std::mem::forget(std::mem::replace(slot, entry));
            return Ok(u64::from(index));
        }
        let index = self.entries.len();
        if index >= u32::MAX as usize {
            return Err(HeapError::OutOfMemory { needed: VALUE_COST, in_use: self.in_use, cap: self.cap });
        }
        self.entries.push(entry);
        Ok(index as u64)
    }

    /// A new text with room for `capacity` bytes.
    ///
    /// # Errors
    ///
    /// Fails when the script would hold more than its cap (D14).
    pub fn new_text(&mut self, text: &str, capacity: usize) -> Result<u64, HeapError> {
        let capacity = capacity.max(text.len());
        // A short text reuses the room of one freed earlier, when there is one.
        if capacity <= POOLED_TEXT
            && let Some(mut owned) = self.spare_texts.pop()
        {
            if owned.capacity() >= capacity {
                owned.push_str(text);
                let room = owned.capacity();
                return match self.allocate(Object::Text(owned), room) {
                    Ok(handle) => Ok(handle),
                    Err(error) => Err(error),
                };
            }
            self.spare_texts.push(owned);
        }
        self.charge(VALUE_COST.saturating_add(capacity))?;
        let mut owned = String::new();
        owned.reserve_exact(capacity);
        owned.push_str(text);
        self.refund(VALUE_COST.saturating_add(capacity));
        self.allocate(Object::Text(owned), capacity)
    }

    /// An empty text with room for `room` bytes, to be built outside the heap and then put on it. A short one
    /// reuses the room of a text freed earlier, when there is one.
    #[must_use]
    pub fn spare_text(&mut self, room: usize) -> String {
        if room <= POOLED_TEXT
            && let Some(spare) = self.spare_texts.pop()
        {
            if spare.capacity() >= room {
                return spare;
            }
            self.spare_texts.push(spare);
        }
        let mut text = String::new();
        text.reserve_exact(room);
        text
    }

    #[inline]
    fn entry(&self, handle: u64) -> Option<&Entry> {
        self.entries.get(usize::try_from(handle).ok()?)
    }

    #[inline]
    fn entry_mut(&mut self, handle: u64) -> Option<&mut Entry> {
        self.entries.get_mut(usize::try_from(handle).ok()?)
    }

    /// The value a handle leads to.
    ///
    /// # Errors
    ///
    /// Fails when the handle leads nowhere, which is a bug in Wax.
    #[inline]
    pub fn get(&self, handle: u64) -> Result<&Object, HeapError> {
        self.entry(handle).map(|entry| &entry.object).ok_or(HeapError::Broken("a handle that leads nowhere"))
    }

    /// The value a handle leads to, to be changed.
    ///
    /// # Errors
    ///
    /// Fails when the handle leads nowhere, which is a bug in Wax.
    #[inline]
    pub fn get_mut(&mut self, handle: u64) -> Result<&mut Object, HeapError> {
        self.entry_mut(handle).map(|entry| &mut entry.object).ok_or(HeapError::Broken("a handle that leads nowhere"))
    }

    /// The text a handle leads to.
    ///
    /// # Errors
    ///
    /// Fails when the handle doesn't lead to text, which is a bug in Wax.
    #[inline]
    pub fn text(&self, handle: u64) -> Result<&str, HeapError> {
        match self.get(handle)? {
            Object::Text(text) => Ok(text),
            _ => Err(HeapError::Broken("a handle that should lead to text")),
        }
    }

    /// Puts a number on the heap (D115, D116).
    ///
    /// # Errors
    ///
    /// Fails when the script would hold more than its cap (D14).
    pub fn new_number(&mut self, number: Number) -> Result<u64, HeapError> {
        let size = number.size();
        match number {
            Number::Decimal(decimal) => self.allocate(Object::Decimal(decimal.into()), size),
            other => {
                let boxed = match self.spare_numbers.pop() {
                    Some(mut spare) => {
                        *spare = other;
                        spare
                    }
                    None => Box::new(other),
                };
                self.allocate(Object::Number(boxed), size)
            }
        }
    }

    /// trace: D15, Building rule 5. Changes a `HugeWhole` or `HugeDecimal` held by nothing else where it is, with
    /// the number in `other` to read when there's one, counting what it costs afterwards. `change` says whether it
    /// could. Nothing when it couldn't, the target is shared or isn't one of those, or `other` is the target itself.
    ///
    /// # Errors
    ///
    /// Fails when the number grew past the script's cap (D14).
    pub fn change_number(
        &mut self,
        target: u64,
        other: Option<u64>,
        change: impl FnOnce(&mut Number, Option<&Number>) -> bool,
    ) -> Option<Result<(), HeapError>> {
        let target_index = usize::try_from(target).ok()?;
        let other_index = match other {
            Some(other) => Some(usize::try_from(other).ok().filter(|&index| index != target_index)?),
            None => None,
        };
        let (entry, other_entry) = match other_index {
            None => (self.entries.get_mut(target_index)?, None),
            Some(other_index) if other_index < target_index => {
                let (low, high) = self.entries.split_at_mut(target_index);
                (high.first_mut()?, Some(low.get(other_index)?))
            }
            Some(other_index) => {
                let (low, high) = self.entries.split_at_mut(other_index);
                (low.get_mut(target_index)?, Some(high.first()?))
            }
        };
        if entry.holders != 1 {
            return None;
        }
        let Object::Number(number) = &mut entry.object else { return None };
        let other_number = match other_entry.map(|entry| &entry.object) {
            None => None,
            Some(Object::Number(other)) => Some(other.as_ref()),
            Some(_) => return None,
        };
        let before = number.size();
        if !change(number, other_number) {
            return None;
        }
        let after = number.size();
        Some(if after >= before {
            self.charge(after - before)
        } else {
            self.refund(before - after);
            Ok(())
        })
    }

    /// trace: D15, Building rule 5. Puts a new number in place of the one a handle leads to, when nothing else holds
    /// it, keeping its place and box; gives the number back when it can't.
    ///
    /// # Errors
    ///
    /// Fails when the new number takes the script past its cap (D14).
    pub fn replace_number(&mut self, handle: u64, number: Number) -> Result<Option<Number>, HeapError> {
        let Some(entry) = self.entry_mut(handle) else { return Ok(Some(number)) };
        if entry.holders != 1 {
            return Ok(Some(number));
        }
        let new_size = number.size();
        let old_size = match (&mut entry.object, number) {
            (Object::Decimal(packed), Number::Decimal(decimal)) => {
                *packed = decimal.into();
                new_size
            }
            (Object::Number(boxed), number) if !matches!(number, Number::Decimal(_)) => {
                let old_size = boxed.size();
                **boxed = number;
                old_size
            }
            (_, number) => return Ok(Some(number)),
        };
        if new_size >= old_size {
            self.charge(new_size - old_size)?;
        } else {
            self.refund(old_size - new_size);
        }
        Ok(None)
    }

    /// The number a handle leads to.
    ///
    /// # Errors
    ///
    /// Fails when the handle doesn't lead to a number, which is a bug in Wax.
    #[inline]
    pub fn number(&self, handle: u64) -> Result<NumberRef<'_>, HeapError> {
        match self.get(handle)? {
            Object::Decimal(decimal) => Ok(NumberRef::Decimal((*decimal).into())),
            Object::Number(number) => Ok(match number.as_ref() {
                Number::Huge(value) => NumberRef::Huge(value),
                Number::Decimal(value) => NumberRef::Decimal(*value),
                Number::HugeDecimal(value) => NumberRef::HugeDecimal(value),
            }),
            _ => Err(HeapError::Broken("a handle that should lead to a number")),
        }
    }

    /// Whether a value has exactly one holder, so that it can be changed in place (D15).
    #[must_use]
    #[inline]
    pub fn has_one_holder(&self, handle: u64) -> bool {
        self.entry(handle).is_some_and(|entry| entry.holders == 1)
    }

    /// The fields of the record at `place` in a list, to be written in place: given only when the list has one
    /// holder and it alone holds the record, so that nothing else can see the change (D15).
    #[inline]
    pub fn item_fields_to_write(&mut self, list: u64, place: usize) -> Option<&mut [u64]> {
        let entry = self.entry(list)?;
        let Object::List { items, .. } = &entry.object else { return None };
        if entry.holders != 1 {
            return None;
        }
        let item = *items.get(place)?;
        let entry = self.entry_mut(item)?;
        match &mut entry.object {
            Object::Record { fields, .. } if entry.holders == 1 => Some(fields),
            _ => None,
        }
    }

    /// Counts one more holder of the value in a slot. A case with no values lives in its slot, with no count.
    #[inline]
    pub fn retain(&mut self, slot: u64) {
        if slot & PLAIN_CASE != 0 {
            return;
        }
        if let Some(entry) = self.entry_mut(slot) {
            entry.holders = entry.holders.saturating_add(1);
        }
    }

    /// Counts one holder fewer of a text, freeing it when that was the last: [`Heap::release`], for a slot known
    /// to hold a text, which has nothing inside it to free in turn.
    #[inline]
    pub fn release_text(&mut self, slot: u64) {
        let Some(entry) = self.entry_mut(slot) else { return };
        if entry.holders > 1 {
            entry.holders -= 1;
            return;
        }
        let Object::Text(text) = &mut entry.object else { return };
        let mut text = std::mem::take(text);
        entry.holders = 0;
        entry.object = Object::Free;
        let room = text.capacity();
        if room <= POOLED_TEXT && self.spare_texts.len() < SPARE_TEXTS {
            text.clear();
            self.spare_texts.push(text);
        }
        self.refund(VALUE_COST.saturating_add(room));
        if let Ok(index) = u32::try_from(slot) {
            self.free.push(index);
        }
    }

    /// Counts one holder fewer of the value in a slot, freeing it, and what only it held, when that was the last.
    pub fn release(&mut self, slot: u64, layouts: &[Layout]) {
        if slot & PLAIN_CASE != 0 {
            return;
        }
        // The common case first: a value with other holders, or with nothing inside to free.
        let Some(entry) = self.entry_mut(slot) else { return };
        if entry.holders > 1 {
            entry.holders -= 1;
            return;
        }
        // Freed with a list of what's left to do, never by nesting, so a deep value can't overrun the host's stack.
        let mut waiting: Vec<u64> = Vec::new();
        let mut first = Some(slot);
        while let Some(slot) = first.take().or_else(|| waiting.pop()) {
            if slot & PLAIN_CASE != 0 {
                continue;
            }
            let Some(entry) = self.entry_mut(slot) else { continue };
            if entry.holders > 1 {
                entry.holders -= 1;
                continue;
            }
            entry.holders = 0;
            let object = std::mem::replace(&mut entry.object, Object::Free);
            let size = match object {
                Object::Free | Object::Taken => continue,
                Object::Text(mut text) => {
                    let room = text.capacity();
                    // A short text's room is kept for the next short text, so making and freeing small texts in a
                    // loop doesn't go to the system for memory each time. What's kept is small and bounded.
                    if room <= POOLED_TEXT && self.spare_texts.len() < SPARE_TEXTS {
                        text.clear();
                        self.spare_texts.push(text);
                    }
                    room
                }
                Object::List { items, references } => {
                    if references {
                        waiting.extend_from_slice(&items);
                    }
                    self.work = self.work.saturating_add(items.len() as u64);
                    items.capacity().saturating_mul(SLOT_COST)
                }
                Object::Record { fields, layout } | Object::Case { fields, layout } => {
                    if let Some(layout) = layouts.get(usize::from(layout)) {
                        for (field, _) in fields.iter().zip(&layout.references).filter(|(_, reference)| **reference) {
                            waiting.push(*field);
                        }
                    }
                    fields.len().saturating_mul(SLOT_COST)
                }
                Object::Dict(dict) => {
                    dict.push_references(&mut waiting);
                    self.work = self.work.saturating_add(dict.len() as u64);
                    dict.size()
                }
                Object::Number(number) => {
                    let size = number.size();
                    if self.spare_numbers.len() < SPARE_NUMBERS {
                        self.spare_numbers.push(number);
                    }
                    size
                }
                Object::Decimal(_) => 16,
            };
            self.refund(VALUE_COST.saturating_add(size));
            if let Ok(index) = u32::try_from(slot) {
                self.free.push(index);
            }
        }
    }

    /// Makes sure the value in a slot has one holder, copying it first when it's shared (copy-on-write, D15).
    /// Gives the handle to change through, which is a new one when a copy was made.
    ///
    /// # Errors
    ///
    /// Fails when the copy would pass the memory cap, or the handle leads nowhere.
    pub fn make_own(&mut self, handle: u64, layouts: &[Layout]) -> Result<u64, HeapError> {
        let entry = self.entry(handle).ok_or(HeapError::Broken("a handle that leads nowhere"))?;
        if entry.holders == 1 {
            return Ok(handle);
        }
        let (copy, size, inside, work): (Object, usize, Vec<u64>, usize) = match &entry.object {
            Object::Text(text) => {
                let mut copy = String::new();
                copy.reserve_exact(text.len());
                copy.push_str(text);
                (Object::Text(copy), text.len(), Vec::new(), text.len())
            }
            Object::List { items, references } => {
                let mut copy = Vec::new();
                copy.reserve_exact(items.len());
                copy.extend_from_slice(items);
                let inside = if *references { items.clone() } else { Vec::new() };
                (
                    Object::List { items: copy, references: *references },
                    items.len().saturating_mul(SLOT_COST),
                    inside,
                    items.len(),
                )
            }
            Object::Record { fields, layout } | Object::Case { fields, layout } => {
                let inside = layouts
                    .get(usize::from(*layout))
                    .map(|layout| {
                        fields
                            .iter()
                            .zip(&layout.references)
                            .filter(|(_, reference)| **reference)
                            .map(|(field, _)| *field)
                            .collect()
                    })
                    .unwrap_or_default();
                let copy = if matches!(entry.object, Object::Record { .. }) {
                    Object::Record { fields: fields.clone(), layout: *layout }
                } else {
                    Object::Case { fields: fields.clone(), layout: *layout }
                };
                (copy, fields.len().saturating_mul(SLOT_COST), inside, fields.len())
            }
            Object::Dict(dict) => {
                let copy = dict.copy();
                let mut inside = Vec::new();
                dict.push_references(&mut inside);
                let size = copy.size();
                (Object::Dict(Box::new(copy)), size, inside, dict.len())
            }
            Object::Number(number) => (Object::Number(number.clone()), number.size(), Vec::new(), 1),
            Object::Decimal(decimal) => (Object::Decimal(*decimal), 16, Vec::new(), 1),
            Object::Free | Object::Taken => return Err(HeapError::Broken("a value that isn't there to copy")),
        };
        self.work = self.work.saturating_add(work as u64);
        let new = self.allocate(copy, size)?;
        // The copy holds what the original holds, and the slot gives up its hold on the original.
        for slot in inside {
            self.retain(slot);
        }
        if let Some(entry) = self.entry_mut(handle) {
            entry.holders = entry.holders.saturating_sub(1);
        }
        Ok(new)
    }

    /// Takes a value out of its place for a moment, to change it while reading other values.
    ///
    /// # Errors
    ///
    /// Fails when the handle leads nowhere, which is a bug in Wax.
    pub fn take(&mut self, handle: u64) -> Result<Object, HeapError> {
        let entry = self.entry_mut(handle).ok_or(HeapError::Broken("a handle that leads nowhere"))?;
        Ok(std::mem::replace(&mut entry.object, Object::Taken))
    }

    pub fn put_back(&mut self, handle: u64, object: Object) {
        if let Some(entry) = self.entry_mut(handle) {
            entry.object = object;
        }
    }

    /// Checks every value's count of holders against who really holds it: `roots` are the slots outside the heap
    /// that hold a value, and each value inside another is held by it. For tests and the fuzzer; a difference is a
    /// bug in Wax.
    ///
    /// # Errors
    ///
    /// Says which value's count is off, and by how much.
    pub fn audit(&self, roots: &[u64], layouts: &[Layout]) -> Result<(), String> {
        let mut expected = vec![0u32; self.entries.len()];
        let hold = |slot: u64, expected: &mut Vec<u32>| -> Result<(), String> {
            if slot & PLAIN_CASE != 0 {
                return Ok(());
            }
            match usize::try_from(slot).ok().and_then(|index| expected.get_mut(index)) {
                Some(count) => {
                    *count += 1;
                    Ok(())
                }
                None => Err(format!("a slot holds {slot}, which is no value at all")),
            }
        };
        for &root in roots {
            hold(root, &mut expected)?;
        }
        for entry in &self.entries {
            let mut inside = Vec::new();
            match &entry.object {
                Object::List { items, references: true } => inside.extend_from_slice(items),
                Object::Record { fields, layout } | Object::Case { fields, layout } => {
                    if let Some(layout) = layouts.get(usize::from(*layout)) {
                        inside.extend(
                            fields
                                .iter()
                                .zip(&layout.references)
                                .filter(|(_, reference)| **reference)
                                .map(|(field, _)| *field),
                        );
                    }
                }
                Object::Dict(dict) => dict.push_references(&mut inside),
                _ => {}
            }
            for slot in inside {
                hold(slot, &mut expected)?;
            }
        }
        for (index, (entry, expected)) in self.entries.iter().zip(&expected).enumerate() {
            let live = !matches!(entry.object, Object::Free);
            if live && entry.holders != *expected {
                return Err(format!(
                    "value {index} counts {} holders, but {expected} hold it: {:?}",
                    entry.holders, entry.object
                ));
            }
            if !live && *expected != 0 {
                return Err(format!("value {index} was freed, but {expected} still hold it"));
            }
        }
        Ok(())
    }

    /// Makes room for one more item in a list of `length` items with room for `capacity`, counting the growth.
    /// Gives how many more to reserve, when the list must grow.
    ///
    /// # Errors
    ///
    /// Fails when the growth would pass the memory cap.
    pub fn grow(&mut self, length: usize, capacity: usize, unit: usize) -> Result<Option<usize>, HeapError> {
        if length < capacity {
            return Ok(None);
        }
        // Doubling, so that adding many items one by one costs a fixed amount each on average.
        let new_capacity = capacity.saturating_mul(2).max(4);
        self.charge((new_capacity - capacity).saturating_mul(unit))?;
        Ok(Some(new_capacity - length))
    }
}
