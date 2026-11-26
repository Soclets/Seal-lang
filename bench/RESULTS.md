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
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.01 2.63 2.55 }` before, `{ 1.70 2.18 2.38 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 34.69 ms | 70.91 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 63.88 ms | 68.77 ms | **met**: Seal takes 93% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.86 ms | 26.80 ms | **met**: Seal takes 70% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 102.82 ms | 212.23 ms | **met**: Seal takes 48% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.54 ms | 29.72 ms | **met**: Seal takes 89% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 336.42 µs | 158.92 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 101.45 ms | 128.60 ms | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.99 ms | 3.56 ms | **met**: Seal takes 56% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 18 ns | 34 ns | **met**: Seal takes 54% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 146 ns | 176 ns | **met**: Seal takes 83% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.68 µs | 4.72 µs | **met**: Seal takes 99% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 34.69 ms | 33.60 ms | **not met**: Seal takes 103% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 63.88 ms | 71.00 ms | **met**: Seal takes 90% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.86 ms | 46.73 ms | **met**: Seal takes 40% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 102.82 ms | 148.17 ms | **met**: Seal takes 69% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.54 ms | 30.01 ms | **met**: Seal takes 88% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 336.42 µs | 40.94 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 101.45 ms | 136.11 ms | **met**: Seal takes 75% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.99 ms | 2.88 ms | **met**: Seal takes 69% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 18 ns | 27 ns | **met**: Seal takes 68% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 146 ns | 174 ns | **met**: Seal takes 84% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.68 µs | 1.77 µs | for reference, no target (D150) |
| D171, D176. Within twice LuaJIT loading its own bytecode | patrolling scripts, loading each from a compiled file of its own | 662 ns | 274 ns | **not met**: Seal takes 242% of its time |
| D176. Loading the same compiled file again | patrolling scripts, each loaded from one `.sealc` file, reusing what was verified | 153 ns | | for reference, no target |
| D171. Checking a `.sealc` file's signature, once per file | patrolling scripts, loading each signed file of its own, signature checked | 39.58 µs | | for reference, no target |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 14 ns | a Rust door: 8 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.8× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 1.57 ms | **met**: Seal takes 86% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.34 ms | 1.51 ms | **met**: Seal takes 89% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 111.26 ms | 238.55 ms | **met**: Seal takes 47% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 67.23 ms | 83.80 ms | **met**: Seal takes 80% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 412.35 µs | 888.95 µs | **met**: Seal takes 46% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 93.33 ms | 160.51 ms | **met**: Seal takes 58% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 34.69 ms | 93.90 ms | **met**: Seal takes 37% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 63.88 ms | 173.45 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.54 ms | 51.00 ms | **met**: Seal takes 52% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 336.42 µs | 493.65 µs | **met**: Seal takes 68% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 18 ns | host call 39 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 85.65 ms | 1.01 ms | 84.32 ms | 107.30 ms | 2.9 MB | 377 B | 812.33 µs cold, 834 ns warm; 2.74 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 193.97 ms | 208.29 µs | 193.63 ms | 194.89 ms | 4.8 MB | 339 B | 5.07 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.12 ms | 1.32 ms | 87.00 ms | 90.64 ms | 1.9 MB | 339 B | 2.61 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 43.93 ms | 269.07 µs | 43.46 ms | 45.72 ms | 2.3 MB | 339 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Purpose | 322.69 ms | 1.77 ms | 320.90 ms | 321.13 ms | 43.5 MB | 362 B | 18.86 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.33 ms | 209.99 µs | 122.03 ms | 127.16 ms | 49.5 MB | 362 B | 18.95 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 121.15 ms | 728.72 µs | 120.50 ms | 122.35 ms | 49.3 MB | 362 B | 18.97 ms as a whole process |  |
| Seal | Normal | 34.69 ms | 704.32 µs | 33.66 ms | 35.95 ms | 2.8 MB | 169 B | 80.96 µs cold, 250 ns warm; 2.67 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled |
| Luau (interpreter) | Normal | 70.91 ms | 339.10 µs | 70.47 ms | 71.78 ms | 4.8 MB | 143 B | 5.10 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.60 ms | 117.88 µs | 33.45 ms | 33.22 ms | 1.9 MB | 143 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.26 ms | 14.83 µs | 7.24 ms | 7.71 ms | 2.2 MB | 143 B | 2.21 ms as a whole process |  |
| Node `--jitless` | Normal | 93.90 ms | 536.79 µs | 92.93 ms | 94.64 ms | 43.4 MB | 137 B | 18.23 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.79 ms | 670.78 µs | 9.34 ms | 10.84 ms | 48.7 MB | 137 B | 18.18 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.55 ms | 469.19 µs | 9.35 ms | 9.71 ms | 48.8 MB | 137 B | 18.50 ms as a whole process |  |
| Seal | Ceil.source | 35.57 ms | 1.14 ms | 34.13 ms | 34.89 ms | 2.8 MB | 169 B | 80.75 µs cold, 250 ns warm; 2.68 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 29.17 ms | 179.26 µs | 28.89 ms | 41.00 ms | 2.8 MB | 169 B | 896.33 µs cold, 375 ns warm; 2.46 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.program | 28.70 ms | 672.85 µs | 27.87 ms | 51.82 ms | 2.6 MB | 169 B | 686.79 µs cold, 292 ns warm; 2.60 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 27.46 ms | 217.28 µs | 27.24 ms | 52.91 ms | 2.7 MB | 169 B | 953.00 µs cold, 375 ns warm; 2.60 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 69.80 ms | 133.53 µs | 69.63 ms | 71.36 ms | 4.8 MB | 143 B | 5.10 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.43 ms | 86.72 µs | 33.26 ms | 33.51 ms | 1.9 MB | 143 B | 2.29 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.03 ms | 30.51 µs | 6.96 ms | 7.54 ms | 2.2 MB | 143 B | 2.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 94.58 ms | 452.96 µs | 93.75 ms | 94.64 ms | 43.5 MB | 137 B | 18.24 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.74 ms | 607.44 µs | 9.34 ms | 9.85 ms | 48.8 MB | 137 B | 18.81 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.43 ms | 54.69 µs | 9.32 ms | 11.64 ms | 48.8 MB | 137 B | 18.95 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.08 ms | 1.31 ms | 49.77 ms | 54.06 ms | 2.9 MB | 532 B | 117.58 µs cold, 292 ns warm; 2.67 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.18 ms | 562.52 µs | 90.58 ms | 91.04 ms | 4.8 MB | 433 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.12 ms | 672.61 µs | 74.78 ms | 75.79 ms | 1.9 MB | 433 B | 2.40 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.60 ms | 84.30 µs | 6.45 ms | 6.66 ms | 2.1 MB | 433 B | 2.29 ms as a whole process |  |
| Node `--jitless` | Purpose | 238.81 ms | 286.95 µs | 238.44 ms | 239.36 ms | 46.4 MB | 460 B | 18.36 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.42 ms | 186.85 µs | 18.11 ms | 18.53 ms | 50.5 MB | 460 B | 17.92 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.22 ms | 86.02 µs | 18.04 ms | 18.40 ms | 50.3 MB | 460 B | 18.00 ms as a whole process |  |
| Seal | Normal | 63.88 ms | 716.37 µs | 63.04 ms | 64.75 ms | 3.4 MB | 3.4 KB | 290.12 µs cold, 250 ns warm; 2.69 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 68.77 ms | 165.81 µs | 68.53 ms | 69.02 ms | 4.8 MB | 3.0 KB | 5.26 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.00 ms | 436.43 µs | 70.57 ms | 70.40 ms | 1.9 MB | 3.0 KB | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.15 ms | 22.98 µs | 5.12 ms | 5.34 ms | 2.2 MB | 3.0 KB | 2.42 ms as a whole process |  |
| Node `--jitless` | Normal | 173.45 ms | 3.50 ms | 169.48 ms | 178.88 ms | 48.1 MB | 3.1 KB | 18.59 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.84 ms | 14.76 µs | 3.82 ms | 5.88 ms | 52.0 MB | 3.1 KB | 18.56 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.84 ms | 14.69 µs | 3.82 ms | 5.61 ms | 52.2 MB | 3.1 KB | 18.71 ms as a whole process |  |
| Seal | Ceil.source | 54.96 ms | 490.61 µs | 53.95 ms | 54.60 ms | 3.3 MB | 3.4 KB | 258.71 µs cold, 416 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.64 ms | 517.13 µs | 54.81 ms | 55.00 ms | 3.4 MB | 3.4 KB | 274.08 µs cold, 291 ns warm; 2.75 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.47 ms | 343.91 µs | 49.99 ms | 50.42 ms | 3.2 MB | 3.4 KB | 261.17 µs cold, 250 ns warm; 2.87 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 48.99 ms | 525.88 µs | 48.04 ms | 49.97 ms | 3.1 MB | 3.4 KB | 261.58 µs cold, 292 ns warm; 2.70 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 68.89 ms | 164.02 µs | 68.65 ms | 69.42 ms | 4.8 MB | 3.0 KB | 5.12 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.15 ms | 537.73 µs | 70.16 ms | 70.91 ms | 1.9 MB | 3.0 KB | 2.22 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.17 ms | 16.98 µs | 5.15 ms | 5.35 ms | 2.2 MB | 3.0 KB | 2.47 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 174.92 ms | 4.29 ms | 170.25 ms | 179.14 ms | 48.1 MB | 3.1 KB | 18.40 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.83 ms | 16.00 µs | 3.82 ms | 5.59 ms | 52.1 MB | 3.1 KB | 18.46 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.83 ms | 13.38 µs | 3.81 ms | 5.58 ms | 52.2 MB | 3.1 KB | 18.25 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.10 ms | 883.61 µs | 47.11 ms | 50.65 ms | 3.0 MB | 393 B | 92.38 µs cold, 209 ns warm; 2.51 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 44.81 ms | 276.92 µs | 44.52 ms | 44.53 ms | 4.8 MB | 366 B | 4.99 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.45 ms | 71.23 µs | 76.37 ms | 76.34 ms | 2.0 MB | 366 B | 2.17 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.13 ms | 137.94 µs | 21.88 ms | 22.40 ms | 2.2 MB | 366 B | 2.33 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.65 ms | 4.09 ms | 364.87 ms | 374.41 ms | 48.6 MB | 398 B | 18.96 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 33.97 ms | 36.20 ms | 15.44 ms | 21.70 ms | 52.9 MB | 398 B | 19.00 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.20 ms | 61.07 µs | 14.16 ms | 18.60 ms | 55.2 MB | 398 B | 18.11 ms as a whole process |  |
| Seal | Normal | 18.86 ms | 199.92 µs | 18.57 ms | 19.41 ms | 3.1 MB | 1.2 KB | 148.96 µs cold, 250 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 26.80 ms | 258.23 µs | 26.34 ms | 29.54 ms | 4.8 MB | 981 B | 5.08 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 46.73 ms | 722.77 µs | 45.57 ms | 45.02 ms | 2.0 MB | 981 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.52 ms | 12.44 µs | 1.51 ms | 1.69 ms | 2.3 MB | 981 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Normal | 63.03 ms | 429.09 µs | 62.42 ms | 62.76 ms | 48.5 MB | 1.0 KB | 18.55 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.59 ms | 174.08 µs | 1.47 ms | 3.14 ms | 53.0 MB | 1.0 KB | 18.31 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.63 ms | 20.22 µs | 1.59 ms | 3.32 ms | 53.0 MB | 1.0 KB | 18.40 ms as a whole process |  |
| Seal | Ceil.source | 18.49 ms | 177.85 µs | 18.20 ms | 18.45 ms | 3.2 MB | 1.2 KB | 150.92 µs cold, 250 ns warm; 2.63 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.08 ms | 300.55 µs | 17.78 ms | 18.07 ms | 3.2 MB | 1.2 KB | 150.00 µs cold, 250 ns warm; 2.61 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.18 ms | 299.96 µs | 18.75 ms | 19.26 ms | 3.0 MB | 1.2 KB | 132.92 µs cold, 208 ns warm; 2.59 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.95 ms | 295.94 µs | 18.53 ms | 19.47 ms | 3.0 MB | 1.2 KB | 134.58 µs cold, 250 ns warm; 2.56 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.01 ms | 414.25 µs | 26.44 ms | 27.25 ms | 4.8 MB | 981 B | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 46.89 ms | 818.48 µs | 46.22 ms | 45.77 ms | 2.0 MB | 981 B | 2.37 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.53 ms | 25.87 µs | 1.51 ms | 1.68 ms | 2.3 MB | 981 B | 2.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.64 ms | 349.92 µs | 62.26 ms | 63.40 ms | 48.6 MB | 1.0 KB | 18.36 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.54 ms | 230.46 µs | 1.39 ms | 3.18 ms | 53.0 MB | 1.0 KB | 18.69 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.62 ms | 33.10 µs | 1.58 ms | 3.23 ms | 52.8 MB | 1.0 KB | 18.16 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 82.23 ms | 333.45 µs | 81.68 ms | 82.45 ms | 2.9 MB | 596 B | 97.71 µs cold, 250 ns warm; 2.58 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 86.79 ms | 769.28 µs | 85.21 ms | 90.36 ms | 4.8 MB | 523 B | 5.02 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.61 ms | 876.07 µs | 51.76 ms | 52.36 ms | 1.9 MB | 523 B | 2.34 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.28 ms | 290.21 µs | 6.08 ms | 6.17 ms | 2.2 MB | 523 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Purpose | 147.13 ms | 551.20 µs | 146.35 ms | 148.41 ms | 43.3 MB | 568 B | 19.51 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.65 ms | 43.65 µs | 3.61 ms | 4.70 ms | 49.4 MB | 568 B | 18.53 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.62 ms | 24.29 µs | 3.61 ms | 4.49 ms | 49.4 MB | 568 B | 18.24 ms as a whole process |  |
| Seal | Normal | 102.82 ms | 251.64 µs | 102.34 ms | 102.75 ms | 3.1 MB | 1.8 KB | 159.92 µs cold, 333 ns warm; 2.66 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 212.23 ms | 1.91 ms | 206.90 ms | 214.24 ms | 4.8 MB | 1.7 KB | 5.08 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 148.17 ms | 620.22 µs | 146.63 ms | 143.98 ms | 2.0 MB | 1.7 KB | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.70 ms | 529.76 µs | 47.82 ms | 48.33 ms | 2.3 MB | 1.7 KB | 2.49 ms as a whole process |  |
| Node `--jitless` | Normal | 153.97 ms | 929.10 µs | 152.48 ms | 155.94 ms | 45.5 MB | 1.7 KB | 18.77 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.57 ms | 84.21 µs | 14.42 ms | 15.98 ms | 52.1 MB | 1.7 KB | 17.90 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.82 ms | 105.83 µs | 13.69 ms | 15.26 ms | 52.2 MB | 1.7 KB | 18.42 ms as a whole process |  |
| Seal | Ceil.source | 101.56 ms | 530.14 µs | 100.95 ms | 101.89 ms | 3.1 MB | 1.8 KB | 158.75 µs cold, 208 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 101.91 ms | 286.29 µs | 101.50 ms | 103.14 ms | 3.2 MB | 1.8 KB | 157.00 µs cold, 292 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 110.35 ms | 182.10 µs | 110.04 ms | 111.64 ms | 2.9 MB | 1.8 KB | 126.54 µs cold, 208 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 109.53 ms | 369.96 µs | 109.03 ms | 109.41 ms | 3.0 MB | 1.8 KB | 140.62 µs cold, 209 ns warm; 2.65 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 208.42 ms | 2.15 ms | 202.40 ms | 210.00 ms | 4.8 MB | 1.7 KB | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 146.87 ms | 303.46 µs | 146.40 ms | 144.96 ms | 2.0 MB | 1.7 KB | 2.44 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.33 ms | 350.25 µs | 47.78 ms | 48.50 ms | 2.3 MB | 1.7 KB | 2.45 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 153.84 ms | 519.05 µs | 153.16 ms | 154.82 ms | 45.5 MB | 1.7 KB | 18.25 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.57 ms | 53.61 µs | 14.49 ms | 16.05 ms | 52.3 MB | 1.7 KB | 18.47 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.11 ms | 122.10 µs | 13.99 ms | 15.61 ms | 52.3 MB | 1.7 KB | 18.18 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 129.71 ms | 903.83 µs | 128.82 ms | 129.89 ms | 4.1 MB | 533 B | 99.96 µs cold, 417 ns warm; 2.65 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.42 ms | 836.45 µs | 62.58 ms | 62.77 ms | 4.8 MB | 540 B | 5.04 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 27.69 ms | 74.07 µs | 27.55 ms | 28.06 ms | 2.2 MB | 540 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.99 ms | 43.69 µs | 4.93 ms | 5.15 ms | 2.4 MB | 540 B | 2.30 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.44 ms | 903.40 µs | 244.03 ms | 245.85 ms | 48.6 MB | 514 B | 18.23 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 84.82 ms | 23.61 ms | 63.26 ms | 88.86 ms | 52.7 MB | 514 B | 18.37 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 68.21 ms | 366.23 µs | 67.85 ms | 90.36 ms | 52.7 MB | 514 B | 18.46 ms as a whole process |  |
| Seal | Normal | 26.54 ms | 395.55 µs | 26.27 ms | 26.17 ms | 4.1 MB | 538 B | 109.04 µs cold, 416 ns warm; 2.74 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.72 ms | 67.90 µs | 29.64 ms | 29.60 ms | 6.0 MB | 478 B | 5.18 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 30.01 ms | 745.77 µs | 29.00 ms | 29.16 ms | 3.1 MB | 478 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.61 ms | 1.03 ms | 11.41 ms | 12.73 ms | 3.4 MB | 478 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Normal | 51.00 ms | 191.33 µs | 50.70 ms | 52.18 ms | 56.4 MB | 477 B | 18.54 ms as a whole process |  |
| Node (JIT), from cold | Normal | 34.96 ms | 224.63 µs | 34.75 ms | 35.77 ms | 62.7 MB | 477 B | 18.80 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.09 ms | 273.35 µs | 34.70 ms | 35.53 ms | 88.1 MB | 477 B | 18.57 ms as a whole process |  |
| Seal | Ceil.source | 26.10 ms | 266.42 µs | 25.77 ms | 26.10 ms | 4.1 MB | 538 B | 115.58 µs cold, 292 ns warm; 2.67 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 26.54 ms | 93.44 µs | 26.36 ms | 26.58 ms | 4.1 MB | 538 B | 106.92 µs cold, 334 ns warm; 2.62 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.71 ms | 301.06 µs | 24.17 ms | 24.54 ms | 4.0 MB | 538 B | 104.33 µs cold, 250 ns warm; 2.63 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.22 ms | 219.02 µs | 23.87 ms | 24.44 ms | 4.1 MB | 538 B | 114.21 µs cold, 250 ns warm; 2.62 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.71 ms | 175.30 µs | 29.30 ms | 30.00 ms | 6.0 MB | 478 B | 5.06 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.07 ms | 382.51 µs | 30.50 ms | 29.56 ms | 3.1 MB | 478 B | 2.35 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.45 ms | 451.13 µs | 11.67 ms | 13.00 ms | 3.4 MB | 478 B | 2.36 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.04 ms | 273.81 µs | 50.64 ms | 52.26 ms | 56.6 MB | 477 B | 18.26 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.87 ms | 151.17 µs | 34.64 ms | 35.39 ms | 62.6 MB | 477 B | 18.08 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.19 ms | 182.47 µs | 35.00 ms | 35.53 ms | 88.2 MB | 477 B | 18.40 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 306.91 µs | 2.28 µs | 304.00 µs | 333.62 µs | 3.2 MB | 215 B | 72.58 µs cold, 375 ns warm; 2.46 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 141.97 ms | 395.75 µs | 141.28 ms | 143.62 ms | 10.5 MB | 195 B | 4.82 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 31.99 ms | 74.85 µs | 31.87 ms | 31.02 ms | 3.6 MB | 195 B | 2.41 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.40 ms | 143.68 µs | 20.25 ms | 20.67 ms | 3.8 MB | 195 B | 2.33 ms as a whole process |  |
| Node `--jitless` | Purpose | 362.17 µs | 144.33 µs | 239.04 µs | 979.12 µs | 52.7 MB | 223 B | 18.69 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 215.79 µs | 148.46 µs | 90.08 µs | 864.62 µs | 57.7 MB | 223 B | 18.14 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 137.36 µs | 116.34 µs | 62.83 µs | 884.04 µs | 57.9 MB | 223 B | 18.20 ms as a whole process |  |
| Seal | Normal | 336.42 µs | 1.19 µs | 335.00 µs | 353.33 µs | 3.4 MB | 182 B | 79.79 µs cold, 375 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 158.92 ms | 231.14 µs | 158.53 ms | 161.12 ms | 11.8 MB | 138 B | 4.83 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 40.94 ms | 169.28 µs | 40.78 ms | 40.90 ms | 5.0 MB | 138 B | 2.28 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.37 ms | 221.73 µs | 18.91 ms | 19.86 ms | 5.3 MB | 138 B | 2.29 ms as a whole process |  |
| Node `--jitless` | Normal | 493.65 µs | 329.97 µs | 267.88 µs | 1.09 ms | 61.4 MB | 155 B | 17.77 ms as a whole process |  |
| Node (JIT), from cold | Normal | 484.22 µs | 453.68 µs | 127.21 µs | 1.03 ms | 65.4 MB | 155 B | 18.14 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 271.08 µs | 313.78 µs | 123.58 µs | 1.03 ms | 69.5 MB | 155 B | 18.07 ms as a whole process |  |
| Seal | Ceil.source | 340.48 µs | 2.04 µs | 337.62 µs | 363.71 µs | 3.4 MB | 182 B | 75.75 µs cold, 375 ns warm; 2.60 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 340.73 µs | 1.93 µs | 338.12 µs | 356.33 µs | 3.3 MB | 182 B | 77.33 µs cold, 292 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 284.41 µs | 1.29 µs | 282.58 µs | 307.00 µs | 3.2 MB | 182 B | 60.46 µs cold, 166 ns warm; 2.43 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 282.01 µs | 1.20 µs | 279.96 µs | 309.38 µs | 3.2 MB | 182 B | 62.13 µs cold, 292 ns warm; 2.42 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 159.17 ms | 258.85 µs | 158.89 ms | 161.46 ms | 11.8 MB | 138 B | 4.96 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.01 ms | 230.46 µs | 40.73 ms | 41.23 ms | 4.9 MB | 138 B | 2.40 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.05 ms | 114.67 µs | 18.91 ms | 19.84 ms | 5.3 MB | 138 B | 2.32 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 491.65 µs | 334.33 µs | 262.67 µs | 1.08 ms | 61.5 MB | 155 B | 18.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 491.42 µs | 476.94 µs | 129.67 µs | 1.06 ms | 65.4 MB | 155 B | 18.13 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 277.26 µs | 328.04 µs | 122.38 µs | 1.05 ms | 69.5 MB | 155 B | 18.35 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 74.06 ms | 476.67 µs | 73.46 ms | 75.40 ms | 3.1 MB | 519 B | 97.38 µs cold, 250 ns warm; 2.68 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 93.83 ms | 232.27 µs | 93.30 ms | 93.51 ms | 4.8 MB | 411 B | 5.02 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 116.95 ms | 488.93 µs | 115.89 ms | 116.10 ms | 2.6 MB | 411 B | 2.32 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 93.01 ms | 905.39 µs | 91.82 ms | 92.66 ms | 2.9 MB | 411 B | 2.27 ms as a whole process |  |
| Node `--jitless` | Purpose | 74.66 ms | 194.19 µs | 74.42 ms | 78.00 ms | 59.7 MB | 444 B | 18.17 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.34 ms | 302.69 µs | 10.05 ms | 12.81 ms | 66.0 MB | 444 B | 18.35 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.27 ms | 579.98 µs | 9.90 ms | 12.91 ms | 115.0 MB | 444 B | 18.06 ms as a whole process |  |
| Seal | Normal | 101.45 ms | 2.25 ms | 98.42 ms | 102.92 ms | 8.0 MB | 956 B | 124.42 µs cold, 208 ns warm; 2.50 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 128.60 ms | 1.70 ms | 126.60 ms | 129.35 ms | 42.6 MB | 799 B | 4.83 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 136.11 ms | 8.46 ms | 126.62 ms | 125.19 ms | 41.6 MB | 799 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 80.61 ms | 822.29 µs | 79.16 ms | 79.08 ms | 39.3 MB | 799 B | 2.38 ms as a whole process |  |
| Node `--jitless` | Normal | 111.67 ms | 1.80 ms | 110.36 ms | 114.03 ms | 213.4 MB | 878 B | 18.27 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.01 ms | 893.46 µs | 13.15 ms | 16.96 ms | 129.8 MB | 878 B | 19.01 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.25 ms | 66.47 µs | 13.16 ms | 16.71 ms | 224.3 MB | 878 B | 18.61 ms as a whole process |  |
| Seal | Ceil.source | 99.05 ms | 1.07 ms | 97.83 ms | 99.97 ms | 8.0 MB | 956 B | 125.67 µs cold, 250 ns warm; 2.75 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 98.34 ms | 613.79 µs | 97.49 ms | 99.63 ms | 8.0 MB | 956 B | 138.58 µs cold, 208 ns warm; 2.54 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 96.92 ms | 725.35 µs | 95.84 ms | 98.50 ms | 7.8 MB | 956 B | 102.42 µs cold, 166 ns warm; 2.59 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 96.76 ms | 509.11 µs | 96.03 ms | 98.35 ms | 7.8 MB | 956 B | 113.75 µs cold, 292 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 128.99 ms | 1.14 ms | 127.12 ms | 129.89 ms | 38.1 MB | 799 B | 4.97 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 127.85 ms | 1.30 ms | 126.28 ms | 127.71 ms | 42.7 MB | 799 B | 2.39 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 81.30 ms | 837.85 µs | 80.02 ms | 82.67 ms | 41.9 MB | 799 B | 2.35 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.08 ms | 1.90 ms | 113.68 ms | 115.61 ms | 213.3 MB | 878 B | 18.29 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 13.96 ms | 818.65 µs | 13.19 ms | 16.63 ms | 129.6 MB | 878 B | 18.74 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.14 ms | 76.88 µs | 13.02 ms | 16.64 ms | 224.3 MB | 878 B | 18.41 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed three ways: every script from a text of its own, so that nothing compiled is reused; every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that; and every script from a compiled file of its own, a `.sealc` for Seal and `string.dump`'s bytecode, as `luajit -b` writes it, for LuaJIT (D171). Seal frees each script loaded from a text or a file of its own inside its timing, before loading the next.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.42 µs (±8.77 µs, least 42.42 µs) | 13.76 µs (±830 ns, least 12.79 µs) | 9.55 µs (±9.07 µs, least 6.00 µs) | 18 ns (±0 ns, least 18 ns) | 39 ns (±0 ns, least 39 ns) | 512 B | 4.2 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.33 µs (±909 ns, least 8.58 µs) | 16.28 µs (±846 ns, least 15.29 µs) | ? | 40 ns (±1 ns, least 39 ns) | 21 ns (±2 ns, least 18 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.90 µs (±568 ns, least 3.00 µs) | 5.00 µs (±471 ns, least 4.00 µs) | 1.10 µs (±568 ns, least 0 ns) | 21 ns (±1 ns, least 19 ns) | 16 ns (±2 ns, least 14 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.80 µs (±789 ns, least 3.00 µs) | 5.00 µs (±471 ns, least 4.00 µs) | 1.10 µs (±316 ns, least 1.00 µs) | 13 ns (±0 ns, least 12 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 36.05 µs (±815 ns, least 35.17 µs) | 57.40 µs (±66.24 µs, least 35.46 µs) | ? | 56 ns (±1 ns, least 55 ns) | 29 ns (±0 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 32.01 µs (±6.78 µs, least 19.00 µs) | 35.61 µs (±725 ns, least 34.62 µs) | ? | 8 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.68 µs (±22 ns, least 4.65 µs) | 146 ns (±9 ns, least 133 ns) | 662 ns (±10 ns, least 648 ns) | 18 ns (±1 ns, least 18 ns) | 39 ns (±0 ns, least 39 ns) | 512 B | 10.2 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.72 µs (±100 ns, least 4.62 µs) | 176 ns (±7 ns, least 168 ns) | ? | 34 ns (±1 ns, least 33 ns) | 17 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.77 µs (±13 ns, least 1.75 µs) | 174 ns (±5 ns, least 167 ns) | 274 ns (±5 ns, least 268 ns) | 27 ns (±0 ns, least 27 ns) | 15 ns (±1 ns, least 14 ns) | 1.0 KB | 5.2 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.79 µs (±7 ns, least 1.78 µs) | 349 ns (±35 ns, least 330 ns) | 480 ns (±83 ns, least 245 ns) | 103 ns (±2 ns, least 101 ns) | 109 ns (±13 ns, least 103 ns) | 1.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.10 µs (±12 ns, least 1.07 µs) | 643 ns (±17 ns, least 619 ns) | ? | 50 ns (±2 ns, least 48 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.05 µs (±13 ns, least 1.03 µs) | 637 ns (±15 ns, least 610 ns) | ? | 21 ns (±0 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 56.0 MB | the engine's own heap |
| Seal | Ceil.source | 4.68 µs (±42 ns, least 4.62 µs) | 146 ns (±7 ns, least 133 ns) | 666 ns (±13 ns, least 653 ns) | 19 ns (±1 ns, least 18 ns) | 39 ns (±0 ns, least 39 ns) | 512 B | 10.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.65 µs (±59 ns, least 4.57 µs) | 152 ns (±9 ns, least 136 ns) | 644 ns (±10 ns, least 628 ns) | 19 ns (±0 ns, least 18 ns) | 39 ns (±1 ns, least 39 ns) | 512 B | 10.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.07 µs (±61 ns, least 4.00 µs) | 142 ns (±5 ns, least 132 ns) | 597 ns (±17 ns, least 577 ns) | 17 ns (±1 ns, least 16 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 10.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.11 µs (±44 ns, least 4.06 µs) | 141 ns (±6 ns, least 133 ns) | 599 ns (±16 ns, least 575 ns) | 17 ns (±1 ns, least 16 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 10.1 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.78 µs (±164 ns, least 4.64 µs) | 183 ns (±6 ns, least 174 ns) | ? | 33 ns (±1 ns, least 32 ns) | 17 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.77 µs (±11 ns, least 1.75 µs) | 175 ns (±5 ns, least 167 ns) | 280 ns (±7 ns, least 272 ns) | 27 ns (±0 ns, least 27 ns) | 15 ns (±1 ns, least 14 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.79 µs (±17 ns, least 1.77 µs) | 352 ns (±8 ns, least 336 ns) | 504 ns (±10 ns, least 492 ns) | 91 ns (±5 ns, least 88 ns) | 99 ns (±2 ns, least 97 ns) | 1.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.12 µs (±21 ns, least 1.09 µs) | 656 ns (±23 ns, least 632 ns) | ? | 50 ns (±1 ns, least 48 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.09 µs (±22 ns, least 1.04 µs) | 663 ns (±23 ns, least 626 ns) | ? | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 34.58 ms | 34.69 ms (+0.3%) |
| n-body, 100,000 steps | 66.60 ms | 60.67 ms (-8.9%) |
| spectral-norm, size 200 | 18.79 ms | 17.84 ms (-5.1%) |
| fannkuch-redux, 9 | 102.32 ms | 102.90 ms (+0.6%) |
| word count, 500,000 words | 25.91 ms | 26.26 ms (+1.4%) |
| text building, 20,000 joins | 341.00 µs | 346.89 µs (+1.7%) |
| binary-trees, depth 14 | 101.17 ms | 100.65 ms (-0.5%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Seal mean | Seal deviation | Seal least | Seal peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 11.27 µs | 1.32 ms | 3.8 MB | 1.57 ms | 1.53 ms | 300.20 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.34 ms | 2.46 µs | 1.34 ms | 3.5 MB | 1.51 ms | 1.50 ms | 1.10 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 111.26 ms | 218.66 µs | 110.88 ms | 3.0 MB | 238.55 ms | 183.82 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 67.23 ms | 126.14 µs | 67.10 ms | 3.0 MB | 83.80 ms | 83.39 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 412.35 µs | 2.09 µs | 407.79 µs | 3.5 MB | 888.95 µs | 846.96 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 93.33 ms | 2.16 ms | 89.20 ms | 3.0 MB | 160.51 ms | 160.32 ms | – |
| the money sums in `Float`s, as a yardstick | 10.50 ms | 135.13 µs | 10.20 ms | 2.9 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Seal's door is written in C, through Seal's C API (`crates/seal-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Seal's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Seal | C | 14 ns | |
| Seal | Rust | 8 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
