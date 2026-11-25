# 8. Built in

Seal's built-ins are only what touches nothing outside the script, so they behave the same in every host (D11). Showing
text, asking, files, time, sound and randomness aren't built in: they're doors, if the host opens them (chapter 9).

| Built-in | What it does | Decided in |
|---|---|---|
| `be.Whole`, `be.Float`, `be.Text`, `be.Bool` and their narrowings | convert | D22, D101, Unio D22, D178 |
| `range[from:, to: or before:, step_by:]` | a run of whole numbers: a `List<Whole>`, usable anywhere a list is | D22, D99, Unio D170, D172 |
| `sqrt(x)` | a `Float`'s square root, stopping on a negative number | D22, Unio D185 |
| `abs(x)` | a `Whole`'s or `Float`'s distance from zero; `abs` of the smallest `Whole` stops with the overflow check | D107 |
| `min(a, b)`, `max(a, b)` | the smaller or the larger of two values of the same type, `Whole` or `Float` | D107 |
| `clamp(x, low:, high:)` | `x`, pulled up to `low` or down to `high` if it's past either; all three of one type | D107 |
| `sin(x)`, `cos(x)`, `tan(x)` | a `Float` angle's sine, cosine and tangent, in radians | D109 |
| `asin(x)`, `acos(x)`, `atan2(y:, x:)` | the angle, in radians, for a sine, a cosine, or a point; `asin` and `acos` stop outside -1 to 1 | D109 |
| `.degrees` on each, as `sin.degrees(90.0)` | the same in degrees, a full turn being 360 | D109 |
| `pi` | the `Float` nearest π | D109 |
| lists' and arrays' methods and facts | `.add`, `.remove`, `.len` | D22, Unio D77 to D80 |

Unio's standard, chapter 9, has `range` and `sqrt` in full.

**Not decided:** every other built-in.
