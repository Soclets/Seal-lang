# 3. Names and values

## Making and changing names

```
name = value
mut name = value
unused name = value
```

where `name` is the name and `value` its value. `mut`, `unused` and `=` are written as they are.

- **`name = value` makes `name` when no name `name` exists yet,** and its type is worked out from the value (D2).
- **The type can be written, as `name: Type = value`,** and must be when the value can't say it, as `mut bag:
  List<Text> = []` (D30).
- **When `name` already exists, `name = value` changes it,** which is allowed only if it was made with `mut` (D2).
- **A name that's made and never read is refused,** saying plainly that it's never used, with no guess at what was
  meant: a typo such as `tires = tries + 1` is caught this way (D2, Unio D54).
- **`unused` marks a name that's never read on purpose,** so it isn't refused, and reading it later is refused.
  Throwing a value away is the same: give it a name and mark it `unused` (D3).

```
mut tries = 0
tries = tries + 1          // changes tries
tires = tries + 1          // refused: `tires` is never used
unused saved = save(game) ?? 0
```

**Not decided:** where a name made inside a block ends; making a name again that's already in use; shorthand like `+=`.

## Spelling

A name is lowercase English letters `a` to `z`, digits, and `_` between words, starting with a letter: `player2`,
`hp_max`, `get_service`. No capitals, and `_` never at the start or end, nor two in a row (D24, Unio D99). Types are
capitalised, as `Whole` and `List`, so a capital always means a type (D24).

- **There's no `$`** (D7).
- **A name can't be a keyword:** `mut if = 3` is refused (D7).
- **The host's doors and their narrowings follow the same spelling,** checked when the host opens them (D24).

**Not decided:** letters beyond English; Seal's full list of keywords.

## Values

Every name owns its own value: after `b = a`, changing `b` never touches `a`, lists included. No reference can be
kept, so nothing dangles, nothing leaks, and no cycles form (D15, Unio D130).

- **There's no garbage collector.** Seal counts the holders of each big value, copies one only when it's written while
  shared (copy-on-write), and frees it the moment its last holder ends (D15).
- **Something that's really one thing, shared with the host, is held by an ID** and changed through the host's doors,
  as in `damage(enemy_id, 10)` (D15, chapter 9).
