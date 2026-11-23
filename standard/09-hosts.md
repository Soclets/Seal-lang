# 9. Hosts

A host is the Rust program that embeds Seal (D1). This chapter says what a script sees of it; how a host's Rust code
calls Seal is the building session's to propose.

## Loading and calling

- **Loading a script checks it and runs its top level once,** setting things up (D4, D17). A refused script doesn't
  load, and the host gets the messages (D17, D26).
- **The host then calls the script's procedures by name,** whenever its own events happen, as a game engine calls an
  enemy's `on_hit` (D4). The call is checked against the procedure's types, written or worked out (D5, D6).
- **A script with no procedures is a program,** run once from top to bottom (D4).

## Doors

- **Doors are the only way a script reaches outside itself.** Each is a procedure the host makes and names; Seal defines
  none (D11).
- **A host gives a script doors only,** not ready-made values: a script gets what it needs through a door, once, at its
  top level, as in `ws = get_service("Workspace")` (D23).
- **A door's name, and its narrowings' names, follow Seal's spelling,** checked when the host opens it (D24).
- **A host can give its doors narrowings,** declared with the door, working exactly like Seal's own; an unknown one is
  refused at load time, with the door's real narrowings listed (D13, Unio D102).
- **Values cross the doors as copies,** and the host's own objects arrive as IDs (D15).

- **A door can fail,** with failures the host declares, handled like any other (D44).

**Not decided:** labels and defaults on doors.

## Scripts apart, and pausing

- **Each loaded script is fully separate,** with its own names, memory, limits and doors; two never share a value, and
  talk only through the host's doors (D15, D49).
- **A door can pause a script,** and the host resumes it later, as `wait(seconds: 2)` in a patrol might (D49).
- **Each call into a script is a task.** While one is paused, the host can call the script again; only one task runs
  at a time, they switch only at pauses, and they share the script's top-level names (D51).
- **A host declares which doors can pause;** Seal works out which procedures can, from their bodies, and the language
  server marks every line where a pause can happen. Nothing is written at a call (D52).
- **A script runs one thing at a time;** there's no parallelism inside it (D49).

## Limits

Each script gets a step budget and a memory cap, set by the host (D14):
- **A call into the script that runs past its step budget stops there,** at the same place on every machine.
- **A script that would hold more than its cap stops** the same way.
- **The host gets control back, told why,** and decides what happens next. A stopped script never crashes the host.

**Not decided:** what a step is, the limits' defaults, and how the host is told; the building session proposes them
(D14).
