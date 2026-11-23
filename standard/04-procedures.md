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
proc on_hit[points]
    score = score + points
end

proc average[a: Float, b: Float] -> Float
    return (a + b) / 2.0
end
```

- **Each parameter is its name, with its type optional,** as `points` or `a: Float` (D5, D21).
- **`-> Type` and `! [failures]` are optional,** worked out from the body when left off (D6, D44, D45).
- **The line has no `:`** (D9).

Seal has no contracts, `requires` or `ensures`, for now (D39).

A procedure that takes nothing has no brackets, and every call writes `()`: `proc on_tick`, called as `on_tick()`
(D60, Unio D4).

**Not decided:** parameter words for reading, changing or taking over (Unio D130).
A procedure's failures: chapter 7.

## Families

A script gives its procedures narrowings in a family, a word and its kinds declared together (D43, Unio D41):

```
fam.proc spawn
    proc spawn[kind]
        ...
    end
    proc spawn.boss[kind]
        ...
    end
end
```

where each `...` stands for that procedure's body.

- **Each member is the word itself, or reads as "a Y kind of X"** for its name `X.Y`; anything else is refused (D43).
- **`fam.handler`,** for a failure's handlers, comes once Seal has handlers (D43).

## Worked-out types

Types left out are worked out when the script loads, from the procedure's own body, never from its callers. The result
is fully static, exactly as if the types were written (D5, D6).

- **A parameter's type comes from how the body uses it:** in `score = score + points`, `score` is a `Whole`, so
  `points` is a `Whole` (D5).
- **When the body can't decide, the procedure is refused,** and the message names the types that would fit and asks
  which, suggesting none: `a + b` alone fits both `Whole` and `Float` (D5).
- **The result type comes from the `return`s,** which must all agree; a procedure with no `return` of a value gives no
  result (D6).
- **Writing a type is always allowed** (D5, D6).
- **The host's calls are checked against these types** at load time (D5, D6, chapter 9).

## `return`

`return value` gives back a value, and a bare `return` leaves early, as in Unio (Unio's standard, chapter 4, D57).

- **If any `return` gives a value, every way through must end in one;** a `while` counts as able to end, so a loop
  with `while true` still needs a `return` after it (D68, Unio D57, D149).
- **Nothing may follow `return`, `break` or `fail` in the same block,** and a bare `return` as a procedure's last line
  is refused (D68, Unio D57).
- **When any way through an `if` leaves, nothing may follow its `end`;** the rest goes in the ways that don't leave
  (D68, Unio D9).

## Calls

A call writes the procedure's name and `( )`, with its arguments inside (Unio D3, D4, D165).

- **Labels:** a procedure's writer can give its arguments labels, and every call writes them, as in
  `heal(50, by: 10)` (D22, Unio D24). By convention the first argument has none.
- **Slides:** a chain of three or more calls, each the first argument of the next, is written as a slide with `|>`,
  and one or two as calls (D22, Unio D5, D128):

```
points |> double |> be.Text |> show_msg
show_msg(be.Text(n))
```

Unio's standard, chapter 4, has the rules for slides in full.

A procedure declares its labels on a `labels` line, the first line of its body, with its list in `[ ]` (D41, D42,
Unio D25). A `defaults` line may follow it, giving a value for a labelled argument a call leaves out, written as the call
would write it (D42):

```
proc heal[target, amount, times]
    labels [amount as by, times as times]
    defaults [times: 1]
    hp = hp + amount * times
end

heal(goblin, by: 10)              // means heal(goblin, by: 10, times: 1)
```

- **Only labelled parameters can have a default** (D42).
- **A default is a plain value,** never a calculation or a call (D42).
- **The host's calls can leave defaulted arguments out too** (D42).

**Not decided:** a value-giving call standing alone (Unio D120, D161).
