# Seal's decisions

Every design choice for Seal, with what was weighed and why it won, in the order made. Tankun designs Seal with the
design session; the choices are Tankun's. Seal is inspired by Unio, so many entries point to the Unio decision
(`Unio D…`) they follow or differ from.

## D1. Seal: an interpreted, embeddable, sandboxed scripting language, inspired by Unio, in Rust

2026-11-23.

**Chosen.**
- **Seal is a scripting language** that's interpreted, embedded in other programs, and sandboxed: it's sealed, so a
  script reaches nothing outside itself except through the doors the program embedding it opens.
- **It's meant to be easier to write than Unio,** and is inspired by Unio rather than defined as Unio with changes:
  each part is decided for Seal, borrowing Unio's choices where they fit.
- **It's implemented in Rust,** by a building session of its own. It's designed in the design session Unio uses.
- **Its repository is `Soclets/Seal-lang`,** public, with no license yet.

**Weighed.**
- "Seal is Unio, except", with every Unio decision applying unless a Seal decision says otherwise, as the design
  claude proposed. Seal is "at least mostly inspired", not bound to follow Unio.

**Why this won.** Unio's design suits a sealed language: its built-ins are already the only doors to the operating
system, and since nothing can keep a reference (Unio D130), reference counting with copy-on-write needs no garbage
collector, even in an interpreter.

**Tankun's words.** "Mf, what's up. Feeling like making Seal (it's sealed) alongside Unio. Design here, impl by
another. 
Seal is an interpreted, embeddable, sandboxed, Unio-based scripting language (meant to be easier to write),
implmented in Rust.", "Bro, Luna was a troll languag", "Well, it's at least mostly inspired. Show me the Unio game,
and I'll rewrite it in Seal sketch".

## D2. `x = …` makes a name only when it's new, changes it only when it's `mut`, and an unused name is refused

2026-11-23.

**Chosen.**
- **`x = value` makes `x` when no name `x` exists yet,** with no word in front, and its type worked out from the value.
- **When `x` already exists, `x = value` changes it,** which is allowed only if it was made with `mut x = …`.
- **A name that's made and never read is refused,** stating plainly that it's never used, with no guess at what was
  meant (as Unio D54). So a typo like `tires = tries + 1` is caught at once.
- A way to make a throwaway name is D3.

```
mut tries = 0
tries = tries + 1     // changes tries
tires = tries + 1     // refused: `tires` is never used
```

**Weighed.**
- Unio's `let` and `var` in front of every new name (Unio D13). Safe, but heavier than Seal wants.
- `x = …` both making and changing names with nothing else, as Python has it. A typo silently makes a new name, and
  the real one never changes.

**Why this won.** It keeps Tankun's sketch as light as Python while catching Python's classic silent typo, by
refusing the unused name the typo makes.

**Tankun's words.** Tankun's sketch of the guessing game (has code), then, asked "Should `x = …` make a name only when it's
new, change it only when it's `mut`, and a name that's never used be refused?", the owner chose: "yea, but we should
have a way to make a throwaway. Also, make `Soclets/Seal-lang` pub too, no license uet".

## D3. `unused` marks a throwaway name

2026-11-23.

**Chosen.** `unused`, in the same place as `mut`, makes a name that's never read on purpose, so D2 doesn't refuse it.
Throwing a value away is the same: give it a name and mark it `unused`. Reading an `unused` name later is refused.

```
unused old_score = score
unused saved = save(game) ?? 0
```

**Weighed.**
- A leading `_`, as Rust, Python, Go and Swift have it, with `_` alone throwing a value away. Short, but a convention
  to learn, where a word says it plainly.

**Why this won.** It's the plain-word way, as Unio writes `at_least` for `>=`, it reads alike with `mut`, and one form
covers both naming a throwaway and discarding a value. Throwaways are rare, so the length hardly matters.

**Tankun's words.** "What about `unused`?", then, asked "Should Seal mark a throwaway with `unused`, in the same place
as `mut`, with reading it later refused?", the owner chose: "yea".

## D4. A script's top level runs once when it's loaded, and the host calls its procedures after

2026-11-23.

**Chosen.**
- **A Seal script has no `proc.main`.** Its top level runs once, from top to bottom, when the host loads it, setting
  things up.
- **The host then calls the script's procedures** by name, whenever its own events happen, as a game engine calls an
  enemy's `on_hit`.
- **The script calls the host only through its doors,** the procedures the host gives it, such as `show_msg`.
- **A script with no procedures is just a program,** run from top to bottom, like the guessing game.

**Weighed.**
- Unio's `proc.main`. Right for a program the operating system starts once; an embedded script is loaded by
  a host that keeps calling into it.

**Why this won.** It's how embedded scripting works in practice, as Lua does in World of Warcraft and Roblox: the
engine calls the script's handlers, and the script calls back through the engine's functions. Tankun's sketch already
has no `proc.main`.

**Tankun's words.** "Wait, the host calls or Seal calls?", then, asked "Should a Seal script run its top level once
when loaded, with no `proc.main`, and the host then calling its procedures?", the owner chose: "yea. Now, a very
important question, should we allow `proc` to be gradual?"

## D5. A procedure's parameter types may be left out when its own body decides them

2026-11-23.

**Chosen.**
- **A procedure's parameter types are optional.** Left out, each is worked out from the procedure's own body when the
  script loads, so the result is fully static, exactly as if the type were written.
- **Only the procedure's own body counts,** never its callers, so a mistake is found where it is.
- **When the body can't decide,** as `a + b` can't between `Whole` and `Float`, the procedure is refused. The message
  names the types that would fit and asks which, without suggesting one, since it can't know (Unio D54).
- **Writing a type is always allowed.** The host's calls are checked against the types, written or worked out.

```
mut score = 0

proc on_hit[points]           // points: Whole, since score + points needs two Wholes
    score = score + points
end proc

proc add[a, b]                // refused: a and b could be Whole or Float; write which
    return a + b
end proc
```

**Weighed.**
- Gradual typing, where a parameter with no type is dynamic and checked only while running, as in TypeScript. Tankun
  meant static inference, not that.
- Types always written on parameters and results, as Rust, Swift, Kotlin and Go have it, as the design claude first
  recommended. Heavier than Seal wants where the body makes them obvious.
- Working types out from callers too, as Haskell and OCaml can. A mistake in one place could change a type elsewhere
  and be reported far from where it is.
- A refusal suggesting one type, as the design claude first drafted it. Tankun: it can't know which.

**Why this won.** Types are left off where the procedure's own lines make them obvious, and written where they don't,
while everything stays static and every mistake is reported in the procedure that has it.

How a procedure's result type is worked out or written, and the refusal's wording, aren't decided yet. (The result
type: D6.)

**Tankun's words.** "Not dynamic param, static infer param. So, that's called gradual, right?", "What does that
mean?", then, asked "Should a procedure's parameter types be optional, worked out from its own body, with the type
required when the body can't decide?", the owner chose: "Ah, yes. But "and asks you to write it: proc add[a: Whole, b:
Whole]", you said it doesn't know".

## D6. A procedure's result type may be left out too, worked out from its `return`s

2026-11-23.

**Chosen.**
- **A procedure's result type is optional,** worked out from its own `return` lines when the script loads, as D5 does
  for parameters.
- **`returns Type` may always be written,** as Unio writes it, for anyone who wants it said.
- **Every `return` must agree:** one giving a `Whole` and another a `Text` is refused.
- **A procedure with no `return` of a value gives no result.**
- **The host's calls are checked against the result type too,** so a Rust call expecting a `Whole` back is checked at
  load time.

```
proc double[n: Whole]                               // result: Whole
    return n * 2
end proc

proc average[a: Float, b: Float] returns Float      // written, as allowed
    return (a + b) / 2.0
end proc
```

How a procedure's first line is written, with or without `:`, isn't decided yet; these examples only show the shape.

**Weighed.**
- `returns` always written, as Unio has it. The same reasoning as D5: the body already says it.

**Why this won.** It completes D5, so a procedure's types are written only where its own lines leave them open.

**Tankun's words.** Asked "Should a procedure's result type be optional too, worked out from its `return`s, with
`returns` allowed for writing it?", the owner chose: "yea".

## D7. Names have no `$`, and Seal's language server colours each kind of name from the start

2026-11-23.

**Chosen.**
- **Seal's names have no `$`,** unlike Unio's (Unio D11): `mut tries = 0`, and `{tries}` in `.fmt` text.
- **A name can't be a keyword:** `mut if = 3` is refused.
- **Seal's language server speaks LSP** (the Language Server Protocol, which any editor or host can plug into) and,
  from the start, gives variables, procedures, narrowings and methods their own colours, so `score.add(1)`, a method on
  a value, and `show_msg.fmt(…)`, a narrowing of a procedure, look different in an editor.

**Weighed.**
- `$` on every name, as Unio has it. It tells a method from a narrowing on the page, with no editor; Seal scripts are
  short and nearly always written in an editor, often one the host provides, as Roblox Studio is for Luau.

**Why this won.** Seal isn't standalone: its scripts live in a host's project and are written in an editor, where the
language server tells the kinds of name apart, so `$` costs more to write than it gives.

**Tankun's words.** "IDEs do help clarify, right? Like a narrowing vs a method", "Here's the thing, Seal unlike Unio,
is not standalone. Which usually makes it IDE'd, right?", then, asked "Should Seal names have no `$`, with Seal's
language server speaking LSP and colouring variables, procedures, narrowings and methods apart from the start?", the
owner chose: "yea".

## D8. A statement ends at the end of its line, carrying on only inside an open `(` or `[`

2026-11-23.

**Chosen.**
- **There's no `;`.** The end of a line ends its statement.
- **Inside an open `(` or `[`, a line break carries on** until the bracket closes, so a long call or list can be split.
- **A line that ends where a value is still wanted is refused,** as `total = a + b +` with `c` on the next line; the
  fix is to wrap it in `( )`.
- **There's no `\` to carry a line on,** and two statements can't share a line, as in Unio (Unio D146).

```
spawn_enemy(kind: "goblin", x: 10, y: 20,
            health: 30, speed: 2)
total = (first_round + second_round +
         third_round)
```

**Weighed.**
- Unio's `;` (Unio D7). Seal is meant to be lighter to write, and a line already ends where a statement does.
- A `\` at the end of a line to carry it on. Easy to miss, and broken by a space after it.

**Why this won.** It's Python's rule, proven over 30 years: the one thing to learn is that an open bracket keeps a line
going.

**Tankun's words.** "What does that make?", then, asked "Should a statement end at the end of its line, with no `;`,
carrying on only inside an open `(` or `[`?", the owner chose: "yea".

## D9. Lines that open a block have no `:`

2026-11-23.

**Chosen.** `if`, `elif`, `else`, `while` and every other line that opens a block end with no `:`. The block is the
deeper-indented lines after it, closed by its `end` line, and indentation is required.

```
if guess < secret
    show_msg("Too low")
elif guess > secret
    show_msg("Too high")
end if
```

**Weighed.**
- Unio's `:` (Unio D8). It marks where a block starts, but in Seal the deeper indentation and the closing `end` line
  already say so.

**Why this won.** The `:` repeats what the indentation and the `end` line both say; dropping it leaves nothing to
forget, and a wrongly indented line is still refused.

How `elif` is spelt is its own decision; this one is about the `:` only.

**Tankun's words.** Asked "Should block-opening lines like `if` and `while` have no `:` in Seal?", the owner chose:
"yea".
