# 4. Procedures

## Definitions

```
proc name[parameters] returns Type
    body
end
```

where `name` is the procedure's name, `parameters` its parameters separated by commas, `Type` the type it gives back,
and `body` its statements. `proc`, `returns`, the brackets and `end` are written as they are (D20, D21).

```
proc on_hit[points]
    score = score + points
end

proc average[a: Float, b: Float] returns Float
    return (a + b) / 2.0
end
```

- **Each parameter is its name, with its type optional,** as `points` or `a: Float` (D5, D21).
- **`returns Type` is optional** (D6, D21).
- **The line has no `:`** (D9).

Seal has no contracts, `requires` or `ensures`, for now (D39).

**Not decided:** a procedure with no parameters; parameter words for reading, changing or taking over (Unio D130);
listing the failures a procedure can give; narrowings on a script's own procedures (D13); families.

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

**Not decided:** whether Unio's other rules for `return` (Unio D57, D149) hold in Seal.

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

A procedure declares its labels on a `labels` line, the first line of its body (D41, Unio D25):

```
proc heal[target, amount, times]
    labels amount as by, times as times
    hp = hp + amount * times
end
```

**Not decided:** a value-giving call standing alone (Unio D120, D161).
