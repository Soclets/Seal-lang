# Seal's implementation: progress

Kept by the building session: "seal claude" built D1 to D73; "seal opus claude" carries it on from 2026-11-25. After a
compaction, read this, `test-status.json` and `git log` to carry on.

**The one command** for the whole build, lint and test run: `cargo run -j 3 --release -p seal-check-all`. It checks
formatting, lints with warnings as errors, builds, runs the packages' own tests, runs the fuzzer and the comparison
with Ralph, runs every test in `tests/` as `tests/README.md` describes, and writes `test-status.json`.

**The benchmarks:** `sh bench/run.sh` builds Seal's benchmark host in each way D58's kinds need, runs every benchmark
on Seal, Luau, LuaJIT and Node, checks every answer, and writes `bench/RESULTS.md`.

## The finish line (from the prompt), and where each point stands

1. **Four parts work.** The `seal` library, the `seal` command, the fuzzer, and the Unio translator with the
   comparison against Ralph all work. The language server is an empty package.
2. **Decisions built.** D1 to D178 are built (D178 only names this implementation Ngin1), except the language server's
   parts and what's listed under "Not built yet" below. All 31 tests in `tests/` pass, from text and from compiled files.
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

## Plan from 2026-11-26 (seal opus claude): D152 to D159

From the prompt in `design/seal-next-prompt.md`, in this order, each step committed and pushed:

- [x] **C15. Catching up.** D157's `return`, `include` and `instead`; D158's `eq` and `not_eq` as keywords, with `==`
  and `!=` gone; D156's licence on every crate (set once in the workspace); D131's missing case after `??` worded
  as the handler's; D135 checked on `bench/programs/` (nothing to change).
- [x] **C16. Big numbers against Python (D155, D159).** Fixed benchmarks for `HugeWhole`, `Decimal` and
  `HugeDecimal`, Python's `int` and `decimal` as the bar, `gmpy2` for reference; faster than Python on each, at 48% to
  88% of its time.
- [x] **C17. `.sealc` files (D141, D153, D154).** `seal build`, bundles, loading by name, the verifier, the cache;
  fuzzing damaged and tampered files; a loading row with LuaJIT's `string.dump` bytecode beside it.
- [x] **C18. The C API (D136, D151, D152).** `seal-c`, a C test program under AddressSanitizer, fuzzing through it, a
  door-call row with LuaJIT's C API beside it. Luau's C library isn't installed; Tankun was asked whether it may be
  built in `target/`.

## Decisions built since D58

Each arrived while building; each is built, tested, fuzzed and pushed.

- **D59** `==` and `!=` kept: nothing to change. (Replaced by D158's `eq` and `not_eq`.)
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
- **D140** The loading rework: names and texts borrowed from the script, one token list, kept lists and tables, and a
  value with no operator after it read without going down the operators' levels. Loading a patrolling script from a
  text of its own went from 12.3 µs to 4.84 µs, faster than Luau's 4.91 µs.
- **D142** LuaJIT's interpreter, `luajit -joff`, is measured as goal 1's second bar, and LuaJIT's JIT for reference.
  It runs the Luau programs as they are. Not met yet: `fib(32)` and loading each patrolling script from its own text.
- **D143** Already built so: the new test passed as it came.
- **D144, D145, D146** `for each` is written `for_each`, one keyword; `for` and `each` alone aren't keywords any
  more. (D144's `use` and D145's `ret` and `incl` were taken back by D157.) The Unio translator writes Unio's words. A `for_each`
  block's `end` note is `\\ for_each`, as the design session answered.
- **D147** `_` on the right of an `=` that changes something is its current value. A name's `_` reads the name; a
  part's reads the part through the same way in, with every position and key worked out once under a hidden name
  first (a change to a name's own item then goes through a way in with no steps). On a `Dict`'s key, `_` is a lookup,
  so it fails with `missing` like one, and is written `(_ ?? 0) + 1`. Anywhere else, `_` is refused (draft wording).
  The fuzzer writes `_` in changes to names, counters, fields and parts of parts.
- **D148** Measuring is written `to`, as `total to secs`, in braces too; `to` isn't a keyword, since `to:` is also
  `range`'s label.
- **D149** Maths measured with `to` without brackets is refused, with the bracketed line as the fix (draft wording).
- **D115's speed and outside checks** (asked for by the design session with Tankun's OK): Karatsuba multiplication
  from 64 base-2^32 digits, division through a Newton reciprocal from 768 (put right exactly, falling back to long
  division if that ever takes more than a few steps), and text both ways in halves past 48, each switch-over
  measured. `tests/reference.rs` checks whole numbers, `Decimal` (as decimal128) and `HugeDecimal` against Python;
  `tests/gmp.rs` checks whole numbers from 60 to 12,000 digits against GMP 6.3, through a small C program built at
  test time, so no binding and no `unsafe` reaches Seal. Each says so and compares nothing where Python, a C compiler
  or GMP isn't installed.
- **D150** Loading a script from a text of its own is judged against Luau alone; `bench/RESULTS.md` shows LuaJIT's
  time beside it for reference.
- **D151** The C API gives every message whole and in parts; `crates/seal-c/PROPOSAL.md` names the functions.
- **D156** Every crate is under Apache-2.0 WITH LLVM-exception, set once in the workspace's `Cargo.toml`.
- **D160** No macros and no comptime: nothing to build.
- **D161** Generic procedures, `proc first<?Item>[items: List<?Item>] -> Maybe<?Item>`. `?Item` is lexed as one type
  name, `?` and all; `<` touching a procedure's name with `?` after it opens its type parameters. Checked once with
  each type parameter a type of its own, equal only to itself. Each call fills them in with fresh types worked out
  from its arguments or where its result goes, and a call nothing fills in is refused. After checking,
  `check/generics.rs` makes an instance, a copy with the types filled in and names of its own, for each set of
  types the calls use, from the top level and every procedure that isn't generic, then from the instances
  themselves; that's what's compiled, so a generic procedure runs as fast as one written for its types. A
  generic procedure itself compiles to a stub a host can't call yet. Maths, comparing, converting, showing or
  reaching into a type parameter's value stops as not designed yet; a list of them can still be gone through,
  reached and counted. The fuzzer's Seal scripts have four generic procedures, one calling another, called with
  `Whole`s, texts, lists and `Maybe`s; the Unio translator leaves scripts with generics out.
- **D162** Messages call a named value a variable, mutable with `mut` and immutable without, and keep "name" for
  the spelling itself.
- **D163, D164** Methods: `proc`s inside a record or `one_of`, after its fields or cases, called only with a dot.
  Each is declared under its type, out of the global names, so two types can each have `size`; its first parameter
  is a value of the type. A dot call on a value whose type is plain to see (a variable, a record being built, a
  field, a call's result) finds the method and passes the value as the first argument; anything else goes to Seal's
  own methods. A host calling a method stops as not designed yet. The fuzzer's `Hero`, `Spot` and `Tree` have
  methods, `size` on two of them.
- **D165** A method whose first parameter is `mut` is lent the mutable variable it's called on: the caller hands
  the value over, the method doesn't free that parameter when it ends, and the caller takes the value back into the
  variable, in a register, a top-level variable or a record kept in registers. Calling one on an immutable variable
  is refused with D2's fix; a `mut` first parameter never changed is refused (D135), and one that's changed counts
  as used. A method that reaches the lent top-level variable itself, or through what it calls, is refused. `mut` on
  any other parameter, a lending method called on anything but a variable, and one that can fail stop as not
  designed yet. The fuzzer's `Hero` has `heal`, called on mutable variables.
- **D166, D167** A type parameter's requirement after `:`. With none, its values are kept, passed, given back,
  compared with `eq` and `not_eq` and shown in braces; `?Item: Number` adds maths, ordering, `abs`, `min` and
  `max`, and each call's type is checked to be a `Whole`, `Float`, `HugeWhole`, `Decimal` or `HugeDecimal` with no
  unit. `meets [ … ]` is spaced as `handles [ … ]`; `meets` isn't a keyword. The fuzzer has a `Number` generic.
- **D168** `requirement Name … end` lists methods' first lines, `?Self` the type meeting it. Each listed method is
  a stub procedure generic over `?Self`, so `thing.score()` on a type parameter's value is a generic call, which the
  instance for a type points at that type's own method. `meets [ … ]` checks every listed method: same name, types
  (`?Self` as the type), labels, result and written failures; each call to a generic checks its types meet what its
  type parameters require. The fuzzer's `Hero` meets a `Powered` requirement, used by a generic.
- **D141, D153, D154** `.sealc` files (`compile/file.rs`, `host/compiled.rs`): `seal build a.seal [b.seal …]
  [-o out.sealc]` and `seal::build` write one script or a bundle with an index; `Script::load_compiled` loads one by
  name; `Cache` and `Script::load_cached` keep them in a folder the host names, keyed by text, this Seal and the
  doors, falling back to the text when a cache file is bad. Each load checks this exact Seal's fingerprint (its
  version and the source of what decides a script's meaning), the doors' key, a checksum, every count against the
  file and every text as UTF-8; then the verifier checks every register against its frame, every jump inside its
  procedure, every table number (constants, texts, layouts, sites, globals, procedures, failures) and the type
  table (all in range, no loops, at most 256 deep). How instructions are written and read is made by
  `compile/make_instruction_file.py` from `bytecode.rs`. The one command runs every test from compiled files too;
  the fuzzer loads every script compiled (exactly the same as from text), damaged (always refused) and tampered
  with a remade checksum (never harming the host). Not done: following each register's kind, so a tampered file
  that swaps registers can still pass and stop with an internal message, which then says the file may have been
  changed. `bench/RESULTS.md` has a loading row, with LuaJIT's bytecode beside it.
- **D136, D151, D152: the C API** (`crates/seal-c`, `include/seal.h`), the one package where `unsafe` is allowed.
  Doors, scripts and values are handles: numbers into tables kept per thread, each with a generation, so a freed,
  made-up or wrong-kind handle is refused as a host mistake, never trusted. A door is a C function with the host's
  pointer; its arguments are an array of plain tagged values (`seal_arg`), `Whole`, `Float`, `Bool` and `Nothing`
  inline and anything else as a value handle lent for the call, and its kinds are worked out when it's opened. A
  door call through C takes about 11 ns, against 8 ns for a Rust door. Every reader of a value says when it's the
  wrong kind (`SEAL_HOST_MISTAKE` with a message), and messages are kept per thread, read whole or in parts (D151)
  until the thread's next call. Every function is guarded, so a panic inside Seal comes back as a status, never
  across into C. `tests/api_test.c` runs under AddressSanitizer from `cargo test` (built with the system's `cc`; it
  says so and compares nothing where there's none); `tests/fuzz.rs` makes 20,000 random calls, with handles made up,
  freed or of the wrong kind, arguments of any kind and bits, damaged scripts and doors that answer anything.
- **D172** `for i from 1 to 3`, `before`, and an optional `step_by`, read with `range`'s words and no brackets; the
  checked tree's counting loop, which `for_each` over a written range used to make, is what `for` makes. A step of 0
  or counting away from the end is refused where it's written as numbers, and a failed check when the loop starts
  otherwise. `for_each` over `range[…]` is refused with the `for` line as the fix (the loop is still checked, so
  nothing else is refused because of it), and `for … in` with the `for_each` line. On a `for` line a bare `to` ends
  the first number, so measuring there is bracketed, and `-` after `from`, `to`, `before` or `step_by` is a sign. A
  counter never read is marked `for unused i` (sent to the design session to check). The benchmarks' programs, the
  fuzzer and the tests count with `for`.
- **D173** A script loaded from a `.sealc` keeps its bytes as one block; each instruction's place, the procedures and
  their names, the failures' names, the pause maps and the source text stay in it, checked at load without being
  built, and are read the first time they're wanted (`Kept` and `ScriptText`). Loading a compiled patrolling script
  makes 8 allocations, from 26, in about 0.60 µs. LuaJIT's is 0.28 to 0.32 µs; the design session has the numbers,
  where the rest goes, the floor (about 0.4 µs) and a recommendation, as D173 asks. Leaving the type table in the block
  too was tried and saved nothing, so it's read at load.
- **D170** No type-following verifier: what `.sealc` loading checks (this Seal, the doors, a checksum, every text,
  register, jump and table number in range) stays as it is, and the VM's checks keep the host safe whatever a file
  holds.
- **D171** Loading a compiled patrolling script went from 2.1 µs to 0.83 µs: a checksum taking four words at a time
  and no second hash of the source; Seal's own unit groups one flag in a file and Seal's own failures a number; names
  and the source read where they are; the index read without making a list; pause maps in two flat lists; and a
  verifier that makes nothing for each instruction or type. LuaJIT loads its own bytecode in 0.28 µs. What's left is
  mostly the compiled script's ~25 separate lists, each an allocation; the design session has the numbers and a
  recommendation. The fuzzer caught a script's own unit groups being lost by the new format; a test covers it.
- **D169, D175, D177: signed `.sealc` files** (`host/signing.rs`), ML-DSA-44 from `fips204` without its
  random-number feature. `seal keygen NAME` writes `NAME.public` and `NAME.secret` from the system's randomness;
  `seal build --sign NAME.secret` appends the signature over the whole file and `SEALSIG\x01`; `seal run --key
  NAME.public` loads only a file signed with it. The secret key is written readable by its owner only, never over
  another, and `seal keygen` says to keep it off repositories. A host calls `Doors::require_signature` (C:
  `seal_doors_require_signature`); a file that isn't signed, is changed in any byte, or is signed with another key is
  refused, and each file that passes is kept whole, so it's checked once in a run (about 43 µs). A host that doesn't
  require signatures loads a signed file as it is. The fuzzer signs one script in four and checks a change after
  signing, or another key, is always refused.
- **D176** Loading a compiled file is judged within twice LuaJIT's time. The same script's block, byte for byte, loaded
  again with the same doors reuses what was read and verified (the last 8 files kept, matched by checksum, then byte
  for byte), in about 0.15 µs; the benchmark's files differ, so its row still measures loading.
- **The second wording review** (`design/wording-review-2026-11-26b.md`) is built: what a type parameter's
  requirement doesn't allow is refused ("`?Item` has no requirement that allows `+`"), the C API's host mistakes name
  their function and parameter with no why, and "already names a type or a requirement" says each name means one
  thing. The C test builds the static library itself.
- **D157, D158** `return`, `include` and `instead` again, and `eq` and `not_eq` as keywords in place of `==` and
  `!=`, which no longer read as one symbol. `ret`, `incl` and `use` are plain names again. Messages, the fuzzer and
  `bench/programs/` use the new words; the Unio translator already wrote Unio's.
- **D115** A `HugeWhole`'s limits may be past a `Whole`'s range: such an edge is kept once in the type table's big
  edges, so equal limits are one type, and it's checked, clamped to and shown as any edge is.
- **D115, D116, D119, D124 to D127: `HugeWhole`, `Decimal`, `HugeDecimal`.** A `Decimal` is held in its heap
  entry itself, in two 64-bit halves, so working with one allocates nothing beyond the entry; the money benchmark
  went from 224 ms to 172 ms (16 times the `Float` sums' 11 ms). `crates/seal/src/decimal.rs` holds
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

`bench/RESULTS.md` has every number; it was re-run on 2026-11-26 at about 3:30 PM, every goal met but D171's, which
came after. Loading a patrolling script from a text of its own: 4.67 µs against Luau's 4.78 µs, a thin margin. On
2026-11-25 (evening), Normal: `fib` 30 to 33 ms, n-body 64 ms,
spectral-norm 17.8 ms, fannkuch 102 ms, word count 26.5 ms, text building 343 µs, binary-trees 99 ms, patrol run
2.2 ms. Loading a 12-line script from a text of its own: about 4.45 µs (12.3 µs this morning).

In short, judged Normal against Normal (D58), against both of goal 1's bars, Luau's interpreter and, by D142,
LuaJIT's interpreter (`luajit -joff`, running the Luau programs as they are):

- **Faster than both on every benchmark:** `fib`, n-body, spectral-norm, fannkuch, word count, text building,
  binary-trees, the patrolling scripts as a whole run, resuming, and loading from the same text.
- **Loading a script from a text of its own: met against Luau, not against LuaJIT.** About 4.45 µs against Luau's
  4.85 µs and LuaJIT's 1.79 µs. D140's rework got it from 12.3 µs: names and texts borrowed from the script, one token
  list, lists and tables kept between scripts on a thread, a value with no operator read without the operators'
  levels, short lists of names before hash tables, and passes that skip what needs nothing. What's left is spread
  across reading (about 1.5 µs), checking (1.6 µs) and compiling (0.8 µs), with no single place over a few percent.
  LuaJIT reads and compiles in one pass with no types to check; matching it would take a front end rebuilt as one
  pass. D150 judges this row against Luau alone, with LuaJIT's shown for reference; D141's bytecode files are the
  fast way to load many scripts.
- **`fib` and the run loop.** Calls were LuaJIT's interpreter's strong point. Three things got `fib` from 49.9 ms to
  about 30 ms, each general: a call's value lands where it's wanted with no move after it, a call with one argument
  written as a `Whole` plus a small number is one instruction, as is `ret a + b`; the loop hands its rarer
  instructions (dictionaries made or emptied, text's pieces, a list's length changing, door calls) to functions
  outside it; and only what nearly every instruction uses has a name of its own in the loop, so the processor keeps
  the instruction list and the frame's registers in its own registers. How the compiler lays the loop out still moves
  `fib` by up to 5% from one build to the next.
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
- **D171, D176, loading a compiled file within twice LuaJIT's time for its own bytecode: not met.** 0.66 µs against
  LuaJIT's 0.29 µs (2.25×; side by side, best of several, about 0.61 µs). From 2.1 µs this afternoon: D173's block,
  the lists a dropped script recycles, one look at each instruction when verifying, Seal's own type table as one flag.
  The same file loaded again reuses what was verified, in about 0.15 µs.
- **`fib` against LuaJIT's interpreter: met, 32.0 against 33.6 ms (95%; about 29.5 ms side by side).** After D173 it
  fell to LuaJIT's time with the same instructions: D173's lists left in a file made each about 120 bytes inline in a
  compiled script, which changed how the whole run loop was compiled (about 10% more cycles). With those lists behind
  one shared allocation, the script is near its old size and `fib` is back, at the cost of about 37 ns on loading a
  compiled file. Memory placement was ruled out first: shifting it moved neither build out of its range. LLVM's
  alignment flags didn't help, and a table for a base case's comparison made it slower (the branch is always
  predicted), so neither is in.
- **Loading from a text of its own against Luau: met, thinly** (98% to 100% across today's runs).
- **D155, faster than Python's built-ins on the big number types: met** on all six, at 46% to 90% of Python's time.
- **Faster than V8 `--jitless` on `fib` and n-body, within twice on word count and text building:** met.
- **A 10,000-line script loads in under 100 ms:** met, about 5.5 ms (its generated script used `&`, which D77
  refuses, so it went unmeasured for a while; it joins with braces now).
- **Footprint:** a paused patrolling script holds 512 bytes of its cap, about 1.8 KB of real memory.
- **Pausing costs about as much as a call:** met; resuming is cheaper than a host call.
- **Ceil.program and Ceil.machine.program** need `llvm-profdata` to merge the profile. Rust's own isn't installed,
  and Tankun was asked on 2026-11-23 whether it may be; meanwhile the one that comes with Xcode turned out to read
  Rust's profiles, so nothing needed installing, and `bench/run.sh` uses whichever is there. One profile is taken
  from all eight benchmarks' Normal runs, and every Ceil.program row uses the VM built with it.
- **Ceil.source programs:** only n-body has one. For the others the Normal program stands in, and each such row says so;
  the Benchmarks Game's hand-tuned Lua and JavaScript programs weren't fetched.

## Plan from 2026-11-27 (seal opus claude): D171's last row and what's left unbuilt

From `design/seal-next-prompt.md`, each step committed and pushed:

- [x] **C19. A change at a place works out its type from a later line (D5).** `name[pos: n] = v`, and a place on the
  way in to a deeper change, on a name whose type a later line settles, as after a generic call: it's a list or an
  array, either changed the same way, refused with the known-type wordings if the later line says otherwise.
- [x] **C20. Types that hold themselves cross the doors (D72),** both ways, from text and from compiled files. A host
  declares one with `#[derive(Seal)]`, through `Vec`, `Option`, `HashMap` or `Box`, and the description names the
  type where it's met again (`HostType::Named`) rather than going on for ever. Copies are made with a list of what's
  left rather than by recursion; at most 256 levels cross (Building rule 1). A host type that can never end is refused
  as a script's is.
- [x] **C21. `_::` (D179).** A group's members are declared `_::name`, a call's handler is `?? _::name` or
  `include _::name`, and a bare `::` is refused with the line, `_` added, as the fix. Messages name members as
  `_::name`.
- [x] **C23. `;` ends a statement holding a slide (D180).** A split slide carries on into lines starting with `|>`,
  indented one level deeper; a missing `;` is refused at the slide's last step with the `;` in the fix; a `;`
  anywhere else is refused.
- [ ] **C22. D171 within twice LuaJIT's time** in the full benchmark run. In a loop of 1,000 different files: about
  650 ns a load this morning; 515 ns with Seal's own type table shared rather than copied and compared again; 474 ns
  with plain crossings read on the spot and an eight-lane checksum.

## Questions for the design session

Sent 2026-11-25 (batch 1 of this session, 21 questions): all answered by D119 to D129 and built, and n-body's Normal
program is rightly the one written with D62's direct changes.

Batch 2, sent 2026-11-25 after C12 (C7 to C10's first three were answered by D130 to D132):
- D108: a negative power of a `Whole` only known while running is a failed check (built), as D89 treats an empty
  separator.
- D107: `clamp` with `low:` above `high:` (stops). (D109's `atan2` of (0, 0) is answered by D133.)
- D91 lists the declarations `///` goes on; a name made inside a block isn't among them (stops).
- A comment after code inside an open bracket, as `foo(a, // the first`: D91 allows a comment line there (stops).
- (Answered: D134 for the angle built-ins' units; D121's example was a slip, noted under it.)
- A `Whole` with a unit becomes a `Float` only when its unit changes: `w: Float[unit: secs] = 2000 ms` passes but
  `= 2 secs` is refused, as a plain `Whole` into a `Float` is (D16). Is that wanted, or should the second pass too?
- (Answered: D149, maths measured with `to` is bracketed, built.)
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

Sent 2026-11-25 on its own: whether D57's "loading each from a text of its own" counts as met within twice Luau's
time, since Seal type-checks before running and Luau doesn't (recommended), or the reader, checker and compiler get
the rework (names as numbers, trees in one block) that closing the rest needs.

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

- a host calling a generic procedure, which would need its types worked out from the host's values (D161)
- a host calling a method (D164)
- `mut` on a parameter other than a method's first (Unio D130's parameter words); a method that changes its value
  called on something other than a variable, or able to fail (D165)
- a `\r` that isn't right before a line break
- a comment after code inside an open bracket
- a `///` before a name made inside a block
- a `Dict` key that isn't a `Text`, a `Whole` or a `Bool` (chapter 6: not decided)
- a handler used inside its own cases
- maths on units whose answer's unit has no written form, as `px * px`, and maths on points beyond `±` a span
- a `clamp` whose `low:` is more than its `high:` (it stops while running too)
- converting a `Float` to a `HugeWhole` or a `HugeDecimal`, which D126 doesn't list
- units and limits on a `Decimal` or `HugeDecimal`
- a `HugeWhole` with a unit measured in a coarser unit, which isn't whole

## Not built yet

- A generic procedure filled in with more than 10,000 sets of types, or with types nested more than 100 deep, as
  one calling itself with a list of what it was given (D161).
- More than 65,000 top-level names used by procedures, registers in one procedure, or constants.
- Blocks nested more than 48 deep, and values nested more than 160 deep in one statement (bounded so that reading a
  script can't overrun the host's stack).
- Values nested more than 256 deep crossing between a host and a script (D72), counting every list, `Maybe`, record
  and case: Seal copies them without recursion, but the host's own code drops, copies, compares, prints and converts
  a `Value` one level at a time on its stack, about 2 KB a level in a debug build.
- The language server (D47): an empty package, as the prompt says.

## Draft wordings

Refusals, failed checks and stops whose wording no decision gives, drafted in Unio's layout (Unio D54): `what:` says
what's wrong, `why:` gives the rule once. Where a fix can be shown as a whole line, `fix:` stands in place of `what:`
(Unio D101), as for indentation, spacing, a `return`'s header, a `mut` that goes or one that's missing; those are
marked.

**New on 2026-11-27 (C19 to C23), sent to the design session:** C23's, for D180:
- refused, a missing `;`: fix: the slide's last line with `;` added
  - why: `|>` points on to the next step, so a `;` says where the slide ends.
- refused, a `;` on a statement with no slide: fix: the line without it
  - why: the end of a line ends a statement; a `;` marks where a slide ends, since `|>` points on.
- refused, a `;` inside brackets or with more after it: a `;` comes only at the end of a statement holding a slide
  - why: the end of a line ends a statement, and a `;` marks where a slide ends; two statements never share a line.

C21's, for D179:
- refused, a bare `::`: fix: the line with `_` before the `::`
  - why: `::` reaches into a group, so something always stands on its left: the group's name, or `_` for the one assumed from where it stands.
- refused: a handler group holds only handlers, each `_::name` / a unit group holds only units, each `_::name`
  - why: a handler group's (a unit group's) members are written with the `_::` that reaches them from inside it, as `_::start_over` (`_::cents base`).
- Every message naming a member writes it `_::name`, as "`level_failed` has no handler `_::begin_again`".

C19 reuses the wordings a change with its type known already has. C20's, for D72:
- refused, as a script loads with the host's types: the host's `{name}` holds itself with no way for a value to end
  - why: (the script's own: a type can hold itself only where a value can end, …)
- refused, a host's call: argument {n} of `{procedure}` is nested more than 256 deep, too deep to go into a script
- refused, a resume: this task was resumed with a value nested more than 256 deep, too deep to go into a script
- host's mistake, a door's value: a door gave a value nested more than 256 deep, too deep to go into a script
- check failed, a script's value going to its host: this value is nested more than 256 deep, too deep to go to the host
  - why: a value reaches its host as a copy, which the host's own code goes through one level at a time on its stack, and that holds only so many.

422 wordings, taken from the code by `crates/seal-check-all/wordings.py` on 2026-11-26, after both of the design
session's reviews (`design/wording-review-2026-11-26.md` and `-26b.md`) were built. The ones marked **(changed
2026-11-26)** differ from the list the first review read (384d500), as that review and D157, D158 and D115 changed
them. The 29 marked **(new since the second review)** came after it, and were sent to the design session: refusals for
what a type parameter's requirement doesn't allow, and the C API's host mistakes as the second review reworded them.
`{…}` stands for what the message fills in; a `why:` held in a constant (as a `Whole`'s and a `Float`'s range) isn't
picked up by the script, so it doesn't show here.

**`crates/seal/src/check/calls.rs`**

- refused: this isn't a procedure, so it can't be called
  - why: `( )` calls a procedure.
- refused: this is a chain of {length} calls, each inside the next, and three or more are written as a slide
  - why: a slide reads in the order it runs, as in `points |> double |> be.Text |> show_msg`.
- refused: `{word}` is a variable, not a procedure, so it can't be called
  - why: `( )` calls a procedure.
- refused: `sqrt` takes a `Float`, but this is {found}
  - why: `sqrt` gives a `Float`'s square root; a number written with a point, as `9.0`, is a `Float`.
- refused: a `range` is written with `[ ]`, as in `range[from: 1, to: 3]`
  - why: `( )` calls a procedure, and a range runs nothing.
- refused: there's no procedure or door named `{word}`
- refused: `{word}` has no narrowing `.{narrowing}`
  - why: `{word}` takes an angle in `rads`, `degs` or `turns`, as `{word}(90 degs)`, or a plain `Float` in radians.
- refused: `{word}` takes a `Float`, but this is {found}
  - why: angles and `lerp` work on `Float`s; a number written with a point, as `2.0`, is a `Float`.
- refused: `{word}` takes values of one type, but this is {right} and the first is {left}
  - why: `Whole` and `Float` don't mix quietly: `be.Float(…)` turns a `Whole` into a `Float`.
- refused: `{word}` takes an angle, but this is {found}
  - why: an angle is in `rads`, `degs` or `turns`, or a plain `Float` in radians.
- refused: `{word}` takes values of one type, but this is {right} and the first is {left}
  - why: the two are `Float`s of one unit group, or both plain.
- not designed yet: a `clamp` whose `low:` is more than its `high:` isn't decided yet
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
- refused: there's no conversion to `{target}` **(changed 2026-11-26)**
  - why: `be` converts to `Text`, `Whole`, `Float`, `Bool`, `HugeWhole`, `Decimal` and `HugeDecimal`; other values are built as they're written.
- refused: there's no `{full}`
  - why: `be`'s narrowings are `be.Text`, `be.Float`, `be.Whole`, `be.Bool`, `be.HugeWhole`, `be.Decimal`, `be.HugeDecimal`, `be.Decimal.shown` and `be.Decimal.nearest`, and `.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` after `be.Whole` or `be.HugeWhole`.
- refused: there's no conversion from {found} to a `Bool` **(changed 2026-11-26)**
  - why: a number is compared, as `n not_eq 0`, for a `Bool`.
- refused: this is already a `{target}`, so `be.{target}` does nothing **(changed 2026-11-26)**
  - why: it gives back the same value, so the `be.` can go.
- refused: there's no conversion from {written} to a `Bool` **(changed 2026-11-26)**
  - why: `be.Bool` reads `"true"` or `"false"` from text; a number is compared, as `n not_eq 0`, for a `Bool`.
- refused: there's no conversion from {written} to a `Float`
  - why: `be.Float` turns a `Whole` into a `Float`.
- refused: `{full}` takes a `Float`, but this is {written}
  - why: `.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` say how a `Float`'s fraction goes.
- refused: `be.Whole` on a `Float` needs to say how the fraction goes **(changed 2026-11-26)**
  - why: the ways are `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.
- refused: there's no conversion from {written} to a `Whole`
  - why: `be.Whole` reads a `Whole` from text, and turns a `Float` into one with `.floor`, `.trunc`, `.ceil`, `.nearest` or `.nearest.even`.
- not designed yet: calling `{}`, which changes its value, on something other than a variable isn't decided yet
- not designed yet: `{}` changing its value and able to fail, which leaves the variable it's lent undecided isn't decided yet
- refused: {found} has no method `has`
- refused: `.has` takes `key:` and its value
- refused: `{name}` splits a text, but this is {found}
- refused: `{name}` takes `by:` and the text to split at
- refused: this splits a text by an empty text **(changed 2026-11-26)**
  - why: an empty text is never found between two pieces; `for_each` goes through a text's characters, as `for_each letter in name`.
- refused: {found} has no method `{}`
- refused: `.add` takes the item, then `pos:` or `offset:` when it goes at a place
- refused: `.add` takes the item to add
- refused: `.remove` takes `pos:` or `offset:` and its number
- refused: `.remove` takes `key:` and its value
- refused: an `Array` has no `.{}`
  - why: an array holds exactly its length, so nothing is added or taken away; its items can still change.
- not built yet: working out a type from a later line (D5), which this `.{}` needs, isn't built yet

**`crates/seal/src/check/expressions.rs`**

- refused: `{digits}` is too big for a `Whole`
  - why: a `Whole` goes up to 9223372036854775807.
- refused: `_` is the value of what an `=` changes, and nothing is changed here **(changed 2026-11-26)**
  - why: on the right of an `=` that changes a variable, `_` is what's on the left, as `score = _ + 10`.
- refused: a `{name}` is built with `[ ]`, naming every field
  - why: a record is built as `Name[field: value]`.
- refused: `{name}` is a type, not a value
- refused: `{path}` is a procedure, and a call writes `( )`
  - why: every call writes `( )`, even with nothing inside.
- refused: there's no variable `{word}` here
- refused: `.len.{}` is a text's length, but this is {found}
  - why: only a text's length comes in graphemes, code points and bytes.
- refused: `[:]` can't say what the dictionary holds, so the variable's type has to be written
  - why: a type is written where the value can't say it, as in `name: Dict<key: Text, value: Whole> = [:]`.
- refused: `-{digits}` is too small for a `Whole`
  - why: a `Whole` goes down to -9223372036854775808.
- refused: `?` hands a failure to a caller, and the top level has none
  - why: a failure at the top level is handled on the spot, with `??` or `otherwise`.
- refused: `?` follows something that can't fail
  - why: `?` passes a failure up, so it goes after a call that can fail.
- refused: a unit stands right after its number, as `2 secs`, or in a type, as `Float[unit: secs]`
  - why: a unit says what a number counts, so it goes with one.
- refused: `places:` takes a whole number written out, but this is `{places}`
  - why: `places:` says how many digits a `Float` shows after its point, as `{speed, places: 2}`.
- refused: `{{inside}}` measures in `{unit}`, which isn't a unit's name **(changed 2026-11-26)**
  - why: `to` in braces measures a variable in one of its group's units, as `{total to secs}`.
- refused: `{{inside}}` isn't a variable with its fields and facts
  - why: braces in text hold a variable, which may be followed by its fields and facts, never a calculation or a call; a `Float` can add `, places: 2`.
- refused: there's no variable `{first}` here
  - why: a variable in braces is checked when the script loads; a brace shown as it is, is written `\{`.
- refused: `places:` shows a `Float` to so many digits, but this is {found}
  - why: `places:` is only for a `Float`, which otherwise shows every digit it needs.
- refused: `{text}` is too big for a `Float`
  - why: a `Float` goes up to about 10 to the 308.
- refused: `{sign}{text}` can't be stored exactly as a `Float`; the inexact number is written `{sign}~{text}`
  - why: a `~` marks a `Float` that's only close to what's written.
- refused: `{text}` is stored exactly as a `Float`, so it takes no `~`
  - why: a `~` marks a `Float` that's only close to what's written.
- refused: `{name}` is marked `unused`, so it can't be read
  - why: `unused` marks a variable that's never read on purpose.
- refused: `{name}` fills in braces in text, so it's written `"{{name}}"`
  - why: `{newln}` and `{tab}` give a line break and a tab inside text; the text holding just one is written `"{newln}"`.
- refused: `{name}` is a procedure, and a call writes `( )`
  - why: every call writes `( )`, even with nothing inside.
- refused: there's no variable `{name}` here
- refused: {operator} takes {wanted}, but this is {found}
- refused: {quoted} can't mix a `Whole` and a `Float`
  - why: `Whole` and `Float` don't mix quietly: `be.Float(…)` turns a `Whole` into a `Float`, and a number written with a point, as `2.0`, is a `Float`.
- refused: {quoted} has {left_text} on its left, but {right_text} on its right
- refused: a `Whole` raised to a negative power isn't a whole number
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
- refused: `[]` can't say what the list holds, so the variable's type has to be written
  - why: a type is written where the value can't say it, as in `name: List<Text> = []`.
- refused: `[all: value]` is only allowed where the length is known, as for an `Array`
  - why: `[all: value]` takes its length from the type, as in `board: Array<Whole>[len: 64] = [all: 0]`.
- refused: the key {plain} is written twice in this dictionary
  - why: a dictionary holds one value for each key, so one of the two would vanish.
- refused: this call gives nothing, so there's no value to stand in for
  - why: after a call that gives nothing, a failure is dealt with by `?? ignore`.
- refused: `?? ignore` follows a call that gives a value
  - why: with the failure ignored there'd be no value to carry on with; `?? ignore` goes after a call that gives nothing.
- refused: a handler deals with a call's failures, and a `Maybe` isn't a failure
  - why: a `Maybe` is given a value to use when it's `none`, as `?? 0`, or `?? crash` when it can't be `none`.
- refused: `?? ignore` would leave nothing in place of a `Maybe` that's `none`
  - why: a `Maybe` is given a value to use when it's `none`, as `?? 0`, or `?? crash` when it can't be `none`; `?? ignore` goes after a call that gives nothing.
- not built yet: working out a type from a later line (D5), which this `??` needs, isn't built yet
- refused: `??` follows something that can't fail and isn't a `Maybe`
  - why: `??` deals with a failure, or gives a fallback when a `Maybe` is `none`.
- refused: `otherwise` follows something that can't fail
  - why: `otherwise` handles each failure of the call before it.
- refused: the failure `{name}` is handled twice
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
- refused: {what} {}
- refused: this `otherwise` has a `case` for {}, which the call can't fail with
  - why: the call can fail with {}.
- refused: `{method}` reaches `{}` itself, so it can't be called on it
  - why: while a variable is lent to a method that changes it, nothing else touches it; the method works on the value it's given.
- refused: `{called}` can change `{name}`, which this `for_each` is going through **(changed 2026-11-26)**
- refused: this can pause, while a `for_each` is going through the top-level variable `{name}`
- refused: `{name}` has no requirement that allows {action} **(new since the second review)**
  - why: a type parameter with no requirement takes any type, so only what every type can do is allowed; `<{name}: Number>` allows maths and ordering. **(new since the second review)**
- refused: `{name}` requires a `Number`, which doesn't allow {action} **(new since the second review)**
  - why: a `Number` allows maths and ordering, and a requirement listing methods allows those methods. **(new since the second review)**
- refused: `{name}` requires `{required}`, which doesn't allow {action} **(new since the second review)**
  - why: a requirement allows the methods it lists, and nothing else that not every type can do. **(new since the second review)**
- refused: {purpose}, but this is {found}
- refused: two texts aren't put in order with `<`, `>`, `at_least` or `at_most`
  - why: text has no one right order: by character codes `"Zebra"` comes before `"apple"`, and alphabetical order differs between languages.
- refused: this is already a `Text`, so `be.Text` does nothing **(changed 2026-11-26)**
  - why: it gives back the same value, so the `be.` can go.
- refused: a `Dict`'s value is reached with `key:`
- refused: this is {found}, but {} is {wanted}
- refused: what `{name}` gives back can't be worked out from its body
  - why: a procedure's result comes from its `return`s; where they can't say, it's written with `-> Type`.
- refused: this value's type can't be worked out
- refused: `{name}`'s type can't be worked out
- refused: `{}` can fail with {}, but its header doesn't say so (shown with a `fix:` line)
- refused: `{}`'s written failures don't match: {body} (shown with a `fix:` line)
- refused: this pauses, and the top level can't pause
  - why: the host waits for loading to finish; a pause belongs in a procedure the host calls.

**`crates/seal/src/check/generics.rs`**

- not built yet: a generic procedure filled in with more than 10,000 sets of types isn't built yet
- not built yet: a generic procedure filled in with types nested more than 100 deep, as one calling itself with a list of what it was given, isn't built yet
- refused: this call can't work out what `{name}` stands for in `{}`
  - why: a call works its types out from its arguments, or from where its result goes, as `top: Maybe<Whole> = first(scores)`; a type is written where nothing says it.

**`crates/seal/src/check/handlers.rs`**

- refused: the handler group `{}` is declared twice
  - why: each handler group is declared once.
- refused: `{}` is already a top-level variable, so a handler group can't be named `{}`
- refused: the handler `::{}` is listed twice in `{}`
  - why: each handler in a group has a name of its own.
- refused: there's no procedure or door named `{full}` for `{}` to handle
- refused: `{full}` can't fail, so `{}` has nothing to handle for it
  - why: `handles [ … ]` names the procedures, doors and built-ins that can fail, whose failures the group's handlers deal with.
- refused: there's no handler group named `{}`
- refused: `::{member}` could be {}
  - why: when two groups that handle the call both have the handler, the group is written, as `group::name`.
- refused: `{}` has no handler `::{member}`
  - why: `{}`'s handlers are {}.
- not designed yet: a handler used inside its own cases isn't decided yet
- refused: `{}` isn't one of `::{member}`'s parameters
- refused: `{}` isn't one of `::{member}`'s labels
- refused: the failure `{}` is handled twice
  - why: a handler has one `case` for each failure.
- not designed yet: a `mut` parameter other than a method's first, whose words Unio D130 leaves open isn't decided yet
- refused: `{}` is never used
  - why: a handler's parameter that's never read on purpose is marked, as `unused {}`.

**`crates/seal/src/check/mod.rs`**

- refused: `{}` is one of the host's types, so the script can't declare it
  - why: the host opens its own types for a script, which uses them by name, as it uses a door.
- refused: `{}` is one of Seal's own types, so a script can't declare it
  - why: each type has one meaning.
- refused: the type `{}` is declared twice
  - why: each type is declared once.
- refused: the case `{}` is listed twice
  - why: each case is listed once.
- refused: `{}` has no cases, so no value of it could exist
  - why: a `one_of`'s value is exactly one of its cases, listed one per line.
- refused: the field `{}` is listed twice
  - why: each field is listed once.
- refused: `{}`'s field `{field}` holds {found}, with no way for a value to end
- refused: every case of `{}` holds more that can't end, so no value of it could end
- refused: `{}` has no setting `{setting}`
  - why: a number's settings are its limits: `min:`, `max:`, `min.clamp:` and `max.clamp:`.
- refused: the {edge} edge is written twice
  - why: each edge is written once, with `min:` or `min.clamp:`, and `max:` or `max.clamp:`.
- refused: this number's lowest edge is above its highest
  - why: a ranged number's `min` is at most its `max`, so some value fits.
- refused: `{name}` isn't an immutable top-level variable made from a number written out
  - why: a number's limits are known while loading: a number written there, or an immutable top-level variable made from one.
- refused: this number's limit has to be a `HugeWhole`, written out **(changed 2026-11-26)**
  - why: a number's limits are known while loading, and are of its own type.
- refused: this number's limit has to be {wanted}, written out **(changed 2026-11-26)**
  - why: a number's limits are known while loading, and are of its own type.
- refused: this is {side} its type's `{word}: {shown}` **(changed 2026-11-26)**
  - why: a ranged number's `min:` and `max:` are hard limits, and one written past them never fits; `min.clamp:` and `max.clamp:` pull a value back instead.
- refused: `{digits}` is too big for a `Whole`
- refused: a type's setting is a whole number known while loading, never a calculation or a call
  - why: a type's setting is a whole number written there, or an immutable top-level variable made from one, as `max_items = 10`.
- refused: `{name}` isn't one of this procedure's type parameters
  - why: a type parameter is declared in `< >` right after its procedure's name, as `proc first<?Item>[items: List<?Item>]`, and used inside that procedure.
- refused: `{name}` is a type parameter, so it takes nothing in `< >` or `[ ]`
  - why: a type parameter stands for a whole type, which each call fills in.
- not designed yet: units and limits on a `{name}` isn't decided yet
- refused: `{name}` has no setting `{setting}`
- refused: `{setting}` can't be less than 0
- refused: `{setting}` is written twice
- refused: `{name}` holds one type, written without a label, as `List<Text>` **(changed 2026-11-26)**
  - why: a type that holds one type needs no label, since nothing can be mixed up.
- refused: an `Array` says how many items it holds, as in `Array<Whole>[len: 64]`
  - why: an array's length is part of its type.
- refused: a `Dict` labels the two types it holds, `key:` and `value:`, as `Dict<key: Text, value: Whole>` **(changed 2026-11-26)**
  - why: a type that holds more than one type labels each.
- refused: there's no type named `{name}`
- refused: a `Float` can't be a `Dict`'s key
  - why: two neighbouring floats that print alike would be different keys.
- not designed yet: {written} as a `Dict`'s key isn't decided yet
- refused: `{}` is a narrowing, so it's declared inside a `fam.proc` block
  - why: a word and its kinds are declared together.
- refused: `{}` isn't `{}` or a kind of it
  - why: each member of a family is the word itself, or reads as "a Y kind of X" for its name `X.Y`.
- refused: `{name}` already names a type or a requirement
  - why: each name means one thing. **(new since the second review)**
- refused: there's no requirement named `{}`
  - why: a type meets requirements the script declares with `requirement`.
- refused: `{type_name}` meets `{}`, but has no method `{name}`
  - why: a type meeting a requirement has every method the requirement lists.
- refused: `{type_name}`'s `{name}` isn't the one `{}` lists: {why}
  - why: a method meeting a requirement takes the same types, with `?Self` as the type, the same labels, and gives the same result and failures.
- refused: `{type_name}` has two methods named `{word}`
  - why: each of a type's methods has a name of its own.
- refused: a method's name is one word, but this is `{word}`
  - why: a method is called with a dot after its value, as `goblin.is_alive()`, so a narrowing would read as a field.
- refused: `{word}`'s first parameter isn't a value of `{type_name}`, the type it's a method of
  - why: a method's first parameter is the value before the dot, as `goblin` in `goblin.is_alive()`.
- refused: `{word}` is one of Seal's own words, so a procedure can't be named `{word}`
- refused: `{word}` is one of the host's doors, so a procedure can't be named `{word}`
- refused: the procedure `{name}` is declared twice
- refused: two type parameters are named `{}`
  - why: each type parameter has a name of its own.
- refused: there's no requirement named `{}`
  - why: a type parameter requires `Number`, or a requirement the script declares with `requirement`.
- refused: two parameters are named `{}`
  - why: each parameter has a name of its own.
- refused: `{}` isn't in `{name}`'s parameters or result
  - why: a call works a type parameter out from its arguments, or from where its result goes, so it's used in one of them.
- refused: `{}` isn't one of `{name}`'s parameters
- refused: two parameters have the label `{}`
  - why: each label names one argument.
- refused: `{}` is given two labels
- refused: `{}` isn't one of `{name}`'s labels
  - why: only a labelled parameter can have a default.
- refused: this default is {found}, but `{}` is {wanted}
- refused: `{}` has two defaults
- refused: a default is a value written out plainly, never a calculation or a call
  - why: a default is the same every time: a number, text, a `Bool`, a list, a record, a case, or an immutable top-level variable made from one.
- refused: `{name}` is reached here before the top level has made it
- refused: a name can't be `all`
  - why: `[all: value]` always means a list or array whose every item starts as `value`, so `all` can't name anything else.
- refused: `{word}` is already {taken_by}, so nothing else can be named `{word}`
- refused: `{}` is already a top-level variable, so {what} can't be named `{}`
- refused: the case `{}` has no value `{}`
  - why: a case's values come out by name.
- refused: the case `{}`'s values are taken out in the order they're declared: `{}`
  - why: a case's values come out in the order its type declares them, skipping any that aren't read.
- refused: `{}` already exists, so it can't be made again
- refused: `{name}` is never changed, so it's made without `mut` **(changed 2026-11-26)**
  - why: `mut` marks a mutable variable, one that changes; one that never does is made without it.
- refused: `{name}` is never used
- refused: `{name}` is never used
  - why: a parameter that's never read on purpose is marked, as `unused {name}`.
- refused: `{name}` is never used
  - why: a `for_each` variable that's never read on purpose is marked, as `for_each unused {name}`.
- refused: `{name}` is never used
  - why: an `at` variable that's never read is marked, as `at key unused {name}`, or the `at` part is left out.
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
- refused: this line follows a `match` that can leave
- refused: this line works out {found}, but nothing uses it
  - why: a line that only works out a value does nothing; keep the value in a variable, or give it to a mutable variable with `=`.
- refused: this call gives {found}, but nothing uses it
  - why: a call that gives a value is used: kept in a variable, passed on, or compared; `unused name = …` drops it on purpose.
- refused: this `break` isn't inside a loop
  - why: `break` leaves the innermost loop it's in.
- refused: `fail` hands a failure to a caller, and the top level has none
- refused: this `instead` isn't inside an `otherwise` case
  - why: `instead` gives the value an `otherwise` case carries on with.
- refused: this is {found}, but {role} is {wanted_text}
- refused: `{word}` checks a `Bool`, but this is {found}
  - why: a number or text doesn't stand for true or false; compare it{hint}.
- refused: `{}` compares two `Float`s in a `while` condition **(changed 2026-11-26)**
  - why: a `Float` that changes each pass rarely lands exactly on a value, so the loop could never stop; `<`, `>`, `at_least` or `at_most` say "until it reaches".
- refused: `{kind}` leaves out what it holds, and the value doesn't show it
  - why: a written type leaves out only what its value shows, so this one writes it, as `{example}`.
- refused: `Maybe.none` can't say what the `Maybe` holds, so the variable's type has to be written
  - why: a type is written where the value can't say it, as in `name: Maybe<Whole> = Maybe.none`.
- refused: this gives nothing, so there's no value for the variable
  - why: a procedure that gives nothing back can't be used as a value.
- refused: `{name}` is being gone through by a `for_each`, so it can't change here **(changed 2026-11-26)**
  - why: what a `for_each` goes through can't change inside it.
- refused: `{name}` can't be given a new value (shown with a `fix:` line) **(changed 2026-11-26)**
- refused: {name} is {found}, which has no fields to change
- refused: {found} has no field `{}`
- refused: a list's item is reached with `pos:` or `offset:`
  - why: `pos:` counts from 1 and `offset:` from 0, so a bare number is never guessed at.
- refused: a dictionary's value is reached with `key:`
- not built yet: working out a type from a later line (D5), which this change needs, isn't built yet
- refused: {name} is {found}, which has no items to change
- refused: a change reaches into a mutable variable, and this isn't a variable
  - why: a change is a variable changing its own value.
- refused: this is {found}, which has no fields
- refused: a change can't go through a `Dict`'s key on its way, since the key might be missing
  - why: a lookup is dealt with on its own line: the value is taken out, as `mut north = shops[key: "north"] ?? crash`, changed, and put back, as `shops[key: "north"] = north`.
- refused: this is {found}, which has no items
- refused: `return` hands a value to a caller, and the top level has none
- refused: this `return` gives back {}, but the procedure's header has no `-> Type`
- refused: this `return` gives a value, but an earlier one gives none
- refused: this gives nothing, so there's nothing to return
  - why: a procedure that gives nothing back can't be used as a value.
- refused: this `return` gives {found}, but the procedure gives {wanted}
- refused: this `return` gives no value, but the procedure gives one
- refused: `at key` is for going through a `Dict`
- refused: `at {word}` is for going through a list; a `Dict` is gone through `at key` (shown with a `fix:` line)
- refused: `at key` is for going through a `Dict`, but this is {found}
- refused: a `range` takes `from:`, one of `to:` or `before:`, and an optional `step_by:`
  - why: `to:` includes its number and `before:` stops short of it.
- refused: this range's step is 0
  - why: a range with a step of 0 never reaches its end.
- refused: this range counts by {step} from {from}, away from its end, {end}
  - why: a range's step points towards its end; a negative `step_by:` counts down.
- not built yet: working out a type from a later line (D5), which this `match` needs, isn't built yet
- refused: `match` reads which case a value is, but this is {found}
  - why: only a `one_of` or a `Maybe` has cases.
- refused: {found} has no case `{}`
- refused: the case `{}` is handled twice
  - why: a `match` has one `case` for each case.
- refused: this `match` has no `case` for {}
  - why: a `match` covers every case; there's no catch-all.

**`crates/seal/src/check/numbers.rs`**

- refused: a `{name}` holds exactly what's written, so its number takes no `~`
  - why: a `~` marks a `Float` that's only close to what's written; a decimal counts in tens, so `0.1` is exact.
- refused: `{text}` has more digits than a `Decimal` keeps
  - why: a `Decimal` keeps 34 digits; a `HugeDecimal` keeps any number of them.
- refused: {quoted} has {left_type} on its left, but {right_type} on its right
- refused: {quoted} can't mix a `{left_type}` and a `{right_type}`
  - why: a number becomes a bigger type by itself only where it always fits; `be.` converts the other ways, and a `Float` mixes with none of these.
- refused: `**` on a `{}` takes a whole power, but this is {found}
  - why: a `HugeWhole`, `Decimal` or `HugeDecimal` is raised to a `Whole` power.
- refused: a `HugeWhole` to a negative power isn't a whole number **(changed 2026-11-26)**
  - why: a negative power is one over the number, as `2 ** -3` is `1 / 8`; a `HugeDecimal` holds it when its digits end, as `0.125` does.
- refused: there's no `{full}`
  - why: `be.HugeWhole` takes `.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` for a decimal, and `be.Decimal` takes `.shown` and `.nearest` for a `Float`.
- refused: there's no conversion from {found} to a `{name}`
  - why: a number type converts from text and from the other number types.
- refused: this is already a `{name}`, so `be.{name}` does nothing **(changed 2026-11-26)**
  - why: it gives back the same value, so the `be.` can go.
- not designed yet: converting a `Float` to a `{name}` isn't decided yet
- refused: `be.Decimal` on a `Float` needs to say which `Decimal` it becomes **(changed 2026-11-26)**
  - why: `be.Decimal.shown` takes the number the `Float` shows, so `~0.1` gives `0.1`; `be.Decimal.nearest` takes the `Decimal` nearest the value it holds.
- refused: `{full}` takes a `Float`, but this is a `{}`
  - why: `.shown` and `.nearest` say which `Decimal` a `Float` becomes.
- refused: `be.HugeWhole` on a `{}` needs to say how the fraction goes **(changed 2026-11-26)**
  - why: the ways are `be.HugeWhole.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even`.
- refused: `{full}` rounds a fraction, and a `{}` has none
  - why: `be.HugeWhole` converts a whole number or text with nothing to round.
- refused: `be.Whole` on a `{source}` needs to say how the fraction goes **(changed 2026-11-26)**
  - why: the ways are `be.Whole.floor`, `be.Whole.trunc`, `be.Whole.ceil`, `be.Whole.nearest` and `be.Whole.nearest.even`.
- refused: `{full}` rounds a fraction, and a `HugeWhole` has none
  - why: `be.Whole` converts a `HugeWhole`, failing with `too_big` when it doesn't fit.

**`crates/seal/src/check/units.rs`**

- refused: the unit group `{}` is declared twice, or is one of Seal's own
  - why: each unit group is declared once; `time`, `angle` and `distance` are Seal's own.
- refused: `{}` has {bases} units marked `base`, where it needs exactly one
  - why: a unit group's base is the unit every other one is counted in, so there's exactly one.
- refused: the unit `::{}` is listed twice in `{}`
  - why: each unit in a group has a name of its own.
- refused: `{}` isn't a unit listed above in `{}`
  - why: each unit is counted in one listed before it, so every size is known.
- refused: a unit's size is a number written out
  - why: a unit is so many of an earlier one, as `::dollars = 100 cents`.
- refused: a unit's size can't be 0
- refused: `{}` could be {}
  - why: when two groups have a unit of the same name, the group is written before it, with `::`.
- refused: a rate isn't a point
  - why: `.point` marks a place counted from a fixed one, as `px.point`; a rate, as `px per secs`, is an amount.
- refused: this is {found}, but {wanted_text} goes here, and {why}
  - why: units of one group convert by themselves only where nothing is lost: into a `Whole`, a finer unit; into a `Float`, any unit with a fixed size.
- refused: this plain number goes where `{unit_text}` is wanted
  - why: where something takes a unit, the number says it, so seconds and milliseconds, or degrees and radians, can't be mixed up.
- refused: this has a unit, `{unit_text}`, where a plain number goes **(changed 2026-11-26)**
  - why: a value with a unit gives its plain number when it's measured in one, as `value to {unit_text}`.
- refused: `to` measures a value with a unit, but this is {found} **(changed 2026-11-26)**
  - why: `value to unit` gives the plain number a value with a unit is, as `total to secs`.
- refused: a value in `{from_text}` isn't measured in `{to_text}`
  - why: only a unit of the value's own group measures it, and only one with a fixed size, which `frames` and `tiles` haven't.
- not designed yet: a `HugeWhole` measured in a unit it isn't always a whole number of isn't decided yet
- refused: `{operator}` has {left_text} on its left, but {right_text} on its right
  - why: units of different groups, as seconds and pixels, don't go together, and a `Whole` and a `Float` don't mix quietly.
- refused: this can't be brought to the other side's unit for `{operator}`: {why}
  - why: units of one group combine where they convert exactly, in the finer unit.
- refused: {quoted} can't mix a `Whole` and a `Float`
  - why: `Whole` and `Float` don't mix quietly, units or not.
- not designed yet: {quoted} on these units, whose answer's unit has no written form, isn't decided yet
- refused: `+` adds two points, and two places can't be added **(changed 2026-11-26)**
  - why: a point plus an amount is a point, and a point minus a point is the amount between them; two points aren't added.
- refused: {quoted} has a value with a unit on one side and a plain number on the other
  - why: a number with a unit is added only to another of its group, so the unit is written on both, as `2 secs + 500 ms`.
- refused: {quoted} compares a value with a unit and a plain number
  - why: a number with a unit is compared only with another of its group, so the unit is written on both.
- refused: this can't be brought to the rate's unit: {why}
  - why: a rate times an amount of its own unit gives the rate's unit, as `px per secs` times `secs` gives `px`.

**`crates/seal/src/compile/compiler.rs`**

- not built yet: more than 65,000 top-level variables used by procedures (D4) aren't built yet
- not built yet: {what} aren't built yet

**`crates/seal/src/host/doors.rs`**

- refused (at a door): `{name}` isn't spelt as a type: a capital first, then letters and digits
  - why: a type's name joins its words with capitals, as `HighScore`, so a capital always means a type.
- refused (at a door): `{word}` is one of Seal's keywords, so it can't name a door or a label
- refused (at a door): `{first}` is one of Seal's own words, so it can't name a door
  - why: each word has one meaning.
- refused (at a door): the door `{name}` is opened twice
  - why: each door is opened once.
- refused (at a door): only a record or a `one_of` is opened as a type
  - why: a host opens its own records and `one_of`s for a script; Seal's own types are there already.
- refused (at a door): the unit group `{name}` is declared twice, or is one of Seal's own
  - why: each unit group is declared once; `time`, `angle` and `distance` are Seal's own.
- refused (at a door): the unit `{unit}` is listed twice in `{name}`
- refused (at a door): `{unit}`'s size, `{size}`, isn't a whole number above 0 **(changed 2026-11-26)**
- refused (at a door): `{name}` needs exactly one unit marked `base`
- refused (at a door): `{name}` takes {} values, but {} labels are given
  - why: each of a door's values has one label, or none.
- refused (at a door): two of `{name}`'s values have the label `{label}`
  - why: each label names one argument.
- refused (at a door): `{name}` takes {} values, but {} units are given
  - why: each of a door's values has one unit, or `None`.
- refused (at a door): only an `i64` or an `f64` has a unit, but `{door}` is given `{unit}` for another type
  - why: a unit says what a number measures, as `secs` or `px per secs`.
- refused (at a door): `{unit}` isn't written as a unit
  - why: a unit is written as a script writes it: `secs`, `px.point`, `px per secs`, or `time::secs`.

**`crates/seal/src/host/script.rs`**

- not designed yet: a host calling `{procedure}`, a method, which a script calls only with a dot isn't decided yet
- not designed yet: a host calling `{procedure}`, a generic procedure, which needs its types worked out from the host's values isn't decided yet

**`crates/seal/src/syntax/lexer.rs`**

- refused: this `///` has no declaration after it
  - why: a `///` comment describes the declaration on the line after it.
- refused: this `end` closes `comment {name}`, so its note is `\\ {wanted}` (shown with a `fix:` line)
- not designed yet: a `\r` that isn't right before a line break isn't decided yet
- refused: this blank line is inside an open bracket
  - why: a statement carries on over lines only inside its brackets, with no blank line between them.
- refused: {extra} extra blank {lines}, after the one on line {}
  - why: one blank line is enough to separate the parts of a script.
- refused: this line is indented with a tab (shown with a `fix:` line)
- refused: this blank line has spaces on it
  - why: a blank line is empty.
- refused: this `///` is inside an open bracket
  - why: a `///` comment describes the declaration on the line after it.
- refused: this line carries on a statement, so it's indented {wanted} spaces, one level deeper than the statement's first line
- refused: this `comment` is never closed with `end`
  - why: a block comment closes with `end`, at the depth of its `comment` line.
- refused: this `{}` is never closed
- refused: this comment is indented {indentation} spaces, where the line after it puts it at {wanted}
- refused: this line has a tab in it
  - why: spacing is written with spaces, since a tab can't be told from them.
- refused: `\\` outside text starts a note after `end`
  - why: an `end` can carry a note naming its block, as `end \\ proc on_hit`; nothing else uses `\\`.
- refused: this text is never closed with `"`
- refused: `\{shown}` isn't something text can hold
  - why: in text, `\` goes only before `"`, `\` or `{`; a line break is `{newln}` and a tab `{tab}`, so `\n` isn't a new line in Seal.
- refused: this text has a tab character in it
  - why: a tab can't be told from spaces, so a tab in text is written `{tab}`.
- not designed yet: a comment inside an open bracket isn't decided yet
- refused: a statement has no `;`
  - why: the end of a line ends its statement.
- refused: a name has no `$`
  - why: Seal's names are plain words.
- refused: `{other}` is beyond the plain English letters a script is written in
  - why: outside text, a script is written in plain English letters, digits and symbols, for now.
- refused: `{other}` can't be read here **(changed 2026-11-26)**
- refused: `{}` closes nothing open here; the `{}` still open closes with `{closer}` **(changed 2026-11-26)**
- refused: this `{}` closes nothing
- refused: a type's name has no `_`, so `{word}` is written `{fixed}`
  - why: a type's name joins its words with capitals, as `HighScore`.
- refused: the name `{word}` {problem}
- refused: a letter comes right after this number (shown with a `fix:` line) **(changed 2026-11-26)**
- refused: `~` goes in front of a number with a point, as in `~0.1`
- refused: a number has no leading zero, so this is written `{shown}`
  - why: a number is written without zeros in front, as `7`, not `007`.
- refused: `~` goes in front of a number with a point, as in `~0.1`
  - why: only a `Float` can be inexact.
- refused: this line's spacing isn't the fixed spacing (shown with a `fix:` line)

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
- not built yet: blocks nested more than {MAX_BLOCK_DEPTH} deep (D20) aren't built yet
- refused: this `end` is at the wrong depth
- refused: the `{}` on line {} has no `end`
- refused: this `end` closes `{}` on line {}, so its note is `\\ {}` (shown with a `fix:` line)
- refused: a requirement holds only methods' first lines
  - why: a requirement lists the methods a type meeting it has, as `proc score[item: ?Self] -> Whole`, each with no body.
- refused: a type parameter is written with `?` in front, as `?{}`
  - why: `?` marks a type that each call fills in, so it can't be mistaken for a real type.
- refused: a procedure's parameters go in `[ ]`, not `( )`
  - why: `( )` calls a procedure; `[ ]` describes one.
- refused: a procedure that takes nothing has no `[ ]`: it's written `{written}`
  - why: `[ ]` describes the values a procedure takes, and this one takes none.
- refused: the parameter `{}` has no type
  - why: a procedure's header writes each parameter's type, as `amount: Whole`, or a group's, as `[hp, amount : Whole]`.
- refused: `labels` has one narrowing, `labels.any_order`
  - why: `labels.any_order [ … ]` lets a call write the labelled arguments in any order.
- refused: a group is `group.handler`, a named set of handlers
  - why: `group.handler name` declares a handler group; Seal has no other kind of group yet.
- refused: a handler group holds only handlers, each `::name`
  - why: a handler group's members are written with the `::` they're reached with, as `::start_over`.
- refused: a unit group holds only units, each `::name`
  - why: a unit group's members are written with the `::` they're reached with, as `::cents base`.
- refused: handlers live in a `group.handler`, not a family
  - why: a family is a word and its kinds; a handler group is a named set, reached with `::`.
- refused: a family's member is written without `proc` (shown with a `fix:` line)
- refused: a `fam.proc` block holds only procedures
  - why: a family holds only the kind of thing it's declared for.
- refused: {what} comes after a method
  - why: {belong} come first, then its methods, so a type's data reads before what it does.
- not designed yet: a `///` before a variable made inside a block isn't decided yet
- refused: a `{}` is declared at the top level, not inside a block
  - why: procedures, families, records and `one_of`s are declared at a script's top level only.
- refused: a `{}` line goes at the top of a procedure's body
  - why: `labels` is the first line of a procedure's body, and `defaults` comes right after it.
- refused: this `{}` has no block to belong to
- refused: this can't be given a value
  - why: `=` gives a value to a variable, a field or an item.
- refused: `otherwise` opens a block of cases; on one line, a handler follows `??` (shown with a `fix:` line)
- refused: Seal writes `elif`, not `else if`
  - why: `elif` is short for "else if".
- not built yet: values nested more than {MAX_EXPRESSION_DEPTH} deep in one statement aren't built yet
- refused: {} comes after a slide's last step
  - why: a slide's step is a procedure, so anything after the slide needs the slide in `( )`.
- refused: two comparisons in a row
  - why: each comparison stands alone; join two with `and`, as in `1 < x and x < 10`.
- refused: this is maths measured with `to`, without brackets **(changed 2026-11-26)**
  - why: `to` has no place in maths' order, so the maths it measures is in brackets, as `(a + b) to ms`.
- refused: `&` doesn't join text; text is joined by filling it in with braces
  - why: text is joined only with braces, as in `"Score: {score}"`, which looks like what it shows.
- refused: `{0}` needs `.floor` or `.trunc`, as in `{0}.floor`
  - why: how the fraction goes is always said.
- refused: this line ends where a value is still wanted
  - why: a line carries on only inside an open `(` or `[`, so wrap the value in `( )`.

**`crates/seal/src/vm/machine.rs`**

- check failed: the result is too big for a `Whole`
- check failed: this divides by zero
  - why: dividing by zero has no answer.
- check failed: the result isn't a real number a `Float` can hold
- check failed: there's no item at this place
  - why: an item is reached at a place the list has.
- check failed: there's no character at this place
  - why: a text is reached at a place it has, counting what a person sees as one character.
- check failed: this `Maybe` is `none`
  - why: `?? crash` says the `Maybe` can't be `none` here, so the script stops when it is.
- check failed: this value is {side} its type's `{word}: {}`
  - why: a ranged number's `min:` and `max:` are hard limits; `min.clamp:` and `max.clamp:` pull a value back instead.
- check failed: this raises a `Whole` to a negative power
  - why: a `Whole`'s power is 0 or more, since a negative one isn't a whole number; `be.Float(…)` gives `Float`s.
- check failed: `{name}` takes a value from -1 to 1
  - why: no angle has a sine or cosine past 1 or below -1.
- check failed: this splits a text by an empty text **(changed 2026-11-26)**
  - why: an empty text is never found between two pieces; `for_each` goes through a text's characters.
- check failed: this division leaves a remainder
  - why: `/` on two `Whole`s divides exactly; `div.floor` and `div.trunc` say how a fraction goes.
- check failed: a negative number has no square root
  - why: the square root of a negative number isn't a real number.
- check failed: this range's step is 0
  - why: a range with a step of 0 never reaches its end.
- check failed: this range's step points away from its end
  - why: a range's step points towards its end; a negative `step_by:` counts down.

**`crates/seal/src/vm/numbers.rs`**

- check failed: the result is too big for a `Decimal`
  - why: a `Decimal` keeps 34 digits, up to almost 10 to the 6145; a `HugeDecimal` has no limit.
- check failed: this has no exact decimal answer
  - why: a `HugeDecimal` is exact, so `/` and `sqrt` give the exact answer or stop; a `Decimal` rounds to 34 digits.
- check failed: this raises a `HugeWhole` to a negative power
  - why: a whole number's power is 0 or more, since a negative one isn't a whole number.
- check failed: this division leaves a remainder
  - why: `/` on whole numbers gives the exact answer or stops; `div.floor` and `div.trunc` say how to round.
- check failed: this value is {side} its type's `{word}: {edge_text}`
  - why: a ranged number's `min:` and `max:` are hard limits; `min.clamp:` and `max.clamp:` pull a value back instead.

**`crates/seal-c/src/lib.rs`**

- host mistake (C API): `{function}`: `{parameter}` is null **(new since the second review)**
- host mistake (C API): `{function}`: `{parameter}` isn't UTF-8 **(new since the second review)**
- host mistake (C API): `{function}`: `{parameter}` is null, but `{counted_by}` says there are {count} **(new since the second review)**
- host mistake (C API): `{function}`: `{parameter}` is null, so there's nowhere to write {what} **(new since the second review)**
- host mistake (C API): `{function}`: a {what} handle that's freed, or was never one **(new since the second review)**
- host mistake (C API): `seal_doors_add`: `function` is null **(new since the second review)**
- host mistake (C API): `{ADD}`: `value_types[{place}]` is {why} **(new since the second review)**
- host mistake (C API): `{ADD}`: `result_type` is {why} **(new since the second review)**
- host mistake (C API): `seal_doors_labels`: there's no open door named `{name}` **(new since the second review)**
- host mistake (C API): `seal_script_load`: `source` isn't UTF-8 **(new since the second review)**
- host mistake (C API): `{function}`: the script is running, and was called again from one of its doors **(new since the second review)**
- host mistake (C API): `{CALL}`: `args[{place}]` is {why} **(new since the second review)**
- host mistake (C API): `seal_script_resume`: `value` is {why} **(new since the second review)**
- host mistake (C API): `seal_value_text`: `text` isn't UTF-8 **(new since the second review)**
- host mistake (C API): `seal_value_number`: `kind` is {kind}, but it makes only `SEAL_HUGE_WHOLE`, `SEAL_DECIMAL` or `SEAL_HUGE_DECIMAL`; a plain number goes in a `seal_arg` itself **(new since the second review)**
- host mistake (C API): `seal_value_number`: `{written}` isn't a number written as a script writes one **(new since the second review)**
- host mistake (C API): `seal_value_list`: `items[{}]` is {why} **(new since the second review)**
- host mistake (C API): `seal_value_maybe`: `inner` is {why} **(new since the second review)**
- host mistake (C API): `seal_value_get_text`: `value` is {}, which has no text; it reads a `SEAL_TEXT` or a big number **(new since the second review)**
- host mistake (C API): `seal_value_count`: `value` is {}, which has no items **(new since the second review)**
- host mistake (C API): `seal_value_item`: `position` is {position}, but `value` has {count} items, counted from 0 **(new since the second review)**
- host mistake (C API): `seal_value_item`: `value` is {}, which has no items **(new since the second review)**
