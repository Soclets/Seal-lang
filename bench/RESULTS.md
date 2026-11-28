# Wax's benchmark results

Written by `wax-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Wax:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **Python:** `Python 3.14.7`, with `gmpy2` `2.3.2` (on GMP), in `target/bench-python`, which `bench/run.sh` makes; Python's `decimal` is on libmpdec `4.0.1`.
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 3.44 3.03 2.58 }` before, `{ 1.90 2.46 2.42 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Wax (cold, then warm, when Wax reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Wax's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Wax | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 31.07 ms | 70.86 ms | **met**: Wax takes 44% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.75 ms | 70.28 ms | **met**: Wax takes 95% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.16 ms | 28.48 ms | **met**: Wax takes 64% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 106.84 ms | 220.92 ms | **met**: Wax takes 48% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.54 ms | 29.78 ms | **met**: Wax takes 89% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 332.58 µs | 161.31 ms | **met**: Wax takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 100.53 ms | 135.72 ms | **met**: Wax takes 74% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.16 ms | 3.52 ms | **met**: Wax takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 33 ns | **met**: Wax takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 157 ns | 191 ns | **met**: Wax takes 82% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.45 µs | 4.90 µs | **met**: Wax takes 91% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 31.07 ms | 33.87 ms | **met**: Wax takes 92% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 66.75 ms | 71.53 ms | **met**: Wax takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.16 ms | 45.08 ms | **met**: Wax takes 40% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 106.84 ms | 146.33 ms | **met**: Wax takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.54 ms | 30.98 ms | **met**: Wax takes 86% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 332.58 µs | 41.30 ms | **met**: Wax takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 100.53 ms | 127.86 ms | **met**: Wax takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.16 ms | 2.93 ms | **met**: Wax takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 28 ns | **met**: Wax takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 157 ns | 184 ns | **met**: Wax takes 86% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.45 µs | 1.79 µs | for reference, no target (D150) |
| D171, D176. Within twice LuaJIT loading its own bytecode | patrolling scripts, loading each from a compiled file of its own | 536 ns | 288 ns | **met**: Wax takes 186% of its time |
| D176. Loading the same compiled file again | patrolling scripts, each loaded from one `.waxc` file, reusing what was verified | 137 ns | | for reference, no target |
| D171. Checking a `.waxc` file's signature, once per file | patrolling scripts, loading each signed file of its own, signature checked | 39.70 µs | | for reference, no target |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 12 ns | a Rust door: 8 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.5× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.39 ms | 1.58 ms | **met**: Wax takes 88% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.40 ms | 1.52 ms | **met**: Wax takes 92% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 116.05 ms | 161.82 ms | **met**: Wax takes 72% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 71.17 ms | 82.68 ms | **met**: Wax takes 86% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 441.50 µs | 908.04 µs | **met**: Wax takes 49% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 96.01 ms | 165.39 ms | **met**: Wax takes 58% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 31.07 ms | 94.85 ms | **met**: Wax takes 33% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.75 ms | 178.17 ms | **met**: Wax takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.54 ms | 50.94 ms | **met**: Wax takes 52% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 332.58 µs | 508.88 µs | **met**: Wax takes 65% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 105.91 ms | 1.90 ms | 103.10 ms | 128.49 ms | 3.0 MB | 377 B | 866.54 µs cold, 458 ns warm; 2.54 ms as a whole process | 32.0 KB counted by Wax; 176 B compiled |
| Luau (interpreter) | Purpose | 197.67 ms | 1.66 ms | 195.49 ms | 196.29 ms | 4.8 MB | 339 B | 5.30 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.96 ms | 747.72 µs | 89.28 ms | 90.21 ms | 1.9 MB | 339 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.71 ms | 177.86 µs | 44.49 ms | 46.20 ms | 2.3 MB | 339 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Purpose | 319.73 ms | 354.22 µs | 319.24 ms | 323.42 ms | 43.5 MB | 362 B | 19.14 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 126.68 ms | 74.21 µs | 126.51 ms | 130.39 ms | 49.6 MB | 362 B | 19.24 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 125.21 ms | 214.52 µs | 124.74 ms | 126.54 ms | 49.4 MB | 362 B | 18.91 ms as a whole process |  |
| Wax | Normal | 31.07 ms | 340.48 µs | 30.60 ms | 31.40 ms | 2.9 MB | 169 B | 87.83 µs cold, 333 ns warm; 2.40 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled |
| Luau (interpreter) | Normal | 70.86 ms | 354.50 µs | 70.36 ms | 71.84 ms | 4.8 MB | 143 B | 5.17 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.87 ms | 67.49 µs | 33.70 ms | 34.01 ms | 1.9 MB | 143 B | 2.57 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.32 ms | 17.86 µs | 7.29 ms | 7.50 ms | 2.2 MB | 143 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 94.85 ms | 887.13 µs | 93.83 ms | 94.25 ms | 43.5 MB | 137 B | 19.16 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.50 ms | 130.90 µs | 9.38 ms | 9.89 ms | 48.8 MB | 137 B | 18.87 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.58 ms | 380.55 µs | 9.38 ms | 10.04 ms | 48.7 MB | 137 B | 18.90 ms as a whole process |  |
| Wax | Ceil.source | 31.18 ms | 444.43 µs | 30.31 ms | 29.74 ms | 2.9 MB | 169 B | 95.42 µs cold, 250 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 32.86 ms | 200.15 µs | 32.41 ms | 55.79 ms | 3.0 MB | 169 B | 1.10 ms cold, 542 ns warm; 2.64 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Wax | Ceil.program | 32.86 ms | 226.33 µs | 32.52 ms | 55.96 ms | 2.6 MB | 169 B | 877.92 µs cold, 667 ns warm; 2.63 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 29.99 ms | 300.41 µs | 29.43 ms | 53.85 ms | 2.6 MB | 169 B | 941.75 µs cold, 500 ns warm; 2.57 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.24 ms | 106.61 µs | 73.08 ms | 73.01 ms | 4.8 MB | 143 B | 5.10 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.78 ms | 78.66 µs | 33.65 ms | 34.03 ms | 1.9 MB | 143 B | 2.44 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.14 ms | 41.71 µs | 7.08 ms | 7.36 ms | 2.2 MB | 143 B | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.33 ms | 358.85 µs | 95.69 ms | 94.25 ms | 43.5 MB | 137 B | 19.23 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.63 ms | 308.28 µs | 9.39 ms | 11.61 ms | 48.9 MB | 137 B | 18.79 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.49 ms | 38.18 µs | 9.45 ms | 10.20 ms | 48.8 MB | 137 B | 18.80 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 49.45 ms | 436.27 µs | 48.91 ms | 49.31 ms | 2.9 MB | 532 B | 182.83 µs cold, 250 ns warm; 2.66 ms as a whole process | 512 B counted by Wax; 232 B compiled |
| Luau (interpreter) | Purpose | 92.16 ms | 502.35 µs | 91.55 ms | 91.95 ms | 4.8 MB | 433 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.74 ms | 188.25 µs | 76.52 ms | 76.33 ms | 1.9 MB | 433 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.79 ms | 34.64 µs | 6.73 ms | 6.79 ms | 2.1 MB | 433 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.00 ms | 1.34 ms | 241.72 ms | 242.63 ms | 46.5 MB | 460 B | 20.15 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.18 ms | 186.78 µs | 18.85 ms | 19.18 ms | 50.5 MB | 460 B | 19.90 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.00 ms | 177.03 µs | 18.78 ms | 19.60 ms | 50.5 MB | 460 B | 20.47 ms as a whole process |  |
| Wax | Normal | 66.75 ms | 2.92 ms | 64.35 ms | 67.59 ms | 3.5 MB | 3.4 KB | 298.58 µs cold, 333 ns warm; 3.09 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.28 ms | 997.91 µs | 69.55 ms | 70.45 ms | 4.8 MB | 3.0 KB | 5.85 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.53 ms | 459.26 µs | 71.04 ms | 71.37 ms | 1.9 MB | 3.0 KB | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.20 ms | 31.71 µs | 5.17 ms | 5.39 ms | 2.2 MB | 3.0 KB | 2.55 ms as a whole process |  |
| Node `--jitless` | Normal | 178.17 ms | 3.39 ms | 173.24 ms | 181.94 ms | 48.3 MB | 3.1 KB | 18.53 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.97 ms | 98.85 µs | 3.87 ms | 5.95 ms | 52.3 MB | 3.1 KB | 19.90 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.89 ms | 21.42 µs | 3.86 ms | 5.77 ms | 52.2 MB | 3.1 KB | 19.97 ms as a whole process |  |
| Wax | Ceil.source | 53.48 ms | 793.95 µs | 52.55 ms | 54.73 ms | 3.4 MB | 3.4 KB | 276.92 µs cold, 334 ns warm; 2.79 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Wax | Ceil.machine | 60.17 ms | 5.46 ms | 56.54 ms | 56.45 ms | 3.4 MB | 3.4 KB | 278.42 µs cold, 292 ns warm; 3.20 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Wax | Ceil.program | 54.67 ms | 7.39 ms | 49.16 ms | 53.67 ms | 3.1 MB | 3.4 KB | 328.75 µs cold, 250 ns warm; 3.23 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Wax | Ceil.machine.program | 46.83 ms | 1.08 ms | 45.80 ms | 47.20 ms | 3.1 MB | 3.4 KB | 283.25 µs cold, 208 ns warm; 2.96 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.78 ms | 304.41 µs | 69.25 ms | 70.40 ms | 4.8 MB | 3.0 KB | 5.39 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 75.90 ms | 3.69 ms | 72.50 ms | 72.84 ms | 1.9 MB | 3.0 KB | 3.00 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.56 ms | 264.01 µs | 5.31 ms | 5.63 ms | 2.2 MB | 3.0 KB | 2.80 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 182.00 ms | 16.83 ms | 171.71 ms | 207.28 ms | 48.3 MB | 3.1 KB | 19.56 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.89 ms | 28.91 µs | 3.84 ms | 5.73 ms | 52.2 MB | 3.1 KB | 20.94 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.92 ms | 19.66 µs | 3.88 ms | 6.86 ms | 52.3 MB | 3.1 KB | 19.65 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 55.48 ms | 2.25 ms | 50.18 ms | 59.83 ms | 3.1 MB | 393 B | 102.83 µs cold, 292 ns warm; 2.87 ms as a whole process | 8.5 KB counted by Wax; 224 B compiled |
| Luau (interpreter) | Purpose | 47.57 ms | 1.75 ms | 45.73 ms | 45.58 ms | 4.8 MB | 366 B | 5.29 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 79.18 ms | 1.83 ms | 77.30 ms | 76.96 ms | 2.0 MB | 366 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 23.79 ms | 76.22 µs | 23.68 ms | 23.67 ms | 2.2 MB | 366 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 379.51 ms | 15.66 ms | 363.06 ms | 391.02 ms | 48.8 MB | 398 B | 19.14 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.30 ms | 37.00 ms | 15.89 ms | 19.38 ms | 52.9 MB | 398 B | 18.94 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.36 ms | 335.89 µs | 14.19 ms | 19.89 ms | 55.7 MB | 398 B | 18.97 ms as a whole process |  |
| Wax | Normal | 18.16 ms | 343.52 µs | 17.57 ms | 19.16 ms | 3.2 MB | 1.2 KB | 152.12 µs cold, 292 ns warm; 2.76 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled |
| Luau (interpreter) | Normal | 28.48 ms | 373.44 µs | 28.14 ms | 28.21 ms | 4.9 MB | 981 B | 6.06 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.08 ms | 718.85 µs | 43.99 ms | 46.90 ms | 2.0 MB | 981 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.53 ms | 14.92 µs | 1.51 ms | 1.71 ms | 2.3 MB | 981 B | 2.38 ms as a whole process |  |
| Node `--jitless` | Normal | 64.84 ms | 1.02 ms | 63.39 ms | 62.82 ms | 48.5 MB | 1.0 KB | 19.50 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.57 ms | 175.00 µs | 1.42 ms | 3.29 ms | 53.0 MB | 1.0 KB | 19.07 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.61 ms | 29.74 µs | 1.57 ms | 3.52 ms | 53.0 MB | 1.0 KB | 19.06 ms as a whole process |  |
| Wax | Ceil.source | 23.51 ms | 12.18 ms | 17.22 ms | 17.86 ms | 3.2 MB | 1.2 KB | 151.58 µs cold, 250 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 18.63 ms | 133.67 µs | 18.45 ms | 18.69 ms | 3.2 MB | 1.2 KB | 157.54 µs cold, 334 ns warm; 2.43 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Wax | Ceil.program | 19.14 ms | 199.78 µs | 18.83 ms | 19.28 ms | 2.8 MB | 1.2 KB | 133.29 µs cold, 125 ns warm; 2.66 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 17.37 ms | 99.63 µs | 17.26 ms | 17.45 ms | 3.0 MB | 1.2 KB | 141.46 µs cold, 208 ns warm; 2.65 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.76 ms | 734.09 µs | 26.96 ms | 27.17 ms | 4.8 MB | 981 B | 5.45 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 46.44 ms | 872.18 µs | 45.03 ms | 47.26 ms | 2.0 MB | 981 B | 2.56 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.55 ms | 42.57 µs | 1.50 ms | 1.72 ms | 2.3 MB | 981 B | 2.40 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.84 ms | 465.48 µs | 62.32 ms | 64.07 ms | 48.6 MB | 1.0 KB | 18.53 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 2.18 ms | 978.58 µs | 1.52 ms | 4.18 ms | 53.0 MB | 1.0 KB | 19.26 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.64 ms | 19.20 µs | 1.61 ms | 3.96 ms | 53.1 MB | 1.0 KB | 18.87 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 91.30 ms | 2.29 ms | 88.59 ms | 90.46 ms | 3.0 MB | 596 B | 178.79 µs cold, 333 ns warm; 2.93 ms as a whole process | 688 B counted by Wax; 288 B compiled |
| Luau (interpreter) | Purpose | 87.60 ms | 1.29 ms | 85.40 ms | 92.56 ms | 4.8 MB | 523 B | 5.31 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.50 ms | 910.84 µs | 51.25 ms | 54.27 ms | 1.9 MB | 523 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.27 ms | 269.19 µs | 6.14 ms | 6.25 ms | 2.2 MB | 523 B | 2.56 ms as a whole process |  |
| Node `--jitless` | Purpose | 149.29 ms | 1.37 ms | 147.86 ms | 148.44 ms | 43.5 MB | 568 B | 18.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.70 ms | 71.90 µs | 3.64 ms | 4.69 ms | 49.5 MB | 568 B | 18.50 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.65 ms | 40.28 µs | 3.61 ms | 4.40 ms | 49.4 MB | 568 B | 19.27 ms as a whole process |  |
| Wax | Normal | 106.84 ms | 2.05 ms | 103.66 ms | 102.46 ms | 3.2 MB | 1.8 KB | 168.92 µs cold, 250 ns warm; 2.83 ms as a whole process | 984 B counted by Wax; 688 B compiled |
| Luau (interpreter) | Normal | 220.92 ms | 13.48 ms | 211.65 ms | 222.67 ms | 4.8 MB | 1.7 KB | 5.58 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 146.33 ms | 346.69 µs | 145.84 ms | 146.56 ms | 2.0 MB | 1.7 KB | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.60 ms | 472.12 µs | 47.70 ms | 47.68 ms | 2.3 MB | 1.7 KB | 2.58 ms as a whole process |  |
| Node `--jitless` | Normal | 154.04 ms | 755.65 µs | 153.03 ms | 157.05 ms | 45.6 MB | 1.7 KB | 20.18 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.69 ms | 110.25 µs | 14.51 ms | 16.52 ms | 52.3 MB | 1.7 KB | 19.76 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.97 ms | 115.25 µs | 14.83 ms | 16.88 ms | 52.4 MB | 1.7 KB | 19.61 ms as a whole process |  |
| Wax | Ceil.source | 104.02 ms | 759.58 µs | 103.19 ms | 104.58 ms | 3.2 MB | 1.8 KB | 161.54 µs cold, 333 ns warm; 2.74 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 106.58 ms | 977.52 µs | 104.91 ms | 105.35 ms | 3.2 MB | 1.8 KB | 153.71 µs cold, 333 ns warm; 2.78 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Wax | Ceil.program | 101.77 ms | 411.94 µs | 101.35 ms | 102.79 ms | 2.9 MB | 1.8 KB | 140.92 µs cold, 250 ns warm; 2.74 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 100.96 ms | 1.09 ms | 100.10 ms | 100.50 ms | 2.9 MB | 1.8 KB | 131.54 µs cold, 208 ns warm; 2.81 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 212.94 ms | 2.06 ms | 208.71 ms | 211.09 ms | 4.8 MB | 1.7 KB | 5.46 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 147.56 ms | 692.67 µs | 146.63 ms | 145.36 ms | 2.0 MB | 1.7 KB | 2.56 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 47.82 ms | 465.18 µs | 47.20 ms | 48.29 ms | 2.3 MB | 1.7 KB | 2.52 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 155.55 ms | 1.35 ms | 154.27 ms | 155.46 ms | 45.6 MB | 1.7 KB | 18.22 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 13.94 ms | 144.35 µs | 13.72 ms | 15.39 ms | 52.3 MB | 1.7 KB | 19.34 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.03 ms | 69.10 µs | 13.95 ms | 15.51 ms | 52.4 MB | 1.7 KB | 18.32 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 125.11 ms | 796.88 µs | 124.32 ms | 124.52 ms | 4.2 MB | 533 B | 146.08 µs cold, 500 ns warm; 2.39 ms as a whole process | 793.6 KB counted by Wax; 336 B compiled |
| Luau (interpreter) | Purpose | 62.57 ms | 431.08 µs | 62.01 ms | 62.45 ms | 4.8 MB | 540 B | 5.20 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 29.60 ms | 85.17 µs | 29.45 ms | 30.77 ms | 2.2 MB | 540 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.85 ms | 18.48 µs | 4.83 ms | 4.95 ms | 2.4 MB | 540 B | 2.26 ms as a whole process |  |
| Node `--jitless` | Purpose | 235.47 ms | 854.03 µs | 233.96 ms | 236.87 ms | 48.9 MB | 514 B | 18.74 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 88.59 ms | 24.25 ms | 66.34 ms | 91.54 ms | 52.9 MB | 514 B | 18.29 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 68.82 ms | 640.30 µs | 68.10 ms | 89.86 ms | 52.9 MB | 514 B | 19.56 ms as a whole process |  |
| Wax | Normal | 26.54 ms | 190.76 µs | 26.28 ms | 26.50 ms | 4.2 MB | 538 B | 121.50 µs cold, 416 ns warm; 2.64 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled |
| Luau (interpreter) | Normal | 29.78 ms | 193.22 µs | 29.36 ms | 29.81 ms | 6.0 MB | 478 B | 5.20 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 30.98 ms | 367.44 µs | 30.37 ms | 28.84 ms | 3.1 MB | 478 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.53 ms | 320.55 µs | 12.04 ms | 13.46 ms | 3.4 MB | 478 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Normal | 50.94 ms | 210.44 µs | 50.56 ms | 52.77 ms | 56.7 MB | 477 B | 18.53 ms as a whole process |  |
| Node (JIT), from cold | Normal | 34.70 ms | 215.10 µs | 34.46 ms | 35.11 ms | 62.7 MB | 477 B | 18.58 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.39 ms | 208.32 µs | 35.12 ms | 35.22 ms | 88.3 MB | 477 B | 19.15 ms as a whole process |  |
| Wax | Ceil.source | 26.30 ms | 86.90 µs | 26.13 ms | 26.57 ms | 4.2 MB | 538 B | 124.79 µs cold, 334 ns warm; 2.70 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 27.25 ms | 517.03 µs | 26.85 ms | 27.20 ms | 4.2 MB | 538 B | 108.42 µs cold, 333 ns warm; 2.80 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Wax | Ceil.program | 24.04 ms | 252.07 µs | 23.64 ms | 24.39 ms | 4.0 MB | 538 B | 100.12 µs cold, 291 ns warm; 2.68 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 23.74 ms | 173.07 µs | 23.51 ms | 23.83 ms | 4.0 MB | 538 B | 88.29 µs cold, 250 ns warm; 2.60 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.33 ms | 307.99 µs | 29.91 ms | 29.95 ms | 6.0 MB | 478 B | 5.31 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.51 ms | 719.83 µs | 30.39 ms | 30.39 ms | 3.1 MB | 478 B | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 13.23 ms | 1.07 ms | 12.01 ms | 12.60 ms | 3.4 MB | 478 B | 2.60 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.27 ms | 237.98 µs | 51.00 ms | 52.93 ms | 56.7 MB | 477 B | 19.57 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.78 ms | 332.30 µs | 34.24 ms | 35.66 ms | 62.7 MB | 477 B | 19.10 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 34.98 ms | 269.07 µs | 34.72 ms | 35.00 ms | 88.0 MB | 477 B | 18.65 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 304.71 µs | 5.11 µs | 296.50 µs | 326.50 µs | 3.3 MB | 215 B | 113.42 µs cold, 459 ns warm; 2.61 ms as a whole process | 64.6 KB counted by Wax; 96 B compiled |
| Luau (interpreter) | Purpose | 144.14 ms | 254.76 µs | 143.82 ms | 145.59 ms | 9.9 MB | 195 B | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.39 ms | 105.44 µs | 32.27 ms | 31.09 ms | 3.5 MB | 195 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 21.10 ms | 483.03 µs | 20.55 ms | 20.89 ms | 3.8 MB | 195 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Purpose | 363.30 µs | 143.82 µs | 236.04 µs | 1.06 ms | 52.6 MB | 223 B | 18.81 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 209.50 µs | 144.99 µs | 86.25 µs | 879.21 µs | 57.9 MB | 223 B | 18.42 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 132.10 µs | 118.68 µs | 54.83 µs | 890.33 µs | 58.0 MB | 223 B | 17.91 ms as a whole process |  |
| Wax | Normal | 332.58 µs | 2.23 µs | 329.25 µs | 356.79 µs | 3.4 MB | 182 B | 80.12 µs cold, 417 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled |
| Luau (interpreter) | Normal | 161.31 ms | 492.45 µs | 160.58 ms | 163.52 ms | 12.1 MB | 138 B | 5.32 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.30 ms | 266.14 µs | 40.83 ms | 41.42 ms | 4.9 MB | 138 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.16 ms | 218.08 µs | 18.83 ms | 20.12 ms | 5.3 MB | 138 B | 2.56 ms as a whole process |  |
| Node `--jitless` | Normal | 508.88 µs | 343.77 µs | 265.96 µs | 1.12 ms | 61.6 MB | 155 B | 18.55 ms as a whole process |  |
| Node (JIT), from cold | Normal | 492.11 µs | 484.29 µs | 123.42 µs | 1.06 ms | 65.4 MB | 155 B | 18.80 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 279.45 µs | 332.07 µs | 123.29 µs | 1.07 ms | 69.5 MB | 155 B | 18.59 ms as a whole process |  |
| Wax | Ceil.source | 335.37 µs | 4.49 µs | 331.79 µs | 365.54 µs | 3.4 MB | 182 B | 79.12 µs cold, 375 ns warm; 2.58 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 351.75 µs | 5.39 µs | 346.54 µs | 399.42 µs | 3.5 MB | 182 B | 80.83 µs cold, 333 ns warm; 2.66 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Wax | Ceil.program | 281.03 µs | 8.52 µs | 273.92 µs | 312.96 µs | 3.1 MB | 182 B | 66.04 µs cold, 208 ns warm; 2.49 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 279.45 µs | 3.63 µs | 275.54 µs | 301.29 µs | 3.2 MB | 182 B | 62.21 µs cold, 292 ns warm; 2.52 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.25 ms | 427.85 µs | 160.54 ms | 164.01 ms | 11.8 MB | 138 B | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.95 ms | 188.41 µs | 41.65 ms | 41.33 ms | 5.1 MB | 138 B | 2.35 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.28 ms | 131.91 µs | 19.08 ms | 20.14 ms | 5.3 MB | 138 B | 2.48 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 500.13 µs | 340.55 µs | 257.46 µs | 1.11 ms | 61.5 MB | 155 B | 18.41 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 487.28 µs | 465.57 µs | 134.00 µs | 1.03 ms | 65.4 MB | 155 B | 18.16 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 279.52 µs | 333.77 µs | 124.79 µs | 1.08 ms | 69.5 MB | 155 B | 19.02 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 74.23 ms | 1.24 ms | 72.94 ms | 75.51 ms | 3.2 MB | 519 B | 144.08 µs cold, 333 ns warm; 2.68 ms as a whole process | 65.9 KB counted by Wax; 216 B compiled |
| Luau (interpreter) | Purpose | 94.37 ms | 509.23 µs | 93.63 ms | 95.22 ms | 4.8 MB | 411 B | 5.09 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 117.84 ms | 1.48 ms | 113.88 ms | 119.21 ms | 2.6 MB | 411 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.74 ms | 1.09 ms | 92.63 ms | 93.98 ms | 2.9 MB | 411 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Purpose | 78.75 ms | 449.36 µs | 78.02 ms | 82.66 ms | 59.8 MB | 444 B | 18.23 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.38 ms | 277.60 µs | 10.12 ms | 12.90 ms | 66.0 MB | 444 B | 18.90 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.34 ms | 602.52 µs | 10.03 ms | 12.81 ms | 114.9 MB | 444 B | 19.78 ms as a whole process |  |
| Wax | Normal | 100.53 ms | 757.75 µs | 99.51 ms | 99.99 ms | 8.0 MB | 956 B | 130.58 µs cold, 208 ns warm; 2.73 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled |
| Luau (interpreter) | Normal | 135.72 ms | 1.82 ms | 132.46 ms | 132.98 ms | 43.8 MB | 799 B | 5.20 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 127.86 ms | 1.39 ms | 126.22 ms | 129.19 ms | 39.3 MB | 799 B | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.12 ms | 1.11 ms | 78.82 ms | 80.80 ms | 44.0 MB | 799 B | 2.31 ms as a whole process |  |
| Node `--jitless` | Normal | 114.16 ms | 1.38 ms | 112.64 ms | 114.68 ms | 213.5 MB | 878 B | 18.12 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.08 ms | 835.72 µs | 13.19 ms | 17.04 ms | 129.9 MB | 878 B | 18.65 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.64 ms | 90.50 µs | 13.53 ms | 16.80 ms | 224.1 MB | 878 B | 19.92 ms as a whole process |  |
| Wax | Ceil.source | 106.48 ms | 8.66 ms | 100.54 ms | 102.51 ms | 8.0 MB | 956 B | 128.79 µs cold, 333 ns warm; 2.86 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 102.67 ms | 1.41 ms | 100.77 ms | 103.49 ms | 8.0 MB | 956 B | 126.50 µs cold, 292 ns warm; 2.74 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Wax | Ceil.program | 99.58 ms | 942.10 µs | 98.10 ms | 100.34 ms | 7.7 MB | 956 B | 108.75 µs cold, 208 ns warm; 2.95 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 95.40 ms | 778.88 µs | 93.74 ms | 95.33 ms | 7.8 MB | 956 B | 108.25 µs cold, 167 ns warm; 2.87 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 140.61 ms | 5.75 ms | 131.03 ms | 134.32 ms | 44.0 MB | 799 B | 5.32 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 127.82 ms | 1.22 ms | 125.50 ms | 126.21 ms | 41.6 MB | 799 B | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 83.01 ms | 1.28 ms | 80.52 ms | 82.34 ms | 41.8 MB | 799 B | 2.66 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 114.62 ms | 2.21 ms | 112.92 ms | 118.50 ms | 213.5 MB | 878 B | 18.72 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.22 ms | 913.98 µs | 13.31 ms | 17.05 ms | 129.9 MB | 878 B | 18.30 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.48 ms | 61.05 µs | 13.40 ms | 17.08 ms | 224.1 MB | 878 B | 19.46 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed three ways: every script from a text of its own, so that nothing compiled is reused; every script from the same text, where Wax reuses what it compiled, and the other engines compile once and make each script from that; and every script from a compiled file of its own, a `.waxc` for Wax and `string.dump`'s bytecode, as `luajit -b` writes it, for LuaJIT (D171). Wax frees each script loaded from a text or a file of its own inside its timing, before loading the next.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 69.65 µs (±12.86 µs, least 54.21 µs) | 20.02 µs (±8.28 µs, least 13.12 µs) | 11.45 µs (±6.07 µs, least 7.21 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 4.3 MB | counted by Wax |
| Luau (interpreter) | Purpose | 10.79 µs (±1.76 µs, least 8.96 µs) | 16.83 µs (±1.59 µs, least 15.38 µs) | ? | 40 ns (±1 ns, least 38 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.70 µs (±823 ns, least 3.00 µs) | 7.40 µs (±2.12 µs, least 5.00 µs) | 1.40 µs (±516 ns, least 1.00 µs) | 21 ns (±1 ns, least 20 ns) | 16 ns (±2 ns, least 14 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.80 µs (±632 ns, least 3.00 µs) | 8.50 µs (±1.08 µs, least 7.00 µs) | 1.20 µs (±422 ns, least 1.00 µs) | 13 ns (±0 ns, least 12 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 34.55 µs (±4.82 µs, least 21.54 µs) | 64.08 µs (±81.45 µs, least 34.83 µs) | ? | 57 ns (±1 ns, least 55 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.9 MB | the engine's own heap |
| Node (JIT) | Purpose | 30.30 µs (±8.11 µs, least 17.88 µs) | 39.88 µs (±2.67 µs, least 36.29 µs) | ? | 8 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.2 MB | the engine's own heap |
| Wax | Normal | 4.45 µs (±42 ns, least 4.41 µs) | 157 ns (±6 ns, least 149 ns) | 536 ns (±9 ns, least 521 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±1 ns, least 40 ns) | 512 B | 10.3 MB | counted by Wax |
| Luau (interpreter) | Normal | 4.90 µs (±157 ns, least 4.71 µs) | 191 ns (±8 ns, least 181 ns) | ? | 33 ns (±1 ns, least 32 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.79 µs (±42 ns, least 1.75 µs) | 184 ns (±4 ns, least 178 ns) | 288 ns (±14 ns, least 275 ns) | 28 ns (±1 ns, least 27 ns) | 15 ns (±1 ns, least 14 ns) | 1.0 KB | 5.2 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.84 µs (±41 ns, least 1.78 µs) | 361 ns (±21 ns, least 339 ns) | 499 ns (±51 ns, least 359 ns) | 96 ns (±3 ns, least 92 ns) | 103 ns (±5 ns, least 100 ns) | 1.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±27 ns, least 1.11 µs) | 684 ns (±23 ns, least 635 ns) | ? | 51 ns (±2 ns, least 49 ns) | 25 ns (±4 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±36 ns, least 1.07 µs) | 678 ns (±18 ns, least 639 ns) | ? | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap |
| Wax | Ceil.source | 4.50 µs (±38 ns, least 4.44 µs) | 161 ns (±8 ns, least 148 ns) | 538 ns (±11 ns, least 521 ns) | 21 ns (±2 ns, least 20 ns) | 41 ns (±0 ns, least 40 ns) | 512 B | 10.3 MB | counted by Wax; the Normal program stands in |
| Wax | Ceil.machine | 4.54 µs (±66 ns, least 4.45 µs) | 174 ns (±13 ns, least 158 ns) | 525 ns (±13 ns, least 512 ns) | 21 ns (±0 ns, least 20 ns) | 42 ns (±1 ns, least 41 ns) | 512 B | 10.3 MB | counted by Wax; the Normal program stands in |
| Wax | Ceil.program | 3.82 µs (±36 ns, least 3.79 µs) | 150 ns (±7 ns, least 138 ns) | 451 ns (±16 ns, least 439 ns) | 17 ns (±1 ns, least 16 ns) | 36 ns (±1 ns, least 36 ns) | 512 B | 10.1 MB | counted by Wax; the Normal program stands in |
| Wax | Ceil.machine.program | 3.88 µs (±48 ns, least 3.82 µs) | 149 ns (±8 ns, least 137 ns) | 452 ns (±18 ns, least 432 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 10.0 MB | counted by Wax; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.87 µs (±222 ns, least 4.67 µs) | 187 ns (±13 ns, least 177 ns) | ? | 34 ns (±1 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.78 µs (±21 ns, least 1.75 µs) | 181 ns (±6 ns, least 173 ns) | 287 ns (±10 ns, least 273 ns) | 27 ns (±0 ns, least 27 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.80 µs (±27 ns, least 1.77 µs) | 359 ns (±32 ns, least 330 ns) | 486 ns (±82 ns, least 254 ns) | 95 ns (±9 ns, least 89 ns) | 106 ns (±16 ns, least 98 ns) | 1.0 KB | 6.0 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.14 µs (±22 ns, least 1.09 µs) | 671 ns (±20 ns, least 635 ns) | ? | 50 ns (±2 ns, least 49 ns) | 24 ns (±2 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±12 ns, least 1.08 µs) | 689 ns (±18 ns, least 662 ns) | ? | 22 ns (±0 ns, least 21 ns) | 6 ns (±1 ns, least 6 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Wax's checks cost (Purpose)

D58 asks for the cost of Wax's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 31.08 ms | 30.13 ms (-3.1%) |
| n-body, 100,000 steps | 66.23 ms | 65.05 ms (-1.8%) |
| spectral-norm, size 200 | 18.49 ms | 18.13 ms (-1.9%) |
| fannkuch-redux, 9 | 106.63 ms | 108.38 ms (+1.6%) |
| word count, 500,000 words | 26.96 ms | 27.38 ms (+1.5%) |
| text building, 20,000 joins | 339.56 µs | 345.22 µs (+1.7%) |
| binary-trees, depth 14 | 102.24 ms | 97.82 ms (-4.3%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Wax mean | Wax deviation | Wax least | Wax peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.39 ms | 41.26 µs | 1.35 ms | 3.9 MB | 1.58 ms | 1.56 ms | 301.50 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.40 ms | 20.58 µs | 1.37 ms | 3.7 MB | 1.52 ms | 1.50 ms | 1.11 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 116.05 ms | 918.59 µs | 115.04 ms | 3.0 MB | 161.82 ms | 161.14 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 71.17 ms | 275.59 µs | 70.95 ms | 3.0 MB | 82.68 ms | 82.00 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 441.50 µs | 13.14 µs | 420.04 µs | 3.6 MB | 908.04 µs | 896.00 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 96.01 ms | 2.21 ms | 92.17 ms | 3.0 MB | 165.39 ms | 163.47 ms | – |
| the money sums in `Float`s, as a yardstick | 10.89 ms | 214.34 µs | 10.59 ms | 2.9 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Wax's door is written in C, through Wax's C API (`crates/wax-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Wax's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Wax | C | 12 ns | |
| Wax | Rust | 8 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
