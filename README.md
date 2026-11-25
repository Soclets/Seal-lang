# Seal

Seal is an interpreted, embeddable, sandboxed scripting language, inspired by [Unio](https://github.com/Soclets/Unio-lang)
and meant to be easier to write. It's implemented in Rust.

It's sealed: a Seal script can reach nothing outside itself except through the doors the program embedding it opens.

## Where it stands

Seal is still being designed, and it's built as it's designed. Every design choice is recorded in
[DECISIONS.md](DECISIONS.md), and every decision so far, D1 to D73, is built in this repository, except the language
server, which is an empty package for now. What a decision hasn't settled stops with a message saying it isn't
decided yet; nothing is guessed at.

Four parts work today:

- **The `seal` library** (`crates/seal`): a host program loads scripts with it, opens doors for them, calls their
  procedures, and sets how many steps and how much memory each may use. No script can crash its host, freeze it or
  take more memory than it's given.
- **The `seal` command** (`crates/seal-command`): checks a script, or runs one with a small test host.
- **The fuzzer** (`fuzz`): throws random scripts, valid and broken, at the library and checks that nothing breaks
  those promises.
- **The Unio translator** (`crates/seal-to-unio`): turns the part of Seal that Unio shares into Unio, so that Seal's
  answers can be compared with Ralph's, Unio's reference interpreter.

## A script

```
mut hp = 30
show_msg("loaded")

proc on_hit[amount: Whole]
    hp = hp - amount
    show_msg("hit for {amount}, {hp} left")
end
```

The top level runs once, when the script is loaded. After that the host calls the script's procedures, here
`on_hit`. `show_msg` is a door: it exists only because the host opened it.

## Trying it

Seal needs Rust, at the version [rust-toolchain.toml](rust-toolchain.toml) names.

Run a script with the test host, which opens `show_msg`, `ask`, `rand` and `wait`, and makes the calls listed in a
`calls.txt` beside the script:

```
cargo run --release -p seal-command -- run tests/d004_host_calls/program.seal
```

Check a script without running it:

```
cargo run --release -p seal-command -- check tests/d004_host_calls/program.seal
```

Build, lint and test everything, the fuzzer and the comparison with Ralph included:

```
cargo run --release -p seal-check-all
```

## Using it from a Rust program

```rust
use seal::{Doors, Limits, Script};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("goblin.seal")?;

    // The doors this script may use: here, only one.
    let mut doors = Doors::new();
    doors.add("show_msg", |text: String| println!("{text}"))?;

    // How many steps one call may take, and how much memory the script may hold.
    let limits = Limits { steps: 1_000_000, memory: 1 << 20 };

    // Loading checks the whole script, then runs its top level.
    let mut goblin = Script::load("goblin.seal", &source, &doors, limits)?;
    goblin.call::<()>("on_hit", (10,))?;
    Ok(())
}
```

A fuller example, a small game host with records, failures and a door that pauses, is in
[crates/seal/tests/host.rs](crates/seal/tests/host.rs).

## Where things are

- [DECISIONS.md](DECISIONS.md): every design choice, with what was weighed and why it won.
- [standard/](standard/README.md): what the rules are today.
- [tests/](tests/README.md): the contract the implementation is built to. All 14 pass.
- [IMPLEMENTATION.md](IMPLEMENTATION.md): the rules the implementation is built by.
- [PROGRESS.md](PROGRESS.md): what's built, what's still open, and the wording drafted for each message.

There's no license yet.
