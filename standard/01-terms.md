# 1. Terms

The words this standard uses. Unio's terms (Unio's standard, chapter 1) hold too, such as procedure, narrowing,
label, slide and failure, except where a term below replaces one.

## Scripts and hosts

- **Script:** a Wax file, ending in `.wax`, loaded and run by a host (D1, D25).
- **Host:** the program, written in Rust, that embeds Wax, loads scripts and calls into them (D1, D4).
- **Door:** a procedure the host gives a script, the only way a script reaches anything outside itself (D11).
- **Top level:** a script's lines outside any procedure, run once when the script is loaded (D4).
- **Built-in:** a procedure Wax provides itself, touching nothing outside the script (D11).

## Values

- **Variable:** a named value, made with `name = value`. A **mutable variable** is made with `mut` and can change; an
  **immutable variable** is made without it and never changes, its insides included (D2, D15, D162). The docs and
  messages say "variable", never "binding", and "name" only for a spelling.

## Checking and running

- **Load time:** when the host loads a script, and Wax reads, checks and prepares it, before any of it runs (D4, D17).
  It takes the place of Unio's compile time.
- **Refused:** rejected at load time, with a message saying why. A refused script doesn't load (D17, D26).
- **Running:** while the script's top level or one of its procedures runs (D4).
- **Stopped:** ended while running, by a failed check or a limit, with the host told why and carrying on (D14, D17).
- **Step:** a unit of the work a script does, counted against the host's step budget (D14).
