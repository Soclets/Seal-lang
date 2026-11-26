# Bytecode files (D141), as proposed

The building session's proposal for D141's file format, its verifier, and what verifying makes affordable, which
D141 leaves to it. Written 2026-11-25 by seal opus claude, and built on 2026-11-26 (`compile/file.rs`,
`host/compiled.rs`), with bundles and the cache that D153 and D154 added. Not built yet: following each register's kind
through the instructions; what's built checks every number in range, and the VM checks every value's kind as it runs.
PROGRESS.md's D141 entry says what's built.

## What a host and a game's build get

- **`seal build SCRIPT.seal`** reads, checks and compiles a script once, when the game is built, and writes
  `SCRIPT.sealc`, refusing the script with the same messages loading would give.
- **`Script::load_compiled(name, bytes, &doors, limits)`** loads one, in Rust and through the C API (D136). It verifies
  the file first, every time, and refuses one that doesn't pass, as a bad script is refused.
- **A script means the same either way** (D141): a file holds what loading the text would have compiled, nothing more.

## The file

All numbers little-endian, every list counted before its items, nothing pointing outside the file.

1. **A header:** `SEAL`, the format's number, and the Seal version that wrote it. Seal makes no compatibility promise
   (D137), so a file from any other version is refused with "rebuilt by this Seal's `seal build`", never guessed at.
2. **The doors it was built for:** each door's name, values, labels, units and result, as the host opened them. Loading
   refuses a file whose doors don't match the host's exactly, as loading text would refuse a call to a door that
   isn't there.
3. **The script's types:** the type table the checker made, records, `one_of`s, units and limits included.
4. **The compiled script:** instructions, constants, texts, procedures with their parameters, labels, defaults and
   failures, layouts, door sites, and the places of every instruction, for messages (D17).
5. **The source's name and a hash of its text**, so a message can say which file and that it may be stale.

## The verifier

One pass over each procedure, quick next to reading and checking text, run on every load:

- **Everything in range:** every register inside its procedure's frame, every constant, text, layout, site, type and
  procedure number one the file has, every jump landing on an instruction of its own procedure, and every procedure's
  last instruction one that can't fall off its end.
- **Every register's kind followed**, as WebAssembly's validator does: going through the instructions in order, and
  meeting at jumps, each register is known as a `Whole`, a `Float`, a `Bool`, a text, a list, a record of a given
  layout, and so on, or as not yet given. Each instruction must find the kinds it works on, and a heap value must be
  held and freed as the compiler's rules have it. A register whose kind differs where two ways meet can't be read.
- **Calls and doors agree with their declarations:** each call gives a procedure what its parameters take, and each
  door call what the door takes.

The VM already can't be made to harm its host by any instruction list (its registers, instructions and heap handles
are all checked, and its limits hold), so the verifier's job is to make sure a file means what its script meant: a
tampered or stale file is refused with a message, rather than running and going wrong in a way a script can't.

## What verifying makes affordable

- **Loading at the speed of loading the same text again:** about 0.16 µs for a patrolling script, rather than 4.4 µs
  from text, since nothing is read, checked or compiled.
- **Compiling harder,** at build time, where a millisecond doesn't matter: more instructions joined into one, as
  `CallAdding` is, and registers placed with more care.
- **Fewer runtime checks** where the verifier proves one can't fail, as D17 allows only with proof: a register's kind
  known for certain needs no check of it.

## Questions it leaves for Tankun

1. The file's extension: `.sealc`, as proposed, for "Seal, compiled".
2. Whether `seal build` writes one file per script, as proposed, or can bundle a game's scripts into one file. The
   proposal: one per script now, a bundle when a game asks for it.
