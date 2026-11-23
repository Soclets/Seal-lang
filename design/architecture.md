# Seal's architecture

How Seal is built, as decided so far. The language itself is in `standard/`; this is the shape of the implementation,
for the building session. The details inside each part are the building session's to propose (IMPLEMENTATION.md,
Building rule 5).

- **Language:** Rust, with no `unsafe` (D1, IMPLEMENTATION.md, Building rules 2 to 4).
- **Memory:** every value owned by one holder, counted holders and copy-on-write for big values, no garbage collector
  (D15).
- **Running a script** (D46):

```
load:  text → syntax tree → checked, types worked out → bytecode
run:   a register VM runs the bytecode, counting steps
```

  No JIT. The instruction set, its names and the optimizations are the building session's to propose (D17, D46).
- **Checks:** at load time and while running, always on, with no build modes (D17, D18).
- **Parts** (D47), one Rust workspace:

| Part | What it is |
|---|---|
| `seal` | the library a host embeds, and the only part hosts depend on |
| `seal` command | checks a script, and runs it with a test host whose stand-in doors use the terminal |
| language server | errors as you type, worked-out types, colours (D7) |
| fuzzer | random scripts against the library (Building rule 6) |
| Unio translator | for checking against Ralph (D19) |

  The four tools are built on the library, never copying its insides. How the library is split inside is the building
  session's to propose.

**Not decided:** the host's side of embedding; isolation between scripts and threads;
determinism across machines.
