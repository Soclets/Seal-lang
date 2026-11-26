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
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 4.36 4.46 3.79 }` before, `{ 4.94 4.45 3.89 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 34.10 ms | 71.90 ms | **met**: Seal takes 47% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.50 ms | 68.69 ms | **met**: Seal takes 97% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.82 ms | 27.15 ms | **met**: Seal takes 69% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 103.44 ms | 208.28 ms | **met**: Seal takes 50% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 27.06 ms | 29.62 ms | **met**: Seal takes 91% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 346.74 µs | 161.30 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 102.14 ms | 129.85 ms | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.03 ms | 3.58 ms | **met**: Seal takes 57% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 19 ns | 34 ns | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 158 ns | 179 ns | **met**: Seal takes 88% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.73 µs | 4.76 µs | **met**: Seal takes 99% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 34.10 ms | 33.62 ms | **not met**: Seal takes 101% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 66.50 ms | 71.14 ms | **met**: Seal takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.82 ms | 46.39 ms | **met**: Seal takes 41% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 103.44 ms | 146.27 ms | **met**: Seal takes 71% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 27.06 ms | 31.89 ms | **met**: Seal takes 85% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 346.74 µs | 41.15 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 102.14 ms | 127.84 ms | **met**: Seal takes 80% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.03 ms | 2.91 ms | **met**: Seal takes 70% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 19 ns | 27 ns | **met**: Seal takes 68% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 158 ns | 180 ns | **met**: Seal takes 88% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.73 µs | 1.77 µs | for reference, no target (D150) |
| D171, D176. Within twice LuaJIT loading its own bytecode | patrolling scripts, loading each from a compiled file of its own | 644 ns | 276 ns | **not met**: Seal takes 233% of its time |
| D176. Loading the same compiled file again | patrolling scripts, each loaded from one `.sealc` file, reusing what was verified | 153 ns | | for reference, no target |
| D171. Checking a `.sealc` file's signature, once per file | patrolling scripts, loading each signed file of its own, signature checked | 39.74 µs | | for reference, no target |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 14 ns | a Rust door: 8 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.8× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 1.58 ms | **met**: Seal takes 86% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.41 ms | 1.51 ms | **met**: Seal takes 93% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 112.54 ms | 162.35 ms | **met**: Seal takes 69% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 68.86 ms | 81.91 ms | **met**: Seal takes 84% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 437.50 µs | 908.21 µs | **met**: Seal takes 48% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 92.11 ms | 160.04 ms | **met**: Seal takes 58% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 34.10 ms | 94.65 ms | **met**: Seal takes 36% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.50 ms | 174.79 ms | **met**: Seal takes 38% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 27.06 ms | 50.71 ms | **met**: Seal takes 53% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 346.74 µs | 500.38 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 19 ns | host call 39 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 86.43 ms | 1.37 ms | 84.77 ms | 108.01 ms | 3.0 MB | 377 B | 773.92 µs cold, 541 ns warm; 2.72 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 194.48 ms | 190.16 µs | 194.12 ms | 193.84 ms | 4.8 MB | 339 B | 5.24 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 88.82 ms | 789.08 µs | 87.63 ms | 89.95 ms | 1.9 MB | 339 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 43.60 ms | 182.14 µs | 43.24 ms | 45.55 ms | 2.3 MB | 339 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Purpose | 317.92 ms | 208.23 µs | 317.60 ms | 318.26 ms | 43.4 MB | 362 B | 18.58 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 126.75 ms | 1.75 ms | 125.16 ms | 125.64 ms | 49.2 MB | 362 B | 19.36 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 125.30 ms | 64.84 µs | 125.22 ms | 125.64 ms | 49.2 MB | 362 B | 18.49 ms as a whole process |  |
| Seal | Normal | 34.10 ms | 941.01 µs | 32.93 ms | 33.55 ms | 2.8 MB | 169 B | 83.25 µs cold, 292 ns warm; 2.63 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled |
| Luau (interpreter) | Normal | 71.90 ms | 305.95 µs | 71.29 ms | 72.12 ms | 4.8 MB | 143 B | 5.23 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.62 ms | 69.75 µs | 33.49 ms | 33.37 ms | 1.9 MB | 143 B | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 6.98 ms | 8.12 µs | 6.96 ms | 7.30 ms | 2.2 MB | 143 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Normal | 94.65 ms | 474.46 µs | 93.69 ms | 95.08 ms | 43.4 MB | 137 B | 18.89 ms as a whole process |  |
| Node (JIT), from cold | Normal | 10.10 ms | 768.70 µs | 9.29 ms | 11.14 ms | 48.7 MB | 137 B | 18.83 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.62 ms | 602.73 µs | 9.30 ms | 11.27 ms | 48.8 MB | 137 B | 18.79 ms as a whole process |  |
| Seal | Ceil.source | 33.92 ms | 759.84 µs | 32.97 ms | 36.61 ms | 2.9 MB | 169 B | 84.12 µs cold, 375 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 29.25 ms | 78.71 µs | 29.14 ms | 54.50 ms | 2.9 MB | 169 B | 828.71 µs cold, 458 ns warm; 2.67 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.program | 27.83 ms | 653.34 µs | 26.67 ms | 54.78 ms | 2.7 MB | 169 B | 617.71 µs cold, 209 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 28.46 ms | 181.79 µs | 28.11 ms | 53.64 ms | 2.7 MB | 169 B | 657.25 µs cold, 292 ns warm; 2.63 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 72.31 ms | 108.23 µs | 72.04 ms | 72.47 ms | 4.8 MB | 143 B | 5.19 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.41 ms | 117.30 µs | 33.19 ms | 33.46 ms | 1.9 MB | 143 B | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.12 ms | 20.30 µs | 7.09 ms | 7.39 ms | 2.2 MB | 143 B | 2.33 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 94.23 ms | 506.82 µs | 93.18 ms | 93.80 ms | 43.3 MB | 137 B | 18.80 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.70 ms | 711.46 µs | 9.29 ms | 10.89 ms | 48.7 MB | 137 B | 19.26 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.87 ms | 616.00 µs | 9.31 ms | 11.08 ms | 48.7 MB | 137 B | 19.18 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 47.92 ms | 863.67 µs | 46.50 ms | 52.56 ms | 2.9 MB | 532 B | 120.71 µs cold, 334 ns warm; 2.68 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.17 ms | 472.10 µs | 90.78 ms | 90.97 ms | 4.8 MB | 433 B | 5.23 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.13 ms | 607.67 µs | 75.12 ms | 75.96 ms | 1.9 MB | 433 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.70 ms | 12.65 µs | 6.68 ms | 6.71 ms | 2.1 MB | 433 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Purpose | 241.34 ms | 436.31 µs | 240.97 ms | 242.11 ms | 46.4 MB | 460 B | 19.30 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.07 ms | 30.94 µs | 19.02 ms | 19.00 ms | 50.3 MB | 460 B | 18.70 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.01 ms | 15.05 µs | 18.99 ms | 19.02 ms | 50.4 MB | 460 B | 19.10 ms as a whole process |  |
| Seal | Normal | 66.50 ms | 646.39 µs | 65.61 ms | 67.62 ms | 3.4 MB | 3.4 KB | 299.29 µs cold, 333 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 68.69 ms | 131.91 µs | 68.49 ms | 69.20 ms | 4.8 MB | 3.0 KB | 5.31 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.14 ms | 432.11 µs | 70.49 ms | 70.85 ms | 1.9 MB | 3.0 KB | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.16 ms | 31.05 µs | 5.13 ms | 5.33 ms | 2.2 MB | 3.0 KB | 2.47 ms as a whole process |  |
| Node `--jitless` | Normal | 174.79 ms | 3.16 ms | 171.09 ms | 180.20 ms | 48.2 MB | 3.1 KB | 19.19 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.84 ms | 17.41 µs | 3.82 ms | 5.64 ms | 52.2 MB | 3.1 KB | 19.06 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.83 ms | 12.37 µs | 3.82 ms | 5.64 ms | 52.1 MB | 3.1 KB | 19.56 ms as a whole process |  |
| Seal | Ceil.source | 56.67 ms | 576.77 µs | 55.96 ms | 56.39 ms | 3.3 MB | 3.4 KB | 263.42 µs cold, 292 ns warm; 2.83 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.37 ms | 420.23 µs | 54.71 ms | 58.91 ms | 3.4 MB | 3.4 KB | 264.62 µs cold, 333 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.33 ms | 583.58 µs | 48.55 ms | 49.53 ms | 3.2 MB | 3.4 KB | 260.21 µs cold, 250 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 48.94 ms | 460.94 µs | 48.31 ms | 48.03 ms | 3.2 MB | 3.4 KB | 269.00 µs cold, 250 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 68.96 ms | 215.73 µs | 68.66 ms | 69.26 ms | 4.8 MB | 3.0 KB | 5.20 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 70.94 ms | 383.62 µs | 70.41 ms | 70.71 ms | 1.9 MB | 3.0 KB | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.16 ms | 21.21 µs | 5.13 ms | 5.36 ms | 2.2 MB | 3.0 KB | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 172.82 ms | 3.47 ms | 168.94 ms | 177.25 ms | 48.2 MB | 3.1 KB | 18.81 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.84 ms | 12.24 µs | 3.83 ms | 5.60 ms | 52.1 MB | 3.1 KB | 19.24 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.83 ms | 11.14 µs | 3.82 ms | 5.63 ms | 52.2 MB | 3.1 KB | 18.87 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 50.27 ms | 913.28 µs | 48.83 ms | 49.35 ms | 3.0 MB | 393 B | 93.00 µs cold, 417 ns warm; 2.56 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.13 ms | 283.34 µs | 44.75 ms | 44.86 ms | 4.8 MB | 366 B | 5.10 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.46 ms | 82.80 µs | 76.36 ms | 76.36 ms | 2.0 MB | 366 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 23.14 ms | 26.86 µs | 23.09 ms | 23.25 ms | 2.2 MB | 366 B | 2.27 ms as a whole process |  |
| Node `--jitless` | Purpose | 370.47 ms | 4.64 ms | 366.43 ms | 378.07 ms | 48.7 MB | 398 B | 18.59 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 33.65 ms | 36.46 ms | 15.40 ms | 19.14 ms | 52.8 MB | 398 B | 18.50 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.19 ms | 20.69 µs | 14.17 ms | 21.14 ms | 55.4 MB | 398 B | 19.19 ms as a whole process |  |
| Seal | Normal | 18.82 ms | 400.43 µs | 18.08 ms | 17.91 ms | 3.2 MB | 1.2 KB | 151.46 µs cold, 250 ns warm; 2.75 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.15 ms | 670.54 µs | 26.17 ms | 26.78 ms | 4.9 MB | 981 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 46.39 ms | 353.08 µs | 45.92 ms | 45.91 ms | 2.0 MB | 981 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.52 ms | 13.83 µs | 1.51 ms | 1.72 ms | 2.3 MB | 981 B | 2.55 ms as a whole process |  |
| Node `--jitless` | Normal | 62.96 ms | 224.16 µs | 62.64 ms | 62.78 ms | 48.4 MB | 1.0 KB | 19.43 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.59 ms | 212.57 µs | 1.44 ms | 3.65 ms | 52.9 MB | 1.0 KB | 18.69 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.63 ms | 17.64 µs | 1.61 ms | 3.13 ms | 52.9 MB | 1.0 KB | 18.82 ms as a whole process |  |
| Seal | Ceil.source | 18.30 ms | 425.05 µs | 17.40 ms | 18.82 ms | 3.2 MB | 1.2 KB | 151.96 µs cold, 250 ns warm; 2.80 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 17.92 ms | 224.27 µs | 17.49 ms | 18.40 ms | 3.1 MB | 1.2 KB | 144.79 µs cold, 250 ns warm; 2.66 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.86 ms | 111.36 µs | 18.74 ms | 19.19 ms | 3.0 MB | 1.2 KB | 136.88 µs cold, 250 ns warm; 2.66 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.76 ms | 239.90 µs | 18.41 ms | 19.30 ms | 3.0 MB | 1.2 KB | 134.33 µs cold, 250 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 26.73 ms | 429.73 µs | 25.86 ms | 26.64 ms | 4.8 MB | 981 B | 5.24 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 44.57 ms | 687.90 µs | 43.69 ms | 44.29 ms | 2.0 MB | 981 B | 2.33 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.51 ms | 14.20 µs | 1.50 ms | 1.74 ms | 2.3 MB | 981 B | 2.29 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.40 ms | 268.59 µs | 63.90 ms | 63.78 ms | 48.4 MB | 1.0 KB | 18.64 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.57 ms | 148.39 µs | 1.43 ms | 3.17 ms | 52.8 MB | 1.0 KB | 18.55 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.62 ms | 39.94 µs | 1.57 ms | 3.60 ms | 52.9 MB | 1.0 KB | 18.69 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.72 ms | 442.08 µs | 84.06 ms | 85.84 ms | 2.9 MB | 596 B | 103.67 µs cold, 292 ns warm; 2.55 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 86.61 ms | 870.68 µs | 85.31 ms | 89.02 ms | 4.8 MB | 523 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.93 ms | 736.78 µs | 51.76 ms | 52.02 ms | 1.9 MB | 523 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.14 ms | 63.43 µs | 6.06 ms | 6.27 ms | 2.2 MB | 523 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Purpose | 145.88 ms | 1.02 ms | 144.27 ms | 147.11 ms | 43.3 MB | 568 B | 18.71 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 112.24 µs | 3.60 ms | 4.40 ms | 49.4 MB | 568 B | 18.97 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.65 ms | 111.63 µs | 3.60 ms | 4.41 ms | 49.5 MB | 568 B | 19.14 ms as a whole process |  |
| Seal | Normal | 103.44 ms | 309.83 µs | 103.01 ms | 102.21 ms | 3.1 MB | 1.8 KB | 152.38 µs cold, 250 ns warm; 2.71 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 208.28 ms | 1.95 ms | 202.83 ms | 210.71 ms | 4.8 MB | 1.7 KB | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 146.27 ms | 457.26 µs | 145.49 ms | 143.70 ms | 2.0 MB | 1.7 KB | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.23 ms | 381.38 µs | 47.65 ms | 48.57 ms | 2.3 MB | 1.7 KB | 2.59 ms as a whole process |  |
| Node `--jitless` | Normal | 155.81 ms | 319.41 µs | 155.35 ms | 156.44 ms | 45.4 MB | 1.7 KB | 19.16 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.73 ms | 94.95 µs | 14.60 ms | 16.08 ms | 52.1 MB | 1.7 KB | 19.07 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.97 ms | 193.17 µs | 13.80 ms | 15.49 ms | 52.3 MB | 1.7 KB | 18.82 ms as a whole process |  |
| Seal | Ceil.source | 104.31 ms | 198.77 µs | 103.93 ms | 104.21 ms | 3.2 MB | 1.8 KB | 155.21 µs cold, 291 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 100.60 ms | 207.96 µs | 100.37 ms | 101.60 ms | 3.1 MB | 1.8 KB | 150.50 µs cold, 250 ns warm; 2.60 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 109.20 ms | 158.48 µs | 109.01 ms | 109.88 ms | 3.0 MB | 1.8 KB | 138.08 µs cold, 167 ns warm; 2.62 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 108.38 ms | 266.46 µs | 108.05 ms | 108.73 ms | 2.9 MB | 1.8 KB | 132.08 µs cold, 209 ns warm; 2.67 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 214.13 ms | 2.42 ms | 207.30 ms | 216.02 ms | 4.8 MB | 1.7 KB | 5.24 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 146.08 ms | 482.16 µs | 145.63 ms | 144.13 ms | 2.0 MB | 1.7 KB | 2.39 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.86 ms | 389.74 µs | 47.97 ms | 48.32 ms | 2.3 MB | 1.7 KB | 2.47 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 152.36 ms | 396.74 µs | 151.77 ms | 155.23 ms | 45.4 MB | 1.7 KB | 18.68 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 13.89 ms | 64.10 µs | 13.77 ms | 15.45 ms | 52.1 MB | 1.7 KB | 19.01 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.43 ms | 34.15 µs | 14.35 ms | 15.99 ms | 52.3 MB | 1.7 KB | 19.25 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 125.75 ms | 519.12 µs | 125.06 ms | 124.56 ms | 4.2 MB | 533 B | 107.00 µs cold, 375 ns warm; 2.65 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.67 ms | 907.48 µs | 61.81 ms | 62.25 ms | 4.8 MB | 540 B | 5.35 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 35.71 ms | 577.50 µs | 34.83 ms | 34.58 ms | 2.2 MB | 540 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.83 ms | 54.67 µs | 4.77 ms | 4.90 ms | 2.4 MB | 540 B | 2.34 ms as a whole process |  |
| Node `--jitless` | Purpose | 240.37 ms | 952.86 µs | 239.18 ms | 240.93 ms | 48.7 MB | 514 B | 19.00 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.78 ms | 23.64 ms | 65.38 ms | 90.94 ms | 52.8 MB | 514 B | 18.97 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 63.34 ms | 609.23 µs | 62.35 ms | 88.22 ms | 52.7 MB | 514 B | 18.34 ms as a whole process |  |
| Seal | Normal | 27.06 ms | 470.95 µs | 26.50 ms | 26.66 ms | 4.1 MB | 538 B | 114.17 µs cold, 375 ns warm; 2.70 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.62 ms | 226.54 µs | 29.21 ms | 29.59 ms | 6.0 MB | 478 B | 5.06 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.89 ms | 787.06 µs | 30.64 ms | 32.11 ms | 3.1 MB | 478 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.28 ms | 530.95 µs | 11.45 ms | 11.66 ms | 3.4 MB | 478 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Normal | 50.71 ms | 268.50 µs | 50.32 ms | 51.91 ms | 56.6 MB | 477 B | 19.03 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.05 ms | 263.10 µs | 34.66 ms | 35.78 ms | 62.6 MB | 477 B | 19.25 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.09 ms | 336.58 µs | 34.81 ms | 35.35 ms | 88.6 MB | 477 B | 18.93 ms as a whole process |  |
| Seal | Ceil.source | 26.51 ms | 461.46 µs | 26.09 ms | 26.24 ms | 4.1 MB | 538 B | 115.54 µs cold, 417 ns warm; 2.72 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 26.84 ms | 171.48 µs | 26.62 ms | 26.78 ms | 4.1 MB | 538 B | 109.21 µs cold, 375 ns warm; 2.54 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.61 ms | 231.82 µs | 24.16 ms | 25.09 ms | 4.1 MB | 538 B | 114.00 µs cold, 250 ns warm; 2.65 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.11 ms | 204.89 µs | 23.86 ms | 23.91 ms | 4.2 MB | 538 B | 109.25 µs cold, 250 ns warm; 2.45 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.68 ms | 96.76 µs | 29.53 ms | 29.68 ms | 6.0 MB | 478 B | 5.31 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 30.98 ms | 597.46 µs | 30.17 ms | 32.24 ms | 3.1 MB | 478 B | 2.38 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.26 ms | 611.97 µs | 11.26 ms | 12.64 ms | 3.4 MB | 478 B | 2.52 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.35 ms | 312.02 µs | 51.01 ms | 52.01 ms | 56.6 MB | 477 B | 18.76 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.43 ms | 270.83 µs | 34.02 ms | 35.24 ms | 62.7 MB | 477 B | 18.59 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 34.72 ms | 182.82 µs | 34.46 ms | 35.39 ms | 88.2 MB | 477 B | 19.35 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 297.38 µs | 7.18 µs | 292.00 µs | 330.96 µs | 3.2 MB | 215 B | 76.08 µs cold, 417 ns warm; 2.73 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.00 ms | 394.82 µs | 143.42 ms | 145.20 ms | 10.4 MB | 195 B | 5.20 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.20 ms | 190.04 µs | 32.01 ms | 31.20 ms | 3.6 MB | 195 B | 2.41 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.68 ms | 173.71 µs | 20.38 ms | 20.65 ms | 3.9 MB | 195 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 371.24 µs | 143.85 µs | 246.92 µs | 995.83 µs | 52.4 MB | 223 B | 18.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 207.77 µs | 146.19 µs | 84.25 µs | 884.12 µs | 57.9 MB | 223 B | 19.64 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 173.51 µs | 165.78 µs | 62.92 µs | 902.21 µs | 58.7 MB | 223 B | 18.58 ms as a whole process |  |
| Seal | Normal | 346.74 µs | 2.30 µs | 343.79 µs | 366.17 µs | 3.4 MB | 182 B | 80.71 µs cold, 333 ns warm; 2.61 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 161.30 ms | 229.05 µs | 160.92 ms | 162.89 ms | 11.8 MB | 138 B | 5.36 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.15 ms | 152.27 µs | 40.86 ms | 41.41 ms | 4.8 MB | 138 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.06 ms | 114.30 µs | 18.94 ms | 20.10 ms | 5.2 MB | 138 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Normal | 500.38 µs | 342.40 µs | 262.00 µs | 1.12 ms | 61.4 MB | 155 B | 18.41 ms as a whole process |  |
| Node (JIT), from cold | Normal | 482.53 µs | 467.93 µs | 126.17 µs | 1.03 ms | 65.2 MB | 155 B | 19.36 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 274.59 µs | 327.52 µs | 122.67 µs | 1.04 ms | 69.5 MB | 155 B | 19.08 ms as a whole process |  |
| Seal | Ceil.source | 347.36 µs | 2.56 µs | 344.38 µs | 375.17 µs | 3.4 MB | 182 B | 82.42 µs cold, 375 ns warm; 2.57 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 340.20 µs | 1.48 µs | 337.38 µs | 364.42 µs | 3.4 MB | 182 B | 78.17 µs cold, 334 ns warm; 2.49 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 279.33 µs | 1.70 µs | 276.75 µs | 309.58 µs | 3.2 MB | 182 B | 64.54 µs cold, 208 ns warm; 2.50 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 278.76 µs | 2.51 µs | 275.12 µs | 308.62 µs | 3.2 MB | 182 B | 62.08 µs cold, 250 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.33 ms | 277.34 µs | 160.87 ms | 163.01 ms | 11.8 MB | 138 B | 5.18 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.21 ms | 388.16 µs | 40.74 ms | 41.46 ms | 4.9 MB | 138 B | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.06 ms | 126.46 µs | 18.92 ms | 19.84 ms | 5.2 MB | 138 B | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 507.26 µs | 346.52 µs | 265.04 µs | 1.10 ms | 61.5 MB | 155 B | 18.93 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 501.40 µs | 474.66 µs | 133.92 µs | 1.06 ms | 65.4 MB | 155 B | 19.02 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 276.91 µs | 332.84 µs | 123.54 µs | 1.04 ms | 69.5 MB | 155 B | 18.66 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.76 ms | 2.13 ms | 73.54 ms | 80.84 ms | 3.1 MB | 519 B | 102.50 µs cold, 417 ns warm; 2.60 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 93.93 ms | 279.43 µs | 93.33 ms | 94.41 ms | 4.8 MB | 411 B | 5.02 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 121.26 ms | 2.08 ms | 117.30 ms | 116.52 ms | 2.6 MB | 411 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 95.72 ms | 1.28 ms | 93.27 ms | 95.08 ms | 2.9 MB | 411 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Purpose | 74.57 ms | 587.85 µs | 74.14 ms | 78.66 ms | 59.8 MB | 444 B | 18.63 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.30 ms | 269.79 µs | 10.05 ms | 12.81 ms | 66.0 MB | 444 B | 18.65 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.38 ms | 616.32 µs | 9.92 ms | 12.77 ms | 114.8 MB | 444 B | 18.80 ms as a whole process |  |
| Seal | Normal | 102.14 ms | 1.11 ms | 100.76 ms | 103.54 ms | 8.0 MB | 956 B | 131.29 µs cold, 209 ns warm; 2.87 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 129.85 ms | 2.80 ms | 125.96 ms | 130.99 ms | 37.8 MB | 799 B | 5.32 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 127.84 ms | 1.11 ms | 126.27 ms | 126.57 ms | 42.5 MB | 799 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 80.82 ms | 1.30 ms | 79.07 ms | 81.80 ms | 40.8 MB | 799 B | 2.30 ms as a whole process |  |
| Node `--jitless` | Normal | 114.13 ms | 1.73 ms | 112.34 ms | 116.33 ms | 213.3 MB | 878 B | 19.10 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.03 ms | 916.84 µs | 13.08 ms | 16.86 ms | 129.6 MB | 878 B | 19.62 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.15 ms | 76.42 µs | 12.99 ms | 16.72 ms | 221.6 MB | 878 B | 18.92 ms as a whole process |  |
| Seal | Ceil.source | 102.12 ms | 435.80 µs | 101.58 ms | 103.25 ms | 8.0 MB | 956 B | 138.83 µs cold, 250 ns warm; 2.60 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 97.07 ms | 501.00 µs | 96.39 ms | 99.23 ms | 8.0 MB | 956 B | 127.67 µs cold, 250 ns warm; 2.61 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 98.22 ms | 760.33 µs | 96.95 ms | 99.21 ms | 7.8 MB | 956 B | 118.75 µs cold, 208 ns warm; 2.70 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 97.12 ms | 756.48 µs | 96.12 ms | 100.89 ms | 7.8 MB | 956 B | 110.58 µs cold, 208 ns warm; 2.59 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 130.73 ms | 1.54 ms | 128.85 ms | 128.93 ms | 42.9 MB | 799 B | 5.19 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.34 ms | 1.01 ms | 127.11 ms | 127.56 ms | 39.5 MB | 799 B | 2.43 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 82.23 ms | 1.54 ms | 80.02 ms | 80.70 ms | 41.1 MB | 799 B | 2.57 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.22 ms | 2.43 ms | 113.12 ms | 117.58 ms | 213.2 MB | 878 B | 19.45 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.77 ms | 933.25 µs | 13.75 ms | 16.76 ms | 129.5 MB | 878 B | 19.64 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.14 ms | 52.26 µs | 13.07 ms | 16.78 ms | 224.1 MB | 878 B | 19.18 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed three ways: every script from a text of its own, so that nothing compiled is reused; every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that; and every script from a compiled file of its own, a `.sealc` for Seal and `string.dump`'s bytecode, as `luajit -b` writes it, for LuaJIT (D171). Seal frees each script loaded from a text or a file of its own inside its timing, before loading the next.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.93 µs (±9.25 µs, least 47.46 µs) | 16.86 µs (±3.95 µs, least 12.96 µs) | 8.07 µs (±4.51 µs, least 5.54 µs) | 18 ns (±0 ns, least 17 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 4.2 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.44 µs (±925 ns, least 9.21 µs) | 16.80 µs (±764 ns, least 15.75 µs) | ? | 41 ns (±1 ns, least 39 ns) | 20 ns (±1 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.50 µs (±527 ns, least 3.00 µs) | 5.50 µs (±850 ns, least 5.00 µs) | 1.20 µs (±632 ns, least 0 ns) | 21 ns (±1 ns, least 20 ns) | 16 ns (±2 ns, least 12 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.90 µs (±568 ns, least 3.00 µs) | 5.80 µs (±1.03 µs, least 4.00 µs) | 1.30 µs (±675 ns, least 0 ns) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 33.20 µs (±6.87 µs, least 20.12 µs) | 37.72 µs (±928 ns, least 36.17 µs) | ? | 57 ns (±1 ns, least 56 ns) | 29 ns (±1 ns, least 28 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 33.67 µs (±5.02 µs, least 19.67 µs) | 36.95 µs (±999 ns, least 35.58 µs) | ? | 8 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.73 µs (±43 ns, least 4.67 µs) | 158 ns (±7 ns, least 151 ns) | 644 ns (±8 ns, least 631 ns) | 19 ns (±0 ns, least 18 ns) | 39 ns (±0 ns, least 39 ns) | 512 B | 10.2 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.76 µs (±96 ns, least 4.68 µs) | 179 ns (±5 ns, least 170 ns) | ? | 34 ns (±1 ns, least 33 ns) | 17 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.77 µs (±14 ns, least 1.75 µs) | 180 ns (±4 ns, least 174 ns) | 276 ns (±5 ns, least 266 ns) | 27 ns (±1 ns, least 26 ns) | 15 ns (±1 ns, least 14 ns) | 1.0 KB | 5.2 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.80 µs (±14 ns, least 1.78 µs) | 362 ns (±35 ns, least 333 ns) | 459 ns (±107 ns, least 252 ns) | 112 ns (±7 ns, least 103 ns) | 117 ns (±16 ns, least 107 ns) | 1.0 KB | 6.0 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.11 µs (±20 ns, least 1.07 µs) | 651 ns (±12 ns, least 634 ns) | ? | 49 ns (±1 ns, least 48 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.06 µs (±17 ns, least 1.02 µs) | 650 ns (±17 ns, least 624 ns) | ? | 21 ns (±0 ns, least 21 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.72 µs (±44 ns, least 4.67 µs) | 152 ns (±8 ns, least 143 ns) | 642 ns (±8 ns, least 630 ns) | 19 ns (±0 ns, least 18 ns) | 39 ns (±0 ns, least 39 ns) | 512 B | 10.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.73 µs (±42 ns, least 4.66 µs) | 152 ns (±5 ns, least 144 ns) | 633 ns (±9 ns, least 620 ns) | 19 ns (±0 ns, least 19 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 10.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.06 µs (±11 ns, least 4.04 µs) | 141 ns (±4 ns, least 133 ns) | 577 ns (±15 ns, least 556 ns) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 10.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.12 µs (±43 ns, least 4.07 µs) | 139 ns (±6 ns, least 131 ns) | 586 ns (±15 ns, least 568 ns) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 10.0 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.72 µs (±44 ns, least 4.66 µs) | 180 ns (±4 ns, least 171 ns) | ? | 33 ns (±1 ns, least 32 ns) | 17 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.77 µs (±15 ns, least 1.75 µs) | 179 ns (±3 ns, least 176 ns) | 276 ns (±6 ns, least 270 ns) | 27 ns (±0 ns, least 26 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.79 µs (±14 ns, least 1.77 µs) | 405 ns (±58 ns, least 335 ns) | 484 ns (±80 ns, least 257 ns) | 124 ns (±4 ns, least 120 ns) | 116 ns (±3 ns, least 113 ns) | 1.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.11 µs (±16 ns, least 1.09 µs) | 644 ns (±12 ns, least 624 ns) | ? | 49 ns (±0 ns, least 48 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.06 µs (±9 ns, least 1.04 µs) | 648 ns (±15 ns, least 629 ns) | ? | 21 ns (±0 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 34.17 ms | 34.49 ms (+0.9%) |
| n-body, 100,000 steps | 64.59 ms | 61.91 ms (-4.1%) |
| spectral-norm, size 200 | 18.71 ms | 18.67 ms (-0.2%) |
| fannkuch-redux, 9 | 102.17 ms | 101.91 ms (-0.3%) |
| word count, 500,000 words | 26.12 ms | 26.10 ms (-0.1%) |
| text building, 20,000 joins | 344.32 µs | 336.89 µs (-2.2%) |
| binary-trees, depth 14 | 107.60 ms | 99.47 ms (-7.6%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Seal mean | Seal deviation | Seal least | Seal peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 7.89 µs | 1.34 ms | 3.8 MB | 1.58 ms | 1.56 ms | 315.26 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.41 ms | 18.54 µs | 1.39 ms | 3.5 MB | 1.51 ms | 1.50 ms | 1.08 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 112.54 ms | 221.08 µs | 112.11 ms | 3.1 MB | 162.35 ms | 162.04 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 68.86 ms | 290.82 µs | 68.32 ms | 3.0 MB | 81.91 ms | 81.15 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 437.50 µs | 2.23 µs | 432.21 µs | 3.6 MB | 908.21 µs | 900.88 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 92.11 ms | 2.19 ms | 87.85 ms | 3.1 MB | 160.04 ms | 159.68 ms | – |
| the money sums in `Float`s, as a yardstick | 10.27 ms | 263.35 µs | 9.69 ms | 2.9 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Seal's door is written in C, through Seal's C API (`crates/seal-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Seal's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Seal | C | 14 ns | |
| Seal | Rust | 8 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
