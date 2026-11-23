# 2. Lines and blocks

```
mut tries = 0
while tries < 3
    show_msg(fmt("Try {tries}"))
    tries = tries + 1
end
```

## Lines

- **The end of a line ends its statement.** There's no `;` (D8).
- **Inside an open `(` or `[`, a line break carries on** until the bracket closes (D8):

```
spawn_enemy(kind: "goblin", x: 10, y: 20,
            health: 30, speed: 2)
total = (first_round + second_round +
         third_round)
```

- **A line that ends where a value is still wanted is refused,** as `total = a + b +` with the rest on the next line; the
  fix wraps it in `( )` (D8).
- **There's no `\` to carry a line on,** and two statements can't share a line (D8, Unio D146).

## Blocks

A block's opening line, such as `if`, `while` or `proc`, has no `:` at its end (D9). Its body is the deeper-indented
lines after it, and it closes with a plain `end` at the opening line's depth (D20):

```
opening line
    body
end
```

where `opening line` is the block's first line and `body` its statements.

- **An `end` at the wrong depth is refused** (D20).
- **A missing `end` is reported naming the block it should close,** as in "the `while` on line 3 has no `end`" (D20).

## Indentation

Indentation is required: a body is indented one level deeper than its opening line (D9, D20).

**Not decided:** how many spaces one level is; tabs; how a statement's later lines are indented.

## Files

A script is one file ending in `.seal` (D25).

**Not decided:** comments; blank lines; spacing around operators; line endings; whether a script can use another
script.
