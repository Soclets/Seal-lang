//! A table from names to what they stand for, for the checker's scopes and lists of names (D140).
//!
//! While it holds few names it's a short list, looked down in order: for a handful of short names that's quicker than
//! hashing each one. Past that it becomes a hash table, so a script with very many names can't make finding one slow.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

/// How many names are kept in the short list before it becomes a hash table.
const FEW: usize = 16;

/// A table keyed by any kind of name that reads as text: borrowed from the script, or made, as a family member's
/// `group.member`.
#[derive(Clone, Debug)]
pub(crate) struct NameTable<K, V> {
    few: Vec<(K, V)>,
    many: HashMap<K, V>,
}

/// The usual table, its names borrowed from the script.
pub(crate) type Names<'a, V> = NameTable<&'a str, V>;

impl<K, V> Default for NameTable<K, V> {
    fn default() -> Self {
        NameTable { few: Vec::new(), many: HashMap::new() }
    }
}

impl<K: Borrow<str> + Eq + Hash, V> NameTable<K, V> {
    pub(crate) fn get(&self, name: &str) -> Option<&V> {
        if self.many.is_empty() {
            self.few.iter().find(|(known, _)| known.borrow() == name).map(|(_, value)| value)
        } else {
            self.many.get(name)
        }
    }

    /// Every name, in no particular order.
    pub(crate) fn keys(&self) -> impl Iterator<Item = &K> {
        self.few.iter().map(|(name, _)| name).chain(self.many.keys())
    }

    pub(crate) fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Gives a name a value, giving back the one it had, when it had one.
    pub(crate) fn insert(&mut self, name: K, value: V) -> Option<V> {
        if self.many.is_empty() {
            if let Some((_, slot)) = self.few.iter_mut().find(|(known, _)| known.borrow() == name.borrow()) {
                return Some(std::mem::replace(slot, value));
            }
            if self.few.len() < FEW {
                if self.few.capacity() == 0 {
                    // Room for a usual scope's names from the start, rather than growing to it.
                    self.few.reserve_exact(FEW / 2);
                }
                self.few.push((name, value));
                return None;
            }
            self.many.extend(self.few.drain(..));
        }
        self.many.insert(name, value)
    }

    /// Every name's value, in no particular order.
    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        self.few.iter().map(|(_, value)| value).chain(self.many.values())
    }

    pub(crate) fn clear(&mut self) {
        self.few.clear();
        self.many.clear();
    }

    /// How much room it holds, for deciding whether it's worth keeping.
    pub(crate) fn capacity(&self) -> usize {
        self.few.capacity().max(self.many.capacity())
    }
}

impl<K: Borrow<str> + Eq + Hash> NameTable<K, ()> {
    /// For a set of names: whether it holds one.
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.contains_key(name)
    }

    /// For a set of names: adds one, saying whether it's new.
    pub(crate) fn add(&mut self, name: K) -> bool {
        self.insert(name, ()).is_none()
    }
}
