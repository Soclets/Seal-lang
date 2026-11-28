# Wax's architecture

This is Ngin1, Wax's first official interpreter (D178).

How Wax is built, as decided so far. The language itself is in `standard/`; this is the shape of the implementation,
for the building session. The details inside each part are the building session's to propose (IMPLEMENTATION.md,
Building rule 5).

- **Language:** Rust, with no `unsafe` outside `wax-c` (D1, D136, IMPLEMENTATION.md, Building rules 2 to 4).
- **Memory:** every value owned by one holder, counted holders and copy-on-write for big values, no garbage collector
  (D15).
- **Running a script** (D46):

```
load:  text → syntax tree → checked, types worked out → bytecode
run:   a register VM runs the bytecode, counting steps
```

  No JIT. The instruction set, its names and the optimizations are the building session's to propose (D17, D46).
- **Ahead of time** (D141): `wax build` can compile a script to a bytecode file once, when a game is built; loading
  one always verifies it first, so a damaged or out-of-date file is refused, and a host can require an ML-DSA
  signature from its own key, so any change is refused (D169). Built after the current work.
- **Checks:** at load time and while running, always on, with no build modes (D17, D18).
- **Parts** (D47), one Rust workspace:

| Part | What it is |
|---|---|
| `wax-lang` (`wax` in Rust) | the library a host embeds, and the only part hosts depend on |
| `wax` command | checks a script, and runs it with a test host whose stand-in doors use the terminal |
| language server | errors as you type, worked-out types, colours (D7) |
| fuzzer | random scripts against the library (Building rule 6) |
| Unio translator | for checking against Ralph (D19) |
| `wax-c` | the C API, for hosts not written in Rust, a thin layer over the library and the only part with `unsafe`; built after the current work (D136) |

  The four tools are built on the library, never copying its insides. How the library is split inside is the building
  session's to propose.

- **The host's side** (D48): doors are typed Rust closures; a host chooses each script's doors and limits when loading
  it; loading checks and runs the top level, returning a handle; calls in are checked by name and types; refusals,
  failed checks and limits come back as errors, never crashes; records and `one_of` cross by `#[derive(Wax)]`.

```rust
let mut goblin_script = Script::load("goblin.wax", &source, &doors, limits)?;
goblin_script.call("on_hit", (10,))?;
```

- **Isolation and threads** (D49): each loaded script is fully separate, runs on one thread at a time and can move
  between threads; scripts talk only through the host; a door can pause a script, and the host resumes it later; no
  parallelism inside one script for now.

- **Determinism** (D50): a script, given the same calls and door answers, gives the same results on every machine, bit
  for bit. Maths functions added later are correctly rounded, ported to Rust with no `unsafe` after CORE-MATH's
  algorithms, and checked against Ralph.
- **Performance goals** (D57): faster than Luau's interpreter on every benchmark, faster than V8 `--jitless` on `fib` and
  n-body and within twice it on word count and text building, loading 10,000 lines in under 100 ms, kilobytes per
  loaded script, cheap pausing, all with every check on, on seven fixed benchmarks.
- **Benchmark rules** (D58): Cyborg's kinds (Purpose, Normal, `Ceil.source`, `Ceil.machine`, `Ceil.program`), every
  benchmark in every kind, six things reported with correctness first, D57's goals judged Normal against Normal, and a
  Purpose benchmark for the cost of checks.
  Binary-trees joins the benchmarks now that a type can hold itself (D72).

- **An oracle,** if Wax ever has one, not soon: proven to match the standard entirely, trusting only the third-party
  tools it's built on, such as Rocq or SPARK at Platinum (D138). Until then, the shared parts are checked against
  Ralph (D19).
