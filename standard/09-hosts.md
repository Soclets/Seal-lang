# 9. Hosts

A host is the program that embeds Seal: a Rust program, or one in any language that can call C, through Seal's C API
(D1, D136). This chapter says what a script sees of it, which is the same either way; how a host's code calls Seal is
the building session's to propose.

## Loading and calling

- **Loading a script checks it and runs its top level once,** setting things up (D4, D17). A refused script doesn't
  load, and the host gets the messages (D17, D26).
- **The host then calls the script's procedures by name,** whenever its own events happen, as a game engine calls an
  enemy's `on_hit` (D4). The call is checked against the types in the procedure's header (D81).
- **A script with no procedures is a program,** run once from top to bottom (D4).
- **A host can load a script from its text, or from a bytecode file compiled ahead of time,** which Seal verifies
  before loading and refuses if it's been changed or comes from another Seal version. The script means the same
  either way (D141). A damaged or out-of-date file is always refused; a host can also require each file to be
  signed with its own key, using ML-DSA, so a file changed in any way is refused (D169).

## Doors

- **Doors are the only way a script reaches outside itself.** Each is a procedure the host makes and names; Seal defines
  none (D11).
- **A host gives a script doors only,** not ready-made values: a script gets what it needs through a door, once, at its
  top level, as in `ws = get_service("Workspace")` (D23).
- **A door's name, and its narrowings' names, follow Seal's spelling,** checked when the host opens it (D24).
- **A host can give its doors narrowings,** declared with the door, working exactly like Seal's own; an unknown one is
  refused at load time, with the door's real narrowings listed (D13, Unio D102).
- **Values cross the doors as copies,** and the host's own objects arrive as IDs (D15).
- **A door can take a unit,** as `wait(2 secs)`, or a plain number with its label saying the unit, as
  `wait(seconds: 2)`; the host chooses (D111, chapter 6).

- **A host can open types too,** its own records and `one_of`s, made with `#[derive(Seal)]` (D48). A script uses them
  by name without declaring them, as it uses a door, and declaring a type of the same name is refused (D71, D85).
- **A door can fail,** with failures the host declares, handled like any other (D44).

- **A door can have labels,** chosen by its host, as `wait(seconds: 2)` (D55).

**Not decided:** defaults on doors.

## Scripts apart, and pausing

- **Each loaded script is fully separate,** with its own names, memory, limits and doors; two never share a value, and
  talk only through the host's doors (D15, D49).
- **A door can pause a script,** and the host resumes it later, as `wait(seconds: 2)` in a patrol might (D49).
- **Each call into a script is a task.** While one is paused, the host can call the script again; only one task runs
  at a time, they switch only at pauses, and they share the script's top-level names (D51).
- **A host declares which doors can pause;** Seal works out which procedures can, from their bodies, and the language
  server marks every line where a pause can happen. Nothing is written at a call (D52).
- **The host can cancel a paused task,** ending it at its pause, while the script's other tasks carry on (D54).
- **A failed check or a limit in any task stops the whole script,** all its tasks, since a task stopped halfway may
  have left shared names half-changed (D54).
- **A script runs one thing at a time;** there's no parallelism inside it (D49).

## Limits

Each script gets a step budget and a memory cap, set by the host (D14):
- **A call into the script that runs past its step budget stops there,** at the same place on every machine. The
  budget counts each run between pauses, so each resume starts a fresh one (D53).
- **A script that would hold more than its cap stops** the same way. The cap covers the whole script, all its tasks
  together (D53).
- **The host gets control back, told why,** and decides what happens next. A stopped script never crashes the host.

**Not decided:** what a step is, the limits' defaults, and how the host is told; the building session proposes them
(D14).
