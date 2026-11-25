# 10. Checks and messages

## Checks

- **At load time:** types, names, unused names, braces in text, doors and narrowings, and every refusal. A script with
  any of these doesn't load (D17).
- **While running:** `Whole` overflow, a bad list position, division by zero, a `Float` becoming infinite or NaN, and
  the limits. A failed check stops the script at its exact line (D14, D17, D18).
- **Floats are checked after every operation,** so every failed float check names its exact line (D18).
- **No check is ever dropped for speed.** One is removed only where it's proved unable to fail, as for a `for_each`
  position or maths on plain numbers (D17).

## Build modes

Seal has none: a script loads and runs one way in every host, with every check on. A host chooses a script's doors and
limits, never whether checking happens (D17).

## Messages

Every message goes to the host two ways (D26):
- **as ready-made text,** in Unio's layout (Unio's standard, chapter 11, Unio D54);
- **as its parts:** its kind (`refused` or `check failed`), file, line, column, and its `what`, `when`, `fix` and `why`.

```
refused: goblin.seal:4:5, line 4 column 5
 4 |     tires = tries + 1
   |     ^^^^^
 what:    `tires` is never used
```

The wording follows Unio's rules: plain sentences, no error codes, no "did you mean" guesses (Unio D54).

**Not decided:** each message's wording; a kind for a script stopped by a limit; how `?? crash`'s message reads in a
script.

## The same everywhere

A script, given the same calls and the same answers from its doors, gives the same results on every machine, bit for
bit, floats included (D50). A maths function Seal adds later must be correctly rounded (D50).

## Testing against Ralph

What Seal shares with Unio is compared against Unio's oracle, Ralph, as a black box, through a translator from Seal to
Unio and a fuzzer (D19).
