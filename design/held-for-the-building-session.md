# Held for the building session

What the design session owes Seal Opus Claude, held by Tankun's choice ("No, later", 2026-11-25) to be sent in one
message once the open questions are settled. Removed once sent.

## Answers to send

All of them are now in `design/seal-next-prompt.md` (2026-11-26), for Tankun to paste.

## Since the prompt was pasted

- D160 (no macros or comptime), D161 (generic procedures, `<?Item>`), D162 ("variable", mutable and immutable, in
  messages where they mean one) are in DECISIONS.md; it finds them by Building rule 10.

## Still to settle with Tankun

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
