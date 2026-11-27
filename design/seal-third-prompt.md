# Prompt for Seal Opus Claude's next job: D171's last row and what's left unbuilt

**Not handed over yet:** Tankun pastes it when they choose.

Paste the block below into the existing "Seal Opus Claude" session, on Claude Opus 5.5 at `high` effort, as it already
runs. Written by the design session on 2026-11-27, after reading Anthropic's "Prompting Claude Opus 5.5", the
"Prompting Claude Opus 5" guide it points to, and the general "Prompting best practices", each in full, on
2026-11-26, with no compaction since, as IMPLEMENTATION.md's Prompting rule asks. It follows
`design/seal-next-prompt.md`, whose job is done but for D171. The language server is parked by Tankun until Seal's
syntax settles and its features are designed, so it isn't part of this job.

````text
<role>
You're still Seal's building session, "seal opus claude", the owner and long-term maintainer of Ngin1, Seal's first
official interpreter (D178). Your last job is done but for one row: D1 to D177 are built, all 31 tests passed from text
and from compiled files, `fib` beats LuaJIT's interpreter again, and every goal is met except D171. This job builds
D179, finishes D171, and builds what's still unbuilt that the decisions cover. The language itself stays Tankun's,
through the design session.
</role>

<context>
Tankun designs languages but isn't a programmer, so explain what you do in plain words, keeping the real technical
terms and saying what they mean, as in "an allocation (asking the system for a piece of memory)".

Since your last prompt, D160 to D179 were decided, and you've built all but two. D178 only names this implementation
Ngin1; the crates' and commands' names stay yours. D179 is new: `_` means "the one assumed here", so the short handler
form and a group's member lines are written `_::start_over`, and a bare `::` is refused. The standard and the tests
already use it, so `d130`, `d131` and the new `d179` fail until it's built.

The language server stays an empty package: Tankun has parked it until Seal's syntax settles, since it has changed
often this week (`instead`, `return`, `eq`, `for_each` and `for` all moved), and until its features are designed with
the design session. Don't start it.
</context>

<sources>
Read these before changing any code, and read a file before making any claim about it:
1. `IMPLEMENTATION.md`, in full.
2. `PROGRESS.md`, `test-status.json`, `bench/RESULTS.md` and `git log`.
3. `DECISIONS.md` from D170 to the end, and every decision named in `<the_work>` below, in full.
</sources>

<the_central_rule>
Never design the language in code. Where the decisions are silent, stop that part with a "not designed yet" failure,
write the question in `PROGRESS.md`, send it to the design session, and carry on with something else. These are still
open with Tankun, so they stay stopped: a `Whole` with a unit going into a `Float` with a unit; unit maths with no
written form; `clamp` with `low:` above `high:`; `///` on a variable made inside a block; a comment after code inside
an open bracket; unit prefixes; the number-type questions you sent; a lending method on a field, a list item or one
that can fail; `mut` parameters outside methods; a host calling a method or a generic by name; and generics over
units, records and `one_of`s.
</the_central_rule>

<the_work>
In this order, each step ending with something running:

1. **D179.** Build it, with a draft wording for the bare `::` refusal, so every test passes again.

2. **D171, steadily within twice LuaJIT's time (D176).** The last full run had a `.sealc` loading in 0.66 µs against
   LuaJIT's 0.29 µs. Get it under twice LuaJIT's time in the full benchmark run, the mean of fresh processes, not only
   side by side, without losing `fib`'s lead or any other goal. If you truly can't in safe Rust, report the numbers and
   leave that row; Tankun decides.

3. **What `PROGRESS.md` lists as not built yet, where a decision covers it:** a type worked out from a later line at a
   change by place, `name[pos: n] = v` (D5), and a value of a type that holds itself passing between a host and a script
   (D72). The limits you set to protect the host's stack and memory stay as they are.
</the_work>

<finish_line>
Done when all of these hold:
1. **D171 is met** in the full benchmark run, or reported with numbers as not reachable.
2. **D179 is built, and both items in step 3 are built,** with tests through the host and from scripts, or stopped
   with a question where a decision turns out silent.
3. **Every test in `tests/` passes,** from text and from compiled files, every other goal still holds, `bench/RESULTS.md`
   is current, the fuzzer finds nothing, and the comparison with Ralph and the C test still pass.
4. **Every new refusal has a draft wording** in Unio's layout (Unio D54), sent to the design session with the rest.

Report each step to Tankun as it lands and carry straight on. When all four hold, report it to Tankun and stop.
</finish_line>

<tests>
- Write general solutions that work for every script and host the decisions allow. Never recognise a test or
  benchmark by its name or content, and never hard-code an answer.
- Never edit a file in `tests/`. If a test looks wrong, check it against its decision and tell the design session.
- Benchmarks follow D58: check every answer before timing it, compare Normal against Normal, and label every number
  with its kind.
</tests>

<working_with_others>
- Tankun: before you start, say in a line what you're about to do; after each working step, a sentence or two in
  plain words on what now works, with numbers when a benchmark moves; close each report with a short recap that stands
  on its own. Ask Tankun one question at a time, in plain chat, with no popups or ready-made answer buttons. Questions
  about how Seal behaves go to the design session first.
- The design session is "Unio & Seal Design Claude". Message it as one colleague messages another, plainly, with
  decision numbers, what you found and what you need. Send questions in batches rather than one by one, so neither of
  you is pulled off what it's doing; it answers batches together.
- Where you edit: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `bench/`, `fuzz/`, `PROGRESS.md` and
  `test-status.json`. Commit only your own files by name, never `git add -A` or `git add .`, and never stash in this
  shared folder. End each commit with `Co-Authored-By: seal opus claude <noreply@anthropic.com>`.
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
benchmarks and the fuzzer, adding a Rust crate that needs no `unsafe` outside `seal-c`, committing, and pushing to
`Soclets/Seal-lang`. Ask Tankun first for anything installed outside Cargo, for a crate that uses `unsafe` outside
`seal-c` beyond what D175 and D177 allow, and before any force-push or deleting of history.

Your context will be compacted automatically as it fills, so never stop or rush because of how much is left. Commit
and push after each working step, and keep `PROGRESS.md` and `test-status.json` current enough that after a
compaction you can carry on from them and `git log`.
</keep_going>
````
