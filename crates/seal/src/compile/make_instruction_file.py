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
out.append('/// The registers an instruction names, in its fields\' order.')
out.append('#[must_use]')
out.append('pub fn registers(instruction: Instruction) -> Vec<Register> {')
out.append('    match instruction {')
for name, fields in variants:
    regs = [f for f, t in fields if t == 'Register']
    if regs:
        out.append(f'        Instruction::{name} {{ {", ".join(regs)}, .. }} => vec![{", ".join(regs)}],')
out.append('        _ => Vec::new(),')
out.append('    }')
out.append('}')
out.append('')
NUMBERED = ('constant', 'failure', 'global', 'procedure', 'layout')
out.append('/// The numbers an instruction picks things out of the script\'s tables with, each by its field\'s name.')
out.append('#[must_use]')
out.append('pub fn numbered(instruction: Instruction) -> Vec<(&\'static str, u32)> {')
out.append('    match instruction {')
for name, fields in variants:
    picked = [(f, t) for f, t in fields if f in NUMBERED and t != 'Register']
    if picked:
        names = ', '.join(f for f, _ in picked)
        items = ', '.join(f'("{f}", u32::from({f}))' if t != 'u32' else f'("{f}", {f})' for f, t in picked)
        out.append(f'        Instruction::{name} {{ {names}, .. }} => vec![{items}],')
out.append('        _ => Vec::new(),')
out.append('    }')
out.append('}')
out.append('')
out.append('/// Where an instruction can jump, as an offset from the instruction after it.')
out.append('#[must_use]')
out.append('pub fn jump(instruction: Instruction) -> Option<i32> {')
out.append('    match instruction {')
for name, fields in variants:
    for f, t in fields:
        if f == 'offset':
            value = 'offset' if t == 'i32' else 'i32::from(offset)'
            out.append(f'        Instruction::{name} {{ offset, .. }} => Some({value}),')
out.append('        _ => None,')
out.append('    }')
out.append('}')
out.append('')
open(OUT, 'w').write('\n'.join(out))
print(f'{len(variants)} instructions written to {OUT}')
