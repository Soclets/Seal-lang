# Held for the building session

What the design session owes Seal Opus Claude, held by Tankun's choice ("No, later", 2026-11-25) to be sent in one
message once the open questions are settled. Removed once sent.

## Answers to send

All of them are now in `design/seal-next-prompt.md` (2026-11-26), for Tankun to paste.

## Since the third prompt was pasted (2026-11-27)

All of these are now in `design/wax-rename-prompt.md` (2026-11-28), for Tankun to paste.

- D183, Seal renamed Wax, to carry out on the code side: the crates and the command (your names, suggested
  `wax-lang` on crates.io, where `wax` is taken, and `wax` for the command), `.seal` scripts read as `.wax` and
  compiled files written as `.waxc`, every message and doc in `crates/`, `bench/` and `PROGRESS.md`, and the test
  runner, which must now read `tests/*/program.wax`: the design session renamed all 35 test files, so every test fails
  until it does. The repository becomes `Soclets/Wax-lang`; the folder on disk stays `/Users/ts/Seal` for now.
- D182 to build: a handler that can jump is marked `^`, as `?? _::^give_up`, in place of `*` (D179); `d179` now uses
  it and `d182_jump_mark_caret` is new.
- D181 to build, answering its comment-line reading: a comment on a line of its own may stand between a split slide's
  steps; a blank line may not. `d181_comment_in_slide` tests it.
- Its D180 wordings: fine, but the missing-`;` example, `x = 3 |> doubled;`, is a one-step slide D128 refuses; use a
  chain of three, as `x = 3 |> doubled |> doubled |> be.Text;`.
- Its D179 and D72 draft wordings (its message of 2026-11-27): all fine as drafted, except that a message naming a
  handler writes it as the script did, `level_failed::begin_again` or `_::begin_again`, not always `_::name`.
- Its 256-level crossing limit (Building rule 1): fine as it is.

## Since the second prompt was pasted

- D163 to D171 are in DECISIONS.md too: methods (D163 to D165), requirements (D166 to D168), signing with ML-DSA
  (D169), no type-following verifier (D170, its question answered), and a `.sealc` loading goal against `luajit -b`
  (D171).
- Its 111 newer draft wordings (78cb5bc): reviewed on 2026-11-26 in `design/wording-review-2026-11-26b.md`; sent
  yet.
- D160 (no macros or comptime), D161 (generic procedures, `<?Item>`), D162 ("variable", mutable and immutable, in
  messages where they mean one) are in DECISIONS.md; it finds them by Building rule 10.

- Its D164 build and four draft wordings, reviewed: all fine. One more: calling a method as a plain procedure,
  `is_alive(goblin)`, deserves its own refusal, "`is_alive` is a method of `Enemy`, so it's called on one, as
  `goblin.is_alive()`", not the general "no procedure named". Its two catches in `d164_methods` are fixed (my slips).

- D175 replaces D174: `fips204` is allowed, so D169 is built now. Tankun's weekly reset goes to NginSeal.
- D172 to D174 are in DECISIONS.md: `for` counting loops, D171's bar kept with flat blocks, and no `fips204`: ML-DSA
  comes from a SPARK project starting 2026-11-27, so D169 waits for it (its crate question is answered).

## Still to settle with Tankun


- A method that changes its value, called on a field or a list item (D165 stops there).
- A method that changes its value and can fail: what the lent variable holds after a failure.
- `mut` parameters outside methods (Unio D130's parameter words).
- A host calling a method or a generic procedure by name.
- Generics over units; generic records and `one_of`s.

- A `Whole` with a unit into a `Float` with a unit of its group (asked; recommended yes, whenever exact).
- Unit maths with no written form, as `px * px`, and maths on points beyond adding a span.
- `clamp` with `low:` above `high:`.
- `///` on a name made inside a block.
- A comment after code inside an open bracket.
- Unit prefixes (D111 says to follow Cyborg; Seal's units have none).
- Batch 4 (number types): how a whole `Decimal` shows; `Float` to `HugeWhole` and `HugeDecimal`; units and limits on
  decimals; `HugeWhole` measured in a coarser unit; `be.Decimal` past 34 digits; `be.Float` past a `Float`'s range.
- Its 373 draft wordings: reviewed on 2026-11-26, in `design/wording-review-2026-11-26.md`, none needing Tankun;
  sent the same day.
- Rocq extracted to OCaml for a future oracle (asked; not answered).
