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
- **A condition must be a `Bool`:** `if lives` is refused, with `if lives not_eq 0` as the fix (D16, Unio D187).

## Comparisons and logic

`eq`, `not_eq`, `<`, `>`, `at_least` and `at_most` compare, and `and`, `or` and `not` combine (D22, D158, Unio D17,
D34, D190). Comparisons on `Float`s are exact (Unio D184).

Unio's standard, chapter 5, has the rules in full.

## `while` and `break`

```
mut tries = 0
while tries < 7 and not won
    tries = tries + 1
end
```

`while` runs its body as long as its condition holds, and `break` leaves the loop (D22, Unio D19, D32). `eq` and
`not_eq` between two `Float`s are refused in a `while` condition (Unio D184).

## `for_each`

`for_each` is one keyword, its two words joined as `one_of`'s and `at_least`'s are (D146).

```
for_each name at pos i in names
    show_msg("{i}. {name}")
end
```

As in Unio (D22, Unio D171 to D174):
- **It goes through a `List`, an `Array` or a `range[…]` kept in a variable,** in order, or a `Dict`'s values, with `at
  key name` giving each key (D33); `at pos` and `at offset` on a `Dict` are refused, with `at key` as the fix (D105).
- **The item's name is new for each pass,** made by the `for_each` line, and ends with the loop.
- **`at pos i` or `at offset i`** gives the position too, counting from 1 or from 0.
- **What it goes through can't change inside it,** nor through a call that can change it, and a loop going through a
  top-level name can't pause; copying it into a name of its own first is the fix (D69).
- **`break` leaves it.**

`range` is in chapter 8. Unio's standard, chapter 5, has the rules in full.

`for_each` goes through a text by graphemes, what a person sees as one character (D65, D66).

## `for`

`for` counts, with `range`'s words, `from`, `to` or `before`, and `step_by`, written without brackets or colons
(D172):

```
for round from 1 to 3
    show_msg("Round {round}!")
end
for i from 0 before size
    show_msg("{i}")
end
for n from 10 to 0 step_by -2
    show_msg("{n}")
end
```

- **`to` includes the last number, `before` stops short of it,** and `step_by` is optional, counting by 1, negative to
  count down, as `range`'s labels are (Unio D170, D172). A step of 0, or one pointing away from the end, is refused
  where Seal can see it and stops with a failed check otherwise, as for `range`.
- **The counter is a new immutable variable for each pass,** ending with the loop; `break` leaves it.
- **`for_each` goes through items; `for` counts.** A `range[…]` written straight into a `for_each`, as
  `for_each i in range[from: 1, to: 3]`, is refused, with `for i from 1 to 3` as the fix; a range kept in a variable
  is a list, gone through with `for_each` (D99).
- **`for` with `in`,** as Python writes `for x in xs`, is refused, with `for_each x in xs` as the fix.

**Not decided:** leaving an outer loop; other loops.
