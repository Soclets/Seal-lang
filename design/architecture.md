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

**Not decided:** how the code is split into parts; the host's side of embedding; isolation between scripts and threads;
determinism across machines.
