# Seal's implementation: progress

Kept by the building session ("seal claude"). After a compaction, read this, `test-status.json` and `git log` to carry on.

**The one command** for the whole build, lint and test run: `cargo run -j 3 --release -p seal-check-all`. It checks
formatting, lints with warnings as errors, builds, runs the packages' own tests, runs every test in `tests/` as
`tests/README.md` describes, and writes `test-status.json`.

## The finish line (from the prompt)

1. Four parts work: the `seal` library, the `seal` command, the fuzzer, the Unio translator with the comparison against
   Ralph. The language server is only an empty package.
2. D1 to D58 built (except the language server's parts), every test in `tests/` passing.
3. The host's side as D48 sketches, shown by an example host in the library's tests.
4. The fuzzer runs throughout.
5. The comparison with Ralph runs.
6. D57's performance goals met, recorded in `bench/RESULTS.md`.

## Checkpoints

Each ends with something running. `[x]` done, `[ ]` not yet.

- [x] **C0. Workspace.** Toolchain pinned (1.96.1), eight packages, Building rule 3's lints in each, the one command,
  `test-status.json`. All 14 tests fail (nothing runs yet).
- [x] **C1a. Reading.** Lexer and parser: every test script parses.
- [x] **C1b. Checking.** Names, `mut`/`unused`, types worked out (D5, D6), failures worked out (D44), pauses worked out
  (D52), every refusal.
- [x] **C1c. Running.** Bytecode compiler and register VM, with its optimizations from the start (see below). All 14
  tests pass (2026-11-23).
- [x] **C2. The host's side** (D48, D49, D51 to D54): `Doors`, `Script::load`, `call`, `resume`, `cancel`, `Limits`,
  `#[derive(Seal)]`, an example host in `crates/seal/tests/`.
- [x] **C3. The fuzzer** (`fuzz/`): random valid and invalid scripts against loading and running, under the limits.
  It runs in the one command (6,000 scripts, fixed seed) and by hand: `cargo run --release -p seal-fuzzer -- --seed N
  --seconds N`. Found so far: `div.floor (x)` and `x div.floor -5` misread by the lexer's spacing check (fixed).
- [x] **C4. The Unio translator and the comparison with Ralph** (D19): `crates/seal-to-unio`, and `seal-fuzzer --ralph`,
  which runs in the one command. Ralph has no `Float`, lists, `for each`, `.len`, dividing or the biggest `Whole`s built
  yet, so the comparison covers `Whole` maths and overflow, `Bool`, `Text`, `&`, `be.Text`, `be.Whole`, comparisons,
  logic, `if`, `while`, procedures and `show_msg`. 2 tests and 400 fuzzed scripts match; no difference found yet.
- [ ] **C5. Benchmarks** (`bench/`): the seven of D57 in D58's kinds, against Luau and Node; `bench/RESULTS.md`.
- [ ] **C6. Performance work** until D57's goals hold, then the final report.

## How it's built (the building session's design)

Packages: `crates/seal` (the library), `crates/seal-derive` (`#[derive(Seal)]`, re-exported by the library),
`crates/seal-command` (the `seal` command), `crates/seal-language-server` (empty), `crates/seal-to-unio` (the
translator), `crates/seal-check-all` (the one command), `fuzz/` (the fuzzer), `bench/` (the benchmark host).

Inside the library:

- `syntax/`: `lexer.rs` (text to lines of tokens), `tree.rs` (the syntax tree), `parser.rs`.
- `check/`: types, name resolution, working out types, failures and pauses; results in side tables keyed by each
  expression's `id`.
- `compile/`: the bytecode (`Instruction`, 8 bytes each) and the compiler from the checked tree.
- `vm/`: the heap, the dictionary, and the run loop.
- `host/`: `Doors`, `Script`, `Limits`, `Value`, the conversion traits.
- `message.rs`: messages in parts and as Unio-layout text (D26).

The VM, and where each optimization arrives (Building rule 5):

- **Untagged registers.** Types are static (D5, D6), so a register is a bare 64-bit slot: a `Whole`, a `Float`'s bits, a
  `Bool`, or a handle. Instructions are typed (`add_whole`, `add_float`), with no type checks while running (D46).
- **A heap of handles, with counted holders** (D15). Big values (text, lists, records, dictionaries, cases) live in a
  slab the script owns; a slot holds an index. No `Rc` (a script must move between threads, D49), no `unsafe`. Holders
  are counted by instructions the compiler emits; a value is copied only when written while shared (copy-on-write) and
  freed the moment its last holder ends.
- **Borrowed parameters and loop items.** A parameter is read-only, so passing a local costs no count. `for each` items
  are borrowed from the list, which can't change in the loop (Unio D173).
- **In-place change.** `x = x & y`, `list.add`, `record.field = v` and `dict[key: k] = v` change the value in place when
  it has one holder.
- **Superinstructions.** Compare-and-jump, arithmetic with a constant, `for each` over a `range` as one instruction.
- **Checks kept** (D17, D18): every `Whole` operation is checked for overflow, every `Float` operation for infinity and
  NaN, every position for range. A check is dropped only where proved unable to fail: constant folding of plain numbers,
  and a `range` counter, whose passes are counted once at the loop's start.
- **Steps** (D14, proposal): one step per backward jump and per call, plus one per 64 units of bulk work (copying a
  list, joining text, a long dictionary probe), so no single instruction does unbounded work for free.
- **Memory** (D14, D50, proposal): Seal's own sizes, counted on every allocation and growth: 8 bytes per slot of
  capacity, 1 per byte of text, plus a fixed header per heap value, and the tasks' stacks. Calls nesting too deep run
  into the memory cap; there's no separate depth limit.
- **Pausing** (D49): the VM's frames are data, not Rust stack, so a pause saves the task as it is and returns.

## Questions for the design session

Sent as they come up; answers get written here. Each is built as a "not designed yet" stop meanwhile.

First batch sent 2026-11-23 (delivered to "Unio & Seal Design Claude"); no answers yet. Each is a "not designed yet" stop:

1. A procedure with no parameters.
2. A parameter or a `for each` name that's never read (D2?). A case's value left unread is allowed (chapter 6's example).
3. Changing a part of a part, as `bodies[pos: i].vx = 1.0` or `goblin.items.add(x)`.
4. Comments, two blank lines in a row, spacing other than Unio's, indentation other than 4 spaces, tabs, `\r`, `\` in text.
5. Comparing two texts; taking text apart.
6. Running a procedure before a top-level name it uses is made; a pause, `return`, `fail` or `?` at the top level.
7. A value-giving procedure that can reach its `end`; a line after `return`, `break` or `fail`.
8. A `for each` over a top-level name whose body calls a procedure that changes it, or pauses while another task could.
9. A value-giving call standing alone; a line that only works out a value.
10. A `for each` or `case` name in a procedure spelt like a top-level name; a name spelt like a procedure, door or built-in.
11. A type that holds itself; a record directly inside a record.
12. The same plain key twice in a `Dict` literal; a list literal longer than its `len.max`; lists whose `len.max` differ.
13. `?? ignore` or `?? crash` on a `Maybe`; `instead` not last in its case; `[all: v]` with a name `all` in reach.
14. A host's record or `one_of` the script doesn't declare (built: the script must declare the same type).

Disagreements reported: chapter 9's "Not decided: labels and defaults on doors" against D51, D55 and tests/README.md
(built: labels on doors, no defaults); tests/d038_maybe's parameter sharing a top-level name against D56 (built the
test's way).

Proposals sent (D14 leaves them to the building session): a step is one jump taken or one call, plus one per 64 units of
bulk work; memory in Seal's own sizes; no default limits; a limit's kind is `limit reached:`; the `seal` command's exit
statuses follow Unio's; a Rust `HashMap`'s keys go in sorted.

Also asked for a `.gitignore` with `target/` (not a file the building session may edit).

## Draft wordings

Refusals and stops that are decided but have no wording yet, drafted in Unio's layout (Unio D54). To be listed here
from the code once checking is in.
