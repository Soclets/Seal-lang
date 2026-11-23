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
- **Text is written in double quotes, and `&` joins text** (D16, Unio D21, D28). A `Whole` or a `Bool` joined with `&`
  becomes text by itself, as in `show_msg("Score: " & score)`; a `Float`, a list or a record is refused, with
  `be.Text(…)` as the fix for a `Float` (D29).
- **Maths:** `+`, `-`, `*`, `/`, `div.floor`, `div.trunc`, `rem` and `sqrt` (D22, Unio D27, D70, D71, D185). Floats
  are checked after every operation (D18).
- **Lists:** `[ ]` literals, `pos:` and `offset:`, `.add`, `.remove`, `.len`, `len.max` with its failure `full`, and
  `[all: value]` (D22, Unio D75 to D80, D167 to D169).

```
mut bag: List<Text>[len.max: 10] = []
board: Array<Whole>[len: 64] = [all: 0]
```

A name's type can be written after it, as `name: Type = value`. It's optional, and required only when the value can't
say the type, as with an empty list: `mut bag = []` is refused (D30).

A type with more than one blank to fill in labels each one, as `Dict<key: Text, value: Whole>`; one with a single
blank, as `List<Text>`, doesn't (D31).

**Not decided:** other types.

## Records

```
record Enemy
    name: Text
    hp: Whole
    x: Float
    y: Float
end

mut goblin = Enemy[name: "Goblin", hp: 30, x: 0.0, y: 0.0]
goblin.hp = goblin.hp - 5
```

- **`record Name` declares a record,** its fields one per line, each with its type, closed by `end` (D20, D27).
- **It's built with `[ ]`, naming every field;** a field left out is refused (D27).
- **A field is read with `.field`,** and changed only through a `mut` name. A record is a value: `b = a` gives `b` its
  own copy (D15, D27).
- **At the doors a record is a Rust struct** (D27).

**Not decided:** defaults for fields; methods on records; records inside records.

## Dictionaries

```
mut stock: Dict<key: Text, value: Whole> = ["sword": 1, "potion": 3]
stock[key: "potion"] = 4
potions = stock[key: "potion"] ?? 0
```

- **`Dict<key: Key, value: Value>` holds values looked up by key,** each key once. `Dict` is short for dictionary
  (D32).
- **Its literal writes the keys as values,** and `[:]` is an empty one (D32).
- **It's reached with `key:`;** setting a key through a `mut` name replaces its value or adds it (D32).
- **A missing key is the failure `missing`,** handled with `??` or `otherwise` (D32, Unio D23).
- **It keeps the order keys were added in,** and is copied as a whole, a Rust `HashMap` at the doors (D15, D32).

`for each` goes through a `Dict`'s values in the order their keys were added, and `at key name` gives each key (D33):

```
for each count at key item in stock
    show_msg(fmt("{item}: {count}"))
end
```

- **`.remove(key: k)`** takes a key out, failing with `missing` if it isn't there; **`.has(key: k)`** gives a `Bool`;
  **`.len`** is the number of keys (D34).
- **Keys can be `Whole`, `Text` or `Bool`;** a `Float` key is refused (D34).

**Not decided:** lists and records as keys; other methods.

## Options

```
one_of Event
    idle
    hurt[amount: Whole]
    healed[amount: Whole, by: Text]
end

mut last = Event.hurt[amount: 5]
```

- **`one_of` declares a type whose value is exactly one of its cases,** listed one per line, closed with `end`
  (D35, D36).
- **A case may carry values, named in `[ ]`,** built as a record is (D27, D35).
- **A case is written as a narrowing of its type,** as `Event.hurt[amount: 5]` and `State.idle`; one the type doesn't
  have is refused at load time (D35, Unio D41).

`match` reads which case a value is (D37):

```
match last
    case hurt[amount]
        hp = hp - amount
    case healed[amount, by]
        show_msg(fmt("{by} healed you"))
    case idle
end
```

- **Every case must be covered;** there's no catch-all (D37).
- **A case's values come out by name,** read-only, for that case's body (D37).
- **A case runs until the next `case`,** and one `end` closes the `match` (D37).

## `fmt`

`fmt("…")` fills in the names in its braces and gives back a `Text` (D12):

```
mut msg = fmt("Hello, {player}")
show_msg(fmt("Correct! You got it in {tries} tries"))
```

- **Braces hold a name, which may be followed by its fields and facts,** as `{goblin.name}` and `{names.len}`, never
  a calculation or a call; a name, field or fact that doesn't exist is refused at load time (D12, D28, Unio D29).
- **Text is plain unless it goes through `fmt`:** `show_msg("{tries}")` shows `{tries}` (D12).

**Not decided:** a literal brace inside `fmt` text; `places:` in braces (Unio D175).

## Converting

`be.Whole`, `be.Float` and `be.Text`, with `be.Whole.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` for a
`Float`, as in Unio (D22, Unio D22, D153, D178). Unio's standard, chapter 7, has them in full.
