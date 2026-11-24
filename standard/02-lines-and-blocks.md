# 2. Lines and blocks

```
mut tries = 0
while tries < 3
    show_msg("Try {tries}")
    tries = tries + 1
end while
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
lines after it, and it closes at the opening line's depth with `end` followed by what it closes, as Unio's do (D20,
D76, Unio D8):

```
opening line
    body
end what
```

where `opening line` is the block's first line, `body` its statements, and `what` is what the block is:

| Block | Closer |
|---|---|
| `if`, with its `elif` and `else` | `end if` |
| `while` | `end while` |
| `for each` | `end for` |
| `match` | `end match` |
| a call's `otherwise` | `end otherwise` |
| `proc heal[…]` | `end proc heal` |
| `record Enemy` | `end record Enemy` |
| `one_of Event` | `end one_of Event` |
| `fam.proc spawn` | `end fam.proc spawn` |

- **A closer at the wrong depth, or naming another block, is refused** (D20, D76).
- **A missing closer is reported naming the block it should close,** as in "the `while` on line 3 has no `end
  while`" (D20, D76).
- **The language server types the closer** when a block is opened (D76).

## Indentation

Indentation is required: a body is indented one level deeper than its opening line (D9, D20). One level is 4 spaces,
and anything else, a tab included, is refused, with the corrected line as the fix (D63, Unio D10).

**Not decided:** how a statement's later lines are indented.

## Spacing and comments

- **Spacing is fixed,** as Unio's table has it (Unio's standard, chapter 2), with one space on each side of operators,
  `->`, `!`, `==` and `!=` included, and none inside brackets; a mistake is refused with the corrected line as the fix
  (D63, Unio D98, D100). The one exception is a parameter group's type, which has a space on each side of its `:`, as
  `[hp, amount : Whole]` (D88).
- **`//` comments to the end of the line,** with one space before it after code, as `x = 1 // hi`; `x = 1// hi` is
  refused, with the spaced line as the fix (D63, D91, Unio D74).
- **A comment on a line of its own is indented like the line after it,** inside an open bracket too (D63, D91, Unio
  D145).
- **`///` is a doc comment** on the lines right before a declaration: a procedure, a family, a record, a `one_of`, a
  field, a case or a top-level name (D63, D91, Unio D74).
- **`comment name` on its own line starts a block comment,** closed by `end comment name`, as in Unio; everything
  between is ignored. `comment` is a keyword, so it can't be a name (D63, D76, D91, Unio D74).
- **One blank line in a row is fine; two or more are refused** (D63, Unio D97).
- **`\r\n` counts as one line break,** and in text **`\`** makes the next character literal (D63, Unio D30, D144):
  `\"`, `\\` and `\{`. A `\` before any other character, as in `"\n"`, is refused, and so is a tab character inside
  text, since it can't be told from spaces (D92).

**Not decided:** how a new line or a tab is written inside text.

## Files

A script is one file ending in `.seal` (D25).

**Not decided:** whether a script can use another script.
