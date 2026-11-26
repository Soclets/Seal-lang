# 3. Variables and values

## Making and changing variables

A variable is a named value. It's **mutable** when it's made with `mut`, so it can change, and **immutable**
otherwise, so it never does (D162).

```
name = value
mut name = value
unused name = value
```

where `name` is the variable's name and `value` its value. `mut`, `unused` and `=` are written as they are.

- **`name = value` makes the variable `name` when none exists yet,** and its type is worked out from the value (D2).
- **The type can be written, as `name: Type = value`,** and must be when the value can't say it, as `mut bag:
  List<Text> = []` (D30). A written type may leave out the parts the value shows, as `board: Array[len: 64] = [all:
  0]` (D82).
- **When the variable already exists, `name = value` changes it,** which is allowed only if it's mutable, made with
  `mut` (D2).
- **A mutable variable must be changed somewhere,** by `=`, a method that changes it, or a change deep inside it, or
  it's refused, with removing `mut` as the fix (D135).
- **A variable that's made and never read is refused,** saying plainly that it's never used, with no guess at what was
  meant: a typo such as `tires = tries + 1` is caught this way (D2, Unio D54).
- **`unused` marks a variable that's never read on purpose,** so it isn't refused, and reading it later is refused.
  Throwing a value away is the same: give it a name and mark it `unused` (D3).
- **The same holds for parameters and `for_each` variables,** as `proc on_hit[unused amount: Whole]` and `for_each
  unused round in …`, and to a `for_each`'s `at` variable, as `at key unused item` (D86); a `case` instead takes out
  only the values it reads (D61).

```
mut tries = 0
tries = tries + 1          // changes tries
tires = tries + 1          // refused: `tires` is never used
unused saved = save(game) ?? 0
```

- **A name made inside a block ends where the block ends,** and a loop's body is a fresh block each pass. Names from
  outside are visible inside, and while a name exists it can't be made again: `name = value` changes it (D56, Unio
  D93, D94).

## `_`: what's being changed

On the right of an `=` that changes a name, `_` is that name's current value, or the current value of whatever part of
it is being changed (D147):

```
mut score = 0
score = _ + 10
hp = clamp(_ + heal, low: 0, high: max_hp)
enemies[pos: i].hp = _ - 1
```

- **It works with anything,** maths, calls or facts, so Seal has no `+=` and its kin.
- **The left side is worked out once,** positions and keys included, and `_` reads it.
- **It can appear more than once,** as `x = _ * _`.
- **It's refused where nothing is being changed:** when the `=` makes a new name, and anywhere outside the right of a
  changing `=`. The name must be `mut`, as for any change (D2).
- `_` alone is never a name, since a name never starts with `_` (D24).

## Spelling

A name is lowercase English letters `a` to `z`, digits, and `_` between words, starting with a letter: `player2`,
`hp_max`, `get_service`. No capitals, and `_` never at the start or end, nor two in a row (D24, Unio D99). Types are
capitalised, as `Whole` and `List`, so a capital always means a type (D24). A type's name has no `_`, and letters
beyond plain English, as in `café`, are refused in any name for now (D93).

- **There's no `$`** (D7).
- **A name can't be a keyword:** `mut if = 3` is refused (D7).
- **A name can't be `all`,** since `[all: 5]` would read both as a list of 5s and as a `Dict` keyed by the name (D83,
  Unio D207).
- **Colours and font are recommended, never forced:** Seal's docs show code in Unio's colours and font (Unio D194,
  D197, D198), and Seal's editor extension offers Unio's colours as a theme, suggested once when it's installed. The
  language server only says what kind each piece of code is, for any theme, and a host shows Seal's messages however
  it likes (D7, D78).
- **A name can't be made where its spelling already means something,** a top-level name, another name in use, a
  procedure, a door or a built-in, whether as a name, a parameter, a `for_each` name or a `case` value: there's no
  shadowing (D71). A procedure can't take a built-in's or a door's name either (D93).
- **The host's doors and their narrowings follow the same spelling,** checked when the host opens them (D24).

**Not decided:** letters beyond English; Seal's full list of keywords.

## Values

Every name owns its own value: after `b = a`, changing `b` never touches `a`, lists included. No reference can be
kept, so nothing dangles, nothing leaks, and no cycles form (D15, Unio D130).

- **There's no garbage collector.** Seal counts the holders of each big value, copies one only when it's written while
  shared (copy-on-write), and frees it the moment its last holder ends (D15).
- **Something that's really one thing, shared with the host, is held by an ID** and changed through the host's doors,
  as in `damage(enemy_id, 10)` (D15, chapter 9).

## The top level

A script's top level runs once, top to bottom, when it's loaded (D4). Three things are refused there, at load time
(D67):
- **reaching a top-level name before the line that makes it,** directly or through a procedure it calls;
- **pausing,** through a door or procedure that can pause;
- **`return`, `fail` and `?`,** since nothing called the top level; a failure there is handled with `??` or `otherwise`.

`record`, `one_of`, `proc` and `fam` are declared at the top level only; inside a block they're refused (D93).
