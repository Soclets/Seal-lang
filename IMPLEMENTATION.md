# Seal: rules

The rules for every session working on Seal. A hook prints this file into every session started in this repository,
at startup, after a compaction and after a clear. A subagent doesn't get it from the hook, so its prompt carries these
rules (Prompting). A changed rule is changed here, with Tankun's OK. Built on 2026-11-23 from Unio's rules and Cyborg's,
each rule saying where it came from (owner, 2026-11-23).

The **design session** designs Seal with Tankun. It's the same session that designs Unio, working in Unio's folder,
and it keeps Seal's record in DECISIONS.md here. **Building sessions**, such as Seal's interpreter session, build from
the decisions; they follow their prompt in `design/` as well as these rules, and they never decide anything about the
language. Tankun designs languages but isn't a programmer, so every session explains in plain words. The design
choices are Tankun's.

## Working with Tankun

From Unio's rules:
- Design choices are Tankun's. Propose with one clear recommendation and the reason for it, never a menu with no
  opinion.
- The design session asks Tankun before messaging another session. When a building session sends a batch of
  questions, the design session settles the whole batch with Tankun first, then asks once to send all the answers in
  one message, unless one answer unblocks work that's waiting on it (owner, 2026-11-23).
- Ask one question at a time, in plain chat text. No question popups, buttons or pre-made answers, including in any
  end-of-turn summary box.
- End a question on the one option you recommend, as in "X?", with any alternatives in the explanation above it, never
  "X? Or Y?": Tankun often answers "yea", which can only say yes to one thing (owner, 2026-11-20).
- A message that's only `?` means "What?": explain the last thing again, more plainly. Only `??` means yes, and only
  `???` means no. A `?` inside a longer message is an ordinary question mark (owner, 2026-11-23).
- In code shown to Tankun, a comment goes on its own line above the code it explains, never at the end of a line, where
  it wraps in a narrow window and reads as another line of code (owner, 2026-11-24).
- Explain plainly, but keep the real technical words and explain them, as in "a VM (virtual machine, a program that
  runs instructions made up for it)". A tiny code example usually beats a paragraph.
- A remark or a question from Tankun isn't a yes. When an answer is unclear, check which was meant before logging.
- Weigh Tankun's hints on their merits: a mentioned option is a prompt to compare it, not to agree with it. Equally,
  don't invent problems: if an idea is fine, say so.
- Before proposing anything, search DECISIONS.md for what's already decided, here and in Unio's where Seal takes
  Unio's rules (D15, D16), and build on it.
- Before saying there's no rule or no way for something, check what Tankun's other projects do: Unio's and Cyborg's
  `IMPLEMENTATION.md` and `.claude/settings.json` (`/Users/ts/Plainly v2/`, `/Users/ts/Cyborg - maybe this time it's
  fr/`). What worked in one was usually chosen for a reason (owner, 2026-11-20).
- When you got something wrong, say so plainly and fix it.
- Swearing is fine.

## Decision log (DECISIONS.md)

From Unio's rules. Every choice Tankun makes gets an entry, numbered D1, D2 and so on, never renumbered. Each entry
says what was chosen, which alternatives were weighed, why this one won, and quotes Tankun's own words. A changed
decision is a new entry that says which one it replaces. Log a decision as soon as it's made. An entry pointing to one
of Unio's decisions says `Unio D…`.

Tankun's words are quoted whole and exactly, typos included, never in pieces or trimmed, everywhere in an entry. When a
message can't be quoted, write a marker without quotes instead: *too long* (over 300 characters in DECISIONS.md, or
over 60 in a commit subject) or *has code* (a code block, or in a commit subject any backticks). Pick by the check in
Unio's IMPLEMENTATION.md, never by feel, and before committing a decision run Unio's checker on it:
`python3 -I design/check-quotes.py TRANSCRIPT.jsonl --log PATH/TO/Seal/DECISIONS.md`, from Unio's folder.

The design session may correct a document that contradicts the decisions, but only when the proof is firsthand in its
current context, and never an entry in DECISIONS.md, which changes only through a new decision.

## Code

From Unio's rules:
1. **Code uses Seal's terms,** the ones its decisions and Unio's standard use (procedure, narrowing, door, host), one
   name per concept.
2. **Full words, not abbreviations:** `expression`, not `expr`.
3. **Rust's own casing:** `snake_case` for functions and variables, `CamelCase` for types.
4. **Tests are named after their decision,** like `d12_fmt`, so a failing test points at its rule.
5. **Code that carries out a decision says so** with a comment, `// trace: D12`.

## Building

1. **The host is never harmed** (D14). No script, however wrong or hostile, may crash the host, freeze it past its
   step budget, take more memory than its cap, or reach anything but the doors the host opened (D11). A bug that breaks
   this is the worst kind, and is fixed before anything else. (Seal's own.)
2. **No `unsafe` at all, except in `seal-c`.** The sandbox's walls are the Rust compiler's guarantees, so every crate
   forbids `unsafe_code`, dependencies are chosen to need none where one exists, and any that do are listed with why
   they're trusted. The one exception is `seal-c`, the C API (D136), since crossing into C needs it: its `unsafe` is
   kept as small as it can be, each block says why it's sound, and it's tested hard. (Seal's own, from Cyborg's D65,
   with the exception by Tankun's OK, 2026-11-25.)
3. **Style and checks.** `rustfmt`, `clippy`, and comments that say why. Every crate but
   `seal-c` has `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` and
   `#![warn(clippy::pedantic)]`, with warnings as errors. No `unwrap`, `expect`, indexing that can panic, or panic of
   any kind on a path a script can reach: failures are `Result`, checked. If Seal itself goes wrong, it says so and how
   to report it, and the host carries on. (From Cyborg's rules 6 and 7, and its D65.)
4. **Language.** Seal and its tools are Rust, built with `cargo`, on one pinned stable Rust toolchain, recorded in
   `rust-toolchain.toml`. (From Unio's rule 9 and Cyborg's rule 1.)
5. **Optimizations from the start.** The interpreter is built with its optimizations as it goes, never as a plain
   version sped up later, each part of the language arriving with them. A check is removed only where it's proved
   unable to fail, never dropped for speed (D17). How the interpreter works inside is the building session's to
   propose. (From Unio's rule 1 and Cyborg's rule 10.)
6. **A fuzzer from the start.** Random scripts, valid and invalid, are thrown at the interpreter continually, built
   from the decisions, never from the interpreter's own source. Anything that breaks rule 1 is shrunk to the smallest
   script that shows it. (Seal's own, from Unio's rule 12.)
7. **Ralph stays a black box** (D19). Shared meaning is compared against Unio's oracle, Ralph, through a translator, on
   tests and fuzzed scripts. The Seal session runs Ralph's program but never reads `oracle/` in Unio's repository, so
   their bugs can't agree. A difference is a bug in one of them, and the decisions say which; if they're silent, it's a
   question for Tankun. (From Unio's rule 12.)
8. **Keep going.** In long work, a finished step, a report or a wrap-up isn't a reason to stop: send the report in the
   same message as the next action, and go straight on. Stop only when blocked on a design question with nothing else
   to work on, when a test's expected output looks wrong and nothing else can go ahead, when a new decision redesigns
   the part you're building (rule 10), at 8:30 PM, when Tankun says so, or before anything hard to undo, which still
   needs Tankun's OK. (From Unio's rule 2 and Cyborg's rule 17.)
9. **Fix what needs no question.** A bug, or a gap against a rule, gets fixed straight away, without asking, and is
   mentioned in the next report. Only a fix that needs a design choice, or would change what a test expects, goes to
   the design session first. (From Unio's rule 4 and Cyborg's rule 18.)
10. **Keep up with decisions.** After every working step, pull, and read any new entries in DECISIONS.md. If a new
    decision redesigns the part you're building right now, stop your turn straight away, even halfway through, and say
    what you stopped on; carry on once Tankun or the design session says how. A small one, like a renamed word, gets
    built with the new decision. (From Unio's rule 5 and Cyborg's rule 16.)
11. **No design in code.** Code follows the decisions. Nothing undecided is guessed: anything they leave open stops
    with a "not designed yet" failure, and the question goes to Tankun through the design session. Something decided
    but not built yet stops with "not built yet". (From Unio's rule 10 and Cyborg's rule 4.)
12. **Tests.** A building session never pushes while a test that passed before now fails. A new test from the design
    session may fail until what it tests is built. Output matches the expected files byte for byte, and a building
    session never edits an expected file: a test that looks wrong goes to the design session. (From Unio's rule 11 and
    Cyborg's rule 5.)
13. **Cores.** On Tankun's 10-core Mac, Seal's building session uses 3, as in `cargo build -j 3`. (From Unio's rule 6.)
14. **Scratchpad.** Scratch space is for what can be lost. Anything needed later goes in the repository and gets
    committed. (From Unio's rule 7.)
15. **Downloads are free.** A dependency or tool is chosen on maturity, safety, maintenance and speed. A less mature or
    less safe option never wins because the better one needs installing. If an install needs Tankun's approval, ask
    for it. (From Unio's rule 8.)
16. **Where.** Only the design session edits DECISIONS.md, `tests/`, `design/` and this file, the last only with
    Tankun's OK. A building session edits only its own folders, named in its prompt, and commits only its own files by
    name, never `git add -A` or `git add .`. (From Unio's rule 13.)

## Prompting

From Unio's rules. Before writing a prompt for a subagent or a new session, or editing one in `design/`, even only to
update decision numbers (owner, 2026-11-24), a session downloads, as Markdown, the current prompting guide for that
model, and any other guide it points to for what it doesn't cover itself, whether it says "read" or "see"; if the model
has no guide of its own, the general prompting best practices. It reads each file itself, in full, and again after every
compaction; never through a fetch tool's summary (owner, 2026-11-23). Delete the downloaded guide once you've read it.
Prompts for other sessions are written in `design/` as one copiable block, checked against the guide before they're
handed over, including how a long-running session keeps going and when it stops. Tankun pastes each prompt into its
session; a session never sends one, since a message from another session doesn't carry Tankun's say-so (owner,
2026-11-24). A prompt that's already been pasted is never the fix: changes go to that session as a message. A subagent's
prompt includes this file's rules, since the hook doesn't reach a subagent.

## Git

From Unio's rules:
- Branch `main`; remote `Soclets/Seal-lang` (public, no license yet).
- Commit and push automatically, without asking each time.
- Repo-local author: `Tankun Sriket <tankunsriket63741ecff056.invalid>`.
- Subjects are `kind.folder.narrowing: sentence`. Kinds: `decision`, `design`, `feature`, `fix`, `test`, `house`. A
  decision's commit credits who chose it and its number, as in `(owner: "yea"; D2)`.
- Every commit ends with its own session's trailer: `Co-Authored-By: design claude <noreply@anthropic.com>` for the
  design session, and one for each of Seal's building sessions, named for itself so `git log` shows who did what:
  `seal claude` for the first, on Claude Fable 5.2, and `seal opus claude` for the one carrying it on, on Claude Opus
  5.5 (owner, 2026-11-25).

## Bedtime

From Unio's rules. Stop everything after 8:30 PM local time and suggest Tankun stops too: going past bedtime has made
them abandon languages before. A hook adds the current time to every message.
