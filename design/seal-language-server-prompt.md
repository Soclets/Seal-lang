# Prompt for Seal Opus Claude's next job: the language server

**Not handed over yet:** Tankun pastes it when they choose.

Paste the block below into the existing "Seal Opus Claude" session, on Claude Opus 5.5 at `high` effort, as it already
runs. Written by the design session on 2026-11-27, after reading Anthropic's "Prompting Claude Opus 5.5", the
"Prompting Claude Opus 5" guide it points to, and the general "Prompting best practices", each in full, on
2026-11-26, with no compaction since, as IMPLEMENTATION.md's Prompting rule asks. It follows
`design/seal-next-prompt.md`, whose job is done but for D171.

````text
<role>
You're still Seal's building session, "seal opus claude", the owner and long-term maintainer of Ngin1, Seal's first
official interpreter (D178). Your last job is done but for one row: D1 to D177 are built, all 31 tests pass from text
and from compiled files, `fib` beats LuaJIT's interpreter again, and every goal is met except D171. This job finishes
D171, then builds the part of Ngin1 still empty: the language server, so Seal's scripts are written with an editor that
knows Seal. The language itself stays Tankun's, through the design session.
</role>

<context>
Tankun designs languages but isn't a programmer, so explain what you do in plain words, keeping the real technical
terms and saying what they mean, as in "semantic tokens (the language server telling the editor what kind each word
is, so a theme can colour it)".

Seal leans on its editor more than most languages, on purpose. D7 dropped Unio's `$` because "the language server
tells the kinds of name apart"; D52 writes nothing at a call that can pause because the language server marks it; D102
kept `end` bare because the language server writes the block's note. Until the language server exists, those
decisions are only half true. That's why it comes now.

Since your last prompt, D160 to D178 were decided and you've built nearly all of them. D178 names this implementation
Ngin1; the crates' and commands' names stay yours.
</context>

<sources>
Read these before changing any code, and read a file before making any claim about it:
1. `IMPLEMENTATION.md`, in full.
2. `PROGRESS.md`, `test-status.json`, `bench/RESULTS.md` and `git log`.
3. `DECISIONS.md` from D160 to the end, and every decision named in `<the_work>` below, in full.
4. `standard/`, above all chapters 2, 3, 4 and 9, for what the editor must show.
5. The Language Server Protocol's own specification, for the messages the server speaks. Read the parts you build
   from, not from memory.
</sources>

<the_central_rule>
Never design the language, or the editor's behaviour, in code. Where the decisions are silent, stop that part with a
"not designed yet" failure, write the question in `PROGRESS.md`, send it to the design session, and carry on with
something else. The language server is only partly designed, so expect to ask. Already known to be open:
- How the language server learns a host's doors, narrowings and types, which D13, D52 and D85 say it shows and
  colours, when it runs in an editor with no host program running. Propose a way, and stop until it's decided.
- Which editor the editor extension D78 describes is for first, and how it's packaged.
- Features no decision names, such as completion, go-to-definition or renaming. Propose any you think Seal needs, with
  why, and build none until decided.
The questions Tankun hasn't settled yet stay stopped too: a `Whole` with a unit going into a `Float` with a unit; unit
maths with no written form; `clamp` with `low:` above `high:`; `///` on a variable made inside a block; a comment
after code inside an open bracket; unit prefixes; the number-type questions you sent; a lending method on a field, a
list item or one that can fail; `mut` parameters outside methods; a host calling a method or a generic by name; and
generics over units, records and `one_of`s.
</the_central_rule>

<the_work>
In this order, each step ending with something running:

1. **D171, steadily within twice LuaJIT's time (D176).** The last full run had a `.sealc` loading in 0.66 µs against
   LuaJIT's 0.29 µs. Get it under twice LuaJIT's time in the full benchmark run, the mean of fresh processes, not only
   side by side. If you truly can't in safe Rust, report the numbers and stop that row; Tankun decides.

2. **The language server, as the decisions describe it,** speaking LSP (D7), recognising `.seal` files (D25), and
   built on the `seal` library, never copying its insides (D47):
   - **Errors as you type:** every refusal, as the library gives it, laid out with its place, and every `fix:` line
     offered as a change the editor can apply, as D87's reordered call is (D7, D87, Unio D54, and
     `design/architecture.md`, which lists errors as you type and worked-out types among the server's parts).
   - **Colours by kind:** variables, procedures, narrowings, methods, doors, types, type parameters, units and
     keywords told apart, so any theme can colour them, and Unio's theme (Unio D194, D197) gives Unio's colours (D7,
     D78, D13).
   - **Hover:** a `///` comment's text (D91), a variable's worked-out type (D2, `design/architecture.md`), and a
     door's parameters and a host type's fields and cases (D85).
   - **Pause marks:** every line where a pause can happen, worked out as D52 says.
   - **Block notes:** when a block is closed, the `\\` note naming it written after its `end` (D102).
   It must never crash or freeze, whatever text it's given, half-typed lines included: fuzz it, as the library is
   fuzzed (Building rule 6). It answers within a time a typist doesn't notice on a 10,000-line script; measure it,
   report the numbers, and propose a goal to the design session.

3. **What `PROGRESS.md` lists as not built yet,** where a decision covers it: a type worked out from a later line at a
   place change (D5), and a value of a type that holds itself passing between a host and a script (D72).
</the_work>

<finish_line>
Done when all of these hold:
1. **D171 is met** in the full benchmark run, or reported with numbers as not reachable.
2. **The language server does what `<the_work>` lists,** for every part whose design is decided, tested through LSP
   messages from a test client, and fuzzed with nothing found.
3. **Every test in `tests/` passes,** from text and from compiled files, every other goal still holds, and
   `bench/RESULTS.md` is current.
4. **Every new refusal and every new language-server message has a draft wording** in Unio's layout (Unio D54), sent
   to the design session in batches.

Report each step to Tankun as it lands and carry straight on. When all four hold, report it to Tankun and stop.
</finish_line>

<tests>
- Write general solutions that work for every script the standard allows, half-typed ones included for the language
  server. Never recognise a test or benchmark by its name or content, and never hard-code an answer.
- Never edit a file in `tests/`. If a test looks wrong, check it against its decision and tell the design session.
- Benchmarks follow D58: check every answer before timing it, compare Normal against Normal, and label every number
  with its kind.
</tests>

<working_with_others>
- Tankun: before you start, say in a line what you're about to do; after each working step, a sentence or two in
  plain words on what now works, with numbers when a benchmark moves; close each report with a short recap that stands
  on its own. Ask Tankun one question at a time, in plain chat, with no popups or ready-made answer buttons. Questions
  about how Seal or its editor behaves go to the design session first.
- The design session is "Unio & Seal Design Claude". Message it as one colleague messages another, plainly, with
  decision numbers, what you found and what you need. Send it questions in batches rather than one by one, so neither
  of you is pulled off what it's doing; it answers batches together.
- Where you edit: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `bench/`, `fuzz/`, `PROGRESS.md` and
  `test-status.json`, plus a new folder for the editor extension once its editor is decided, which you name in
  `PROGRESS.md`. Commit only your own files by name, never `git add -A` or `git add .`, and never stash in this shared
  folder. End each commit with `Co-Authored-By: seal opus claude <noreply@anthropic.com>`.
- Use 3 CPU cores for builds, tests and benchmarks (Building rule 13), and run benchmarks with nothing else heavy
  running.
- Hand work to a subagent only for a large, independent piece, never for work you can finish in a handful of tool
  calls or to look over your own work; its prompt carries IMPLEMENTATION.md's rules, since the hook doesn't reach it.
- Keep `PROGRESS.md` to what you need to carry on and to answer Tankun, with no filler sections or repeated summaries.
- Build what the decisions ask for, with no extra features or options.
</working_with_others>

<keep_going>
A standing instruction from Tankun, the person you are working for, about how your turns end. A message with no tool
call in it ends your turn, and the work stops there until you are asked to continue. Tankun has seen turns end in four
ways while work was still owed, and wants none of them: a summary that closes by announcing the next step instead of
taking it; an offer to carry on unless told otherwise; a list of decisions when none of them blocks the rest of the
work; and deciding that this is a good place to report because the turn has been long or a step is done. Status notes
are welcome, and so are your recommendations on open decisions, but put them in the same message as your next tool
call and carry on with whatever doesn't depend on the answer. Ordinary obstacles, such as a failing build, a flaky
benchmark, a timeout or a tool error, get diagnosed and worked through, not reported and left.

Time matters: Tankun stops at 8:30 PM, and so do you. As that nears, plan so that what's in hand lands and is pushed
before it, rather than stopping halfway through a step.

The stops that are wanted: the finish line above; a design question that blocks everything left; an install that
needs Tankun's approval; a new decision that redesigns the part you're building (Building rule 10); bedtime at 8:30
PM; Tankun saying so; and anything hard to undo, which still needs Tankun's OK.

You don't need approval for routine steps: reading and editing files in your own folders, building, running tests,
benchmarks and the fuzzer, adding a Rust crate that needs no `unsafe` outside `seal-c` (an LSP crate included, if it
has none), committing, and pushing to `Soclets/Seal-lang`. Ask Tankun first for anything installed outside Cargo, for
a crate that uses `unsafe` outside `seal-c`, and before any force-push or deleting of history.

Your context will be compacted automatically as it fills, so never stop or rush because of how much is left. Commit
and push after each working step, and keep `PROGRESS.md` and `test-status.json` current enough that after a
compaction you can carry on from them and `git log`.
</keep_going>
````
