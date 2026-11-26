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
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.06 1.98 2.27 }` before, `{ 1.85 1.97 2.21 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 31.96 ms | 70.12 ms | **met**: Seal takes 46% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.58 ms | 69.44 ms | **met**: Seal takes 96% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 17.97 ms | 27.24 ms | **met**: Seal takes 66% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 102.70 ms | 209.02 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.62 ms | 29.94 ms | **met**: Seal takes 89% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 363.95 µs | 162.22 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 101.45 ms | 136.53 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.16 ms | 3.64 ms | **met**: Seal takes 59% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 35 ns | **met**: Seal takes 58% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 163 ns | 187 ns | **met**: Seal takes 87% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.78 µs | 4.91 µs | **met**: Seal takes 98% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 31.96 ms | 33.63 ms | **met**: Seal takes 95% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 66.58 ms | 71.24 ms | **met**: Seal takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 17.97 ms | 45.60 ms | **met**: Seal takes 39% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 102.70 ms | 145.75 ms | **met**: Seal takes 70% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.62 ms | 31.34 ms | **met**: Seal takes 85% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 363.95 µs | 42.46 ms | **met**: Seal takes 0.9% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 101.45 ms | 127.37 ms | **met**: Seal takes 80% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.16 ms | 2.92 ms | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 27 ns | **met**: Seal takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 163 ns | 184 ns | **met**: Seal takes 89% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.78 µs | 1.79 µs | for reference, no target (D150) |
| D171, D176. Within twice LuaJIT loading its own bytecode | patrolling scripts, loading each from a compiled file of its own | 662 ns | 294 ns | **not met**: Seal takes 225% of its time |
| D176. Loading the same compiled file again | patrolling scripts, each loaded from one `.sealc` file, reusing what was verified | 156 ns | | for reference, no target |
| D171. Checking a `.sealc` file's signature, once per file | patrolling scripts, loading each signed file of its own, signature checked | 40.13 µs | | for reference, no target |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 11 ns | a Rust door: 7 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.5× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.37 ms | 1.58 ms | **met**: Seal takes 86% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.38 ms | 1.52 ms | **met**: Seal takes 91% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 112.40 ms | 161.53 ms | **met**: Seal takes 70% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 67.06 ms | 80.37 ms | **met**: Seal takes 83% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 432.54 µs | 897.41 µs | **met**: Seal takes 48% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 92.14 ms | 160.65 ms | **met**: Seal takes 57% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 31.96 ms | 94.21 ms | **met**: Seal takes 34% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.58 ms | 176.34 ms | **met**: Seal takes 38% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.62 ms | 51.01 ms | **met**: Seal takes 52% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 363.95 µs | 528.65 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 41 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.01 ms | 1.93 ms | 94.32 ms | 104.10 ms | 3.0 MB | 377 B | 575.79 µs cold, 541 ns warm; 2.76 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 195.20 ms | 319.48 µs | 194.71 ms | 196.00 ms | 4.8 MB | 339 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.50 ms | 745.76 µs | 88.80 ms | 88.14 ms | 1.9 MB | 339 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 43.99 ms | 231.75 µs | 43.48 ms | 45.95 ms | 2.3 MB | 339 B | 2.62 ms as a whole process |  |
| Node `--jitless` | Purpose | 317.86 ms | 1.22 ms | 316.38 ms | 319.88 ms | 43.3 MB | 362 B | 18.71 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.95 ms | 436.60 µs | 122.44 ms | 127.35 ms | 49.5 MB | 362 B | 18.62 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 123.25 ms | 378.95 µs | 122.70 ms | 123.37 ms | 49.3 MB | 362 B | 19.48 ms as a whole process |  |
| Seal | Normal | 31.96 ms | 721.60 µs | 30.86 ms | 31.57 ms | 2.9 MB | 169 B | 82.33 µs cold, 209 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled |
| Luau (interpreter) | Normal | 70.12 ms | 225.87 µs | 69.88 ms | 72.31 ms | 4.8 MB | 143 B | 5.33 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.63 ms | 141.37 µs | 33.46 ms | 33.61 ms | 1.9 MB | 143 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.29 ms | 68.23 µs | 7.22 ms | 7.46 ms | 2.2 MB | 143 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Normal | 94.21 ms | 852.97 µs | 93.13 ms | 93.36 ms | 43.3 MB | 137 B | 18.57 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.64 ms | 526.25 µs | 9.37 ms | 10.47 ms | 48.8 MB | 137 B | 19.43 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.44 ms | 125.94 µs | 9.34 ms | 11.87 ms | 48.7 MB | 137 B | 19.20 ms as a whole process |  |
| Seal | Ceil.source | 31.47 ms | 963.73 µs | 29.99 ms | 31.18 ms | 2.9 MB | 169 B | 82.00 µs cold, 166 ns warm; 2.61 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 32.11 ms | 375.55 µs | 31.27 ms | 58.20 ms | 2.8 MB | 169 B | 978.92 µs cold, 500 ns warm; 2.59 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.program | 33.42 ms | 830.05 µs | 32.34 ms | 58.46 ms | 2.7 MB | 169 B | 831.00 µs cold, 375 ns warm; 2.71 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 30.35 ms | 722.59 µs | 29.03 ms | 54.98 ms | 2.7 MB | 169 B | 848.88 µs cold, 375 ns warm; 2.52 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.69 ms | 528.64 µs | 71.13 ms | 73.04 ms | 4.8 MB | 143 B | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.62 ms | 127.61 µs | 33.45 ms | 33.71 ms | 1.9 MB | 143 B | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.05 ms | 16.08 µs | 7.02 ms | 7.35 ms | 2.2 MB | 143 B | 2.46 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 94.69 ms | 798.67 µs | 93.13 ms | 93.57 ms | 43.4 MB | 137 B | 19.16 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 10.03 ms | 884.17 µs | 9.37 ms | 10.18 ms | 48.7 MB | 137 B | 18.87 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.46 ms | 124.89 µs | 9.35 ms | 10.96 ms | 48.8 MB | 137 B | 18.16 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 47.76 ms | 470.52 µs | 46.99 ms | 50.62 ms | 2.9 MB | 532 B | 117.17 µs cold, 250 ns warm; 2.63 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.58 ms | 472.10 µs | 90.86 ms | 91.65 ms | 4.8 MB | 433 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.17 ms | 364.65 µs | 75.54 ms | 75.85 ms | 1.9 MB | 433 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.65 ms | 98.33 µs | 6.48 ms | 6.79 ms | 2.1 MB | 433 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.96 ms | 4.32 ms | 241.88 ms | 244.23 ms | 46.5 MB | 460 B | 19.68 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.88 ms | 138.02 µs | 18.60 ms | 19.61 ms | 50.7 MB | 460 B | 20.05 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.61 ms | 168.52 µs | 18.36 ms | 19.23 ms | 50.5 MB | 460 B | 19.89 ms as a whole process |  |
| Seal | Normal | 66.58 ms | 2.48 ms | 62.82 ms | 66.53 ms | 3.4 MB | 3.4 KB | 295.00 µs cold, 292 ns warm; 2.92 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.44 ms | 433.93 µs | 68.79 ms | 69.54 ms | 4.8 MB | 3.0 KB | 5.41 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.24 ms | 400.47 µs | 70.71 ms | 71.08 ms | 1.9 MB | 3.0 KB | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.18 ms | 11.18 µs | 5.16 ms | 5.40 ms | 2.2 MB | 3.0 KB | 2.41 ms as a whole process |  |
| Node `--jitless` | Normal | 176.34 ms | 2.74 ms | 172.89 ms | 178.89 ms | 48.2 MB | 3.1 KB | 18.34 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 28.14 µs | 3.83 ms | 5.75 ms | 52.2 MB | 3.1 KB | 18.67 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 15.11 µs | 3.84 ms | 5.64 ms | 52.0 MB | 3.1 KB | 18.46 ms as a whole process |  |
| Seal | Ceil.source | 56.06 ms | 813.42 µs | 55.00 ms | 55.42 ms | 3.3 MB | 3.4 KB | 259.33 µs cold, 292 ns warm; 2.97 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 58.14 ms | 514.24 µs | 57.51 ms | 58.98 ms | 3.3 MB | 3.4 KB | 314.38 µs cold, 292 ns warm; 2.96 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.08 ms | 552.81 µs | 48.13 ms | 49.93 ms | 3.2 MB | 3.4 KB | 276.21 µs cold, 250 ns warm; 2.82 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 46.82 ms | 289.55 µs | 46.27 ms | 46.70 ms | 3.2 MB | 3.4 KB | 276.29 µs cold, 250 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.83 ms | 306.09 µs | 69.32 ms | 70.58 ms | 4.8 MB | 3.0 KB | 5.54 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 76.22 ms | 11.61 ms | 71.82 ms | 71.55 ms | 1.9 MB | 3.0 KB | 2.66 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.18 ms | 17.68 µs | 5.15 ms | 5.38 ms | 2.2 MB | 3.0 KB | 2.50 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 177.84 ms | 3.91 ms | 172.78 ms | 180.68 ms | 48.3 MB | 3.1 KB | 20.02 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.87 ms | 32.06 µs | 3.84 ms | 5.69 ms | 52.2 MB | 3.1 KB | 19.61 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 23.67 µs | 3.83 ms | 5.65 ms | 52.0 MB | 3.1 KB | 18.78 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.53 ms | 2.10 ms | 54.18 ms | 58.26 ms | 3.0 MB | 393 B | 92.04 µs cold, 166 ns warm; 2.54 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.35 ms | 357.73 µs | 44.76 ms | 44.67 ms | 4.8 MB | 366 B | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.79 ms | 197.98 µs | 77.57 ms | 77.68 ms | 2.0 MB | 366 B | 2.40 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.61 ms | 70.58 µs | 22.47 ms | 22.83 ms | 2.2 MB | 366 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Purpose | 369.43 ms | 4.43 ms | 364.60 ms | 382.93 ms | 48.7 MB | 398 B | 19.09 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.18 ms | 36.29 ms | 15.49 ms | 21.47 ms | 53.0 MB | 398 B | 19.16 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.55 ms | 459.38 µs | 14.19 ms | 20.80 ms | 55.6 MB | 398 B | 18.57 ms as a whole process |  |
| Seal | Normal | 17.97 ms | 383.83 µs | 17.20 ms | 17.45 ms | 3.2 MB | 1.2 KB | 162.67 µs cold, 250 ns warm; 2.62 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.24 ms | 489.94 µs | 26.71 ms | 27.80 ms | 4.8 MB | 981 B | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.60 ms | 1.12 ms | 44.22 ms | 44.85 ms | 2.0 MB | 981 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.53 ms | 29.47 µs | 1.51 ms | 1.68 ms | 2.3 MB | 981 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 64.33 ms | 588.18 µs | 63.60 ms | 64.74 ms | 48.4 MB | 1.0 KB | 18.83 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.57 ms | 179.34 µs | 1.44 ms | 3.56 ms | 52.9 MB | 1.0 KB | 18.20 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.63 ms | 19.51 µs | 1.59 ms | 3.29 ms | 52.9 MB | 1.0 KB | 18.41 ms as a whole process |  |
| Seal | Ceil.source | 18.66 ms | 317.74 µs | 18.32 ms | 18.68 ms | 3.1 MB | 1.2 KB | 161.50 µs cold, 250 ns warm; 2.60 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.97 ms | 184.67 µs | 18.77 ms | 19.00 ms | 3.2 MB | 1.2 KB | 148.92 µs cold, 208 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.08 ms | 255.75 µs | 18.77 ms | 19.41 ms | 3.0 MB | 1.2 KB | 132.62 µs cold, 208 ns warm; 2.72 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 17.67 ms | 449.05 µs | 17.27 ms | 18.59 ms | 3.0 MB | 1.2 KB | 131.62 µs cold, 208 ns warm; 2.45 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.01 ms | 319.46 µs | 26.63 ms | 26.75 ms | 4.8 MB | 981 B | 5.04 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 44.80 ms | 528.33 µs | 43.96 ms | 45.36 ms | 2.0 MB | 981 B | 2.33 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.54 ms | 27.47 µs | 1.50 ms | 1.69 ms | 2.3 MB | 981 B | 2.36 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.75 ms | 394.85 µs | 63.04 ms | 63.60 ms | 48.4 MB | 1.0 KB | 19.02 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.67 ms | 229.02 µs | 1.53 ms | 3.21 ms | 53.1 MB | 1.0 KB | 18.56 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.62 ms | 14.46 µs | 1.60 ms | 3.26 ms | 52.8 MB | 1.0 KB | 19.39 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 85.81 ms | 394.79 µs | 84.96 ms | 85.49 ms | 2.9 MB | 596 B | 110.79 µs cold, 167 ns warm; 2.74 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 89.82 ms | 2.06 ms | 86.77 ms | 91.48 ms | 4.8 MB | 523 B | 5.62 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.57 ms | 951.09 µs | 51.83 ms | 51.81 ms | 1.9 MB | 523 B | 2.36 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.24 ms | 59.81 µs | 6.18 ms | 6.39 ms | 2.2 MB | 523 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.57 ms | 1.12 ms | 145.32 ms | 147.52 ms | 43.4 MB | 568 B | 19.57 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.74 ms | 97.53 µs | 3.62 ms | 5.05 ms | 49.3 MB | 568 B | 19.97 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.67 ms | 32.43 µs | 3.63 ms | 4.49 ms | 49.5 MB | 568 B | 18.82 ms as a whole process |  |
| Seal | Normal | 102.70 ms | 351.29 µs | 102.21 ms | 102.29 ms | 3.2 MB | 1.8 KB | 150.04 µs cold, 208 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 209.02 ms | 2.15 ms | 203.05 ms | 210.43 ms | 4.9 MB | 1.7 KB | 5.32 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 145.75 ms | 689.65 µs | 144.61 ms | 143.78 ms | 2.0 MB | 1.7 KB | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.34 ms | 545.01 µs | 47.34 ms | 48.10 ms | 2.3 MB | 1.7 KB | 2.54 ms as a whole process |  |
| Node `--jitless` | Normal | 159.67 ms | 3.09 ms | 156.09 ms | 157.82 ms | 45.3 MB | 1.7 KB | 19.59 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.59 ms | 389.21 µs | 14.24 ms | 16.31 ms | 52.3 MB | 1.7 KB | 19.60 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.45 ms | 72.33 µs | 14.31 ms | 16.91 ms | 52.3 MB | 1.7 KB | 20.24 ms as a whole process |  |
| Seal | Ceil.source | 103.60 ms | 461.64 µs | 103.07 ms | 104.74 ms | 3.2 MB | 1.8 KB | 158.33 µs cold, 208 ns warm; 2.83 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 106.80 ms | 2.88 ms | 105.21 ms | 106.17 ms | 3.2 MB | 1.8 KB | 181.75 µs cold, 250 ns warm; 2.85 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 103.48 ms | 326.17 µs | 102.94 ms | 104.75 ms | 3.0 MB | 1.8 KB | 141.96 µs cold, 209 ns warm; 2.74 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 100.07 ms | 492.20 µs | 99.45 ms | 100.61 ms | 3.0 MB | 1.8 KB | 135.71 µs cold, 208 ns warm; 2.62 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 211.47 ms | 1.95 ms | 207.52 ms | 211.48 ms | 4.8 MB | 1.7 KB | 5.37 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 147.05 ms | 786.28 µs | 145.97 ms | 146.15 ms | 2.0 MB | 1.7 KB | 2.55 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 47.80 ms | 271.27 µs | 47.34 ms | 47.66 ms | 2.3 MB | 1.7 KB | 2.66 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 155.24 ms | 822.81 µs | 153.82 ms | 155.95 ms | 45.5 MB | 1.7 KB | 18.45 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.67 ms | 86.06 µs | 14.57 ms | 16.14 ms | 52.1 MB | 1.7 KB | 18.99 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.79 ms | 115.92 µs | 14.64 ms | 16.40 ms | 52.3 MB | 1.7 KB | 18.60 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 131.20 ms | 2.70 ms | 127.26 ms | 128.33 ms | 4.2 MB | 533 B | 107.29 µs cold, 375 ns warm; 2.71 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.54 ms | 1.00 ms | 62.19 ms | 63.72 ms | 4.8 MB | 540 B | 5.25 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 28.95 ms | 271.68 µs | 28.55 ms | 28.92 ms | 2.2 MB | 540 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.68 ms | 31.95 µs | 4.62 ms | 4.77 ms | 2.4 MB | 540 B | 2.30 ms as a whole process |  |
| Node `--jitless` | Purpose | 239.60 ms | 806.70 µs | 238.75 ms | 241.29 ms | 48.7 MB | 514 B | 18.41 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.03 ms | 24.45 ms | 63.40 ms | 89.48 ms | 52.7 MB | 514 B | 19.12 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 70.68 ms | 1.47 ms | 68.19 ms | 94.49 ms | 52.8 MB | 514 B | 20.00 ms as a whole process |  |
| Seal | Normal | 26.62 ms | 172.35 µs | 26.43 ms | 27.01 ms | 4.2 MB | 538 B | 122.12 µs cold, 583 ns warm; 3.10 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.94 ms | 94.61 µs | 29.74 ms | 30.19 ms | 6.0 MB | 478 B | 5.19 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.34 ms | 512.43 µs | 30.52 ms | 30.34 ms | 3.1 MB | 478 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.18 ms | 280.33 µs | 11.68 ms | 12.63 ms | 3.4 MB | 478 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Normal | 51.01 ms | 246.49 µs | 50.69 ms | 52.56 ms | 56.5 MB | 477 B | 19.82 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.03 ms | 159.62 µs | 34.83 ms | 35.68 ms | 62.6 MB | 477 B | 19.19 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.66 ms | 260.40 µs | 35.25 ms | 35.61 ms | 88.3 MB | 477 B | 20.77 ms as a whole process |  |
| Seal | Ceil.source | 26.65 ms | 174.16 µs | 26.39 ms | 27.19 ms | 4.1 MB | 538 B | 111.33 µs cold, 291 ns warm; 2.84 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.70 ms | 151.86 µs | 27.41 ms | 27.83 ms | 4.1 MB | 538 B | 117.62 µs cold, 417 ns warm; 2.84 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.07 ms | 787.77 µs | 23.57 ms | 24.05 ms | 4.1 MB | 538 B | 106.96 µs cold, 334 ns warm; 2.93 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 28.63 ms | 8.18 ms | 23.79 ms | 24.52 ms | 4.3 MB | 538 B | 108.88 µs cold, 208 ns warm; 2.90 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.46 ms | 829.14 µs | 29.76 ms | 30.01 ms | 6.0 MB | 478 B | 5.45 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 30.98 ms | 606.05 µs | 29.98 ms | 31.82 ms | 3.1 MB | 478 B | 2.47 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.17 ms | 569.24 µs | 11.67 ms | 11.96 ms | 3.4 MB | 478 B | 2.59 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.74 ms | 340.88 µs | 51.24 ms | 52.96 ms | 56.5 MB | 477 B | 19.20 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.87 ms | 256.77 µs | 34.55 ms | 36.16 ms | 62.6 MB | 477 B | 19.11 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.22 ms | 228.24 µs | 34.89 ms | 36.02 ms | 88.0 MB | 477 B | 19.12 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 322.73 µs | 3.82 µs | 318.42 µs | 344.08 µs | 3.2 MB | 215 B | 75.79 µs cold, 333 ns warm; 2.57 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.65 ms | 907.63 µs | 143.52 ms | 146.46 ms | 11.0 MB | 195 B | 5.18 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 33.13 ms | 199.18 µs | 32.93 ms | 31.40 ms | 3.7 MB | 195 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 21.10 ms | 365.67 µs | 20.69 ms | 20.96 ms | 3.9 MB | 195 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Purpose | 384.41 µs | 151.82 µs | 246.29 µs | 1.05 ms | 52.5 MB | 223 B | 19.55 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 219.57 µs | 150.71 µs | 89.42 µs | 902.12 µs | 57.8 MB | 223 B | 18.77 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 138.77 µs | 124.34 µs | 56.12 µs | 887.88 µs | 58.0 MB | 223 B | 19.29 ms as a whole process |  |
| Seal | Normal | 363.95 µs | 8.55 µs | 354.54 µs | 382.17 µs | 3.4 MB | 182 B | 81.00 µs cold, 375 ns warm; 2.62 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 162.22 ms | 1.32 ms | 160.28 ms | 164.26 ms | 11.9 MB | 138 B | 5.41 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 42.46 ms | 411.81 µs | 41.96 ms | 42.04 ms | 4.9 MB | 138 B | 2.60 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 20.90 ms | 337.83 µs | 20.39 ms | 21.23 ms | 5.3 MB | 138 B | 2.65 ms as a whole process |  |
| Node `--jitless` | Normal | 528.65 µs | 337.45 µs | 273.88 µs | 1.16 ms | 61.3 MB | 155 B | 20.17 ms as a whole process |  |
| Node (JIT), from cold | Normal | 508.90 µs | 495.92 µs | 134.17 µs | 1.18 ms | 65.4 MB | 155 B | 19.28 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 286.74 µs | 333.11 µs | 129.00 µs | 1.10 ms | 69.4 MB | 155 B | 19.59 ms as a whole process |  |
| Seal | Ceil.source | 371.66 µs | 18.44 µs | 357.50 µs | 392.33 µs | 3.4 MB | 182 B | 81.25 µs cold, 292 ns warm; 2.65 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 363.19 µs | 6.83 µs | 356.67 µs | 399.33 µs | 3.4 MB | 182 B | 81.88 µs cold, 208 ns warm; 2.63 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 299.53 µs | 13.78 µs | 287.88 µs | 309.21 µs | 3.2 MB | 182 B | 62.04 µs cold, 292 ns warm; 2.45 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 292.03 µs | 9.31 µs | 285.83 µs | 325.92 µs | 3.2 MB | 182 B | 69.50 µs cold, 250 ns warm; 2.60 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.24 ms | 980.61 µs | 159.70 ms | 165.44 ms | 11.9 MB | 138 B | 5.27 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 43.32 ms | 368.04 µs | 42.68 ms | 41.39 ms | 5.1 MB | 138 B | 2.52 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.80 ms | 232.87 µs | 19.31 ms | 19.49 ms | 5.2 MB | 138 B | 2.52 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 522.79 µs | 334.63 µs | 264.46 µs | 1.09 ms | 61.6 MB | 155 B | 18.50 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 482.70 µs | 472.08 µs | 125.46 µs | 1.05 ms | 65.3 MB | 155 B | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 277.14 µs | 329.33 µs | 123.38 µs | 1.06 ms | 69.4 MB | 155 B | 19.45 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 72.08 ms | 811.39 µs | 71.14 ms | 71.56 ms | 3.1 MB | 519 B | 109.58 µs cold, 208 ns warm; 2.70 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 94.30 ms | 300.53 µs | 93.89 ms | 94.52 ms | 4.8 MB | 411 B | 5.07 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 121.01 ms | 1.80 ms | 117.95 ms | 118.06 ms | 2.6 MB | 411 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.42 ms | 786.72 µs | 93.24 ms | 95.45 ms | 2.9 MB | 411 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.28 ms | 603.14 µs | 75.61 ms | 80.04 ms | 59.7 MB | 444 B | 19.17 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.43 ms | 259.49 µs | 10.14 ms | 12.82 ms | 65.9 MB | 444 B | 18.27 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.39 ms | 674.59 µs | 9.95 ms | 12.92 ms | 114.8 MB | 444 B | 18.63 ms as a whole process |  |
| Seal | Normal | 101.45 ms | 770.13 µs | 100.42 ms | 103.10 ms | 8.0 MB | 956 B | 133.33 µs cold, 208 ns warm; 2.63 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 136.53 ms | 1.64 ms | 134.52 ms | 133.22 ms | 43.2 MB | 799 B | 5.24 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 127.37 ms | 1.23 ms | 125.16 ms | 127.09 ms | 42.0 MB | 799 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.53 ms | 1.13 ms | 79.67 ms | 82.00 ms | 41.6 MB | 799 B | 2.42 ms as a whole process |  |
| Node `--jitless` | Normal | 114.86 ms | 1.89 ms | 113.31 ms | 117.54 ms | 213.3 MB | 878 B | 18.58 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.20 ms | 988.11 µs | 13.29 ms | 17.08 ms | 129.6 MB | 878 B | 18.58 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.41 ms | 204.59 µs | 13.29 ms | 17.06 ms | 224.2 MB | 878 B | 19.39 ms as a whole process |  |
| Seal | Ceil.source | 103.49 ms | 668.43 µs | 102.05 ms | 104.19 ms | 8.0 MB | 956 B | 133.50 µs cold, 333 ns warm; 2.74 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 101.60 ms | 914.05 µs | 100.78 ms | 100.78 ms | 8.0 MB | 956 B | 127.25 µs cold, 250 ns warm; 2.62 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 97.21 ms | 742.82 µs | 96.28 ms | 98.55 ms | 7.8 MB | 956 B | 111.62 µs cold, 167 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 93.44 ms | 1.01 ms | 92.34 ms | 93.67 ms | 7.8 MB | 956 B | 111.83 µs cold, 208 ns warm; 2.74 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 136.66 ms | 4.82 ms | 131.15 ms | 134.43 ms | 43.3 MB | 799 B | 5.46 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 130.50 ms | 1.01 ms | 128.78 ms | 129.22 ms | 44.3 MB | 799 B | 2.48 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 81.74 ms | 1.02 ms | 80.06 ms | 80.36 ms | 39.9 MB | 799 B | 2.55 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.71 ms | 1.70 ms | 113.80 ms | 118.68 ms | 213.3 MB | 878 B | 18.76 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.21 ms | 901.24 µs | 13.23 ms | 17.05 ms | 129.6 MB | 878 B | 19.52 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.47 ms | 118.12 µs | 13.32 ms | 17.36 ms | 224.1 MB | 878 B | 19.06 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed three ways: every script from a text of its own, so that nothing compiled is reused; every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that; and every script from a compiled file of its own, a `.sealc` for Seal and `string.dump`'s bytecode, as `luajit -b` writes it, for LuaJIT (D171). Seal frees each script loaded from a text or a file of its own inside its timing, before loading the next.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 72.03 µs (±13.56 µs, least 58.67 µs) | 17.91 µs (±4.04 µs, least 14.08 µs) | 11.06 µs (±6.73 µs, least 7.33 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 4.2 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.15 µs (±939 ns, least 8.75 µs) | 17.18 µs (±740 ns, least 16.54 µs) | ? | 40 ns (±1 ns, least 39 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.80 µs (±789 ns, least 3.00 µs) | 7.10 µs (±1.52 µs, least 5.00 µs) | 1.20 µs (±422 ns, least 1.00 µs) | 22 ns (±2 ns, least 20 ns) | 14 ns (±2 ns, least 12 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.50 µs (±527 ns, least 3.00 µs) | 6.70 µs (±1.34 µs, least 5.00 µs) | 1.10 µs (±316 ns, least 1.00 µs) | 13 ns (±0 ns, least 12 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 31.52 µs (±7.79 µs, least 19.42 µs) | 62.15 µs (±72.46 µs, least 36.79 µs) | ? | 56 ns (±1 ns, least 54 ns) | 29 ns (±0 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 30.57 µs (±7.27 µs, least 19.83 µs) | 38.90 µs (±3.09 µs, least 35.38 µs) | ? | 9 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 4.78 µs (±46 ns, least 4.70 µs) | 163 ns (±9 ns, least 152 ns) | 662 ns (±6 ns, least 654 ns) | 20 ns (±0 ns, least 20 ns) | 41 ns (±1 ns, least 40 ns) | 512 B | 10.2 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.91 µs (±126 ns, least 4.73 µs) | 187 ns (±11 ns, least 169 ns) | ? | 35 ns (±2 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.79 µs (±22 ns, least 1.76 µs) | 184 ns (±8 ns, least 172 ns) | 294 ns (±16 ns, least 278 ns) | 27 ns (±1 ns, least 26 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.82 µs (±22 ns, least 1.79 µs) | 378 ns (±46 ns, least 345 ns) | 510 ns (±63 ns, least 345 ns) | 94 ns (±3 ns, least 91 ns) | 103 ns (±5 ns, least 99 ns) | 1.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±81 ns, least 1.11 µs) | 686 ns (±29 ns, least 655 ns) | ? | 52 ns (±3 ns, least 49 ns) | 24 ns (±2 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.19 µs (±164 ns, least 1.08 µs) | 738 ns (±112 ns, least 657 ns) | ? | 23 ns (±2 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap |
| Seal | Ceil.source | 4.91 µs (±74 ns, least 4.76 µs) | 170 ns (±13 ns, least 148 ns) | 703 ns (±61 ns, least 667 ns) | 20 ns (±0 ns, least 20 ns) | 41 ns (±1 ns, least 40 ns) | 512 B | 10.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 5.44 µs (±2.39 µs, least 4.63 µs) | 170 ns (±15 ns, least 148 ns) | 741 ns (±270 ns, least 630 ns) | 20 ns (±1 ns, least 20 ns) | 46 ns (±16 ns, least 40 ns) | 512 B | 10.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.03 µs (±44 ns, least 3.99 µs) | 141 ns (±8 ns, least 133 ns) | 605 ns (±16 ns, least 581 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 10.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.12 µs (±51 ns, least 4.03 µs) | 148 ns (±7 ns, least 138 ns) | 601 ns (±20 ns, least 575 ns) | 17 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 10.1 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.83 µs (±121 ns, least 4.70 µs) | 186 ns (±8 ns, least 175 ns) | ? | 35 ns (±1 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.81 µs (±33 ns, least 1.78 µs) | 186 ns (±12 ns, least 173 ns) | 293 ns (±8 ns, least 280 ns) | 27 ns (±1 ns, least 26 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.84 µs (±44 ns, least 1.79 µs) | 360 ns (±13 ns, least 340 ns) | 477 ns (±121 ns, least 245 ns) | 96 ns (±3 ns, least 91 ns) | 110 ns (±13 ns, least 100 ns) | 1.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±27 ns, least 1.12 µs) | 672 ns (±15 ns, least 650 ns) | ? | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±21 ns, least 1.08 µs) | 662 ns (±14 ns, least 638 ns) | ? | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 30.99 ms | 32.52 ms (+4.9%) |
| n-body, 100,000 steps | 63.83 ms | 58.87 ms (-7.8%) |
| spectral-norm, size 200 | 18.18 ms | 17.66 ms (-2.8%) |
| fannkuch-redux, 9 | 102.95 ms | 104.51 ms (+1.5%) |
| word count, 500,000 words | 26.93 ms | 26.55 ms (-1.4%) |
| text building, 20,000 joins | 365.37 µs | 352.42 µs (-3.5%) |
| binary-trees, depth 14 | 100.77 ms | 100.09 ms (-0.7%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Seal mean | Seal deviation | Seal least | Seal peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.37 ms | 14.23 µs | 1.34 ms | 3.8 MB | 1.58 ms | 1.55 ms | 302.07 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.38 ms | 20.26 µs | 1.36 ms | 3.5 MB | 1.52 ms | 1.50 ms | 1.11 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 112.40 ms | 1.71 ms | 110.76 ms | 3.0 MB | 161.53 ms | 161.08 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 67.06 ms | 78.90 µs | 66.98 ms | 3.0 MB | 80.37 ms | 79.95 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 432.54 µs | 7.59 µs | 414.00 µs | 3.5 MB | 897.41 µs | 862.17 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 92.14 ms | 1.75 ms | 88.66 ms | 3.0 MB | 160.65 ms | 160.37 ms | – |
| the money sums in `Float`s, as a yardstick | 10.08 ms | 524.09 µs | 9.55 ms | 2.9 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Seal's door is written in C, through Seal's C API (`crates/seal-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Seal's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Seal | C | 11 ns | |
| Seal | Rust | 7 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
