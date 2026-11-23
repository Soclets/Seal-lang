# 5. Control flow

## `if`

```
if guess < secret
    show_msg("Too low")
elif guess > secret
    show_msg("Too high")
else
    show_msg("Correct!")
end
```

- **No `:`** after `if`, `elif` or `else` (D9), **one `end`** for the whole `if` (D20).
- **`elif`** is short for "else if" (D10, D40). Unio writes `else if`, and Seal's `elif` is a deliberate difference.
- **A condition must be a `Bool`:** `if lives` is refused, with `if lives != 0` as the fix (D16, Unio D187).

## Comparisons and logic

`==`, `!=`, `<`, `>`, `at_least` and `at_most` compare, and `and`, `or` and `not` combine (D22, Unio D17, D34). Unio
now writes `eq` and `not_eq` (Unio D190); Seal keeps `==` and `!=` (D59). Comparisons on `Float`s are exact (Unio D184).

Unio's standard, chapter 5, has the rules in full.

## `while` and `break`

```
mut tries = 0
while tries < 7 and not won
    tries = tries + 1
end
```

`while` runs its body as long as its condition holds, and `break` leaves the loop (D22, Unio D19, D32). `==` and `!=`
between two `Float`s are refused in a `while` condition (Unio D184).

## `for each`

```
for each name at pos i in names
    show_msg(fmt("{i}. {name}"))
end
for each round in range[from: 1, to: 3]
    show_msg(fmt("Round {round}!"))
end
```

As in Unio (D22, Unio D171 to D174):
- **It goes through a `List`, an `Array` or a `range[…]`,** in order, or a `Dict`'s values, with `at key name` giving
  each key (D33).
- **The item's name is new for each pass,** made by the `for each` line, and ends with the loop.
- **`at pos i` or `at offset i`** gives the position too, counting from 1 or from 0.
- **What it goes through can't change inside it.**
- **`break` leaves it.**

`range` is in chapter 8. Unio's standard, chapter 5, has the rules in full.

`for each` goes through a text by graphemes, what a person sees as one character (D65, D66).

**Not decided:** leaving an outer loop; other loops.
