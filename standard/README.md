# The Seal standard

What Seal is now, by topic. [`DECISIONS.md`](../DECISIONS.md) records how each rule was decided; this standard says
what the rules are today.

## How it's written

- **`DECISIONS.md` is the source.** Every rule here names its decision, as in "(D7)". Where the two disagree, the
  decision wins and the standard is wrong.
- **Seal takes many rules from Unio** (D15, D16, D22). Those name Unio's decision, as in "(Unio D170)", and point to the
  chapter of [Unio's standard](https://github.com/Soclets/Unio-lang/tree/main/standard) that has them in full. Unio's
  examples are read in Seal's syntax, as the table below shows.
- **Undecided means "Not decided".** Anything the decisions leave open is marked **Not decided** where it would go,
  never filled in here. A building session stops there with "not designed yet" (IMPLEMENTATION.md, rule 11).

## Reading Unio's rules in Seal's syntax

Where a chapter here points to Unio's standard, Unio's examples change only as Seal's decisions say:

| Unio | Seal | Decided in |
|---|---|---|
| `proc.main:` … `end proc.main` around the program | the script's top level, run once when it's loaded | D4 |
| `let $x: Whole = 0;` | `x = 0`, the type worked out | D2, D7, D8 |
| `var $x: Whole = 0;` | `mut x = 0` | D2 |
| `$x` | `x` | D7 |
| `;` at the end of a statement | nothing: the line ends it | D8 |
| `:` at the end of an opening line | nothing | D9 |
| `end if`, `end while`, `end proc name`, `end otherwise` | `end` | D20 |
| `else if` | `elif` | D10 |
| `proc name[$a: Whole] returns Whole:` | `proc name[a: Whole] returns Whole`, types and `returns` optional | D5, D6, D21 |
| `show_msg.fmt("{$x}")` | `fmt("{x}")`, given to whatever the host's door is | D12 |
| `show_msg`, `ask`, `random` | doors, if the host opens them, under the names it chooses | D11 |
| `.unio` | `.seal` | D25 |

## Chapters

1. [Terms](01-terms.md)
2. [Lines and blocks](02-lines-and-blocks.md): lines, `end`, indentation, files
3. [Names and values](03-names-and-values.md): making and changing names, `mut`, `unused`, names' spelling, values
4. [Procedures](04-procedures.md): definitions, worked-out types, calls, labels and slides
5. [Control flow](05-control-flow.md): `if`, `elif`, comparisons, logic, `while`, `for each` and `break`
6. [Types](06-types.md): `Whole`, `Float`, `Bool`, `Text`, lists and arrays, records, dictionaries, options and `match`, `Maybe`, `fmt`, converting
7. [Failures](07-failures.md): `?`, `??` and `otherwise`
8. [Built in](08-built-in.md): what Seal provides itself
9. [Hosts](09-hosts.md): loading, doors, narrowings on doors, IDs and limits
10. [Checks and messages](10-checks-and-messages.md): when checks run, build modes, and how the host is told
