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
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.28 2.03 2.01 }` before, `{ 1.96 1.96 1.98 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 32.12 ms | 71.76 ms | **met**: Seal takes 45% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 64.55 ms | 69.86 ms | **met**: Seal takes 92% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.56 ms | 27.20 ms | **met**: Seal takes 68% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 101.53 ms | 214.58 ms | **met**: Seal takes 47% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.54 ms | 30.39 ms | **met**: Seal takes 87% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 352.73 µs | 160.32 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 99.40 ms | 134.65 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.30 ms | 3.78 ms | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 21 ns | 36 ns | **met**: Seal takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 168 ns | 213 ns | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.76 µs | 5.75 µs | **met**: Seal takes 83% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 32.12 ms | 33.95 ms | **met**: Seal takes 95% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 64.55 ms | 73.06 ms | **met**: Seal takes 88% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.56 ms | 45.91 ms | **met**: Seal takes 40% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 101.53 ms | 147.09 ms | **met**: Seal takes 69% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.54 ms | 32.04 ms | **met**: Seal takes 83% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 352.73 µs | 42.25 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 99.40 ms | 129.50 ms | **met**: Seal takes 77% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.30 ms | 3.72 ms | **met**: Seal takes 62% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 21 ns | 35 ns | **met**: Seal takes 61% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 168 ns | 218 ns | **met**: Seal takes 77% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.76 µs | 2.20 µs | for reference, no target (D150) |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.36 ms | 1.57 ms | **met**: Seal takes 86% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.35 ms | 1.53 ms | **met**: Seal takes 88% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 111.48 ms | 163.40 ms | **met**: Seal takes 68% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 68.08 ms | 85.00 ms | **met**: Seal takes 80% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 431.86 µs | 904.72 µs | **met**: Seal takes 48% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 93.52 ms | 167.95 ms | **met**: Seal takes 56% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 32.12 ms | 94.34 ms | **met**: Seal takes 34% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 64.55 ms | 182.00 ms | **met**: Seal takes 35% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.54 ms | 52.33 ms | **met**: Seal takes 51% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 352.73 µs | 512.16 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 21 ns | host call 42 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 88.05 ms | 347.83 µs | 87.72 ms | 104.74 ms | 2.8 MB | 395 B | 715.96 µs cold, 542 ns warm; 2.62 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 198.53 ms | 1.66 ms | 196.42 ms | 195.80 ms | 4.8 MB | 339 B | 5.29 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.70 ms | 1.15 ms | 88.58 ms | 89.47 ms | 1.9 MB | 339 B | 2.77 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.81 ms | 243.26 µs | 44.38 ms | 46.83 ms | 2.3 MB | 339 B | 2.54 ms as a whole process |  |
| Node `--jitless` | Purpose | 321.05 ms | 3.37 ms | 317.14 ms | 325.87 ms | 43.3 MB | 362 B | 19.43 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.45 ms | 1.22 ms | 121.20 ms | 128.05 ms | 49.6 MB | 362 B | 19.37 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 120.17 ms | 207.71 µs | 119.91 ms | 125.42 ms | 49.3 MB | 362 B | 18.90 ms as a whole process |  |
| Seal | Normal | 32.12 ms | 362.37 µs | 31.65 ms | 32.03 ms | 2.7 MB | 169 B | 80.33 µs cold, 209 ns warm; 2.71 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled |
| Luau (interpreter) | Normal | 71.76 ms | 382.53 µs | 71.04 ms | 71.56 ms | 4.9 MB | 143 B | 5.29 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.95 ms | 97.63 µs | 33.81 ms | 33.72 ms | 1.9 MB | 143 B | 2.36 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.35 ms | 23.79 µs | 7.33 ms | 7.51 ms | 2.2 MB | 143 B | 2.38 ms as a whole process |  |
| Node `--jitless` | Normal | 94.34 ms | 505.32 µs | 93.63 ms | 94.55 ms | 43.3 MB | 137 B | 19.09 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.74 ms | 87.09 µs | 9.62 ms | 10.22 ms | 48.7 MB | 137 B | 19.03 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.70 ms | 463.56 µs | 9.50 ms | 10.32 ms | 48.7 MB | 137 B | 18.61 ms as a whole process |  |
| Seal | Ceil.source | 33.09 ms | 717.63 µs | 31.17 ms | 31.46 ms | 2.7 MB | 169 B | 84.38 µs cold, 292 ns warm; 2.69 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 33.68 ms | 179.26 µs | 33.38 ms | 48.64 ms | 2.7 MB | 169 B | 643.50 µs cold, 500 ns warm; 2.84 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.program | 28.23 ms | 239.69 µs | 27.93 ms | 29.03 ms | 2.5 MB | 169 B | 353.21 µs cold, 166 ns warm; 2.78 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 28.23 ms | 383.54 µs | 27.62 ms | 49.44 ms | 2.6 MB | 169 B | 713.46 µs cold, 416 ns warm; 2.64 ms as a whole process | 2.0 KB counted by Seal; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 72.26 ms | 562.73 µs | 71.55 ms | 70.84 ms | 4.8 MB | 143 B | 5.92 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 34.40 ms | 281.75 µs | 34.19 ms | 34.41 ms | 1.9 MB | 143 B | 2.55 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.37 ms | 29.95 µs | 7.35 ms | 7.53 ms | 2.2 MB | 143 B | 2.58 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.22 ms | 539.96 µs | 95.38 ms | 94.60 ms | 43.3 MB | 137 B | 19.18 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.65 ms | 203.81 µs | 9.48 ms | 9.84 ms | 48.7 MB | 137 B | 20.23 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.56 ms | 61.53 µs | 9.45 ms | 10.42 ms | 48.6 MB | 137 B | 18.85 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 47.44 ms | 340.19 µs | 46.84 ms | 49.98 ms | 2.7 MB | 550 B | 115.33 µs cold, 250 ns warm; 2.54 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.63 ms | 459.42 µs | 91.11 ms | 91.74 ms | 4.8 MB | 433 B | 5.34 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.91 ms | 430.21 µs | 76.27 ms | 76.24 ms | 1.9 MB | 433 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.73 ms | 73.13 µs | 6.54 ms | 6.74 ms | 2.1 MB | 433 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.36 ms | 738.17 µs | 241.80 ms | 243.11 ms | 46.4 MB | 460 B | 18.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.78 ms | 282.37 µs | 18.18 ms | 19.14 ms | 50.5 MB | 460 B | 19.06 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.56 ms | 285.67 µs | 18.20 ms | 18.89 ms | 50.4 MB | 460 B | 19.04 ms as a whole process |  |
| Seal | Normal | 64.55 ms | 711.75 µs | 63.29 ms | 71.19 ms | 3.3 MB | 3.5 KB | 300.92 µs cold, 292 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.86 ms | 302.85 µs | 69.51 ms | 69.75 ms | 4.8 MB | 3.0 KB | 5.49 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 73.06 ms | 378.80 µs | 72.60 ms | 71.94 ms | 1.9 MB | 3.0 KB | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.28 ms | 34.44 µs | 5.24 ms | 5.55 ms | 2.2 MB | 3.0 KB | 2.75 ms as a whole process |  |
| Node `--jitless` | Normal | 182.00 ms | 5.18 ms | 174.92 ms | 188.07 ms | 48.2 MB | 3.1 KB | 18.84 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.92 ms | 47.67 µs | 3.86 ms | 6.09 ms | 52.0 MB | 3.1 KB | 19.24 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.91 ms | 33.80 µs | 3.86 ms | 5.83 ms | 52.1 MB | 3.1 KB | 19.07 ms as a whole process |  |
| Seal | Ceil.source | 53.80 ms | 627.26 µs | 52.79 ms | 55.10 ms | 3.2 MB | 3.4 KB | 258.33 µs cold, 417 ns warm; 2.93 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 57.30 ms | 630.06 µs | 56.13 ms | 58.70 ms | 3.3 MB | 3.4 KB | 275.42 µs cold, 292 ns warm; 2.86 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.11 ms | 469.09 µs | 49.66 ms | 50.26 ms | 2.9 MB | 3.4 KB | 267.58 µs cold, 250 ns warm; 2.91 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.48 ms | 510.80 µs | 49.55 ms | 50.48 ms | 3.0 MB | 3.4 KB | 306.92 µs cold, 291 ns warm; 2.87 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.80 ms | 376.11 µs | 69.33 ms | 70.32 ms | 4.8 MB | 3.0 KB | 5.39 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.94 ms | 571.54 µs | 71.13 ms | 70.83 ms | 1.9 MB | 3.0 KB | 2.63 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.24 ms | 30.59 µs | 5.20 ms | 5.43 ms | 2.2 MB | 3.0 KB | 2.61 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.79 ms | 3.21 ms | 172.72 ms | 189.84 ms | 48.3 MB | 3.1 KB | 18.86 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 24.88 µs | 3.85 ms | 5.79 ms | 52.0 MB | 3.1 KB | 19.29 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.88 ms | 24.58 µs | 3.85 ms | 5.71 ms | 52.2 MB | 3.1 KB | 19.36 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.34 ms | 12.93 ms | 50.68 ms | 55.29 ms | 2.9 MB | 429 B | 89.38 µs cold, 291 ns warm; 2.80 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.75 ms | 308.87 µs | 45.25 ms | 45.36 ms | 4.8 MB | 366 B | 5.52 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.53 ms | 313.23 µs | 76.94 ms | 77.10 ms | 2.0 MB | 366 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.40 ms | 258.48 µs | 22.02 ms | 22.58 ms | 2.2 MB | 366 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Purpose | 371.96 ms | 6.69 ms | 365.50 ms | 383.56 ms | 48.7 MB | 398 B | 19.63 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.33 ms | 36.88 ms | 15.65 ms | 20.95 ms | 53.0 MB | 398 B | 18.77 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.32 ms | 24.02 µs | 14.30 ms | 20.80 ms | 55.6 MB | 398 B | 18.98 ms as a whole process |  |
| Seal | Normal | 18.56 ms | 185.08 µs | 18.34 ms | 18.89 ms | 3.0 MB | 1.2 KB | 145.62 µs cold, 250 ns warm; 2.56 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.20 ms | 387.47 µs | 26.65 ms | 33.80 ms | 4.8 MB | 981 B | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.91 ms | 277.44 µs | 45.49 ms | 45.64 ms | 2.0 MB | 981 B | 2.38 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.53 ms | 10.70 µs | 1.52 ms | 1.72 ms | 2.3 MB | 981 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Normal | 63.27 ms | 896.21 µs | 61.66 ms | 63.65 ms | 48.4 MB | 1.0 KB | 19.42 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.64 ms | 184.26 µs | 1.52 ms | 3.56 ms | 53.0 MB | 1.0 KB | 18.94 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.65 ms | 14.27 µs | 1.62 ms | 3.22 ms | 52.8 MB | 1.0 KB | 19.55 ms as a whole process |  |
| Seal | Ceil.source | 18.61 ms | 296.12 µs | 18.26 ms | 18.55 ms | 3.0 MB | 1.2 KB | 150.62 µs cold, 292 ns warm; 2.72 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.20 ms | 194.91 µs | 18.88 ms | 19.33 ms | 3.1 MB | 1.2 KB | 146.17 µs cold, 250 ns warm; 2.67 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.83 ms | 71.01 µs | 18.74 ms | 18.83 ms | 2.9 MB | 1.2 KB | 121.46 µs cold, 208 ns warm; 2.73 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.87 ms | 71.49 µs | 18.75 ms | 18.96 ms | 2.9 MB | 1.2 KB | 122.92 µs cold, 167 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.15 ms | 405.10 µs | 26.62 ms | 27.03 ms | 4.9 MB | 981 B | 5.24 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 46.91 ms | 350.54 µs | 46.40 ms | 48.54 ms | 2.0 MB | 981 B | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.53 ms | 14.57 µs | 1.51 ms | 1.71 ms | 2.3 MB | 981 B | 2.36 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.40 ms | 950.15 µs | 61.54 ms | 62.74 ms | 48.5 MB | 1.0 KB | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.61 ms | 164.20 µs | 1.49 ms | 3.24 ms | 52.9 MB | 1.0 KB | 19.57 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.68 ms | 17.98 µs | 1.65 ms | 3.23 ms | 53.0 MB | 1.0 KB | 19.32 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 85.61 ms | 456.72 µs | 84.94 ms | 84.76 ms | 2.8 MB | 632 B | 97.83 µs cold, 250 ns warm; 2.73 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 89.79 ms | 1.82 ms | 87.30 ms | 87.86 ms | 4.8 MB | 523 B | 5.25 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.27 ms | 551.63 µs | 52.80 ms | 52.09 ms | 1.9 MB | 523 B | 2.36 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.19 ms | 72.82 µs | 6.12 ms | 7.07 ms | 2.2 MB | 523 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.45 ms | 1.17 ms | 146.75 ms | 146.71 ms | 43.4 MB | 568 B | 18.93 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 44.91 µs | 3.62 ms | 4.84 ms | 49.4 MB | 568 B | 19.36 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.71 ms | 39.60 µs | 3.64 ms | 4.47 ms | 49.5 MB | 568 B | 20.57 ms as a whole process |  |
| Seal | Normal | 101.53 ms | 311.80 µs | 101.03 ms | 104.06 ms | 3.0 MB | 1.8 KB | 155.46 µs cold, 292 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 214.58 ms | 2.26 ms | 208.34 ms | 217.57 ms | 4.8 MB | 1.7 KB | 5.23 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.09 ms | 326.24 µs | 146.68 ms | 145.55 ms | 2.0 MB | 1.7 KB | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 49.04 ms | 342.63 µs | 48.59 ms | 49.47 ms | 2.3 MB | 1.7 KB | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 152.77 ms | 2.09 ms | 150.01 ms | 156.98 ms | 45.5 MB | 1.7 KB | 18.96 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.72 ms | 73.32 µs | 14.59 ms | 16.24 ms | 52.1 MB | 1.7 KB | 19.15 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 15.09 ms | 92.24 µs | 14.94 ms | 17.13 ms | 52.3 MB | 1.7 KB | 19.17 ms as a whole process |  |
| Seal | Ceil.source | 101.57 ms | 905.14 µs | 101.00 ms | 103.26 ms | 3.0 MB | 1.8 KB | 141.54 µs cold, 209 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 106.04 ms | 308.45 µs | 105.57 ms | 105.73 ms | 3.0 MB | 1.8 KB | 150.12 µs cold, 250 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 103.53 ms | 302.79 µs | 103.15 ms | 102.93 ms | 2.8 MB | 1.8 KB | 128.25 µs cold, 208 ns warm; 2.64 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 104.35 ms | 493.62 µs | 103.36 ms | 104.59 ms | 2.9 MB | 1.8 KB | 126.33 µs cold, 208 ns warm; 2.88 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 213.31 ms | 1.96 ms | 208.42 ms | 214.55 ms | 4.9 MB | 1.7 KB | 5.32 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 149.94 ms | 3.04 ms | 146.78 ms | 144.53 ms | 2.0 MB | 1.7 KB | 2.88 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.90 ms | 384.68 µs | 48.09 ms | 55.83 ms | 2.3 MB | 1.7 KB | 2.52 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.06 ms | 829.55 µs | 155.51 ms | 162.57 ms | 45.5 MB | 1.7 KB | 18.58 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.28 ms | 183.70 µs | 14.03 ms | 15.92 ms | 52.1 MB | 1.7 KB | 19.16 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.67 ms | 96.90 µs | 14.47 ms | 16.40 ms | 52.2 MB | 1.7 KB | 19.86 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 129.01 ms | 1.50 ms | 126.79 ms | 129.11 ms | 4.1 MB | 569 B | 112.08 µs cold, 417 ns warm; 2.66 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.21 ms | 382.34 µs | 61.82 ms | 62.54 ms | 4.8 MB | 540 B | 5.20 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 28.54 ms | 102.63 µs | 28.39 ms | 28.92 ms | 2.2 MB | 540 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.62 ms | 24.65 µs | 4.59 ms | 4.71 ms | 2.3 MB | 540 B | 2.38 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.04 ms | 1.19 ms | 241.22 ms | 244.01 ms | 48.6 MB | 514 B | 18.88 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.34 ms | 24.75 ms | 63.27 ms | 90.57 ms | 52.8 MB | 514 B | 18.97 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.11 ms | 1.51 ms | 63.71 ms | 91.14 ms | 52.9 MB | 514 B | 19.98 ms as a whole process |  |
| Seal | Normal | 26.54 ms | 96.38 µs | 26.40 ms | 26.63 ms | 4.0 MB | 538 B | 115.21 µs cold, 334 ns warm; 2.81 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.39 ms | 459.32 µs | 29.95 ms | 30.59 ms | 6.0 MB | 478 B | 5.33 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 32.04 ms | 1.39 ms | 29.59 ms | 29.76 ms | 3.1 MB | 478 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.63 ms | 692.92 µs | 11.60 ms | 12.64 ms | 3.4 MB | 478 B | 2.57 ms as a whole process |  |
| Node `--jitless` | Normal | 52.33 ms | 852.27 µs | 51.40 ms | 53.49 ms | 56.6 MB | 477 B | 21.39 ms as a whole process |  |
| Node (JIT), from cold | Normal | 36.53 ms | 371.10 µs | 36.04 ms | 37.34 ms | 62.6 MB | 477 B | 19.82 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 36.04 ms | 249.24 µs | 35.72 ms | 35.84 ms | 88.1 MB | 477 B | 19.84 ms as a whole process |  |
| Seal | Ceil.source | 26.79 ms | 66.98 µs | 26.68 ms | 26.76 ms | 4.0 MB | 538 B | 105.92 µs cold, 292 ns warm; 2.75 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.56 ms | 112.37 µs | 27.39 ms | 28.51 ms | 4.0 MB | 538 B | 121.79 µs cold, 375 ns warm; 2.71 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.26 ms | 238.96 µs | 24.11 ms | 24.47 ms | 4.0 MB | 538 B | 93.96 µs cold, 292 ns warm; 2.54 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.67 ms | 192.18 µs | 23.37 ms | 23.77 ms | 4.0 MB | 538 B | 92.25 µs cold, 250 ns warm; 2.80 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.19 ms | 193.75 µs | 29.74 ms | 30.31 ms | 6.0 MB | 478 B | 5.35 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.48 ms | 340.96 µs | 30.95 ms | 30.59 ms | 3.1 MB | 478 B | 2.52 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.60 ms | 701.89 µs | 11.79 ms | 12.28 ms | 3.4 MB | 478 B | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.65 ms | 626.50 µs | 51.06 ms | 52.71 ms | 56.5 MB | 477 B | 19.35 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.63 ms | 283.20 µs | 35.15 ms | 35.80 ms | 62.6 MB | 477 B | 19.22 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.70 ms | 255.49 µs | 35.35 ms | 36.01 ms | 88.0 MB | 477 B | 18.99 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 316.55 µs | 4.30 µs | 312.67 µs | 344.83 µs | 3.1 MB | 233 B | 72.67 µs cold, 417 ns warm; 2.56 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 143.57 ms | 743.74 µs | 142.54 ms | 144.20 ms | 10.5 MB | 195 B | 5.26 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.27 ms | 288.74 µs | 32.12 ms | 31.59 ms | 3.6 MB | 195 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.71 ms | 86.71 µs | 20.59 ms | 20.93 ms | 3.8 MB | 195 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Purpose | 375.09 µs | 140.03 µs | 242.83 µs | 1.05 ms | 52.6 MB | 223 B | 18.54 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 220.74 µs | 154.56 µs | 87.33 µs | 914.21 µs | 57.7 MB | 223 B | 19.06 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 174.91 µs | 170.97 µs | 54.38 µs | 881.54 µs | 58.7 MB | 223 B | 18.63 ms as a whole process |  |
| Seal | Normal | 352.73 µs | 6.19 µs | 344.08 µs | 367.92 µs | 3.3 MB | 182 B | 77.83 µs cold, 417 ns warm; 2.62 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.32 ms | 1.44 ms | 158.80 ms | 161.34 ms | 12.5 MB | 138 B | 5.23 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 42.25 ms | 90.35 µs | 42.08 ms | 41.59 ms | 4.9 MB | 138 B | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.48 ms | 238.71 µs | 19.22 ms | 20.14 ms | 5.2 MB | 138 B | 2.61 ms as a whole process |  |
| Node `--jitless` | Normal | 512.16 µs | 353.66 µs | 270.92 µs | 1.19 ms | 61.5 MB | 155 B | 18.90 ms as a whole process |  |
| Node (JIT), from cold | Normal | 510.31 µs | 479.32 µs | 131.83 µs | 1.05 ms | 65.4 MB | 155 B | 18.77 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 274.94 µs | 326.12 µs | 122.67 µs | 1.03 ms | 69.5 MB | 155 B | 19.14 ms as a whole process |  |
| Seal | Ceil.source | 348.21 µs | 4.36 µs | 343.88 µs | 361.88 µs | 3.3 MB | 182 B | 76.50 µs cold, 333 ns warm; 2.52 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 357.04 µs | 4.20 µs | 350.79 µs | 370.25 µs | 3.3 MB | 182 B | 73.88 µs cold, 333 ns warm; 2.49 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 294.10 µs | 4.98 µs | 288.38 µs | 325.71 µs | 3.1 MB | 182 B | 60.00 µs cold, 208 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 292.84 µs | 6.29 µs | 285.08 µs | 315.04 µs | 3.1 MB | 182 B | 62.83 µs cold, 292 ns warm; 2.49 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.41 ms | 939.53 µs | 159.59 ms | 163.73 ms | 12.1 MB | 138 B | 5.26 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 42.18 ms | 333.95 µs | 41.89 ms | 41.18 ms | 5.1 MB | 138 B | 2.62 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.62 ms | 312.82 µs | 19.17 ms | 19.67 ms | 5.2 MB | 138 B | 2.63 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 516.54 µs | 349.94 µs | 275.62 µs | 1.53 ms | 61.4 MB | 155 B | 19.85 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 508.28 µs | 471.35 µs | 147.67 µs | 1.11 ms | 65.3 MB | 155 B | 19.19 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 273.51 µs | 324.89 µs | 122.87 µs | 1.06 ms | 69.4 MB | 155 B | 19.33 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 72.91 ms | 664.47 µs | 72.04 ms | 72.20 ms | 3.0 MB | 537 B | 92.04 µs cold, 208 ns warm; 2.71 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 94.32 ms | 309.63 µs | 93.78 ms | 95.39 ms | 4.9 MB | 411 B | 5.25 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 121.09 ms | 2.99 ms | 116.15 ms | 119.57 ms | 2.6 MB | 411 B | 2.43 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.43 ms | 637.82 µs | 93.65 ms | 97.61 ms | 2.9 MB | 411 B | 2.55 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.72 ms | 990.19 µs | 73.99 ms | 77.98 ms | 59.7 MB | 444 B | 18.76 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.52 ms | 278.23 µs | 10.21 ms | 13.11 ms | 66.0 MB | 444 B | 18.94 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.50 ms | 777.33 µs | 10.07 ms | 13.03 ms | 114.8 MB | 444 B | 19.26 ms as a whole process |  |
| Seal | Normal | 99.40 ms | 551.80 µs | 98.26 ms | 99.27 ms | 7.8 MB | 992 B | 124.50 µs cold, 208 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 134.65 ms | 1.51 ms | 132.35 ms | 132.99 ms | 43.3 MB | 799 B | 5.24 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 129.50 ms | 1.22 ms | 127.45 ms | 128.99 ms | 41.6 MB | 799 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.43 ms | 1.05 ms | 79.64 ms | 81.53 ms | 39.8 MB | 799 B | 2.59 ms as a whole process |  |
| Node `--jitless` | Normal | 114.90 ms | 2.26 ms | 113.19 ms | 118.79 ms | 213.3 MB | 878 B | 18.64 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.35 ms | 856.22 µs | 13.49 ms | 17.22 ms | 129.6 MB | 878 B | 19.03 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 16.20 ms | 1.37 ms | 14.62 ms | 17.07 ms | 224.3 MB | 878 B | 18.91 ms as a whole process |  |
| Seal | Ceil.source | 98.35 ms | 1.11 ms | 97.09 ms | 101.34 ms | 7.9 MB | 992 B | 126.42 µs cold, 250 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 101.97 ms | 819.86 µs | 100.03 ms | 104.23 ms | 7.8 MB | 992 B | 124.00 µs cold, 250 ns warm; 2.75 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 87.17 ms | 742.20 µs | 85.86 ms | 88.28 ms | 7.6 MB | 992 B | 113.29 µs cold, 209 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 85.35 ms | 793.51 µs | 84.46 ms | 86.20 ms | 7.7 MB | 992 B | 100.42 µs cold, 209 ns warm; 2.75 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 128.95 ms | 2.03 ms | 125.42 ms | 128.13 ms | 42.3 MB | 799 B | 5.40 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.58 ms | 1.21 ms | 127.10 ms | 127.67 ms | 42.7 MB | 799 B | 2.76 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 83.56 ms | 1.34 ms | 82.20 ms | 84.32 ms | 43.9 MB | 799 B | 2.48 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 129.18 ms | 25.04 ms | 114.88 ms | 117.74 ms | 213.2 MB | 878 B | 20.21 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.81 ms | 895.65 µs | 13.58 ms | 17.57 ms | 129.8 MB | 878 B | 21.21 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.19 ms | 414.09 µs | 13.65 ms | 17.62 ms | 224.5 MB | 878 B | 19.18 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 73.33 µs (±30.42 µs, least 50.38 µs) | 22.20 µs (±5.30 µs, least 14.67 µs) | 20 ns (±1 ns, least 19 ns) | 41 ns (±2 ns, least 39 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.99 µs (±1.01 µs, least 8.79 µs) | 16.42 µs (±873 ns, least 14.83 µs) | 40 ns (±2 ns, least 38 ns) | 20 ns (±1 ns, least 18 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 4.30 µs (±1.06 µs, least 3.00 µs) | 6.60 µs (±1.90 µs, least 4.00 µs) | 22 ns (±1 ns, least 20 ns) | 15 ns (±2 ns, least 11 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 4.30 µs (±1.34 µs, least 3.00 µs) | 5.90 µs (±1.10 µs, least 4.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 35.30 µs (±10.60 µs, least 21.79 µs) | 72.46 µs (±85.86 µs, least 38.17 µs) | 58 ns (±5 ns, least 54 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 46.0 MB | the engine's own heap |
| Node (JIT) | Purpose | 36.63 µs (±7.79 µs, least 20.67 µs) | 43.97 µs (±4.32 µs, least 39.08 µs) | 9 ns (±0 ns, least 9 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.2 MB | the engine's own heap |
| Seal | Normal | 4.76 µs (±152 ns, least 4.54 µs) | 168 ns (±12 ns, least 155 ns) | 21 ns (±1 ns, least 20 ns) | 42 ns (±3 ns, least 40 ns) | 512 B | 4.6 MB | counted by Seal |
| Luau (interpreter) | Normal | 5.75 µs (±1.61 µs, least 4.80 µs) | 213 ns (±23 ns, least 197 ns) | 36 ns (±3 ns, least 33 ns) | 18 ns (±1 ns, least 18 ns) | 2.0 KB | 6.2 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 2.20 µs (±377 ns, least 1.79 µs) | 218 ns (±78 ns, least 181 ns) | 35 ns (±7 ns, least 27 ns) | 15 ns (±1 ns, least 13 ns) | 1.0 KB | 3.4 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 2.09 µs (±252 ns, least 1.81 µs) | 424 ns (±85 ns, least 379 ns) | 121 ns (±17 ns, least 102 ns) | 126 ns (±16 ns, least 109 ns) | 1.0 KB | 4.9 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.28 µs (±126 ns, least 1.15 µs) | 849 ns (±194 ns, least 691 ns) | 54 ns (±3 ns, least 50 ns) | 26 ns (±3 ns, least 24 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.18 µs (±137 ns, least 1.06 µs) | 866 ns (±271 ns, least 697 ns) | 25 ns (±2 ns, least 22 ns) | 8 ns (±2 ns, least 7 ns) | 643 B | 56.0 MB | the engine's own heap |
| Seal | Ceil.source | 5.06 µs (±299 ns, least 4.65 µs) | 184 ns (±31 ns, least 158 ns) | 26 ns (±10 ns, least 20 ns) | 44 ns (±3 ns, least 41 ns) | 512 B | 4.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.86 µs (±307 ns, least 4.53 µs) | 184 ns (±17 ns, least 167 ns) | 22 ns (±2 ns, least 21 ns) | 42 ns (±2 ns, least 40 ns) | 512 B | 4.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.46 µs (±358 ns, least 4.15 µs) | 173 ns (±38 ns, least 153 ns) | 20 ns (±2 ns, least 18 ns) | 40 ns (±2 ns, least 38 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.38 µs (±412 ns, least 4.07 µs) | 163 ns (±14 ns, least 148 ns) | 19 ns (±1 ns, least 18 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.00 µs (±168 ns, least 4.78 µs) | 193 ns (±6 ns, least 183 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.83 µs (±42 ns, least 1.77 µs) | 207 ns (±53 ns, least 174 ns) | 28 ns (±0 ns, least 27 ns) | 17 ns (±4 ns, least 13 ns) | 1.0 KB | 3.4 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.87 µs (±94 ns, least 1.78 µs) | 396 ns (±68 ns, least 348 ns) | 106 ns (±5 ns, least 99 ns) | 109 ns (±7 ns, least 104 ns) | 1.0 KB | 4.7 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.17 µs (±118 ns, least 1.10 µs) | 697 ns (±47 ns, least 656 ns) | 51 ns (±2 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.09 µs (±41 ns, least 1.01 µs) | 694 ns (±44 ns, least 635 ns) | 23 ns (±2 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 33.02 ms | 32.09 ms (-2.8%) |
| n-body, 100,000 steps | 64.11 ms | 60.41 ms (-5.8%) |
| spectral-norm, size 200 | 19.39 ms | 19.79 ms (+2.1%) |
| fannkuch-redux, 9 | 103.66 ms | 104.27 ms (+0.6%) |
| word count, 500,000 words | 26.85 ms | 27.22 ms (+1.4%) |
| text building, 20,000 joins | 349.86 µs | 352.49 µs (+0.8%) |
| binary-trees, depth 14 | 100.00 ms | 97.84 ms (-2.2%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Seal mean | Seal deviation | Seal least | Seal peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.36 ms | 9.08 µs | 1.35 ms | 3.6 MB | 1.57 ms | 1.56 ms | 307.87 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.35 ms | 5.32 µs | 1.34 ms | 3.2 MB | 1.53 ms | 1.51 ms | 1.10 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 111.48 ms | 502.85 µs | 110.73 ms | 2.9 MB | 163.40 ms | 163.00 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 68.08 ms | 349.41 µs | 67.77 ms | 2.9 MB | 85.00 ms | 84.53 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 431.86 µs | 11.95 µs | 413.04 µs | 3.4 MB | 904.72 µs | 878.46 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 93.52 ms | 1.70 ms | 89.85 ms | 2.9 MB | 167.95 ms | 162.43 ms | – |
| the money sums in `Float`s, as a yardstick | 10.50 ms | 98.48 µs | 10.33 ms | 2.8 MB | – | – | – |
