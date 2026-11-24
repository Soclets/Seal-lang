# 8. Built in

Seal's built-ins are only what touches nothing outside the script, so they behave the same in every host (D11). Showing
text, asking, files, time, sound and randomness aren't built in: they're doors, if the host opens them (chapter 9).

| Built-in | What it does | Decided in |
|---|---|---|
| `be.Whole`, `be.Float`, `be.Text`, `be.Bool` and their narrowings | convert | D22, D101, Unio D22, D178 |
| `range[from:, to: or before:, step_by:]` | a run of whole numbers: a `List<Whole>`, usable anywhere a list is | D22, D99, Unio D170, D172 |
| `sqrt(x)` | a `Float`'s square root, stopping on a negative number | D22, Unio D185 |
| lists' and arrays' methods and facts | `.add`, `.remove`, `.len` | D22, Unio D77 to D80 |

Unio's standard, chapter 9, has `range` and `sqrt` in full.

**Not decided:** every other built-in.
