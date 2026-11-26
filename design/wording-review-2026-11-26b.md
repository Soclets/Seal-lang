# The design session's review of the 111 newer draft wordings

The drafts marked **(new since the review)** in `PROGRESS.md` (commit 78cb5bc), reviewed on 2026-11-26 by the design
session, as Unio D132 has it. D162's "variable" rewording, the generics, methods and requirements read well and stand
as drafted, apart from what's below. None needed Tankun.

## Decided since you drafted them

- "`{full}` on {found}: what a type parameter can do isn't decided yet" and "{purpose}, on {found}: what a type
  parameter can do isn't decided yet" are decided by D166: a type parameter with no requirement takes any type, and
  its values can only be stored, passed on, returned, compared with `eq` and `not_eq`, and shown in braces; `Number`
  allows maths and ordering; a requirement allows its methods. So they're refusals now, as "`?Item` has no requirement
  that allows `+`" / why: "a type parameter with no requirement takes any type, so only what every type can do is
  allowed; `<?Item: Number>` allows maths and ordering." A method call on a `?Item` whose requirement doesn't list it
  is refused the same way, naming the requirement.
- D169 (signing), D170 (no type-following verifier) and D171 (a `.sealc` loading goal) need wordings of their own when
  built, as a missing or wrong signature: "this file's signature doesn't match the host's key" / why: "a host that
  requires signatures loads only files signed with its own key, so a file changed in any way is refused."

## Single wordings

- "{what} {}", the whole message held in its parts: show the design session the sentences it can make, so they can be
  reviewed.
- "`{name}` already names a type or a requirement" gets `why: each name means one thing` (D71).
- The "comes after a method" refusal's `why:` reads "{belong} come first, then its methods": check that every filling
  makes a sentence, as "a record's fields come first" and "a `one_of`'s cases come first".

## The C API's host mistakes

No `why:` is needed: a host's programmer knows C. What each needs instead is **the function and the parameter**, since
a C programmer reading `SEAL_HOST_MISTAKE` has only the message to find the line:
- "nowhere to write the count" becomes "`seal_value_count`: `count` is null, so there's nowhere to write the count",
  and the same for every "nowhere to write …".
- "a door's value type that's null or isn't UTF-8" becomes "`seal_doors_add`: value type {n} is null or isn't UTF-8",
  and the same for labels and failures, naming which one by its position.
- "labels that aren't there" becomes "`seal_script_call`: `labels` is null, but `count` says there are {n}".
- "`seal_value_number` makes a `HugeWhole`, `Decimal` or `HugeDecimal`" doesn't say what went wrong; say what the host
  did and what it should do instead.
- Handles stay as drafted ("a value handle that's freed, or was never one"), with the function's name in front.

## New open questions, for the design session to settle with Tankun

These stops are right, and the design session has them on its list: a lending method called on a field or a list
item; a lending method that can fail; `mut` parameters outside methods (Unio D130); and a host calling a method or a
generic procedure by name.
