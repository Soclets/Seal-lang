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
- **Text counts graphemes by default,** what a person sees as one character: `.len` counts them, and `[pos:]`,
  `[offset:]` and `for each` each give one grapheme as a `Text`. `.len.ucp` counts Unicode code points and `.len.bytes`
  UTF-8 bytes. Each Seal version pins one Unicode version (D65, D66).
- **`.split(by: t)` gives a `List<Text>`,** keeping empty pieces, and `.split.no_empty(by: t)` drops them (D65).
- **`==` and `!=` compare two texts exactly,** character by character, with nothing normalised: `"Yes" != "yes"`;
  `<` and `>` on text aren't decided (D64).
- **Text is written in double quotes, and joined only by filling it in with braces,** as in `"{first} {last}"` (D16,
  D75, D77, Unio D21, D201). Seal has no operator for joining text: `&` is refused, and so is `+` on text, each with
  the braces form as the fix.
- **Maths:** `+`, `-`, `*`, `/`, `div.floor`, `div.trunc`, `rem` and `sqrt` (D22, Unio D27, D70, D71, D185). Floats
  are checked after every operation (D18).
- **Lists:** `[ ]` literals, `pos:` and `offset:`, `.add`, `.remove`, `.len`, `len.max` with its failure `full`, and
  `[all: value]` (D22, Unio D75 to D80, D167 to D169).
- **A list's `len.max` is part of its type:** moving a list into a limit at least as big, or none, is allowed, and into
  a smaller one it can fail with `full`, as `big_bag ?? []`; a literal longer than its `len.max` is refused (D73).

```
mut bag: List<Text>[len.max: 10] = []
board: Array<Whole>[len: 64] = [all: 0]
```

A name's type can be written after it, as `name: Type = value`. It's optional, and required only when the value can't
say the type, as with an empty list: `mut bag = []` is refused (D30).

A written type can leave out what the value shows and write only the rest, such as a length the value can't give:
`board: Array[len: 64] = [all: 0]` takes `Whole` from the `0`. What the value doesn't show must still be written, as
the item type of an empty list: `bag: List[len.max: 10] = []` is refused (D82).

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
end record Enemy

mut goblin = Enemy[name: "Goblin", hp: 30, x: 0.0, y: 0.0]
goblin.hp = goblin.hp - 5
```

- **`record Name` declares a record,** its fields one per line, each with its type, closed by `end` (D20, D27).
- **It's built with `[ ]`, naming every field;** a field left out is refused (D27).
- **A field is read with `.field`,** and changed only through a `mut` name. A record is a value: `b = a` gives `b` its
  own copy (D15, D27).
- **A change can reach through any depth of fields and positions** of a `mut` name, as `bodies[pos: 1].vx = 2.0`, with
  a method allowed at the end, as `goblin.items.add("Gem")`; a `Dict` key partway along isn't decided (D62).
- **At the doors a record is a Rust struct** (D27).

- **A record can hold a record,** and a type can hold itself where a value can end, through a `one_of` case, a
  `List`, a `Dict` or a `Maybe`; with no way to end, as a field `next: Node` in `record Node`, it's refused (D72).

**Not decided:** defaults for fields; methods on records.

## Dictionaries

```
mut stock: Dict<key: Text, value: Whole> = ["sword": 1, "potion": 3]
stock[key: "potion"] = 4
potions = stock[key: "potion"] ?? 0
```

- **`Dict<key: Key, value: Value>` holds values looked up by key,** each key once. `Dict` is short for dictionary
  (D32).
- **Its literal writes the keys as values,** and `[:]` is an empty one; the same key twice in one literal is refused
  (D32, D73).
- **It's reached with `key:`;** setting a key through a `mut` name replaces its value or adds it (D32).
- **A missing key is the failure `missing`,** handled with `??` or `otherwise` (D32, Unio D23).
- **It keeps the order keys were added in,** and is copied as a whole, a Rust `HashMap` at the doors (D15, D32).

`for each` goes through a `Dict`'s values in the order their keys were added, and `at key name` gives each key (D33):

```
for each count at key item in stock
    show_msg("{item}: {count}")
end for
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
end one_of Event

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
    case healed[by]
        show_msg("{by} healed you")
    case idle
end match
```

- **Every case must be covered;** there's no catch-all (D37).
- **A case's values come out by name,** read-only, for that case's body, and a case takes out only the values it
  reads (D37, D61).
- **A case runs until the next `case`,** and one `end` closes the `match` (D37).

## `Maybe`

Seal has no `nil`, `null` or `None`: a name of a type always holds a real value of that type (D38). "Might not be
there" is `Maybe<T>`, a built-in `one_of` with the cases `none` and `some[value: T]`, read with `match` or given a
fallback with `??`, and Rust's `Option<T>` at the doors (D38):

```
target = nearest_enemy()
match target
    case some[value]
        attack(value)
    case none
end match
id = nearest_enemy() ?? 0
```

`?? crash` on a `Maybe` stops the script with a failed check when it's `none`, as for a failure (D54, D79). `?? ignore`
on a `Maybe` is refused, since the name would be left with nothing in it, with `??` and a value as the fix (D79).

## Filling in text

Every text fills in what's in its braces, wherever it's written (D75, Unio D200):

```
mut msg = "Hello, {player}"
show_msg("Correct! You got it in {tries} tries")
```

- **Braces hold a name, which may be followed by its fields and facts,** as `{goblin.name}` and `{names.len}`, never
  a calculation or a call; a name, field or fact that doesn't exist is refused at load time (D12, D28, Unio D29).
- **A brace to show as it is is written `\{`,** as in `"Type \{help}"`, which shows `Type {help}` (D63, D75, Unio
  D30).

**Not decided:** `places:` in braces (Unio D175).

## Converting

`be.Whole`, `be.Float` and `be.Text`, with `be.Whole.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` for a
`Float`, as in Unio (D22, Unio D22, D153, D178). Unio's standard, chapter 7, has them in full.
