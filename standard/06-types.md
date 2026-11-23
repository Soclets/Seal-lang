# 6. Types

Seal's types are Unio's, with Unio's rules for them (D16). Unio's standard, chapter 7, has them in full; read its
examples in Seal's syntax (README).

| Type | What it holds | At the doors, in Rust |
|---|---|---|
| `Whole` | a 64-bit signed whole number, stopping with a failed check on overflow (Unio D15, D46) | `i64` |
| `Float` | an IEEE 754 binary64 number, with `~` on inexact literals and Unio's rules for showing, rounding and comparing (Unio D155, D157, D175 to D184) | `f64` |
| `Bool` | `true` or `false` | `bool` |
| `Text` | text | `String` |
| `List<T>` | a list of `T`s, with an optional most it may hold, `len.max` (Unio D167) | `Vec` |
| `Array<T>[len: n]` | exactly `n` `T`s (Unio D167) | `Vec` |

- **A number with a point is a `Float`, and one without is a `Whole`** (Unio D180), so a worked-out type is never in
  doubt for a literal: `speed = 2.0` makes a `Float`.
- **`Whole` and `Float` don't mix quietly:** `be.Float(lives) * speed` (Unio D162).
- **Maths:** `+`, `-`, `*`, `/`, `div.floor`, `div.trunc`, `rem` and `sqrt` (D22, Unio D27, D70, D71, D185). Floats
  are checked after every operation (D18).
- **Lists:** `[ ]` literals, `pos:` and `offset:`, `.add`, `.remove`, `.len`, `len.max` with its failure `full`, and
  `[all: value]` (D22, Unio D75 to D80, D167 to D169).

```
mut bag: List<Text>[len.max: 10] = []
board: Array<Whole>[len: 64] = [all: 0]
```

**Not decided:** how a type is written on a name in Seal, as in the example's `bag: List<Text>[len.max: 10]`, when it
can't be worked out from the value; records, maps and other types.

## `fmt`

`fmt("…")` fills in the names in its braces and gives back a `Text` (D12):

```
mut msg = fmt("Hello, {player}")
show_msg(fmt("Correct! You got it in {tries} tries"))
```

- **Braces hold only names,** never a calculation or a call, and a name that doesn't exist is refused at load time
  (D12, Unio D29).
- **Text is plain unless it goes through `fmt`:** `show_msg("{tries}")` shows `{tries}` (D12).

**Not decided:** a literal brace inside `fmt` text; `places:` in braces (Unio D175).

## Converting

`be.Whole`, `be.Float` and `be.Text`, with `be.Whole.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` for a
`Float`, as in Unio (D22, Unio D22, D153, D178). Unio's standard, chapter 7, has them in full.
