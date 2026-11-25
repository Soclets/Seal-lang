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

(Replaced by D81: a procedure's header writes its types.)

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

(Changed by D45: written `-> Type`.)

**Tankun's words.** Asked "Should a procedure's result type be optional too, worked out from its `return`s, with
`returns` allowed for writing it?", the owner chose: "yea".

(Replaced by D81: a procedure's header writes its types.)

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

(Corrected by D40: Unio's D16 turned `elif` down as breaking D2, so this entry's reason is wrong.)

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

(Replaced by D75: every text fills its braces, and `fmt` goes.)

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

(Changed by D76: a block closes with `end` and what it closes.)

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

(Changed by D45: `-> Type` in place of `returns`.)

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

(Replaced by D77: `&` goes, and text is joined only with braces.)

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

## D33. `for each` goes through a `Dict`'s values, with `at key name` giving each key

2026-11-23.

**Chosen.** `for each` goes through a `Dict`'s values in the order their keys were added (D32), and `at key name` gives
each value's key, as `at pos` and `at offset` give a list item's position (Unio D174). Like a list, the `Dict` can't
change inside the loop (Unio D173).

```
for each count at key item in stock
    show_msg(fmt("{item}: {count}"))
end

for each count in stock
    total = total + count
end
```

**Weighed.**
- `for item, count in stock`, as Python has it. Shorter, but which name is the key and which the value is by position,
  the problem D27 and D31 removed.

**Why this won.** One shape for going through anything, and it reads aloud as a sentence: "for each count, at key item,
in stock".

**Tankun's words.** "`for each count at key item in stock`
Holy shit, that's an entire sentence", then, asked "Should `for each` go through a `Dict`'s
values, with `at key name` giving each key, in the order they were added?", the owner chose: "yea".

## D34. A `Dict` has `.remove(key:)`, `.has(key:)` and `.len`, with `Whole`, `Text` or `Bool` keys

2026-11-23.

**Chosen.**
- **`.remove(key: k)`** takes a key out, through a `mut` name, and fails with `missing` if it isn't there, as reading
  one does (D32).
- **`.has(key: k)`** gives a `Bool`: whether the key is there.
- **`.len`** is the number of keys, a fact read without running anything, as for lists (Unio D79).
- **Keys can be `Whole`, `Text` or `Bool`,** types whose equality is exact and unsurprising. A `Float` key is refused,
  since two neighbouring floats that print alike are different keys (Unio D184).

```
stock.remove(key: "sword")
if stock.has(key: "potion")
    show_msg(fmt("{stock.len} kinds"))
end
```

Lists and records as keys, and other methods, aren't decided yet.

**Weighed.**
- `Float` keys, as Python allows. `~0.1 + ~0.2` and `~0.3` would be two keys, the classic trap.
- Removing a missing key quietly. Unlike Seal's reading of one, and a typo in the key would go unnoticed.

**Why this won.** The methods take `key:` as lookups do, so a `Dict` reads alike everywhere, and keys stay to the types
where "the same key" can't surprise anyone.

**Tankun's words.** "Don't you have a tool for that?", then, asked "Should a `Dict` have `.remove(key:)`, `.has(key:)`
and `.len`, with keys of `Whole`, `Text` or `Bool` only?", the owner chose: "yea".

## D35. One construct for options, plain ones and ones carrying values, with its cases as narrowings

2026-11-23.

**Chosen.**
- **One construct declares a type whose value is exactly one of several cases,** listed one per line and closed with
  `end` (D20). Its keyword isn't decided yet; `enum` stands in for it below. (Decided by D36: `one_of`.)
- **A case may carry values,** named in `[ ]`, as `hurt[amount: Whole]`, built as a record is (D27). A case with none
  is a plain option.
- **A case is written as a narrowing of its type:** `Event.hurt[amount: 5]`, "a hurt kind of Event", and `State.idle`
  (Unio D41). A case the type doesn't have is refused at load time.
- **It's a value** (D15), and a Rust enum at the doors.

```
enum Event
    idle
    hurt[amount: Whole]
    healed[amount: Whole, by: Text]
end

mut last = Event.hurt[amount: 5]
```

Reading which case a value is, and its values, needs a construct of its own, such as a `match`, not decided yet.

**Weighed.**
- A plain list of options now, and a second construct for cases carrying values later. Two constructs for one idea;
  Cyborg covers both with one, its tagged union (Cyborg D33).
- `Event::hurt`, as Rust has it: `::` names a package in Unio (Unio D64). `.hurt`, as Swift has it: which type it
  belongs to is invisible. A bare `hurt`: could be any name.
- Text for options, as `state = "chasing"`: a typo like `"chasnig"` never matches, silently.

**Why this won.** One construct for one idea, with nothing new to learn: its cases are narrowings and its values are
built as records are, and a typo in a case is caught at load time.

**Tankun's words.** "That's an `enum`, right?", "The fuck?", "`one_of`?", "Cyborg uses tagged unions for more complex ones", "Its cases as
narrowings?", "What does Enumeration mean in maths", "Does Seal have `match`?", then, asked "Should Seal have one
construct for options, plain ones and ones carrying values in `[ ]`, with its cases as narrowings, as Cyborg's tagged
union does?", the owner chose: "yea".

## D36. Options are declared with `one_of`

2026-11-23.

**Chosen.** D35's construct is declared with `one_of`: "an Event is one of these".

```
one_of Event
    idle
    hurt[amount: Whole]
    healed[amount: Whole, by: Text]
end
```

**Weighed.**
- `enum`, short for enumeration, as C, Rust, Swift and Roblox's `Enum` have it. In maths, an enumeration counts a set
  out member by member, which is exact for plain options but not for cases carrying values, such as
  `Event.hurt[amount: 5]`, which can't be counted out. Rust and Swift stretched the word.
- `union.tagged`, as Cyborg has it (Cyborg D33). Precise, but jargon twice over for Seal.
- `choice`. Plain, but no more accurate than `one_of`, and it says less.

**Why this won.** It's exact for the whole construct, plain and in the style of `at_least`, and Protocol Buffers'
`oneof` means the same thing. The cost is one new word for scripters who know `Enum`.

**Tankun's words.** "`one_of`?" (quoted in D35), then, asked "Should the keyword be `one_of`?", the owner chose:
"yea".

## D37. `match` reads which case a value is, covering every case

2026-11-23.

**Chosen.**
- **`match value` opens a block with a `case` for each of the value's cases** (D35, D36), closed by one `end` (D20).
  No `:` on either line (D9).
- **Every case must be covered,** as `otherwise` requires for failures (Unio D40), so a case added later makes every
  `match` that misses it refused at load time. There's no catch-all.
- **A case's values come out by name,** as `case hurt[amount]`, as read-only names for that case's body.
- **A case runs until the next `case`,** like `elif` and `else` inside one `if`, and one `end` closes the whole block.
  The same holds for the cases of `otherwise`, which chapter 7 of the standard left open.

```
match last
    case hurt[amount]
        hp = hp - amount
    case healed[amount, by]
        hp = hp + amount
        show_msg(fmt("{by} healed you"))
    case idle
end
```

**Weighed.**
- A catch-all for every other case, as Rust's `_` and Swift's `default`. It would quietly swallow a case added later,
  which is what covering every case is there to catch.
- An `end` for each case. Unlike `if`, whose `elif` and `else` share one `end`.
- A plainer word than `match`, as Unio's D40 chose `otherwise` over `match` for failures. `otherwise` follows a call
  and reads as English there; `match` starts a line about a value, as in Cyborg, Rust and Python.

**Why this won.** It reuses `case` and "cover every case" from `otherwise`, so options and failures are handled alike,
and a new case can't be forgotten anywhere.

**Tankun's words.** Asked "Should Seal have `match`, with a `case` for every case, values taken out by name, and one
`end` closing it, the same for `otherwise`'s cases?", the owner chose: "yea".

## D38. No `nil`: "might not be there" is a built-in `Maybe<T>`

2026-11-23.

**Chosen.**
- **Seal has no `nil`, `null` or `None`.** A name of a type always holds a real value of that type.
- **`Maybe<T>` is built in,** a `one_of` (D35, D36) with two cases: `none`, and `some[value: T]`. It's read with
  `match`, which must cover both (D37), and `??` gives a fallback when it's `none`, as for a failure (Unio D23).
- **At the doors it's Rust's `Option<T>`,** so a door like `nearest_enemy()` can say it found nothing.

```
target = nearest_enemy()
match target
    case some[value]
        attack(value)
    case none
end
id = nearest_enemy() ?? 0
```

**Weighed.**
- `nil`, as Luau has it. It sneaks into every type, and the crash comes later, elsewhere; Tony Hoare, who invented
  null, called it his "billion-dollar mistake".
- No built-in, each script or host writing its own `one_of`, as the design claude first proposed. Every host would
  invent its own for Rust's `Option`, each named differently.
- `Option`, as Rust has it: Seal already calls `one_of`'s cases its options. `Optional`, as Swift has it: longer.
  `Nullable`, as C# has it: built on null.

**Why this won.** Nothing is ever missing silently, "might not be there" is said in the type, and the host's
`Option<T>` crosses the doors one for one. `Maybe` reads as what it means, "maybe a whole number", as in Haskell and
Elm.

**Tankun's words.** "Why not?", "Why `Maybe`?", then, asked "Should Seal have no `nil`, but a built-in `Maybe<T>`, a
`one_of` with `none` and `some[value]`, usable with `match` and `??`?", the owner chose: "yea".

## D39. No contracts for now

2026-11-23.

**Chosen.** Seal has no `requires` or `ensures` for now. A check a procedure needs on its arguments is written as an
ordinary check in its body. Contracts can be added later, when a real script wants one.

**Weighed.**
- Optional contracts, as Cyborg has them in its base (Cyborg D29), as `requires` and `ensures` lines at the top of a
  body. Without a prover, each is a runtime check, a tidier way of writing one by hand; the gain is that the promise
  sits at the top, where a reader, the language server and the host can see it.
- Waiting for Unio, as the design claude first argued, "with Ralph able to prove them". Ralph proves things about the
  language, not about one program's code, and Unio plans no program verifier, so contracts in Unio would be runtime
  checks too.

**Why this won.** The gain is small without a prover, and adding a feature later is easy where taking one away isn't.
Types worked out at load time, `Maybe` in place of `nil` (D38), and `match` covering every case (D37) already catch much
of what simple contracts would.

**Tankun's words.** "Should there be contracts?", "Huh? Are they required?", "Ralph able to prove them? What does that
mean", "Are labels, families, and defaults decided in Seal?", then, asked "Should Seal leave contracts out for now?",
the owner chose: "yea".

## D40. `elif` stays, for Seal's own reasons, and D10's reason is corrected

2026-11-23.

**Chosen.** Seal keeps `elif` (D10), for its own reasons: Tankun's sketch chose it, Python and Bash scripters already
know it, and it's lighter to write. D10 said it follows Unio's D2; it doesn't. Unio's D16 turned `elif` down because
it's a made-up contraction, not short for one word, breaking D2's "learn the concept, then type it short". So this is
a deliberate difference from Unio, as Seal is inspired by Unio, not bound to it (D1).

**Weighed.**
- Unio's `else if`: two plain words, consistent with Unio, for three more characters.

**Why this won.** Seal parts from Unio on purpose wherever writing gets lighter, as with no `$` (D7), no `;` (D8) and a
plain `end` (D20), and `elif` belongs with those. The design claude found D10's wrong reason while reading Cyborg's
and Unio's decisions on contracts.

**Tankun's words.** Asked "Should Seal keep `elif`, with a new decision giving the real reason and noting that Unio
refused it?", the owner chose: "yea".

## D41. Labels are declared with Unio's `labels` line at the top of the body

2026-11-23.

**Chosen.** A procedure declares its labels as Unio does (Unio D25), on a `labels` line, the first line of its body, in
Seal's syntax: no `$` and no `;`.

```
proc heal[target, amount, times]
    labels amount as by, times as times
    hp = hp + amount * times
end

heal(goblin, by: 10, times: 2)
```

**Weighed.**
- Labels in the parameter list, as `proc heal[target, amount as by]`, as the design claude proposed, so the first line
  holds the whole promise. Tankun: "Fuck no".

**Why this won.** The parameter list stays just the names, with types where wanted, and labels keep Unio's form.

(Changed by D42: the list goes in `[ ]`.)

**Tankun's words.** "Fuck no", then, asked "Should Seal declare labels with Unio's `labels` line at the top of the
body?", the owner chose: "yea".

## D42. `labels [ ]` and `defaults [ ]`, with a default written as the call would write it

2026-11-23.

**Chosen.**
- **`labels` puts its list in `[ ]`,** as `labels [amount as by, times as times]`, changing D41's bracket-free line. A
  long list can then be split over lines inside the `[` (D8).
- **`defaults [ ]` comes right after `labels`,** as in Cyborg (Cyborg's standard, 5.4), giving a value for a labelled
  argument a call leaves out.
- **A default is written as the call would write it,** the label, `:` and the value: `defaults [times: 1]`, so
  `heal(goblin, by: 10)` means exactly `heal(goblin, by: 10, times: 1)`. With `amount as by`, it's `defaults [by: 10]`.
- **Only labelled parameters can have one,** since leaving out an unlabelled argument would make the rest ambiguous.
- **A default is a plain value,** worked out at load time, never a calculation or a call, as D12 keeps braces free of
  hidden work.
- **The host's calls can leave defaulted arguments out too.**

```
proc heal[target, amount, times]
    labels [amount as by, times as times]
    defaults [times: 1]
    hp = hp + amount * times
end

heal(goblin, by: 10)
```

**Weighed.**
- `labels` with no brackets, as Unio's D25 and D41 have it. Such a line can't be split, with no `;` or `\` in Seal.
- `defaults [times = 1]`, as Cyborg writes it. In Seal `=` makes or changes a name (D2), which a default doesn't, as
  D27 refused `=` for building records.

**Why this won.** It fits Unio's D165, since the lists describe the parameters, already in `[ ]`, and a default reads
as the very piece of the call it stands for. Tankun asked for the brackets.

**Tankun's words.** "Wait a minute, can we use `[]` there now?", "That still fits the brackets rule, right?", "Is `=`
in `defaults` corect?", then, asked "Should `labels` and `defaults` put their lists in `[ ]`, with `defaults` written as
the call would write it, `[times: 1]`, after `labels`, for labelled parameters only, with plain values?", the owner
chose: "yea".

## D43. Unio's families: `fam.proc` now, `fam.handler` once Seal has handlers

2026-11-23.

**Chosen.**
- **Seal takes Unio's families** (Unio D41), in Seal's syntax: a word and its kinds declared together in a `fam.kind`
  block, holding only that kind, with no `:` and a plain `end` (D9, D20).
- **`fam.proc` now,** so a script can give its own procedures narrowings. Each member is the word itself or reads as
  "a Y kind of X" for its name `X.Y`; anything else is refused.
- **`fam.handler` comes with handlers,** once Seal decides how a procedure lists the failures it can give (Unio D39,
  D42), since a handler covers a known set of failures.

```
fam.proc spawn
    proc spawn[kind]
        ...
    end
    proc spawn.boss[kind]
        ...
    end
end

spawn.boss("dragon")
```

where each `...` stands for that procedure's body.

**Weighed.**
- Narrowings declared anywhere, as `proc spawn.boss` beside `proc spawn` with no block, as the design claude first
  proposed. Unio's D41 weighed and refused exactly that: variants scatter, and families stop being a visible thing.
  "Short scripts" doesn't change it; a script can grow past a screen.
- `fam.proc` alone, without handlers ever. `fam` is one idea in Unio, with two kinds.

**Why this won.** Every variant of a word sits in one place, checked as a kind of it, the same in Seal as in Unio, and
the same for a host's doors and their narrowings, declared together (D13).

**Tankun's words.** "Why no `fam`?", "You do know `fam` houses handlers too, right?", then, asked "Should Seal take
Unio's families, `fam.proc` now and `fam.handler` once Seal has handlers?", the owner chose: "yea".

(Changed by D97: family members are written without `proc`, and `fam.handler` arrives.)

## D44. `fail name` fails on purpose, and a procedure's failures are worked out from its body

2026-11-23.

**Chosen.**
- **`fail name` fails on purpose,** with a plain failure name, as Cyborg writes it.
- **A procedure's failures are worked out from its body:** every `fail` in it, and every failure it passes up with `?`,
  as its types are (D5, D6), so they never need listing by hand.
- **They may be written, as `fails [not_found, broken]`, on the first line,** after `returns` if that's written, in
  `[ ]` since it describes the procedure (Unio D165). Written, they must match what the body does.
- **A door can fail too:** the host declares its failures, in Rust a `Result` with its error cases, and a script
  handles them like any other.

```
proc load_level[name]
    if not level_exists(name)
        fail not_found
    end
    return read_level(name)?
end

level = load_level("cave") ?? default_level
```

**Weighed.**
- Failures always listed, as Unio's D39 has them after `returns`. Seal already works out what the body says (D5, D6).
- `fails` at the top of the body, beside `labels` and `defaults`. Those say how to call a procedure; a failure is what
  a call can give back, so it goes with `returns`, as in Unio and Cyborg.

**Why this won.** Failing on purpose gets a plain word, and failures are handled the way types are: worked out, and
written only where wanted.

(Changed by D45: written `! [ … ]`.)

**Tankun's words.** "Why is it there?", then, asked "Should Seal fail on purpose with `fail name`, with a procedure's
failures worked out from its body, optionally written as `fails [ … ]` on the first line?", the owner chose: "yea.
Should Seal have `->` instead of `returns`?"

(Changed by D84: a procedure that can fail writes `! [ … ]` in its header.)

## D45. `-> Type` for the result and `! [ … ]` for failures, in place of `returns` and `fails`

2026-11-23.

**Chosen.** A procedure's first line writes its result as `-> Type` and its failures as `! [ … ]`, result first, both
optional and worked out from the body when left off (D6, D44). `returns` and `fails` aren't keywords in Seal.

```
proc load_level[name] -> Level ! [not_found, broken]
proc average[a: Float, b: Float] -> Float
proc save[game] ! [disk_full]
```

This changes D6, D21 and D44.

**Weighed.**
- `returns Level fails [not_found, broken]`, as Unio writes them, which the design claude recommended keeping: words
  read aloud and explain themselves, and Seal so far chooses words where they read naturally. `!` means "not" in most
  languages, so a first guess at `-> Level ! [not_found]` could be "returns not a Level".

**Why this won.** Tankun's choice, compact where both parts are written, with `->` from Rust and Swift and `!` for
failures from Zig. Since both are usually worked out, they're written only where wanted.

**Tankun's words.** "What about `proc load_level[name] -> Level ! [not_found, broken]`", "No", then,
asked "Should Seal write a procedure's result as `-> Type` and its failures as `! [ … ]`, both optional, in place of
`returns` and `fails`?", the owner chose: "yea".

## D46. Scripts are compiled at load time to bytecode, run by a register VM, with no JIT

2026-11-23.

**Chosen.**
- **At load time,** a script goes from text, to a syntax tree, to a checked tree with its types worked out, to
  bytecode: instructions for a virtual machine (D17).
- **While running,** a register VM, the kind Lua and Luau use, runs the bytecode, counting steps (D14). Since every type
  is known before running, its instructions need no type checks, as `add_whole` just adds.
- **No JIT,** turning code into machine code while running: iPhones and game consoles forbid it, and it makes the
  sandbox harder to trust.
- **The instruction set, its names and the optimizations are the building session's** to propose, under D17.

```
load:  text → syntax tree → checked, types worked out → bytecode
run:   a register VM runs the bytecode, counting steps
```

**Weighed.**
- Walking the syntax tree directly. Simpler, but often several times slower, and would need replacing.
- A JIT. Far more work, harder to trust, and forbidden where embedded scripts often run.
- A stack VM, as the JVM and CPython have. A register VM does the same work in fewer instructions, as Lua found.

**Why this won.** It's the proven shape for a fast embedded interpreter, and Seal's static types make its VM faster
than a dynamic one can be: there's no type check left to make while running.

**Tankun's words.** "We should decide arch first?  Memory and impl lang is done", then, asked "Should Seal compile
scripts at load time to bytecode run by a register VM, with no JIT?", the owner chose: "yea. Btw, the impl-tor could
decide bytecode names".

## D47. One workspace of five parts, with the library alone what hosts depend on

2026-11-23.

**Chosen.** Seal is one Rust workspace, a set of packages built together, of five parts:
1. **`seal`, the library a host embeds:** reading, checking, compiling, the VM, and the host's side.
2. **The `seal` command:** checks a script, and runs it with a test host whose stand-in doors print to and read from the
   terminal.
3. **The language server** (D7).
4. **The fuzzer** (IMPLEMENTATION.md, Building rule 6).
5. **The Unio translator,** for checking against Ralph (D19).

Hosts depend on the library alone, so nothing else ships inside their programs. The other four are built on the
library, never copying its insides, so they all agree on what Seal means. How the library is split inside is the
building session's to propose.

**Weighed.**
- One package holding everything. A host would carry the language server and the fuzzer inside its game.
- Each tool with its own copy of the reading and checking it needs. They'd come to disagree.

**Why this won.** Each kind of user gets only what it needs, and one library is the single meaning of Seal.

**Tankun's words.** Asked "Should Seal be one workspace with these five parts, the library alone being what hosts
depend on?", the owner chose: "yea".

## D48. The host's side: doors as typed Rust closures, chosen per script, and errors, never crashes

2026-11-23.

**Chosen.** The shape of embedding Seal from Rust; the exact names are the building session's:
- **A door is an ordinary Rust closure,** and Seal reads its types from the closure's signature, an `i64` being a
  `Whole` (D16), so the host never describes types twice.
- **The host chooses each script's doors and limits** when it loads it (D11, D14).
- **Loading checks everything and runs the top level** (D4, D17), and the host gets a handle to the loaded script. A
  refused script comes back as an error holding its messages (D26).
- **Calling in is checked by name and types.** A call to a procedure that doesn't exist, or with the wrong types, is
  refused; a script stopped by a failed check or a limit comes back as an error too, and the host carries on (D14).
- **Records and `one_of` cross the doors by deriving,** as `#[derive(Seal)]` on a Rust struct or enum (D27, D35).

```rust
let mut doors = Doors::new();
doors.add("show_msg", |text: String| println!("{text}"));
doors.add("damage", |id: i64, amount: i64| world.damage(id, amount));

let mut goblin_script = Script::load("goblin.seal", &source, &doors,
                                     Limits { steps: 1_000_000, memory: 16 * 1024 * 1024 })?;
goblin_script.call("on_hit", (10,))?;
```

A script's own data lives inside it, as `mut hp = 30`; the host's own data is reached by ID through doors (D15).

**Weighed.**
- Doors described by hand, their names and types listed apart from the closure. The types would be written twice, and
  could disagree.
- One set of doors for every script a host loads. A UI script would get an enemy's `damage`.

**Why this won.** A host writes plain Rust, Seal checks every crossing, and nothing a script does can crash the host.

**Tankun's words.** "Wait, isn't goblin made by Seal insdie?", then, asked "Should the host's side work this way: doors
as typed Rust closures, doors and limits chosen per script, and loading and calling returning errors, never
crashing?", the owner chose: "yea".

## D49. Scripts are fully separate and movable between threads, and pause through doors

2026-11-23.

**Chosen.**
- **Each loaded script is fully separate:** its own names, memory, limits and doors. Two scripts never share a value
  (D15), so a bug or an attack in one can't reach another, and loading a script twice gives two separate copies.
- **Scripts talk only through the host,** calling doors the host connects as it likes.
- **A script runs on one thread at a time, and can move between threads,** so a host can run many scripts at once,
  with nothing shared and no locks.
- **A door can pause a script.** Seal keeps exactly where the script was, the host's call comes back as paused, and
  the host resumes it whenever it likes, such as after two seconds of game time. Time stays the host's (D11), so `wait`
  is a door, not a built-in.
- **No parallelism inside one script for now,** unlike Unio's workers (Unio D68).

```
proc patrol[guard]
    while true
        walk_to(guard, x: 10)
        wait(seconds: 2)
        walk_to(guard, x: 0)
        wait(seconds: 2)
    end
end
```

How a door asks to pause, and how the host resumes, are the building session's to propose under D48's shape.

**Weighed.**
- No pausing. Every timed behaviour would be chopped into "what do I do this frame?" pieces, the classic mess pausing
  exists to avoid.
- A built-in `wait`. Time belongs to the host, which may run game time, real time, or none.
- Threads inside a script, as Unio's workers. Game scripts almost never need them, and a host already runs scripts in
  parallel.

**Why this won.** Game scripts get the waiting they need, as Lua's coroutines and Roblox's `task.wait` give them,
without Seal owning time, and parallelism comes for free from scripts sharing nothing.

**Tankun's words.** "Should Seal be able to be concurrent?", then, asked "Should Seal scripts be fully separate and
movable between threads, able to pause through a door and be resumed by the host, with no parallelism inside one script
for now?", the owner chose: "yea".

## D50. The same results on every machine, bit for bit, and future maths functions correctly rounded

2026-11-23.

**Chosen.**
- **A script, given the same calls and the same answers from its doors, gives the same results on every machine, bit
  for bit,** floats included. Seal's part is what happens inside the seal; recording and replaying what comes in, and
  keeping its own code deterministic, is the host's (D11).
- **What already keeps it:** step counting (D14), a `Dict`'s insertion order (D32), randomness through the host (D11),
  and `+`, `-`, `*`, `/` and `sqrt`, which IEEE 754 makes correctly rounded and so the same on every machine, with
  nothing fused or reordered (D17). The memory cap counts Seal's own sizes, not the operating system's.
- **A maths function Seal adds later, such as `sin`, must be correctly rounded:** the closest `Float` to the true answer.
  It's ported to Rust with no `unsafe`, following CORE-MATH's algorithms, and checked against Ralph, which computes the
  correctly rounded answer the slow, provable way, with extra digits and one final rounding, as Cyborg's oracle does
  (Cyborg D256), through D19's comparison.

**Weighed.**
- No promise, as Unio's D163 makes none. For Unio the cost was proofs and holding LLVM back; an interpreter does each
  float operation one at a time, so for Seal it's nearly free.
- Stability through one pure-Rust maths library, not correctly rounded. Enough for the same bits on every machine, as
  Cyborg's design session confirmed, but an update to the library can change the last bit, breaking replays recorded
  before it, and an independent implementation such as Ralph can only agree with a correctly rounded one.
- CORE-MATH itself. Correctly rounded and fast, but C: calling it needs `unsafe`, a bug in it would sit inside the
  sandbox's walls, and every host's build would need a C compiler for every target. Allowed only as a last resort,
  listed under IMPLEMENTATION.md's rule 2.
- Proving the functions in Rocq and turning them into Rust. Months of research per function, and Rocq can't produce
  Rust that's fast, or trusted.

**Why this won.** Replays and lockstep multiplayer become possible for a careful host, Seal's own tests give the same
output everywhere, and correct rounding keeps that true across library updates and lets Ralph check every bit. Proof
goes where it's cheap and decisive, in Ralph's definition of the right answer, and speed where it matters, in the Rust
port, with tests joining them.

**Tankun's words.** "Do we really need full stable floats?", "Wait, wait. Doesn't that depend on the host? The replay
accuracy?", "Why not full stable floats?", "Wait, doesn't stable floats mean both?", "Wait, so why does Cyborg even use
CORE-MATH?", "Yeah, but stable floats as you said, don't require that", "No, does stable floats mean that?", "Ah, so
CORE-MATH isn't about stable floats, just about math?", "Can you just fucking ask Cyborg Design Claude like the fucking
rule says?", "Why not CORE-MATH?", "80k LOC only? I thought it was gonna be 3m LOC and with Rocq", "So, do we port
some to Rust? Or port some to Rocq, and also make Rocq be able to transpile to Rust?", then, asked "Should Seal promise
the same results on every machine, bit for bit, with any maths function it adds later correctly rounded, ported to Rust
and checked against Ralph?", the owner chose: "yea".

## D51. Each call into a script is a task; one runs at a time, switching only at pauses

2026-11-23.

**Chosen.**
- **Each call the host makes into a script is a task.** While one task is paused at a door (D49), the host can call
  the script again, and the new call runs as another task.
- **Only one task of a script runs at a time,** and they switch only at pauses. Between two pauses a task is never
  interrupted, so nothing needs locks and no change is torn in half.
- **A script's tasks share its top-level names,** so a task that resumes sees what others changed meanwhile.
- **A door's name and labels, such as `wait(seconds: 2)`, are the host's** (D11); this decision is only about what
  pausing does.

```
mut hp = 30

proc patrol[guard]
    while true
        walk_to(guard, x: 10)
        wait(seconds: 2)
    end
end

proc on_hit[amount]
    hp = hp - amount
end
```

While `patrol` waits, the host can call `on_hit`; when `patrol` resumes, `hp` is lower.

**Weighed.**
- A paused script refusing every call until it's resumed. Simpler, but a patrolling guard couldn't be hit.
- Tasks switching at any moment, as threads do. Every shared name would need locks, and a change could be torn.

**Why this won.** It's how Lua's coroutines and Roblox's scripts behave, which game scripting relies on, and switching
only at pauses keeps every stretch between them as simple as a script with no tasks.

**Tankun's words.** "When the fuck are we gonna design `wait` shit?", "What the fuck is `seconds: 2`, isn't it
non-rich `wait(2 secs)`?", then, asked "Should each call into a script be a task, with only one running at a time,
switching only at pauses, and all sharing the script's top-level names?", the owner chose: "yea".

## D52. Where a script can pause is worked out, and the language server marks it

2026-11-23.

**Chosen.**
- **A host declares which of its doors can pause** (D49).
- **Seal works out which procedures can pause from their bodies:** one that calls a pausing door, or a procedure that
  can pause, can pause too, as failures are worked out (D44).
- **Nothing is written at a call.** The language server marks every line where a pause can happen (D7), so a reader
  sees where other tasks may change shared names (D51).

```
proc on_attack[target]
    old_hp = hp
    play_animation("swing")      // marked by the language server, if the host's door can pause
    hp = old_hp - 1
end
```

**Weighed.**
- Pauses invisible, as in Roblox. Simple, but a bug where another task changes a shared name during a pause is
  invisible too.
- A word at every pausing call, as JavaScript's `await`. Visible, but it spreads: every procedure calling one must be
  marked, and its callers, all the way up, the problem known as "what colour is your function".

**Why this won.** Pauses are visible where scripts are written, in the editor (D7), with nothing that spreads through a
script's code, the same way Seal treats types and failures.

**Tankun's words.** Asked "Should pausing be worked out, with hosts declaring which doors can pause, Seal finding which
procedures can from their bodies, and the language server marking each place a pause can happen?", the owner chose:
"yea".

## D53. The step budget counts each run between pauses; the memory cap covers the whole script

2026-11-23.

**Chosen.**
- **The step budget (D14) counts each run between pauses:** from a call or a resume to the next pause or the end. Each
  resume starts a fresh budget.
- **So a loop that pauses every pass can run forever,** as a patrol, a spawner or a day-night cycle wants, while a loop
  that never pauses is still stopped.
- **The memory cap covers the whole script,** across all its tasks (D51), since memory is held, not spent.

```
proc patrol[guard]
    while true
        walk_to(guard, x: 10)
        wait(seconds: 2)          // each pass starts a fresh step budget
    end
end
```

**Weighed.**
- A budget for a whole call, across its pauses. A patrol would eventually be stopped, though it never hogs the host.
- A memory cap per task. A script's tasks share its names and values (D51), so its memory is one whole.

**Why this won.** The step budget exists to stop a script freezing its host (D14), and a script that pauses hands
control back each time, so only a stretch with no pause can freeze anything.

**Tankun's words.** Asked "Should the step budget count each run between pauses, with a fresh budget at every resume,
while the memory cap covers the whole script?", the owner chose: "yea".

## D54. The host can cancel a paused task; a failed check or limit in any task stops the whole script

2026-11-23.

**Chosen.**
- **The host can cancel a paused task.** It ends right at its pause: nothing after the pause runs, and the script's
  other tasks carry on. A guard that dies mid-patrol has its `patrol` task cancelled.
- **A failed check or a limit in any task stops the whole script,** all its tasks, and the host is told (D14). Tasks
  share the script's top-level names (D51), and a task stopped halfway may have left them half-changed.
- **The host decides what comes next,** such as reloading the script.

```rust
goblin_script.cancel(patrol_task);     // a sketch; the names are the building session's
```

**Weighed.**
- Paused tasks that can never be cancelled. A dead guard's patrol would wait forever, holding its place.
- A failed check stopping only its own task. The other tasks would carry on with names it left half-changed.

**Why this won.** The host, which owns time and the game's world (D11), can end what no longer makes sense, while a
script that went wrong never keeps running on broken state.

**Tankun's words.** Asked "Should the host be able to cancel a paused task, ending it at its pause, while a failed
check or limit in any task stops the whole script?", the owner chose: "yea".

## D55. No units: hosts say units through their doors' labels

2026-11-23.

**Chosen.** Seal has no units. A host says what unit a door takes through its label, as in `wait(seconds: 2)` or
`move(guard, by_meters: 3)`, so every call says its unit and the language server shows it (D7). Units can come later
if scripts keep tripping over them.

**Weighed.**
- Units as Cyborg has them (Cyborg D255), on types, with conversions and families, so `wait(2 secs)` is a `Duration`
  and `wait(2)` is refused. They also catch adding seconds to meters, and an unlabelled value carried around before it
  reaches a door; far too heavy for an easy scripting language.

**Why this won.** Labels already give most of the benefit for no new concept, and a door's labels are the host's to
choose (D11).

**Tankun's words.** "What the fuck is `seconds: 2`, isn't it non-rich `wait(2 secs)`?" (quoted in D51), "Cyborg units?
💀", then, asked "Should Seal leave units out, with hosts saying units through their doors' labels?", the owner chose:
"yea. Shouldn't it be `secs:`?"

## D56. A name made inside a block ends where the block ends, and a loop's body is fresh each pass

2026-11-23.

**Chosen.** As in Unio (Unio D93, D94):
- **A name made inside a block ends where the block ends.** A loop's body is a fresh block on every pass, so a name made
  in it is made again each pass, and doesn't exist after the loop.
- **Names from outside a block are visible inside it,** so `tries = tries + 1` inside a loop changes the outer
  `mut tries` (D2).
- **While a name exists, it can't be made again;** `name = value` changes it instead, which needs `mut` (D2).

```
mut tries = 0
while tries < 7 and not won
    guess = be.Whole(ask("Your guess: ")) ?? 0     // a fresh guess each pass
    tries = tries + 1                              // changes the outer tries
end
```

**Weighed.**
- A name living on after its block, as in Python. The second pass's `guess = …` would be a change to a name not made
  with `mut`, refused, and Tankun's own guessing game would break.

**Why this won.** It's Unio's rule, and it's what the guessing game assumes: a value made for one pass belongs to that
pass. Found while writing Seal's first tests.

**Tankun's words.** Asked "Should a name made inside a block end where the block ends, with a loop's body a fresh block
each pass, as in Unio's D93?", the owner chose: "yea".

## D57. Performance goals, with Luau's and V8's interpreters as bars, on seven fixed benchmarks

2026-11-23.

**Chosen.** The building session gets goals, not a list of optimizations, and finds the optimizations that reach them
(D17, D46):
1. **Faster than Luau's interpreter,** with its native code generation off, on every benchmark below.
2. **Faster than V8 with `--jitless`,** which runs only Ignition, V8's bytecode register VM, on `fib` and n-body, where
   static types matter most; within twice its time on word count and text building, where V8's strings and hash
   tables are among the most tuned code anywhere.
3. **Full V8, with its JITs, measured for reference only,** with no target.
4. **Loading:** a 10,000-line script checked and compiled in under 100 ms.
5. **Footprint:** thousands of loaded scripts at once, each costing kilobytes beyond its code.
6. **Pausing:** pausing and resuming a task (D49) costs about as much as a call.
7. **Every number measured with every check on,** none dropped to reach it (D17).

The benchmarks are fixed; the building session may add more, never swap or drop any:

| Benchmark | Tests |
|---|---|
| `fib(32)`, recursive | calls |
| n-body, from the Computer Language Benchmarks Game | float maths, records |
| spectral-norm, from the Benchmarks Game | loops, arrays, floats |
| fannkuch-redux, from the Benchmarks Game | array shuffling, `Whole` maths |
| word count | `Dict` and `Text` |
| text building | joining text many times |
| 1,000 patrolling scripts | loading, pausing, footprint |

Luau and V8 run the Benchmarks Game's existing Lua and JavaScript versions where they exist. Binary-trees joins once
Seal has records inside records.

(Changed by D58: comparisons are Normal against Normal, under Cyborg's benchmark rules.)

**Weighed.**
- A list of optimizations, as Unio's D124 gave `Unlo\n`. It would be a guess at what matters before anything runs.
- Benchmarks chosen by the building session. A goal whose test is picked by whoever has to meet it drifts towards
  tests it already passes.
- Beating full V8. It compiles hot code to machine code, which Seal doesn't do on purpose (D46).

**Why this won.** Measurable goals against the strongest interpreters in game scripting and the web, on programs
written by others, keep the building honest, and leave how to reach them to the session that can measure.

**Tankun's words.** "What should the optimizations be? Or say the goal of perf", "What about some V8 goals too? Not beat
all, just yk", "What benchmarks? Or let the impl-tor decide what benchmarks?", then, asked "Should the prompt fix these
seven benchmarks, with the Luau and V8 goals, letting the builder add more but never swap or drop any?", the owner
chose: "yea. What about benchmark types. Ask Cyborg".

## D58. Seal's benchmarks follow Cyborg's rules, and D57's goals are judged Normal against Normal

2026-11-23.

**Chosen.** Seal takes Cyborg's benchmark rules (Cyborg's standard 10.9, from its D77, D78, D364 and D426):
- **Three kinds, by how a program is written:** Purpose, which isolates one thing and says what it measures; Normal,
  written as a normal person would, with no tuning; and Ceil, as fast as it can go, split as Cyborg's D364 does:
  `Ceil.source` (tuned code, built for the baseline CPU), `Ceil.machine` (adding a build for one exact CPU, as
  `target-cpu=native`), `Ceil.program` (adding a profile of its own run, profile-guided optimization) and
  `Ceil.machine.program` (both). There's no bare Ceil. A script is never compiled for a CPU (D46), so for Seal the CPU
  tuning and the profile apply to the VM that runs it.
- **Every benchmark runs in every kind,** and every number says its kind.
- **Every result reports six things:** run time (mean, standard deviation and minimum, after warm-up, over at least
  10 runs), peak memory, program size, load time (cold, and warm where Seal caches), correctness, checked before
  timing so a wrong answer is never timed, and the setup (machine, operating system, versions, and how busy the
  machine was). A language with a JIT also gets a warmed-up row.
- **D57's goals are judged Normal against Normal:** a plain Seal script against plain Luau and JavaScript versions,
  never against the Benchmarks Game's hand-tuned ones, which are Ceil. Ceil results are reported with no targets.
- **A Purpose benchmark for the cost of Seal's checks,** above all the float check after every operation (D18):
  measured, not guessed.
- **Binary-trees joins** once Seal has records inside records (D57).

**Weighed.**
- D57 as it stood, running the Benchmarks Game's existing Lua and JavaScript versions. Many are hand-tuned, so a plain
  Seal script would be compared with tuned code, making the numbers meaningless, as Cyborg's design session pointed out.

**Why this won.** They're the rules Tankun made for Cyborg, proven there: comparing like with like, labelling every
number, and checking answers before timing them caught real compiler bugs in Cyborg, and its check-cost benchmark is
what changed Cyborg's D440.

**Tankun's words.** "measured all?", "That isn't what `Ceil.machine` means", "It means shit like PGO", "Shit, what does
PGO mean", "Wait ,what the fuck. So, what does `Ceil.machine` do?", then, asked "Should Seal take Cyborg's benchmark
rules as listed, with D57's goals on Normal against Normal?", the owner chose: "yea".

## D59. Seal keeps `==` and `!=`, where Unio now writes `eq` and `not_eq`

2026-11-23.

**Chosen.** Seal keeps `==` and `!=` for comparing (D22), unchanged by Unio's D190, which replaced them with `eq` and
`not_eq`. Seal's `!` before a failure list (D45) and its `!=` don't clash: `!=` is a symbol of its own, as a longer
symbol always is (Unio D60).

```
if guess == secret
if guess != secret
proc load_level[name] -> Level ! [not_found, broken]
```

A deliberate difference from Unio, as D1 allows.

**Weighed.**
- `eq` and `not_eq`, following Unio's D190, as the design claude recommended, so that `!` has one job in Seal too.
  Seal is meant to be quicker to write, `not_eq` is longer than `!=`, and Tankun judged the overlap no real trouble.

**Why this won.** Seal keeps the short, familiar symbols, and Unio's reason, a beginner trap, matters less in a
language written in an editor (D7).

**Tankun's words.** "It's Seal, lol", "It's supposed to be shorter, that overlap isn't even that hard".

## D60. A procedure that takes nothing has no brackets, and every call writes `()`

2026-11-23.

**Chosen.** As in Unio (Unio D4): a procedure that takes nothing is written with no brackets after its name, and every
call to it, by the script or by the host, writes `()`.

```
proc on_tick
    timer = timer + 1
end

on_tick()
```

**Weighed.**
- Empty brackets, as `proc on_tick[]`. A list describing nothing; `[ ]` holds or describes values (Unio D165).
- A call without `()`, as `on_tick`. A call would look like reading a name, hiding that something runs.

**Why this won.** It's Unio's rule, and a host's `on_tick` is one of the commonest things a game script has. Seal's
building session stopped on it as "not designed yet".

**Tankun's words.** Asked "Should a procedure that takes nothing be written with no brackets, and always called with
`()`?", the owner chose: "y".

## D61. Unread parameters and `for each` names take `unused`; a `case` takes out only what it reads

2026-11-23.

**Chosen.** D2's rule, that a name made and never read is refused, holds everywhere a name is made:
- **A parameter never read is refused,** unless marked `unused`, as `proc on_hit[unused amount]`, for a host's callback
  whose shape is fixed (D3).
- **A `for each` name never read is refused,** unless marked `unused`, as `for each unused round in range[from: 1, to:
  3]`.
- **A `case` takes out only the values it reads,** as `case healed[by]`; taking out a value and never reading it is
  refused. A case needs no `unused`.

```
proc on_hit[unused amount]
    show_msg("ouch")
end

match event
    case healed[by]
        show_msg(fmt("{by} healed you"))
    ...
end
```

where `...` stands for the other cases.

**Weighed.**
- Letting parameters and `for each` names go unread. A typo, as `amont` for `amount`, would hide there as D2 refuses it
  elsewhere.
- `unused` inside a case, as `case healed[unused amount, by]`. Leaving the value out says the same, shorter.

**Why this won.** One rule wherever a name is made, so a typo never hides, and `unused` says "on purpose" wherever it's
needed. Chapter 6's `match` example broke it, by taking out `amount` and never reading it; it's fixed with this entry.
Seal's building session stopped on it as "not designed yet".

**Tankun's words.** "`?` means "What?", `??` means "yes", `???` means "no". Only works if the msg only has that",
"?", then, asked "Should unread parameters and `for each` names be refused unless marked `unused`, with a `case`
taking out only the values it reads?", the owner chose: "??".

## D62. A `mut` name can be changed through any depth of fields and positions

2026-11-23.

**Chosen.**
- **A change can reach through any number of fields and positions** of a `mut` name, as `bodies[pos: i].vx = 1.0`,
  and a method can end the chain, as `goblin.items.add("Gem")`.
- **It changes only that name's own value** (D15), and copy-on-write lets it happen in place, so it costs no more than
  one step does.
- **Every step is checked:** a bad position anywhere stops the script at its line (Unio D46).
- **`Dict` keys partway along a chain,** as `stock[key: "sword"].count = 1`, aren't decided yet, since the key might be
  missing partway through a line.

```
mut bodies = [Body[x: 0.0, vx: 1.0], Body[x: 5.0, vx: -1.0]]
bodies[pos: 1].vx = 2.0
goblin.items.add("Gem")
```

**Weighed.**
- One step only, as decided before (D27, D32, Unio D80): a part of a part is copied out, changed, and put back, three
  lines for one change, as n-body showed Seal's building session.

**Why this won.** It's the same change as one step, a name changing its own value, written once; Hylo, which Seal's
memory model comes from, works the same way.

**Tankun's words.** Asked "Should a `mut` name allow changes through any depth of fields and positions, with a method at
the end, leaving `Dict` keys partway along for later?", the owner chose: "??".

## D63. Unio's line rules, in Seal's syntax: comments, blank lines, indentation, spacing, line endings and `\`

2026-11-23.

**Chosen.** Seal takes Unio's rules for how lines are written:
- **Comments:** `//` comments to the end of the line, and `///` is a doc comment on the lines right before a
  declaration (Unio D74). A block comment is `comment name` on its own line, closed by a plain `end` (D20), and
  everything between is ignored.
- **Blank lines:** one in a row anywhere; two or more in a row are refused (Unio D97).
- **Indentation:** 4 spaces per level; anything else, a tab included, is refused, with the corrected line as the fix
  (Unio D10).
- **Spacing:** fixed, as Unio's table has it, with one space on each side of operators, `->`, `!`, `==` and `!=`
  included (D45, D59), and none inside brackets; a spacing mistake is refused, with the corrected line as the fix
  (Unio D98, D100).
- **Line endings:** `\r\n` counts as one line break (Unio D144).
- **Text:** `\` makes the next character literal, as `\"` and `\\`, and `\{` inside `fmt` text (Unio D30).

**Weighed.**
- Free spacing, as Lua and Python allow. Every refusal gives the fix, and fixed spacing makes every script look alike
  to whoever reads it.

**Why this won.** They're designed and tested in Unio, and nothing in Seal's purpose asks for them to differ. Seal's
building session read Unio's conventions meanwhile and stopped on everything else.

**Tankun's words.** Asked "Should Seal take Unio's line rules as listed, in Seal's syntax?", the owner chose: "??".

## D64. `==` and `!=` compare two texts exactly, character by character

2026-11-23.

**Chosen.**
- **`==` and `!=` work on two `Text`s:** equal when they hold the same characters in the same order. It's exact and
  case-sensitive, so `"Yes" != "yes"`.
- **Nothing is normalised behind the script's back:** "é" stored as one character and "e" followed by an accent mark
  are different texts, the same on every machine (D50).
- **`<` and `>` on text,** alphabetical order, aren't decided yet: what counts as alphabetical depends on the language
  being written.

```
if answer == "yes"
if name != ""
```

**Weighed.**
- Normalising Unicode before comparing. It needs tables that change between Unicode versions, which would break D50
  across Seal versions, and it hides work. A way to normalise on purpose can come later.
- Ignoring case. "Yes" and "yes" being equal surprises as often as it helps, and lower- and upper-casing depend on
  the language too.

**Why this won.** The simplest rule that's the same everywhere, as Rust, Lua and JavaScript compare text. Unio leaves
comparing text open (its chapter 5); Seal's building session needed it for the word-count benchmark.

**Tankun's words.** Asked "Should `==` and `!=` compare two texts exactly, character by character, with no
normalising, and `<` and `>` on text left for later?", the owner chose: "??".

## D65. Text's length says its unit, `.len.egc`, `.len.ucp` or `.len.bytes`; text is reached by code points

2026-11-23.

**Chosen.**
- **A text's length always names its unit,** and a bare `.len` on text is refused, with the three as the fix:
  - `.len.egc`, extended grapheme clusters, what a person sees as one character: 👍🏽 is 1;
  - `.len.ucp`, Unicode code points: 👍🏽 is 2;
  - `.len.bytes`, its UTF-8 bytes: 👍🏽 is 8.
  The docs teach `egc` as short for extended grapheme cluster, Unicode's own abbreviation, and `ucp` as short for
  Unicode code point, as Unio's D2 has short forms taught.
- **Each Seal version pins one Unicode version,** so where a grapheme ends never changes within a version (D50); moving to
  a newer Unicode is a listed change in a new Seal version.
- **`[pos:]`, `[offset:]` and `for each` reach a text by code points,** each giving a `Text` of one code point.
- **`.split(by: t)` splits a text into a `List<Text>`,** keeping empty pieces, so `"a  b"` gives `["a", "", "b"]`;
  `.split.no_empty(by: t)` drops them.
- Lists keep their bare `.len`, since an item is never ambiguous.

```
words = line.split.no_empty(by: " ")
if name.len.egc > 12
    show_msg("That name is too long")
end
```

**Weighed.**
- A bare `.len` counting characters by default. Counting bytes as characters, or the other way round, is the classic
  text bug in C and Go.
- `.len.chars`. "Char" means a byte in C, a UTF-16 unit in Java and JavaScript, and a code point in Rust.
- `.len.code_points`, Unicode's term spelt out. Precise but long; `ucp` keeps the precision, with the `u` saying it's
  Unicode's.
- `cp`, for code point: an abbreviation with an unfortunate other meaning.
- `ugc` or `ugp` for graphemes: in games, UGC means user-generated content; `egc` is Unicode's own abbreviation.
- Graphemes left out for now, as the design claude first proposed, since Unicode's tables change between versions.
  Games need them, as for a name limit that counts what the player sees, and pinning one Unicode version per Seal
  version keeps D50.

**Why this won.** Every length says exactly what it counts, in short words that stay accurate, and a script can count
what players see without ever cutting an emoji in half.

(Changed by D66: graphemes are the default, so a bare `.len` counts them, and text is reached by graphemes.)

**Tankun's words.** "Mf, where's narrowings", "Can we shorten the other two? Not bytes", "Where's codepoints", "The fuck?
What unit is chars", "`.bytes`, `ucp`, `ugp`? Not bare `cp` for obvious reasons.", "Why no graphemes?", then, asked
"Should text's length come in all three, `.len.egc`, `.len.ucp` and `.len.bytes`, with each Seal version pinning one
Unicode version, a bare `.len` refused, and the rest working in code points?", the owner chose: "yea".

## D66. Seal's text defaults to graphemes: `.len`, `[pos:]`, `[offset:]` and `for each` all count what a person sees

2026-11-23.

**Chosen.**
- **A bare `.len` on text counts extended grapheme clusters,** what a person sees as one character, so `.len` and
  `.len.egc` are the same.
- **`[pos:]`, `[offset:]` and `for each` reach a text by graphemes,** each giving one grapheme as a `Text`, so they
  always agree with `.len`.
- **`.len.ucp` and `.len.bytes` stay as narrowings** for the rarer need, as `.nearest.even` is beside `.nearest`
  (Unio D178).
- **`.split(by:)` and `.split.no_empty(by:)`** are unchanged.

```
for each i in range[from: 1, to: name.len]
    show_msg(name[pos: i])          // never half an emoji
end
```

This changes D65, which refused a bare `.len` and reached text by code points.

**Weighed.**
- D65's way, every length naming its unit and text reached by code points: right for Unio, the language for learning
  things properly, but a surprise to a scripter who sees 👍🏽 counted as 2.
- Graphemes for `.len` alone. `.len` and `[pos:]` would count different things, and a loop over a name would cut an
  emoji in half.

**Why this won.** Seal is meant to be easier to write, so it counts what people see, as Swift's `String.count` does,
and everything about text agrees with that count. The cost is speed: reaching the 500th grapheme means walking the
text, negligible for the short texts games use, and the VM can remember the boundaries it has found.

**Tankun's words.** "Actually, wait. Since it's Seal, and not Unio (learn master). Should we default to egc?", then,
asked "Should Seal's text default to graphemes for `.len`, `[pos:]`, `[offset:]` and `for each`, with `.len.ucp` and
`.len.bytes` as narrowings?", the owner chose: "yea".

## D67. At the top level, a name is used only after it's made, nothing pauses, and there's no `return`, `fail` or `?`

2026-11-23.

**Chosen.** A script's top level runs once, top to bottom, when it's loaded (D4), and three things are refused there,
at load time:
1. **Reaching a top-level name before the line that makes it,** directly or through a procedure the top level calls,
   since loading hasn't made it yet. The host's calls come after loading, when every top-level name exists.
2. **Pausing,** through a door or a procedure that can pause (D49, D52): the host waits for loading to finish (D48).
3. **`return`, `fail` and `?`,** which hand something to a caller, and the top level has none. A failure there is
   handled on the spot, with `??` or `otherwise`.

```
show_msg(greeting())                          // refused: greeting reads title, not made yet
title = "Goblin Cave"
level = load_level("cave") ?? default_level   // fine: handled here
```

**Weighed.**
- Letting the top level pause, with loading coming back paused. Every host would have to handle a half-loaded script.
- Leaving order to the running script, failing when a missing name is reached. Load time can see it, so it's caught
  there, as Seal catches everything it can (D17).

**Why this won.** It follows from what the top level is: code that runs once, at load, with nobody calling it and the
host waiting for it. It belonged with D4; Seal's building session found the gap.

**Tankun's words.** "?", "?", "Oopsies. I meant yes", after being asked "Should those three be refused at the top
level?".

## D68. Unio's `return` rules and its D9: every way gives a value, nothing follows a leaving line or an `if` that can leave

2026-11-23.

**Chosen.** Seal takes, in its own syntax:
- **Unio's D57:** if any `return` in a procedure gives a value, every way through must end in one; nothing may follow
  `return`, `break` or `fail` in the same block; and a bare `return` as a procedure's last line is refused.
- **Unio's D149:** a `while` counts as able to end, whatever its condition, so a procedure looping with `while true`
  still needs a `return` after the loop.
- **Unio's D9:** when any way through an `if` leaves, by `return`, `break` or a failure, nothing may follow its `end` in
  the same block; code that runs afterwards goes in the ways that don't leave.

```
proc on_hit[amount]
    if not alive
        return
    else
        hp = hp - amount
    end
end
```

Since Seal works out a procedure's result (D6), "a procedure that gives a value" means one with any `return` that gives
a value, written `-> Type` or not.

**Weighed.**
- Guard clauses, as in Lua, Roblox scripts and most languages: `if not alive` … `return` … `end`, then the rest. A
  habit Roblox scripters bring, and the body sits one level shallower; but a guard clause reads as two things in a row
  when it's one or the other.
- Leaving D9 out of Seal, as the design claude first did, judging it too strict for a quick scripting language.

**Why this won.** Tankun has chosen D9's rule each time it came up: in Plainly v1 (its D50), in Cyborg (its D430) and in
Unio, and its reasons, paths that read as alternatives at no cost at run time, don't depend on who the language is
for. `elif` keeps long chains flat.

**Tankun's words.** "Hm, why no `if` rule?", then, asked "Should Seal take Unio's `return` rules (D57, D149) and its D9,
so that nothing follows an `if` that can leave?", the owner chose: "incl".

## D69. A `for each` refuses calls that can change what it goes through, and pausing over a top-level name

2026-11-23.

**Chosen.** Extending Unio's D173, which refuses changing what a `for each` goes through inside its body:
- **A call that can change it is refused inside the loop.** Seal works out which procedures can change which
  top-level names, from their bodies, as it does for failures and pauses (D44, D52).
- **Pausing inside a loop going through a top-level name is refused,** since another task could change it meanwhile
  (D51). The fix is to copy it into a name of its own first, which costs almost nothing with copy-on-write (D15).

```
enemies_now = enemies
for each e in enemies_now
    hurt(e)
    wait(seconds: 1)
end
```

**Weighed.**
- Only D173's direct change refused. A change hidden inside a call, or made by another task during a pause, would
  slip past it.
- Every loop silently going through a snapshot. D173 refused that for Unio: a reader may expect the loop to see the
  change, and nothing would say it won't.

**Why this won.** A loop never goes through something that can change under it, and where a snapshot is wanted, the
copy line says so. Seal's building session found the gaps.

**Tankun's words.** "Bro, the comments are stacking onto another line", then, asked "Should a `for each` refuse calls
that can change what it goes through, and refuse pausing while going through a top-level name, with copying first as
the fix?", the owner chose: "??".

## D70. A value-giving call can't stand alone, nor can a line that only works out a value

2026-11-23.

**Chosen.** As in Unio (Unio D120, D161):
- **A call that gives a value can't stand alone,** its value dropped. To drop one on purpose, give it a name marked
  `unused` (D3): `unused roll = roll_dice()`.
- **A line that only works out a value is refused,** as `score + 1` alone, almost always a typo for
  `score = score + 1`.
- **A call that gives nothing stands alone,** as `show_msg("hi")`.

**Weighed.**
- Dropping values silently, as most languages do. A dropped value is usually a bug, and Seal already has a way to drop
  one on purpose.

**Why this won.** It's Unio's rule, and it fits D2's: nothing made is quietly ignored, and `unused` says when it's on
purpose. Seal's building session stopped on it as "not designed yet".

**Tankun's words.** Asked "Should a value-giving call standing alone, and a line that only works out a value, be
refused, with `unused` as the way to drop a value on purpose?", the owner chose: "??".

## D71. One spelling means one thing wherever it's seen: no shadowing

2026-11-23.

**Chosen.** A name can't be made, as a name, a parameter, a `for each` name or a `case` value, where its spelling
already means something: a top-level name, another name in use, a procedure, a door or a built-in. It's D56's "while a
name exists, it can't be made again" applied everywhere, with no exceptions.

```
results = [3, 12, 7]
proc best[results]        // refused: results is a top-level name
roll = roll()             // refused: roll is a procedure
show_msg = "hi"           // refused: show_msg is a door
```

**Weighed.**
- Shadowing, as most languages allow, where an inner name hides an outer one of the same spelling. A classic source of
  "why didn't my change stick?" bugs, and easier to hit in Seal, whose names have no `$` (D7).
- Unio's split, where a `$` name can share a procedure's spelling (Unio D127). Seal has no `$` to tell them apart.

**Why this won.** A reader never has to work out which of two things a spelling means, and the cost is picking another
word now and then. Seal's building session asked, after test d038 relied on a parameter sharing a top-level name.

**Tankun's words.** Asked "Should a name be refused wherever its spelling already means something there, with no
shadowing at all?", the owner chose: "??".

## D72. Records inside records, and a type holding itself where a value can end

2026-11-23.

**Chosen.**
- **A record can hold a record,** as a field of a record type. It's one bigger value, copied as a whole (D15) and
  changeable deep down through a `mut` name (D62).
- **A type can hold itself where a value can end:** through a `one_of` case, as `node[left: Tree, right: Tree]` beside
  `leaf`, or through a `List`, a `Dict` or a `Maybe`, all of which can be empty.
- **A type holding itself with no way to end is refused,** as a `record Node` with a field `next: Node`; the fix wraps
  it, as `next: Maybe<Node>`.
- **No box is needed,** unlike Cyborg (Cyborg D453), since Seal's VM keeps such values on its own heap; and since no
  reference can be kept (D15), they're always trees, never cycles.

```
record Dialogue
    line: Text
    choices: List<Dialogue>
end

one_of Tree
    leaf
    node[left: Tree, right: Tree]
end
```

This lets D57's set-aside benchmark, binary-trees, join.

**Weighed.**
- Flat records only, with branching faked by numbered IDs into one big list. A missing ID is found only while running,
  where Seal could check the real thing at load time.

**Why this won.** Games are full of things made of parts and things that branch: stats inside an enemy, dialogue trees,
behaviour trees, menus, skill trees and bags inside bags.

**Tankun's words.** "What for?", then, asked "Should records be allowed inside records, and a type allowed to hold
itself only where a value can end, through a `one_of` case, a list, a `Dict` or a `Maybe`?", the owner chose: "??".

## D73. A list moves freely into a limit at least as big, and fails with `full` into a smaller one

2026-11-23.

**Chosen.**
- **A list's `len.max` is part of its type** (Unio D167).
- **Moving a list into a name whose limit is at least as big, or has none, is allowed:** it always fits.
- **Moving it into a smaller limit, or from no limit into a limited name, can fail with `full`** (Unio D168), handled
  with `??` or `otherwise` (Unio D23), as `mut small: List<Text>[len.max: 5] = big_bag ?? []`. Like a `Dict` lookup
  (D32), it's a failure that isn't a call.
- **The same key twice in one `Dict` literal is refused,** since one of the two would silently vanish.
- **A list literal longer than its `len.max` is refused,** the written form of `full`.

```
mut roomy: List<Text>[len.max: 10] = small_bag
mut small: List<Text>[len.max: 5] = big_bag ?? []
```

**Weighed.**
- Refusing every move between different limits, as the design claude first proposed. Seal already has the failure
  for "it might not fit".

**Why this won.** A move that always fits needs nothing, and one that might not says what happens when it doesn't,
with the failure Seal already has for a full list. Tankun pointed out it could be handled.

**Tankun's words.** "?", "Don't you handle num 3?", "Wait, does Seal have full err handling yet?", "correct?", then,
asked "Should a list move into any limit at least as big freely, and into a smaller limit with `full` as a failure
handled with `??` or `otherwise`, while a repeated `Dict` key and an over-long list literal are refused?", the owner
chose: "??".

## D74. `record` stays the word for a record

2026-11-24.

**Chosen.** D27 stands: a record's type is declared with `record`, and each value made from it is a record.

```
record Student
    name: Text
    class: Text
    number: Whole
    year: Whole
end

fern = Student[name: "Fern Wattana", class: "M.3/2",
    number: 17, year: 2026]
```

A record is one stored entry with named parts, as in English a student's school record or a patient's file: the
`record Student` line is the blank form, and each `Student[…]` one filled-in form.

**Weighed.**
- `blueprint`, since the declaration is a template, as Tankun felt. Unreal Engine's Blueprints already mean visual
  scripting in game making, where NginSeal is aimed, and it's long for a word typed often.
- `type`, the design session's alternative. `Whole` is a type too, as Tankun pointed out, so it doesn't say what kind
  of type, and `one_of` makes types as well.
- `struct`, Rust's and C's word, which Tankun guessed it was.

**Why this won.** `record` says what each value is, a group of named fields, in the word business languages have used
for it since COBOL, and nothing else in game making uses it.

**Tankun's words.** "Okay, so why is it called a record? Seems more like a blueprint", "Honestly? Ahhh, I feel
blueprint, man. But idk if I should change it", "`type`? You profound fuck, `Whole` is also a type", "Okay, okay.
Explain to me how English record works", "Bro thinks I know more Thai than English.", "Show me a student record, not
real", then, asked "Should Seal keep `record`?", the owner chose: "Ahhhh, keep that shi"

## D75. Every text fills in its braces, with `\{` for a brace as it is; `fmt` goes

2026-11-24.

**Chosen.** Text fills in what's in its braces wherever it's written, replacing D12's `fmt("…")`, as Unio's D200 does:

```
show_msg("Hi {player.name}, {lives} lives left")
greeting = "Round {round}!"
show_msg("Type \{help} for commands")
```

- **A brace to show as it is is written `\{`,** as D63 already has it, so the last line shows `Type {help} for
  commands`.
- **Braces keep what D12 and D28 allow:** a name, followed by its fields and facts, as in `{player.name}` and
  `{bag.len}`; a name, field or fact that doesn't exist is refused at load time.
- **`fmt` goes,** from the built-ins too.

**Weighed.**
- `fmt("…")` (D12). Text not meant to fill needs no `\`, but there are two kinds of text, and `show_msg("{tries}")`
  showing `{tries}` is a trap, which Unio's D200 removed for the same reason.

**Why this won.** One kind of text, filled the same way in a call or a name, matching Unio's text, which Tankun wanted
Seal's formatting to follow.

**Tankun's words.** "Mf, you know. Let me design some Seal instead Ngin rn. First of all, I'll make its formatting
follow Unio, and maybe add named `end`s, because I'm not used to bare", "yea`" (meaning text formatting), "`{player.name}`
Let me guess that's a struct? Just trying to learn programming rn, haha. Yes, I don't fucking know how to program
<skull emoji>", "What's it fir?", then, after D74, asked "Should every Seal text fill its braces, with `\{` for a brace
as it is, replacing `fmt(...)`?", the owner chose: "yea"

## D76. Blocks close with `end` and what they close, as Unio's do

2026-11-24.

**Chosen.** A block closes with `end` followed by what it closes, replacing D20's plain `end`, in Unio's forms (Unio
D8) and the same pattern for Seal's own blocks:

```
proc heal[hp: Whole] -> Whole
    if hp < 10
        return hp + 5
    else
        return hp
    end if
end proc heal
```

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
| `comment old_scoring` | `end comment old_scoring` |

A closer at the wrong depth or naming another block is refused, and the language server types the closer when a block
is opened. D20's rules on depth and on naming the block a missing closer belongs to stand.

**Weighed.**
- A plain `end` (D20), as Lua, Ruby and Luau write it: shorter, and an editor shows which block each one closes.

**Why this won.** Tankun is used to Unio's closers and writes both languages, and with the language server typing the
closer, a longer one costs nothing to write while staying readable where there's no editor: chats, screenshots and
messages.

**Tankun's words.** "Mf, you know. Let me design some Seal instead Ngin rn. First of all, I'll make its formatting
follow Unio, and maybe add named `end`s, because I'm not used to bare" (quoted whole in D75), "One word: IDE", "And
btw, that is illegal Seal?", "IDEs could fucking auto insert end", then, asked "Should Seal close blocks with Unio's
named `end`s, replacing D20's plain `end`?", the owner chose: "yea"

(Replaced by D102: bare `end` again, with an optional, checked `\\` note.)

## D77. Text is joined only with braces; `&` goes

2026-11-24.

**Chosen.** Seal has no operator for joining text, replacing D29's `&`, as Unio's D201 does. Text is joined by filling
it in (D75):

```
show_msg("A {kind} appears")
show_msg("Score: {score}")
```

- **A `Whole` and a `Bool` in braces become text by themselves,** as they did with D29's `&`.
- **`&` and `+` on text are refused,** each with the braces form as the fix; their wording comes with a later batch.
- D29's test becomes `d077_braces_join`.

**Weighed.**
- Keeping `&`, which turns a `Whole` or a `Bool` into text by itself (D29). With braces filling everywhere (D75), it's
  a second way to build the same text, and braces already hold everything it could join, fields and facts included.

**Why this won.** One way to build text, the one that looks like what it shows, matching Unio.

**Tankun's words.** Asked "Should Seal drop `&`, so text is joined only with braces, as in Unio?", the owner chose:
"yea"

## D78. Seal recommends Unio's colours and font, and forces neither

2026-11-24.

**Chosen.**
- **Seal's docs show code in Unio's colours and font:** D194's colours, with D197's rule for narrowings, and Atkinson
  Hyperlegible Mono with no ligatures (Unio D198).
- **Seal's editor extension offers Unio's colours as a theme** and suggests it once, when it's installed, without
  switching to it. The editor's own theme stays the default.
- **The language server only says what kind each piece of code is** (D7), which any theme colours; with Unio's theme
  picked, those kinds get Unio's colours.
- **A host shows Seal's messages however it likes:** Seal's library colours nothing.

**Weighed.**
- Unio's colours and font as the fixed choice, as Unio does for its own command and docs (Unio D193). Seal runs inside
  other people's programs and editors, which choose their own look.

**Why this won.** Seal and Unio look alike wherever Seal shows its own code, and nobody embedding Seal or writing it in
their own editor has a look forced on them.

**Tankun's words.** Asked "Should Seal recommend Unio's colours and font, used in its docs and offered as an optional
editor theme, without forcing either?", the owner chose: "yea"

## D79. `?? crash` works on a `Maybe`; `?? ignore` on one is refused

2026-11-24.

**Chosen.**
- **`?? crash` on a `Maybe`** stops the script with a failed check when it's `none`, as `?? crash` does for a failure
  (D54): `id = nearest_enemy() ?? crash` says there must be one.
- **`?? ignore` on a `Maybe` is refused,** with `??` and a value as the fix, since the name would be left with nothing
  in it, which is the `nil` D38 rules out. `?? ignore` stays for calls that can fail and give back no value, as
  `save(game) ?? ignore`.

The wordings come with a later batch.

**Weighed.**
- Refusing both on a `Maybe`, leaving `??` with a value and `match`. `?? crash` says plainly "this can't be missing
  here", which a learner otherwise writes as a `match` with a stop in one case.

**Why this won.** Each keeps its meaning from failures, and a name never holds nothing.

**Tankun's words.** "Wait, what the fuck happens if you make a List wit no mut?", "Wait, wait. No value? Like fucking
NULL?", "What would `ignore` do?", then, asked "Should `?? crash` work on a `Maybe` and `?? ignore` be refused?", the
owner chose: "yea"

## D80. `instead` is the last line of its case

2026-11-24.

**Chosen.** Once `instead` runs, the call has its value and the script carries on after `end otherwise`, so a line
after `instead` in the same case is refused, as a line after `return` or `break` is (Unio D57). Anything else the case
does goes before it:

```
level = load_level(name) otherwise
    case not_found
        show_msg("Starting at 1")
        instead 1
end otherwise
```

The wording comes with a later batch.

**Weighed.**
- Allowing lines after it, run before carrying on. `instead` would then mean "remember this value" rather than "use
  this value now", and the order of the lines would no longer be the order things happen.

**Why this won.** A line that can never run is a mistake, and Seal already refuses it after every other way of leaving
a block.

**Tankun's words.** Asked "Should `instead` have to be the last line of its case?", the owner chose: "yea"

## D81. A procedure's header writes its types

2026-11-24.

**Chosen.** Every parameter writes its type, and a procedure that gives back a value writes `-> Type`, replacing D5's
and D6's types worked out from the body:

```
proc greet[name: Text] -> Text
    return "Hello, {name}"
end proc greet

proc on_hit[amount: Whole]
    hp = hp - amount
end proc on_hit
```

- **A parameter with no type is refused,** and so is a procedure that gives back a value without `-> Type`.
- **The body is checked against the header,** so a mistake is reported where it is, and the host's calls are checked
  against the header at load time (D48).
- **Names made inside a body, and at the top level, still take their type from their value** (D2, D30).
- **`! [failures]` stays optional,** worked out from the body (D44); whether to require it is its own question.

**Weighed.**
- Types worked out from the procedure's own body (D5, D6). Shorter headers, but a mistake in the body can change a
  procedure's type, and what the host is calling isn't written anywhere.

**Why this won.** A procedure is what the host calls (D4), so its header is Seal's interface, and an interface is
written down. It's the split Tankun sketched for Keel: names inferred, procedures typed.

**Tankun's words.** has code, then, asked "Should Seal procedures write their parameters' types and their result
type, replacing D5's and D6's inference for procedures?", the owner chose: "yea"

## D82. A written type may leave out the parts its value shows

2026-11-24.

**Chosen.** D30's rule, written only where the value can't say it, applies to each part of a type, not only the whole:

```
board: Array[len: 64] = [all: 0]          // Whole, from the 0
names: List[len.max: 3] = ["A", "B"]      // Text, from the items
```

- **What the value doesn't show must be written:** `bag: List[len.max: 10] = []` is refused, since `[]` shows no item
  type, with the full type as the fix.
- **Writing the full type stays allowed,** as `Array<Whole>[len: 64]` (D30).

**Weighed.**
- The full type whenever any of it is written (D30 as it stood). A length that only the writer knows would bring the
  item type with it, though the value already says it.

**Why this won.** It's D30's own reason taken one step further: write what only you know, and let the value say the
rest.

**Tankun's words.** The idea came with Tankun's sketch for D81 (`board: List = [all: 0] // <Whole> infer`). Asked
"Should a written type be able to leave out its item type when the value shows it, as `board: Array[len: 64] = [all:
0]`?", the owner chose: "yea"

## D83. A name can't be `all`

2026-11-24.

**Chosen.** `all = 5` is refused, as is a parameter, a `for each` name or a `case` value named `all`, so `[all: value]`
always means a list or array whose every item starts as `value` (D22, Unio D169), never a `Dict` keyed by a name. Its
wording comes with a later batch.

**Weighed.**
- Reading `[all: …]` as the list form unless a name `all` is in reach, as the building session stops on now. The same
  text would then mean a list in one script and a `Dict` in another.

**Why this won.** It's the case Unio settled the same morning for `range` (Unio D207): a word with a meaning of its own
in brackets can't also be a name.

**Tankun's words.** Asked "Should `all` be refused as a name in Seal?", the owner chose: "yea"

## D84. A procedure that can fail writes `! [ … ]` in its header

2026-11-24.

**Chosen.** A procedure that can fail lists its failures with `! [ … ]` on its first line, after `-> Type` if it has
one, replacing D44's failures worked out from the body:

```
proc load_level[name: Text] -> Level ! [not_found, broken]
```

- **The list must match the body exactly:** every `fail`, and every failure passed up with `?`. A missing or extra
  failure is refused, with the right list as the fix, so it can be copied.
- **A procedure that can't fail writes no `!`.**
- `fail name` stays as D44 has it.

**Weighed.**
- Failures worked out from the body, written only if wanted (D44). Shorter headers, but a change deep in a body, such
  as a new `?`, could quietly add a failure every caller then has to deal with.

**Why this won.** D81's reason: the header is what the host calls, and a host gets each failure back as an error
(D48), so what can come back is written there. It's also how Unio writes it.

**Tankun's words.** Asked "Should a procedure that can fail write `! [ … ]` in its header, replacing D44's worked-out
failures?", the owner chose: "yea"

## D85. A host's types are used without declaring them, as doors are

2026-11-24.

**Chosen.** A host opens its own records and `one_of`s for a script, made with `#[derive(Seal)]` (D48), the way it
opens doors (D11). A script uses them by name with nothing declared:

```
// Vec2 is the host's: no record Vec2 here
player_pos = Vec2[x: 40.0, y: 80.0]
```

- **A script declaring a type with a host type's name is refused,** since one spelling means one thing (D71).
- **The language server shows a host type's fields and cases,** as it shows a door's parameters.
- A host type's names follow Seal's spelling, checked when the host opens it, as for doors (D24).

**Weighed.**
- The script declaring a matching copy, checked against the host's when it loads, as the building session built
  meanwhile, since no decision covered it. Every script for a host, such as every NginSeal game, would start by
  repeating the host's types, and a slip in the copy would read as an error in the host.

**Why this won.** Everything a script gets from its host arrives the same way, opened by the host and used by name.

**Tankun's words.** "What the fuck? Why would you decl in Seal", then, asked "So, should host types be usable without
declaring them, the way doors are?", the owner chose: "yea"

## D86. D68's rule covers `match`, and an unread `at` name is marked `unused`

2026-11-24.

**Chosen.**
- **When any case of a `match` leaves,** by `return`, `break` or `fail`, nothing may follow its `end match` in the same
  block, as D68 has it for `if` (Unio D9). The rest goes in the cases that don't leave.
- **A `for each`'s `at` name that's never read is marked `unused`,** as `for each count at key unused item in stock`,
  as D61 has it for the `for each` name itself.

**Weighed.**
- Leaving `match` out of D68. A line after it would run on only some of its ways, which is what D68 is there to show.
- Allowing an unread `at` name. It's a name made and never read, which D2 refuses everywhere else.

**Why this won.** Each follows a rule already decided, for the same reason.

**Tankun's words.** Asked "Should both work that way?", the owner chose: "yea"

## D87. Labelled arguments and a case's values go in their declared order, unless `labels.any_order`

2026-11-24.

**Chosen.**
- **A call writes its arguments in the order the procedure declares them,** leaving out any with a default (D42).
  Another order is refused, with the reordered call as the fix, which the language server can apply.
- **`labels.any_order [ … ]`,** written by the procedure's maker in place of `labels`, lets its labelled arguments go
  in any order. It's a narrowing: an any-order kind of `labels` line. This is Cyborg's rule (its standard, 5.4).
- **A case takes out its values in their declared order,** skipping any it doesn't read (D61).

```
proc heal[hp: Whole, amount: Whole, times: Whole] -> Whole
    labels.any_order [amount as by, times as times]
    return hp + amount * times
end proc heal

hp = heal(hp, times: 2, by: 10)    // allowed, only for heal
```

**Weighed.**
- Any order everywhere, as Python and Kotlin allow named arguments. The names keep it unambiguous, but calls to one
  procedure would read differently from place to place.
- Declared order only, as the design session first proposed. Tankun pointed to Cyborg, which lets a maker opt out.
- A case's values in any order, as the design session first proposed, comparing them to labels at a call, which it
  wrongly said already went in any order: neither Seal nor Unio had decided label order.

**Why this won.** One way by default, so every call reads alike, and freedom where the maker says order doesn't
matter.

**Tankun's words.** "Wait, labels at a call are ordered?", "Cyborg just defaults to ordered, and allow maker to
unorder", "Is it really a narrowing?", then, asked "Should Seal take Cyborg's rule, with labels in declared order
unless the maker writes `labels.any_order`, and a case's values in declared order?", the owner chose: "yea. Should we
add optional `Whole, hp, amount, times`?"

## D88. Parameters sharing a type can be grouped, with ` : ` before the group's type

2026-11-24.

**Chosen.** Parameters next to each other that share a type can list their names and write the type once after them,
as Go does, with a space on each side of that `:`, since it covers the whole group:

```
proc heal[hp, amount, times : Whole]
proc hit[hp, amount : Whole, by: Text]
proc greet[name: Text]
```

- **Each group takes the type after it,** so a parameter of another type is one more group at the end, with nothing
  rewritten.
- **A group's `:` without the spaces, and a single name's `:` with them, are refused,** with the respaced header as
  the fix, as spacing always is (D63). It's spacing's one exception: a different spacing for a different meaning, as a
  negating `-` has no space after it (Unio D27).
- **Writing each type stays allowed,** as `[hp: Whole, amount: Whole]` (D21, D81).

**Weighed.**
- Each parameter writing its own type only, as Unio's D126 chose. Long headers once D81 made types required.
- The type once in front, as `[Whole hp, amount, times]`, Unio's D61, which D126 replaced: a parameter of another type
  means rewriting the header into the other form.
- The group's type with the usual `: `, as `[hp, amount, times: Whole]`. As Tankun pointed out, a group's type isn't
  the same thing as one name's, and that reads as the type of `times` alone.

**Why this won.** Shorter headers where types repeat, a header that grows by adding a group, and spacing that shows at a
glance which `:` covers a group.

**Tankun's words.** "yea. Should we add optional `Whole, hp, amount, times`?" (quoted whole in D87), "Who the fuck
said they could mix", has code, "Why no space? It's not the same thing", then, asked "Should Seal let parameters share
a type as a group, `[hp, amount, times : Whole]`, with a space on both sides of the group's `:`?", the owner chose:
"yea"

## D89. Splitting by empty text is refused while loading wherever it can be seen

2026-11-24.

**Chosen.**
- **`split(by: "")` is refused while loading,** and so is any separator the checker can see is empty, with `for each`
  over the text as the fix, since that gives its letters (D66).
- **A separator that only turns out empty while running is a failed check,** as dividing by zero is (D18, D54, Unio
  D46): nothing is guessed for it.

Its wording comes with a later batch.

**Weighed.**
- Giving each letter, as JavaScript does. A silent answer to what's most likely a mistake.
- `by:` taking only text written in the call, so every case is caught while loading. A special rule for one argument.
- A type for text that can't be empty, so the checker proves it. Machinery Seal isn't built for: it's neither CyborgPL
  nor Lua, as Tankun put it.

**Why this won.** Refusing at load time wherever it's possible, and stopping where it isn't, is how Seal treats every
other mistake.

**Tankun's words.** "Why not comptime?", "What the fuck", "What the fuck?", then, asked what they meant by load time,
Tankun answered: "Like, bro. We aren't fucking Cyborg**PL**, and we're not fucking Lua either.
What I'm saying is fucking refusing at compile time is possible, do it"

## D90. A `Dict` key partway along a change is refused

2026-11-24.

**Chosen.** A change can't go through a `Dict` key on its way, since a missing key is a failure (D32) and a change has
nowhere to deal with it: `??` and `otherwise` belong to calls. The value is taken out, changed and put back:

```
shops[key: "north"].stock.add("Key")      // refused

mut north = shops[key: "north"] ?? crash
north.stock.add("Key")
shops[key: "north"] = north
```

- **A key at the very end of a change stays allowed,** as `shops[key: "north"] = north`, which sets or adds it (D32).
- The refusal's fix is the three lines; its wording comes with a later batch.

**Weighed.**
- Allowing it, with a missing key a failed check that stops the script. A lookup that's expected to fail sometimes
  would then stop scripts silently, nowhere in sight.

**Why this won.** Every way a lookup can fail stays visible on the line where it's dealt with.

**Tankun's words.** "?" (to the question), then, asked "Should a lookup in the middle of a change like that be
refused?", the owner chose: "??" (yes)

## D91. Comments: a space before `//`, comments inside brackets, `///` on any declaration, `comment` a keyword

2026-11-24.

**Chosen.**
- **A `//` after code has one space before it,** as `x = 1 // hi`; `x = 1// hi` is refused, with the spaced line as
  the fix.
- **A comment line inside an open bracket is allowed,** indented like the line after it, as any comment line is (Unio
  D145, through D63).
- **`///` documents any declaration:** a procedure, a family, a record, a `one_of`, a field, a case or a top-level
  name. The language server shows it on hover.
- **`comment` is a keyword,** as in Unio's keyword table, so it can't be a name.

Two of the building session's questions were already answered: a comment line's indentation follows Unio D145
through D63, and a block comment ends at its own `end comment name` (D76).

**Weighed.** Nothing else was weighed: each follows what Unio does, or what Seal's spacing rules already say.

**Why this won.** Comments follow the same rules as everything else on a line.

**Tankun's words.** Asked "Do all four work that way?", the owner chose: "yea"

## D92. In text, `\` goes only before `"`, `\` or `{`, and a tab character is refused

2026-11-24.

**Chosen.**
- **A `\` before any character but `"`, `\` or `{` is refused while loading,** as `"Line 1\nLine 2"`, with a `why:`
  saying `\n` isn't a new line in Seal, so a habit from another language doesn't turn into a stray `n`.
- **A tab character inside text is refused,** as tabs are everywhere else in a script (D63), since it can't be told
  from spaces.

How a new line or a tab is written inside text isn't decided, in Seal or Unio. The wordings come with a later batch.

**Weighed.**
- Showing the character after `\` as it is, so `\n` prints `n`, as Unio D30's rule reads on its own. A silent
  surprise for anyone used to C, Python or JavaScript.

**Why this won.** A script that can't mean what it looks like it means is refused, with the reason given, rather than
run.

**Tankun's words.** Asked "Should `\` before anything but `"`, `\` or `{`, and a tab inside text, both be refused?",
the owner chose: "yea"

## D93. Seven refusals: empty braces, built-in names, nested declarations, `_` in types, leading zeros, other letters, blank lines in brackets, empty `one_of`s

2026-11-24.

**Chosen.** Each is refused while loading, with its wording in a later batch:
- **Empty braces in text,** as `"{}"`, with `"\{}"` as the fix to show them.
- **A procedure named after a built-in or a door,** since one spelling means one thing (D71).
- **A `record`, `one_of`, `proc` or `fam` inside a block:** they're declared at a script's top level only.
- **A `_` in a type's name,** since types are `CamelCase` (D24), and **a number with a leading zero,** as `007`.
- **Letters beyond plain English in a name,** as `café`, for now, as Unio's names are (Unio D99).
- **A blank line inside an open bracket,** so a statement spread over lines stays in one piece (D8).
- **A `one_of` with no cases,** since no value of it could exist.

**Weighed.** Nothing else was weighed: each follows a rule Seal or Unio already has, and the building session had
stopped on each as not designed yet.

**Why this won.** What Seal can't mean, it refuses with a fix.

**Tankun's words.** Asked "Should all seven be refused?", the owner chose: "yea"

## D94. `places:` in braces, as Unio has it

2026-11-24.

**Chosen.** A `Float` in braces can take `places:` after a comma, exactly as Unio's D175 has it, with D176's rule for
values rounding to zero and D178's ties going away from zero:

```
show_msg("Speed: {speed, places: 2}")       // Speed: 0.30
```

It always shows that many digits, shows no `~`, since the text is exactly the rounding asked for, and `places:` on
anything but a `Float` is refused.

**Weighed.** Nothing else was weighed: Seal takes Unio's floats (D16, D22), and this was the part left open.

**Why this won.** A game shows a speed, a timer or a percentage without seventeen digits.

**Tankun's words.** Asked "Should Seal take Unio's `places:` in braces as it is?", the owner chose: "yea"

## D95. `==` and `!=` compare records, lists, `Dict`s and `one_of`s by their contents

2026-11-24.

**Chosen.** Since every name owns its value and nothing is shared (D15), two bigger values are equal when their contents
are:
- **records:** every field equal;
- **lists and arrays:** the same length, with equal items in the same order;
- **`Dict`s:** the same keys, each with an equal value; the order keys were added in doesn't count;
- **`one_of`s:** the same case, with equal values.

```
if player.pos == spawn.pos
if hand == ["A", "K"]
```

**Weighed.**
- Refusing `==` on them, comparing field by field by hand. Long, and easy to get wrong when a field is added later.
- Counting a `Dict`'s order. The order shows when going through it (D33), but it isn't part of what the dictionary
  holds.

**Why this won.** With no references, "the same contents" is the only thing "equal" can mean.

**Tankun's words.** Asked "Should `==` and `!=` work on records, lists, `Dict`s and `one_of`s, comparing their
contents?", the owner chose: "yea"

## D96. `<` and `>` on text are refused, for now

2026-11-24.

**Chosen.** `<`, `>`, and the other orderings, on two texts are refused while loading. How text is put in order is
decided when sorting is, as something that says how it orders.

**Weighed.**
- Ordering by character codes, as Lua and Python do. `"Zebra" < "apple"`, and `"é"` after `"z"`.
- Alphabetical order by Unicode's rules. It differs between languages, and needs large tables and a locale, much to
  carry in an embedded language.

**Why this won.** An operator would quietly pick one of several orders, none of them right everywhere.

**Tankun's words.** Asked "Should `<` and `>` on text stay refused for now?", the owner chose: "yea. When the fuck is
handler"

## D97. Handlers, as Unio's, with family members written without a keyword

2026-11-24.

**Chosen.** Seal has Unio's handlers (Unio D41 to D44), and inside any family, members are written without a keyword,
since the family's first line says what they are:

```
fam.proc spawn
    spawn[kind: Text]
        show_msg("A {kind} appears")
    end spawn
    spawn.boss[kind: Text]
        show_msg("The boss {kind} appears!")
    end spawn.boss
end fam.proc spawn

fam.handler level_failed
    start_over
        case not_found
            instead 1
        case broken
            instead 1
    end start_over
end fam.handler level_failed

level = load_level(name) otherwise level_failed.start_over
```

- **A procedure member is written by its full name,** as `spawn.boss`, since it's called on its own anywhere; a
  narrowed procedure is declared only inside its word's family (D43, Unio D41). This replaces D43's `proc` inside a
  family.
- **A handler member is written by its short name,** as `start_over`, since a handler is only ever used after
  `otherwise`, by its full name `level_failed.start_over`.
- **A member's closer is `end` and its name,** as `end spawn.boss` and `end start_over` (D76).
- **From Unio:** a handler is used as if its cases were written at the call and must cover every failure left there
  (Unio D40, D42); one that can `return` or `break` is marked `*` where it's used, as `level_failed.*give_up` (Unio
  D42); `include` brings a handler's cases into an `otherwise` block (Unio D43); and a handler can take parameters
  (Unio D44).

**Weighed.**
- Every member in full with its keyword, as `handler level_failed.start_over`, as Unio has it and the design session
  argued for. Tankun: "Do you really have to write `handler` again?"; the family's first line already says it.
- Handler members as `.start_over`, a short name with a dot, or as `handler start_over`, both proposed and taken back
  by the design session, which changed its answer several times when Tankun only asked whether it was sure. Cyborg's
  design session, asked by Tankun for an outside read, pointed out Cyborg's rule: full names, no repeated keyword
  (Cyborg D46, D47).
- The same naming in both families. Tankun, through Cyborg's design session: a short procedure member, as
  `boss[kind: Text]`, looks like another procedure, while a handler is never used on its own, so the two kinds of
  member are used differently and needn't match.
- Keeping `proc` inside procedure families, so every procedure starts with `proc`. A narrowed procedure only ever
  exists inside its family, as Tankun pointed out ("It's obviously a proc, though"), so there's no alone version for it
  to match.

**Why this won.** No word is repeated that the family's first line already says, each member is written the way it's
used, and handlers arrive as Unio designed them.

**Tankun's words.** "yea. When the fuck is handler" (quoted whole in D96), "Do you really have to write `handler`
again?", "What the fuck is readable on its own do?", has code, "Would the err only show one line?", "Why `.`
there?'[==]", "How?", "You sure?", "You sure?", "You sure?", "`start_at[level: Whole]`
How to reach into a list exactly like that, in Seal", "Ask Cyborg Design Claude about how fucked you are", "?", "So,
what's the idea, and why", "What about proc fams?", "Should we remove `proc` inside proc fam/", "Can you make `.boss`
outside of fam?", "It's obviously a proc, though. Since it's inside that, yk", then, asked "Should family members have
no keyword, with procedures by full name, handlers by short name, and the rest from Unio's handlers?", the owner
chose: "yea. What about `?!` replacing `otherwise` for non-block ver?"

(Changed by D98: on one line a handler follows `??`, and `otherwise` is only a block.)

(Changed by D110: handlers live in a `group.handler`, declared and reached with `::`.)

## D98. On one line, a handler follows `??`; `otherwise` is only a block

2026-11-24.

**Chosen.** `??` deals with a failure on one line with a value, `ignore`, `crash`, or now a handler, and `otherwise`
only ever opens a block of cases:

```
level = load_level(name) ?? 1
level = load_level(name) ?? level_failed.start_over
level = load_level(name) ?? level_failed.*give_up
level = load_level(name) otherwise
    include level_failed.start_over
end otherwise
```

Seal always knows a handler from a value, since one spelling means one thing (D71). This replaces D97's, and Unio
D42's, `otherwise level_failed.start_over` on one line.

**Weighed.**
- `otherwise` and the handler's name on one line, as Unio has it. A word doing two jobs, a one-line form and a block.
- `?!`, a new symbol for a handler on one line, as Tankun first asked and the design session then recommended: a third
  failure mark to learn beside `?` and `??`.
- `!!`, which Kotlin uses for the opposite, crashing when there's nothing there.

**Why this won.** `??` already means "deal with it on one line", and already takes things that aren't values.

**Tankun's words.** "yea. What about `?!` replacing `otherwise` for non-block ver?" (quoted whole in D97), has code,
"or just `??`?", then, asked "Should `??` take a handler on one line, with `otherwise` only for the block?", the owner
chose: "yea"

(Changed by D110: a handler is reached with `::`, as `?? level_failed::start_over` or `?? ::start_over`.)

## D99. A `range` is a `List<Whole>`

2026-11-24.

**Chosen.** `range[…]` gives a `List<Whole>`, so it can go anywhere a list can: stored in a name, passed to a
procedure, read with `.len` or `[pos: …]`, or gone through with `for each`.

```
rounds = range[from: 1, to: 3]        // [1, 2, 3]
for each i in range[from: 0, before: bag.len]
```

- **Going through a range with `for each` builds no list:** Seal counts as it goes. A script can't tell, so it's not a
  rule, only speed.
- **A range kept in a name is a real list,** and counts against the script's memory cap like any other (D14).
- A short, fixed run reads better written out, as `[1, 2, 3]`; a range is for long runs, steps, and runs that depend on
  values.

**Weighed.**
- A range only inside `for each`, as the building session stops on now. Storing or passing one would be refused for no
  reason a learner could see.
- A `Range` type of its own, as in Python and Rust. Another type, with its own answers to what `.add`, `[pos: …]` and
  passing it as a list do.

**Why this won.** Nothing new to learn: a range is a list you didn't have to write out.

**Tankun's words.** "why?", "Why not just make it a list?", "Why not just write it normally?", then, asked "Should a
`range` just be a `List<Whole>`, usable anywhere a list is?", the owner chose: "yea"

## D100. Every value shows as it's written in Seal

2026-11-24.

**Chosen.** A list, array, record, `Dict` or `one_of` in braces, or through `be.Text`, shows exactly as it would be
written in a script, as a `Float` already does with its `~` (Unio D155):

```
show_msg("{bag}")          // ["Gem", "Key"]
show_msg("{orc}")          // Enemy[name: "Orc", hp: 50]
show_msg("{stock}")        // ["sword": 1, "potion": 3]
show_msg("{event}")        // Event.hurt[amount: 5]
show_msg("{speeds}")       // [~0.1, 0.5]
```

- **Text inside shows its quotes,** so items can't run together.
- **A host's types show the same way,** by their Seal names.
- Text for a player, as "Gem, Key", is built by the script itself.

**Weighed.**
- Refusing them in braces, as the building session stops on now. Checking what a value holds while making a game is
  the most common reason to show one.
- A format of their own, as Python's or Rust's debug printing. Something new to learn, which can't be pasted back.

**Why this won.** One rule for every value: what's shown is how it's written.

**Tankun's words.** "Just like how floats say `~`?", then, asked "Should lists, records, `Dict`s and `one_of`s show in
braces exactly as they'd be written in Seal?", the owner chose: "??" (yes)

## D101. `be.Bool` reads `"true"` or `"false"`; other new conversions are refused

2026-11-24.

**Chosen.**
- **`be.Bool(text)` gives `true` for `"true"` and `false` for `"false"`,** as a `Bool` shows (Unio D153), and fails
  with `not_a_bool` on any other text, handled as `be.Whole`'s failure is: `music_on = be.Bool(line) ?? true`.
- **Every other conversion not already decided is refused,** as text into a list or a record, or a `Whole` into a
  `Bool`.

**Weighed.**
- Reading `"yes"`, `"1"` and the like as `true`. A guess about what the text meant.
- Reading any value back from the text D100 shows. Possible later as one feature for every type, if a game needs it.

**Why this won.** A save file can hold a setting as `true` or `false`, and nothing is guessed.

**Tankun's words.** Asked "Should Seal add `be.Bool` from text, and refuse every other new conversion?", the owner
chose: "??" (yes)

## D102. Bare `end` again, with an optional, checked `\\` note naming the block

2026-11-25.

**Chosen.** Every block closes with a bare `end`, replacing D76's named closers, as D20 had it. After `\\`, an `end`
can carry a note naming its block, which the language server writes when it closes a block:

```
proc on_hit[amount: Whole]
    for each enemy in enemies
        if amount > enemy.hp
            show_msg("{enemy.name} falls")
        else
            show_msg("{enemy.name} is hit")
        end \\ if
    end \\ for
end \\ proc on_hit
```

- **The note is optional,** and coloured like a comment (D78), so closers stay quiet and `end` is always the real one.
- **The note is checked:** naming any block but the one the `end` closes is refused, so it can't go stale.
- **It names what the block is, with its name when it has one:** `\\ if`, `\\ while`, `\\ for`, `\\ match`,
  `\\ otherwise`, `\\ proc heal`, `\\ record Enemy`, `\\ one_of Event`, `\\ fam.proc spawn`, a family member's own
  name as `\\ spawn.boss`, and `\\ comment old_scoring`.
- `\\` outside text marks only this note; inside text it's still an escaped backslash (D92).

**Weighed.**
- Named closers, required (D76). Every block's closer written out, coloured like code, as loud as the code.
- The name after `end` with no mark, optional, as `end proc on_hit`, as the design session first proposed, and argued
  for over several turns before coming back to Tankun's version. Coloured like code, and not marked as optional.
- The note after `//`. `//` would then be a free comment everywhere but after `end`, where it would be checked, so an
  ordinary comment there, as `end // done with enemies`, would be refused.

**Why this won.** Short closers by default, a full note when wanted that can never lie, and the note looks like what it
is: something for the reader.

**Tankun's words.** "Show `end` vs `end details`", has code, "But the comment is checked, though", "Why `//`?", "Why did
you drop `\\`?", then, as their answer, quoting their first message back, has code.

## D103. A type's setting can be a top-level name made from a plain number

2026-11-25.

**Chosen.** A type's setting, such as `len.max:` or `len:`, takes a whole number written there, or a name made at the
script's top level from a whole number written there, with no `mut`:

```
max_items = 10
bag: List<Text>[len.max: max_items] = []
slots: Array<Text>[len: max_items] = [all: ""]
```

A `mut` name, a name made from a calculation or a call, and a name made inside a procedure are refused, since their
value isn't known while loading. The wording comes with a later batch.

**Weighed.**
- Only a number written there, as the building session stops on now. A limit used in several places would be written
  several times, and changed in several.

**Why this won.** A name made once from a number and never changed is known while loading as surely as the number,
and one name for a limit changes in one place.

**Tankun's words.** Asked "should a type's setting accept a top-level, non-`mut` name made from a plain number, as in
`len.max: max_items`?", the owner chose: "yea"

## D104. A default is any value written out plainly, never a calculation or a call

2026-11-25.

**Chosen.** A `defaults` line (D42) can give any type of value, written out plainly, or a name made from one at the top
level with no `mut`, as D103 has it for a type's setting:

```
proc spawn[kind: Text, color: Color, loot: List<Text>]
    labels [color as color, loot as loot]
    defaults [color: Color.red, loot: []]
end
```

A calculation or a call, as `defaults [loot: make_loot()]`, is refused, with its wording in a later batch.

**Weighed.**
- Only numbers, `Bool`s and text, as the building session stops on now. A `one_of` case or an empty list is as plain a
  value as a number.
- Calculations and calls too. When they'd run, and whether a default could differ between calls, would need rules of
  their own.

**Why this won.** A default is the same every time, and anything that can be written out plainly is.

**Tankun's words.** Asked "Should a default be any value written out plainly, but never a calculation or a call?",
the owner chose: "yea"

## D105. `at pos` and `at offset` are refused when going through a `Dict`

2026-11-25.

**Chosen.** Going through a `Dict`, `at key` gives each key (D33), and `at pos` and `at offset` are refused, with `at
key` as the fix. A count for showing is kept with a `mut` name.

**Weighed.**
- Allowing them, since a `Dict` keeps the order its keys were added in (D32). A `Dict` is never reached by position,
  as `stock[pos: 2]`, so a position from the loop could be used for nothing.

**Why this won.** How something is gone through matches how it's reached.

**Tankun's words.** Asked "Should `at pos` and `at offset` be refused on a `Dict`?", the owner chose: "yea"

## D106. A statement's later lines are indented one level deeper than its first

2026-11-25.

**Chosen.** When a statement carries on inside an open bracket (D8), its later lines are indented one level, 4 spaces,
deeper than its first line, checked like all indentation, with the corrected line as the fix:

```
spawn_enemy(kind: "goblin", x: 10, y: 20,
    health: 30, speed: 2)
```

**Weighed.**
- Lining later lines up under the open bracket, as the standard's own example had it. Renaming what's before the
  bracket means moving every later line to keep it lined up.
- Leaving it unchecked, as now. Scripts would each look their own way.

**Why this won.** One rule, the same 4 spaces as a block's body, as Rust's and Python's standard formatters do.

**Tankun's words.** Asked "Should a statement's later lines be indented one level deeper than its first?", the owner
chose: "yea"

## D107. Ranged numbers, and `abs`, `min`, `max` and `clamp`

2026-11-25.

**Chosen.**
- **A `Whole` or a `Float` can carry `min:` and `max:` as type settings,** as `gold: Whole[min: 0]`. They're hard
  limits: storing a value outside them stops the script with a failed check, and one Seal can see while loading is
  refused.
- **`min.clamp:` and `max.clamp:` pull a value back to the edge instead,** each time one is stored, as
  `hp: Whole[min.clamp: 0, max.clamp: 100]`, after which `hp = hp - 150` leaves `hp` at 0. Each edge chooses for
  itself, and either can be left off. Being type settings, they're numbers or top-level names made from one (D103).
- **`clamp(x, low:, high:)`** does the same to one value, for edges only known while running, as
  `hp = clamp(hp + heal, low: 0, high: max_hp)` when `max_hp` grows with level.
- **`abs(x)`, `min(a, b)` and `max(a, b)`** work on two `Whole`s or two `Float`s, never mixed (Unio D162). `abs` of the
  smallest `Whole` stops with the overflow check, as `-` does (Unio D142).

The wordings come with a later batch.

**Weighed.**
- `clamp` alone, as the design session first proposed. A limit fixed in the script would be repeated at every change,
  and nothing would catch a value going where it never should.
- `clamp(x, min:, max:)`, matching the ranged types, as the design session next proposed. Tankun: `len.max` already
  means going past is bad, and a clamp makes going past fine.
- `clamp(x, from:, to:)`, `range`'s labels, as first proposed. They read as moving from one to the other.
- `.loose` for a limit that pulls back, as Tankun first put it. It could read as not checked at all.
- `hp.clamp(…)` as a method. It reads as changing `hp`, the way `bag.add(…)` changes `bag`, while clamping usually
  happens to a value on its way somewhere.

**Why this won.** A hard limit and a pulled-back one are different promises, so they're written differently, and a
limit fixed in the script is written once, in the type.

**Tankun's words.** "yea. Maybe more types too? Since, I swear, they have some big ass money. `InfWhole`, `Deci`, and
stuff", "What's `clamp` for?", "What about `hp[min: 1, max: 100]`, not a realistic case, because these unlike clamps,
overflow is bad", "I meant those aswell", "Idk about that, I mean `len.max` is also treated as bad", "Or just
`max.loose`? Or something", "Why is it called `clamp`?", "Wait, why do we have `clamp()` still?", "Why not
`hp.clamp`?", "Does `from` and `to` make sensE?", "Why, again", "Why low & high", then, asked "Should Seal have ranged
types with `min:`/`max:` and `min.clamp:`/`max.clamp:`, `clamp(x, low:, high:)`, and `abs`, `min` and `max`?", the
owner chose: "yea"

## D108. `**` raises to a power

2026-11-25.

**Chosen.** `a ** b` raises `a` to the power `b`, binding tighter than `*` and `/`, as in maths, so `2 * x ** 2` is
`2 * (x ** 2)`. It's spaced as every operator is (D63).
- **`Whole ** Whole`** gives a `Whole`, stopping with the overflow check when it's too big; a negative power of a
  `Whole` is refused, since `2 ** -1` isn't whole.
- **`Float ** Float`** gives a `Float`, correctly rounded and checked as every float operation is (D18, D50).
- **The two don't mix,** as everywhere (Unio D162): `be.Float(n) ** 0.5`.

Its wordings come with a later batch.

**Weighed.**
- `^`, as school maths, calculators, Lua and Luau write it, which the design session recommended for Seal's users
  coming from Lua. In C, JavaScript, Python and Rust, `^` is something else (XOR).
- `pow(a, b)`, a procedure, as in C.

**Why this won.** Tankun's choice: `**` explains itself, and with Seal's spacing it's always written `x ** 2`.

**Tankun's words.** "Why not just `**`? We're not gonna allow `x**`, so it wouldn't look like a stroke", "It's self
explainatory, though?", then, asked "Should Seal write powers with `^`?", the owner chose: "No, use `**`"

## D109. `sin`, `cos`, `tan`, `asin`, `acos` and `atan2`, in radians, with `.degrees` versions, and `pi`

2026-11-25.

**Chosen.**
- **`sin`, `cos` and `tan` take a `Float` angle in radians;** `asin` and `acos` give one for a value from -1 to 1,
  stopping with a failed check outside it; `atan2(y:, x:)` gives the angle of the point `x`, `y`.
- **Each has a `.degrees` version,** a narrowing, taking or giving degrees, a full turn being 360:
  `sin.degrees(90.0)` is `1.0`, and `atan2.degrees(y: 1.0, x: 0.0)` is `90.0`.
- **`pi` is a built-in `Float`,** the one nearest π.
- **Each is correctly rounded and the same on every machine** (D50), checked against Ralph's floats.

```
dx = cos(angle) * speed
aim = atan2.degrees(y: target_y - y, x: target_x - x)
```

The wordings come with a later batch.

**Weighed.**
- Degrees as the plain form, with `.radians` versions. Friendlier at first, but every formula copied from a tutorial
  or another engine assumes radians and would quietly break.
- A label for the unit, as `sin(radians: a)` (D55). The first argument has no label by convention.

**Why this won.** Radians are what the maths and every other library use, and degrees, the human convention, are a
kind of the same operation.

**Tankun's words.** "So, default is radians?", "Wait a fucking minute, don't you have a script for that? Quote", then,
asked "Should Seal have `sin`, `cos`, `tan`, `asin`, `acos` and `atan2` in radians, `.degrees` versions of each, and
`pi`?", the owner chose: "yea"

## D110. Handlers live in a `group.handler`, reached with `::`, and a group can say which procedures it handles

2026-11-25.

**Chosen.** Following Cyborg D488, a member picked out of a named set is reached with `::`, while `.` keeps narrowings,
fields and `one_of` cases. Handlers, which are members of a set and not kinds of a word, move out of families:

```
group.handler level_failed
    handles [load_level, load_save]
    ::start_over
        case not_found
            instead 1
        case broken
            instead 1
    end
end

level = load_level("cave") ?? ::start_over
level = load_save("slot1") ?? level_failed::start_over
```

- **`group.handler name … end`** declares a handler group, "a handler kind of group", replacing D97's `fam.handler`.
  `fam` keeps one meaning: a word and its kinds, joined by `.`. Other named sets, if Seal gets them, would be
  `group.` too.
- **Members are declared as `::name`,** the mark they're reached with.
- **`handles [ … ]`,** an optional first line, lists the procedures the group is for. Right after a call to one of
  them, `::start_over` reaches into the groups that handle it, with `?? ::*give_up` for one that can jump, and
  `include ::start_over` in an `otherwise` block. If two such groups both have that member, the short form is refused
  there.
- **The full form, `level_failed::start_over`, works anywhere,** including for a group with no `handles` line.

The wordings come with a later batch.

**Weighed.**
- `level_failed.start_over`, as D97 and Unio have it. Tankun: a narrowing goes into the word to make a different kind
  of it, while a handler group's member goes to the right, a different thing, which Cyborg D488 settled.
- No group in front, as `?? start_over`. Two groups could each have one, and it would read like a fallback value.
- A group named after its procedure, as `load_level::start_over`, as the design session proposed. One spelling would
  mean two things, which nothing else in Seal does (D71).
- `handlers level_failed`, as the design session proposed. `group.handler` mirrors `fam.proc` and extends to other
  named sets.
- `kit`, `set`, `space`, `bundle` or `pack` in place of `group`. `group` is plain English and collides with nothing
  planned.

**Why this won.** Each mark means one thing: `.` goes into a word, `::` reaches into a named set, and a call's own
procedure says which set a short `::` name means.

**Tankun's words.** "Before that. I think the diff between `fam.proc` and `fam.handler` is weird. Well, look.
proc fam could make `println.stored`, while handler fam could make `level_failed.start_over`.
it can't really make a more specific `start_over`, rather just reaching into `level_failed` like a path, or some shit",
"Cyborg has this too, and since its fam model is what I want it to be. You should msg Cyborg dclaude about this, and
that I will be dealing with it there for a bit", "Why not just no mark>", "Then, you should probably shorten it to
`level::start_over`", "Or have a way to tie handler groups to proc? So, you could say shit like `::start_over` because
of the left", "Ohhh, Idk about name being like that. Since, we usually don't do that, right?", "Can we write
::start_over inside too? just feel like it", "Woah, `fam`? Maybe we should use another name? Or would we just make
`fam` very diverse", "What about `group.`?", "Any other names for group?", then, asked "Should handler groups be
`group.handler level_failed`, with `handles [ … ]`, members declared as `::start_over`, and `::start_over` usable after
a handled procedure's call?", the owner chose: "yea"
