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
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.82 2.25 2.25 }` before, `{ 2.25 2.26 2.25 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 31.22 ms | 73.94 ms | **met**: Seal takes 42% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.02 ms | 69.18 ms | **met**: Seal takes 95% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.40 ms | 27.46 ms | **met**: Seal takes 67% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 116.28 ms | 224.29 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.70 ms | 32.87 ms | **met**: Seal takes 81% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 354.92 µs | 162.73 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 100.76 ms | 129.37 ms | **met**: Seal takes 78% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.38 ms | 3.73 ms | **met**: Seal takes 64% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 22 ns | 35 ns | **met**: Seal takes 63% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 169 ns | 205 ns | **met**: Seal takes 83% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.79 µs | 4.83 µs | **met**: Seal takes 99% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 31.22 ms | 33.81 ms | **met**: Seal takes 92% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 66.02 ms | 71.92 ms | **met**: Seal takes 92% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.40 ms | 45.87 ms | **met**: Seal takes 40% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 116.28 ms | 147.94 ms | **met**: Seal takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.70 ms | 33.17 ms | **met**: Seal takes 81% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 354.92 µs | 41.38 ms | **met**: Seal takes 0.9% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 100.76 ms | 126.76 ms | **met**: Seal takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.38 ms | 3.02 ms | **met**: Seal takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 22 ns | 28 ns | **met**: Seal takes 78% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 169 ns | 192 ns | **met**: Seal takes 88% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.79 µs | 1.83 µs | for reference, no target (D150) |
| D171. Faster than LuaJIT loading its own bytecode | patrolling scripts, loading each from a compiled file of its own | 965 ns | 323 ns | **not met**: Seal takes 299% of its time |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 12 ns | a Rust door: 8 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.6× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.38 ms | 1.60 ms | **met**: Seal takes 86% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.44 ms | 1.54 ms | **met**: Seal takes 94% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 111.13 ms | 169.86 ms | **met**: Seal takes 65% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 70.65 ms | 86.05 ms | **met**: Seal takes 82% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 425.16 µs | 925.21 µs | **met**: Seal takes 46% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 92.21 ms | 173.38 ms | **met**: Seal takes 53% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 31.22 ms | 93.33 ms | **met**: Seal takes 33% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.02 ms | 177.36 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.70 ms | 52.44 ms | **met**: Seal takes 51% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 354.92 µs | 526.65 µs | **met**: Seal takes 67% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 22 ns | host call 42 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.41 ms | 799.10 µs | 83.32 ms | 107.06 ms | 2.9 MB | 395 B | 830.38 µs cold, 13.25 µs warm; 2.64 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 195.89 ms | 345.62 µs | 195.41 ms | 195.93 ms | 4.8 MB | 339 B | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 89.82 ms | 898.29 µs | 88.07 ms | 89.03 ms | 1.9 MB | 339 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.80 ms | 319.03 µs | 44.25 ms | 45.30 ms | 2.3 MB | 339 B | 2.57 ms as a whole process |  |
| Node `--jitless` | Purpose | 321.75 ms | 2.36 ms | 318.76 ms | 324.61 ms | 43.5 MB | 362 B | 19.47 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 125.95 ms | 999.21 µs | 124.93 ms | 131.36 ms | 49.5 MB | 362 B | 18.83 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 123.78 ms | 250.37 µs | 123.34 ms | 125.00 ms | 49.4 MB | 362 B | 19.30 ms as a whole process |  |
| Seal | Normal | 31.22 ms | 282.08 µs | 30.72 ms | 31.49 ms | 2.8 MB | 169 B | 82.62 µs cold, 833 ns warm; 2.71 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled |
| Luau (interpreter) | Normal | 73.94 ms | 1.01 ms | 72.66 ms | 71.92 ms | 4.8 MB | 143 B | 5.68 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.81 ms | 148.02 µs | 33.58 ms | 33.85 ms | 1.9 MB | 143 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 6.94 ms | 21.06 µs | 6.92 ms | 7.12 ms | 2.2 MB | 143 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 93.33 ms | 815.44 µs | 92.25 ms | 92.66 ms | 43.4 MB | 137 B | 19.47 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.54 ms | 202.90 µs | 9.40 ms | 10.23 ms | 48.6 MB | 137 B | 19.23 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.45 ms | 44.38 µs | 9.38 ms | 10.46 ms | 48.7 MB | 137 B | 19.59 ms as a whole process |  |
| Seal | Ceil.source | 31.56 ms | 316.41 µs | 30.98 ms | 30.76 ms | 2.8 MB | 169 B | 83.00 µs cold, 792 ns warm; 2.67 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 32.26 ms | 417.14 µs | 31.45 ms | 51.66 ms | 2.8 MB | 169 B | 878.71 µs cold, 583 ns warm; 2.75 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.program | 30.52 ms | 623.31 µs | 29.67 ms | 41.61 ms | 2.6 MB | 169 B | 657.92 µs cold, 333 ns warm; 2.78 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 32.85 ms | 301.67 µs | 32.46 ms | 57.30 ms | 2.6 MB | 169 B | 837.33 µs cold, 292 ns warm; 2.62 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.41 ms | 645.35 µs | 70.18 ms | 71.67 ms | 4.8 MB | 143 B | 5.04 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.94 ms | 221.59 µs | 33.48 ms | 33.78 ms | 1.9 MB | 143 B | 2.63 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.39 ms | 43.65 µs | 7.31 ms | 7.70 ms | 2.2 MB | 143 B | 2.65 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 103.91 ms | 15.65 ms | 94.13 ms | 96.22 ms | 43.3 MB | 137 B | 19.99 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 10.35 ms | 597.67 µs | 9.53 ms | 10.02 ms | 48.7 MB | 137 B | 20.36 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.56 ms | 436.09 µs | 10.23 ms | 10.52 ms | 48.8 MB | 137 B | 19.42 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 47.94 ms | 683.42 µs | 46.59 ms | 49.90 ms | 2.8 MB | 550 B | 130.79 µs cold, 834 ns warm; 2.78 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.40 ms | 601.76 µs | 90.66 ms | 91.45 ms | 4.8 MB | 433 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.35 ms | 396.81 µs | 75.80 ms | 76.56 ms | 1.9 MB | 433 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.72 ms | 15.04 µs | 6.70 ms | 6.75 ms | 2.1 MB | 433 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 242.98 ms | 1.10 ms | 240.89 ms | 243.83 ms | 46.5 MB | 460 B | 18.67 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.95 ms | 59.72 µs | 18.86 ms | 19.10 ms | 50.5 MB | 460 B | 19.29 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.96 ms | 126.45 µs | 18.77 ms | 18.93 ms | 50.5 MB | 460 B | 19.88 ms as a whole process |  |
| Seal | Normal | 66.02 ms | 397.04 µs | 65.37 ms | 66.56 ms | 3.4 MB | 3.5 KB | 290.46 µs cold, 917 ns warm; 2.89 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.18 ms | 283.48 µs | 68.82 ms | 69.02 ms | 4.8 MB | 3.0 KB | 5.36 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.92 ms | 504.87 µs | 70.93 ms | 71.34 ms | 1.9 MB | 3.0 KB | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.26 ms | 39.63 µs | 5.19 ms | 5.47 ms | 2.2 MB | 3.0 KB | 2.51 ms as a whole process |  |
| Node `--jitless` | Normal | 177.36 ms | 5.83 ms | 171.20 ms | 180.54 ms | 48.2 MB | 3.1 KB | 19.67 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 30.32 µs | 3.82 ms | 5.92 ms | 52.1 MB | 3.1 KB | 18.30 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 37.43 µs | 3.83 ms | 5.68 ms | 52.0 MB | 3.1 KB | 19.55 ms as a whole process |  |
| Seal | Ceil.source | 53.91 ms | 577.49 µs | 53.00 ms | 56.12 ms | 3.3 MB | 3.4 KB | 313.75 µs cold, 875 ns warm; 2.79 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 57.51 ms | 626.19 µs | 56.16 ms | 57.05 ms | 3.3 MB | 3.4 KB | 305.46 µs cold, 333 ns warm; 2.92 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 46.83 ms | 331.21 µs | 46.31 ms | 45.99 ms | 3.1 MB | 3.4 KB | 259.92 µs cold, 250 ns warm; 2.82 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 46.49 ms | 225.38 µs | 46.13 ms | 46.41 ms | 3.1 MB | 3.4 KB | 282.50 µs cold, 250 ns warm; 3.00 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.44 ms | 341.40 µs | 68.98 ms | 69.95 ms | 4.8 MB | 3.0 KB | 5.64 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.42 ms | 294.47 µs | 70.96 ms | 71.42 ms | 1.9 MB | 3.0 KB | 2.53 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.22 ms | 24.20 µs | 5.19 ms | 5.45 ms | 2.2 MB | 3.0 KB | 2.55 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.99 ms | 3.76 ms | 174.20 ms | 181.97 ms | 48.2 MB | 3.1 KB | 19.80 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 30.53 µs | 3.84 ms | 5.73 ms | 52.2 MB | 3.1 KB | 19.55 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 19.97 µs | 3.84 ms | 5.66 ms | 52.2 MB | 3.1 KB | 19.70 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 55.27 ms | 2.68 ms | 49.71 ms | 56.15 ms | 3.0 MB | 429 B | 139.58 µs cold, 917 ns warm; 2.66 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 44.90 ms | 300.71 µs | 44.58 ms | 44.51 ms | 4.8 MB | 366 B | 5.28 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.34 ms | 306.42 µs | 76.93 ms | 76.69 ms | 2.0 MB | 366 B | 2.72 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 23.33 ms | 262.82 µs | 22.87 ms | 23.26 ms | 2.2 MB | 366 B | 2.35 ms as a whole process |  |
| Node `--jitless` | Purpose | 367.74 ms | 4.75 ms | 361.97 ms | 375.93 ms | 48.7 MB | 398 B | 18.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 33.38 ms | 36.33 ms | 15.47 ms | 20.23 ms | 52.6 MB | 398 B | 18.86 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.22 ms | 15.27 µs | 14.19 ms | 20.28 ms | 55.5 MB | 398 B | 18.38 ms as a whole process |  |
| Seal | Normal | 18.40 ms | 554.22 µs | 17.63 ms | 19.06 ms | 3.2 MB | 1.2 KB | 151.17 µs cold, 834 ns warm; 2.76 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.46 ms | 502.26 µs | 26.83 ms | 27.31 ms | 4.8 MB | 981 B | 5.08 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.87 ms | 916.29 µs | 45.09 ms | 46.46 ms | 2.0 MB | 981 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.52 ms | 15.42 µs | 1.51 ms | 1.69 ms | 2.3 MB | 981 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Normal | 63.89 ms | 413.72 µs | 63.30 ms | 63.72 ms | 48.5 MB | 1.0 KB | 19.02 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.54 ms | 172.14 µs | 1.42 ms | 3.30 ms | 53.0 MB | 1.0 KB | 18.96 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.65 ms | 18.98 µs | 1.62 ms | 3.46 ms | 52.9 MB | 1.0 KB | 19.84 ms as a whole process |  |
| Seal | Ceil.source | 19.00 ms | 133.89 µs | 18.82 ms | 19.12 ms | 3.2 MB | 1.2 KB | 152.54 µs cold, 875 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.03 ms | 113.09 µs | 18.81 ms | 18.85 ms | 3.2 MB | 1.2 KB | 155.88 µs cold, 291 ns warm; 2.82 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 17.38 ms | 96.94 µs | 17.26 ms | 17.39 ms | 2.9 MB | 1.2 KB | 133.54 µs cold, 208 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 17.72 ms | 289.41 µs | 17.43 ms | 17.62 ms | 2.9 MB | 1.2 KB | 140.62 µs cold, 208 ns warm; 2.99 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.19 ms | 220.18 µs | 26.84 ms | 27.52 ms | 4.8 MB | 981 B | 5.82 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 47.03 ms | 562.05 µs | 46.31 ms | 47.70 ms | 2.0 MB | 981 B | 2.63 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.54 ms | 23.15 µs | 1.51 ms | 1.68 ms | 2.3 MB | 981 B | 2.58 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 65.74 ms | 1.29 ms | 63.99 ms | 113.78 ms | 48.5 MB | 1.0 KB | 20.90 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.66 ms | 210.34 µs | 1.43 ms | 3.28 ms | 53.0 MB | 1.0 KB | 20.05 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.70 ms | 56.70 µs | 1.62 ms | 3.71 ms | 53.0 MB | 1.0 KB | 20.13 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 87.58 ms | 1.04 ms | 86.37 ms | 88.00 ms | 2.9 MB | 632 B | 140.75 µs cold, 917 ns warm; 2.82 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 125.47 ms | 31.70 ms | 98.22 ms | 94.12 ms | 4.9 MB | 523 B | 5.72 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 63.80 ms | 7.10 ms | 56.00 ms | 59.10 ms | 1.9 MB | 523 B | 3.24 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.82 ms | 368.46 µs | 6.46 ms | 6.51 ms | 2.2 MB | 523 B | 3.27 ms as a whole process |  |
| Node `--jitless` | Purpose | 153.47 ms | 4.25 ms | 148.15 ms | 184.70 ms | 43.5 MB | 568 B | 19.41 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.75 ms | 90.00 µs | 3.61 ms | 4.77 ms | 49.5 MB | 568 B | 21.07 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.73 ms | 50.68 µs | 3.65 ms | 4.54 ms | 49.5 MB | 568 B | 21.27 ms as a whole process |  |
| Seal | Normal | 116.28 ms | 8.30 ms | 106.81 ms | 108.04 ms | 3.1 MB | 1.8 KB | 163.12 µs cold, 958 ns warm; 2.74 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 224.29 ms | 9.72 ms | 213.42 ms | 220.96 ms | 4.8 MB | 1.7 KB | 5.39 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.94 ms | 970.24 µs | 146.98 ms | 146.14 ms | 2.0 MB | 1.7 KB | 2.79 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 49.44 ms | 789.76 µs | 48.31 ms | 48.96 ms | 2.3 MB | 1.7 KB | 2.78 ms as a whole process |  |
| Node `--jitless` | Normal | 160.58 ms | 3.68 ms | 157.06 ms | 160.28 ms | 45.4 MB | 1.7 KB | 19.70 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.37 ms | 104.35 µs | 14.23 ms | 16.51 ms | 52.0 MB | 1.7 KB | 18.80 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.79 ms | 94.85 µs | 14.58 ms | 16.37 ms | 52.3 MB | 1.7 KB | 19.57 ms as a whole process |  |
| Seal | Ceil.source | 103.15 ms | 394.96 µs | 102.74 ms | 103.22 ms | 3.2 MB | 1.8 KB | 155.54 µs cold, 917 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 105.10 ms | 337.46 µs | 104.64 ms | 106.24 ms | 3.2 MB | 1.8 KB | 2.41 ms cold, 750 ns warm; 2.80 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 98.44 ms | 141.30 µs | 98.27 ms | 98.68 ms | 2.9 MB | 1.8 KB | 1.69 ms cold, 208 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 100.33 ms | 376.47 µs | 99.70 ms | 99.60 ms | 2.9 MB | 1.8 KB | 2.14 ms cold, 208 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 222.61 ms | 11.81 ms | 215.74 ms | 217.00 ms | 4.8 MB | 1.7 KB | 5.62 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 147.65 ms | 776.81 µs | 146.57 ms | 146.63 ms | 2.0 MB | 1.7 KB | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 50.59 ms | 887.17 µs | 49.56 ms | 49.92 ms | 2.3 MB | 1.7 KB | 2.58 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 150.64 ms | 1.12 ms | 149.37 ms | 155.15 ms | 45.4 MB | 1.7 KB | 18.89 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.19 ms | 43.27 µs | 14.12 ms | 15.56 ms | 52.1 MB | 1.7 KB | 19.63 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.09 ms | 71.47 µs | 14.01 ms | 15.73 ms | 52.4 MB | 1.7 KB | 19.02 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 129.72 ms | 2.69 ms | 127.67 ms | 127.92 ms | 4.2 MB | 569 B | 162.04 µs cold, 1.21 µs warm; 2.70 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 64.13 ms | 2.79 ms | 62.86 ms | 63.12 ms | 4.8 MB | 540 B | 5.46 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 30.14 ms | 1.46 ms | 28.32 ms | 30.24 ms | 2.2 MB | 540 B | 2.38 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 5.19 ms | 39.62 µs | 5.12 ms | 5.12 ms | 2.4 MB | 540 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 241.22 ms | 786.10 µs | 240.17 ms | 243.61 ms | 48.8 MB | 514 B | 18.79 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 82.75 ms | 19.02 ms | 68.12 ms | 89.93 ms | 52.7 MB | 514 B | 19.17 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.75 ms | 515.46 µs | 67.21 ms | 94.86 ms | 52.8 MB | 514 B | 19.52 ms as a whole process |  |
| Seal | Normal | 26.70 ms | 111.86 µs | 26.57 ms | 26.49 ms | 4.1 MB | 538 B | 148.54 µs cold, 1.04 µs warm; 2.78 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 32.87 ms | 669.17 µs | 31.68 ms | 30.23 ms | 6.0 MB | 478 B | 5.29 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.17 ms | 577.70 µs | 31.66 ms | 31.79 ms | 3.1 MB | 478 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.47 ms | 466.98 µs | 11.98 ms | 12.56 ms | 3.4 MB | 478 B | 2.56 ms as a whole process |  |
| Node `--jitless` | Normal | 52.44 ms | 327.07 µs | 51.95 ms | 56.85 ms | 56.5 MB | 477 B | 19.54 ms as a whole process |  |
| Node (JIT), from cold | Normal | 36.01 ms | 506.96 µs | 35.35 ms | 36.56 ms | 62.5 MB | 477 B | 19.48 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 36.06 ms | 242.62 µs | 35.73 ms | 36.95 ms | 88.6 MB | 477 B | 19.67 ms as a whole process |  |
| Seal | Ceil.source | 26.74 ms | 110.54 µs | 26.59 ms | 26.81 ms | 4.1 MB | 538 B | 112.79 µs cold, 1.08 µs warm; 2.50 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.31 ms | 93.00 µs | 27.16 ms | 27.49 ms | 4.1 MB | 538 B | 111.08 µs cold, 291 ns warm; 2.75 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.10 ms | 222.36 µs | 23.73 ms | 24.77 ms | 3.9 MB | 538 B | 4.83 ms cold, 250 ns warm; 2.72 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.96 ms | 192.61 µs | 24.57 ms | 25.09 ms | 4.1 MB | 538 B | 2.89 ms cold, 292 ns warm; 2.83 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.30 ms | 131.04 µs | 30.06 ms | 30.13 ms | 6.0 MB | 478 B | 5.54 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.00 ms | 322.66 µs | 30.50 ms | 30.64 ms | 3.1 MB | 478 B | 2.59 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.90 ms | 428.06 µs | 12.11 ms | 12.33 ms | 3.4 MB | 478 B | 2.45 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.93 ms | 365.41 µs | 51.41 ms | 53.04 ms | 56.5 MB | 477 B | 19.05 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.99 ms | 489.09 µs | 35.34 ms | 36.66 ms | 62.7 MB | 477 B | 19.28 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 36.42 ms | 412.86 µs | 35.88 ms | 36.02 ms | 88.3 MB | 477 B | 19.76 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 329.79 µs | 4.20 µs | 325.67 µs | 344.42 µs | 3.2 MB | 233 B | 106.17 µs cold, 1.04 µs warm; 2.61 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 147.07 ms | 1.73 ms | 145.74 ms | 147.36 ms | 10.8 MB | 195 B | 5.42 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 33.14 ms | 100.15 µs | 32.96 ms | 32.17 ms | 3.6 MB | 195 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.75 ms | 155.76 µs | 20.55 ms | 21.19 ms | 3.9 MB | 195 B | 2.61 ms as a whole process |  |
| Node `--jitless` | Purpose | 401.39 µs | 147.54 µs | 275.00 µs | 1.12 ms | 52.5 MB | 223 B | 18.82 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 223.95 µs | 159.98 µs | 87.50 µs | 976.08 µs | 57.9 MB | 223 B | 19.54 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 165.50 µs | 165.86 µs | 55.96 µs | 935.00 µs | 58.6 MB | 223 B | 19.17 ms as a whole process |  |
| Seal | Normal | 354.92 µs | 7.90 µs | 346.79 µs | 382.71 µs | 3.4 MB | 182 B | 171.96 µs cold, 1.08 µs warm; 2.56 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 162.73 ms | 135.75 µs | 162.57 ms | 165.40 ms | 11.9 MB | 138 B | 5.39 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.38 ms | 232.09 µs | 40.98 ms | 42.29 ms | 5.2 MB | 138 B | 2.38 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 20.41 ms | 291.01 µs | 19.97 ms | 20.57 ms | 5.3 MB | 138 B | 2.55 ms as a whole process |  |
| Node `--jitless` | Normal | 526.65 µs | 339.42 µs | 286.38 µs | 1.15 ms | 61.4 MB | 155 B | 18.36 ms as a whole process |  |
| Node (JIT), from cold | Normal | 496.44 µs | 462.77 µs | 148.17 µs | 1.07 ms | 65.3 MB | 155 B | 19.17 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 283.63 µs | 334.39 µs | 122.29 µs | 1.06 ms | 69.4 MB | 155 B | 19.02 ms as a whole process |  |
| Seal | Ceil.source | 355.20 µs | 5.30 µs | 349.83 µs | 374.08 µs | 3.4 MB | 182 B | 80.29 µs cold, 875 ns warm; 2.59 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 360.04 µs | 10.54 µs | 352.33 µs | 380.42 µs | 3.4 MB | 182 B | 78.71 µs cold, 375 ns warm; 2.66 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 291.72 µs | 4.84 µs | 285.08 µs | 324.00 µs | 3.1 MB | 182 B | 61.00 µs cold, 292 ns warm; 2.59 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 287.60 µs | 2.94 µs | 284.88 µs | 316.50 µs | 3.2 MB | 182 B | 62.42 µs cold, 250 ns warm; 2.62 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 162.41 ms | 165.46 µs | 162.15 ms | 164.53 ms | 11.8 MB | 138 B | 5.28 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.90 ms | 323.94 µs | 41.44 ms | 41.13 ms | 5.3 MB | 138 B | 2.48 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.62 ms | 186.57 µs | 19.37 ms | 19.41 ms | 5.4 MB | 138 B | 2.51 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 512.73 µs | 348.61 µs | 266.42 µs | 1.14 ms | 61.4 MB | 155 B | 19.36 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 496.76 µs | 470.65 µs | 136.71 µs | 1.12 ms | 65.3 MB | 155 B | 19.46 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 280.43 µs | 333.42 µs | 122.96 µs | 1.11 ms | 69.5 MB | 155 B | 18.87 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 74.43 ms | 789.16 µs | 73.13 ms | 75.67 ms | 3.1 MB | 537 B | 124.42 µs cold, 833 ns warm; 2.70 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 96.90 ms | 1.17 ms | 95.54 ms | 94.91 ms | 4.9 MB | 411 B | 5.24 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 119.14 ms | 1.55 ms | 115.48 ms | 118.49 ms | 2.6 MB | 411 B | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 95.45 ms | 829.55 µs | 94.26 ms | 95.89 ms | 2.9 MB | 411 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.92 ms | 478.92 µs | 76.89 ms | 82.02 ms | 59.7 MB | 444 B | 18.77 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.46 ms | 300.22 µs | 10.14 ms | 13.09 ms | 65.9 MB | 444 B | 19.09 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.62 ms | 754.75 µs | 10.15 ms | 13.12 ms | 115.0 MB | 444 B | 19.59 ms as a whole process |  |
| Seal | Normal | 100.76 ms | 882.50 µs | 99.97 ms | 103.20 ms | 7.9 MB | 992 B | 186.38 µs cold, 958 ns warm; 2.63 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 129.37 ms | 1.62 ms | 128.06 ms | 130.85 ms | 41.5 MB | 799 B | 5.41 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 126.76 ms | 1.18 ms | 124.94 ms | 126.65 ms | 41.6 MB | 799 B | 2.57 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.11 ms | 602.72 µs | 80.38 ms | 81.36 ms | 40.9 MB | 799 B | 2.53 ms as a whole process |  |
| Node `--jitless` | Normal | 112.68 ms | 2.21 ms | 110.33 ms | 117.33 ms | 213.3 MB | 878 B | 18.68 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.36 ms | 981.61 µs | 13.48 ms | 17.01 ms | 129.6 MB | 878 B | 19.03 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.36 ms | 72.77 µs | 13.24 ms | 17.15 ms | 223.9 MB | 878 B | 18.83 ms as a whole process |  |
| Seal | Ceil.source | 100.55 ms | 616.93 µs | 99.91 ms | 101.50 ms | 7.9 MB | 992 B | 126.88 µs cold, 834 ns warm; 2.73 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 99.82 ms | 811.68 µs | 98.70 ms | 101.64 ms | 8.0 MB | 992 B | 147.12 µs cold, 250 ns warm; 2.68 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 91.82 ms | 545.43 µs | 91.09 ms | 92.29 ms | 7.7 MB | 992 B | 118.62 µs cold, 167 ns warm; 2.60 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 97.37 ms | 13.83 ms | 92.59 ms | 93.91 ms | 7.7 MB | 992 B | 113.08 µs cold, 167 ns warm; 2.73 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 132.77 ms | 2.88 ms | 129.25 ms | 130.86 ms | 37.8 MB | 799 B | 5.20 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 129.03 ms | 1.08 ms | 127.55 ms | 127.15 ms | 42.1 MB | 799 B | 2.72 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 82.12 ms | 1.30 ms | 80.19 ms | 81.00 ms | 43.9 MB | 799 B | 2.61 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.54 ms | 2.18 ms | 113.15 ms | 118.84 ms | 213.3 MB | 878 B | 19.74 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.36 ms | 1.14 ms | 13.30 ms | 17.20 ms | 129.5 MB | 878 B | 19.40 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.43 ms | 147.24 µs | 13.29 ms | 17.69 ms | 223.7 MB | 878 B | 19.10 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed three ways: every script from a text of its own, so that nothing compiled is reused; every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that; and every script from a compiled file of its own, a `.sealc` for Seal and `string.dump`'s bytecode, as `luajit -b` writes it, for LuaJIT (D171). Seal frees each script loaded from a text or a file of its own inside its timing, before loading the next.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 61.01 µs (±12.62 µs, least 52.08 µs) | 20.34 µs (±6.81 µs, least 13.67 µs) | 9.33 µs (±4.13 µs, least 6.71 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 3.9 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.79 µs (±1.05 µs, least 9.25 µs) | 17.85 µs (±501 ns, least 17.08 µs) | ? | 42 ns (±2 ns, least 40 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.80 µs (±632 ns, least 3.00 µs) | 7.40 µs (±1.84 µs, least 5.00 µs) | 1.20 µs (±422 ns, least 1.00 µs) | 22 ns (±1 ns, least 20 ns) | 15 ns (±1 ns, least 14 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.90 µs (±316 ns, least 3.00 µs) | 7.00 µs (±1.76 µs, least 5.00 µs) | 1.10 µs (±316 ns, least 1.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 44.05 µs (±21.70 µs, least 20.83 µs) | 87.04 µs (±131.75 µs, least 39.71 µs) | ? | 57 ns (±1 ns, least 55 ns) | 30 ns (±0 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 51.88 µs (±32.69 µs, least 19.75 µs) | 58.27 µs (±35.18 µs, least 38.50 µs) | ? | 10 ns (±3 ns, least 8 ns) | 2 ns (±1 ns, least 2 ns) | 0 B | 53.2 MB | the engine's own heap |
| Seal | Normal | 4.79 µs (±112 ns, least 4.66 µs) | 169 ns (±12 ns, least 157 ns) | 965 ns (±32 ns, least 925 ns) | 22 ns (±3 ns, least 20 ns) | 42 ns (±1 ns, least 41 ns) | 512 B | 8.3 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.83 µs (±94 ns, least 4.71 µs) | 205 ns (±12 ns, least 189 ns) | ? | 35 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.83 µs (±60 ns, least 1.78 µs) | 192 ns (±14 ns, least 180 ns) | 323 ns (±93 ns, least 281 ns) | 28 ns (±1 ns, least 27 ns) | 15 ns (±1 ns, least 13 ns) | 1.0 KB | 5.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.88 µs (±66 ns, least 1.80 µs) | 380 ns (±17 ns, least 354 ns) | 526 ns (±99 ns, least 254 ns) | 104 ns (±4 ns, least 99 ns) | 112 ns (±13 ns, least 105 ns) | 1.0 KB | 6.0 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.18 µs (±41 ns, least 1.14 µs) | 714 ns (±43 ns, least 680 ns) | ? | 51 ns (±1 ns, least 49 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.19 µs (±86 ns, least 1.08 µs) | 718 ns (±72 ns, least 664 ns) | ? | 23 ns (±1 ns, least 22 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 56.1 MB | the engine's own heap |
| Seal | Ceil.source | 4.78 µs (±75 ns, least 4.70 µs) | 159 ns (±13 ns, least 147 ns) | 957 ns (±14 ns, least 929 ns) | 21 ns (±1 ns, least 20 ns) | 41 ns (±1 ns, least 41 ns) | 512 B | 8.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.83 µs (±79 ns, least 4.74 µs) | 173 ns (±12 ns, least 155 ns) | 942 ns (±18 ns, least 914 ns) | 21 ns (±0 ns, least 20 ns) | 42 ns (±0 ns, least 41 ns) | 512 B | 8.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.15 µs (±93 ns, least 4.04 µs) | 154 ns (±13 ns, least 138 ns) | 868 ns (±28 ns, least 829 ns) | 18 ns (±1 ns, least 18 ns) | 38 ns (±1 ns, least 38 ns) | 512 B | 8.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.11 µs (±53 ns, least 4.05 µs) | 154 ns (±14 ns, least 133 ns) | 859 ns (±15 ns, least 840 ns) | 18 ns (±0 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 8.1 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.93 µs (±107 ns, least 4.82 µs) | 192 ns (±11 ns, least 180 ns) | ? | 35 ns (±1 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.83 µs (±27 ns, least 1.79 µs) | 187 ns (±6 ns, least 179 ns) | 296 ns (±18 ns, least 269 ns) | 27 ns (±1 ns, least 27 ns) | 16 ns (±2 ns, least 14 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.88 µs (±41 ns, least 1.81 µs) | 370 ns (±18 ns, least 349 ns) | 446 ns (±133 ns, least 245 ns) | 98 ns (±3 ns, least 94 ns) | 114 ns (±18 ns, least 101 ns) | 1.0 KB | 6.0 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.17 µs (±30 ns, least 1.12 µs) | 691 ns (±18 ns, least 667 ns) | ? | 51 ns (±2 ns, least 48 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.15 µs (±88 ns, least 1.06 µs) | 713 ns (±66 ns, least 640 ns) | ? | 22 ns (±1 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 31.54 ms | 31.02 ms (-1.7%) |
| n-body, 100,000 steps | 66.75 ms | 61.35 ms (-8.1%) |
| spectral-norm, size 200 | 18.75 ms | 18.74 ms (-0.1%) |
| fannkuch-redux, 9 | 104.63 ms | 106.79 ms (+2.1%) |
| word count, 500,000 words | 26.96 ms | 27.15 ms (+0.7%) |
| text building, 20,000 joins | 356.81 µs | 354.02 µs (-0.8%) |
| binary-trees, depth 14 | 102.21 ms | 100.60 ms (-1.6%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Seal mean | Seal deviation | Seal least | Seal peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.38 ms | 22.79 µs | 1.36 ms | 3.7 MB | 1.60 ms | 1.57 ms | 309.32 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.44 ms | 34.27 µs | 1.41 ms | 3.5 MB | 1.54 ms | 1.51 ms | 1.11 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 111.13 ms | 936.09 µs | 110.11 ms | 3.0 MB | 169.86 ms | 162.62 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 70.65 ms | 3.06 ms | 68.26 ms | 3.0 MB | 86.05 ms | 82.12 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 425.16 µs | 12.48 µs | 413.58 µs | 3.5 MB | 925.21 µs | 902.54 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 92.21 ms | 1.99 ms | 88.49 ms | 3.0 MB | 173.38 ms | 163.41 ms | – |
| the money sums in `Float`s, as a yardstick | 10.38 ms | 107.01 µs | 10.25 ms | 2.9 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Seal's door is written in C, through Seal's C API (`crates/seal-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Seal's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Seal | C | 12 ns | |
| Seal | Rust | 8 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
