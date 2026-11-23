//! A script's heap: its big values, each with its holders counted (D15).
//!
//! Values live in one list the script owns, and a slot holds a value's place in it, its handle. There are no
//! pointers and no garbage collector: a value is freed the moment its last holder ends. The heap counts its own
//! size, in Seal's own sizes, the same on every machine (D50), against the script's memory cap (D14).

use super::dict::Dict;
use crate::compile::bytecode::{Layout, PLAIN_CASE};

/// What every heap value costs beyond its contents, in bytes of the memory cap.
pub const VALUE_COST: usize = 48;
/// What one slot costs: an item of a list, a field, a register.
pub const SLOT_COST: usize = 8;

/// Why the heap couldn't do what was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeapError {
    /// The script would hold more than its cap (D14): how much was needed, how much was in use, and the cap.
    OutOfMemory { needed: usize, in_use: usize, cap: usize },
    /// A handle didn't lead to the kind of value its instruction works on: a bug in Seal.
    Broken(&'static str),
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
    cap: usize,
    /// Bulk work done since it was last asked about, in items and bytes, to be counted as steps (D14).
    pub work: u64,
}

impl Heap {
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Heap { cap, ..Heap::default() }
    }

    /// How many bytes of its cap the script holds.
    #[must_use]
    pub fn in_use(&self) -> usize {
        self.in_use
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
            *slot = entry;
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
        self.charge(VALUE_COST.saturating_add(capacity))?;
        let mut owned = String::new();
        owned.reserve_exact(capacity);
        owned.push_str(text);
        self.refund(VALUE_COST.saturating_add(capacity));
        self.allocate(Object::Text(owned), capacity)
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
    /// Fails when the handle leads nowhere, which is a bug in Seal.
    #[inline]
    pub fn get(&self, handle: u64) -> Result<&Object, HeapError> {
        self.entry(handle).map(|entry| &entry.object).ok_or(HeapError::Broken("a handle that leads nowhere"))
    }

    /// The value a handle leads to, to be changed.
    ///
    /// # Errors
    ///
    /// Fails when the handle leads nowhere, which is a bug in Seal.
    #[inline]
    pub fn get_mut(&mut self, handle: u64) -> Result<&mut Object, HeapError> {
        self.entry_mut(handle).map(|entry| &mut entry.object).ok_or(HeapError::Broken("a handle that leads nowhere"))
    }

    /// The text a handle leads to.
    ///
    /// # Errors
    ///
    /// Fails when the handle doesn't lead to text, which is a bug in Seal.
    #[inline]
    pub fn text(&self, handle: u64) -> Result<&str, HeapError> {
        match self.get(handle)? {
            Object::Text(text) => Ok(text),
            _ => Err(HeapError::Broken("a handle that should lead to text")),
        }
    }

    /// Whether a value has exactly one holder, so that it can be changed in place (D15).
    #[must_use]
    #[inline]
    pub fn has_one_holder(&self, handle: u64) -> bool {
        self.entry(handle).is_some_and(|entry| entry.holders == 1)
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
                Object::Text(text) => text.capacity(),
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
    /// Fails when the handle leads nowhere, which is a bug in Seal.
    pub fn take(&mut self, handle: u64) -> Result<Object, HeapError> {
        let entry = self.entry_mut(handle).ok_or(HeapError::Broken("a handle that leads nowhere"))?;
        Ok(std::mem::replace(&mut entry.object, Object::Taken))
    }

    pub fn put_back(&mut self, handle: u64, object: Object) {
        if let Some(entry) = self.entry_mut(handle) {
            entry.object = object;
        }
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
