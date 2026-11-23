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

How a procedure's first line is written, with or without `:`, isn't decided yet; these examples only show the shape. (Decided
by D9, D20 and D21.)

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

## D10. `elif`, taught as short for "else if"

2026-11-23.

**Chosen.** Seal writes `elif` where Unio writes `else if`. It's one keyword, and the docs teach it as short for "else
if", as Unio teaches `fmt` and `pos` (Unio D2).

```
if guess < secret
    show_msg("Too low")
elif guess > secret
    show_msg("Too high")
else
    show_msg("Correct!")
end if
```

**Weighed.**
- `else if`, as Unio has it. Plain English, but two keywords doing one job.
- `elsif`, as Ruby has it. Less known.

**Why this won.** One keyword for one job, in the short spelling Unio's D2 prefers, and already known from Python and
Bash. Tankun's sketch used it.

**Tankun's words.** Asked "Should Seal use `elif`, taught as short for "else if"?", the owner chose: "yea".

## D11. Built-ins touch nothing outside the script, and every door is made and named by the host

2026-11-23.

**Chosen.**
- **Seal's built-ins are only what touches nothing outside the script:** maths such as `sqrt`, `range`, conversions
  such as `be.Whole`, text and lists. They behave the same in every host.
- **Everything that touches the outside is a door the host makes and names itself:** showing text, asking, files,
  time, sound, and randomness too. Seal defines none of them, not even a suggested set, so `rand`, `ask` and
  `show_msg` in Tankun's sketch are what that host chose.
- **Randomness is the host's,** so a host can control the seed, to replay a game exactly or keep two players in sync.

**Weighed.**
- Built-in showing and asking, as Unio has them. A script in a text editor or a game server has no terminal to show
  text on; each host knows what showing means.
- A suggested standard set of doors from Seal, as the design claude proposed. Tankun: hosts make their own, and they
  differ too much for one set to fit.
- Built-in randomness. A script could bring in randomness the host doesn't know about, breaking replays.

**Why this won.** It keeps Seal's design about the language itself, as Luau's is apart from Roblox's functions, and the
seal is exactly the set of doors each host chose to open.

How `.fmt` works, since in Unio it narrows the built-in `show_msg`, isn't decided yet. (Decided by D12: `fmt("…")`.)

**Tankun's words.** "Actually, we don't define host things. They make their own", then, asked "Should Seal's built-ins
be only what touches nothing outside the script, with every door, randomness included, made and named by the host, and
none defined by Seal?", the owner chose: "yea".

## D12. `fmt("…")` is a built-in that fills in braces and gives a `Text`

2026-11-23.

**Chosen.** `fmt("…")` fills in the names in its braces and gives back a `Text`, usable anywhere a `Text` is, with any
host's door:

```
mut msg = fmt("Hello, {player}")
show_msg(fmt("Correct! You got it in {tries} tries"))
show_msg("{tries}")                    // plain text, shown as written
```

- **Braces hold only names,** never a calculation or a call, and a name that doesn't exist is refused when the script
  loads (Unio D29).
  (Changed by D28: a name may be followed by its fields and facts.)
- **Text is plain unless it goes through `fmt`.**
- **It's a built-in under D11,** since it touches nothing outside the script, so no host has to do anything for it.

**Weighed.**
- Unio's `.fmt` narrowing on `show_msg` (Unio D29). In Seal `show_msg` is the host's door, which Seal can't add to.
- `.fmt` as Seal's narrowing on any procedure whose first value is text, as the design claude proposed. It works, but
  bolts a Seal word onto every host's doors.
- Every text filling in its braces, as Swift, Kotlin and Ruby do, which the design claude proposed first. A literal
  `{` would always need `\{`, and text would stop being plain by default.
- Cyborg's `@fmt`, which gives a "format value" that becomes text where it's used, because Cyborg makes every memory
  allocation visible. Seal doesn't, so `fmt` gives a plain `Text`.

**Why this won.** One built-in makes text with values in it, anywhere, for every host alike. Tankun proposed it, after
Cyborg's.

**Tankun's words.** "Hm... If Seal itself can make narrowings, the host should be able", Tankun's message proposing
`fmt` like Cyborg's (has code), then, asked "Should Seal fill text with a built-in `fmt("…")` that gives a `Text`,
with braces holding only names, checked at load time?", the owner chose: "yea".

## D13. A host can give its doors narrowings, working exactly like Seal's own

2026-11-23.

**Chosen.**
- **A host can declare narrowings for its doors,** along with the door, in the code that opens it.
- **To a script they work exactly like Seal's own narrowings,** such as `be.Whole.floor`, under the same rules: an
  unknown one is refused when the script loads, with the door's real narrowings listed (Unio D102).
- **The language server shows and colours them** from what the host declared (D7).

```
show_msg.same_line("Lives: 3, ")
play_sound.loud("boom.wav")
```

Whether a script can give its own procedures narrowings isn't decided yet, nor is it in Unio.

**Weighed.**
- Doors with no narrowings, so a host writes `show_msg_same_line`. Doors would be second-class next to Seal's own
  words.

**Why this won.** If Seal's own words can have narrowings, a host's doors should too, so a script reads the same
whichever made the word. Tankun raised it.

**Tankun's words.** "Hm... If Seal itself can make narrowings, the host should be able", then, asked "Should a host
be able to give its doors narrowings, declared with the door and working exactly like Seal's own?", the owner chose:
"yea".

## D14. The host sets a step budget and a memory cap for each script

2026-11-23.

**Chosen.**
- **A step budget:** the interpreter counts the work a script does, and a call into the script that runs past the
  budget the host set stops there. Counting steps, not seconds, makes the stop land at the same place on every machine.
- **A memory cap:** a script that would hold more than the host allows stops the same way.
- **The host gets control back, told why,** as a failure such as "out of steps" or "out of memory", and decides what
  happens next, such as reloading the script or switching that enemy off.
- **A stopped script never crashes the host.**

```
while true          // stops when the step budget runs out, and the host carries on
end while
```

What a step is, the limits' defaults, and how the host is told are the building session's to propose and Tankun's to
decide.

**Weighed.**
- No limits, relying on the doors alone. A script could still freeze the host with a loop, or eat its memory.
- A time limit in seconds. A script would stop at different places on fast and slow machines, so a game couldn't
  replay exactly.

**Why this won.** The doors keep a script from reaching out; the limits keep it from hurting its host from inside.
Luau and Wasmtime, a sandbox for WebAssembly, do the same.

**Tankun's words.** Asked "Should the host set a step budget and a memory cap for each script, with a script that
passes either stopped and the host told, never crashed?", the owner chose: "yea".

## D15. Unio's memory model: every variable owns its value, with no garbage collector

2026-11-23.

**Chosen.** Seal takes Unio's memory model (Unio D130):
- **Every variable owns its own value.** After `b = a`, changing `b` never touches `a`, lists included.
- **No reference can be kept,** so nothing dangles, nothing leaks, and no cycles can form.
- **No garbage collector.** The interpreter counts the holders of each big value, copies one only when it's written
  while shared (copy-on-write), and frees it the moment its last holder ends, so there are no pauses mid-frame.
- **Values cross the doors as copies,** so neither the host nor the script can change the other's value behind its
  back.
- **Things shared with the host, such as a game's enemies, are held by ID,** and changed through the host's doors, as
  in `damage(enemy_id, 10)`, keeping the host in charge of its own objects.

**Weighed.**
- A garbage collector, as Lua and JavaScript have. Pauses a host can't time, and needed only because references can
  form cycles.
- Shared references between script and host. A script could change a host's object behind its back, or keep it alive
  after the host ended it.

**Why this won.** It's the model Unio chose for its simplicity and safety, and it suits a sandbox even better: with no
cycles, counting holders is all an interpreter needs, and copies at the doors keep the seal tight.

**Tankun's words.** Asked "Should Seal use Unio's memory model, with every variable owning its value, no kept
references, no garbage collector, and values crossing the doors as copies?", the owner chose: "yea".

## D16. Seal's types are Unio's, with Unio's rules for them

2026-11-23.

**Chosen.** Seal's types are Unio's, with Unio's rules for them:
- **`Whole`,** a 64-bit integer that stops with a failed check on overflow (Unio D15, D46).
- **`Float`,** IEEE 754 binary64, with `~` on inexact literals and Unio's rules for showing, rounding and comparing
  (Unio D155, D157, D175 to D184).
- **`Bool`** and **`Text`.**
- **`List<T>`** and **`Array<T>[len: n]`** (Unio D167).
- **No quiet mixing of `Whole` and `Float`** (Unio D162), and **a condition must be a `Bool`** (Unio D187).

At the doors they're Rust's `i64`, `f64`, `bool`, `String` and `Vec`. Records, maps and other types come later, as
Seal or Unio needs them.

**Weighed.**
- Types of Seal's own. Nothing in Seal's purpose asks for different ones, and sharing them keeps a Unio programmer at
  home in Seal.

**Why this won.** Unio's types are already designed and tested, and map one for one onto the Rust types a host passes
through its doors.

**Tankun's words.** Asked "Should Seal's types be Unio's (`Whole`, `Float`, `Bool`, `Text`, `List` and `Array`), with
Unio's rules for them?", the owner chose: "yea".

## D17. No build modes: every script loads and runs one way, with every check kept

2026-11-23.

**Chosen.**
- **Seal has no build modes,** no `--working` or `--final`: a script loads and runs one way in every host.
- **Every check is always on,** at load time (types, names, refusals) and while running (overflow, positions, division
  by zero, floats, and D14's limits). A failed load-time check means the script doesn't load, and the host gets the
  messages; a failed running check stops the script at its exact line, and the host is told.
- **Load-time optimization is always on,** and a check is removed only where it's proved unable to fail, never
  dropped for speed (as in Unio D46 and D124): a `for each` position, maths on plain numbers, a counter over a
  `range`. Which optimizations, and when, is the building session's to propose, arriving with the part they apply to.
- **What a host chooses is the script's limits,** its doors, steps and memory (D11, D13, D14), never whether checking
  happens.
- **To be revisited only if the building session shows a real need with numbers,** such as slow loading for big
  scripts.

**Weighed.**
- Unio's `working` and `final` (Unio D46). They exist for a compiler's trade-off between fast compiling and fast
  running; Seal compiles at load time, with no slow optimizing pass to switch off.
- A mode that drops checks for speed. Checks are the sandbox's walls.

**Why this won.** A script behaves exactly the same in the host developer's tests and in a player's game, and with no
modes there's nowhere for a check to be dropped. Lua and Luau have none either. Adding a mode later is easy; taking one
away after hosts rely on it isn't.

**Tankun's words.** "This is up to the impl-tor. But we have to decide build modes first", "Idk, man.", "So,
`--Working ` and `--Final`?", "So, what would it have?", "Where's the checks?", "So, would the checks be optimized?",
then, asked "Should Seal start with no build modes, with checks removed only where proved unable to fail, to be
revisited only if the building session shows a real need with numbers?", the owner chose: "yea, but how do you check?
The optimized way or the other way?"

## D18. Floats are checked after every operation, so every failed check names its exact line

2026-11-23.

**Chosen.** Seal checks each float operation as it happens, so a float that would become infinite or NaN (Unio D177)
stops the script at its exact line, with no exception like Unio's `final` (Unio D46).

**Weighed.**
- Cyborg's sticky-flag check at points, which `Unlo\n`'s `final` uses (Unio D46, Cyborg D440). It matters in compiled
  code, where a float `+` is one CPU instruction; in a VM each instruction already costs many, so one comparison more
  is a few percent at most. Rust also doesn't expose the CPU's flags without unsafe code per platform, which a sandbox
  should avoid.
- No float checks at all, as Luau, Lua and JavaScript have it: `0 / 0` becomes NaN and spreads silently. Seal doesn't
  need silence to keep the host running, since a stopped script never crashes it (D14).

**Why this won.** Exact lines in every message for almost nothing, paid for many times over by static types, which
spare Seal's VM the type checks a dynamic VM makes on every operation.

**Tankun's words.** "yea, but how do you check? The optimized way or the other way?" (quoted in D17), "What does Luau
do?", "What the fuck did I just hear", "Would our static help with speed?", then, asked "Should Seal check floats after
every operation, so every failed check names its exact line?", the owner chose: "yea".

## D19. Seal compares what it shares with Unio against Ralph, as a black box

2026-11-23.

**Chosen.**
- **What Seal shares with Unio is checked against Ralph,** Unio's proven oracle: values, `Whole` and `Float` maths,
  overflow, float printing and rounding, comparisons and lists (D15, D16).
- **A small translator turns a Seal script into the same Unio program,** adding `$`, `let` or `var`, `;` and `:`, and
  wrapping it in `proc.main`, with a test host whose doors `show_msg` and `ask` become Unio's built-ins.
- **The fuzzer feeds both:** random scripts using the shared parts run on Seal's VM and, translated, on Ralph. A
  difference is a bug in one of them, and the decisions say which.
- **Unio's tests of shared meaning get Seal versions,** with the same expected output.
- **Ralph stays a black box:** Seal's building session runs Ralph's program but never reads `oracle/` in Unio's
  repository, so their bugs can't agree (as Unio's Building rule 12).

What's Seal's alone, such as type inference, doors and limits, is covered by Seal's own tests and the fuzzer.

**Weighed.**
- Seal tested on its own. Its interpreter has no proofs, and the shared meaning, above all float printing, is where
  subtle bugs hide.
- Reusing Ralph's code in Seal. OCaml inside a Rust sandbox, and a copied bug would agree with itself.

**Why this won.** It lends Seal some of Ralph's proven certainty for the price of a translator, the same idea as
Csmith, which found hundreds of bugs in GCC and LLVM by comparing compilers.

**Tankun's words.** "Should Seal compare behaviour with Ralph?", then, asked "Should Seal compare its shared behaviour
with Ralph's, as a black box, through a translator and the fuzzer?", the owner chose: "yea".

## D20. Every block closes with a plain `end`

2026-11-23.

**Chosen.** Every block in Seal, `if`, `while`, a procedure and the rest, closes with a plain `end`, at the depth of the
line that opened it. An `end` at the wrong depth is refused, and a missing one is reported naming the block it should
close, as in "the `while` on line 3 has no `end`".

```
proc on_hit[points]
    if points > 10
        play_sound("crit.wav")
    end
    score = score + points
end
```

**Weighed.**
- Unio's `end` with what it closes, `end while` and `end proc on_hit` (Unio D8). It reads well on paper, without an
  editor; in Seal, required indentation already says which block a line belongs to, and the language server shows
  which block an `end` closes (D7).
- `end proc` with no name, as the design claude first proposed. It breaks Unio's rule that a block closes with what it
  closes, which for a procedure includes its name, as Tankun pointed out.
- No closer at all, as Python has it. A visible bottom keeps a block's end clear, and survives text pasted with its
  indentation lost.

**Why this won.** Seal's indentation and editors already carry what `end while` would repeat, so one short word is
enough, as in Lua and Luau, which most game scripters know. It changes Tankun's sketch's `end if` and `end while`.

**Tankun's words.** "`ebd proc`? The fuck", "Wait, wait. Doesn't Unio use `end while` too? Because it doesn't have a
name, but `proc` does", "What if we went just `end`?", then, asked "Should every block in Seal close with a plain
`end`?", the owner chose: "yea".

## D21. A procedure's first line is `proc name[params]`, with optional types and `returns`

2026-11-23.

**Chosen.** A procedure starts with `proc`, its name, and its parameters in `[ ]`, as in Unio (Unio D165, D166). Each
parameter's type is optional (D5), `returns` with the result type is optional (D6), the line has no `:` (D9), and the
procedure closes with `end` (D20).

```
proc on_hit[points]
    score = score + points
end

proc average[a: Float, b: Float] returns Float
    return (a + b) / 2.0
end
```

**Weighed.**
- A shape of Seal's own, such as `fn` or `def`. Unio's `proc` and `[ ]` already fit, and keep a Unio programmer at
  home.

**Why this won.** It's Unio's procedure line with only what Seal's earlier decisions removed or made optional.

**Tankun's words.** Asked "Should a Seal procedure's first line be `proc name[params]`, with optional types and an
optional `returns`?", the owner chose: "yea".

## D22. Seven groups of Unio's features come into Seal as they are, in Seal's syntax

2026-11-23.

**Chosen.** These come into Seal as Unio has them, changed only by what Seal's own decisions change (no `$`, `;` or
`:`, and a plain `end`):
1. **Comparisons and logic:** `==`, `!=`, `<`, `>`, `at_least`, `at_most`, `and`, `or`, `not` (Unio D17, D34).
2. **Loops:** `while`, `break`, `for each … in` with `at pos` and `at offset`, `range[from:, to: or before:,
   step_by:]`, and no changing what a `for each` goes through (Unio D19, D32, D170 to D174).
3. **Lists:** `[ ]` literals, `pos:` and `offset:`, `.add`, `.remove`, `.len`, `len.max` with its failure `full`, and
   `[all: value]` (Unio D75 to D80, D167 to D169).
4. **Failures:** `?`, `??` with a value, `ignore` or `crash`, and `otherwise` with `case`, `instead` and a case that
   just ends after a call that gives back nothing (Unio D23, D40, D186).
5. **Conversions:** `be.Whole`, `be.Float`, `be.Text`, with `be.Whole.floor` and the rest (Unio D22, D153, D178).
6. **Calls:** labels, as in `heal(50, by: 10)`, and slides `|>` for chains of three or more (Unio D24, D128).
7. **Maths:** `+`, `-`, `*`, `/`, `div.floor`, `div.trunc`, `rem` and `sqrt` (Unio D27, D70, D71, D185).

```
for each name at pos i in names
    show_msg(fmt("{i}. {name}"))
end
```

A later Unio decision changing one of these is brought to Tankun for Seal too; it doesn't change Seal by itself (D1).

**Weighed.**
- A question for each feature. Slower, for features that already fit Seal unchanged.

**Why this won.** They're designed, tested and run in Ralph, and nothing in Seal's purpose asks for them to differ.

**Tankun's words.** Asked "Should Seal adopt these seven groups from Unio, with Seal's syntax changes applied?", the
owner chose: "Yea. How the fuck does Luau shorten Workspace, should we have that?"

## D23. No host-given values: a script gets what it needs from the host through doors

2026-11-23.

**Chosen.** A host gives a script doors only (D11), not ready-made named values. A script that needs a host's object or
value gets it through a door, once, at its top level:

```
ws = get_service("Workspace")
level = level_name()
```

The name it makes is read-only unless made with `mut` (D2), and an object arrives as an ID (D15).

**Weighed.**
- Named, read-only values from the host, as Roblox gives every script `workspace`, `game` and `script`, which the
  design claude first proposed. They save one line per script, for one more concept with rules of its own: read-only,
  clashes with the script's names, and how the language server shows them.

**Why this won.** A door already does it in one line, as Roblox's own `game:GetService("Workspace")` does. If many
hosts come to want ready-made names, they can be added later without breaking anything.

**Tankun's words.** "That's the same as doing `local ws = game:GetService("Workspace")`, right?", then, asked "Should
Seal leave out host-given values, with doors covering them?", the owner chose: "yea. What about Seal naming
conventions? `GetService` or `get_service`".

## D24. Names are lowercase words joined by `_`, the host's doors included

2026-11-23.

**Chosen.** Seal names follow Unio's D99: lowercase English letters, digits and `_` between words, starting with a
letter, as in `get_service`, `player2` and `hp_max`. Types are capitalised, as `Whole` and `List`, so a capital always
means a type. This holds for the host's doors and their narrowings too (D11, D13): a door named `GetService` is refused
when the host opens it, in the host's own code, so a script reads the same in every host.

**Weighed.**
- Each host naming its doors in its own style. Roblox's API mixes `GetService`, `FindFirstChild`, `workspace` and
  `game.Players`, and Luau can't stop it, since the names are Roblox's.
- `GetService`, capitalised as Roblox has it. A capital would then mean a type in one place and a door in another.

**Why this won.** One style everywhere, the one Unio already has, and since Seal's hosts name their own doors (D11),
checking them is the only way to keep scripts uniform.

**Tankun's words.** "yea. What about Seal naming conventions? `GetService` or `get_service`" (quoted in D23), then,
asked "Should Seal names, the host's door names included, follow Unio's D99: lowercase words joined by `_`, with types
capitalised?", the owner chose: "yea".

## D25. Seal scripts use the `.seal` extension

2026-11-23.

**Chosen.** A Seal script is a file ending in `.seal`, as in `goblin.seal`, matching Unio's `.unio`. Tools and the
language server recognise Seal files by it.

**Weighed.**
- `.sl` or `.se`. Shorter, but `.sl` is already used by other tools, and neither says "Seal".

**Why this won.** It's the language's own name, short, and not used by any well-known format. Tankun's first sketch
was already `guessing_game.seal`.

**Tankun's words.** Asked "Should Seal scripts use the `.seal` extension?", the owner chose: "yea".

## D26. Every message goes to the host as Unio-style text and as its separate parts

2026-11-23.

**Chosen.** A message, from a script refused at load time or stopped while running, goes to the host two ways:
- **as ready-made text,** in Unio's layout (Unio D54), which a host can show as it is;
- **as its parts:** its kind (`refused` or `check failed`), file, line, column, and its `what`, `when`, `fix` and
  `why`, so a host can show them its own way, such as underlining the word in its editor.

The wording follows Unio's rules: plain sentences, no error codes, no "did you mean" guesses (Unio D54).

```
refused: goblin.seal:4:5, line 4 column 5
 4 |     tires = tries + 1
   |     ^^^^^
 what:    `tires` is never used
```

**Weighed.**
- Text only, as Unio prints it. A host showing messages in its own editor would have to take the text apart again.
- Parts only. Every host would have to lay messages out itself, and they'd differ.

**Why this won.** A Seal message's reader is the host, which may print it, log it or show it in an editor; both forms
cost little and suit each.

**Tankun's words.** Asked "Should Seal give every message to the host both as Unio-style text and as its separate
parts?", the owner chose: "yea".

## D27. Records: declared with `record Name`, built with `Name[field: value]`, read with `.field`

2026-11-23.

**Chosen.**
- **`record Name` declares a record,** its fields one per line, each with its type written, closed by `end` (D20). The
  fields are its contract, as a procedure's line is.
- **It's built with `[ ]`, naming every field:** `Enemy[name: "Goblin", hp: 30, x: 0.0, y: 0.0]`, the shape labels
  and settings already have (Unio D24, D165, D167, D170). A field left out is refused, so adding a field later makes
  every place that builds one say its value.
- **A field is read with `.field`,** and changed only through a `mut` name. A record is a value, so `b = a` gives `b`
  its own copy (D15).
- **At the doors a record is a Rust struct,** so a host can pass whole records in and out (D15, D16).

```
record Enemy
    name: Text
    hp: Whole
    x: Float
    y: Float
end

mut goblin = Enemy[name: "Goblin", hp: 30, x: 0.0, y: 0.0]
goblin.hp = goblin.hp - 5
```

Defaults for fields, methods on records, records inside records, and `fmt` braces holding `name.field`, which D12
doesn't allow yet, aren't decided.

**Weighed.**
- `struct`, as C, Rust, Swift and Go have it: short for "structure", jargon to a beginner. `class` and `object`: they
  promise methods, inheritance and shared references, which Seal doesn't have. `type`: too general.
- Building by position, as `Enemy["Goblin", 30, 0.0, 0.0]`: which number is which can't be seen, and reordering the
  fields silently breaks every place a record is built.
- `Enemy[name = "Goblin"]`, as Python and Kotlin have it: in Seal `=` makes or changes a name (D2).
- `Enemy{name: "Goblin"}`, as Rust and JavaScript have it: in Seal braces fill in text (D12).

**Why this won.** "Record" is the standard technical name for exactly this, a fixed set of named, typed fields, from
the office record card of facts about one thing, and it says what D15 makes it: values written down together and
copied as a whole. Unio has no records yet, only the word in passing in D165, so Seal's choice is likely to become
Unio's.

**Tankun's words.** "Why `record`?", "Unio doesn't have records yet", "Is it accurate?", "What does record mean in
Englusg", "Why `field: value`?", then, asked "Should Seal have records, declared with `record Name`, built with
`Name[field: value]`, and read with `.field`?", the owner chose: "yea".

## D28. `fmt` braces may hold a name followed by its fields and facts

2026-11-23.

**Chosen.** Inside `fmt`'s braces, a name may be followed by its fields (D27) and facts, such as `.len` (Unio D79):
`{goblin.name}`, `{names.len}`. A calculation or a call is still refused, and a field or fact the value doesn't have is
refused at load time.

```
show_msg(fmt("{goblin.name} has {goblin.hp} HP"))
show_msg(fmt("{names.len} names"))
show_msg(fmt("{goblin.hp - 5}"))        // refused: a calculation
```

This changes D12's "braces hold only names".

**Weighed.**
- Names only, as D12 had it, so a field needs a name of its own first. A field or a fact is read directly, running
  nothing and unable to fail, so D12's reason doesn't apply to it.

**Why this won.** Text still never hides work or something that can fail, which is what D12 and Unio's D29 guard
against, while records show cleanly.

**Tankun's words.** Asked "Should `fmt` braces allow a name followed by its fields and facts, like `{goblin.name}` and
`{names.len}`?", the owner chose: "yea. Should Seal be able to join text with `&`?"

## D29. `&` turns a `Whole` or a `Bool` into text by itself; a `Float`, a list or a record is still refused

2026-11-23.

**Chosen.** When `&` joins text with a `Whole` or a `Bool`, it turns that value into text by itself: `42` becomes
`"42"`, and a `Bool` becomes `"true"` or `"false"` (Unio D153). A `Float` joined with `&` is still refused, with
`be.Text(…)` as the fix, and so are lists and records, which have no text form yet.

```
show_msg("Score: " & score)          // "Score: 42"
show_msg("Alive: " & alive)          // "Alive: true"
show_msg("Speed: " & speed)          // refused, fix: be.Text(speed)
```

This changes, for Seal, the rule D16 brought in from Unio's D28, which refuses every number joined with `&`.

**Weighed.**
- Unio's rule, refusing every number with `be.Text(…)` as the fix, so the step into text is always visible. For a
  `Whole` or a `Bool` there's only one text it can become, so there's nothing to see.
- Turning a `Float` into text by itself too. Its text follows Unio's D155, with the `~` and every digit, as in
  `~0.30000000000000004`, which a player shouldn't see by accident.

**Why this won.** Seal is meant to be easier to write, and only the conversions that can't surprise anyone happen by
themselves.

**Tankun's words.** "I think `&` should auto-convert where good", then, asked "Should `&` in Seal turn a `Whole` or a
`Bool` into text by itself, while a `Float`, a list or a record is still refused?", the owner chose: "yea".

## D30. A name's type is written `name: Type = value`, required only when the value can't say it

2026-11-23.

**Chosen.** A name's type may be written after it, as `name: Type = value`, the shape a parameter's type has (D21). It's
optional everywhere, and required only when the value can't say the type, as with an empty list; that is refused with a
message saying the type has to be written. It reads the same after `mut` or `unused`.

```
mut bag = []                            // refused: a list of what?
mut bag: List<Text> = []
mut bag: List<Text>[len.max: 10] = []
lives: Whole = 3                        // allowed, though 3 already says it
```

**Weighed.**
- Types never written on names, only on parameters. An empty list, or a list with a limit, couldn't be made.
- Writing the type before the name, as `List<Text> bag = []`, as C and Java do. Unlike every other type Seal writes,
  which comes after a name and `:`.

**Why this won.** It's the shape Unio writes and Seal's parameters already use, written only where it's needed.

**Tankun's words.** Asked "Should a name's type be written `name: Type = value`, optional, and required only when the
value can't say it?", the owner chose: "yea".

## D31. A type with more than one blank to fill in labels each one

2026-11-23.

**Chosen.** Inside `< >`, a type with more than one blank to fill in labels each one, as `Map<key: Text, value:
Whole>`. A type with one blank, as `List<Text>`, leaves it unlabelled, since nothing can be mixed up (Unio D24's
convention for a lone first argument).

**Weighed.**
- By position, as `Map<Text, Whole>` in Rust, Swift and Java: the reader has to know which blank comes first, the same
  problem D27 refused for building a record by position.

**Why this won.** Wherever two things could be mixed up, each says what it is, as labels do in calls (Unio D24) and
field names do in records (D27). It adds labels to `< >`, which Unio's D165 has only as "a type, filled in".

**Tankun's words.** "Where's the labels?", "No, I meant the label on the type", then, asked "Should a type with more
than one blank to fill in label each one, as in `Map<key: Text, value: Whole>`, while `List<Text>` stays unlabelled?",
the owner chose: "yea".

## D32. `Dict<key: …, value: …>` pairs keys with values, reached with `key:`, with a missing key as a failure

2026-11-23.

**Chosen.**
- **`Dict<key: Key, value: Value>` holds values looked up by key,** each key once (D31's labels). `Dict` is taught as
  short for dictionary, as Unio teaches `proc` and `fmt` (Unio D2).
- **It's written with its keys as values:** `["sword": 1, "potion": 3]`, and `[:]` is an empty one.
- **It's reached with `key:`,** as a list is with `pos:` and `offset:` (Unio D75): `stock[key: "potion"]`. Setting
  `stock[key: "potion"] = 4` replaces the value, or adds the key, through a `mut` name.
- **A missing key is a failure, `missing`,** handled with `??` or `otherwise` (Unio D23), never a quiet "nothing".
- **It keeps the order keys were added in,** so going through one is the same on every machine.
- **It's a value,** copied as a whole (D15), and a Rust `HashMap` at the doors.

```
mut stock: Dict<key: Text, value: Whole> = ["sword": 1, "potion": 3]
stock[key: "potion"] = 4
potions = stock[key: "potion"] ?? 0
```

Going through a `Dict` with `for each`, its methods, such as removing a key or asking whether one is there, and which
types can be keys, aren't decided yet.

**Weighed.**
- `Map`, as Java, Go, C++, Kotlin and JavaScript have it, from maths' mapping. In games, a map is the level or world
  map, one of the likeliest record names a script would want.
- `Dictionary`, as Swift and C# have it. The same word in full, longer where it's typed, as `Bool` is short for
  Boolean.
- `HashMap`, as Rust has it: names how it's stored inside. `Table`, as Lua has it: Lua's tables are its lists and
  objects too.
- Arbitrary order, as a plain hash map gives. A game going through one would differ from machine to machine.
- Luau's `nil` for a missing key. The failure comes later, far from the lookup.

**Why this won.** It works as a paper dictionary does: look up by the word, not by position, each word once, and a word
that isn't there isn't there. The name comes from Smalltalk's `Dictionary`, shortened as Python's `dict`.

**Tankun's words.** "Why `Map`?", "Why `Dict`?", "No, I meant why `Dict`", then, asked "Should Seal's key-to-value
type be `Dict<key: …, value: …>`, taught as short for dictionary, with the rest as proposed?", the owner chose: "yea".
