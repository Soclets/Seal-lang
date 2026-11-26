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
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.41 2.39 2.23 }` before, `{ 2.23 2.19 2.16 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 35.52 ms | 71.30 ms | **met**: Seal takes 50% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 64.06 ms | 69.22 ms | **met**: Seal takes 93% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.82 ms | 27.67 ms | **met**: Seal takes 68% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 104.15 ms | 221.10 ms | **met**: Seal takes 47% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.69 ms | 30.90 ms | **met**: Seal takes 86% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 360.35 µs | 160.46 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 99.87 ms | 136.92 ms | **met**: Seal takes 73% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.05 ms | 3.70 ms | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 19 ns | 35 ns | **met**: Seal takes 54% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 148 ns | 188 ns | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.83 µs | 4.90 µs | **met**: Seal takes 99% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 35.52 ms | 36.35 ms | **met**: Seal takes 98% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 64.06 ms | 71.23 ms | **met**: Seal takes 90% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.82 ms | 46.19 ms | **met**: Seal takes 41% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 104.15 ms | 149.66 ms | **met**: Seal takes 70% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.69 ms | 31.70 ms | **met**: Seal takes 84% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 360.35 µs | 41.79 ms | **met**: Seal takes 0.9% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 99.87 ms | 128.34 ms | **met**: Seal takes 78% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.05 ms | 3.03 ms | **met**: Seal takes 68% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 19 ns | 28 ns | **met**: Seal takes 67% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 148 ns | 188 ns | **met**: Seal takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.83 µs | 1.84 µs | for reference, no target (D150) |
| D171. Faster than LuaJIT loading its own bytecode | patrolling scripts, loading each from a compiled file of its own | 702 ns | 295 ns | **not met**: Seal takes 238% of its time |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 11 ns | a Rust door: 7 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.5× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 1.60 ms | **met**: Seal takes 85% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.38 ms | 1.51 ms | **met**: Seal takes 92% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 112.79 ms | 162.06 ms | **met**: Seal takes 70% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 66.99 ms | 80.38 ms | **met**: Seal takes 83% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 416.06 µs | 903.47 µs | **met**: Seal takes 46% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 92.39 ms | 160.02 ms | **met**: Seal takes 58% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 35.52 ms | 94.26 ms | **met**: Seal takes 38% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 64.06 ms | 185.54 ms | **met**: Seal takes 35% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.69 ms | 51.18 ms | **met**: Seal takes 52% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 360.35 µs | 503.24 µs | **met**: Seal takes 72% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 19 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 86.05 ms | 640.42 µs | 85.02 ms | 104.75 ms | 2.9 MB | 377 B | 976.88 µs cold, 667 ns warm; 2.81 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 197.64 ms | 892.95 µs | 196.85 ms | 197.96 ms | 4.8 MB | 339 B | 5.67 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.12 ms | 734.32 µs | 88.34 ms | 90.49 ms | 1.9 MB | 339 B | 2.39 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.14 ms | 394.74 µs | 43.22 ms | 46.76 ms | 2.3 MB | 339 B | 2.55 ms as a whole process |  |
| Node `--jitless` | Purpose | 325.51 ms | 4.16 ms | 320.07 ms | 321.49 ms | 43.4 MB | 362 B | 19.67 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.56 ms | 985.14 µs | 123.55 ms | 129.89 ms | 49.5 MB | 362 B | 18.61 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 122.71 ms | 251.85 µs | 122.20 ms | 124.26 ms | 49.3 MB | 362 B | 18.89 ms as a whole process |  |
| Seal | Normal | 35.52 ms | 1.06 ms | 34.23 ms | 38.18 ms | 2.8 MB | 169 B | 82.96 µs cold, 291 ns warm; 2.71 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled |
| Luau (interpreter) | Normal | 71.30 ms | 458.52 µs | 70.63 ms | 71.82 ms | 4.8 MB | 143 B | 5.05 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 36.35 ms | 3.56 ms | 33.48 ms | 33.45 ms | 1.9 MB | 143 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.10 ms | 23.39 µs | 7.06 ms | 7.27 ms | 2.2 MB | 143 B | 2.42 ms as a whole process |  |
| Node `--jitless` | Normal | 94.26 ms | 514.65 µs | 93.47 ms | 93.93 ms | 43.4 MB | 137 B | 18.42 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.88 ms | 550.72 µs | 9.37 ms | 10.31 ms | 48.8 MB | 137 B | 18.28 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.46 ms | 145.42 µs | 9.33 ms | 11.82 ms | 48.7 MB | 137 B | 18.92 ms as a whole process |  |
| Seal | Ceil.source | 34.43 ms | 804.98 µs | 33.36 ms | 37.95 ms | 2.8 MB | 169 B | 79.83 µs cold, 292 ns warm; 2.49 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 29.31 ms | 380.65 µs | 28.80 ms | 53.18 ms | 2.8 MB | 169 B | 957.58 µs cold, 625 ns warm; 2.61 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.program | 27.37 ms | 317.10 µs | 26.87 ms | 51.88 ms | 2.7 MB | 169 B | 827.92 µs cold, 417 ns warm; 2.61 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 28.12 ms | 710.98 µs | 27.13 ms | 52.47 ms | 2.6 MB | 169 B | 845.83 µs cold, 375 ns warm; 2.58 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 70.50 ms | 137.41 µs | 70.19 ms | 71.99 ms | 4.9 MB | 143 B | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.51 ms | 50.32 µs | 33.41 ms | 33.43 ms | 1.9 MB | 143 B | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.01 ms | 15.29 µs | 6.98 ms | 7.45 ms | 2.2 MB | 143 B | 2.30 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 94.12 ms | 114.75 µs | 93.95 ms | 94.57 ms | 43.5 MB | 137 B | 18.37 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.74 ms | 364.07 µs | 9.33 ms | 10.40 ms | 48.7 MB | 137 B | 19.47 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.42 ms | 66.26 µs | 9.33 ms | 11.11 ms | 48.8 MB | 137 B | 18.63 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 51.52 ms | 1.28 ms | 49.93 ms | 52.52 ms | 2.8 MB | 532 B | 115.12 µs cold, 209 ns warm; 2.66 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.24 ms | 465.09 µs | 90.67 ms | 91.24 ms | 4.9 MB | 433 B | 5.05 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.31 ms | 450.53 µs | 75.45 ms | 75.92 ms | 1.9 MB | 433 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.65 ms | 28.44 µs | 6.61 ms | 6.71 ms | 2.1 MB | 433 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Purpose | 239.96 ms | 514.87 µs | 239.40 ms | 241.60 ms | 46.5 MB | 460 B | 18.42 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.58 ms | 160.82 µs | 18.35 ms | 18.95 ms | 50.6 MB | 460 B | 18.05 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.40 ms | 81.32 µs | 18.24 ms | 18.70 ms | 50.4 MB | 460 B | 18.49 ms as a whole process |  |
| Seal | Normal | 64.06 ms | 817.99 µs | 62.93 ms | 73.06 ms | 3.4 MB | 3.4 KB | 297.00 µs cold, 292 ns warm; 2.92 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.22 ms | 312.46 µs | 68.80 ms | 69.90 ms | 4.9 MB | 3.0 KB | 5.28 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.23 ms | 467.72 µs | 70.50 ms | 70.49 ms | 1.9 MB | 3.0 KB | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.18 ms | 16.30 µs | 5.16 ms | 5.39 ms | 2.2 MB | 3.0 KB | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 185.54 ms | 14.66 ms | 177.22 ms | 179.35 ms | 48.3 MB | 3.1 KB | 19.34 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.94 ms | 36.46 µs | 3.88 ms | 6.13 ms | 52.0 MB | 3.1 KB | 20.30 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.93 ms | 50.65 µs | 3.83 ms | 5.90 ms | 52.2 MB | 3.1 KB | 20.45 ms as a whole process |  |
| Seal | Ceil.source | 54.86 ms | 580.88 µs | 53.65 ms | 58.27 ms | 3.3 MB | 3.4 KB | 284.12 µs cold, 250 ns warm; 2.96 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 53.21 ms | 777.21 µs | 51.97 ms | 52.71 ms | 3.3 MB | 3.4 KB | 270.92 µs cold, 375 ns warm; 2.90 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.25 ms | 439.27 µs | 48.55 ms | 48.79 ms | 3.2 MB | 3.4 KB | 289.71 µs cold, 250 ns warm; 2.69 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.23 ms | 222.58 µs | 48.91 ms | 49.54 ms | 3.1 MB | 3.4 KB | 270.33 µs cold, 250 ns warm; 2.80 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 68.94 ms | 224.55 µs | 68.63 ms | 69.34 ms | 4.8 MB | 3.0 KB | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.06 ms | 359.53 µs | 70.72 ms | 70.85 ms | 1.9 MB | 3.0 KB | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.19 ms | 18.40 µs | 5.15 ms | 5.37 ms | 2.2 MB | 3.0 KB | 2.42 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.12 ms | 3.03 ms | 174.55 ms | 179.60 ms | 48.1 MB | 3.1 KB | 20.66 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.93 ms | 42.36 µs | 3.87 ms | 5.83 ms | 52.2 MB | 3.1 KB | 20.67 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.92 ms | 33.74 µs | 3.88 ms | 5.93 ms | 52.2 MB | 3.1 KB | 21.12 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.20 ms | 924.39 µs | 47.45 ms | 48.30 ms | 3.0 MB | 393 B | 91.58 µs cold, 250 ns warm; 2.72 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.73 ms | 452.89 µs | 45.12 ms | 45.51 ms | 4.9 MB | 366 B | 5.29 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.62 ms | 297.87 µs | 77.24 ms | 77.47 ms | 2.0 MB | 366 B | 2.71 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.72 ms | 164.25 µs | 22.45 ms | 22.99 ms | 2.2 MB | 366 B | 2.37 ms as a whole process |  |
| Node `--jitless` | Purpose | 375.80 ms | 6.27 ms | 368.89 ms | 385.89 ms | 48.7 MB | 398 B | 18.88 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.57 ms | 37.51 ms | 15.71 ms | 21.94 ms | 52.8 MB | 398 B | 19.35 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.69 ms | 437.90 µs | 14.37 ms | 21.69 ms | 55.5 MB | 398 B | 19.38 ms as a whole process |  |
| Seal | Normal | 18.82 ms | 358.38 µs | 18.32 ms | 18.93 ms | 3.1 MB | 1.2 KB | 148.38 µs cold, 292 ns warm; 2.75 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.67 ms | 772.38 µs | 26.72 ms | 27.14 ms | 4.9 MB | 981 B | 5.37 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 46.19 ms | 824.05 µs | 44.73 ms | 44.18 ms | 2.0 MB | 981 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.55 ms | 46.11 µs | 1.51 ms | 1.71 ms | 2.3 MB | 981 B | 2.41 ms as a whole process |  |
| Node `--jitless` | Normal | 64.57 ms | 521.18 µs | 63.76 ms | 64.55 ms | 48.5 MB | 1.0 KB | 18.79 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.62 ms | 225.32 µs | 1.46 ms | 3.52 ms | 53.2 MB | 1.0 KB | 19.67 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 37.20 µs | 1.60 ms | 3.35 ms | 53.0 MB | 1.0 KB | 18.81 ms as a whole process |  |
| Seal | Ceil.source | 18.23 ms | 386.67 µs | 17.84 ms | 18.82 ms | 3.1 MB | 1.2 KB | 151.46 µs cold, 250 ns warm; 2.68 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 17.77 ms | 152.85 µs | 17.63 ms | 17.88 ms | 3.1 MB | 1.2 KB | 168.38 µs cold, 250 ns warm; 2.75 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.24 ms | 216.98 µs | 18.90 ms | 19.21 ms | 3.0 MB | 1.2 KB | 140.38 µs cold, 209 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 19.72 ms | 258.24 µs | 19.40 ms | 19.93 ms | 3.0 MB | 1.2 KB | 136.17 µs cold, 209 ns warm; 2.73 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.83 ms | 595.89 µs | 26.64 ms | 27.69 ms | 4.8 MB | 981 B | 5.34 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 47.77 ms | 648.29 µs | 46.88 ms | 48.39 ms | 2.0 MB | 981 B | 2.52 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.54 ms | 32.10 µs | 1.52 ms | 1.68 ms | 2.3 MB | 981 B | 2.45 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.62 ms | 377.64 µs | 62.02 ms | 63.16 ms | 48.5 MB | 1.0 KB | 20.78 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.67 ms | 200.75 µs | 1.52 ms | 3.59 ms | 53.2 MB | 1.0 KB | 19.00 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.65 ms | 19.49 µs | 1.62 ms | 3.22 ms | 53.0 MB | 1.0 KB | 18.43 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 83.78 ms | 965.62 µs | 82.48 ms | 84.95 ms | 2.9 MB | 596 B | 100.75 µs cold, 250 ns warm; 2.70 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 87.21 ms | 1.24 ms | 85.77 ms | 90.44 ms | 4.8 MB | 523 B | 5.28 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.54 ms | 608.62 µs | 51.67 ms | 51.83 ms | 1.9 MB | 523 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.31 ms | 270.33 µs | 6.09 ms | 6.18 ms | 2.2 MB | 523 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.97 ms | 1.01 ms | 145.62 ms | 152.65 ms | 43.5 MB | 568 B | 18.99 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 40.72 µs | 3.64 ms | 4.98 ms | 49.5 MB | 568 B | 19.01 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.66 ms | 36.94 µs | 3.61 ms | 4.41 ms | 49.5 MB | 568 B | 18.66 ms as a whole process |  |
| Seal | Normal | 104.15 ms | 661.60 µs | 103.05 ms | 103.69 ms | 3.2 MB | 1.8 KB | 153.00 µs cold, 292 ns warm; 2.68 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 221.10 ms | 14.56 ms | 208.20 ms | 215.01 ms | 4.8 MB | 1.7 KB | 5.84 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 149.66 ms | 796.28 µs | 148.31 ms | 148.52 ms | 2.0 MB | 1.7 KB | 2.73 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 50.71 ms | 471.19 µs | 49.83 ms | 49.83 ms | 2.3 MB | 1.7 KB | 2.75 ms as a whole process |  |
| Node `--jitless` | Normal | 155.09 ms | 1.37 ms | 153.74 ms | 157.26 ms | 45.4 MB | 1.7 KB | 19.28 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.11 ms | 116.32 µs | 13.86 ms | 15.32 ms | 52.0 MB | 1.7 KB | 19.15 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.41 ms | 75.46 µs | 14.29 ms | 16.11 ms | 52.3 MB | 1.7 KB | 19.41 ms as a whole process |  |
| Seal | Ceil.source | 104.20 ms | 1.93 ms | 101.71 ms | 102.51 ms | 3.1 MB | 1.8 KB | 152.71 µs cold, 250 ns warm; 2.79 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 102.66 ms | 488.05 µs | 101.61 ms | 104.71 ms | 3.1 MB | 1.8 KB | 152.46 µs cold, 292 ns warm; 2.86 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 108.97 ms | 495.83 µs | 108.65 ms | 109.17 ms | 2.9 MB | 1.8 KB | 133.33 µs cold, 166 ns warm; 2.70 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 112.18 ms | 281.90 µs | 111.70 ms | 113.16 ms | 3.0 MB | 1.8 KB | 144.38 µs cold, 250 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 207.77 ms | 2.01 ms | 202.10 ms | 209.98 ms | 4.9 MB | 1.7 KB | 5.32 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 151.96 ms | 12.45 ms | 146.74 ms | 146.59 ms | 2.0 MB | 1.7 KB | 2.38 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 49.00 ms | 227.19 µs | 48.62 ms | 49.80 ms | 2.3 MB | 1.7 KB | 2.55 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 154.43 ms | 249.56 µs | 153.97 ms | 156.34 ms | 45.5 MB | 1.7 KB | 18.58 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 13.94 ms | 120.06 µs | 13.67 ms | 15.66 ms | 52.2 MB | 1.7 KB | 18.68 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.31 ms | 105.57 µs | 14.15 ms | 15.89 ms | 52.4 MB | 1.7 KB | 18.67 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 128.24 ms | 318.00 µs | 127.80 ms | 129.16 ms | 4.2 MB | 533 B | 118.92 µs cold, 333 ns warm; 2.58 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.71 ms | 372.17 µs | 62.21 ms | 62.37 ms | 4.8 MB | 540 B | 5.98 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 36.08 ms | 1.75 ms | 31.23 ms | 33.64 ms | 2.2 MB | 540 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.94 ms | 28.41 µs | 4.87 ms | 4.88 ms | 2.4 MB | 540 B | 2.20 ms as a whole process |  |
| Node `--jitless` | Purpose | 235.66 ms | 2.62 ms | 232.86 ms | 241.52 ms | 48.7 MB | 514 B | 19.34 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 88.01 ms | 25.01 ms | 64.89 ms | 91.02 ms | 52.8 MB | 514 B | 19.35 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.87 ms | 321.68 µs | 67.38 ms | 87.05 ms | 53.0 MB | 514 B | 18.56 ms as a whole process |  |
| Seal | Normal | 26.69 ms | 517.08 µs | 26.35 ms | 27.01 ms | 4.1 MB | 538 B | 115.54 µs cold, 333 ns warm; 2.74 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.90 ms | 377.42 µs | 30.07 ms | 30.24 ms | 6.0 MB | 478 B | 5.33 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.70 ms | 227.43 µs | 31.32 ms | 28.77 ms | 3.1 MB | 478 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.18 ms | 582.86 µs | 11.62 ms | 13.87 ms | 3.4 MB | 478 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Normal | 51.18 ms | 232.46 µs | 50.89 ms | 52.73 ms | 56.6 MB | 477 B | 18.71 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.06 ms | 340.14 µs | 34.67 ms | 36.23 ms | 62.6 MB | 477 B | 18.59 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 34.84 ms | 331.96 µs | 34.56 ms | 35.70 ms | 88.8 MB | 477 B | 18.75 ms as a whole process |  |
| Seal | Ceil.source | 26.53 ms | 158.18 µs | 26.22 ms | 26.96 ms | 4.1 MB | 538 B | 109.33 µs cold, 459 ns warm; 2.57 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 26.72 ms | 1.34 ms | 26.09 ms | 26.46 ms | 4.1 MB | 538 B | 118.21 µs cold, 375 ns warm; 2.68 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.33 ms | 144.75 µs | 24.05 ms | 24.66 ms | 4.0 MB | 538 B | 93.42 µs cold, 250 ns warm; 2.63 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.08 ms | 223.67 µs | 23.88 ms | 23.92 ms | 4.1 MB | 538 B | 91.54 µs cold, 250 ns warm; 2.65 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.88 ms | 407.27 µs | 29.24 ms | 30.23 ms | 6.0 MB | 478 B | 5.25 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 30.61 ms | 715.95 µs | 28.90 ms | 31.26 ms | 3.1 MB | 478 B | 2.56 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.80 ms | 441.14 µs | 12.12 ms | 12.55 ms | 3.4 MB | 478 B | 2.50 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.54 ms | 276.39 µs | 51.14 ms | 52.40 ms | 56.5 MB | 477 B | 18.83 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.16 ms | 324.40 µs | 34.82 ms | 36.20 ms | 62.6 MB | 477 B | 18.98 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.09 ms | 411.30 µs | 34.68 ms | 35.88 ms | 88.7 MB | 477 B | 18.69 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 323.63 µs | 6.34 µs | 316.33 µs | 357.29 µs | 3.2 MB | 215 B | 79.25 µs cold, 333 ns warm; 2.64 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 143.81 ms | 505.75 µs | 142.92 ms | 145.51 ms | 10.8 MB | 195 B | 5.27 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.31 ms | 115.49 µs | 32.19 ms | 31.66 ms | 3.6 MB | 195 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.81 ms | 225.70 µs | 20.53 ms | 20.73 ms | 3.8 MB | 195 B | 2.53 ms as a whole process |  |
| Node `--jitless` | Purpose | 365.02 µs | 145.64 µs | 237.25 µs | 1.06 ms | 52.7 MB | 223 B | 18.60 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 218.54 µs | 145.13 µs | 89.42 µs | 925.88 µs | 57.9 MB | 223 B | 18.94 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 162.95 µs | 160.12 µs | 54.79 µs | 899.21 µs | 58.7 MB | 223 B | 19.44 ms as a whole process |  |
| Seal | Normal | 360.35 µs | 7.59 µs | 355.12 µs | 374.83 µs | 3.4 MB | 182 B | 85.54 µs cold, 375 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.46 ms | 175.21 µs | 160.20 ms | 165.13 ms | 11.8 MB | 138 B | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.79 ms | 335.31 µs | 41.33 ms | 41.66 ms | 5.1 MB | 138 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.40 ms | 248.15 µs | 19.11 ms | 20.12 ms | 5.2 MB | 138 B | 2.35 ms as a whole process |  |
| Node `--jitless` | Normal | 503.24 µs | 345.47 µs | 263.12 µs | 1.11 ms | 61.4 MB | 155 B | 18.40 ms as a whole process |  |
| Node (JIT), from cold | Normal | 486.10 µs | 467.33 µs | 128.83 µs | 1.04 ms | 65.2 MB | 155 B | 18.38 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 286.24 µs | 330.81 µs | 132.21 µs | 1.08 ms | 69.5 MB | 155 B | 19.25 ms as a whole process |  |
| Seal | Ceil.source | 345.57 µs | 4.32 µs | 342.29 µs | 370.96 µs | 3.4 MB | 182 B | 82.62 µs cold, 375 ns warm; 2.56 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 348.39 µs | 6.35 µs | 342.92 µs | 374.08 µs | 3.4 MB | 182 B | 79.83 µs cold, 292 ns warm; 2.53 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 283.66 µs | 5.37 µs | 277.00 µs | 315.50 µs | 3.2 MB | 182 B | 61.12 µs cold, 250 ns warm; 2.62 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 282.82 µs | 2.49 µs | 278.29 µs | 315.71 µs | 3.2 MB | 182 B | 66.04 µs cold, 250 ns warm; 2.56 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 162.25 ms | 1.20 ms | 160.46 ms | 162.35 ms | 12.1 MB | 138 B | 5.18 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 42.47 ms | 268.75 µs | 42.22 ms | 42.52 ms | 5.1 MB | 138 B | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.47 ms | 214.88 µs | 19.12 ms | 20.11 ms | 5.4 MB | 138 B | 2.47 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 541.75 µs | 345.37 µs | 268.75 µs | 1.12 ms | 61.4 MB | 155 B | 18.93 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 499.38 µs | 482.97 µs | 123.58 µs | 1.06 ms | 65.4 MB | 155 B | 18.82 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 277.97 µs | 329.55 µs | 125.00 µs | 1.10 ms | 69.5 MB | 155 B | 19.26 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.36 ms | 884.74 µs | 74.22 ms | 76.80 ms | 3.1 MB | 519 B | 102.54 µs cold, 292 ns warm; 2.64 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 93.82 ms | 707.57 µs | 92.51 ms | 94.13 ms | 4.9 MB | 411 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 117.43 ms | 1.50 ms | 113.73 ms | 117.06 ms | 2.6 MB | 411 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 93.94 ms | 790.96 µs | 92.79 ms | 94.75 ms | 2.9 MB | 411 B | 2.63 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.38 ms | 377.66 µs | 75.82 ms | 79.84 ms | 59.8 MB | 444 B | 18.98 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.38 ms | 307.80 µs | 10.09 ms | 12.84 ms | 65.9 MB | 444 B | 18.64 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.37 ms | 737.38 µs | 9.94 ms | 12.81 ms | 114.8 MB | 444 B | 18.66 ms as a whole process |  |
| Seal | Normal | 99.87 ms | 500.33 µs | 98.93 ms | 100.51 ms | 8.0 MB | 956 B | 131.67 µs cold, 208 ns warm; 2.90 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 136.92 ms | 8.90 ms | 128.54 ms | 128.17 ms | 44.0 MB | 799 B | 5.29 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 128.34 ms | 1.03 ms | 126.76 ms | 128.96 ms | 41.6 MB | 799 B | 2.63 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.71 ms | 982.49 µs | 80.53 ms | 82.81 ms | 42.2 MB | 799 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Normal | 115.54 ms | 2.07 ms | 112.41 ms | 115.71 ms | 213.3 MB | 878 B | 19.46 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.40 ms | 833.28 µs | 13.53 ms | 17.12 ms | 129.6 MB | 878 B | 19.59 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.84 ms | 151.05 µs | 13.60 ms | 17.48 ms | 224.1 MB | 878 B | 19.92 ms as a whole process |  |
| Seal | Ceil.source | 102.35 ms | 1.52 ms | 100.41 ms | 101.14 ms | 8.0 MB | 956 B | 133.21 µs cold, 250 ns warm; 2.82 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 99.29 ms | 971.50 µs | 97.86 ms | 104.67 ms | 8.0 MB | 956 B | 131.71 µs cold, 250 ns warm; 2.83 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 137.13 ms | 82.62 ms | 96.55 ms | 99.52 ms | 7.8 MB | 956 B | 112.29 µs cold, 167 ns warm; 3.02 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 101.83 ms | 4.36 ms | 98.33 ms | 101.83 ms | 7.8 MB | 956 B | 118.83 µs cold, 208 ns warm; 3.05 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 138.48 ms | 4.70 ms | 132.11 ms | 136.88 ms | 41.9 MB | 799 B | 5.63 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 132.47 ms | 2.35 ms | 129.40 ms | 133.00 ms | 42.0 MB | 799 B | 2.82 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 89.43 ms | 5.40 ms | 83.23 ms | 84.12 ms | 42.9 MB | 799 B | 2.92 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.91 ms | 2.50 ms | 113.93 ms | 122.69 ms | 213.5 MB | 878 B | 19.33 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.28 ms | 887.67 µs | 13.43 ms | 17.05 ms | 130.1 MB | 878 B | 19.26 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.67 ms | 86.96 µs | 13.52 ms | 17.35 ms | 224.5 MB | 878 B | 19.75 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed three ways: every script from a text of its own, so that nothing compiled is reused; every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that; and every script from a compiled file of its own, a `.sealc` for Seal and `string.dump`'s bytecode, as `luajit -b` writes it, for LuaJIT (D171). Seal frees each script loaded from a text or a file of its own inside its timing, before loading the next.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 55.58 µs (±7.48 µs, least 47.79 µs) | 14.71 µs (±3.05 µs, least 12.58 µs) | 9.02 µs (±6.43 µs, least 6.17 µs) | 18 ns (±0 ns, least 18 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 3.9 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.12 µs (±904 ns, least 9.08 µs) | 17.48 µs (±1.75 µs, least 15.54 µs) | ? | 41 ns (±1 ns, least 39 ns) | 20 ns (±2 ns, least 18 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 4.00 µs (±816 ns, least 3.00 µs) | 5.90 µs (±1.20 µs, least 5.00 µs) | 1.40 µs (±516 ns, least 1.00 µs) | 23 ns (±1 ns, least 21 ns) | 17 ns (±1 ns, least 14 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 4.30 µs (±1.06 µs, least 3.00 µs) | 6.10 µs (±876 ns, least 5.00 µs) | 1.50 µs (±1.58 µs, least 1.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 30.42 µs (±7.96 µs, least 19.21 µs) | 85.50 µs (±151.58 µs, least 36.33 µs) | ? | 57 ns (±1 ns, least 55 ns) | 30 ns (±0 ns, least 30 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 30.42 µs (±7.75 µs, least 18.96 µs) | 38.31 µs (±3.46 µs, least 35.67 µs) | ? | 9 ns (±0 ns, least 9 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.83 µs (±57 ns, least 4.75 µs) | 148 ns (±4 ns, least 142 ns) | 702 ns (±21 ns, least 679 ns) | 19 ns (±0 ns, least 19 ns) | 40 ns (±1 ns, least 40 ns) | 512 B | 8.4 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.90 µs (±80 ns, least 4.74 µs) | 188 ns (±7 ns, least 174 ns) | ? | 35 ns (±1 ns, least 34 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.84 µs (±20 ns, least 1.81 µs) | 188 ns (±8 ns, least 175 ns) | 295 ns (±9 ns, least 275 ns) | 28 ns (±1 ns, least 27 ns) | 17 ns (±2 ns, least 14 ns) | 1.0 KB | 5.2 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.85 µs (±24 ns, least 1.82 µs) | 362 ns (±21 ns, least 332 ns) | 487 ns (±80 ns, least 261 ns) | 103 ns (±2 ns, least 101 ns) | 111 ns (±14 ns, least 105 ns) | 1.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.14 µs (±16 ns, least 1.12 µs) | 666 ns (±13 ns, least 646 ns) | ? | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±18 ns, least 1.07 µs) | 667 ns (±18 ns, least 644 ns) | ? | 22 ns (±0 ns, least 21 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 56.0 MB | the engine's own heap |
| Seal | Ceil.source | 4.72 µs (±31 ns, least 4.68 µs) | 151 ns (±6 ns, least 142 ns) | 684 ns (±15 ns, least 663 ns) | 19 ns (±0 ns, least 18 ns) | 39 ns (±1 ns, least 39 ns) | 512 B | 8.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.72 µs (±28 ns, least 4.68 µs) | 151 ns (±4 ns, least 145 ns) | 647 ns (±14 ns, least 634 ns) | 19 ns (±0 ns, least 19 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 8.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.06 µs (±33 ns, least 4.01 µs) | 146 ns (±6 ns, least 133 ns) | 596 ns (±10 ns, least 573 ns) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 8.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.09 µs (±43 ns, least 4.03 µs) | 145 ns (±7 ns, least 133 ns) | 594 ns (±9 ns, least 575 ns) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 8.1 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.81 µs (±125 ns, least 4.59 µs) | 182 ns (±4 ns, least 178 ns) | ? | 34 ns (±2 ns, least 32 ns) | 17 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.78 µs (±16 ns, least 1.76 µs) | 188 ns (±7 ns, least 180 ns) | 287 ns (±11 ns, least 273 ns) | 27 ns (±1 ns, least 26 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.81 µs (±19 ns, least 1.79 µs) | 370 ns (±33 ns, least 345 ns) | 454 ns (±111 ns, least 243 ns) | 92 ns (±4 ns, least 89 ns) | 106 ns (±12 ns, least 98 ns) | 1.0 KB | 6.0 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±26 ns, least 1.10 µs) | 669 ns (±20 ns, least 643 ns) | ? | 50 ns (±1 ns, least 49 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±20 ns, least 1.08 µs) | 664 ns (±17 ns, least 636 ns) | ? | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 34.60 ms | 37.34 ms (+7.9%) |
| n-body, 100,000 steps | 66.72 ms | 62.97 ms (-5.6%) |
| spectral-norm, size 200 | 18.41 ms | 20.65 ms (+12.2%) |
| fannkuch-redux, 9 | 103.92 ms | 109.68 ms (+5.5%) |
| word count, 500,000 words | 26.32 ms | 26.54 ms (+0.8%) |
| text building, 20,000 joins | 344.75 µs | 344.18 µs (-0.2%) |
| binary-trees, depth 14 | 98.46 ms | 107.25 ms (+8.9%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Seal mean | Seal deviation | Seal least | Seal peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.35 ms | 9.23 µs | 1.35 ms | 3.8 MB | 1.60 ms | 1.58 ms | 303.34 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.38 ms | 13.53 µs | 1.37 ms | 3.5 MB | 1.51 ms | 1.49 ms | 1.09 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 112.79 ms | 291.07 µs | 112.12 ms | 3.0 MB | 162.06 ms | 161.74 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 66.99 ms | 139.09 µs | 66.83 ms | 3.0 MB | 80.38 ms | 79.92 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 416.06 µs | 1.51 µs | 414.42 µs | 3.5 MB | 903.47 µs | 887.21 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 92.39 ms | 2.19 ms | 88.24 ms | 3.0 MB | 160.02 ms | 159.25 ms | – |
| the money sums in `Float`s, as a yardstick | 10.04 ms | 313.23 µs | 9.54 ms | 2.9 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Seal's door is written in C, through Seal's C API (`crates/seal-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Seal's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Seal | C | 11 ns | |
| Seal | Rust | 7 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
