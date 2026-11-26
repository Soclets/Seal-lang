//! `.sealc` files: scripts compiled ahead of time, one to a file or many in a bundle, loaded by name and verified
//! every time (D141, D153, D154).
//!
//! The file is the compiled script as it is in memory, written field by field, little-endian, every list counted
//! before its items: a header with this Seal's fingerprint, an index of the scripts by name, each script, and a
//! checksum of all that comes before it. Reading it back checks everything as it goes: every count against what's
//! left of the file, every text as UTF-8, every number that picks something out of a list against that list. Then
//! the verifier goes through the instructions: each register inside its procedure's frame, each jump landing inside
//! its own procedure, each table it reads from one the script has. A file that fails any of it is refused, as a
//! script with a mistake is.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::bytecode::{
    Compare, Compiled, CompiledParameter, CompiledProcedure, Crossing, CrossingCase, DoorSite, FAR, Instruction,
    JoinPiece, Layout, ProcedureEntry, Register,
};
use super::instruction_file;
use crate::big::Big;
use crate::check::tree::{NumberConversion, NumberHow, NumberType, Rounding};
use crate::check::types::{Edge, Limits, Size, TypeTable};
use crate::decimal::{Decimal, HugeDecimal, Number};
use crate::host::Value;
use crate::message::Place;

/// What every `.sealc` file starts with.
const MAGIC: &[u8; 8] = b"SEALC\0\0\x01";

/// How deep a value or a crossing written in a file may nest, so reading one can't overrun the host's stack.
const DEEPEST: u32 = 64;

/// Why a file can't be loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileError {
    /// Not a `.sealc` file at all.
    NotSeal,
    /// Built by another Seal, or another build of this one (D137).
    OtherSeal,
    /// Cut short, changed, or wrong inside.
    Damaged(&'static str),
    /// The verifier found an instruction that doesn't fit its script.
    Unverified(String),
    /// No script of that name in the file.
    NotInFile(String),
    /// Built for doors other than the host's.
    OtherDoors,
}

impl std::fmt::Display for FileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::NotSeal => formatter.write_str("this isn't a `.sealc` file"),
            FileError::OtherSeal => formatter.write_str("this `.sealc` file was built by another Seal"),
            FileError::Damaged(what) => write!(formatter, "this `.sealc` file is damaged: {what}"),
            FileError::Unverified(what) => write!(formatter, "this `.sealc` file doesn't pass verifying: {what}"),
            FileError::NotInFile(name) => write!(formatter, "this `.sealc` file has no script named `{name}`"),
            FileError::OtherDoors => {
                formatter.write_str("this `.sealc` file was built for doors other than the ones this host opens")
            }
        }
    }
}

/// A 64-bit hash, for the checksum and the keys, eight bytes at a time: quick, and the same everywhere. It finds a
/// file damaged by chance, never one changed on purpose (D141).
#[must_use]
pub fn hash(bytes: &[u8]) -> u64 {
    // Four lanes of eight bytes each, worked on side by side, so the processor needn't wait on one to start the next
    // (D171); then joined, and what's left over added a word or a byte at a time.
    const MULTIPLY: u64 = 0x9e37_79b9_7f4a_7c15;
    let word = |eight: &[u8]| {
        let mut array = [0u8; 8];
        array.copy_from_slice(eight);
        u64::from_le_bytes(array)
    };
    let mix = |hash: u64, word: u64| {
        let hash = (hash ^ word).wrapping_mul(MULTIPLY);
        hash ^ (hash >> 29)
    };
    let mut lanes = [
        0xcbf2_9ce4_8422_2325_u64 ^ (bytes.len() as u64),
        0x8422_2325_cbf2_9ce4,
        0x2545_f491_4f6c_dd1d,
        0x9e37_79b9_7f4a_7c15,
    ];
    let mut blocks = bytes.chunks_exact(32);
    for block in &mut blocks {
        for (lane, eight) in lanes.iter_mut().zip(block.chunks_exact(8)) {
            *lane = mix(*lane, word(eight));
        }
    }
    let mut hash = lanes.iter().fold(0u64, |joined, &lane| mix(joined, lane));
    let mut words = blocks.remainder().chunks_exact(8);
    for eight in &mut words {
        hash = mix(hash, word(eight));
    }
    for &byte in words.remainder() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    hash ^ (hash >> 32)
}

/// This build of Seal: its version, and the code that decides what a compiled script means, so a file built by any
/// other build, even of the same version, is refused and built again (D137, D141).
#[must_use]
pub fn fingerprint() -> u64 {
    static FINGERPRINT: OnceLock<u64> = OnceLock::new();
    *FINGERPRINT.get_or_init(|| {
        let parts: [&str; 13] = [
            env!("CARGO_PKG_VERSION"),
            include_str!("bytecode.rs"),
            include_str!("compiler.rs"),
            include_str!("file.rs"),
            include_str!("instruction_file.rs"),
            include_str!("../vm/machine.rs"),
            include_str!("../vm/numbers.rs"),
            include_str!("../vm/heap.rs"),
            include_str!("../vm/show.rs"),
            include_str!("../vm/crossing.rs"),
            include_str!("../vm/dict.rs"),
            include_str!("../decimal.rs"),
            include_str!("../check/types.rs"),
        ];
        let mut joined = 0u64;
        for part in parts {
            joined = hash(&[joined.to_le_bytes().as_slice(), part.as_bytes()].concat());
        }
        joined
    })
}

/// Bytes being written.
#[derive(Default)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u128(&mut self, value: u128) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn flag(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    /// A count, as the number of items a list has.
    pub fn count(&mut self, count: usize) {
        self.u32(u32::try_from(count).unwrap_or(u32::MAX));
    }

    pub fn text(&mut self, text: &str) {
        self.count(text.len());
        self.bytes.extend_from_slice(text.as_bytes());
    }

    fn place(&mut self, place: Place) {
        self.u32(place.line);
        self.u32(place.column);
        self.u32(place.width);
    }
}

/// Bytes being read, each read checked against what's left.
pub struct Reader<'b> {
    bytes: &'b [u8],
    at: usize,
}

impl<'b> Reader<'b> {
    #[must_use]
    pub fn new(bytes: &'b [u8]) -> Self {
        Reader { bytes, at: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'b [u8], FileError> {
        let end = self.at.checked_add(length).ok_or(FileError::Damaged("a length past its end"))?;
        let taken = self.bytes.get(self.at..end).ok_or(FileError::Damaged("it ends too soon"))?;
        self.at = end;
        Ok(taken)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], FileError> {
        let mut array = [0u8; N];
        array.copy_from_slice(self.take(N)?);
        Ok(array)
    }

    /// # Errors
    ///
    /// When the file ends.
    pub fn u8(&mut self) -> Result<u8, FileError> {
        Ok(self.array::<1>()?[0])
    }

    /// # Errors
    ///
    /// When the file ends.
    pub fn u16(&mut self) -> Result<u16, FileError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    /// # Errors
    ///
    /// When the file ends.
    pub fn u32(&mut self) -> Result<u32, FileError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    /// # Errors
    ///
    /// When the file ends.
    pub fn u64(&mut self) -> Result<u64, FileError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    /// # Errors
    ///
    /// When the file ends.
    pub fn u128(&mut self) -> Result<u128, FileError> {
        Ok(u128::from_le_bytes(self.array()?))
    }

    /// # Errors
    ///
    /// When the file ends, or the byte is neither 0 nor 1.
    pub fn flag(&mut self) -> Result<bool, FileError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(FileError::Damaged("a yes-or-no that's neither")),
        }
    }

    /// A count of items, each at least `smallest` bytes, so a damaged count can't make room for more than the file
    /// could hold.
    ///
    /// # Errors
    ///
    /// When the file ends, or the count is more than what's left could hold.
    pub fn count(&mut self, smallest: usize) -> Result<usize, FileError> {
        let count = self.u32()? as usize;
        if count.saturating_mul(smallest.max(1)) > self.bytes.len().saturating_sub(self.at) {
            return Err(FileError::Damaged("a count bigger than the file"));
        }
        Ok(count)
    }

    /// # Errors
    ///
    /// When the file ends, or the text isn't UTF-8.
    pub fn text(&mut self) -> Result<String, FileError> {
        self.text_in_file().map(str::to_owned)
    }

    /// A text, as it is in the file, with nothing made.
    ///
    /// # Errors
    ///
    /// When the file ends, or the text isn't UTF-8.
    pub fn text_in_file(&mut self) -> Result<&'b str, FileError> {
        let length = self.count(1)?;
        let bytes = self.take(length)?;
        std::str::from_utf8(bytes).map_err(|_| FileError::Damaged("a text that isn't UTF-8"))
    }

    /// # Errors
    ///
    /// When the file ends, or the byte isn't a comparison's.
    pub fn compare(&mut self) -> Result<Compare, FileError> {
        Ok(match self.u8()? {
            0 => Compare::Equal,
            1 => Compare::NotEqual,
            2 => Compare::Less,
            3 => Compare::AtMost,
            4 => Compare::Greater,
            5 => Compare::AtLeast,
            _ => return Err(FileError::Damaged("a comparison that isn't one")),
        })
    }

    fn place(&mut self) -> Result<Place, FileError> {
        Ok(Place { line: self.u32()?, column: self.u32()?, width: self.u32()? })
    }

    fn done(&self) -> bool {
        self.at == self.bytes.len()
    }
}

/// One script in a file, read back, its name and text as they are in the file.
pub struct FileScript<'b> {
    pub name: &'b str,
    pub source: &'b str,
    pub compiled: Compiled,
}

/// One script to write into a file: its name, its source, and what it compiled to.
pub struct ToWrite<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub compiled: &'a Compiled,
}

/// trace: D153. A `.sealc` file holding these scripts, compiled for the doors whose key is `doors`: one script, or
/// a bundle of many, with an index at its front.
#[must_use]
pub fn write(scripts: &[ToWrite], doors: u64) -> Vec<u8> {
    let blobs: Vec<Vec<u8>> = scripts
        .iter()
        .map(|script| {
            let mut out = Writer::default();
            out.text(script.name);
            out.text(script.source);
            write_compiled(script.compiled, &mut out);
            out.bytes
        })
        .collect();
    let mut out = Writer::default();
    out.bytes.extend_from_slice(MAGIC);
    out.u64(fingerprint());
    out.u64(doors);
    out.count(scripts.len());
    for (script, blob) in scripts.iter().zip(&blobs) {
        out.text(script.name);
        out.count(blob.len());
    }
    for blob in &blobs {
        out.bytes.extend_from_slice(blob);
    }
    let checksum = hash(&out.bytes);
    out.u64(checksum);
    out.bytes
}

/// The header checked and the index read: the scripts' names, each with where its bytes are.
fn index(bytes: &[u8], doors: u64) -> Result<Vec<(&str, std::ops::Range<usize>)>, FileError> {
    if bytes.len() < MAGIC.len() || bytes.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
        return Err(FileError::NotSeal);
    }
    let mut input = Reader::new(bytes);
    input.take(MAGIC.len())?;
    if input.u64()? != fingerprint() {
        return Err(FileError::OtherSeal);
    }
    // The checksum covers everything before it, so a damaged file is found before anything in it is trusted.
    let body = bytes.len().checked_sub(8).ok_or(FileError::Damaged("it ends too soon"))?;
    let (covered, written) = bytes.split_at(body);
    let mut sum = [0u8; 8];
    sum.copy_from_slice(written);
    if hash(covered) != u64::from_le_bytes(sum) {
        return Err(FileError::Damaged("its checksum doesn't match"));
    }
    if input.u64()? != doors {
        return Err(FileError::OtherDoors);
    }
    let count = input.count(8)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let name = input.text_in_file()?;
        let length = input.count(0)?;
        entries.push((name, length));
    }
    let mut start = input.at;
    let mut index = Vec::with_capacity(entries.len());
    for (name, length) in entries {
        let end =
            start.checked_add(length).filter(|&end| end <= body).ok_or(FileError::Damaged("a script past its end"))?;
        index.push((name, start..end));
        start = end;
    }
    if start != body {
        return Err(FileError::Damaged("bytes between its scripts and its checksum"));
    }
    Ok(index)
}

/// The names of the scripts a file holds, in order.
///
/// # Errors
///
/// When the file isn't one this Seal built for these doors, or is damaged.
pub fn names(bytes: &[u8], doors: u64) -> Result<Vec<String>, FileError> {
    Ok(index(bytes, doors)?.into_iter().map(|(name, _)| name.to_owned()).collect())
}

/// trace: D141, D153. Reads the script named `name` from a file, and verifies it.
///
/// # Errors
///
/// When the file isn't one this Seal built for these doors, is damaged, has no such script, or fails verifying.
pub fn read<'b>(bytes: &'b [u8], name: &str, doors: u64, door_count: usize) -> Result<FileScript<'b>, FileError> {
    let index = index(bytes, doors)?;
    let range = index
        .into_iter()
        .find(|(written, _)| *written == name)
        .map(|(_, range)| range)
        .ok_or_else(|| FileError::NotInFile(name.to_owned()))?;
    let mut input = Reader::new(bytes.get(range).ok_or(FileError::Damaged("a script past its end"))?);
    let written_name = input.text_in_file()?;
    let source = input.text_in_file()?;
    if written_name != name {
        return Err(FileError::Damaged("a script that isn't what its index says"));
    }
    let compiled = read_compiled(&mut input)?;
    if !input.done() {
        return Err(FileError::Damaged("bytes after a script"));
    }
    verify(&compiled, door_count).map_err(FileError::Unverified)?;
    Ok(FileScript { name: written_name, source, compiled })
}

// ---- the compiled script, field by field ----

#[allow(clippy::too_many_lines)] // each field in turn, in the order they're read back
fn write_compiled(compiled: &Compiled, out: &mut Writer) {
    out.count(compiled.code.len());
    for instruction in &compiled.code {
        instruction_file::write(*instruction, out);
    }
    out.count(compiled.places.len());
    for &place in &compiled.places {
        out.place(place);
    }
    out.count(compiled.constants.len());
    for &constant in &compiled.constants {
        out.u64(constant);
    }
    out.count(compiled.texts.len());
    for text in &compiled.texts {
        out.text(text);
    }
    out.count(compiled.procedures.len());
    for procedure in &compiled.procedures {
        write_procedure(procedure, out);
    }
    out.count(compiled.entries.len());
    for entry in &compiled.entries {
        out.u32(entry.entry);
        out.u32(entry.frame_size);
    }
    out.u32(compiled.top_level_entry);
    out.u32(compiled.top_level_frame_size);
    out.u32(compiled.global_count);
    out.count(compiled.global_references.len());
    for &reference in &compiled.global_references {
        out.flag(reference);
    }
    out.count(compiled.layouts.len());
    for layout in &compiled.layouts {
        out.u32(layout.case);
        out.count(layout.references.len());
        for &reference in &layout.references {
            out.flag(reference);
        }
    }
    out.count(compiled.door_sites.len());
    for site in &compiled.door_sites {
        out.u32(site.door);
        out.flag(site.plain);
        out.u32(site.first_argument);
        out.u32(site.argument_count);
        write_crossing(&site.result, out);
    }
    out.count(compiled.door_arguments.len());
    for crossing in &compiled.door_arguments {
        write_crossing(crossing, out);
    }
    out.count(compiled.joins.len());
    for pieces in &compiled.joins {
        out.count(pieces.len());
        for piece in pieces {
            write_piece(*piece, out);
        }
    }
    out.count(compiled.compared_types.len());
    for &type_id in &compiled.compared_types {
        out.u32(type_id);
    }
    out.count(compiled.scales.len());
    for &(size, whole) in &compiled.scales {
        write_size(size, out);
        out.flag(whole);
    }
    out.count(compiled.limits.len());
    for &limits in &compiled.limits {
        write_limits(limits, out);
    }
    out.count(compiled.numbers.len());
    for number in &compiled.numbers {
        write_number(number, out);
    }
    out.count(compiled.number_conversions.len());
    for &(conversion, failure) in &compiled.number_conversions {
        write_conversion(conversion, out);
        out.u16(failure);
    }
    // Maps are written in their keys' order, so a script always builds to the same bytes.
    let mut far: Vec<(u32, i32)> = compiled.far_jumps.iter().map(|(&at, &offset)| (at, offset)).collect();
    far.sort_unstable();
    out.count(far.len());
    for (at, offset) in far {
        out.u32(at);
        out.u32(offset.cast_unsigned());
    }
    out.count(compiled.fused_places.len());
    for &(at, place) in &compiled.fused_places {
        out.u32(at);
        out.place(place);
    }
    let mut pauses: Vec<(&u32, &Vec<Register>)> = compiled.pause_maps.iter().collect();
    pauses.sort_unstable_by_key(|(at, _)| **at);
    out.count(pauses.len());
    for (&at, registers) in pauses {
        out.u32(at);
        out.count(registers.len());
        for &register in registers {
            out.u16(register);
        }
    }
    // A failure of Seal's own is its number among them, from 1, so reading it makes nothing (D171); 0 is a text.
    out.count(compiled.failure_names.len());
    for name in &compiled.failure_names {
        if let Some(own) = crate::check::BUILT_IN_FAILURES.iter().position(|own| *own == name.as_ref()) {
            out.u8(u8::try_from(own + 1).unwrap_or(0));
        } else {
            out.u8(0);
            out.text(name);
        }
    }
    for &failure in &compiled.built_in_failures {
        out.u32(failure);
    }
    compiled.types.write_file(out);
}

#[allow(clippy::too_many_lines)] // each field in turn, in the order they were written
fn read_compiled(input: &mut Reader) -> Result<Compiled, FileError> {
    let count = input.count(1)?;
    let mut code = Vec::with_capacity(count);
    for _ in 0..count {
        code.push(instruction_file::read(input)?);
    }
    let count = input.count(12)?;
    let mut places = Vec::with_capacity(count);
    for _ in 0..count {
        places.push(input.place()?);
    }
    let count = input.count(8)?;
    let mut constants = Vec::with_capacity(count);
    for _ in 0..count {
        constants.push(input.u64()?);
    }
    let count = input.count(4)?;
    let mut texts = Vec::with_capacity(count);
    for _ in 0..count {
        texts.push(input.text()?);
    }
    let count = input.count(16)?;
    let mut procedures = Vec::with_capacity(count);
    for _ in 0..count {
        procedures.push(read_procedure(input)?);
    }
    let count = input.count(8)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(ProcedureEntry { entry: input.u32()?, frame_size: input.u32()? });
    }
    let (top_level_entry, top_level_frame_size, global_count) = (input.u32()?, input.u32()?, input.u32()?);
    let count = input.count(1)?;
    let mut global_references = Vec::with_capacity(count);
    for _ in 0..count {
        global_references.push(input.flag()?);
    }
    let count = input.count(8)?;
    let mut layouts = Vec::with_capacity(count);
    for _ in 0..count {
        let case = input.u32()?;
        let fields = input.count(1)?;
        let mut references = Vec::with_capacity(fields);
        for _ in 0..fields {
            references.push(input.flag()?);
        }
        layouts.push(Layout { case, references });
    }
    let count = input.count(14)?;
    let mut door_sites = Vec::with_capacity(count);
    for _ in 0..count {
        door_sites.push(DoorSite {
            door: input.u32()?,
            plain: input.flag()?,
            first_argument: input.u32()?,
            argument_count: input.u32()?,
            result: read_crossing(input, 0)?,
        });
    }
    let count = input.count(1)?;
    let mut door_arguments = Vec::with_capacity(count);
    for _ in 0..count {
        door_arguments.push(read_crossing(input, 0)?);
    }
    let count = input.count(4)?;
    let mut joins = Vec::with_capacity(count);
    for _ in 0..count {
        let pieces = input.count(3)?;
        let mut join = Vec::with_capacity(pieces);
        for _ in 0..pieces {
            join.push(read_piece(input)?);
        }
        joins.push(join);
    }
    let count = input.count(4)?;
    let mut compared_types = Vec::with_capacity(count);
    for _ in 0..count {
        compared_types.push(input.u32()?);
    }
    let count = input.count(35)?;
    let mut scales = Vec::with_capacity(count);
    for _ in 0..count {
        scales.push((read_size(input)?, input.flag()?));
    }
    let count = input.count(4)?;
    let mut limits = Vec::with_capacity(count);
    for _ in 0..count {
        limits.push(read_limits(input)?);
    }
    let count = input.count(1)?;
    let mut numbers = Vec::with_capacity(count);
    for _ in 0..count {
        numbers.push(read_number(input)?);
    }
    let count = input.count(5)?;
    let mut number_conversions = Vec::with_capacity(count);
    for _ in 0..count {
        number_conversions.push((read_conversion(input)?, input.u16()?));
    }
    let count = input.count(8)?;
    let mut far_jumps = HashMap::with_capacity(count);
    for _ in 0..count {
        far_jumps.insert(input.u32()?, input.u32()?.cast_signed());
    }
    let count = input.count(16)?;
    let mut fused_places = Vec::with_capacity(count);
    for _ in 0..count {
        fused_places.push((input.u32()?, input.place()?));
    }
    let count = input.count(8)?;
    let mut pause_maps = HashMap::with_capacity(count);
    for _ in 0..count {
        let at = input.u32()?;
        let registers = input.count(2)?;
        let mut list = Vec::with_capacity(registers);
        for _ in 0..registers {
            list.push(input.u16()?);
        }
        pause_maps.insert(at, list);
    }
    let count = input.count(4)?;
    let mut failure_names = Vec::with_capacity(count);
    for _ in 0..count {
        let name = match input.u8()? {
            0 => std::borrow::Cow::Owned(input.text()?),
            own => std::borrow::Cow::Borrowed(
                *crate::check::BUILT_IN_FAILURES
                    .get(usize::from(own) - 1)
                    .ok_or(FileError::Damaged("a failure of Seal's own that isn't one"))?,
            ),
        };
        failure_names.push(name);
    }
    let built_in_failures = [input.u32()?, input.u32()?, input.u32()?];
    let types = TypeTable::read_file(input)?;
    Ok(Compiled {
        code,
        places,
        constants,
        texts,
        procedures,
        entries,
        top_level_entry,
        top_level_frame_size,
        global_count,
        global_references,
        layouts,
        door_sites,
        door_arguments,
        joins,
        compared_types,
        scales,
        limits,
        numbers,
        number_conversions,
        far_jumps,
        fused_places,
        pause_maps,
        failure_names,
        built_in_failures,
        types,
    })
}

fn write_procedure(procedure: &CompiledProcedure, out: &mut Writer) {
    out.text(&procedure.name);
    out.u32(procedure.entry);
    out.u32(procedure.frame_size);
    out.count(procedure.parameters.len());
    for parameter in &procedure.parameters {
        out.text(&parameter.name);
        match &parameter.label {
            Some(label) => {
                out.flag(true);
                out.text(label);
            }
            None => out.flag(false),
        }
        write_crossing(&parameter.crossing, out);
        out.u32(parameter.type_id.0);
        match &parameter.default {
            Some(value) => {
                out.flag(true);
                write_value(value, out);
            }
            None => out.flag(false),
        }
    }
    write_crossing(&procedure.result, out);
    out.u32(procedure.result_type.0);
    out.count(procedure.failures.len());
    for failure in &procedure.failures {
        out.text(failure);
    }
    out.flag(procedure.can_pause);
    out.flag(procedure.holds_itself);
    out.flag(procedure.generic);
    out.flag(procedure.method);
}

fn read_procedure(input: &mut Reader) -> Result<CompiledProcedure, FileError> {
    let name = input.text()?;
    let (entry, frame_size) = (input.u32()?, input.u32()?);
    let count = input.count(8)?;
    let mut parameters = Vec::with_capacity(count);
    for _ in 0..count {
        let name = input.text()?;
        let label = if input.flag()? { Some(input.text()?) } else { None };
        let crossing = read_crossing(input, 0)?;
        let type_id = crate::check::types::TypeId(input.u32()?);
        let default = if input.flag()? { Some(read_value(input, 0)?) } else { None };
        parameters.push(CompiledParameter { name, label, crossing, type_id, default });
    }
    let result = read_crossing(input, 0)?;
    let result_type = crate::check::types::TypeId(input.u32()?);
    let count = input.count(4)?;
    let mut failures = Vec::with_capacity(count);
    for _ in 0..count {
        failures.push(input.text()?);
    }
    Ok(CompiledProcedure {
        name,
        entry,
        frame_size,
        parameters,
        result,
        result_type,
        failures,
        can_pause: input.flag()?,
        holds_itself: input.flag()?,
        generic: input.flag()?,
        method: input.flag()?,
    })
}

fn write_crossing(crossing: &Crossing, out: &mut Writer) {
    let fields = |fields: &[(String, Crossing)], out: &mut Writer| {
        out.count(fields.len());
        for (name, crossing) in fields {
            out.text(name);
            write_crossing(crossing, out);
        }
    };
    match crossing {
        Crossing::Nothing => out.u8(0),
        Crossing::Whole => out.u8(1),
        Crossing::Float => out.u8(2),
        Crossing::Bool => out.u8(3),
        Crossing::Text => out.u8(4),
        Crossing::HugeWhole => out.u8(5),
        Crossing::Decimal => out.u8(6),
        Crossing::HugeDecimal => out.u8(7),
        Crossing::List(item) => {
            out.u8(8);
            write_crossing(item, out);
        }
        Crossing::Dict(key, value) => {
            out.u8(9);
            write_crossing(key, out);
            write_crossing(value, out);
        }
        Crossing::Maybe { inner, layout } => {
            out.u8(10);
            write_crossing(inner, out);
            out.u16(*layout);
        }
        Crossing::Record { name, layout, fields: written } => {
            out.u8(11);
            out.text(name);
            out.u16(*layout);
            fields(written, out);
        }
        Crossing::OneOf { name, cases } => {
            out.u8(12);
            out.text(name);
            out.count(cases.len());
            for case in cases {
                out.text(&case.name);
                out.u16(case.layout);
                fields(&case.fields, out);
            }
        }
        Crossing::Itself { name } => {
            out.u8(13);
            out.text(name);
        }
    }
}

fn read_crossing(input: &mut Reader, depth: u32) -> Result<Crossing, FileError> {
    if depth > DEEPEST {
        return Err(FileError::Damaged("a type nested too deep"));
    }
    let fields = |input: &mut Reader| -> Result<Vec<(String, Crossing)>, FileError> {
        let count = input.count(5)?;
        let mut fields = Vec::with_capacity(count);
        for _ in 0..count {
            fields.push((input.text()?, read_crossing(input, depth + 1)?));
        }
        Ok(fields)
    };
    Ok(match input.u8()? {
        0 => Crossing::Nothing,
        1 => Crossing::Whole,
        2 => Crossing::Float,
        3 => Crossing::Bool,
        4 => Crossing::Text,
        5 => Crossing::HugeWhole,
        6 => Crossing::Decimal,
        7 => Crossing::HugeDecimal,
        8 => Crossing::List(Box::new(read_crossing(input, depth + 1)?)),
        9 => Crossing::Dict(Box::new(read_crossing(input, depth + 1)?), Box::new(read_crossing(input, depth + 1)?)),
        10 => Crossing::Maybe { inner: Box::new(read_crossing(input, depth + 1)?), layout: input.u16()? },
        11 => Crossing::Record { name: input.text()?, layout: input.u16()?, fields: fields(input)? },
        12 => {
            let name = input.text()?;
            let count = input.count(10)?;
            let mut cases = Vec::with_capacity(count);
            for _ in 0..count {
                cases.push(CrossingCase { name: input.text()?, layout: input.u16()?, fields: fields(input)? });
            }
            Crossing::OneOf { name, cases }
        }
        13 => Crossing::Itself { name: input.text()? },
        _ => return Err(FileError::Damaged("a type that isn't one")),
    })
}

fn write_piece(piece: JoinPiece, out: &mut Writer) {
    match piece {
        JoinPiece::Constant(text) => {
            out.u8(0);
            out.u32(text);
        }
        JoinPiece::Text(register) => {
            out.u8(1);
            out.u16(register);
        }
        JoinPiece::Whole(register) => {
            out.u8(2);
            out.u16(register);
        }
        JoinPiece::Bool(register) => {
            out.u8(3);
            out.u16(register);
        }
        JoinPiece::Float(register) => {
            out.u8(4);
            out.u16(register);
        }
        JoinPiece::Places(register, places) => {
            out.u8(5);
            out.u16(register);
            out.u8(places);
        }
        JoinPiece::NumberPlaces(register, places) => {
            out.u8(6);
            out.u16(register);
            out.u8(places);
        }
        JoinPiece::Value(register, type_id) => {
            out.u8(7);
            out.u16(register);
            out.u32(type_id);
        }
    }
}

fn read_piece(input: &mut Reader) -> Result<JoinPiece, FileError> {
    Ok(match input.u8()? {
        0 => JoinPiece::Constant(input.u32()?),
        1 => JoinPiece::Text(input.u16()?),
        2 => JoinPiece::Whole(input.u16()?),
        3 => JoinPiece::Bool(input.u16()?),
        4 => JoinPiece::Float(input.u16()?),
        5 => JoinPiece::Places(input.u16()?, input.u8()?),
        6 => JoinPiece::NumberPlaces(input.u16()?, input.u8()?),
        7 => JoinPiece::Value(input.u16()?, input.u32()?),
        _ => return Err(FileError::Damaged("a piece of text that isn't one")),
    })
}

pub(crate) fn write_size(size: Size, out: &mut Writer) {
    out.u128(size.numerator);
    out.u128(size.denominator);
    out.u8(size.pi.cast_unsigned());
    out.flag(size.inexact);
}

pub(crate) fn read_size(input: &mut Reader) -> Result<Size, FileError> {
    Ok(Size {
        numerator: input.u128()?,
        denominator: input.u128()?,
        pi: input.u8()?.cast_signed(),
        inexact: input.flag()?,
    })
}

pub(crate) fn write_limits(limits: Limits, out: &mut Writer) {
    for edge in [limits.min, limits.max] {
        match edge {
            Some(edge) => {
                out.flag(true);
                out.u64(edge.bits);
                out.flag(edge.clamp);
                out.flag(edge.big);
            }
            None => out.flag(false),
        }
    }
    out.flag(limits.float);
    out.flag(limits.huge);
}

pub(crate) fn read_limits(input: &mut Reader) -> Result<Limits, FileError> {
    let mut edge = || -> Result<Option<Edge>, FileError> {
        Ok(if input.flag()? {
            Some(Edge { bits: input.u64()?, clamp: input.flag()?, big: input.flag()? })
        } else {
            None
        })
    };
    let (min, max) = (edge()?, edge()?);
    Ok(Limits { min, max, float: input.flag()?, huge: input.flag()? })
}

fn write_number(number: &Number, out: &mut Writer) {
    match number {
        Number::Huge(value) => {
            out.u8(0);
            value.write_file(out);
        }
        Number::Decimal(value) => {
            out.u8(1);
            out.flag(value.negative);
            out.u128(value.coefficient);
            out.u32(value.exponent.cast_unsigned());
        }
        Number::HugeDecimal(value) => {
            out.u8(2);
            value.coefficient.write_file(out);
            out.u64(value.exponent.cast_unsigned());
        }
    }
}

fn read_number(input: &mut Reader) -> Result<Number, FileError> {
    Ok(match input.u8()? {
        0 => Number::Huge(Big::read_file(input)?),
        1 => {
            let decimal =
                Decimal { negative: input.flag()?, coefficient: input.u128()?, exponent: input.u32()?.cast_signed() };
            // A `Decimal` keeps at most 34 digits inside its exponents (D116).
            if decimal.coefficient >= 10u128.pow(34)
                || !(crate::decimal::EXPONENT_MIN..=crate::decimal::EXPONENT_MAX).contains(&i64::from(decimal.exponent))
            {
                return Err(FileError::Damaged("a `Decimal` past what one holds"));
            }
            Number::Decimal(decimal)
        }
        2 => Number::HugeDecimal(HugeDecimal::new(Big::read_file(input)?, input.u64()?.cast_signed())),
        _ => return Err(FileError::Damaged("a number that isn't one")),
    })
}

fn write_conversion(conversion: NumberConversion, out: &mut Writer) {
    let kind = |kind: NumberType| match kind {
        NumberType::Whole => 0,
        NumberType::Float => 1,
        NumberType::Text => 2,
        NumberType::Huge => 3,
        NumberType::Decimal => 4,
        NumberType::HugeDecimal => 5,
    };
    out.u8(kind(conversion.from));
    out.u8(kind(conversion.to));
    match conversion.how {
        NumberHow::Exact => out.u8(0),
        NumberHow::Round(rounding) => out.u8(1 + rounding_number(rounding)),
        NumberHow::Shown => out.u8(6),
        NumberHow::Nearest => out.u8(7),
    }
}

fn rounding_number(rounding: Rounding) -> u8 {
    match rounding {
        Rounding::Floor => 0,
        Rounding::Trunc => 1,
        Rounding::Ceil => 2,
        Rounding::Nearest => 3,
        Rounding::NearestEven => 4,
    }
}

fn read_conversion(input: &mut Reader) -> Result<NumberConversion, FileError> {
    let kind = |byte: u8| -> Result<NumberType, FileError> {
        Ok(match byte {
            0 => NumberType::Whole,
            1 => NumberType::Float,
            2 => NumberType::Text,
            3 => NumberType::Huge,
            4 => NumberType::Decimal,
            5 => NumberType::HugeDecimal,
            _ => return Err(FileError::Damaged("a number type that isn't one")),
        })
    };
    let (from, to) = (kind(input.u8()?)?, kind(input.u8()?)?);
    let how = match input.u8()? {
        0 => NumberHow::Exact,
        1 => NumberHow::Round(Rounding::Floor),
        2 => NumberHow::Round(Rounding::Trunc),
        3 => NumberHow::Round(Rounding::Ceil),
        4 => NumberHow::Round(Rounding::Nearest),
        5 => NumberHow::Round(Rounding::NearestEven),
        6 => NumberHow::Shown,
        7 => NumberHow::Nearest,
        _ => return Err(FileError::Damaged("a rounding that isn't one")),
    };
    Ok(NumberConversion { from, to, how })
}

fn write_value(value: &Value, out: &mut Writer) {
    match value {
        Value::Nothing => out.u8(0),
        Value::Whole(whole) => {
            out.u8(1);
            out.u64(whole.cast_unsigned());
        }
        Value::Float(float) => {
            out.u8(2);
            out.u64(float.to_bits());
        }
        Value::Bool(flag) => {
            out.u8(3);
            out.flag(*flag);
        }
        Value::Text(text) => {
            out.u8(4);
            out.text(text);
        }
        Value::HugeWhole(value) => {
            out.u8(5);
            write_number(&Number::Huge(value.clone()), out);
        }
        Value::Decimal(value) => {
            out.u8(6);
            write_number(&Number::Decimal(*value), out);
        }
        Value::HugeDecimal(value) => {
            out.u8(7);
            write_number(&Number::HugeDecimal(value.clone()), out);
        }
        Value::List(items) | Value::Record(items) => {
            out.u8(if matches!(value, Value::List(_)) { 8 } else { 9 });
            out.count(items.len());
            for item in items {
                write_value(item, out);
            }
        }
        Value::Dict(pairs) => {
            out.u8(10);
            out.count(pairs.len());
            for (key, value) in pairs {
                write_value(key, out);
                write_value(value, out);
            }
        }
        Value::Maybe(inner) => {
            out.u8(11);
            match inner {
                Some(inner) => {
                    out.flag(true);
                    write_value(inner, out);
                }
                None => out.flag(false),
            }
        }
        Value::Case { case, fields } => {
            out.u8(12);
            out.u32(*case);
            out.count(fields.len());
            for value in fields {
                write_value(value, out);
            }
        }
    }
}

fn read_value(input: &mut Reader, depth: u32) -> Result<Value, FileError> {
    if depth > DEEPEST {
        return Err(FileError::Damaged("a value nested too deep"));
    }
    let list = |input: &mut Reader| -> Result<Vec<Value>, FileError> {
        let count = input.count(1)?;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(read_value(input, depth + 1)?);
        }
        Ok(items)
    };
    let number = |input: &mut Reader, wanted: u8| -> Result<Number, FileError> {
        let number = read_number(input)?;
        let kind = match number {
            Number::Huge(_) => 0,
            Number::Decimal(_) => 1,
            Number::HugeDecimal(_) => 2,
        };
        if kind == wanted { Ok(number) } else { Err(FileError::Damaged("a number of the wrong type")) }
    };
    Ok(match input.u8()? {
        0 => Value::Nothing,
        1 => Value::Whole(input.u64()?.cast_signed()),
        2 => Value::Float(f64::from_bits(input.u64()?)),
        3 => Value::Bool(input.flag()?),
        4 => Value::Text(input.text()?),
        5 => match number(input, 0)? {
            Number::Huge(value) => Value::HugeWhole(value),
            _ => return Err(FileError::Damaged("a number of the wrong type")),
        },
        6 => match number(input, 1)? {
            Number::Decimal(value) => Value::Decimal(value),
            _ => return Err(FileError::Damaged("a number of the wrong type")),
        },
        7 => match number(input, 2)? {
            Number::HugeDecimal(value) => Value::HugeDecimal(value),
            _ => return Err(FileError::Damaged("a number of the wrong type")),
        },
        8 => Value::List(list(input)?),
        9 => Value::Record(list(input)?),
        10 => {
            let count = input.count(2)?;
            let mut pairs = Vec::with_capacity(count);
            for _ in 0..count {
                pairs.push((read_value(input, depth + 1)?, read_value(input, depth + 1)?));
            }
            Value::Dict(pairs)
        }
        11 => Value::Maybe(if input.flag()? { Some(Box::new(read_value(input, depth + 1)?)) } else { None }),
        12 => Value::Case { case: input.u32()?, fields: list(input)? },
        _ => return Err(FileError::Damaged("a value that isn't one")),
    })
}

// ---- the verifier ----

/// trace: D141. Checks that every instruction fits its script: each register inside its procedure's frame, each jump
/// landing inside its own procedure, each table it reads from one the script has, and each procedure ending where it
/// can't run on into the next. The VM checks each of these again as it runs, so a script can never reach past itself
/// (Building rule 1); verifying refuses a file that would, while it loads, as a script with a mistake is refused.
///
/// # Errors
///
/// What doesn't fit, and where.
#[allow(clippy::too_many_lines)] // one case per kind of table an instruction reads
pub fn verify(compiled: &Compiled, door_count: usize) -> Result<(), String> {
    let code_length = compiled.code.len();
    if compiled.places.len() != code_length {
        return Err("its instructions and their places don't pair up".to_owned());
    }
    // Every procedure, and the top level, is a stretch of instructions from its entry to the next one's.
    let mut starts: Vec<(u32, u32)> = compiled.entries.iter().map(|entry| (entry.entry, entry.frame_size)).collect();
    starts.push((compiled.top_level_entry, compiled.top_level_frame_size));
    if compiled.entries.len() != compiled.procedures.len() {
        return Err("its procedures and their entries don't pair up".to_owned());
    }
    for (procedure, entry) in compiled.procedures.iter().zip(&compiled.entries) {
        if procedure.entry != entry.entry || procedure.frame_size != entry.frame_size {
            return Err(format!("`{}`'s entry isn't what the entries say", procedure.name));
        }
        if procedure.parameters.len() >= entry.frame_size as usize {
            return Err(format!("`{}` takes more values than its frame holds", procedure.name));
        }
    }
    starts.sort_unstable();
    starts.dedup_by_key(|(start, _)| *start);
    for &(start, frame) in &starts {
        if start as usize >= code_length || frame == 0 || frame > u32::from(u16::MAX) {
            return Err(format!("a procedure starts at {start}, outside its instructions, or has no frame"));
        }
    }
    let ends = starts.iter().skip(1).map(|&(start, _)| start as usize).chain(std::iter::once(code_length));
    for (&(start, frame), end) in starts.iter().zip(ends) {
        let start = start as usize;
        if end <= start {
            return Err(format!("a procedure at {start} has no instructions"));
        }
        // The last instruction can't run on into the next procedure.
        let last = compiled.code.get(end - 1);
        if !matches!(
            last,
            Some(
                Instruction::ReturnNothing
                    | Instruction::Return { .. }
                    | Instruction::ReturnAddWhole { .. }
                    | Instruction::Unreachable
                    | Instruction::Crash
                    | Instruction::Jump { .. }
            )
        ) {
            return Err(format!("the procedure at {start} can run past its end"));
        }
        for at in start..end {
            let Some(instruction) = compiled.code.get(at) else { continue };
            let wrong = |what: &str| format!("instruction {at}, {instruction:?}: {what}");
            for register in instruction_file::registers(*instruction) {
                if u32::from(register) >= frame {
                    return Err(wrong("a register outside its procedure's frame"));
                }
            }
            if let Some(offset) = instruction_file::jump(*instruction) {
                let offset = if offset == i32::from(FAR) {
                    *compiled
                        .far_jumps
                        .get(&u32::try_from(at).unwrap_or(u32::MAX))
                        .ok_or_else(|| wrong("a far jump with no distance"))?
                } else {
                    offset
                };
                let target = i64::try_from(at).unwrap_or(i64::MAX) + 1 + i64::from(offset);
                let (low, high) = (i64::try_from(start).unwrap_or(i64::MAX), i64::try_from(end).unwrap_or(i64::MAX));
                if target < low || target >= high {
                    return Err(wrong("a jump out of its procedure"));
                }
            }
            check_tables(*instruction, compiled, door_count).map_err(|what| wrong(&what))?;
            for (field, number) in instruction_file::numbered(*instruction).into_iter().flatten() {
                let length = match field {
                    "constant" => compiled.constants.len(),
                    // A failure's number counts from 1, with 0 for none.
                    "failure" => compiled.failure_names.len() + 1,
                    "global" => compiled.global_count as usize,
                    "procedure" => compiled.entries.len(),
                    _ => compiled.layouts.len(),
                };
                if number as usize >= length {
                    return Err(wrong(&format!("a {field} that isn't one of the script's")));
                }
            }
        }
    }
    for site in &compiled.door_sites {
        if site.door as usize >= door_count {
            return Err("a door the host doesn't open".to_owned());
        }
        let end = u64::from(site.first_argument) + u64::from(site.argument_count);
        if end > compiled.door_arguments.len() as u64 {
            return Err("a door's values past their list".to_owned());
        }
    }
    let type_count = compiled.types.len();
    for join in &compiled.joins {
        for piece in join {
            match *piece {
                JoinPiece::Constant(text) if text as usize >= compiled.texts.len() => {
                    return Err("a piece of text that isn't one of the script's".to_owned());
                }
                JoinPiece::Value(_, type_id) if type_id as usize >= type_count => {
                    return Err("a type that isn't in the script's table".to_owned());
                }
                _ => {}
            }
        }
    }
    if compiled.compared_types.iter().any(|&type_id| type_id as usize >= type_count) {
        return Err("a type that isn't in the script's table".to_owned());
    }
    for procedure in &compiled.procedures {
        let types = procedure.parameters.iter().map(|parameter| parameter.type_id).chain([procedure.result_type]);
        if types.into_iter().any(|type_id| type_id.0 as usize >= type_count) {
            return Err(format!("`{}` has a type that isn't in the script's table", procedure.name));
        }
    }
    if compiled.global_references.len() != compiled.global_count as usize {
        return Err("its top-level names and what they hold don't pair up".to_owned());
    }
    if compiled.built_in_failures.iter().any(|&failure| failure as usize > compiled.failure_names.len()) {
        return Err("a failure that isn't one of the script's".to_owned());
    }
    compiled.types.verify()
}

/// The tables an instruction reads from: each number that picks something out of one, in range.
fn check_tables(instruction: Instruction, compiled: &Compiled, door_count: usize) -> Result<(), String> {
    let within = |index: u32, length: usize, what: &str| {
        if (index as usize) < length { Ok(()) } else { Err(format!("{what} that isn't one of the script's")) }
    };
    match instruction {
        Instruction::LoadConstant { constant, .. } => within(constant, compiled.constants.len(), "a constant"),
        Instruction::LoadText { text, .. } => within(text, compiled.texts.len(), "a text"),
        Instruction::GetGlobal { global, .. } | Instruction::SetGlobal { global, .. } => {
            within(u32::from(global), compiled.global_count as usize, "a top-level name")
        }
        Instruction::Call { procedure, .. } => within(procedure, compiled.entries.len(), "a procedure"),
        Instruction::CallAdding { procedure, .. } => {
            within(u32::from(procedure), compiled.entries.len(), "a procedure")
        }
        Instruction::CallDoor { site, .. } => within(site, compiled.door_sites.len(), "a door's site")
            .and_then(|()| within(0, door_count.max(1), "a door")),
        Instruction::EqualValues { site, .. } => within(u32::from(site), compiled.compared_types.len(), "a comparison"),
        Instruction::LoadNumber { site, .. } => within(u32::from(site), compiled.numbers.len(), "a number"),
        Instruction::ConvertNumber { site, .. } => {
            within(u32::from(site), compiled.number_conversions.len(), "a conversion")
        }
        Instruction::CheckLimits { site, .. } => within(u32::from(site), compiled.limits.len(), "a type's limits"),
        Instruction::Scale { site, .. } => within(u32::from(site), compiled.scales.len(), "a unit's size"),
        Instruction::JoinPieces { site, .. } | Instruction::AppendPieces { site, .. } => {
            within(site, compiled.joins.len(), "a join")
        }
        Instruction::NewRecord { layout, .. } | Instruction::NewCase { layout, .. } => {
            within(u32::from(layout), compiled.layouts.len(), "a layout")
        }
        _ => Ok(()),
    }
}
