//! A `Dict`'s insides (D32): keys looked up by hash, kept in the order they were added.
//!
//! The order is what a script sees, so going through a dictionary is the same on every machine (D50). The hash is
//! fixed, and only decides how fast a key is found, never what is found; long searches are counted as steps (D14),
//! so keys made to collide can't freeze the host.

/// What one pair costs against the memory cap: its key, its value, and what finding it again needs.
const PAIR_COST: usize = 40;

const EMPTY: u32 = u32::MAX;
const REMOVED: u32 = u32::MAX - 1;

#[derive(Clone, Copy, Debug)]
struct Pair {
    hash: u64,
    key: u64,
    value: u64,
    /// The start of the key itself, as [`Probe::prefix`].
    prefix: u64,
    length: u32,
    live: bool,
}

/// A key as a dictionary looks it up: its hash, and enough of the key itself to tell nearly every pair of keys
/// apart without reading either. A `Whole` or a `Bool` is all there in `prefix`; a text gives its length and its
/// first 8 bytes, which is the whole of a short text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Probe {
    pub hash: u64,
    pub prefix: u64,
    pub length: u32,
}

impl Probe {
    /// For a `Whole` or a `Bool` key, by its bits.
    #[must_use]
    #[inline]
    pub fn of_bits(bits: u64) -> Probe {
        Probe { hash: hash_bits(bits), prefix: bits, length: 0 }
    }

    /// For a text key.
    #[must_use]
    #[inline]
    pub fn of_text(text: &str) -> Probe {
        let mut first = [0u8; 8];
        for (slot, byte) in first.iter_mut().zip(text.as_bytes()) {
            *slot = *byte;
        }
        Probe {
            hash: hash_text(text),
            prefix: u64::from_le_bytes(first),
            length: u32::try_from(text.len()).unwrap_or(u32::MAX),
        }
    }
}

#[derive(Debug, Default)]
pub struct Dict {
    /// For each place a hash can land, which pair is there.
    places: Vec<u32>,
    /// The pairs, in the order they were added. A removed pair stays as a gap until the gaps are closed up.
    pairs: Vec<Pair>,
    live: usize,
    /// How many places hold a pair or the mark of a removed one.
    used_places: usize,
    pub text_keys: bool,
    pub reference_values: bool,
}

/// Mixes a `Whole`'s or a `Bool`'s bits into a hash.
#[must_use]
#[inline]
pub fn hash_bits(bits: u64) -> u64 {
    let mut hash = bits.wrapping_add(0x9E37_79B9_7F4A_7C15);
    hash = (hash ^ (hash >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    hash = (hash ^ (hash >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    hash ^ (hash >> 31)
}

/// Mixes text into a hash, eight bytes at a time.
#[must_use]
pub fn hash_text(text: &str) -> u64 {
    const FACTOR: u64 = 0x9E37_79B9_7F4A_7C15;
    let bytes = text.as_bytes();
    let mut hash = (bytes.len() as u64).wrapping_mul(FACTOR);
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let word = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
        hash = (hash.rotate_left(5) ^ word).wrapping_mul(FACTOR);
    }
    let mut last = [0u8; 8];
    for (slot, byte) in last.iter_mut().zip(chunks.remainder()) {
        *slot = *byte;
    }
    hash = (hash.rotate_left(5) ^ u64::from_le_bytes(last)).wrapping_mul(FACTOR);
    // Mixed once more, so that keys differing only in their last characters still land far apart: a place is
    // picked by the hash's low bits, which multiplying alone leaves blind to a key's high bytes.
    hash_bits(hash)
}

impl Dict {
    #[must_use]
    pub fn new(text_keys: bool, reference_values: bool) -> Self {
        Dict { text_keys, reference_values, ..Dict::default() }
    }

    /// How many keys the dictionary has (D34).
    #[must_use]
    pub fn len(&self) -> usize {
        self.live
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// What the dictionary's contents cost against the memory cap, in Seal's own sizes.
    #[must_use]
    pub fn size(&self) -> usize {
        self.pairs.capacity().saturating_mul(PAIR_COST).saturating_add(self.places.len().saturating_mul(4))
    }

    /// How much [`Dict::size`] would grow if one more key were added now.
    #[must_use]
    pub fn growth_for_one_more(&self) -> usize {
        let mut growth = 0usize;
        if self.pairs.len() == self.pairs.capacity() {
            let new_capacity = self.pairs.capacity().saturating_mul(2).max(4);
            growth = growth.saturating_add((new_capacity - self.pairs.capacity()).saturating_mul(PAIR_COST));
        }
        if (self.used_places + 1).saturating_mul(4) > self.places.len().saturating_mul(3) {
            let new_places = self.places.len().saturating_mul(2).max(8);
            growth = growth.saturating_add((new_places - self.places.len()).saturating_mul(4));
        }
        growth
    }

    /// Looks a key up by its hash, with `same` saying whether a stored key is the one wanted. Gives the pair's
    /// number when the key is there, and how many places were looked at.
    pub fn find(&self, probe: &Probe, same: impl Fn(u64) -> bool) -> (Option<usize>, u64) {
        let hash = probe.hash;
        if self.places.is_empty() {
            return (None, 0);
        }
        let mask = self.places.len() - 1;
        // Only the low bits pick the place, and `mask` keeps them inside the list.
        #[allow(clippy::cast_possible_truncation)]
        let mut place = (hash as usize) & mask;
        let mut looked = 0u64;
        loop {
            looked += 1;
            match self.places.get(place).copied() {
                None | Some(EMPTY) => return (None, looked),
                Some(REMOVED) => {}
                Some(number) => {
                    if let Some(pair) = self.pairs.get(number as usize)
                        && pair.live
                        && pair.hash == hash
                        && pair.prefix == probe.prefix
                        && pair.length == probe.length
                        // A key of 8 bytes or fewer is all in its prefix, so it's the same key without a look.
                        && (probe.length <= 8 || same(pair.key))
                    {
                        return (Some(number as usize), looked);
                    }
                }
            }
            if looked > self.places.len() as u64 {
                return (None, looked);
            }
            place = (place + 1) & mask;
        }
    }

    #[must_use]
    pub fn value_at(&self, number: usize) -> Option<u64> {
        self.pairs.get(number).filter(|pair| pair.live).map(|pair| pair.value)
    }

    /// Gives a pair a new value, handing back the old one.
    pub fn replace_value(&mut self, number: usize, value: u64) -> Option<u64> {
        let pair = self.pairs.get_mut(number)?;
        Some(std::mem::replace(&mut pair.value, value))
    }

    fn place_pair(places: &mut [u32], hash: u64, number: u32) {
        if places.is_empty() {
            return;
        }
        let mask = places.len() - 1;
        #[allow(clippy::cast_possible_truncation)]
        let mut place = (hash as usize) & mask;
        for _ in 0..places.len() {
            match places.get_mut(place) {
                Some(slot) if *slot == EMPTY || *slot == REMOVED => {
                    *slot = number;
                    return;
                }
                _ => place = (place + 1) & mask,
            }
        }
    }

    fn rebuild_places(&mut self, size: usize) {
        self.places.clear();
        self.places.resize(size, EMPTY);
        for (number, pair) in (0u32..).zip(&self.pairs) {
            if pair.live {
                Self::place_pair(&mut self.places, pair.hash, number);
            }
        }
        self.used_places = self.live;
    }

    /// Adds a key that isn't there yet, at the end of the order.
    pub fn add(&mut self, probe: &Probe, key: u64, value: u64) {
        let hash = probe.hash;
        if self.pairs.len() == self.pairs.capacity() {
            let new_capacity = self.pairs.capacity().saturating_mul(2).max(4);
            self.pairs.reserve_exact(new_capacity - self.pairs.len());
        }
        if (self.used_places + 1).saturating_mul(4) > self.places.len().saturating_mul(3) {
            let size = self.places.len().saturating_mul(2).max(8);
            self.rebuild_places(size);
        }
        let number = u32::try_from(self.pairs.len()).unwrap_or(REMOVED - 1);
        self.pairs.push(Pair { hash, key, value, prefix: probe.prefix, length: probe.length, live: true });
        self.live += 1;
        self.used_places += 1;
        Self::place_pair(&mut self.places, hash, number);
    }

    /// Takes a pair out, handing back its key and value.
    pub fn remove(&mut self, number: usize) -> Option<(u64, u64)> {
        let pair = self.pairs.get_mut(number)?;
        if !pair.live {
            return None;
        }
        pair.live = false;
        let (hash, key, value) = (pair.hash, pair.key, pair.value);
        self.live -= 1;
        if !self.places.is_empty() {
            let mask = self.places.len() - 1;
            #[allow(clippy::cast_possible_truncation)]
            let mut place = (hash as usize) & mask;
            for _ in 0..self.places.len() {
                match self.places.get_mut(place) {
                    Some(slot) if *slot as usize == number => {
                        *slot = REMOVED;
                        break;
                    }
                    Some(slot) if *slot == EMPTY => break,
                    _ => place = (place + 1) & mask,
                }
            }
        }
        // Once the gaps outnumber the pairs, they're closed up, keeping the order.
        let gaps = self.pairs.len() - self.live;
        if gaps > 8 && gaps > self.live {
            self.pairs.retain(|pair| pair.live);
            let size = self.places.len();
            self.rebuild_places(size);
        }
        Some((key, value))
    }

    /// The next pair at or after `position`, in the order the keys were added: the position after it, its key and
    /// its value.
    #[must_use]
    pub fn next_from(&self, position: usize) -> Option<(usize, u64, u64)> {
        self.pairs
            .iter()
            .enumerate()
            .skip(position)
            .find(|(_, pair)| pair.live)
            .map(|(number, pair)| (number + 1, pair.key, pair.value))
    }

    /// Every pair, in order.
    pub fn pairs(&self) -> impl Iterator<Item = (u64, u64)> + '_ {
        self.pairs.iter().filter(|pair| pair.live).map(|pair| (pair.key, pair.value))
    }

    /// A copy with its own lists, sized to what it holds.
    #[must_use]
    pub fn copy(&self) -> Dict {
        let mut pairs = Vec::new();
        pairs.reserve_exact(self.live);
        pairs.extend(self.pairs.iter().filter(|pair| pair.live).copied());
        let mut copy = Dict {
            places: Vec::new(),
            pairs,
            live: self.live,
            used_places: 0,
            text_keys: self.text_keys,
            reference_values: self.reference_values,
        };
        let mut size = 8usize;
        while self.live.saturating_mul(4) > size.saturating_mul(3) {
            size = size.saturating_mul(2);
        }
        if self.live > 0 {
            copy.rebuild_places(size);
        }
        copy
    }

    /// Adds the heap values the dictionary holds, its text keys and its heap values, to `out`.
    pub fn push_references(&self, out: &mut Vec<u64>) {
        for pair in self.pairs.iter().filter(|pair| pair.live) {
            if self.text_keys {
                out.push(pair.key);
            }
            if self.reference_values {
                out.push(pair.value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: D32, D34
    #[test]
    fn d32_a_dict_keeps_the_order_keys_were_added_in() {
        let mut dict = Dict::new(false, false);
        for key in [5u64, 3, 9, 1] {
            assert_eq!(dict.find(&Probe::of_bits(key), |stored| stored == key).0, None);
            dict.add(&Probe::of_bits(key), key, key * 10);
        }
        let found = dict.find(&Probe::of_bits(3), |stored| stored == 3).0;
        assert_eq!(found.and_then(|number| dict.value_at(number)), Some(30));
        assert_eq!(found.and_then(|number| dict.remove(number)), Some((3, 30)));
        assert_eq!(dict.find(&Probe::of_bits(3), |stored| stored == 3).0, None);
        assert_eq!(dict.pairs().collect::<Vec<_>>(), vec![(5, 50), (9, 90), (1, 10)]);
        assert_eq!(dict.len(), 3);
    }

    #[test]
    fn d32_many_keys_added_and_removed() {
        let mut dict = Dict::new(false, false);
        for key in 0u64..1000 {
            dict.add(&Probe::of_bits(key), key, key);
        }
        for key in (0u64..1000).filter(|key| key % 3 != 0) {
            let number = dict.find(&Probe::of_bits(key), |stored| stored == key).0;
            assert!(number.and_then(|number| dict.remove(number)).is_some());
        }
        assert_eq!(dict.len(), 334);
        for key in 0u64..1000 {
            let found = dict.find(&Probe::of_bits(key), |stored| stored == key).0.is_some();
            assert_eq!(found, key % 3 == 0);
        }
        assert_eq!(
            dict.pairs().map(|(key, _)| key).collect::<Vec<_>>(),
            (0u64..1000).filter(|key| key % 3 == 0).collect::<Vec<_>>()
        );
    }
}
