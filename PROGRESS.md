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
2. **Decisions built.** D1 to D135 are built, except the language server's parts and what's listed under "Not built
   yet" below. All 21 tests in `tests/` pass.
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
- [x] **C8. The smaller decisions.** D79, D80, D82, D83, D85, D86, D87, D89 to D96, D99 to D101, D103 to D106: each
  stop the code had becomes the decided refusal or feature, `==` on bigger values (D95), a `range` as a list (D99),
  every value shown as written (D100), `places:` (D94), `be.Bool` (D101). Every D57 goal still met after it.
- [x] **C9. The translator writes Unio's current spellings** (Unio D190 to D203) and the comparison with Ralph runs
  again: 404 the same, 0 different, from seed 1, with 4 of the 15 tests compared.
- [x] **C10. Handlers** (D97, D98, D110): `group.handler`, `::name`, `handles [ … ]`, `??` with a handler, `include`,
  `*`, parameters.
- [x] **C11. Maths:** ranged numbers and `abs`, `min`, `max`, `clamp` (D107), `**` (D108), `sin` to `atan2` and `pi`
  (D109), `lerp` (D114), each correctly rounded (D50).
- [x] **C12. Units** (D111 to D113, D120 to D123): numbers with units, Seal's three groups, a script's and a host's own
  groups, conversions, rates, points, `in`, the angle built-ins taking angles, and units at a host's doors.
- [x] **C13. `HugeWhole`, `Decimal`, `HugeDecimal`** (D115, D116, D119, D124 to D127), timed in the benchmarks.
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
- **D79** `?? crash` on a `Maybe` is a failed check when it's `none`; `?? ignore` on one is refused.
- **D80** A line after `instead` in its case is refused.
- **D82** A name's written type may leave out the parts its value shows (`Array[len: 4]`, `List[len.max: 3]`); what
  the value doesn't show is refused. Only a name with a value; parameters and fields write their types whole.
- **D83** A name can't be `all`, so `[all: v]` always means the list form.
- **D86** A line after a `match` where a case leaves is refused; `at key unused item`.
- **D87** Arguments go in the declared order, refused otherwise with the call put in order as the fix;
  `labels.any_order [ … ]`; a case's values in declared order.
- **D89** Splitting by an empty text the checker can see (written, or a top-level name made from `""`) is refused;
  one only known while running is a failed check.
- **D90** A `Dict` key partway along a change is refused.
- **D93** A `one_of` with no cases is refused.
- **D95** `==` and `!=` on lists, arrays, `Dict`s (whatever their order), records, `one_of`s and `Maybe`s compare
  contents, walking both without nesting in the host's stack, counted as bulk work.
- **D96** `<`, `>`, `at_least` and `at_most` on text are refused.
- **D99** `range[…]` is a `List<Whole>` anywhere; a `for each` over one written there still builds nothing. A kept
  range's memory is counted before it's built.
- **D101** `be.Bool` reads exactly `"true"` or `"false"`, failing with `not_a_bool`; other undecided conversions are
  refused.
- **D103, D104** A type's setting, and a default, can be a top-level name made with no `mut` from a value written
  out plainly; a default can be any plain value (lists, `Dict`s, records, cases), and reaches the host's calls too.
- **D105** `at pos` and `at offset` on a `Dict` are refused, with `at key` as the fix.
- **D97, D98, D110** Handlers, in `group.handler name` with an optional `handles [ … ]` line and members `::name`,
  with parameters, `labels` and `defaults` as a procedure's. A use, `?? ::name`, `?? group::name(…)`, or
  `include …` in an `otherwise` block, is checked where it's written, as if its cases were written there: the short
  form reaches the groups whose `handles` names the procedure or door just called, refused when none or two have it;
  `*` is required on a handler that can `return` or `break` and refused on one that can't; a failure covered twice
  is refused. The arguments are worked out when the call fails, just before the cases run (Unio D44's "at the call,
  before the cases run"), each case starting with lines that make the parameters. `group`, `handles` and `include`
  are read as such only where they're written, not made keywords. `otherwise` with a handler on one line is refused
  with `??` as the fix; `fam.handler` is refused. A `fail` in a handler's case is a jump (D129); `handles` names
  any procedure, door or built-in that can fail (`be.Whole`, `be.Bool`), and one that can't fail is refused.
- **D129** `newln` and `tab` outside braces are refused, with `"{newln}"` as the way.
- **D107, D128** Ranged numbers: `Whole[…]` and `Float[…]` with `min:`, `max:`, `min.clamp:`, `max.clamp:` (a number,
  `-` allowed, or a top-level plain name), one per edge. A ranged type resolves to its plain type for every type rule;
  the checker wraps each value landing where one is written (a name, a field, a list or `Dict` item, a `return`, a
  handler's parameter) in a check, and a procedure checks its ranged parameters as it starts, so the host's calls are
  held too; such a procedure isn't written out in place. A value Seal can see past a hard edge is refused while
  loading.
- **D107, D108, D109, D114** `abs`, `min`, `max`, `clamp` on two `Whole`s or `Float`s; `**`, binding tighter than `*`;
  `sin`, `cos`, `tan`, `asin`, `acos`, `atan2(y:, x:)`, `lerp(a, to:, by:)` and `pi`. A negative power of a written
  `Whole` is refused; one only known while running is a failed check. `pi` can be filled into braces.
- **D50: correctly rounded maths.** `crates/seal/src/big.rs` is a whole number of any size, written here with no
  `unsafe` (every candidate crate, `num-bigint`, `astro-float`, `dashu`, uses some). `crates/seal/src/maths.rs` works
  each answer out in big fixed point with an error bound and keeps the `Float` the whole interval rounds to, doubling
  the precision when it can't tell (Ziv's strategy); `**` finds exact answers and ties first, `lerp` is worked out
  exactly and rounded once. 600 answers checked against Python's 120-digit decimals: all the same. The platform's own
  `tan` is 2 ulps off at -7.59421142430311, checked by hand. Cost: about 14 µs a call, with no fast path yet.
- An unclosed `{` in text is refused with `\\{` as the fix (D75: a brace shown as it is, is written `\\{`).
- **D111 to D113, D120 to D123: units.** A unit is part of a number's type (`Type::Measured`), written bare after a
  number (`2 secs`, `5.0 px per secs`, `100.0 px.point`) or in a type (`Float[unit: secs]`), with its group only to
  settle a clash (`time::secs`). Seal's groups are `time` (`ms`, `secs`, `mins`, `frames`), `angle` (`degs`,
  `turns`, `rads`) and `distance` (`px`, `tiles`); a radian is 180/π degrees, held exactly, so `rads` converts to
  the others correctly rounded, while `frames` and `tiles` have no fixed size and convert to nothing. A unit's size
  is a fraction times a power of π, marked inexact when it has π or was written with `~`. Values convert by
  themselves where they land and where two meet in `+`, `-` or a comparison: into a `Whole` only by a whole number
  of times and never through an inexact size, into a `Float` by any size, rounded once; two sides meet in the finer
  unit. A plain number where a unit goes is refused, with the unit added as the fix (and `.0` for a `Float`). An
  amount over one of its own group is a plain number; over another group, a rate; a rate times its `per` unit is an
  amount. A point plus or minus a span is a point, a point minus a point a span, two points added refused.
  `value in unit` measures (D122), also inside text braces as D122's own example writes it. `group.unit name` with
  `::unit base` and `::unit = amount earlier_unit` (D123). The angle built-ins take `degs`, `turns` or `rads` (exact
  at multiples of 30° and 45°), and `asin`, `acos` and `atan2` give `rads`. A host gives a door's values and result
  units (`DoorOptions::units`, `result_unit`) and declares its own groups (`Doors::add_unit_group`); a host type
  carries one as `HostType::Measured`.
- **D130** Already built so (C10).
- **D131** A handler's case for a failure the call can't give is allowed and never runs; one written straight into
  an `otherwise` block is still refused.
- **D132** `**` chains from the right and comes before a negating `-`.
- **D133** `atan2` of (0, 0) gives 0, and -0.0 counts as zero, where IEEE 754 gives ±π.
- **D134** `asin` and `acos` give a plain `Float`; `atan2` gives `rads` only when given values with units.
- **D135** A `mut` name that's never changed is refused, with the line without `mut` as the fix (a draft wording, as
  D135 says its wording comes later): any change counts, `=`, a method, or deep inside. A name never read gets only
  "never used".
- **D115, D116, D119, D124 to D127: `HugeWhole`, `Decimal`, `HugeDecimal`.** `crates/seal/src/decimal.rs` holds
  decimal128, with IEEE 754's rounding (34 digits, to the nearest, a tie to even, exponents -6176 to 6111) and its
  preferred exponents, so places carry through (`1.50 + 1.50` is `3.00`), each operation worked out exactly on
  `Big` and rounded once; and the exact `HugeDecimal`, whose `/` and `sqrt` give the exact answer or stop. Checked
  against Python's decimal128 context on 21,000 random operations: all the same, except `0 ** 0`, which is 1 as for
  `Whole` and `Float`, where Python calls it invalid. On the heap, all three are a `Number`, never changed once made.
  A number becomes a bigger type by itself where it always fits (a `Whole` into any, a `HugeWhole` or `Decimal` into
  a `HugeDecimal`), where it lands and beside another in maths; a number written out takes the type wanted, or the
  other side's (D119), exactly as written, refused with `~` or with more digits than a `Decimal` keeps. D127's
  operations, `abs`, `min`, `max`, `clamp`, `sqrt` and `lerp` as its table says, `places:` on decimals, `==` by
  amount. D126's conversions, with `too_big` and `not_a_number` as failures that `handles [ … ]` can name. A host
  passes them as `seal::big::Big`, `seal::decimal::Decimal` and `seal::decimal::HugeDecimal`. Their work is paid for
  before it's done, refused when the steps left can't pay, and a `HugeDecimal`'s exponent counts against the memory
  cap, since lining two up writes its zeros out. A `HugeWhole` takes limits, whose edges are `Whole`s, a clamp making
  a new number in place of the old, and units: a `Whole` with a unit widens beside one, and a change of unit
  multiplies by the exact whole factor. the whole lands where
  it's used, so `sqrt(7.0 secs / ~1.1 secs)` and `Whole[max: 5] = 10 - 7` check as the whole, not each part.

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
- **Rare, big instructions live outside the loop.** Joining text, comparing bigger values, ranges kept as lists, `**`,
  the maths built-ins, ranged numbers' checks, `be.Bool` and `?? crash` on a `Maybe` share one case in the loop,
  which hands them to one function, never inlined. Each added as a case of its own made the loop's machine code worse
  for everything: `fib` 18% slower on the same bytecode after joining text was added, and 10 to 16% again after C11's
  instructions. Shared and kept out, `fib` is about 52 ms against 55.5 before this session.
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
- **Where Seal is behind Luau: loading a script from a text of its own.** On 2026-11-25 (after C13), timed apart from
  the rest of the patrol run: from the same text, where Seal reuses what it compiled and Luau compiles once, a script
  loads and runs to its first pause in about 190 ns against Luau's 200 ns (met, from 230 ns: the top level's task
  becomes the first call's, a new stack comes zeroed from the system, and the kept scripts are searched newest
  first). From a text of its own, with nothing reused, a 12-line script takes Seal about 8.6 µs and Luau about 5.1 µs
  (not met; it was 12.3 µs after C12, and about 10 µs before). Split up: reading about 3.3 µs, checking about 3.1 µs,
  compiling about 1.0 µs, and setting the script up about 0.9 µs. Done: a syntax expression is 72 bytes instead of 208
  (C12's units and handlers had grown it), the reader moves names out of tokens instead of copying them and compares
  tokens in place, Seal's own types and unit groups and the doors' description are worked out once and shared, door
  calls borrow instead of copying, built-in failure names aren't copied, and the D84 fix is written only when shown.
  What's left is spread thin across many small allocations: every name is its own string from the reader to the
  checked tree, and every node, scope and table its own allocation, with no single place over a few percent. Luau
  also does no type checking, which is a third of Seal's time. Closing the rest needs names interned as numbers and
  the trees built in one block, a rework of the reader, checker and compiler together, not a fix. A 10,000-line script
  loads in about 9 ms, far inside goal 4, so this only shows on very small scripts loaded in great numbers from
  different texts.
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

Sent 2026-11-25 (batch 1 of this session, 21 questions). Answered so far: 11 (D124), 12 (D119), 15 (D125), and most of
1 to 10 by D120 to D123 (units convert by themselves when exact, `per`, `.point`, `in`, `group.unit`). Still open:
- 13, 14: which `be.` conversions and which operations the new number types take.
- 16: what crosses a door for each new number type (mine to choose; to report).
- 17: whether a ranged number is its own type, and whether "storing" covers parameters, fields, items and `return`s.
- 18 to 20: a handler's parameter labels (chapter 7's example), `fail` in a handler as a jump, what `handles` can name.
- 21: `newln` outside braces.
- D58: whether n-body's Normal program is rightly the one written with D62's direct changes.

Batch 2, sent 2026-11-25 after C12 (C7 to C10's first three were answered by D130 to D132):
- D108: a negative power of a `Whole` only known while running is a failed check (built), as D89 treats an empty
  separator.
- D107: `clamp` with `low:` above `high:` (stops). (D109's `atan2` of (0, 0) is answered by D133.)
- D91 lists the declarations `///` goes on; a name made inside a block isn't among them (stops).
- A comment after code inside an open bracket, as `foo(a, // the first`: D91 allows a comment line there (stops).
- (Answered: D134 for the angle built-ins' units; D121's example was a slip, noted under it.)
- A `Whole` with a unit becomes a `Float` only when its unit changes: `w: Float[unit: secs] = 2000 ms` passes but
  `= 2 secs` is refused, as a plain `Whole` into a `Float` is (D16). Is that wanted, or should the second pass too?
- `a + b in ms`: `in` after maths without brackets could measure the whole or only `b` (stops).
- Multiplying or dividing units into a unit with no written form, as `px * px` or a rate over a rate, and maths on
  points other than adding and subtracting a span (stop).
- D131: two handlers brought into one `otherwise` that both have a case for a failure the call can't give: refused
  as handled twice (built), though neither case can run.
- D111 says Seal follows Cyborg on "prefixes"; Seal's units have none, and nothing says what a prefix is in Seal.

## What Ralph can't compare yet

Ralph has no `Float`, lists, `for each`, `.len`, dividing or the biggest `Whole`s built yet, so the comparison covers
`Whole` maths and overflow, `Bool`, `Text`, `&`, `be.Text`, `be.Whole`, comparisons, logic, `if`, `while`, procedures
(with and without parameters), comments and `show_msg`. 2 of the 14 tests and every fuzzed script match. Seal's
dialect alone has: comparing texts (D64), text by graphemes (D65, D66), `unused` parameters (D61), deeper changes
(D62), and everything with records, `one_of`, `Dict` and failures.

Batch 3, sent 2026-11-25 after C13 with the draft wordings:
- How a whole-valued `Decimal` shows: built as D125 reads, with the places it keeps, so `d: Decimal = 3` shows `3`
  and `3.0` shows `3.0`, unlike a `Float`'s `3.0`.
- Converting a `Float` to a `HugeWhole` or `HugeDecimal`: D126 lists neither (stops as not designed).
- `be.Decimal` on text with more than 34 digits rounds them, as decimal128 does; a number written in the script with
  more is refused. A text whose number is past a `Decimal`'s range fails with `not_a_number`.
- `be.Float` on a `HugeWhole`, `Decimal` or `HugeDecimal` past a `Float`'s range fails with `too_big` (D126 says only
  "to the nearest `Float`").
- Units and limits on a `Decimal` or `HugeDecimal`: D115 gives them to a `HugeWhole`, and nothing says whether decimals
  take them (stops).
- A `HugeWhole` with a unit measured in a coarser one, as `450 cents in dollars`, which isn't whole: a `Whole` gives a
  `Float` there, which a `HugeWhole` doesn't mix with (stops).

## Not designed yet: every stop left in the code

- a `\r` that isn't right before a line break
- a comment after code inside an open bracket
- a `///` before a name made inside a block
- a `Dict` key that isn't a `Text`, a `Whole` or a `Bool` (chapter 6: not decided)
- a handler used inside its own cases
- `in` after maths without brackets, as `a + b in ms`
- maths on units whose answer's unit has no written form, as `px * px`, and maths on points beyond `±` a span
- a `clamp` whose `low:` is more than its `high:` (it stops while running too)
- converting a `Float` to a `HugeWhole` or a `HugeDecimal`, which D126 doesn't list
- units and limits on a `Decimal` or `HugeDecimal`
- a `HugeWhole` with a unit measured in a coarser unit, which isn't whole

## Not built yet

- Working out a type from a later line (D5) in one spot: a change at a place, `name[pos: n] = v`, on a name whose
  type isn't known by then, since a place doesn't say whether it's a list or an array. (A `??`, a `match`, `.add`,
  `.remove`, and a change by field or by key now say what the type must be, and it's worked out there.)
- More than 65,000 top-level names used by procedures, registers in one procedure, or constants.
- Blocks nested more than 48 deep, and values nested more than 160 deep in one statement (bounded so that reading a
  script can't overrun the host's stack).
- Passing a value of a type that holds itself between a host and a script (D72).
- A `HugeWhole`'s limit past a `Whole`'s range, as `HugeWhole[max: 100000000000000000000]` (refused as too big).
- The language server (D47): an empty package, as the prompt says.

## Draft wordings

Refusals, failed checks and stops whose wording no decision gives, drafted in Unio's layout (Unio D54): `what:` says
what's wrong, `why:` gives the rule once. Where a fix can be shown as a whole line, `fix:` stands in place of `what:`
(Unio D101), as for indentation, spacing, a `return`'s header or a `mut` that goes; those are marked.

373 wordings, taken from the code by `crates/seal-check-all/wordings.py` on 2026-11-25, replacing the list of 2026-11-23: `fmt`'s and `&`'s
are gone with them (D75, D77), and so are the named closers' (D102). The 156 marked **(new)** weren't in the
earlier list: units, handlers, ranged numbers, the maths built-ins, the new number types, D135's `mut` and the rest
built since. `{…}` stands for what the message fills in.

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
- refused: `{word}` has no narrowing `.{narrowing}`
  - why: `{word}` takes an angle in `rads`, `degs` or `turns`, as `{word}(90 degs)`, or a plain `Float` in radians.
- refused: `{word}` takes a `Float`, but this is {found} **(new)**
  - why: angles and `lerp` work on `Float`s; a number written with a point, as `2.0`, is a `Float`.
- refused: `{word}` takes values of one type, but this is {right} and the first is {left} **(new)**
  - why: `Whole` and `Float` don't mix quietly: `be.Float(…)` turns a `Whole` into a `Float`.
- refused: `{word}` takes an angle, but this is {found} **(new)**
  - why: an angle is in `rads`, `degs` or `turns`, or a plain `Float` in radians.
- refused: `{word}` takes values of one type, but this is {right} and the first is {left} **(new)**
  - why: the two are `Float`s of one unit group, or both plain.
- not designed yet: a `clamp` whose `low:` is more than its `high:` isn't decided yet **(new)**
- refused: `{word}` is only ever one of its kinds
  - why: `{word}`'s narrowings are {}.
- refused: `{word}` has no narrowing `.{narrowing}`
  - why: `{word}` has no narrowings.
- refused: `{word}` has no narrowing `.{narrowing}`, so there's no `{full}`
  - why: `{word}`'s narrowings are {}.
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
- refused: `be` is narrowed by the type the value becomes, as in `be.Whole`
  - why: `be.Type(value)` converts a value to a type.
- refused: there's no conversion to `{target}` **(new)**
  - why: `be` converts to `Text`, `Whole`, `Float` and `Bool`; other values are built as they're written.
- refused: there's no `{full}`
  - why: `be`'s narrowings are `be.Text`, `be.Float`, `be.Whole`, `be.Bool`, `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.
- refused: there's no conversion from {found} to a `Bool` **(new)**
  - why: a number is compared, as `n != 0`, for a `Bool`.
- refused: this is already a `{target}`, so `be.{target}` does nothing **(new)**
  - why: converting a value to the type it already is, is refused.
- refused: there's no conversion from {written} to a `Bool` **(new)**
  - why: `be.Bool` reads `"true"` or `"false"` from text; a number is compared, as `n != 0`, for a `Bool`.
- refused: there's no conversion from {written} to a `Float` **(new)**
  - why: `be.Float` turns a `Whole` into a `Float`.
- refused: `{full}` takes a `Float`, but this is {written}
  - why: `.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` say how a `Float`'s fraction goes.
- refused: `be.Whole` on a `Float` says how the fraction goes
  - why: the ways are `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.
- refused: there's no conversion from {written} to a `Whole` **(new)**
  - why: `be.Whole` reads a `Whole` from text, and turns a `Float` into one with `.floor`, `.trunc`, `.ceil`, `.nearest` or `.nearest.even`.
- refused: {found} has no method `has`
- refused: `.has` takes `key:` and its value
- refused: `{name}` splits a text, but this is {found}
- refused: `{name}` takes `by:` and the text to split at
- refused: this splits a text by an empty text **(new)**
  - why: an empty text is never found between two pieces; `for each` goes through a text's characters, as `for each letter in name`.
- refused: {found} has no method `{}`
- refused: `.add` takes the item, then `pos:` or `offset:` when it goes at a place
- refused: `.add` takes the item to add
- refused: `.remove` takes `pos:` or `offset:` and its number
- refused: `.remove` takes `key:` and its value
- refused: an `Array` has no `.{}`
  - why: an array holds exactly its length, so nothing is added or taken away; its items can still change.
- not built yet: working out a type from a later line (D5), which this `.{}` needs, isn't built yet **(new)**

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
- refused: a unit stands right after its number, as `2 secs`, or in a type, as `Float[unit: secs]` **(new)**
  - why: a unit says what a number counts, so it goes with one.
- refused: `places:` takes a whole number written out, but this is `{places}` **(new)**
  - why: `places:` says how many digits a `Float` shows after its point, as `{speed, places: 2}`.
- refused: `{{inside}}` measures in `{unit}`, which isn't a unit's name **(new)**
  - why: `in` in braces measures a name in one of its group's units, as `{total in secs}`.
- refused: `{{inside}}` isn't a name with its fields and facts **(new)**
  - why: braces in text hold a name, which may be followed by its fields and facts, never a calculation or a call; a `Float` can add `, places: 2`.
- refused: there's no name `{first}` here
  - why: a name in braces is checked when the script loads; a brace shown as it is, is written `\{`.
- refused: `places:` shows a `Float` to so many digits, but this is {found} **(new)**
  - why: `places:` is only for a `Float`, which otherwise shows every digit it needs.
- refused: `{text}` is too big for a `Float`
  - why: a `Float` goes up to about 10 to the 308.
- refused: `{sign}{text}` can't be stored exactly as a `Float`; the inexact number is written `{sign}~{text}`
  - why: a `~` marks a `Float` that's only close to what's written.
- refused: `{text}` is stored exactly as a `Float`, so it takes no `~`
  - why: a `~` marks a `Float` that's only close to what's written.
- refused: `{name}` is marked `unused`, so it can't be read
  - why: `unused` marks a name that's never read on purpose.
- refused: `{name}` fills in braces in text, so it's written `"{{name}}"` **(new)**
  - why: `{newln}` and `{tab}` give a line break and a tab inside text; the text holding just one is written `"{newln}"`.
- refused: `{name}` is a procedure, and a call writes `( )`
  - why: every call writes `( )`, even with nothing inside.
- refused: there's no name `{name}` here
- refused: {operator} takes {wanted}, but this is {found}
- refused: {quoted} can't mix a `Whole` and a `Float`
  - why: `Whole` and `Float` don't mix quietly: `be.Float(…)` turns a `Whole` into a `Float`, and a number written with a point, as `2.0`, is a `Float`.
- refused: {quoted} has {left_text} on its left, but {right_text} on its right
- refused: a `Whole` raised to a negative power isn't a whole number **(new)**
  - why: a `Whole`'s power is 0 or more; `be.Float(…)` gives `Float`s, whose powers can be negative.
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
  - why: `{type_name}`'s cases are {}.
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
  - why: with the failure ignored there'd be no value to carry on with; `?? ignore` goes after a call that gives nothing.
- refused: a handler deals with a call's failures, and a `Maybe` isn't a failure **(new)**
  - why: a `Maybe` is given a value to use when it's `none`, as `?? 0`, or `?? crash` when it can't be `none`.
- refused: `?? ignore` would leave nothing in place of a `Maybe` that's `none` **(new)**
  - why: a `Maybe` is given a value to use when it's `none`, as `?? 0`, or `?? crash` when it can't be `none`; `?? ignore` goes after a call that gives nothing.
- not built yet: working out a type from a later line (D5), which this `??` needs, isn't built yet **(new)**
- refused: `??` follows something that can't fail and isn't a `Maybe`
  - why: `??` deals with a failure, or gives a fallback when a `Maybe` is `none`.
- refused: `otherwise` follows something that can't fail
  - why: `otherwise` handles each failure of the call before it.
- refused: the failure `{name}` is handled twice **(new)**
  - why: an `otherwise` handles each failure once, with its own `case` or an included handler's.
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
- refused: two texts aren't put in order with `<`, `>`, `at_least` or `at_most` **(new)**
  - why: text has no one right order: by character codes `"Zebra"` comes before `"apple"`, and alphabetical order differs between languages.
- refused: this is already a `Text`, so `be.Text` does nothing
  - why: converting a value to the type it already is, is refused.
- refused: a `Dict`'s value is reached with `key:`
- refused: this is {found}, but {} is {wanted}
- refused: what `{name}` gives back can't be worked out from its body
  - why: a procedure's result comes from its `return`s; where they can't say, it's written with `-> Type`.
- refused: this value's type can't be worked out
- refused: `{name}`'s type can't be worked out
- refused: `{}` can fail with {}, but its header doesn't say so (shown with a `fix:` line) **(new)**
- refused: `{}`'s written failures don't match: {body} (shown with a `fix:` line) **(new)**
- refused: this pauses, and the top level can't pause
  - why: the host waits for loading to finish; a pause belongs in a procedure the host calls.

**`crates/seal/src/check/handlers.rs`**

- refused: the handler group `{}` is declared twice **(new)**
  - why: each handler group is declared once.
- refused: `{}` is already a top-level name, so a handler group can't be named `{}` **(new)**
- refused: the handler `::{}` is listed twice in `{}` **(new)**
  - why: each handler in a group has a name of its own.
- refused: there's no procedure or door named `{full}` for `{}` to handle **(new)**
- refused: `{full}` can't fail, so `{}` has nothing to handle for it **(new)**
  - why: `handles [ … ]` names the procedures, doors and built-ins that can fail, whose failures the group's handlers deal with.
- refused: there's no handler group named `{}` **(new)**
- refused: `::{member}` could be {} **(new)**
  - why: when two groups that handle the call both have the handler, the group is written, as `group::name`.
- refused: `{}` has no handler `::{member}` **(new)**
  - why: `{}`'s handlers are {}.
- not designed yet: a handler used inside its own cases isn't decided yet **(new)**
- refused: `{}` isn't one of `::{member}`'s parameters **(new)**
- refused: `{}` isn't one of `::{member}`'s labels **(new)**
- refused: the failure `{}` is handled twice
  - why: a handler has one `case` for each failure.
- refused: `{}` is never used **(new)**
  - why: a handler's parameter that's never read on purpose is marked, as `unused {}`.

**`crates/seal/src/check/mod.rs`**

- refused: `{}` is one of the host's types, so the script can't declare it **(new)**
  - why: the host opens its own types for a script, which uses them by name, as it uses a door.
- refused: `{}` is one of Seal's own types, so a script can't declare it
  - why: each type has one meaning.
- refused: the type `{}` is declared twice
  - why: each type is declared once.
- refused: the case `{}` is listed twice
  - why: each case is listed once.
- refused: `{}` has no cases, so no value of it could exist **(new)**
  - why: a `one_of`'s value is exactly one of its cases, listed one per line.
- refused: the field `{}` is listed twice
  - why: each field is listed once.
- refused: `{}`'s field `{field}` holds {found}, with no way for a value to end
- refused: every case of `{}` holds more that can't end, so no value of it could end
- refused: `{}` has no setting `{setting}` **(new)**
  - why: a number's settings are its limits: `min:`, `max:`, `min.clamp:` and `max.clamp:`.
- refused: the {edge} edge is written twice **(new)**
  - why: each edge is written once, with `min:` or `min.clamp:`, and `max:` or `max.clamp:`.
- refused: this number's lowest edge is above its highest **(new)**
  - why: a ranged number's `min` is at most its `max`, so some value fits.
- refused: `{name}` isn't a top-level name made from a number written out, with no `mut` **(new)**
  - why: a number's limits are known while loading: a number written there, or a name made at the top level from one, with no `mut`.
- refused: this number's limit is {wanted} written out **(new)**
  - why: a number's limits are known while loading, and are of its own type.
- refused: this is {side} its type's `{word}: {}` **(new)**
  - why: a ranged number's `min:` and `max:` are hard limits, and one written past them never fits; `min.clamp:` and `max.clamp:` pull a value back instead.
- refused: `{digits}` is too big for a `Whole`
- refused: a type's setting is a whole number known while loading, never a calculation or a call **(new)**
  - why: a type's setting is a whole number written there, or a name made at the top level from one, with no `mut`, as `max_items = 10`.
- not designed yet: units and limits on a `{name}` isn't decided yet **(new)**
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
- not designed yet: {written} as a `Dict`'s key isn't decided yet **(new)**
- refused: `{}` is a narrowing, so it's declared inside a `fam.proc` block
  - why: a word and its kinds are declared together.
- refused: `{}` isn't `{}` or a kind of it
  - why: each member of a family is the word itself, or reads as "a Y kind of X" for its name `X.Y`.
- refused: `{word}` is one of Seal's own words, so a procedure can't be named `{word}` **(new)**
- refused: `{word}` is one of the host's doors, so a procedure can't be named `{word}` **(new)**
- refused: the procedure `{name}` is declared twice
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
- refused: a default is a value written out plainly, never a calculation or a call **(new)**
  - why: a default is the same every time: a number, text, a `Bool`, a list, a record, a case, or a top-level name made from one with no `mut`.
- refused: `{name}` is reached here before the top level has made it
- refused: a name can't be `all` **(new)**
  - why: `[all: value]` always means a list or array whose every item starts as `value`, so `all` can't name anything else.
- refused: `{word}` is already {taken_by}, so nothing else can be named `{word}`
- refused: `{}` is already a top-level name, so {what} can't be named `{}`
- refused: the case `{}` has no value `{}`
  - why: a case's values come out by name.
- refused: the case `{}`'s values are taken out in the order they're declared: `{}` **(new)**
  - why: a case's values come out in the order its type declares them, skipping any that aren't read.
- refused: `{}` already exists, so it can't be made again
- refused: `{name}` is never changed, so it isn't `mut` **(new)**
  - why: `mut` marks a name that changes; one that never does is made without it.
- refused: `{name}` is never used
- refused: `{name}` is never used
  - why: a parameter that's never read on purpose is marked, as `unused {name}`.
- refused: `{name}` is never used
  - why: a `for each` name that's never read on purpose is marked, as `for each unused {name}`.
- refused: `{name}` is never used
  - why: an `at` name that's never read is marked, as `at key unused {name}`, or the `at` part is left out.
- refused: `{name}` is never used
  - why: a `case` takes out only the values it reads.
- refused: `{}` gives a value, but it can reach its `end` without a `return`
  - why: when any `return` in a procedure gives a value, every way through ends in one; a `while` counts as able to end, whatever its condition.
- refused: this `return` is the procedure's last line, where the procedure ends anyway
  - why: a bare `return` leaves a procedure early.
- refused: `{}` says it gives {wanted}, but it has no `return` with a value
  - why: a procedure's result comes from its `return`s.
- refused: this line can never run
- refused: this line follows an `if` that can leave
- refused: this line follows a `match` that can leave **(new)**
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
  - why: a `Float` that changes each pass rarely lands exactly on a value, so the loop could never stop; `<`, `>`, `at_least` or `at_most` say "until it reaches".
- refused: `{kind}` leaves out what it holds, and the value doesn't show it **(new)**
  - why: a written type leaves out only what its value shows, so this one writes it, as `{example}`.
- refused: `Maybe.none` can't say what the `Maybe` holds, so the name's type has to be written
  - why: a type is written where the value can't say it, as in `name: Maybe<Whole> = Maybe.none`.
- refused: this gives nothing, so there's no value for the name
  - why: a procedure that gives nothing back can't be used as a value.
- refused: `{name}` is being gone through by a `for each`, so it can't change here
  - why: what a `for each` goes through can't change inside it.
- refused: `{name}` can't be given a new value
- refused: {name} is {found}, which has no fields to change
- refused: {found} has no field `{}`
- refused: a list's item is reached with `pos:` or `offset:`
  - why: `pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.
- refused: a dictionary's value is reached with `key:`
- not built yet: working out a type from a later line (D5), which this change needs, isn't built yet **(new)**
- refused: {name} is {found}, which has no items to change
- refused: a change reaches into a `mut` name, and this isn't a name
  - why: a change is a name changing its own value.
- refused: this is {found}, which has no fields
- refused: a change can't go through a `Dict`'s key on its way, since the key might be missing **(new)**
  - why: a lookup is dealt with on its own line: the value is taken out, as `mut north = shops[key: "north"] ?? crash`, changed, and put back, as `shops[key: "north"] = north`.
- refused: this is {found}, which has no items
- refused: `return` hands a value to a caller, and the top level has none
- refused: this `return` gives back {}, but the procedure's header has no `-> Type` **(new)**
- refused: this `return` gives a value, but an earlier one gives none
- refused: this gives nothing, so there's nothing to return
  - why: a procedure that gives nothing back can't be used as a value.
- refused: this `return` gives {found}, but the procedure gives {wanted}
- refused: this `return` gives no value, but the procedure gives one
- refused: `at key` is for going through a `Dict`
- refused: `at {word}` is for going through a list; a `Dict` is gone through `at key` (shown with a `fix:` line) **(new)**
- refused: `at key` is for going through a `Dict`, but this is {found}
- refused: a `range` takes `from:`, one of `to:` or `before:`, and an optional `step_by:`
  - why: `to:` includes its number and `before:` stops short of it.
- refused: this range's step is 0
  - why: a range with a step of 0 never reaches its end.
- refused: this range counts by {step} from {from}, away from its end, {end}
  - why: a range's step points towards its end; a negative `step_by:` counts down.
- not built yet: working out a type from a later line (D5), which this `match` needs, isn't built yet **(new)**
- refused: `match` reads which case a value is, but this is {found}
  - why: only a `one_of` or a `Maybe` has cases.
- refused: {found} has no case `{}`
- refused: the case `{}` is handled twice
  - why: a `match` has one `case` for each case.
- refused: this `match` has no `case` for {}
  - why: a `match` covers every case; there's no catch-all.

**`crates/seal/src/check/numbers.rs`**

- refused: a `{name}` holds exactly what's written, so its number takes no `~` **(new)**
  - why: a `~` marks a `Float` that's only close to what's written; a decimal counts in tens, so `0.1` is exact.
- refused: `{text}` has more digits than a `Decimal` keeps **(new)**
  - why: a `Decimal` keeps 34 digits; a `HugeDecimal` keeps any number of them.
- refused: {quoted} has {left_type} on its left, but {right_type} on its right **(new)**
- refused: {quoted} can't mix a `{left_type}` and a `{right_type}` **(new)**
  - why: a number becomes a bigger type by itself only where it always fits; `be.` converts the other ways, and a `Float` mixes with none of these.
- refused: `**` on a `{}` takes a whole power, but this is {found} **(new)**
  - why: a `HugeWhole`, `Decimal` or `HugeDecimal` is raised to a `Whole` power.
- refused: a `HugeWhole` to a negative power isn't a whole number **(new)**
  - why: a negative power is one over the number; a `HugeDecimal` can hold it when it has an end.
- refused: there's no `{full}`
  - why: `be.HugeWhole` takes `.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` for a decimal, and `be.Decimal` takes `.shown` and `.nearest` for a `Float`.
- refused: there's no conversion from {found} to a `{name}` **(new)**
  - why: a number type converts from text and from the other number types.
- refused: this is already a `{name}`, so `be.{name}` does nothing **(new)**
  - why: converting a value to the type it already is, is refused.
- not designed yet: converting a `Float` to a `{name}` isn't decided yet **(new)**
- refused: `be.Decimal` on a `Float` says which `Decimal` it becomes **(new)**
  - why: `be.Decimal.shown` takes the number the `Float` shows, so `~0.1` gives `0.1`; `be.Decimal.nearest` takes the `Decimal` nearest the value it holds.
- refused: `{full}` takes a `Float`, but this is a `{}` **(new)**
  - why: `.shown` and `.nearest` say which `Decimal` a `Float` becomes.
- refused: `be.HugeWhole` on a `{}` says how the fraction goes **(new)**
  - why: the ways are `be.HugeWhole.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even`.
- refused: `{full}` rounds a fraction, and a `{}` has none **(new)**
  - why: `be.HugeWhole` converts a whole number or text with nothing to round.
- refused: `be.Whole` on a `{source}` says how the fraction goes **(new)**
  - why: the ways are `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.
- refused: `{full}` rounds a fraction, and a `HugeWhole` has none **(new)**
  - why: `be.Whole` converts a `HugeWhole`, failing with `too_big` when it doesn't fit.

**`crates/seal/src/check/units.rs`**

- refused: the unit group `{}` is declared twice, or is one of Seal's own **(new)**
  - why: each unit group is declared once; `time`, `angle` and `distance` are Seal's own.
- refused: `{}` has {bases} units marked `base`, where it needs exactly one **(new)**
  - why: a unit group's base is the unit every other one is counted in, so there's exactly one.
- refused: the unit `::{}` is listed twice in `{}` **(new)**
  - why: each unit in a group has a name of its own.
- refused: `{}` isn't a unit listed above in `{}` **(new)**
  - why: each unit is counted in one listed before it, so every size is known.
- refused: a unit's size is a number written out **(new)**
  - why: a unit is so many of an earlier one, as `::dollars = 100 cents`.
- refused: a unit's size can't be 0 **(new)**
- refused: `{}` could be {} **(new)**
  - why: when two groups have a unit of the same name, the group is written before it, with `::`.
- refused: a rate isn't a point **(new)**
  - why: `.point` marks a place counted from a fixed one, as `px.point`; a rate, as `px per secs`, is an amount.
- refused: this is {found}, but {wanted_text} goes here, and {why} **(new)**
  - why: units of one group convert by themselves only where nothing is lost: into a `Whole`, a finer unit; into a `Float`, any unit with a fixed size.
- refused: this plain number goes where `{unit_text}` is wanted **(new)**
  - why: where something takes a unit, the number says it, so seconds and milliseconds, or degrees and radians, can't be mixed up.
- refused: this has a unit, `{unit_text}`, where a plain number goes **(new)**
  - why: a value with a unit gives its plain number when it's measured in one, as `value in {unit_text}`.
- refused: `in` measures a value with a unit, but this is {found} **(new)**
  - why: `value in unit` gives the plain number a value with a unit is, as `total in secs`.
- refused: a value in `{from_text}` isn't measured in `{to_text}` **(new)**
  - why: only a unit of the value's own group measures it, and only one with a fixed size, which `frames` and `tiles` haven't.
- not designed yet: a `HugeWhole` measured in a unit it isn't always a whole number of isn't decided yet **(new)**
- refused: `{operator}` has {left_text} on its left, but {right_text} on its right **(new)**
  - why: units of different groups, as seconds and pixels, don't go together, and a `Whole` and a `Float` don't mix quietly.
- refused: this can't be brought to the other side's unit for `{operator}`: {why} **(new)**
  - why: units of one group combine where they convert exactly, in the finer unit.
- refused: {quoted} can't mix a `Whole` and a `Float`
  - why: `Whole` and `Float` don't mix quietly, units or not.
- not designed yet: {quoted} on these units, whose answer's unit has no written form, isn't decided yet **(new)**
- refused: `+` adds two points, two places, which has no meaning **(new)**
  - why: a point plus an amount is a point, and a point minus a point is the amount between them; two points aren't added.
- refused: {quoted} has a value with a unit on one side and a plain number on the other **(new)**
  - why: a number with a unit is added only to another of its group, so the unit is written on both, as `2 secs + 500 ms`.
- refused: {quoted} compares a value with a unit and a plain number **(new)**
  - why: a number with a unit is compared only with another of its group, so the unit is written on both.
- refused: this can't be brought to the rate's unit: {why} **(new)**
  - why: a rate times an amount of its own unit gives the rate's unit, as `px per secs` times `secs` gives `px`.

**`crates/seal/src/compile/compiler.rs`**

- not built yet: more than 65,000 top-level names used by procedures (D4) aren't built yet **(new)**
- not built yet: {what} aren't built yet **(new)**

**`crates/seal/src/host/doors.rs`**

- refused (at a door): `{name}` isn't spelt as a type: a capital first, then letters and digits **(new)**
  - why: a type's name joins its words with capitals, as `HighScore`, so a capital always means a type.
- refused (at a door): `{word}` is one of Seal's keywords, so it can't name a door or a label **(new)**
- refused (at a door): `{first}` is one of Seal's own words, so it can't name a door **(new)**
  - why: each word has one meaning.
- refused (at a door): the door `{name}` is opened twice **(new)**
  - why: each door is opened once.
- refused (at a door): only a record or a `one_of` is opened as a type **(new)**
  - why: a host opens its own records and `one_of`s for a script; Seal's own types are there already.
- refused (at a door): the unit group `{name}` is declared twice, or is one of Seal's own **(new)**
  - why: each unit group is declared once; `time`, `angle` and `distance` are Seal's own.
- refused (at a door): the unit `{unit}` is listed twice in `{name}` **(new)**
- refused (at a door): `{unit}`'s size, `{size}`, isn't a size **(new)**
- refused (at a door): `{name}` needs exactly one unit marked `base` **(new)**
- refused (at a door): `{name}` takes {} values, but {} labels are given **(new)**
  - why: each of a door's values has one label, or none.
- refused (at a door): two of `{name}`'s values have the label `{label}` **(new)**
  - why: each label names one argument.
- refused (at a door): `{name}` takes {} values, but {} units are given **(new)**
  - why: each of a door's values has one unit, or `None`.
- refused (at a door): only an `i64` or an `f64` has a unit, but `{door}` is given `{unit}` for another type **(new)**
  - why: a unit says what a number measures, as `secs` or `px per secs`.
- refused (at a door): `{unit}` isn't written as a unit **(new)**
  - why: a unit is written as a script writes it: `secs`, `px.point`, `px per secs`, or `time::secs`.

**`crates/seal/src/syntax/lexer.rs`**

- refused: this `///` has no declaration after it
  - why: a `///` comment describes the declaration on the line after it.
- refused: this `end` closes `comment {name}`, so its note is `\\ {wanted}` (shown with a `fix:` line) **(new)**
- not designed yet: a `\r` that isn't right before a line break isn't decided yet **(new)**
- refused: this blank line is inside an open bracket **(new)**
  - why: a statement carries on over lines only inside its brackets, with no blank line between them.
- refused: {extra} extra blank {lines}, after the one on line {}
  - why: one blank line is enough to separate the parts of a script.
- refused: this line is indented with a tab (shown with a `fix:` line) **(new)**
- refused: this blank line has spaces on it
  - why: a blank line is empty.
- refused: this `///` is inside an open bracket **(new)**
  - why: a `///` comment describes the declaration on the line after it.
- refused: this line carries on a statement, so it's indented {wanted} spaces, one level deeper than the statement's first line **(new)**
- refused: this `comment` is never closed with `end`
  - why: a block comment closes with `end`, at the depth of its `comment` line.
- refused: this `{}` is never closed
- refused: this comment is indented {indentation} spaces, where the line after it puts it at {wanted} **(new)**
- refused: this line has a tab in it **(new)**
  - why: spacing is written with spaces, since a tab can't be told from them.
- refused: `\\` outside text starts a note after `end` **(new)**
  - why: an `end` can carry a note naming its block, as `end \\ proc on_hit`; nothing else uses `\\`.
- refused: this text is never closed with `"`
- refused: `\{shown}` isn't something text can hold **(new)**
  - why: in text, `\` goes only before `"`, `\` or `{`; a line break is `{newln}` and a tab `{tab}`, so `\n` isn't a new line in Seal.
- refused: this text has a tab character in it **(new)**
  - why: a tab can't be told from spaces, so a tab in text is written `{tab}`.
- not designed yet: a comment inside an open bracket isn't decided yet **(new)**
- refused: a statement has no `;`
  - why: the end of a line ends its statement.
- refused: a name has no `$`
  - why: Seal's names are plain words.
- refused: `{other}` is beyond the plain English letters a script is written in **(new)**
  - why: outside text, a script is written in plain English letters, digits and symbols, for now.
- refused: `{other}` can't be read here
- refused: `{}` closes a `{}`
- refused: this `{}` closes nothing
- refused: a type's name has no `_`, so `{word}` is written `{fixed}` **(new)**
  - why: a type's name joins its words with capitals, as `HighScore`.
- refused: the name `{word}` {problem}
- refused: a letter comes right after this number
- refused: `~` goes in front of a number with a point, as in `~0.1`
- refused: a number has no leading zero, so this is written `{shown}` **(new)**
  - why: a number is written without zeros in front, as `7`, not `007`.
- refused: `~` goes in front of a number with a point, as in `~0.1`
  - why: only a `Float` can be inexact.
- refused: this line's spacing isn't the fixed spacing (shown with a `fix:` line) **(new)**

**`crates/seal/src/syntax/parser.rs`**

- refused: this line is indented {} spaces, where its block's body is indented {wanted}
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
- not built yet: blocks nested more than {MAX_BLOCK_DEPTH} deep (D20) aren't built yet **(new)**
- refused: this `end` is at the wrong depth
- refused: the `{}` on line {} has no `end`
- refused: this `end` closes `{}` on line {}, so its note is `\\ {}` (shown with a `fix:` line) **(new)**
- refused: a procedure's parameters go in `[ ]`, not `( )`
  - why: `( )` calls a procedure; `[ ]` describes one.
- refused: a procedure that takes nothing has no `[ ]`: it's written `{written}` **(new)**
  - why: `[ ]` describes the values a procedure takes, and this one takes none.
- refused: the parameter `{}` has no type **(new)**
  - why: a procedure's header writes each parameter's type, as `amount: Whole`, or a group's, as `[hp, amount : Whole]`.
- refused: `labels` has one narrowing, `labels.any_order` **(new)**
  - why: `labels.any_order [ … ]` lets a call write the labelled arguments in any order.
- refused: a group is `group.handler`, a named set of handlers **(new)**
  - why: `group.handler name` declares a handler group; Seal has no other kind of group yet.
- refused: a handler group holds only handlers, each `::name` **(new)**
  - why: a handler group's members are written with the `::` they're reached with, as `::start_over`.
- refused: a unit group holds only units, each `::name` **(new)**
  - why: a unit group's members are written with the `::` they're reached with, as `::cents base`.
- refused: handlers live in a `group.handler`, not a family **(new)**
  - why: a family is a word and its kinds; a handler group is a named set, reached with `::`.
- refused: a family's member is written without `proc` (shown with a `fix:` line) **(new)**
- refused: a `fam.proc` block holds only procedures
  - why: a family holds only the kind of thing it's declared for.
- not designed yet: a `///` before a name made inside a block isn't decided yet **(new)**
- refused: a `{}` is declared at the top level, not inside a block **(new)**
  - why: procedures, families, records and `one_of`s are declared at a script's top level only.
- refused: a `{}` line goes at the top of a procedure's body
  - why: `labels` is the first line of a procedure's body, and `defaults` comes right after it.
- refused: this `{}` has no block to belong to
- refused: this can't be given a value
  - why: `=` gives a value to a name, a field or an item.
- refused: `otherwise` opens a block of cases; on one line, a handler follows `??` (shown with a `fix:` line) **(new)**
- refused: Seal writes `elif`, not `else if`
  - why: `elif` is short for "else if".
- not built yet: values nested more than {MAX_EXPRESSION_DEPTH} deep in one statement aren't built yet **(new)**
- refused: {} comes after a slide's last step
  - why: a slide's step is a procedure, so anything after the slide needs the slide in `( )`.
- refused: two comparisons in a row
  - why: each comparison stands alone; join two with `and`, as in `1 < x and x < 10`.
- not designed yet: `in` after maths written without brackets, which could measure either side, isn't decided yet **(new)**
- refused: `&` doesn't join text; text is joined by filling it in with braces **(new)**
  - why: text is joined only with braces, as in `"Score: {score}"`, which looks like what it shows.
- refused: `{0}` needs `.floor` or `.trunc`, as in `{0}.floor`
  - why: how the fraction goes is always said.
- refused: this line ends where a value is still wanted
  - why: a line carries on only inside an open `(` or `[`, so wrap the value in `( )`.

**`crates/seal/src/vm/machine.rs`**

- check failed: the result is too big for a `Whole`
- check failed: this divides by zero
  - why: nothing can be divided by zero.
- check failed: the result isn't a real number a `Float` can hold
- check failed: there's no item at this place
  - why: an item is reached at a place the list has.
- check failed: there's no character at this place
  - why: a text is reached at a place it has, counting what a person sees as one character.
- check failed: this `Maybe` is `none` **(new)**
  - why: `?? crash` says the `Maybe` can't be `none` here, so the script stops when it is.
- check failed: this value is {side} its type's `{word}: {}` **(new)**
  - why: a ranged number's `min:` and `max:` are hard limits; `min.clamp:` and `max.clamp:` pull a value back instead.
- check failed: this raises a `Whole` to a negative power **(new)**
  - why: a `Whole`'s power is 0 or more, since a negative one isn't a whole number; `be.Float(…)` gives `Float`s.
- check failed: `{name}` takes a value from -1 to 1 **(new)**
  - why: no angle has a sine or cosine past 1 or below -1.
- check failed: this division leaves a remainder
  - why: `/` on two `Whole`s divides exactly; `div.floor` and `div.trunc` say how a fraction goes.
- check failed: a negative number has no square root
  - why: the square root of a negative number isn't a real number.
- check failed: this splits a text by an empty text **(new)**
  - why: an empty text is never found between two pieces; `for each` goes through a text's characters.
- check failed: this range's step is 0
  - why: a range with a step of 0 never reaches its end.
- check failed: this range's step points away from its end
  - why: a range's step points towards its end; a negative `step_by:` counts down.

**`crates/seal/src/vm/numbers.rs`**

- check failed: this divides by zero
  - why: dividing by zero has no answer.
- check failed: the result is too big for a `Decimal` **(new)**
  - why: a `Decimal` keeps 34 digits, up to almost 10 to the 6145; a `HugeDecimal` has no limit.
- check failed: this has no exact decimal answer **(new)**
  - why: a `HugeDecimal` is exact, so `/` and `sqrt` give the exact answer or stop; a `Decimal` rounds to 34 digits.
- check failed: a negative number has no square root
  - why: the square root is taken of a number that isn't negative.
- check failed: this raises a `HugeWhole` to a negative power **(new)**
  - why: a whole number's power is 0 or more, since a negative one isn't a whole number.
- check failed: this division leaves a remainder
  - why: `/` on whole numbers gives the exact answer or stops; `div.floor` and `div.trunc` say how to round.
- check failed: this value is {side} its type's `{word}: {edge_text}` **(new)**
  - why: a ranged number's `min:` and `max:` are hard limits; `min.clamp:` and `max.clamp:` pull a value back instead.
