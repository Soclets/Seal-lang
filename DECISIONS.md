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
