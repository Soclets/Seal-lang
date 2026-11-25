# Held for the building session

What the design session owes Seal Opus Claude, held by Tankun's choice ("No, later", 2026-11-25) to be sent in one
message once the open questions are settled. Removed once sent.

## Answers to send

- D130 to D149, in DECISIONS.md. D144 (`use`), D145 (`ret`, `incl`) and D146 (`for_each`) change eight tests, which fail until built;
  the benchmark programs in `bench/programs/` need the same spellings. D140 needs no message (the rework is done now, as it planned); D136, D138 and D141
  are for after the current work; D142 (LuaJIT's interpreter as a bar) is part of the current work. LuaJIT is
  installed through Homebrew.
- Batch 1's "still open" 13, 14, 17 and 21 are already answered: D126, D127, D128 and D129. 16 is its own call.
- Batch 2 and 3, readings the design session gives itself:
  - A negative power of a `Whole` known only while running is a failed check, as D89 has it (built; right).
  - A value with a unit shows as written, as `2500 ms` (D100; right).
  - Its reading of D120's "can't be exact" (Whole only to a finer unit, Float any fixed ratio, rounded once) is right.
  - D121's example was a slip, noted under it: `= 100.0 px.point`.
  - D131: two included handlers that both have a case for a failure the call can't give don't handle it twice, since
    neither case can run there.
- Its D146 question: yes, a `for_each` block's note is `end \\ for_each`, naming the block by its opening keyword
  (D102, D146).
- Its refusal for D131's case says "this `otherwise`" even after `??`.
- D135: check the benchmark programs in `bench/programs/` for `mut` names that are never changed.
- An idea, its own call (D57): Unio's fib trick from `Beta++`, inlining a procedure with no side effects into itself
  where its calls repeat, then merging them, so `fib(n - 1) + fib(n - 2)` becomes `2 × fib(n - 2) + fib(n - 3)`. It
  costs analysis at load, so it may suit D141's ahead-of-time compiling best.

## Still to settle with Tankun

- A `Whole` with a unit into a `Float` with a unit of its group (asked; recommended yes, whenever exact).
- Unit maths with no written form, as `px * px`, and maths on points beyond adding a span.
- `clamp` with `low:` above `high:`.
- `///` on a name made inside a block.
- A comment after code inside an open bracket.
- Unit prefixes (D111 says to follow Cyborg; Seal's units have none).
- Batch 4 (number types): how a whole `Decimal` shows; `Float` to `HugeWhole` and `HugeDecimal`; units and limits on
  decimals; `HugeWhole` measured in a coarser unit; `be.Decimal` past 34 digits; `be.Float` past a `Float`'s range.
- Its 373 draft wordings (PROGRESS.md at 384d500): the design session reviews them, bringing only unsure ones to
  Tankun.
- Rocq extracted to OCaml for a future oracle (asked; not answered).
