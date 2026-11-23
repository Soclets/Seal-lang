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
end
bag.add("Gem") otherwise
    case full
        show_msg("Your bag is full!")
end
```

- **`?`** passes the failure up (Unio D23).
- **`??`** handles it on one line, with a value, `ignore` or `crash` (Unio D23, D121).
- **`otherwise`** handles each failure in its own `case`, ending in `instead`, `return`, `break` or failing on
  purpose, or simply ending when the call gives back nothing (Unio D40, D186).

In a script, `?? crash` stops the script, not the host, which is told (D14, chapter 10).

`otherwise` and `case` lines have no `:` (D9), and the `otherwise` closes with one `end` (D20).

Each `case` runs until the next `case`, as in Unio (D37).

## Failing on purpose

`fail name` fails on purpose, and a procedure's failures are worked out from its body: every `fail`, and every failure
passed up with `?` (D44). They may be written on the first line as `! [not_found, broken]`, after `-> Type` if
that's written, and must then match the body (D44, D45). A door can fail too, with failures its host declares (D44).

```
proc load_level[name]
    if not level_exists(name)
        fail not_found
    end
    return read_level(name)?
end
```

**Not decided:** handler families, which come with handlers (D43).
