# Seal

Seal is an interpreted, embeddable, sandboxed scripting language, inspired by [Unio](https://github.com/Soclets/Unio-lang)
and meant to be easier to write. Its first official interpreter, Ngin1, is written in Rust.

It's sealed: a Seal script can reach nothing outside itself except through the doors the program embedding it opens.

## A script

```
mut hp = 30
show_msg("loaded")

proc on_hit[amount: Whole]
    hp = max(_ - amount, 0)
    show_msg("hit for {amount}, {hp} left")
end
```

The top level runs once, when the script is loaded. After that the host calls the script's procedures, here
`on_hit`. `show_msg` is a door: it exists only because the host opened it.

## Where things are

- [docs/getting-started.md](docs/getting-started.md): running a script, and embedding Seal in a Rust program.
- [DECISIONS.md](DECISIONS.md): every design choice, with what was weighed and why it won.
- [standard/](standard/README.md): what the rules are today.
- [tests/](tests/README.md): the contract the implementation is built to.
- [IMPLEMENTATION.md](IMPLEMENTATION.md): the rules the implementation is built by.
- [PROGRESS.md](PROGRESS.md): what's built, what's still open, and the benchmarks.

## Licences

- [`LICENSE`](LICENSE): Apache-2.0 WITH LLVM-exception, for the implementation: the `seal` library, its C API, the
  `seal` command and the other tools. It allows commercial use and gives a patent promise; a game shipping Seal
  includes this licence, as Apache-2.0 asks, and scripts Seal compiles owe nothing.
- [`LICENSE-MIT-0`](LICENSE-MIT-0): MIT-0, for examples and demos: the test scripts in `tests/`, the benchmark programs
  in `bench/programs/`, and every code sample inside the Markdown files. Copy and remix them freely, with no credit
  needed.
- [`LICENSE-CC-BY-4.0`](LICENSE-CC-BY-4.0): CC BY 4.0, for the text of this repo's Markdown files, including this
  README, `DECISIONS.md` and the standard, apart from their code samples. Credit them as `Seal GitHub repo text:
  https://github.com/Soclets/Seal-lang`, say they're under CC BY 4.0, and say whether you changed them.

[`REUSE.toml`](REUSE.toml) maps each file to its licence.
