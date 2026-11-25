# 6. Types

Seal's types are Unio's, with Unio's rules for them (D16). Unio's standard, chapter 7, has them in full; read its
examples in Seal's syntax (README).

| Type | What it holds | At the doors, in Rust |
|---|---|---|
| `Whole` | a 64-bit signed whole number, stopping with a failed check on overflow (Unio D15, D46) | `i64` |
| `HugeWhole` | a whole number of any size, limited only by the script's memory cap (D14, D115) | a big integer, the building session's choice |
| `Decimal` | an IEEE 754 decimal128 number: 34 decimal digits, exact for numbers written with that many, rounding past them (D116) | the building session's choice |
| `HugeDecimal` | a decimal number of any size, exact (D116) | the building session's choice |
| `Float` | an IEEE 754 binary64 number, with `~` on inexact literals and Unio's rules for showing, rounding and comparing (Unio D155, D157, D175 to D184) | `f64` |
| `Bool` | `true` or `false` | `bool` |
| `Text` | text | `String` |
| `List<T>` | a list of `T`s, with an optional most it may hold, `len.max` (Unio D167) | `Vec` |
| `Array<T>[len: n]` | exactly `n` `T`s (Unio D167) | `Vec` |

- **A number never starts with a leading zero,** as `007`; `0` itself and `0.5` are fine (D93).
- **A number with a point is a `Float`, and one without is a `Whole`** (Unio D180), so a worked-out type is never in
  doubt for a literal: `speed = 2.0` makes a `Float`.
- **`Whole` and `Float` don't mix quietly:** `be.Float(lives) * speed` (Unio D162).
- **A `Whole` becomes a `HugeWhole` by itself,** since it always fits exactly, as in `cookies + clicks`; a `HugeWhole`
  becomes a `Whole` only through `be.Whole`, failing with `too_big` when it doesn't fit, and a `HugeWhole` and a
  `Float` don't mix quietly (D115). A number written too big for a `Whole` is a `HugeWhole` where one is wanted, and
  refused elsewhere (D115).
- **A `Decimal` counts in tens,** so `0.1` is exactly 0.1. It holds 34 digits; past them it rounds, nearest with ties
  to even, and shows `~` as a `Float` does, and `/` rounds the same way. A `HugeDecimal` never rounds on `+`, `-` or
  `*`, and its `/` gives the exact answer or stops, as `/` on a `Whole` does (D116, Unio D70, D155).
- **Where it always fits, a number becomes the bigger type by itself:** a `Whole` into a `Decimal` or `HugeDecimal`,
  a `HugeWhole` into a `HugeDecimal`, and a `Decimal` into a `HugeDecimal`. Every other way is written with `be.` and
  can fail, and a `Float` mixes quietly with none of them (D116).
- **A number written with a point is a `Decimal` or `HugeDecimal` where one is wanted,** exactly as written, the other
  side of an operator included, so `gold + 0.5` and `gold > 0.5` just work with `gold` a `Decimal`; likewise a whole
  number too big for a `Whole` beside a `HugeWhole` (D116, D119).
- **Text counts graphemes by default,** what a person sees as one character: `.len` counts them, and `[pos:]`,
  `[offset:]` and `for each` each give one grapheme as a `Text`. `.len.ucp` counts Unicode code points and `.len.bytes`
  UTF-8 bytes. Each Seal version pins one Unicode version (D65, D66).
- **`.split(by: t)` gives a `List<Text>`,** keeping empty pieces, and `.split.no_empty(by: t)` drops them (D65).
- **Splitting by empty text is refused while loading** wherever the empty text can be seen, as `split(by: "")`; one
  that only turns out empty while running is a failed check, as dividing by zero is (D89).
- **`==` and `!=` compare two texts exactly,** character by character, with nothing normalised: `"Yes" != "yes"`;
  `<` and `>` on text are refused for now, since text has no one right order (D64, D96).
- **`==` and `!=` compare bigger values by their contents** (D95): records field by field; lists and arrays by length
  and their items in order; `Dict`s by their keys, each with an equal value, whatever order they were added in; and
  `one_of`s by their case and its values.
- **Text is written in double quotes, and joined only by filling it in with braces,** as in `"{first} {last}"` (D16,
  D75, D77, Unio D21, D201). Seal has no operator for joining text: `&` is refused, and so is `+` on text, each with
  the braces form as the fix.
- **Maths:** `+`, `-`, `*`, `/`, `div.floor`, `div.trunc`, `rem` and `sqrt` (D22, Unio D27, D70, D71, D185). Floats
  are checked after every operation (D18).
- **`**` raises to a power,** binding tighter than `*`, so `2 * x ** 2` is `2 * (x ** 2)` (D108). `Whole ** Whole`
  gives a `Whole`, stopping with the overflow check when it's too big, and a negative power of a `Whole` is refused;
  `Float ** Float` gives a `Float`, correctly rounded (D50). The two don't mix.
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

A type's setting, as `len.max:` or `len:`, is a whole number written there, or a name made at the top level from a
whole number written there and never changed, with no `mut`, as `max_items = 10` and then
`bag: List<Text>[len.max: max_items] = []`. Any other name, a calculation or a call is refused, since its value isn't
known while loading (D103).

## Units

Units are optional: a `Whole` or a `Float` can carry one, as Cyborg's do, and nothing is ever required to (D111):

```
wait(2 secs)
wait(500 ms)
total = 2 secs + 500 ms                  // 2500 ms
```

- **A number takes its unit bare, right after it,** as `2 secs`. Units belong to groups, and the group is written,
  as `time::secs`, only to settle a clash between two groups with a unit of the same name. A unit only ever stands
  after a number or in a type, so it never takes a word away from names (D111, D112).
- **Units of one group combine and convert by themselves when the result is exact:** `2 secs + 500 ms` is `2500 ms`,
  in the finer unit, so a `Whole` stays exact, and a value goes where another unit of its group is wanted when it
  converts exactly. A conversion that can't be exact is refused, with the fix saying how to round. Units of different
  groups, as seconds and pixels, never combine by `+` or `-` (D120).
- **`per` writes a rate right where it's used,** as `120.0 px per secs`, with no declaration: multiplying and dividing
  work the units out, so `speed * dt` with `dt` in `secs` gives `px`, and a result is refused where its units don't
  fit (D113, D120).
- **Where a unit is expected, a plain number is refused,** with the unit added as the fix, so a door taking a time
  can't be given a bare `2` (D111).
- **Units have short names,** as `secs` and `ms`, since their group says what they measure (D111).
- **Everything else follows Cyborg's units,** decided there: derived units, angles, points and spans, prefixes and
  showing a unit (D111).

Seal's own unit groups are (D113):

| Group | Units |
|---|---|
| `time` | `ms`, `secs`, `mins`, `frames` |
| `angle` | `degs`, `rads`, `turns` |
| `distance` | `px`, `tiles` |

`frames` and `tiles` have no fixed size, so they convert only through a rate, as `16.0 px per tiles`, or a frame's
measured time in `secs per frames`. Other units, as money or a game's own, a script or host declares as a group of
its own (D113).

## Ranged numbers

A `Whole` or a `Float` can be given a lowest and a highest value as type settings (D107):

```
mut gold: Whole[min: 0] = 0
mut hp: Whole[min.clamp: 0, max.clamp: 100] = 100
mut speed: Float[min: 0.0, max: 12.5] = 0.0
```

- **`min:` and `max:` are hard limits:** storing a value outside them stops the script with a failed check, and one
  Seal can see while loading, as `gold: Whole[min: 0] = -5`, is refused (D18, D54, D107).
- **`min.clamp:` and `max.clamp:` pull a value back to that edge** each time one is stored, so `hp = hp - 150` leaves
  `hp` at 0. Each edge chooses for itself (D107).
- **Either can be left off,** and both are type settings, so each is a number written there or a top-level name made
  from one (D103).

For edges only known while the script runs, `clamp` does the same to one value (chapter 8).

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
- **A change can reach through any depth of fields and positions** of a `mut` name, as `bodies[pos: 1].vx = 2.0`, with
  a method allowed at the end, as `goblin.items.add("Gem")`; a `Dict` key partway along is refused, since a
  missing key would have nowhere to be dealt with: the value is taken out, changed and put back (D62, D90).
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

- **`one_of` declares a type whose value is exactly one of its cases,** listed one per line, closed with `end`; one with no
  cases is refused, since no value of it could exist (D35, D36, D93).
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
end
```

- **Every case must be covered;** there's no catch-all (D37).
- **A case's values come out by name,** read-only, for that case's body, and a case takes out only the values it
  reads, in their declared order (D37, D61, D87).
- **A case runs until the next `case`,** and one `end` closes the `match` (D37, D102).

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
end
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
- **Every value shows as it's written in Seal,** a list, record, `Dict` or `one_of` included, with text inside in its
  quotes and a `Float` with its `~` when it isn't exact: `"{bag}"` with `["Gem", "Key"]` shows `["Gem", "Key"]`, and
  `be.Text` gives the same (D100, Unio D155).
- **`{newln}` and `{tab}` give a line break and a tab,** built-in names filled in like any other, as
  `"Game over{newln}Score: {score}"`; no name can be `newln` or `tab` (D71, D118).
- **Empty braces, as `"{}"`, are refused,** with `"\{}"` as the fix (D93).
- **A brace to show as it is is written `\{`,** as in `"Type \{help}"`, which shows `Type {help}` (D63, D75, Unio
  D30).

A `Float` in braces can take `places:` after a comma, showing exactly that many digits after the point, rounded to the
nearest with a tie going away from zero, and with no `~`, as Unio's chapter 7 has it (D94, Unio D175, D176, D178):

```
show_msg("Speed: {speed, places: 2}")       // Speed: 0.30
```

## Converting

`be.Whole`, `be.Float` and `be.Text`, with `be.Whole.floor`, `.trunc`, `.ceil`, `.nearest` and `.nearest.even` for a
`Float`, as in Unio (D22, Unio D22, D153, D178). Unio's standard, chapter 7, has them in full.

- **`be.Bool` reads `"true"` or `"false"`,** as a `Bool` shows, and fails with `not_a_bool` on any other text (D101).
- **`be.Text` takes any value,** shown as it's written (D100).
- **Any other conversion is refused,** as text into a list, or a `Whole` into a `Bool` (D101).
