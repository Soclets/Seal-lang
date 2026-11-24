# 7. Failures

A failure is something a procedure can give back instead of its result, and every call to one that can fail must deal
with it, as in Unio (D22, Unio D23). Unio's standard, chapter 8, has the rules in full.

```
guess = be.Whole(typed) ?? 0
save(game) ?? ignore
load_level(name) ?? crash
best = load_best() otherwise
    case not_found
        instead 0
end otherwise
bag.add("Gem") otherwise
    case full
        show_msg("Your bag is full!")
end otherwise
```

- **`?`** passes the failure up (Unio D23).
- **`??`** handles it on one line, with a value, `ignore`, `crash` or a handler (Unio D23, D121, D98).
- **`otherwise`** handles each failure in its own `case`, ending in `instead`, `return`, `break` or failing on
  purpose, or simply ending when the call gives back nothing (Unio D40, D186).
- **`instead` is the last line of its case:** a line after it can never run, so it's refused, as one after `return`
  or `break` is (D80, Unio D57).

In a script, `?? crash` stops the script, not the host, which is told (D14, chapter 10).

`otherwise` and `case` lines have no `:` (D9), and the `otherwise` closes with one `end` (D20).

Each `case` runs until the next `case`, as in Unio (D37).

## Failing on purpose

`fail name` fails on purpose (D44). A procedure that can fail lists its failures on its first line, as `! [not_found,
broken]`, after `-> Type` if it has one: every `fail` in its body, and every failure it passes up with `?`. A list
that doesn't match the body is refused, with the right list as the fix (D44, D45, D84). A door can fail too, with
failures its host declares (D44).

```
proc load_level[name: Text] -> Level ! [not_found, broken]
    if not level_exists(name)
        fail not_found
    else
        return read_level(name)?
    end if
end proc load_level
```

## Handlers

A handler is a named, reusable set of `case`s, declared in a `fam.handler` named after what fails, as Unio's are (D97,
Unio D41 to D44):

```
fam.handler level_failed
    start_over
        case not_found
            instead 1
        case broken
            instead 1
    end start_over
    start_at[level: Whole]
        case not_found
            instead level
        case broken
            instead level
    end start_at
end fam.handler level_failed

a = load_level("forest") ?? level_failed.start_over
b = load_level("cave") ?? level_failed.start_at(level: 3)
```

- **A member is written by its short name,** with no `handler`, since a handler is only ever used after `??` or
  inside an `otherwise` block; it's used by its full name, as `level_failed.start_over` (D97, D98).
- **On one line, a handler follows `??`,** as a value, `ignore` or `crash` would; `otherwise` is only ever a block
  (D98).
- **It's used as if its cases were written at the call,** so it must cover every failure still possible there (Unio
  D40, D42).
- **A handler that can `return` or `break` is marked `*` where it's used,** as `?? level_failed.*give_up`; the
  mark is required on one that can jump and refused on one that can't (Unio D42).
- **`include level_failed.start_over`** inside an `otherwise` block brings in its cases beside the block's own; together
  they cover every failure, none twice (Unio D43).
- **A handler can take parameters,** written as a procedure's are, and is given arguments where it's used (Unio D44).
