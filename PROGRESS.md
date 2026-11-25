# Seal's implementation: progress

Kept by the building session: "seal claude" built D1 to D73; "seal opus claude" carries it on from 2026-11-25. After a
compaction, read this, `test-status.json` and `git log` to carry on.

**The one command** for the whole build, lint and test run: `cargo run -j 3 --release -p seal-check-all`. It checks
formatting, lints with warnings as errors, builds, runs the packages' own tests, runs the fuzzer and the comparison
with Ralph, runs every test in `tests/` as `tests/README.md` describes, and writes `test-status.json`.

**The benchmarks:** `sh bench/run.sh` builds Seal's benchmark host in each way D58's kinds need, runs every benchmark
on Seal, Luau and Node, checks every answer, and writes `bench/RESULTS.md`.

## The finish line (from the prompt), and where each point stands

1. **Four parts work.** The `seal` library, the `seal` command, the fuzzer, and the Unio translator with the
   comparison against Ralph all work. The language server is an empty package.
2. **Decisions built.** D1 to D73 are built, except the language server's parts and what's listed under "Not built
   yet" below. All 14 tests in `tests/` pass.
3. **The host's side** is built as D48 sketches, with an example game host in `crates/seal/tests/host.rs`.
4. **The fuzzer** runs in the one command and by hand, and has never been left failing.
5. **The comparison with Ralph** runs in the one command. No difference found.
6. **Performance goals.** See `bench/RESULTS.md` for the numbers and "Benchmarks" below for what's met and what isn't.

## Checkpoints

- [x] **C0. Workspace.** Toolchain pinned (1.96.1), eight packages, Building rule 3's lints in each, the one command.
- [x] **C1. Reading, checking, running.** Lexer, parser, checker, bytecode compiler, register VM.
- [x] **C2. The host's side** (D48, D49, D51 to D54): `Doors`, `Script::load`, `call`, `resume`, `cancel`, `Limits`,
  `#[derive(Seal)]`.
- [x] **C3. The fuzzer** (`fuzz/`): random valid, changed and nonsense scripts against loading and running, under the
  limits, with a check that nothing leaks and that every value's count of holders is right. By hand:
  `cargo run --release -p seal-fuzzer -- --seed N --count N`.
- [x] **C4. The Unio translator and the comparison with Ralph** (D19): `crates/seal-to-unio`, and
  `seal-fuzzer --ralph`.
- [x] **C5. Benchmarks** (`bench/`): D57's seven and binary-trees, which D72 let join, in D58's kinds, against Luau
  and Node; `bench/RESULTS.md`.
- [x] **C6. Performance work.** Every goal of D57 is met, Normal against Normal. See "Benchmarks" and
  `bench/RESULTS.md`.

## Plan from 2026-11-25 (seal opus claude): D74 to D118

Baseline before any change: every one of the 15 tests failed (they're in today's spellings), one unit test failed, and
all 401 Ralph comparisons differed (Ralph reads only Unio's new spellings). The benchmarks were re-run on the old
build: `bench/RESULTS.md` as of that run.

Each checkpoint ends with something running, committed and pushed, the benchmarks re-run when it touched the reader,
checker, compiler or VM.

- [x] **C7. Today's spellings.** Bare `end` with a checked `\\` note (D102); family members without `proc` (D97);
  header types required (D81), `! [ … ]` required and exact (D84), grouped parameters (D88); every text fills its
  braces, `fmt` and `&` go (D75, D77); `{newln}` and `{tab}` (D118). Every test passes again; the benchmark programs
  are rewritten in today's spellings. Every D57 goal still met; most programs got faster (see "Benchmarks").
- [ ] **C8. The smaller decisions.** D79, D80, D82, D83, D85, D86, D87, D89 to D96, D99 to D101, D103 to D106: each
  stop the code had becomes the decided refusal or feature, `==` on bigger values (D95), a `range` as a list (D99),
  every value shown as written (D100), `places:` (D94), `be.Bool` (D101).
- [x] **C9. The translator writes Unio's current spellings** (Unio D190 to D203) and the comparison with Ralph runs
  again: 404 the same, 0 different, from seed 1, with 4 of the 15 tests compared.
- [ ] **C10. Handlers** (D97, D98, D110): `group.handler`, `::name`, `handles [ … ]`, `??` with a handler, `include`,
  `*`, parameters.
- [ ] **C11. Maths:** ranged numbers and `abs`, `min`, `max`, `clamp` (D107), `**` (D108), `sin` to `atan2` and `pi`
  (D109), `lerp` (D114), each correctly rounded (D50).
- [ ] **C12. Units** (D111 to D113), once the design session has answered how Cyborg's units work in Seal.
- [ ] **C13. `HugeWhole`, `Decimal`, `HugeDecimal`** (D115, D116), timed in the benchmarks.
- [ ] **C14. Finish:** the fuzzer covers every new feature; the benchmarks re-run; the draft wordings brought up to date
  and sent to the design session.

## Decisions built since D58

Each arrived while building; each is built, tested, fuzzed and pushed.

- **D59** `==` and `!=` kept: nothing to change.
- **D60** A procedure that takes nothing has no brackets and is called with `()`, by the script and by the host.
- **D61** Unread parameters and `for each` names take `unused`; a `case` takes out only the values it reads.
- **D62** A `mut` name is changed through any depth of fields and places, with `.add` or `.remove` at the end. Each
  step along the way is made singly held where it sits, so the change is in place and a copy held elsewhere stays as
  it was. A `Dict` key partway along stops as not designed yet.
- **D63** Comments (`//`, `///`, `comment name` … `end`), blank lines, indentation and spacing refused with the
  corrected line as the fix (one message a line, a `^` under each gap), `\r\n`, and `\` in text.
- **D64** `==` and `!=` on two texts; `<` and `>` on text stop as not designed yet.
- **D65, D66** Text counts and is reached by graphemes: `.len`, `.len.egc`, `.len.ucp`, `.len.bytes`, `[pos:]`,
  `[offset:]`, `for each`, `.split(by:)` and `.split.no_empty(by:)`. Unicode's tables come from the crate
  `unicode-segmentation`, which has no `unsafe`; its version is pinned exactly, so this Seal version pins Unicode 17.
- **D67** At the top level: reaching a name before it's made, pausing, `return`, `fail` and `?` are refused.
- **D68** Every way through a value-giving procedure ends in a `return`; nothing follows a leaving line or an `if`
  that can leave; a bare `return` can't be a procedure's last line.
- **D69** A `for each` refuses a call that can change what it goes through, and a pause while going through a
  top-level name.
- **D70** A value-giving call can't stand alone, nor a line that only works out a value.
- **D71** No shadowing: a spelling that already means something can't be taken again.
- **D72** Records inside records, and types that hold themselves where a value can end. A type whose values could
  never end is refused. Freeing a deep value never nests in the host's stack.
- **D73** A list moves freely into a `len.max` at least as big; into a smaller one it can fail with `full`. A list
  literal longer than its `len.max`, and a key written twice in a `Dict` literal, are refused.

## Decisions built since D73

- **D74** `record` stays: nothing to change.
- **D75, D77** Every text fills its braces; `fmt` and `&` go. `&` is refused, and so is `+` on text, with the braces
  form as the fix when the pieces are texts and names. Braces fill in place as `&` did: `text = "{text}…"` joins onto
  the text in place.
- **D81** Every parameter writes its type, and a procedure giving back a value writes `-> Type`; a `return` with a
  value in a header without one is refused, with the header put right as the fix. Names inside bodies still take
  their type from their value.
- **D84** A procedure that can fail writes `! [ … ]`, exactly as its body fails; the fix is the header put right.
- **D85** A host opens its records and `one_of`s: a door's types with the door, and others with `Doors::add_type`.
  A script uses them by name; declaring one is refused.
- **D88** Parameter groups, `[hp, amount : Whole]`; a group's `:` without its spaces, or a single name's with them,
  is refused with the respaced header as the fix. The lexer lets a spaced `:` through only inside a header's `[ ]`.
- **D91** `x = 1// hi` is a spacing mistake; comment lines inside brackets are fine; a comment line is indented like
  the line after it, or one level in before `end`, `elif` or `else` (Unio D145); `comment` is a keyword; `///` goes
  before a top-level name, a field, a case, a procedure, a family, a record or a `one_of`.
- **D92, D93** The old stops become refusals: `\` before other characters, tabs, empty braces, a procedure named
  after a built-in or door, declarations inside blocks, `_` in a type's name, leading zeros, letters beyond plain
  English, blank lines inside brackets.
- **D94** `{speed, places: 2}`, rounded from the `Float`'s exact value, ties away from zero, no minus on zero.
- **D97** A family's members are written without `proc`; `proc` inside a family is refused with the fix.
- **D100** Every value shows as it's written: lists, arrays, `Dict`s, records, `one_of`s, `Maybe`s, text in quotes
  with `"`, `\`, `{`, line breaks and tabs written as a script would. Showing walks the value without nesting in the
  host's stack and stops at the memory cap.
- **D102** Bare `end`, with an optional `\\ note` checked against the block it closes, block comments included.
- **D106** A statement's later lines are indented one level deeper than its first.
- **D118** `{newln}` and `{tab}`; `newln` and `tab` can't be names. Outside braces they stop as not designed yet.

## How it's built (the building session's design)

Packages: `crates/seal` (the library), `crates/seal-derive` (`#[derive(Seal)]`, re-exported by the library),
`crates/seal-command` (the `seal` command), `crates/seal-language-server` (empty), `crates/seal-to-unio` (the
translator), `crates/seal-check-all` (the one command), `fuzz/` (the fuzzer), `bench/` (the benchmark host).

Inside the library:

- `syntax/`: `lexer.rs` (text to lines of tokens, with spacing, comments and blank lines), `tree.rs`, `parser.rs`.
- `check/`: types, names, working out types, failures and pauses; the checked tree.
- `compile/`: the bytecode (`Instruction`, 8 bytes each) and the compiler from the checked tree.
- `vm/`: the heap, the dictionary, crossing values to and from the host, and the run loop.
- `host/`: `Doors`, `Script`, `Limits`, `Value`, the conversion traits.
- `text.rs`: graphemes, code points and bytes (D65, D66).
- `message.rs`: messages in parts and as Unio-layout text (D26).

The VM, and its optimizations (Building rule 5):

- **Untagged registers.** Types are static (D5, D6), so a register is a bare 64-bit slot, and instructions are typed,
  with no type checks while running (D46).
- **A heap of handles, with counted holders** (D15). No `Rc`, no `unsafe`. A value is copied only when written while
  shared (copy-on-write), and freed the moment its last holder ends.
- **Borrowed loop items and case values; arguments handed over.** `name = f(name)` hands the value over with no copy.
- **In-place change.** `x = x & y`, `list.add`, `record.field = v`, `dict[key: k] = v`, and D62's deeper changes change
  the value in place when it has one holder.
- **Superinstructions.** Compare-and-jump, arithmetic with a constant, `for each` over a `range` as one instruction, a
  `Dict` read-then-set as one lookup, joining text in one step, a field of a list's record read or changed in one
  step (`bodies[pos: i].vx`), and two or three float operations in one step, each checked as on its own.
- **A list item's fields kept in registers** through a run of plain statements (ones that only work out numbers and
  change such fields): read into registers at the first use, changed there, written back once when the run ends.
  Two places that could be the same item are never both kept with a change waiting, unless a range says they can't
  be the same, as `for each j in range[from: i + 1, …]` does.
- **A `mut` record of plain numbers kept in registers**, written back in one step.
- **Small procedures written out in place** of a call to them.
- **Rare, big instructions live outside the loop.** Joining text is a function of its own, never inlined: with it
  inside the loop, the loop's machine code got worse for everything, `fib` 18% slower on the same bytecode. Kept
  out, `fib` is 6% faster than before, and a join costs about 3 ns more (one call).
- **The host's side:** a script's compiled form is kept by its doors and shared by every script loaded from the same
  text; plain numbers cross a door or a host call as their bare bits; a finished call's room is reused; a paused task
  is resumed where it sits.
- **Checks kept** (D17, D18): every `Whole` operation is checked for overflow, every `Float` operation for infinity and
  NaN, every position for range. `bench/RESULTS.md` has what the float check costs, measured.
- **Steps** (D14, proposal): one step per jump taken and per call, plus one per 64 units of bulk work.
- **Memory** (D14, D50, proposal): Seal's own sizes, counted on every allocation and growth.
- **Pausing** (D49): the VM's frames are data, not Rust stack, so a pause saves the task as it is and returns.

## Benchmarks

`bench/RESULTS.md` has every number. After C7 (2026-11-25), Normal: `fib` 52.1 ms (was 55.5), n-body 66.0 ms (67.6),
spectral-norm 19.2 ms (21.7), fannkuch 113 ms (120), word count 24.4 ms (25.9), text building 335 µs (287), binary-trees
114.5 ms (115.2), patrol run 2.10 ms (2.08). Loading a 12-line script from a text of its own is about 10.2 µs (9.7):
the reader keeps the script's text for the fixes it shows, and checks comment lines' indentation (D91).

In short, judged Normal against Normal (D58):

- **Faster than Luau's interpreter:** met on all eight as the benchmarks are defined: `fib`, n-body, spectral-norm,
  fannkuch, word count, text building, binary-trees, and the patrolling scripts as a whole run (1,000 scripts loaded
  from one text, each resumed 100 times), with resuming and footprint each better than Luau's.
- **Where Seal is behind Luau: loading a script.** Timed apart from the rest of the patrol run, loading is slower in
  Seal. From the same text, where Seal reuses what it compiled and Luau compiles once and makes each script from
  that, a script takes Seal about 225 ns and Luau about 195 ns. From a text of its own, with nothing reused, a
  12-line script takes Seal about 10 µs and Luau about 5 µs (Node about 1.2 µs, which puts most of its compiling off
  until a function first runs). Seal reads, type-checks and compiles the whole script before it runs any of it, and
  the time goes on many small allocations spread across the reader, the checker and the compiler; no one place
  dominates. Tried: the reader now reuses one buffer for every line and gives each line's tokens their room at once
  (14 µs to 10 µs). Closing the rest needs the reader and checker to stop allocating for each word and each node,
  which is a rework, not a fix. A 10,000-line script still loads in 8 ms, far inside goal 4, so this only shows on
  very small scripts loaded in great numbers from different texts.
- **n-body, and what D62 changed.** Until D62, a body's fields could only be changed by copying the body out, changing
  it and putting it back. With D62 the plain way is `bodies[pos: i].vx = bodies[pos: i].vx - …`, so that's now the
  Normal program, and the copy-out one is kept as `nbody.ceil.seal`, the Ceil.source program. Written the plain way it
  first took 167 ms against Luau's 70 ms. Three things brought it under Luau's time, each general, none knowing the
  benchmark: a field of a list's record read or changed in one instruction (106 ms); a list item's fields kept in
  registers through a run of plain statements, read once and written back once (74 ms); and two or three float
  operations joined into one instruction, each still checked as on its own (67 ms). The margin over Luau is small,
  about 4%.
- **The fuzzer checks those shortcuts:** it compiles every script a second, plain way, with none of them, and the two
  must give the same output and the same messages. Breaking either shortcut on purpose was caught at once.
- **Faster than V8 `--jitless` on `fib` and n-body, within twice on word count and text building:** met.
- **A 10,000-line script loads in under 100 ms:** met, about 9 ms.
- **Footprint:** a paused patrolling script holds 512 bytes of its cap, about 1.8 KB of real memory.
- **Pausing costs about as much as a call:** met; resuming is cheaper than a host call.
- **Ceil.program and Ceil.machine.program** need `llvm-profdata` to merge the profile. Rust's own isn't installed,
  and Tankun was asked on 2026-11-23 whether it may be; meanwhile the one that comes with Xcode turned out to read
  Rust's profiles, so nothing needed installing, and `bench/run.sh` uses whichever is there. One profile is taken
  from all eight benchmarks' Normal runs, and every Ceil.program row uses the VM built with it.
- **Ceil.source programs:** only n-body has one. For the others the Normal program stands in, and each such row says so;
  the Benchmarks Game's hand-tuned Lua and JavaScript programs weren't fetched.

## Questions for the design session

The first batch of 14 went on 2026-11-23. Answered so far: 1 (D60), 2 (D61), 3 (D62), 4 (D63), 5 (D64 to D66),
6 (D67), 7 (D68), 8 (D69), 9 (D70), 10 (D71), 11 (D72), 12 (D73). Still open from that batch, each a "not designed
yet" stop:

13. `?? ignore` or `?? crash` on a `Maybe`; `instead` not last in its case; `[all: v]` with a name `all` in reach.
14. A host's record or `one_of` the script doesn't declare (built: the script must declare the same type).

New questions the new decisions raised, each built as a stop or noted beside what was built:

- **D61.** `unused` on a `for each`'s `at` name (stops). A `case` taking its values out in another order than they're
  declared (stops).
- **D62.** A `Dict` key partway along a change (stops, as D62 says).
- **D63.** A `//` comment touching what's before it, and a comment inside an open bracket (both stop). A `//` line's
  own indentation isn't checked. A block comment closes at the first `end` at its own depth, so commented-out code
  must be indented inside it. `comment` isn't a reserved word: a name can be spelt `comment`. A `///` before a name,
  a field or a case (stops). A `\` before anything but `"`, `\` or `{`, and a tab inside text (both stop).
- **D65.** Splitting by an empty text (stops the script, as not designed yet, when it happens).
- **D68.** Does Unio's D9 cover a `match` where some case leaves? (A line after one stops.) Built: a `return` or `fail`
  anywhere inside an `if`, loops included, counts as a way that leaves; `?` doesn't count.
- **D69.** Built as written: pausing is refused while going through any top-level name, `mut` or not.
- **D72.** Passing a value of a type that holds itself between a host and a script (stops as not built yet): the
  host's side needs a way to name a type inside itself, and a deep value mustn't overrun the host's stack.
- **D73.** Built wherever a list goes where a type is wanted: a name, an argument, a field, a `return`.
- **D58.** Is n-body's Normal program rightly the one written with D62's direct changes, with the copy-out one as
  Ceil.source? (Built that way; both beat Luau.)

Disagreements reported: chapter 9's "Not decided: labels and defaults on doors" against D51, D55 and tests/README.md
(built: labels on doors, no defaults).

Proposals sent (D14 leaves them to the building session): a step is one jump taken or one call, plus one per 64 units of
bulk work; memory in Seal's own sizes; no default limits; a limit's kind is `limit reached:`; the `seal` command's exit
statuses follow Unio's; a Rust `HashMap`'s keys go in sorted.

Also asked for a `.gitignore` with `target/` (not a file the building session may edit).

## What Ralph can't compare yet

Ralph has no `Float`, lists, `for each`, `.len`, dividing or the biggest `Whole`s built yet, so the comparison covers
`Whole` maths and overflow, `Bool`, `Text`, `&`, `be.Text`, `be.Whole`, comparisons, logic, `if`, `while`, procedures
(with and without parameters), comments and `show_msg`. 2 of the 14 tests and every fuzzed script match. Seal's
dialect alone has: comparing texts (D64), text by graphemes (D65, D66), `unused` parameters (D61), deeper changes
(D62), and everything with records, `one_of`, `Dict` and failures.

## Not designed yet: every stop left in the code

- arguments written in another order than the parameters
- empty braces in `fmt` text; `places:` in `fmt`'s braces
- converting with `be` to anything but `Text`, `Whole` or `Float`, or from a type no conversion is decided for
- a `range[…]` anywhere but after `in` in a `for each`
- a name spelt `all` as a dictionary's key, which reads like `[all: value]`
- `?? ignore` or `?? crash` on a `Maybe`
- an `instead` that isn't the last line of its case
- putting two texts in order, as `<` and `>` would; comparing two records, lists, dictionaries or `one_of`s with `==`
- showing anything but a `Whole`, a `Float`, a `Bool` or a `Text` in `fmt`'s braces, or converting it to text
- a `one_of` with no cases
- a type's setting that isn't a plain whole number
- a `Dict` key that isn't a `Text`, a `Whole` or a `Bool`
- a procedure named after a built-in or one of the host's doors
- a default that isn't a number, a `Bool` or text
- taking out a case's values in another order than they're declared
- a line after a `match` where some case leaves
- a `Dict`'s key partway along a change
- `at pos` or `at offset` when going through a `Dict`
- using the host's record or `one_of` without declaring it in the script
- a comment inside an open bracket; a `//` comment touching what's before it
- a `///` before anything but a procedure, a family, a record or a `one_of`
- a tab outside indentation; a character beyond plain English ones outside text
- a `\` in text before anything but `"`, `\` or `{`; a tab inside text; a `\r` that isn't right before a line break
- a `_` in a type's name; a number written with a leading zero
- `fam.handler` and a handler after `otherwise`, which come with handlers (D43)
- a `record`, `one_of`, `proc` or `fam.proc` inside a block
- `unused` on a `for each`'s `at` name
- a blank line inside an open bracket; how a statement's later lines are indented (not checked)
- splitting a text by an empty text

## Not built yet

- Working out a type from a later line (D5) in one spot: a change at a place, `name[pos: n] = v`, on a name whose
  type isn't known by then, since a place doesn't say whether it's a list or an array. (A `??`, a `match`, `.add`,
  `.remove`, and a change by field or by key now say what the type must be, and it's worked out there.)
- More than 65,000 top-level names used by procedures, registers in one procedure, or constants.
- Blocks nested more than 48 deep, and values nested more than 160 deep in one statement (bounded so that reading a
  script can't overrun the host's stack).
- Passing a value of a type that holds itself between a host and a script (D72).
- The language server (D47): an empty package, as the prompt says.

## Draft wordings

Refusals and failed checks whose wording no decision gives, drafted in Unio's layout (Unio D54): `what:` says what's
wrong, `why:` gives the rule once. Where a fix can be shown as a whole line, `fix:` stands in place of `what:` (Unio
D101), as for indentation and spacing.

240 wordings, taken from the code by a script on 2026-11-23. `{…}` stands for what the message fills in.


**`crates/seal/src/check/calls.rs`**

- refused: this isn't a procedure, so it can't be called
  - why: `( )` calls a procedure.
- refused: this is a chain of {length} calls, each inside the next, and three or more are written as a slide
  - why: a slide reads in the order it runs, as in `points |> double |> be.Text |> show_msg`.
- refused: `{word}` is a name, not a procedure, so it can't be called
  - why: `( )` calls a procedure.
- refused: `sqrt` takes a `Float`, but this is {found}
  - why: `sqrt` gives a `Float`'s square root; a number written with a point, as `9.0`, is a `Float`.
- refused: a `range` is written with `[ ]`, as in `range[from: 1, to: 3]`
  - why: `( )` calls a procedure, and a range runs nothing.
- refused: there's no procedure or door named `{word}`
- refused: `{word}` is only ever one of its kinds
- refused: `{word}` has no narrowing `.{narrowing}`
  - why: `{word}` has no narrowings.
- refused: `{word}` has no narrowing `.{narrowing}`, so there's no `{full}`
- refused: `{name}` has no narrowing `.{narrowing}`
  - why: `{name}` can't be narrowed further.
- refused: `{name}` takes one value, with no label
- refused: `{name}` has no label `{}:`
  - why: a call writes the labels its procedure declares, and no others.
- refused: this argument needs its label, `{label}:`
  - why: a call writes every label its procedure declares.
- refused: `{name}` doesn't take this many arguments
- refused: this argument is given twice
- refused: this call leaves out `{label}:`
  - why: a call writes every label its procedure declares, unless the label has a default.
- refused: `{name}` takes {} argument{}, but this call gives {}
- refused: `fmt` takes its text written out in quotes
  - why: the names in `fmt`'s braces are checked when the script loads, so the text has to be there to read.
- refused: `{{{inside}}}` isn't a name with its fields and facts
  - why: `fmt`'s braces hold a name, which may be followed by its fields and facts, never a calculation or a call.
- refused: there's no name `{first}` here
  - why: a name in `fmt`'s braces is checked when the script loads.
- refused: `be` is narrowed by the type the value becomes, as in `be.Whole`
  - why: `be.Type(value)` converts a value to a type.
- refused: there's no `{full}`
- refused: `{full}` takes a `Float`, but this is {written}
  - why: `.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` say how a `Float`'s fraction goes.
- refused: `be.Whole` on a `Float` says how the fraction goes
  - why: the ways are `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.
- refused: {found} has no method `has`
- refused: `.has` takes `key:` and its value
- refused: `.has` takes `key:` and its value
- refused: `{name}` splits a text, but this is {found}
- refused: `{name}` takes `by:` and the text to split at
- refused: `{name}` takes `by:` and the text to split at
- refused: {found} has no method `{}`
- refused: `.add` takes the item, then `pos:` or `offset:` when it goes at a place
- refused: `.add` takes the item to add
- refused: `.remove` takes `pos:` or `offset:` and its number
- refused: `.remove` takes `pos:` or `offset:` and its number
- refused: `.remove` takes `key:` and its value
- refused: `.remove` takes `key:` and its value
- refused: an `Array` has no `.{}`
  - why: an array holds exactly its length, so nothing is added or taken away; its items can still change.
- refused: {found} has no method `{}`

**`crates/seal/src/check/expressions.rs`**

- refused: `{digits}` is too big for a `Whole`
  - why: a `Whole` goes up to 9223372036854775807.
- refused: a `{name}` is built with `[ ]`, naming every field
  - why: a record is built as `Name[field: value]`.
- refused: `{name}` is a type, not a value
- refused: `{path}` is a procedure, and a call writes `( )`
  - why: every call writes `( )`, even with nothing inside.
- refused: there's no name `{word}` here
- refused: `.len.{}` is a text's length, but this is {found}
  - why: only a text's length comes in graphemes, code points and bytes.
- refused: `[:]` can't say what the dictionary holds, so the name's type has to be written
  - why: a type is written where the value can't say it, as in `name: Dict<key: Text, value: Whole> = [:]`.
- refused: `-{digits}` is too small for a `Whole`
  - why: a `Whole` goes down to -9223372036854775808.
- refused: `?` hands a failure to a caller, and the top level has none
  - why: a failure at the top level is handled on the spot, with `??` or `otherwise`.
- refused: `?` follows something that can't fail
  - why: `?` passes a failure up, so it goes after a call that can fail.
- refused: `{text}` is too big for a `Float`
  - why: a `Float` goes up to about 10 to the 308.
- refused: `{sign}{text}` can't be stored exactly as a `Float`; the inexact number is written `{sign}~{text}`
  - why: a `~` marks a `Float` that's only close to what's written.
- refused: `{text}` is stored exactly as a `Float`, so it takes no `~`
  - why: a `~` marks a `Float` that's only close to what's written.
- refused: `{name}` is marked `unused`, so it can't be read
  - why: `unused` marks a name that's never read on purpose.
- refused: `{name}` is a procedure, and a call writes `( )`
  - why: every call writes `( )`, even with nothing inside.
- refused: there's no name `{name}` here
- refused: {operator} takes {wanted}, but this is {found}
- refused: {quoted} can't mix a `Whole` and a `Float`
- refused: {quoted} has {left_text} on its left, but {right_text} on its right
- refused: `{path}` is a procedure, and a call writes `( )`
  - why: `( )` calls a procedure; `[ ]` holds or describes values.
- refused: an item is reached with one of `pos:`, `offset:` or `key:`
- refused: `key:` reaches a value of a `Dict`, but this is {found}
- refused: an item is reached with `pos:`, `offset:` or `key:`
  - why: `pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.
- refused: a `Maybe` is written `Maybe.none` or `Maybe.some[value: …]`
  - why: `Maybe` has the cases `none` and `some[value]`.
- refused: `{type_name}` is a record, which has no cases
- refused: a `{type_name}` is written as one of its cases, as in `{type_name}.case`
  - why: a case is written as a narrowing of its type.
- refused: `{type_name}` has no case `{}`
- refused: `{type_name}` is a type, not a value
- refused: there's no type named `{type_name}`
- refused: `{what}` carries values, so it's built with `[ ]`, as in `{what}[{}]`
  - why: a case's values are named in `[ ]`, as a record's fields are.
- refused: `{what}` carries no values, so it has no `[ ]`
- refused: each value of `{what}` is named, as in `field: value`
  - why: a record is built naming every field, so which value is which can be seen.
- refused: `{what}` has no field `{}`
- refused: `{}` is given twice
- refused: `{what}` is built without {}
  - why: a record is built naming every field; one left out is refused.
- refused: this list has {} items, but its `len.max` is {max}
  - why: a list never holds more than its `len.max`.
- refused: this has {} items, but the array holds exactly {length}
  - why: an array's length is part of its type.
- refused: `[]` can't say what the list holds, so the name's type has to be written
  - why: a type is written where the value can't say it, as in `name: List<Text> = []`.
- refused: `[all: value]` is only allowed where the length is known, as for an `Array`
  - why: `[all: value]` takes its length from the type, as in `board: Array<Whole>[len: 64] = [all: 0]`.
- refused: the key {plain} is written twice in this dictionary
  - why: a dictionary holds one value for each key, so one of the two would vanish.
- refused: this call gives nothing, so there's no value to stand in for
  - why: after a call that gives nothing, a failure is dealt with by `?? ignore`.
- refused: `?? ignore` follows a call that gives a value
- refused: `??` follows something that can't fail and isn't a `Maybe`
  - why: `??` deals with a failure, or gives a fallback when a `Maybe` is `none`.
- refused: `otherwise` follows something that can't fail
  - why: `otherwise` handles each failure of the call before it.
- refused: the failure `{}` is handled twice
  - why: an `otherwise` has one `case` for each failure.
- refused: this call gives nothing, so there's no value to give `instead`
  - why: after a call that gives nothing, a case may just end.
- refused: the case `{}` can reach its end without giving a value
  - why: each case ends with `instead` and a value, `return`, `break` or `fail`.
- refused: this slide is a chain of {calls}, and one or two calls are written as calls
  - why: a chain of three or more calls is a slide; a shorter one is written with `( )`, as in `show_msg(be.Text(n))`.

**`crates/seal/src/check/finish.rs`**

- refused: this can fail with {}, and nothing deals with the failure
  - why: every call that can fail is dealt with, by `?`, `??` or `otherwise`.
- refused: this `otherwise` has no `case` for {}
  - why: the cases cover every failure the call can give.
- refused: this `otherwise` has a `case` for {}, which the call can't fail with
  - why: the call can fail with {}.
- refused: `{called}` can change `{name}`, which this `for each` is going through
- refused: this can pause, while a `for each` is going through the top-level name `{name}`
- refused: {purpose}, but this is {found}
- refused: {purpose}, but this is {found}
- refused: a `Float` isn't joined with `&` by itself; `be.Text(…)` turns it into text first
  - why: a `Float`'s text carries its `~` and every digit, which shouldn't appear by accident.
- refused: {purpose}, but this is {found}
  - why: only text, a `Whole` or a `Bool` is joined with `&`.
- refused: this is already a `Text`, so `be.Text` does nothing
  - why: converting a value to the type it already is, is refused.
- refused: a `Dict`'s value is reached with `key:`
- refused: {purpose}, but this is {found}
- refused: {purpose}, but this is {found}
- refused: this is {found}, but {} is {wanted}
- refused: what `{name}` gives back can't be worked out from its body
  - why: a procedure's result comes from its `return`s; where they can't say, it's written with `-> Type`.
- refused: this value's type can't be worked out
- refused: `{name}`'s type can't be worked out
- refused: `{}`'s written failures don't match: {body}
  - why: failures written after `!` must match what the body does.
- refused: this pauses, and the top level can't pause

**`crates/seal/src/check/mod.rs`**

- refused: `{}` is one of Seal's own types, so a script can't declare it
  - why: each type has one meaning.
- refused: the type `{}` is declared twice
  - why: each type is declared once.
- refused: the case `{}` is listed twice
  - why: each case is listed once.
- refused: the field `{}` is listed twice
  - why: each field is listed once.
- refused: `{}`'s field `{field}` holds {found}, with no way for a value to end
- refused: every case of `{}` holds more that can't end, so no value of it could end
- refused: `{digits}` is too big for a `Whole`
- refused: `{name}` has no setting `{setting}`
- refused: `{setting}` can't be less than 0
- refused: `{setting}` is written twice
- refused: `{name}` has one blank, so it takes no label
  - why: a type with a single blank leaves it unlabelled, since nothing can be mixed up.
- refused: an `Array` says how many items it holds, as in `Array<Whole>[len: 64]`
  - why: an array's length is part of its type.
- refused: a `Dict`'s blanks are labelled `key:` and `value:`, as in `Dict<key: Text, value: Whole>`
  - why: a type with more than one blank labels each one.
- refused: there's no type named `{name}`
- refused: a `Float` can't be a `Dict`'s key
  - why: two neighbouring floats that print alike would be different keys.
- refused: `{}` is a narrowing, so it's declared inside a `fam.proc` block
  - why: a word and its kinds are declared together.
- refused: `{}` isn't `{}` or a kind of it
  - why: each member of a family is the word itself, or reads as "a Y kind of X" for its name `X.Y`.
- refused: the procedure `{name}` is declared twice
  - why: each procedure is declared once.
- refused: two parameters are named `{}`
  - why: each parameter has a name of its own.
- refused: `{}` isn't one of `{name}`'s parameters
- refused: two parameters have the label `{}`
  - why: each label names one argument.
- refused: `{}` is given two labels
- refused: `{}` isn't one of `{name}`'s labels
  - why: only a labelled parameter can have a default.
- refused: this default is {found}, but `{}` is {wanted}
- refused: `{}` has two defaults
- refused: a default is a plain value, never a calculation or a call
  - why: a default is worked out when the script loads.
- refused: `{name}` is reached here before the top level has made it
  - why: the top level runs from top to bottom when the script loads, and a procedure it calls runs right then.
- refused: `{word}` is already {taken_by}, so nothing else can be named `{word}`
- refused: `{}` is already a top-level name, so {what} can't be named `{}`
- refused: the case `{}` has no value `{}`
  - why: a case's values come out by name.
- refused: `{}` already exists, so it can't be made again
  - why: while a name exists, it can't be made again.
- refused: `{name}` is never used
- refused: `{name}` is never used
  - why: a parameter that's never read on purpose is marked, as `unused {name}`.
- refused: `{name}` is never used
  - why: a `for each` name that's never read on purpose is marked, as `for each unused {name}`.
- refused: `{name}` is never used
  - why: a `for each` that doesn't read where it is leaves its `at` part out.
- refused: `{name}` is never used
  - why: a `case` takes out only the values it reads.
- refused: `{}` gives a value, but it can reach its `end` without a `return`
  - why: when any `return` in a procedure gives a value, every way through ends in one; a `while` counts as able to end, whatever its condition.
- refused: this `return` is the procedure's last line, where the procedure ends anyway
  - why: a bare `return` leaves a procedure early.
- refused: `{}` says it gives {wanted}, but it has no `return` with a value
  - why: a procedure's result comes from its `return`s.
- refused: this line can never run
  - why: nothing follows `return`, `break` or `fail` in the same block.
- refused: this line follows an `if` that can leave
- refused: this line works out {found}, but nothing uses it
  - why: a line that only works out a value does nothing; keep the value under a name, or give it to a `mut` name with `=`.
- refused: this call gives {found}, but nothing uses it
  - why: a call that gives a value is used: kept under a name, passed on, or compared; `unused name = …` drops it on purpose.
- refused: this `break` isn't inside a loop
  - why: `break` leaves the innermost loop it's in.
- refused: `fail` hands a failure to a caller, and the top level has none
- refused: this `instead` isn't inside an `otherwise` case
  - why: `instead` gives the value an `otherwise` case carries on with.
- refused: this is {found}, but {role} is {wanted_text}
- refused: `{word}` checks a `Bool`, but this is {found}
  - why: a number or text doesn't stand for true or false; compare it{hint}.
- refused: `{}` between two `Float`s in a `while` condition
  - why: until it reaches".
- refused: `Maybe.none` can't say what the `Maybe` holds, so the name's type has to be written
  - why: a type is written where the value can't say it, as in `name: Maybe<Whole> = Maybe.none`.
- refused: this gives nothing, so there's no value for the name
  - why: a procedure that gives nothing back can't be used as a value.
- refused: `{name}` is being gone through by a `for each`, so it can't change here
  - why: what a `for each` goes through can't change inside it.
- refused: `{name}` can't be given a new value
- refused: there's no name `{name}` here
- refused: {name} is {found}, which has no fields to change
- refused: {found} has no field `{}`
- refused: an item is reached with one of `pos:`, `offset:` or `key:`
- refused: a list's item is reached with `pos:` or `offset:`
  - why: `pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.
- refused: a dictionary's value is reached with `key:`
- refused: {name} is {found}, which has no items to change
- refused: a change reaches into a `mut` name, and this isn't a name
  - why: a change is a name changing its own value.
- refused: there's no name `{name}` here
- refused: this is {found}, which has no fields
- refused: {found} has no field `{}`
- refused: an item is reached with one of `pos:`, `offset:` or `key:`
- refused: a list's item is reached with `pos:` or `offset:`
  - why: `pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.
- refused: this is {found}, which has no items
- refused: `return` hands a value to a caller, and the top level has none
- refused: this `return` gives a value, but an earlier one gives none
- refused: this gives nothing, so there's nothing to return
  - why: a procedure that gives nothing back can't be used as a value.
- refused: this `return` gives {found}, but the procedure gives {wanted}
- refused: this `return` gives no value, but the procedure gives one
- refused: a `range` takes `from:`, one of `to:` or `before:`, and an optional `step_by:`
  - why: `to:` includes its number and `before:` stops short of it.
- refused: a `range` takes `from:`, one of `to:` or `before:`, and an optional `step_by:`
  - why: `to:` includes its number and `before:` stops short of it.
- refused: `at key` is for going through a `Dict`
- refused: `at key` is for going through a `Dict`, but this is {found}
- refused: this range's step is 0
  - why: a range with a step of 0 never reaches its end.
- refused: this range counts by {step} from {from}, away from its end, {end}
  - why: a range's step points towards its end; a negative `step_by:` counts down.
- refused: `match` reads which case a value is, but this is {found}
  - why: only a `one_of` or a `Maybe` has cases.
- refused: {found} has no case `{}`
- refused: the case `{}` is handled twice
  - why: a `match` has one `case` for each case.
- refused: this `match` has no `case` for {}
  - why: a `match` covers every case; there's no catch-all.
- refused: the script's record `{name}` doesn't have the fields the host's `{name}` has
  - why: a record crosses a door as the host's struct, field for field.
- refused: the script's `one_of` `{name}` doesn't have the cases the host's `{name}` has
  - why: a `one_of` crosses a door as the host's enum, case for case.

**`crates/seal/src/syntax/lexer.rs`**

- refused: this `///` has no declaration after it
  - why: a `///` comment describes the declaration on the line after it.
- refused: {extra} extra blank {lines}, after the one on line {}
  - why: one blank line is enough to separate the parts of a script.
- refused: this line is indented with a tab
  - why: one level of indentation is 4 spaces.
- refused: this blank line has spaces on it
  - why: a blank line is empty.
- refused: this `comment` is never closed with `end`
  - why: a block comment closes with `end`, at the depth of its `comment` line.
- refused: this `{}` is never closed
- refused: this `///` has no declaration after it
  - why: a `///` comment describes the declaration on the line after it.
- refused: this text is never closed with `"`
- refused: a statement has no `;`
  - why: the end of a line ends its statement.
- refused: a name has no `$`
  - why: Seal's names are plain words.
- refused: `{other}` can't be read here
- refused: `{}` closes a `{}`
- refused: this `{}` closes nothing
- refused: the name `{word}` {problem}
- refused: a letter comes right after this number
- refused: `~` goes in front of a number with a point, as in `~0.1`
- refused: `~` goes in front of a number with a point, as in `~0.1`
  - why: only a `Float` can be inexact.
- refused: this line's spacing isn't the fixed spacing
  - why: spacing is fixed, so every script reads alike: one space around operators and after `,` and `:`, and none inside brackets or at a line's end.

**`crates/seal/src/syntax/parser.rs`**

- refused: this line is indented {} spaces, where its block's body is indented {wanted}
  - why: one level of indentation is 4 spaces.
- refused: this line is indented, but it isn't inside a block
  - why: only a block's body is indented.
- refused: {wanted} goes here, but {} was found
- refused: {wanted} goes here, but the line ends
- refused: `{}` is a keyword, so it can't be a name
  - why: a name can't be one of Seal's own words.
- refused: a type's name starts with a capital, but `{name}` doesn't
  - why: a capital always means a type.
- refused: {} comes after the end of this line's statement
  - why: each statement goes on a line of its own.
- refused: this `end` is at the wrong depth
  - why: an `end` goes at the depth of the line that opened its block.
- refused: the `{}` on line {} has no `end`
  - why: every block closes with `end`, at the depth of the line that opened it.
- refused: a procedure's parameters go in `[ ]`, not `( )`
- refused: a procedure that takes nothing has no `[ ]`: it's written `proc {name}`
  - why: `[ ]` describes the values a procedure takes, and this one takes none.
- refused: a `fam.proc` block holds only procedures
  - why: a family holds only the kind of thing it's declared for.
- refused: a `{}` line goes at the top of a procedure's body
  - why: `labels` is the first line of a procedure's body, and `defaults` comes right after it.
- refused: this `{}` has no block to belong to
- refused: this can't be given a value
  - why: `=` gives a value to a name, a field or an item.
- refused: Seal writes `elif`, not `else if`
  - why: `elif` is short for "else if".
- refused: {} comes after a slide's last step
  - why: a slide's step is a procedure, so anything after the slide needs the slide in `( )`.
- refused: two comparisons in a row
- refused: maths next to `&` needs `( )` around it
  - why: maths doesn't say whether `&` or the sum goes first, so brackets do.
- refused: `{0}` needs `.floor` or `.trunc`, as in `{0}.floor`
  - why: how the fraction goes is always said.
- refused: this line ends where a value is still wanted
  - why: a line carries on only inside an open `(` or `[`, so wrap the value in `( )`.

**`crates/seal/src/vm/machine.rs`**

- check failed: the result is too big for a `Whole`
  - why: `{left} {operator} {right}`
- check failed: this divides by zero
  - why: nothing can be divided by zero.
- check failed: the result isn't a real number a `Float` can hold
  - why: `{} {operator} {}`
- check failed: there's no item at this place
  - why: an item is reached at a place the list has.
- check failed: there's no character at this place
  - why: a text is reached at a place it has, counting what a person sees as one character.
- check failed: the result is too big for a `Whole`
  - why: `-({value})`
- check failed: this division leaves a remainder
  - why: `/` on two `Whole`s divides exactly; `div.floor` and `div.trunc` say how a fraction goes.
- check failed: a negative number has no square root
  - why: the square root of a negative number isn't a real number.
- check failed: the result is too big for a `Whole`
  - why: `be.Whole.{name}({})`
- check failed: this range's step is 0
  - why: a range with a step of 0 never reaches its end.
- check failed: this range's step points away from its end
  - why: a range's step points towards its end; a negative `step_by:` counts down.

