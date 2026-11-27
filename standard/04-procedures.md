# 4. Procedures

## Definitions

```
proc name[parameters] -> Type ! [failures]
    body
end
```

where `name` is the procedure's name, `parameters` its parameters separated by commas, `Type` the type it gives back,
`failures` the failures it can give, and `body` its statements. `proc`, `->`, `!`, the brackets and `end` are written
as they are (D20, D21, D45).

```
proc on_hit[points: Whole]
    score = score + points
end

proc average[a: Float, b: Float] -> Float
    return (a + b) / 2.0
end
```

- **Each parameter is its name, `:` and its type,** as `points: Whole`; a parameter with no type is refused (D21, D81).
- **Parameters that share a type can be grouped,** their names listed and the type written once after them, with a
  space on each side of that `:` since it covers the whole group: `[hp, amount, times : Whole, source: Text]`. A
  group's `:` without the spaces, and a single name's with them, are refused, with the respaced header as the fix
  (D88).
- **`-> Type` is written whenever the procedure gives back a value,** and left off when it gives back none (D81).
- **`! [failures]` is written whenever the procedure can fail,** listing exactly what its body can fail with (D44, D45,
  D84).
- **The line has no `:`** (D9).

Seal has no contracts, `requires` or `ensures`, for now (D39).

A procedure that takes nothing has no brackets, and every call writes `()`: `proc on_tick`, called as `on_tick()`
(D60, Unio D4).

## Generic procedures

A procedure can work on any type, written as a type parameter in `< >` right after its name: `?` and a capitalised
name, as `?Item`, marking a type that's filled in by each call (D161, from Cyborg D234 and D239):

```
proc first<?Item>[items: List<?Item>] -> Maybe<?Item>
    for_each item in items
        return Maybe.some[value: item]
    end
    return Maybe.none
end

top = first([3, 12, 7])           // ?Item is Whole
name = first(["Ann", "Bo"])       // ?Item is Text
```

- **`?` marks the type parameter wherever it's used,** so it's never mistaken for a real type, as `~` marks an inexact
  `Float` and `*` a handler that can jump.
- **A call never writes the type:** it's worked out from the arguments, or from where the result goes, and refused
  where nothing says it.

- **A type parameter with no requirement takes any type,** and inside, its values can only be stored, passed on,
  returned, compared with `eq` and `not_eq`, and shown in braces (D166).
- **A requirement after `:` says what the type must be able to do,** and lets the body do it: `<?Item: Number>` allows
  maths and ordering on `?Item` (D166). Each call is checked: a type that doesn't meet it is refused at the call.

```
proc largest<?Item: Number>[a: ?Item, b: ?Item] -> ?Item
    if a > b
        return a
    else
        return b
    end
end
```

- **`Number`,** the first built-in requirement: `Whole`, `Float`, `HugeWhole`, `Decimal` and `HugeDecimal`, with `+`,
  `-`, `*`, `/`, `<`, `>`, `at_least`, `at_most`, `abs`, `min` and `max`. One call's `?Item` is one type, so `Whole`
  and `Float` still don't mix (D16). Values with units aren't among them (D166).
- **A script can write its own requirements,** listing methods a type must have, so a generic can call them (D166).

- **A script's requirement is declared with `requirement`,** listing the method headers a type must have, with `?Self`
  for the type that meets it (D168):

```
requirement Scored
    proc score[item: ?Self] -> Whole
end
```

  A type's method matches a listed one when its name, the types it takes (with `?Self` as the type), their labels,
  its result and its failures are the same; its parameters' names and its body are its own.
- **A record or `one_of` meets a script's requirement only by saying so,** with `meets [ … ]` as its first line, as
  `meets [Scored]`; Seal checks there that it has every method the requirement lists (D167). Nothing meets one by
  accident.

**Not decided:** generics over units; generic records and `one_of`s; how generics are compiled.

**Not decided:** parameter words for reading, changing or taking over (Unio D130).
A procedure's failures: chapter 7.

## Families

A script gives its procedures narrowings in a family, a word and its kinds declared together (D43, Unio D41):

```
fam.proc spawn
    spawn[kind: Text]
        ...
    end
    spawn.boss[kind: Text]
        ...
    end
end
```

where each `...` stands for that procedure's body.

- **Members are written without `proc`,** since the family's first line says what they are, each by its full name,
  since it's called on its own anywhere (D97).
- **Each member is the word itself, or reads as "a Y kind of X"** for its name `X.Y`; anything else is refused (D43).
- **A narrowed procedure is declared only inside its word's family** (D43, Unio D41).
- **Handlers aren't a family:** they live in a `group.handler`, a named set reached with `::` (chapter 7, D110).

## Types

A procedure's header always says its types: each parameter's, and its result's when it gives one back (D81). The body
is checked against the header, so a mistake is reported in the procedure that has it, and the host's calls are checked
against the header at load time (chapter 9). Names made inside a body still have their types worked out from their
values (D2, D30).

## `return`

`return value` gives back a value, and a bare `return` leaves early, as in Unio (Unio's standard, chapter 4, D57).

- **If any `return` gives a value, every way through must end in one;** a `while` counts as able to end, so a loop
  with `while true` still needs a `return` after it (D68, Unio D57, D149).
- **Nothing may follow `return`, `break` or `fail` in the same block,** and a bare `return` as a procedure's last line
  is refused (D68, Unio D57).
- **When any way through an `if` or a `match` leaves, nothing may follow its `end`;** the rest goes in
  the ways that don't leave (D68, D86, Unio D9).
- **An `otherwise` block is different:** lines may follow its `end` even when a case leaves, as they follow a call
  passed up with `?`, since its cases only repair a failure and success carries on past it (D143).

## Calls

A call writes the procedure's name and `( )`, with its arguments inside (Unio D3, D4, D165).

- **Labels:** a procedure's writer can give its arguments labels, and every call writes them, as in
  `heal(50, by: 10)` (D22, Unio D24). By convention the first argument has none.
- **Slides:** a chain of three or more calls, each the first argument of the next, is written as a slide with `|>`,
  and one or two as calls (D22, Unio D5, D128):

```
points |> double |> be.Text |> show_msg;
show_msg(be.Text(n))
```

- **A statement holding a slide ends with `;`,** on one line or split, and a slide inside brackets counts too (D180).
- **A split slide carries on until its `;`,** each carried line starting with `|>`; a line ending in `|>` is refused
  (D8). A missing `;` is refused at the slide's last step, with the `;` in the fix (D180). A comment on a line of its
  own may stand between steps; a blank line may not (D181):

```
heal = potion
    |> doubled
    // potions can't heal past full
    |> clamp(low: 0, high: max_hp)
    |> as_label;
total = (points |> doubled |> doubled |> doubled) + 1;
```

Unio's standard, chapter 4, has the rules for slides in full.

A procedure declares its labels on a `labels` line, the first line of its body, with its list in `[ ]` (D41, D42,
Unio D25). A `defaults` line may follow it, giving a value for a labelled argument a call leaves out, written as the call
would write it (D42):

```
proc heal[hp: Whole, amount: Whole, times: Whole] -> Whole
    labels [amount as by, times as times]
    defaults [times: 1]
    return hp + amount * times
end

// means heal(hp, by: 10, times: 1)
hp = heal(hp, by: 10)
```

- **Only labelled parameters can have a default** (D42).
- **A default is a value written out plainly,** of any type, as `5`, `"red"`, `Color.red`, `[]` or a record written out,
  or a top-level name with no `mut` made from one; a calculation or a call is refused (D104).
- **A call writes its arguments in the declared order,** skipping any with a default; another order is refused, with
  the reordered call as the fix. A procedure whose maker writes `labels.any_order [ … ]` lets its labelled arguments go
  in any order (D87, from Cyborg's standard, 5.4).
- **A default is a plain value,** never a calculation or a call (D42).
- **The host's calls can leave defaulted arguments out too** (D42).

- **A call that gives a value can't stand alone,** and nor can a line that only works out a value, as `score + 1`; a
  value is dropped on purpose by naming it `unused` (D70, D3, Unio D120, D161).
