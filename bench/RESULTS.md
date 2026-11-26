# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **Python:** `Python 3.14.7`, with `gmpy2` `2.3.2` (on GMP), in `target/bench-python`, which `bench/run.sh` makes; Python's `decimal` is on libmpdec `4.0.1`.
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.59 3.13 2.76 }` before, `{ 1.50 2.51 2.58 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 31.82 ms | 73.87 ms | **met**: Seal takes 43% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.59 ms | 69.77 ms | **met**: Seal takes 94% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.70 ms | 26.69 ms | **met**: Seal takes 70% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 104.17 ms | 219.17 ms | **met**: Seal takes 48% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 27.68 ms | 29.95 ms | **met**: Seal takes 92% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 345.16 µs | 161.44 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 101.03 ms | 131.43 ms | **met**: Seal takes 77% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.15 ms | 3.53 ms | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 33 ns | **met**: Seal takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 147 ns | 185 ns | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.67 µs | 4.78 µs | **met**: Seal takes 98% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 31.82 ms | 34.49 ms | **met**: Seal takes 92% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 65.59 ms | 72.48 ms | **met**: Seal takes 90% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.70 ms | 48.13 ms | **met**: Seal takes 39% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 104.17 ms | 150.29 ms | **met**: Seal takes 69% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 27.68 ms | 30.51 ms | **met**: Seal takes 91% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 345.16 µs | 41.84 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 101.03 ms | 126.82 ms | **met**: Seal takes 80% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.15 ms | 2.88 ms | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 27 ns | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 147 ns | 185 ns | **met**: Seal takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.67 µs | 1.77 µs | for reference, no target (D150) |
| D141. Loading a compiled file | patrolling scripts, loading each from a compiled file of its own | 2.08 µs | LuaJIT's bytecode: 284 ns | for reference, no target |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 12 ns | a Rust door: 7 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.6× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 1.58 ms | **met**: Seal takes 85% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.39 ms | 1.51 ms | **met**: Seal takes 92% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 110.59 ms | 161.05 ms | **met**: Seal takes 69% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 67.15 ms | 81.11 ms | **met**: Seal takes 83% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 412.38 µs | 903.22 µs | **met**: Seal takes 46% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 91.56 ms | 160.26 ms | **met**: Seal takes 57% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 31.82 ms | 95.84 ms | **met**: Seal takes 33% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.59 ms | 190.23 ms | **met**: Seal takes 34% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 27.68 ms | 50.88 ms | **met**: Seal takes 54% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 345.16 µs | 503.32 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.81 ms | 423.58 µs | 84.25 ms | 104.43 ms | 2.9 MB | 395 B | 769.21 µs cold, 416 ns warm; 2.73 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 194.51 ms | 305.40 µs | 194.09 ms | 195.43 ms | 4.8 MB | 339 B | 5.43 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 95.19 ms | 15.03 ms | 89.00 ms | 89.81 ms | 1.9 MB | 339 B | 2.79 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.77 ms | 151.05 µs | 44.41 ms | 46.46 ms | 2.3 MB | 339 B | 2.70 ms as a whole process |  |
| Node `--jitless` | Purpose | 326.93 ms | 6.70 ms | 320.02 ms | 329.58 ms | 43.5 MB | 362 B | 19.50 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 128.79 ms | 1.42 ms | 126.84 ms | 129.72 ms | 49.4 MB | 362 B | 19.89 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 130.25 ms | 8.43 ms | 124.95 ms | 136.26 ms | 49.5 MB | 362 B | 21.01 ms as a whole process |  |
| Seal | Normal | 31.82 ms | 447.52 µs | 31.23 ms | 32.37 ms | 2.8 MB | 169 B | 105.33 µs cold, 166 ns warm; 2.87 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled |
| Luau (interpreter) | Normal | 73.87 ms | 957.24 µs | 72.62 ms | 73.95 ms | 4.8 MB | 143 B | 5.42 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 34.49 ms | 427.25 µs | 33.94 ms | 34.38 ms | 1.9 MB | 143 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.41 ms | 64.40 µs | 7.34 ms | 7.63 ms | 2.2 MB | 143 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Normal | 95.84 ms | 577.31 µs | 94.87 ms | 94.58 ms | 43.4 MB | 137 B | 20.07 ms as a whole process |  |
| Node (JIT), from cold | Normal | 10.07 ms | 364.82 µs | 9.40 ms | 10.67 ms | 48.7 MB | 137 B | 19.27 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.17 ms | 371.15 µs | 9.58 ms | 9.71 ms | 48.7 MB | 137 B | 18.99 ms as a whole process |  |
| Seal | Ceil.source | 31.15 ms | 236.22 µs | 30.69 ms | 30.49 ms | 2.8 MB | 169 B | 82.29 µs cold, 209 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 32.53 ms | 335.19 µs | 31.78 ms | 57.64 ms | 2.8 MB | 169 B | 1.11 ms cold, 667 ns warm; 2.60 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.program | 30.40 ms | 922.94 µs | 29.12 ms | 55.57 ms | 2.6 MB | 169 B | 826.50 µs cold, 292 ns warm; 2.62 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 31.78 ms | 590.42 µs | 30.94 ms | 56.55 ms | 2.6 MB | 169 B | 808.88 µs cold, 667 ns warm; 2.54 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.35 ms | 491.30 µs | 70.77 ms | 72.81 ms | 4.8 MB | 143 B | 5.30 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.78 ms | 206.78 µs | 33.51 ms | 33.44 ms | 1.9 MB | 143 B | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.85 ms | 29.05 µs | 7.80 ms | 8.18 ms | 2.2 MB | 143 B | 2.44 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.40 ms | 536.99 µs | 94.57 ms | 95.50 ms | 43.5 MB | 137 B | 19.16 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.89 ms | 576.05 µs | 9.52 ms | 9.82 ms | 48.8 MB | 137 B | 19.59 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.57 ms | 60.16 µs | 9.50 ms | 12.03 ms | 48.7 MB | 137 B | 19.56 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 49.45 ms | 416.93 µs | 48.81 ms | 50.78 ms | 2.8 MB | 550 B | 165.58 µs cold, 250 ns warm; 2.74 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.00 ms | 1.05 ms | 91.10 ms | 93.64 ms | 4.8 MB | 433 B | 5.23 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.40 ms | 390.93 µs | 75.94 ms | 76.25 ms | 1.9 MB | 433 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.74 ms | 17.00 µs | 6.72 ms | 6.75 ms | 2.1 MB | 433 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.79 ms | 1.68 ms | 242.37 ms | 245.11 ms | 46.5 MB | 460 B | 18.93 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.09 ms | 71.82 µs | 18.99 ms | 19.55 ms | 50.5 MB | 460 B | 19.23 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.10 ms | 166.60 µs | 18.90 ms | 19.06 ms | 50.4 MB | 460 B | 19.62 ms as a whole process |  |
| Seal | Normal | 65.59 ms | 985.21 µs | 63.70 ms | 66.54 ms | 3.4 MB | 3.5 KB | 299.29 µs cold, 250 ns warm; 2.98 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.77 ms | 441.93 µs | 68.75 ms | 70.12 ms | 4.8 MB | 3.0 KB | 5.36 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 72.48 ms | 776.69 µs | 71.69 ms | 70.88 ms | 1.9 MB | 3.0 KB | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.22 ms | 25.22 µs | 5.17 ms | 5.38 ms | 2.2 MB | 3.0 KB | 2.54 ms as a whole process |  |
| Node `--jitless` | Normal | 190.23 ms | 21.21 ms | 179.48 ms | 211.11 ms | 48.2 MB | 3.1 KB | 20.87 ms as a whole process |  |
| Node (JIT), from cold | Normal | 4.08 ms | 61.08 µs | 3.96 ms | 6.15 ms | 52.2 MB | 3.1 KB | 20.45 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.96 ms | 31.01 µs | 3.93 ms | 6.14 ms | 52.1 MB | 3.1 KB | 19.79 ms as a whole process |  |
| Seal | Ceil.source | 54.19 ms | 520.87 µs | 53.61 ms | 56.87 ms | 3.3 MB | 3.4 KB | 261.21 µs cold, 250 ns warm; 2.95 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 57.80 ms | 2.28 ms | 55.79 ms | 59.81 ms | 3.3 MB | 3.4 KB | 284.08 µs cold, 250 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.97 ms | 247.40 µs | 50.42 ms | 51.00 ms | 3.0 MB | 3.4 KB | 262.62 µs cold, 250 ns warm; 2.95 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.62 ms | 468.52 µs | 48.73 ms | 50.36 ms | 3.1 MB | 3.4 KB | 261.08 µs cold, 250 ns warm; 3.00 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 70.99 ms | 218.59 µs | 70.60 ms | 72.05 ms | 4.9 MB | 3.0 KB | 5.50 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 73.11 ms | 496.44 µs | 72.38 ms | 74.15 ms | 1.9 MB | 3.0 KB | 2.56 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.35 ms | 56.85 µs | 5.29 ms | 5.54 ms | 2.2 MB | 3.0 KB | 2.55 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 190.26 ms | 24.78 ms | 178.47 ms | 185.75 ms | 48.3 MB | 3.1 KB | 20.35 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 4.05 ms | 66.03 µs | 3.95 ms | 6.03 ms | 52.1 MB | 3.1 KB | 19.84 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 4.11 ms | 268.56 µs | 3.90 ms | 5.93 ms | 52.2 MB | 3.1 KB | 23.25 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.41 ms | 8.36 ms | 50.50 ms | 58.59 ms | 3.0 MB | 429 B | 101.38 µs cold, 334 ns warm; 2.77 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 46.98 ms | 712.11 µs | 45.92 ms | 46.88 ms | 4.9 MB | 366 B | 6.13 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 78.21 ms | 971.90 µs | 76.66 ms | 79.86 ms | 2.0 MB | 366 B | 2.58 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 23.34 ms | 67.25 µs | 23.24 ms | 23.64 ms | 2.2 MB | 366 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 367.65 ms | 6.48 ms | 361.89 ms | 381.95 ms | 48.6 MB | 398 B | 18.66 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.42 ms | 35.81 ms | 15.72 ms | 22.00 ms | 53.0 MB | 398 B | 19.21 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.43 ms | 28.89 µs | 15.39 ms | 19.19 ms | 55.5 MB | 398 B | 18.73 ms as a whole process |  |
| Seal | Normal | 18.70 ms | 316.92 µs | 18.01 ms | 18.79 ms | 3.1 MB | 1.2 KB | 161.38 µs cold, 250 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 26.69 ms | 319.81 µs | 26.19 ms | 26.60 ms | 4.8 MB | 981 B | 5.22 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 48.13 ms | 1.65 ms | 45.65 ms | 45.40 ms | 2.0 MB | 981 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.52 ms | 25.06 µs | 1.49 ms | 1.66 ms | 2.3 MB | 981 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Normal | 63.67 ms | 268.35 µs | 63.14 ms | 63.24 ms | 48.5 MB | 1.0 KB | 18.32 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.68 ms | 253.79 µs | 1.53 ms | 3.22 ms | 53.0 MB | 1.0 KB | 18.44 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.62 ms | 17.48 µs | 1.59 ms | 3.33 ms | 53.0 MB | 1.0 KB | 18.57 ms as a whole process |  |
| Seal | Ceil.source | 18.47 ms | 486.97 µs | 17.72 ms | 18.99 ms | 3.1 MB | 1.2 KB | 147.67 µs cold, 250 ns warm; 2.61 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.58 ms | 115.50 µs | 18.41 ms | 18.51 ms | 3.1 MB | 1.2 KB | 155.50 µs cold, 250 ns warm; 2.73 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.19 ms | 177.20 µs | 18.80 ms | 19.61 ms | 2.9 MB | 1.2 KB | 129.75 µs cold, 250 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.87 ms | 322.19 µs | 18.44 ms | 19.64 ms | 3.0 MB | 1.2 KB | 135.38 µs cold, 208 ns warm; 2.54 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.03 ms | 507.91 µs | 26.30 ms | 26.97 ms | 4.8 MB | 981 B | 5.20 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 45.56 ms | 904.07 µs | 43.45 ms | 44.74 ms | 2.0 MB | 981 B | 2.56 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.54 ms | 39.62 µs | 1.51 ms | 1.68 ms | 2.3 MB | 981 B | 2.44 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.99 ms | 360.37 µs | 62.64 ms | 63.29 ms | 48.6 MB | 1.0 KB | 18.28 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.62 ms | 178.40 µs | 1.49 ms | 3.31 ms | 53.0 MB | 1.0 KB | 19.43 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.66 ms | 56.46 µs | 1.61 ms | 3.16 ms | 53.0 MB | 1.0 KB | 20.40 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 87.83 ms | 3.73 ms | 85.15 ms | 88.14 ms | 2.9 MB | 632 B | 198.58 µs cold, 250 ns warm; 2.69 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 89.46 ms | 951.30 µs | 88.16 ms | 90.27 ms | 4.8 MB | 523 B | 5.98 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.80 ms | 584.28 µs | 52.13 ms | 52.65 ms | 1.9 MB | 523 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.23 ms | 60.44 µs | 6.16 ms | 6.29 ms | 2.2 MB | 523 B | 2.55 ms as a whole process |  |
| Node `--jitless` | Purpose | 150.29 ms | 870.61 µs | 149.55 ms | 150.18 ms | 43.4 MB | 568 B | 19.09 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.67 ms | 38.47 µs | 3.62 ms | 4.77 ms | 49.5 MB | 568 B | 19.30 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.70 ms | 31.10 µs | 3.65 ms | 4.55 ms | 49.5 MB | 568 B | 19.52 ms as a whole process |  |
| Seal | Normal | 104.17 ms | 202.68 µs | 103.87 ms | 103.94 ms | 3.1 MB | 1.8 KB | 166.88 µs cold, 291 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 219.17 ms | 12.95 ms | 212.21 ms | 211.55 ms | 4.8 MB | 1.7 KB | 5.72 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 150.29 ms | 2.71 ms | 146.78 ms | 147.37 ms | 2.0 MB | 1.7 KB | 2.57 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 50.25 ms | 1.10 ms | 49.06 ms | 49.54 ms | 2.3 MB | 1.7 KB | 2.51 ms as a whole process |  |
| Node `--jitless` | Normal | 159.84 ms | 4.54 ms | 155.18 ms | 158.44 ms | 45.5 MB | 1.7 KB | 19.29 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.95 ms | 141.80 µs | 14.67 ms | 16.85 ms | 52.1 MB | 1.7 KB | 20.25 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.26 ms | 49.30 µs | 14.19 ms | 15.82 ms | 52.4 MB | 1.7 KB | 19.18 ms as a whole process |  |
| Seal | Ceil.source | 106.33 ms | 4.55 ms | 103.43 ms | 105.81 ms | 3.1 MB | 1.8 KB | 163.50 µs cold, 292 ns warm; 2.96 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 111.76 ms | 14.02 ms | 105.70 ms | 106.41 ms | 3.1 MB | 1.8 KB | 167.38 µs cold, 208 ns warm; 2.77 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 103.74 ms | 385.94 µs | 103.29 ms | 104.90 ms | 2.9 MB | 1.8 KB | 124.88 µs cold, 250 ns warm; 2.90 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 101.12 ms | 342.03 µs | 100.82 ms | 101.60 ms | 3.0 MB | 1.8 KB | 144.33 µs cold, 208 ns warm; 2.83 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 210.51 ms | 2.15 ms | 204.58 ms | 211.44 ms | 4.8 MB | 1.7 KB | 5.22 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 146.90 ms | 502.21 µs | 146.29 ms | 144.15 ms | 2.0 MB | 1.7 KB | 2.80 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.38 ms | 335.70 µs | 47.90 ms | 48.50 ms | 2.3 MB | 1.7 KB | 2.46 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.36 ms | 1.40 ms | 155.39 ms | 158.83 ms | 45.4 MB | 1.7 KB | 19.38 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.09 ms | 351.94 µs | 13.87 ms | 15.51 ms | 52.1 MB | 1.7 KB | 18.91 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.71 ms | 55.54 µs | 14.61 ms | 16.35 ms | 52.3 MB | 1.7 KB | 18.98 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 126.14 ms | 951.36 µs | 125.17 ms | 123.03 ms | 4.1 MB | 569 B | 129.58 µs cold, 333 ns warm; 2.76 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.64 ms | 337.63 µs | 62.21 ms | 63.16 ms | 4.8 MB | 540 B | 5.26 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 34.71 ms | 1.44 ms | 32.52 ms | 33.81 ms | 2.2 MB | 540 B | 2.60 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 5.16 ms | 118.02 µs | 5.04 ms | 4.75 ms | 2.4 MB | 540 B | 2.28 ms as a whole process |  |
| Node `--jitless` | Purpose | 240.64 ms | 1.35 ms | 238.88 ms | 244.25 ms | 48.8 MB | 514 B | 18.87 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 89.29 ms | 22.47 ms | 68.58 ms | 91.50 ms | 52.8 MB | 514 B | 18.84 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.70 ms | 719.91 µs | 66.52 ms | 88.71 ms | 52.7 MB | 514 B | 19.05 ms as a whole process |  |
| Seal | Normal | 27.68 ms | 143.74 µs | 27.50 ms | 27.69 ms | 4.1 MB | 538 B | 106.42 µs cold, 292 ns warm; 2.76 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.95 ms | 234.34 µs | 29.62 ms | 29.66 ms | 6.0 MB | 478 B | 5.23 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 30.51 ms | 634.35 µs | 29.49 ms | 31.52 ms | 3.1 MB | 478 B | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.12 ms | 594.61 µs | 11.42 ms | 12.99 ms | 3.4 MB | 478 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Normal | 50.88 ms | 439.30 µs | 50.25 ms | 52.82 ms | 56.6 MB | 477 B | 19.56 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.36 ms | 345.85 µs | 34.77 ms | 35.92 ms | 62.7 MB | 477 B | 18.53 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.11 ms | 236.39 µs | 34.82 ms | 35.93 ms | 88.2 MB | 477 B | 19.14 ms as a whole process |  |
| Seal | Ceil.source | 26.37 ms | 195.52 µs | 26.07 ms | 26.52 ms | 4.1 MB | 538 B | 118.96 µs cold, 333 ns warm; 2.96 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.47 ms | 182.80 µs | 27.23 ms | 27.88 ms | 4.1 MB | 538 B | 110.38 µs cold, 375 ns warm; 2.50 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.73 ms | 118.35 µs | 23.53 ms | 23.81 ms | 4.1 MB | 538 B | 97.17 µs cold, 250 ns warm; 2.78 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.76 ms | 99.51 µs | 23.60 ms | 24.14 ms | 4.1 MB | 538 B | 96.42 µs cold, 292 ns warm; 2.67 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.94 ms | 346.50 µs | 29.61 ms | 29.68 ms | 6.0 MB | 478 B | 5.30 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.18 ms | 317.49 µs | 32.76 ms | 31.55 ms | 3.1 MB | 478 B | 2.53 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.48 ms | 773.13 µs | 11.21 ms | 11.53 ms | 3.4 MB | 478 B | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.44 ms | 329.33 µs | 50.95 ms | 52.59 ms | 56.6 MB | 477 B | 19.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.88 ms | 299.85 µs | 34.33 ms | 36.37 ms | 62.6 MB | 477 B | 18.42 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.31 ms | 243.61 µs | 34.93 ms | 36.14 ms | 88.3 MB | 477 B | 18.66 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 316.85 µs | 1.72 µs | 314.83 µs | 340.62 µs | 3.2 MB | 233 B | 117.58 µs cold, 458 ns warm; 2.60 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.23 ms | 517.23 µs | 143.65 ms | 145.54 ms | 10.9 MB | 195 B | 5.28 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.72 ms | 270.55 µs | 32.52 ms | 31.29 ms | 3.6 MB | 195 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 21.22 ms | 66.54 µs | 21.14 ms | 21.59 ms | 3.7 MB | 195 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 367.32 µs | 144.69 µs | 235.58 µs | 1.03 ms | 52.6 MB | 223 B | 19.13 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 215.92 µs | 147.58 µs | 84.88 µs | 901.21 µs | 57.9 MB | 223 B | 18.79 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 130.79 µs | 117.81 µs | 55.96 µs | 904.00 µs | 58.0 MB | 223 B | 18.84 ms as a whole process |  |
| Seal | Normal | 345.16 µs | 1.63 µs | 343.29 µs | 368.38 µs | 3.4 MB | 182 B | 83.33 µs cold, 333 ns warm; 2.66 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 161.44 ms | 288.00 µs | 160.98 ms | 163.17 ms | 11.8 MB | 138 B | 5.27 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.84 ms | 461.01 µs | 41.36 ms | 41.06 ms | 5.1 MB | 138 B | 2.33 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.55 ms | 321.79 µs | 19.06 ms | 20.17 ms | 5.3 MB | 138 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Normal | 503.32 µs | 334.81 µs | 282.96 µs | 1.11 ms | 61.3 MB | 155 B | 18.54 ms as a whole process |  |
| Node (JIT), from cold | Normal | 496.90 µs | 474.28 µs | 126.79 µs | 1.03 ms | 65.3 MB | 155 B | 19.10 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 285.73 µs | 334.12 µs | 126.00 µs | 1.05 ms | 69.5 MB | 155 B | 18.69 ms as a whole process |  |
| Seal | Ceil.source | 361.58 µs | 7.17 µs | 352.00 µs | 373.62 µs | 3.4 MB | 182 B | 80.96 µs cold, 334 ns warm; 2.69 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 352.23 µs | 2.75 µs | 350.21 µs | 381.08 µs | 3.4 MB | 182 B | 79.17 µs cold, 292 ns warm; 2.69 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 327.83 µs | 2.21 µs | 323.75 µs | 347.79 µs | 3.2 MB | 182 B | 62.88 µs cold, 250 ns warm; 2.64 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 276.24 µs | 3.66 µs | 272.83 µs | 312.42 µs | 3.2 MB | 182 B | 64.33 µs cold, 292 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.35 ms | 283.50 µs | 160.94 ms | 163.21 ms | 11.9 MB | 138 B | 5.14 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.00 ms | 245.77 µs | 40.70 ms | 41.03 ms | 4.9 MB | 138 B | 2.53 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.38 ms | 116.74 µs | 19.22 ms | 20.05 ms | 5.2 MB | 138 B | 2.42 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 534.61 µs | 366.79 µs | 275.38 µs | 1.12 ms | 61.4 MB | 155 B | 18.62 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 498.22 µs | 461.34 µs | 129.54 µs | 1.08 ms | 65.2 MB | 155 B | 19.10 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 278.43 µs | 329.51 µs | 126.71 µs | 1.08 ms | 69.4 MB | 155 B | 19.37 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 74.38 ms | 465.57 µs | 73.68 ms | 73.51 ms | 3.1 MB | 537 B | 149.33 µs cold, 208 ns warm; 2.75 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 93.39 ms | 373.64 µs | 92.84 ms | 92.88 ms | 4.8 MB | 411 B | 5.15 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 117.42 ms | 1.36 ms | 113.82 ms | 117.86 ms | 2.6 MB | 411 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 93.52 ms | 667.41 µs | 92.52 ms | 93.87 ms | 2.9 MB | 411 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.75 ms | 487.92 µs | 77.11 ms | 82.05 ms | 59.7 MB | 444 B | 18.57 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.40 ms | 281.89 µs | 10.08 ms | 12.90 ms | 66.0 MB | 444 B | 18.89 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.37 ms | 742.36 µs | 9.98 ms | 12.89 ms | 115.0 MB | 444 B | 19.01 ms as a whole process |  |
| Seal | Normal | 101.03 ms | 950.81 µs | 99.66 ms | 101.04 ms | 7.9 MB | 992 B | 129.12 µs cold, 250 ns warm; 2.77 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 131.43 ms | 1.07 ms | 129.49 ms | 134.53 ms | 40.7 MB | 799 B | 5.33 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 126.82 ms | 785.49 µs | 125.05 ms | 125.68 ms | 39.9 MB | 799 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.95 ms | 1.27 ms | 80.21 ms | 81.50 ms | 44.5 MB | 799 B | 2.54 ms as a whole process |  |
| Node `--jitless` | Normal | 112.97 ms | 1.85 ms | 111.29 ms | 114.59 ms | 213.5 MB | 878 B | 18.88 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.15 ms | 924.86 µs | 13.23 ms | 16.90 ms | 129.4 MB | 878 B | 18.35 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.38 ms | 122.67 µs | 13.21 ms | 17.05 ms | 224.2 MB | 878 B | 19.28 ms as a whole process |  |
| Seal | Ceil.source | 99.62 ms | 800.60 µs | 97.86 ms | 102.90 ms | 8.0 MB | 992 B | 138.88 µs cold, 208 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 99.10 ms | 814.11 µs | 97.40 ms | 98.97 ms | 7.9 MB | 992 B | 127.79 µs cold, 250 ns warm; 2.77 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 98.67 ms | 736.60 µs | 97.46 ms | 100.11 ms | 7.7 MB | 992 B | 114.58 µs cold, 167 ns warm; 2.58 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 101.87 ms | 1.31 ms | 100.60 ms | 105.24 ms | 7.7 MB | 992 B | 107.08 µs cold, 250 ns warm; 2.56 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 130.41 ms | 2.02 ms | 127.26 ms | 130.23 ms | 42.5 MB | 799 B | 5.25 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 134.73 ms | 4.40 ms | 129.12 ms | 128.52 ms | 38.8 MB | 799 B | 2.47 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 81.78 ms | 1.20 ms | 79.61 ms | 82.50 ms | 39.4 MB | 799 B | 2.60 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 114.96 ms | 2.04 ms | 112.80 ms | 116.76 ms | 213.4 MB | 878 B | 19.47 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.21 ms | 922.13 µs | 13.38 ms | 16.80 ms | 129.9 MB | 878 B | 18.29 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.27 ms | 57.86 µs | 13.15 ms | 16.98 ms | 224.1 MB | 878 B | 18.60 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 56.68 µs (±8.15 µs, least 49.67 µs) | 15.97 µs (±4.73 µs, least 12.21 µs) | 13.86 µs (±6.09 µs, least 10.00 µs) | 19 ns (±0 ns, least 18 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 3.9 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.61 µs (±598 ns, least 9.75 µs) | 16.68 µs (±1.59 µs, least 14.71 µs) | ? | 41 ns (±1 ns, least 40 ns) | 20 ns (±2 ns, least 17 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.90 µs (±738 ns, least 3.00 µs) | 7.20 µs (±1.69 µs, least 5.00 µs) | 1.20 µs (±422 ns, least 1.00 µs) | 22 ns (±1 ns, least 19 ns) | 16 ns (±2 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.60 µs (±516 ns, least 3.00 µs) | 7.90 µs (±876 ns, least 7.00 µs) | 900 ns (±316 ns, least 0 ns) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 30.19 µs (±8.22 µs, least 20.38 µs) | 64.89 µs (±76.66 µs, least 36.75 µs) | ? | 57 ns (±2 ns, least 53 ns) | 29 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 30.65 µs (±7.32 µs, least 19.88 µs) | 37.60 µs (±2.47 µs, least 34.58 µs) | ? | 9 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.67 µs (±54 ns, least 4.62 µs) | 147 ns (±4 ns, least 141 ns) | 2.08 µs (±76 ns, least 2.01 µs) | 20 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 9.6 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.78 µs (±126 ns, least 4.62 µs) | 185 ns (±5 ns, least 180 ns) | ? | 33 ns (±1 ns, least 32 ns) | 17 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.77 µs (±16 ns, least 1.75 µs) | 185 ns (±5 ns, least 178 ns) | 284 ns (±7 ns, least 277 ns) | 27 ns (±1 ns, least 26 ns) | 16 ns (±3 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.79 µs (±13 ns, least 1.77 µs) | 367 ns (±36 ns, least 341 ns) | 483 ns (±81 ns, least 255 ns) | 90 ns (±1 ns, least 89 ns) | 102 ns (±12 ns, least 97 ns) | 1.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.13 µs (±25 ns, least 1.11 µs) | 668 ns (±16 ns, least 648 ns) | ? | 50 ns (±1 ns, least 49 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.10 µs (±21 ns, least 1.06 µs) | 659 ns (±16 ns, least 633 ns) | ? | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.69 µs (±53 ns, least 4.62 µs) | 147 ns (±6 ns, least 142 ns) | 2.03 µs (±36 ns, least 2.00 µs) | 20 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 9.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.60 µs (±57 ns, least 4.54 µs) | 151 ns (±4 ns, least 146 ns) | 2.06 µs (±101 ns, least 1.97 µs) | 20 ns (±0 ns, least 20 ns) | 41 ns (±0 ns, least 40 ns) | 512 B | 9.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.00 µs (±46 ns, least 3.93 µs) | 137 ns (±7 ns, least 130 ns) | 1.89 µs (±96 ns, least 1.81 µs) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 9.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.05 µs (±49 ns, least 3.97 µs) | 138 ns (±3 ns, least 135 ns) | 1.90 µs (±23 ns, least 1.86 µs) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 9.5 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.68 µs (±42 ns, least 4.62 µs) | 183 ns (±10 ns, least 169 ns) | ? | 34 ns (±2 ns, least 32 ns) | 17 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.77 µs (±20 ns, least 1.74 µs) | 182 ns (±3 ns, least 179 ns) | 285 ns (±8 ns, least 274 ns) | 27 ns (±0 ns, least 26 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.79 µs (±11 ns, least 1.77 µs) | 376 ns (±45 ns, least 347 ns) | 475 ns (±72 ns, least 336 ns) | 90 ns (±3 ns, least 88 ns) | 102 ns (±6 ns, least 97 ns) | 1.0 KB | 6.0 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±17 ns, least 1.13 µs) | 668 ns (±21 ns, least 625 ns) | ? | 50 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±17 ns, least 1.06 µs) | 670 ns (±12 ns, least 650 ns) | ? | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 31.04 ms | 31.55 ms (+1.6%) |
| n-body, 100,000 steps | 64.29 ms | 60.63 ms (-5.7%) |
| spectral-norm, size 200 | 19.15 ms | 17.71 ms (-7.5%) |
| fannkuch-redux, 9 | 102.85 ms | 104.79 ms (+1.9%) |
| word count, 500,000 words | 26.17 ms | 26.49 ms (+1.2%) |
| text building, 20,000 joins | 348.89 µs | 352.90 µs (+1.1%) |
| binary-trees, depth 14 | 98.60 ms | 98.55 ms (-0.1%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Seal mean | Seal deviation | Seal least | Seal peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 4.44 µs | 1.35 ms | 3.7 MB | 1.58 ms | 1.57 ms | 299.58 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.39 ms | 18.84 µs | 1.37 ms | 3.6 MB | 1.51 ms | 1.49 ms | 1.09 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 110.59 ms | 707.28 µs | 109.54 ms | 3.0 MB | 161.05 ms | 160.68 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 67.15 ms | 145.09 µs | 66.90 ms | 2.9 MB | 81.11 ms | 80.50 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 412.38 µs | 2.38 µs | 410.17 µs | 3.4 MB | 903.22 µs | 897.46 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 91.56 ms | 2.06 ms | 87.53 ms | 2.9 MB | 160.26 ms | 159.86 ms | – |
| the money sums in `Float`s, as a yardstick | 10.51 ms | 159.75 µs | 10.23 ms | 2.8 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Seal's door is written in C, through Seal's C API (`crates/seal-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Seal's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Seal | C | 12 ns | |
| Seal | Rust | 7 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
