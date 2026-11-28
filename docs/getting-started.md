# Getting started with Wax

How to run a script, and how to embed Wax in a Rust program. What Wax is, and its rules, are in the
[README](../README.md) and the [standard](../standard/README.md).

## Trying it

Wax needs Rust, at the version [rust-toolchain.toml](../rust-toolchain.toml) names.

Run a script with the test host, which opens `show_msg`, `ask`, `rand` and `wait`, and makes the calls listed in a
`calls.txt` beside the script:

```
cargo run --release -p wax-command -- run tests/d004_host_calls/program.wax
```

Check a script without running it:

```
cargo run --release -p wax-command -- check tests/d004_host_calls/program.wax
```

Build, lint and test everything, the fuzzer and the comparison with Ralph included:

```
cargo run --release -p wax-check-all
```

## Using it from a Rust program

```rust
use wax::{Doors, Limits, Script};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string("goblin.wax")?;

    // The doors this script may use: here, only one.
    let mut doors = Doors::new();
    doors.add("show_msg", |text: String| println!("{text}"))?;

    // How many steps one call may take, and how much memory the script may hold.
    let limits = Limits { steps: 1_000_000, memory: 1 << 20 };

    // Loading checks the whole script, then runs its top level.
    let mut goblin = Script::load("goblin.wax", &source, &doors, limits)?;
    goblin.call::<()>("on_hit", (10,))?;
    Ok(())
}
```

A fuller example, a small game host with records, failures and a door that pauses, is in
[crates/wax/tests/host.rs](../crates/wax/tests/host.rs).
