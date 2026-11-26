# The design session's review of the draft wordings

Of the 373 drafts in `PROGRESS.md` (the list taken from the code on 2026-11-25), reviewed on 2026-11-26 by the design
session, as Unio D132 has it: most read well and stand as drafted. The changes below are the design session's own
calls; none needed Tankun. Build them, and draft any new wording the same way.

## Everywhere

- **Today's spellings:** every `ret` becomes `return`, every `use` that ends a case becomes `instead` (D157), and every
  `!=` and `==` becomes `not_eq` and `eq` (D158), as in `n not_eq 0`.
- **A `why:` never just repeats the refusal.** "Converting a value to the type it already is, is refused" (for
  `be.Text`, `be.{target}` and `be.{name}` on a value already of that type) becomes: "it gives back the same value, so
  the `be.` can go."
- **A refusal says what's missing, not a fact.** "`be.Whole` on a `Float` says how the fraction goes", and the same
  shape for `be.Decimal`, `be.HugeWhole` and `be.Whole` on another number type, becomes "`be.Whole` on a `Float` needs
  to say how the fraction goes", and so on.
- **The same failure has the same wording everywhere.** Dividing by zero is "this divides by zero / why: dividing by
  zero has no answer" in both `vm/machine.rs` and `vm/numbers.rs`; a square root of a negative number is "a negative
  number has no square root / why: the square root of a negative number isn't a real number" in both.
- **No new terms.** "Blank" for the part in `< >` isn't a word Seal or Unio uses. "`{name}` has one blank, so it takes
  no label" becomes "`{name}` holds one type, written without a label, as `List<Text>`"; "a `Dict`'s blanks are
  labelled …" becomes "a `Dict` labels the two types it holds, `key:` and `value:`, as `Dict<key: Text, value:
  Whole>`", with the `why:` "a type that holds more than one type labels each."

## Where a `why:` or `fix:` is missing and matters

- "`{name}` can't be given a new value" gets `why: a name changes only when it's made with `mut`` (D2), and, where
  the name was made with `=`, a `fix:` showing that line with `mut` added. A `for_each` name, a parameter or a `case`
  value can never change, so its `why:` says so instead. This is the refusal beginners will meet most.
- "this line follows an `if` that can leave" and "… a `match` that can leave" get `why: when any way through an `if`
  leaves, nothing may follow its `end`; what runs afterwards goes in the ways that don't leave` (D68, D86).
- "this line can never run" gets `why: it comes after a `return`, `break` or `fail` in the same block`.
- "a letter comes right after this number" gets a `fix:` with a space, and `why: a unit is written after its number
  with a space, as `2 secs``, since that's what it will almost always be (D112).
- "`{called}` can change `{name}`, which this `for_each` is going through" and "this can pause, while a `for_each` is
  going through the top-level name `{name}`" get `why: what a loop goes through can't change under it; copy it into a
  name of its own first, as `enemies_now = enemies`` (D69).
- "this value's type can't be worked out" and "`{name}`'s type can't be worked out" get `why: a type is written where
  nothing says it, as `name: Type = …`` (D30).
- "the result is too big for a `Whole`" gets `why: a `Whole` goes up to 9223372036854775807; a `HugeWhole` has no
  limit`.
- "the result isn't a real number a `Float` can hold" gets `why: a `Float` stops at its largest value, and never holds
  infinity or "not a number"` (D18).
- "`{}`'s field `{field}` holds {found}, with no way for a value to end" and "every case of `{}` holds more that can't
  end …" get `why: a type can hold itself only through a `List`, a `Dict` or a `Maybe`, so a value can end` (D72).

## Single wordings

- "`be` converts to `Text`, `Whole`, `Float` and `Bool`" and the list of `be`'s narrowings leave out the newer types:
  add `HugeWhole`, `Decimal` and `HugeDecimal`, and `be.Decimal.shown`, `be.Decimal.nearest` and `be.HugeWhole`'s
  roundings (D126).
- "this number's limit is {wanted} written out" becomes "this number's limit has to be {wanted}, written out".
- "`{}` between two `Float`s in a `while` condition" becomes "`{}` compares two `Float`s in a `while` condition".
- "`+` adds two points, two places, which has no meaning" becomes "`+` adds two points, and two places can't be added".
- "`{}` closes a `{}`" becomes "`{}` closes nothing open here; the `{}` still open closes with `{}`".
- "a negative power is one over the number; a `HugeDecimal` can hold it when it has an end" becomes "a negative power
  is one over the number, as `2 ** -3` is `1 / 8`; a `HugeDecimal` holds it when its digits end, as `0.125` does".
- "`{unit}`'s size, `{size}`, isn't a size" becomes "`{unit}`'s size, `{size}`, isn't a whole number above 0".
- "`{name}` is never changed, so it isn't `mut`" (D135) becomes "`{name}` is never changed, so it's made without
  `mut`", with its `fix:` as drafted.
- "this `end` is at the wrong depth" gets `why: an `end` lines up with the line that opened its block`.
- "`{other}` can't be read here" gets `why:` naming what can stand there, where the parser knows.
- The D131 slip: the refusal for a case the call can't fail with says "this `otherwise`" even after `??`; after `??`
  it says "this handler".
