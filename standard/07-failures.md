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
- **`??`** handles it on one line, with a value, `ignore`, `crash` or a handler (Unio D23, D121, D98).
- **What deals with a failure runs only when there is one:** a fallback after `??`, as `?? default_level()`, and a
  handler's arguments are worked out only when the call fails (D130).
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
    end
end
```

## Handlers

A handler is a named, reusable set of `case`s. Handlers live in a handler group, a named set reached with `::`, as
Cyborg reaches every named set (D110, Cyborg D488, Unio D41 to D44):

```
group.handler level_failed
    handles [load_level, load_save]
    ::start_over
        case not_found
            instead 1
        case broken
            instead 1
    end
    ::start_at[level: Whole]
        case not_found
            instead level
        case broken
            instead level
    end
end

a = load_level("forest") ?? ::start_over
b = load_save("slot1") ?? level_failed::start_at(3)
c = load_level("cave") otherwise
    include ::start_over
end
```

- **`group.handler name`** declares a group: "a handler kind of group". It isn't a family, since its members aren't
  kinds of its name (D110).
- **Each member is declared as `::name`,** the mark it's reached with (D110).
- **`handles [ … ]`,** an optional first line, lists the procedures the group is for: any that can fail, the
  script's own, narrowed ones as `spawn.boss`, the host's doors, and built-ins as `be.Whole` (D129). Right after a call to one of
  them, `::start_over` reaches into the groups that handle it; if two such groups both have that member, the short form
  is refused there (D110).
- **The full form, `level_failed::start_over`, works anywhere** (D110).
- **On one line, a handler follows `??`,** as a value, `ignore` or `crash` would; `otherwise` is only ever a block
  (D98).
- **It's used as if its cases were written at the call,** so it must cover every failure still possible there (Unio
  D40, D42). Its cases for failures that call can't give are allowed and never run there, since a handler is shared
  across calls; a case written straight into an `otherwise` block for a failure its call can't give is refused (D131).
- **A handler that can `return` or `break` is marked `*` where it's used,** as `?? ::*give_up` or
  `?? level_failed::*give_up`; the mark is required on one that can jump and refused on one that can't (Unio D42).
  A `fail` inside a handler's case leaves the procedure too, so it counts as a jump (D129).
- **`include`** inside an `otherwise` block brings in a handler's cases beside the block's own; together they cover
  every failure, none twice (Unio D43).
- **A handler can take parameters,** written as a procedure's are, and is given arguments where it's used (Unio D44).
  The arguments are worked out only when the call fails, right before the handler's cases run (D130).
