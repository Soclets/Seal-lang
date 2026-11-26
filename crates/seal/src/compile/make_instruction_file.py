# Writes `instruction_file.rs` from `bytecode.rs`'s `Instruction`: how each instruction is written into a `.sealc`
# file and read back (D141, D153), and which of its fields are registers and jumps, for the verifier. Run it after
# changing `Instruction`, from the repository: python3 -I crates/seal/src/compile/make_instruction_file.py
# An instruction added without running it is a compile error in `instruction_file.rs`, whose matches are exhaustive.
import re
import sys

SOURCE = 'crates/seal/src/compile/bytecode.rs'
OUT = 'crates/seal/src/compile/instruction_file.rs'

text = open(SOURCE).read()
body = text[text.index('pub enum Instruction {'):]
body = body[:body.index('\n}\n')]
variants = []
current = None
for line in body.splitlines()[1:]:
    stripped = line.strip()
    if not stripped or stripped.startswith('///') or stripped.startswith('//') or stripped.startswith('#['):
        continue
    m = re.match(r'^    ([A-Z][A-Za-z]*)\s*\{$', line)
    if m:
        current = (m.group(1), [])
        variants.append(current)
        continue
    m = re.match(r'^    ([A-Z][A-Za-z]*),$', line)
    if m:
        variants.append((m.group(1), []))
        current = None
        continue
    m = re.match(r'^        ([a-z_]+): ([A-Za-z0-9]+),$', line)
    if m and current is not None:
        current[1].append((m.group(1), m.group(2)))
        continue
    if line.strip() == '},':
        current = None
        continue
    sys.exit(f'make_instruction_file.py: a line it can\'t read: {line!r}')

if len(variants) > 255:
    sys.exit('make_instruction_file.py: more than 255 instructions')

WRITE = {
    'Register': 'out.u16({})', 'u8': 'out.u8({})', 'u16': 'out.u16({})', 'u32': 'out.u32({})',
    'i8': 'out.u8({}.cast_unsigned())', 'i16': 'out.u16({}.cast_unsigned())', 'i32': 'out.u32({}.cast_unsigned())',
    'bool': 'out.u8(u8::from({}))', 'Compare': 'out.u8({} as u8)',
}
READ = {
    'Register': 'input.u16()?', 'u8': 'input.u8()?', 'u16': 'input.u16()?', 'u32': 'input.u32()?',
    'i8': 'input.u8()?.cast_signed()', 'i16': 'input.u16()?.cast_signed()', 'i32': 'input.u32()?.cast_signed()',
    'bool': 'input.flag()?', 'Compare': 'input.compare()?',
}

out = []
out.append('//! Made by `make_instruction_file.py` from `bytecode.rs`\'s `Instruction`; run it again after changing')
out.append('//! `Instruction`, never edit this by hand. How each instruction is written into a `.sealc` file and read back')
out.append('//! (D141, D153), and which of its fields are registers and jumps, for the verifier.')
out.append('// Each instruction\'s fields are written and read in the order they\'re declared.')
out.append('#![allow(clippy::too_many_lines, clippy::match_same_arms)]')
out.append('')
out.append('use super::bytecode::{Instruction, Register};')
out.append('use super::file::{Reader, Writer, FileError};')
out.append('')
out.append('/// Writes an instruction: its number, then its fields.')
out.append('pub fn write(instruction: Instruction, out: &mut Writer) {')
out.append('    match instruction {')
for number, (name, fields) in enumerate(variants):
    names = ', '.join(f for f, _ in fields)
    pattern = f'Instruction::{name} {{ {names} }}' if fields else f'Instruction::{name}'
    writes = ' '.join(WRITE[t].format(f) + ';' for f, t in fields)
    out.append(f'        {pattern} => {{')
    out.append(f'            out.u8({number});')
    if writes:
        out.append(f'            {writes}')
    out.append('        }')
out.append('    }')
out.append('}')
out.append('')
out.append('/// Reads an instruction back.')
out.append('///')
out.append('/// # Errors')
out.append('///')
out.append('/// When the file ends, or the number isn\'t an instruction\'s.')
out.append('pub fn read(input: &mut Reader) -> Result<Instruction, FileError> {')
out.append('    Ok(match input.u8()? {')
for number, (name, fields) in enumerate(variants):
    if fields:
        reads = ', '.join(f'{f}: {READ[t]}' for f, t in fields)
        out.append(f'        {number} => Instruction::{name} {{ {reads} }},')
    else:
        out.append(f'        {number} => Instruction::{name},')
out.append('        _ => return Err(FileError::Damaged("an instruction that isn\'t one")),')
out.append('    })')
out.append('}')
out.append('')
MOST_REGISTERS = max(sum(1 for _, t in fields if t == 'Register') for _, fields in variants)
NUMBERED = ('constant', 'failure', 'global', 'procedure', 'layout')
MOST_NUMBERED = max(sum(1 for f, t in fields if f in NUMBERED and t != 'Register') for _, fields in variants)
out.append('/// The most registers one instruction names.')
out.append(f'pub const MOST_REGISTERS: usize = {MOST_REGISTERS};')
out.append('')
out.append('/// The most table numbers one instruction holds.')
out.append(f'pub const MOST_NUMBERED: usize = {MOST_NUMBERED};')
out.append('')
out.append('/// The script\'s tables an instruction picks things out of by number.')
out.append('#[derive(Clone, Copy, Debug, PartialEq, Eq)]')
out.append('pub enum Table {')
for f in NUMBERED:
    out.append(f'    {f.capitalize()},')
out.append('}')
out.append('')
out.append('/// What the verifier checks of an instruction, found in one look at it (D173): the registers it names, in its')
out.append('/// fields\' order, then register 0, which every frame has, for the rest; the numbers it picks things out of the')
out.append('/// script\'s tables with; and where it can jump, as an offset from the instruction after it.')
out.append('#[derive(Clone, Copy, Debug)]')
out.append('pub struct Facts {')
out.append('    pub registers: [Register; MOST_REGISTERS],')
out.append('    pub numbered: [Option<(Table, u32)>; MOST_NUMBERED],')
out.append('    pub jump: Option<i32>,')
out.append('}')
out.append('')
out.append('/// An instruction\'s [`Facts`].')
out.append('#[must_use]')
out.append('pub fn facts(instruction: Instruction) -> Facts {')
out.append('    match instruction {')
for name, fields in variants:
    regs = [f for f, t in fields if t == 'Register']
    picked = [(f, t) for f, t in fields if f in NUMBERED and t != 'Register']
    jump = [(f, t) for f, t in fields if f == 'offset']
    used = regs + [f for f, _ in picked] + [f for f, _ in jump]
    if not used:
        continue
    pattern = f'Instruction::{name} {{ {", ".join(used)}, .. }}'
    registers = '[' + ', '.join(regs + ['0'] * (MOST_REGISTERS - len(regs))) + ']'
    items = [f'Some((Table::{f.capitalize()}, u32::from({f})))' if t != 'u32' else f'Some((Table::{f.capitalize()}, {f}))' for f, t in picked]
    numbered = '[' + ', '.join(items + ['None'] * (MOST_NUMBERED - len(picked))) + ']'
    jumped = 'None'
    if jump:
        f, t = jump[0]
        jumped = f'Some({f})' if t == 'i32' else f'Some(i32::from({f}))'
    out.append(f'        {pattern} => Facts {{ registers: {registers}, numbered: {numbered}, jump: {jumped} }},')
out.append(f'        _ => Facts {{ registers: [0; MOST_REGISTERS], numbered: [None; MOST_NUMBERED], jump: None }},')
out.append('    }')
out.append('}')
out.append('')
open(OUT, 'w').write('\n'.join(out))
print(f'{len(variants)} instructions written to {OUT}')
