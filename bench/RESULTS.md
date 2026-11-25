# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.07 2.05 2.01 }` before, `{ 2.01 2.07 2.02 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 49.85 ms | 79.92 ms | **met**: Seal takes 62% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.08 ms | 69.74 ms | **met**: Seal takes 93% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.66 ms | 27.42 ms | **met**: Seal takes 72% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 112.21 ms | 216.44 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 23.98 ms | 30.63 ms | **met**: Seal takes 78% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 339.51 µs | 160.46 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 111.02 ms | 130.36 ms | **met**: Seal takes 85% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.85 ms | 3.56 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 34 ns | **met**: Seal takes 50% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 147 ns | 185 ns | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.84 µs | 4.91 µs | **met**: Seal takes 98% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 49.85 ms | 34.05 ms | **not met**: Seal takes 146% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 65.08 ms | 71.86 ms | **met**: Seal takes 91% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 19.66 ms | 44.60 ms | **met**: Seal takes 44% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 112.21 ms | 144.64 ms | **met**: Seal takes 78% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 23.98 ms | 31.79 ms | **met**: Seal takes 75% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 339.51 µs | 42.59 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 111.02 ms | 128.74 ms | **met**: Seal takes 86% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.85 ms | 2.95 ms | **met**: Seal takes 63% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 17 ns | 28 ns | **met**: Seal takes 61% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 147 ns | 182 ns | **met**: Seal takes 81% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.84 µs | 1.78 µs | **not met**: Seal takes 271% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 49.85 ms | 95.75 ms | **met**: Seal takes 52% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.08 ms | 177.20 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 23.98 ms | 52.24 ms | **met**: Seal takes 46% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 339.51 µs | 528.85 µs | **met**: Seal takes 64% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 133.17 ms | 354.38 µs | 132.46 ms | 151.80 ms | 2.8 MB | 395 B | 709.21 µs cold, 542 ns warm; 2.68 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 196.90 ms | 321.12 µs | 196.24 ms | 197.74 ms | 4.8 MB | 339 B | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 94.46 ms | 10.72 ms | 88.71 ms | 88.46 ms | 1.9 MB | 339 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.84 ms | 309.93 µs | 44.47 ms | 46.15 ms | 2.2 MB | 339 B | 2.56 ms as a whole process |  |
| Node `--jitless` | Purpose | 330.17 ms | 8.59 ms | 323.44 ms | 324.94 ms | 43.3 MB | 362 B | 19.49 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.79 ms | 403.09 µs | 122.34 ms | 122.62 ms | 49.2 MB | 362 B | 18.36 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 120.76 ms | 637.31 µs | 120.18 ms | 121.23 ms | 49.2 MB | 362 B | 18.77 ms as a whole process |  |
| Seal | Normal | 49.85 ms | 83.46 µs | 49.77 ms | 49.93 ms | 2.7 MB | 169 B | 70.62 µs cold, 250 ns warm; 2.66 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 79.92 ms | 5.19 ms | 72.31 ms | 73.22 ms | 4.8 MB | 143 B | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 34.05 ms | 52.99 µs | 33.95 ms | 33.79 ms | 1.9 MB | 143 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.20 ms | 21.19 µs | 7.17 ms | 7.39 ms | 2.1 MB | 143 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Normal | 95.75 ms | 705.17 µs | 94.85 ms | 95.61 ms | 43.2 MB | 137 B | 18.50 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.91 ms | 502.14 µs | 9.58 ms | 10.46 ms | 48.7 MB | 137 B | 18.48 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.55 ms | 166.65 µs | 9.45 ms | 10.09 ms | 48.5 MB | 137 B | 19.07 ms as a whole process |  |
| Seal | Ceil.source | 49.16 ms | 112.72 µs | 48.97 ms | 49.16 ms | 2.7 MB | 169 B | 70.71 µs cold, 209 ns warm; 2.68 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 50.61 ms | 217.91 µs | 50.17 ms | 73.63 ms | 2.7 MB | 169 B | 971.42 µs cold, 667 ns warm; 2.57 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 44.86 ms | 372.68 µs | 44.29 ms | 68.66 ms | 2.4 MB | 169 B | 658.33 µs cold, 583 ns warm; 2.58 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 44.58 ms | 180.67 µs | 44.20 ms | 68.19 ms | 2.5 MB | 169 B | 856.12 µs cold, 542 ns warm; 2.70 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.01 ms | 278.07 µs | 70.55 ms | 72.24 ms | 4.8 MB | 143 B | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 34.09 ms | 157.64 µs | 33.81 ms | 33.98 ms | 1.9 MB | 143 B | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.19 ms | 27.93 µs | 7.14 ms | 7.33 ms | 2.1 MB | 143 B | 2.59 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 94.34 ms | 1.64 ms | 92.51 ms | 95.97 ms | 43.3 MB | 137 B | 19.26 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 10.36 ms | 748.09 µs | 9.67 ms | 10.77 ms | 48.5 MB | 137 B | 18.98 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.60 ms | 60.13 µs | 9.49 ms | 11.04 ms | 48.6 MB | 137 B | 19.22 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.00 ms | 350.96 µs | 52.57 ms | 53.56 ms | 2.7 MB | 550 B | 97.12 µs cold, 208 ns warm; 2.55 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.61 ms | 397.32 µs | 92.07 ms | 91.80 ms | 4.8 MB | 433 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.18 ms | 405.81 µs | 76.77 ms | 76.72 ms | 1.9 MB | 433 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.64 ms | 208.71 µs | 6.48 ms | 6.76 ms | 2.1 MB | 433 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.48 ms | 828.16 µs | 243.47 ms | 244.95 ms | 46.4 MB | 460 B | 19.02 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.27 ms | 143.28 µs | 18.12 ms | 18.47 ms | 50.4 MB | 460 B | 18.92 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.26 ms | 55.91 µs | 18.18 ms | 18.21 ms | 50.2 MB | 460 B | 18.64 ms as a whole process |  |
| Seal | Normal | 65.08 ms | 445.40 µs | 64.44 ms | 66.11 ms | 3.3 MB | 3.5 KB | 308.62 µs cold, 250 ns warm; 2.97 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.74 ms | 156.02 µs | 69.53 ms | 70.07 ms | 4.8 MB | 3.0 KB | 5.09 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.86 ms | 294.49 µs | 71.44 ms | 71.48 ms | 1.9 MB | 3.0 KB | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.24 ms | 26.78 µs | 5.21 ms | 5.41 ms | 2.2 MB | 3.0 KB | 2.50 ms as a whole process |  |
| Node `--jitless` | Normal | 177.20 ms | 3.05 ms | 173.80 ms | 183.32 ms | 48.2 MB | 3.1 KB | 18.61 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.88 ms | 23.01 µs | 3.84 ms | 5.66 ms | 51.9 MB | 3.1 KB | 18.28 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 15.13 µs | 3.84 ms | 5.71 ms | 52.0 MB | 3.1 KB | 18.89 ms as a whole process |  |
| Seal | Ceil.source | 56.47 ms | 533.75 µs | 55.27 ms | 58.56 ms | 3.2 MB | 3.4 KB | 264.21 µs cold, 333 ns warm; 2.69 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 56.57 ms | 347.11 µs | 56.08 ms | 56.49 ms | 3.2 MB | 3.4 KB | 274.12 µs cold, 292 ns warm; 2.97 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.07 ms | 220.14 µs | 49.59 ms | 50.95 ms | 2.9 MB | 3.4 KB | 270.75 µs cold, 250 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.24 ms | 228.85 µs | 49.64 ms | 50.34 ms | 2.9 MB | 3.4 KB | 255.50 µs cold, 250 ns warm; 2.73 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 73.87 ms | 6.79 ms | 69.56 ms | 69.94 ms | 4.8 MB | 3.0 KB | 5.15 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.70 ms | 209.78 µs | 71.37 ms | 71.38 ms | 1.9 MB | 3.0 KB | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.26 ms | 27.87 µs | 5.21 ms | 5.44 ms | 2.2 MB | 3.0 KB | 2.56 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 175.43 ms | 3.66 ms | 171.48 ms | 181.46 ms | 48.0 MB | 3.1 KB | 18.80 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 29.47 µs | 3.83 ms | 5.74 ms | 51.9 MB | 3.1 KB | 18.68 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.89 ms | 18.95 µs | 3.86 ms | 5.73 ms | 52.0 MB | 3.1 KB | 18.72 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.93 ms | 550.53 µs | 52.15 ms | 52.59 ms | 2.9 MB | 429 B | 85.17 µs cold, 208 ns warm; 2.73 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.65 ms | 278.75 µs | 45.18 ms | 45.48 ms | 4.8 MB | 366 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.24 ms | 84.99 µs | 77.12 ms | 77.23 ms | 1.9 MB | 366 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.80 ms | 514.66 µs | 22.10 ms | 22.94 ms | 2.1 MB | 366 B | 2.60 ms as a whole process |  |
| Node `--jitless` | Purpose | 367.95 ms | 6.14 ms | 363.11 ms | 376.16 ms | 48.7 MB | 398 B | 18.86 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.42 ms | 37.48 ms | 15.65 ms | 19.54 ms | 52.7 MB | 398 B | 18.42 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.37 ms | 28.24 µs | 14.33 ms | 19.41 ms | 55.2 MB | 398 B | 19.09 ms as a whole process |  |
| Seal | Normal | 19.66 ms | 72.67 µs | 19.51 ms | 19.55 ms | 3.0 MB | 1.2 KB | 143.88 µs cold, 208 ns warm; 2.55 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.42 ms | 332.29 µs | 26.89 ms | 27.05 ms | 4.8 MB | 981 B | 5.15 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 44.60 ms | 760.65 µs | 43.67 ms | 46.45 ms | 2.0 MB | 981 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.55 ms | 18.66 µs | 1.53 ms | 1.72 ms | 2.2 MB | 981 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Normal | 63.14 ms | 293.86 µs | 62.77 ms | 63.47 ms | 48.4 MB | 1.0 KB | 19.15 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.60 ms | 203.36 µs | 1.46 ms | 3.64 ms | 52.8 MB | 1.0 KB | 18.69 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 37.14 µs | 1.62 ms | 3.31 ms | 52.8 MB | 1.0 KB | 18.97 ms as a whole process |  |
| Seal | Ceil.source | 19.72 ms | 73.82 µs | 19.59 ms | 19.64 ms | 3.0 MB | 1.2 KB | 143.17 µs cold, 250 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.69 ms | 130.58 µs | 19.59 ms | 19.99 ms | 3.0 MB | 1.2 KB | 144.38 µs cold, 208 ns warm; 2.62 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.44 ms | 162.81 µs | 19.15 ms | 19.20 ms | 2.8 MB | 1.2 KB | 133.71 µs cold, 208 ns warm; 2.60 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.79 ms | 51.71 µs | 18.75 ms | 18.74 ms | 2.8 MB | 1.2 KB | 116.25 µs cold, 167 ns warm; 2.63 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.63 ms | 249.34 µs | 27.16 ms | 28.93 ms | 4.8 MB | 981 B | 5.00 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 45.05 ms | 490.19 µs | 44.40 ms | 44.96 ms | 1.9 MB | 981 B | 2.34 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.53 ms | 10.22 µs | 1.52 ms | 1.67 ms | 2.2 MB | 981 B | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.19 ms | 273.30 µs | 61.78 ms | 62.51 ms | 48.5 MB | 1.0 KB | 19.02 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.58 ms | 164.68 µs | 1.42 ms | 3.39 ms | 52.8 MB | 1.0 KB | 18.60 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.65 ms | 31.60 µs | 1.58 ms | 3.34 ms | 53.0 MB | 1.0 KB | 18.27 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 96.97 ms | 608.53 µs | 96.03 ms | 97.38 ms | 2.8 MB | 632 B | 95.71 µs cold, 291 ns warm; 2.62 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.71 ms | 1.18 ms | 87.57 ms | 87.87 ms | 4.8 MB | 523 B | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.17 ms | 853.78 µs | 51.77 ms | 51.12 ms | 1.9 MB | 523 B | 2.43 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.18 ms | 93.08 µs | 6.12 ms | 6.19 ms | 2.1 MB | 523 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.42 ms | 838.76 µs | 145.28 ms | 147.14 ms | 43.4 MB | 568 B | 19.20 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.70 ms | 37.06 µs | 3.63 ms | 4.46 ms | 49.4 MB | 568 B | 18.27 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.71 ms | 42.39 µs | 3.63 ms | 4.48 ms | 49.4 MB | 568 B | 18.87 ms as a whole process |  |
| Seal | Normal | 112.21 ms | 186.55 µs | 111.94 ms | 113.03 ms | 3.0 MB | 1.8 KB | 151.04 µs cold, 208 ns warm; 2.68 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 216.44 ms | 12.09 ms | 208.50 ms | 213.24 ms | 4.8 MB | 1.7 KB | 5.34 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 144.64 ms | 270.03 µs | 144.14 ms | 154.93 ms | 1.9 MB | 1.7 KB | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.82 ms | 218.95 µs | 48.51 ms | 48.91 ms | 2.3 MB | 1.7 KB | 2.36 ms as a whole process |  |
| Node `--jitless` | Normal | 157.05 ms | 235.95 µs | 156.72 ms | 156.85 ms | 45.4 MB | 1.7 KB | 19.13 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.33 ms | 154.71 µs | 14.07 ms | 15.73 ms | 51.9 MB | 1.7 KB | 18.52 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.55 ms | 73.54 µs | 14.39 ms | 16.02 ms | 52.2 MB | 1.7 KB | 18.81 ms as a whole process |  |
| Seal | Ceil.source | 113.06 ms | 201.73 µs | 112.82 ms | 113.48 ms | 3.0 MB | 1.8 KB | 134.71 µs cold, 292 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 111.86 ms | 213.07 µs | 111.64 ms | 113.04 ms | 3.0 MB | 1.8 KB | 144.25 µs cold, 292 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 103.84 ms | 117.72 µs | 103.70 ms | 103.98 ms | 2.7 MB | 1.8 KB | 120.29 µs cold, 209 ns warm; 2.61 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.16 ms | 154.16 µs | 103.02 ms | 103.38 ms | 2.8 MB | 1.8 KB | 135.38 µs cold, 250 ns warm; 2.68 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 213.61 ms | 2.32 ms | 207.08 ms | 214.95 ms | 4.8 MB | 1.7 KB | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 148.36 ms | 500.86 µs | 147.71 ms | 145.69 ms | 1.9 MB | 1.7 KB | 2.52 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 49.54 ms | 219.36 µs | 49.25 ms | 49.28 ms | 2.3 MB | 1.7 KB | 2.51 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.25 ms | 578.79 µs | 156.55 ms | 163.77 ms | 45.5 MB | 1.7 KB | 19.22 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.08 ms | 72.82 µs | 13.97 ms | 15.43 ms | 52.0 MB | 1.7 KB | 18.56 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.20 ms | 72.26 µs | 14.05 ms | 15.82 ms | 52.2 MB | 1.7 KB | 19.25 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.42 ms | 920.59 µs | 94.47 ms | 93.73 ms | 4.0 MB | 569 B | 91.62 µs cold, 375 ns warm; 2.72 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.90 ms | 210.39 µs | 62.60 ms | 62.99 ms | 4.8 MB | 540 B | 5.14 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 30.24 ms | 165.84 µs | 30.05 ms | 29.07 ms | 2.1 MB | 540 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 5.00 ms | 26.06 µs | 4.97 ms | 4.91 ms | 2.3 MB | 540 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.76 ms | 5.11 ms | 240.03 ms | 243.49 ms | 48.8 MB | 514 B | 18.53 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.85 ms | 23.76 ms | 65.31 ms | 89.91 ms | 52.5 MB | 514 B | 19.08 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.83 ms | 580.69 µs | 67.07 ms | 88.23 ms | 52.7 MB | 514 B | 18.59 ms as a whole process |  |
| Seal | Normal | 23.98 ms | 124.78 µs | 23.82 ms | 24.08 ms | 4.0 MB | 538 B | 97.25 µs cold, 291 ns warm; 2.61 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.63 ms | 190.84 µs | 30.37 ms | 30.37 ms | 6.0 MB | 478 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.79 ms | 476.43 µs | 31.27 ms | 30.68 ms | 3.0 MB | 478 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.30 ms | 526.13 µs | 11.68 ms | 12.15 ms | 3.3 MB | 478 B | 2.41 ms as a whole process |  |
| Node `--jitless` | Normal | 52.24 ms | 330.23 µs | 51.89 ms | 53.35 ms | 56.4 MB | 477 B | 18.79 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.97 ms | 323.81 µs | 35.65 ms | 36.27 ms | 62.4 MB | 477 B | 18.78 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.82 ms | 444.54 µs | 35.20 ms | 36.48 ms | 88.5 MB | 477 B | 18.64 ms as a whole process |  |
| Seal | Ceil.source | 23.97 ms | 123.74 µs | 23.70 ms | 24.16 ms | 4.0 MB | 538 B | 94.17 µs cold, 375 ns warm; 2.58 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 23.91 ms | 110.97 µs | 23.66 ms | 23.88 ms | 4.0 MB | 538 B | 100.12 µs cold, 291 ns warm; 2.57 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.33 ms | 166.78 µs | 23.06 ms | 23.15 ms | 3.9 MB | 538 B | 84.67 µs cold, 250 ns warm; 2.62 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.55 ms | 272.91 µs | 24.08 ms | 24.52 ms | 3.9 MB | 538 B | 83.62 µs cold, 291 ns warm; 2.61 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 31.73 ms | 853.50 µs | 30.64 ms | 30.74 ms | 6.0 MB | 478 B | 5.10 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.40 ms | 677.77 µs | 29.77 ms | 31.53 ms | 3.0 MB | 478 B | 2.31 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.96 ms | 329.29 µs | 12.42 ms | 13.65 ms | 3.3 MB | 478 B | 2.46 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.88 ms | 230.16 µs | 51.53 ms | 53.45 ms | 56.3 MB | 477 B | 18.74 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.51 ms | 373.97 µs | 34.96 ms | 36.03 ms | 62.4 MB | 477 B | 18.42 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.97 ms | 351.94 µs | 35.60 ms | 36.84 ms | 87.9 MB | 477 B | 18.53 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 305.71 µs | 5.27 µs | 298.79 µs | 311.92 µs | 3.1 MB | 233 B | 66.29 µs cold, 291 ns warm; 2.51 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.42 ms | 865.39 µs | 142.25 ms | 144.96 ms | 10.0 MB | 195 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.37 ms | 185.81 µs | 32.13 ms | 31.44 ms | 3.5 MB | 195 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 21.49 ms | 255.60 µs | 21.02 ms | 21.16 ms | 3.7 MB | 195 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 363.95 µs | 143.33 µs | 236.92 µs | 1.03 ms | 52.5 MB | 223 B | 18.80 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 219.33 µs | 158.63 µs | 87.42 µs | 930.67 µs | 57.8 MB | 223 B | 18.18 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 139.39 µs | 120.93 µs | 62.88 µs | 905.46 µs | 57.9 MB | 223 B | 18.65 ms as a whole process |  |
| Seal | Normal | 339.51 µs | 3.62 µs | 333.92 µs | 367.50 µs | 3.2 MB | 182 B | 72.83 µs cold, 291 ns warm; 2.67 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.46 ms | 863.31 µs | 159.70 ms | 163.85 ms | 11.8 MB | 138 B | 4.98 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 42.59 ms | 493.87 µs | 41.50 ms | 41.88 ms | 4.9 MB | 138 B | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.54 ms | 178.95 µs | 19.28 ms | 20.23 ms | 5.1 MB | 138 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Normal | 528.85 µs | 337.67 µs | 305.12 µs | 1.09 ms | 61.4 MB | 155 B | 19.16 ms as a whole process |  |
| Node (JIT), from cold | Normal | 492.63 µs | 489.47 µs | 121.46 µs | 1.06 ms | 65.2 MB | 155 B | 19.08 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 272.57 µs | 315.51 µs | 125.54 µs | 1.04 ms | 69.2 MB | 155 B | 18.89 ms as a whole process |  |
| Seal | Ceil.source | 339.43 µs | 4.81 µs | 334.08 µs | 371.71 µs | 3.2 MB | 182 B | 75.54 µs cold, 292 ns warm; 2.61 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 341.77 µs | 4.01 µs | 335.04 µs | 393.17 µs | 3.2 MB | 182 B | 78.04 µs cold, 292 ns warm; 2.53 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 289.25 µs | 8.65 µs | 282.54 µs | 309.38 µs | 3.0 MB | 182 B | 57.62 µs cold, 167 ns warm; 2.51 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 286.10 µs | 3.25 µs | 281.08 µs | 321.75 µs | 3.0 MB | 182 B | 63.75 µs cold, 250 ns warm; 2.44 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.79 ms | 369.40 µs | 161.27 ms | 162.98 ms | 11.3 MB | 138 B | 4.93 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.95 ms | 338.26 µs | 41.29 ms | 41.60 ms | 5.0 MB | 138 B | 2.40 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.70 ms | 233.02 µs | 19.23 ms | 20.20 ms | 5.2 MB | 138 B | 2.42 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 514.19 µs | 337.70 µs | 269.42 µs | 1.14 ms | 61.6 MB | 155 B | 18.97 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 501.61 µs | 474.11 µs | 127.17 µs | 1.03 ms | 65.2 MB | 155 B | 18.93 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 273.79 µs | 325.66 µs | 123.08 µs | 1.09 ms | 69.3 MB | 155 B | 18.27 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 78.73 ms | 578.81 µs | 77.87 ms | 76.96 ms | 3.0 MB | 537 B | 90.79 µs cold, 250 ns warm; 2.50 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 95.11 ms | 249.44 µs | 94.58 ms | 95.04 ms | 4.8 MB | 411 B | 4.98 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 119.56 ms | 766.16 µs | 118.24 ms | 118.03 ms | 2.6 MB | 411 B | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.11 ms | 905.72 µs | 92.53 ms | 94.43 ms | 2.9 MB | 411 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.63 ms | 235.05 µs | 76.29 ms | 79.99 ms | 59.5 MB | 444 B | 19.29 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.52 ms | 314.80 µs | 10.16 ms | 12.94 ms | 65.6 MB | 444 B | 18.32 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.45 ms | 617.28 µs | 10.06 ms | 13.07 ms | 114.7 MB | 444 B | 18.69 ms as a whole process |  |
| Seal | Normal | 111.02 ms | 735.15 µs | 110.05 ms | 111.25 ms | 7.8 MB | 992 B | 119.62 µs cold, 250 ns warm; 2.65 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 130.36 ms | 1.28 ms | 129.05 ms | 132.03 ms | 41.9 MB | 799 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 128.74 ms | 1.06 ms | 127.02 ms | 127.43 ms | 41.5 MB | 799 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 87.31 ms | 4.88 ms | 81.12 ms | 82.23 ms | 42.4 MB | 799 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 116.44 ms | 1.71 ms | 115.01 ms | 119.88 ms | 213.1 MB | 878 B | 18.91 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.23 ms | 886.61 µs | 13.35 ms | 17.08 ms | 129.6 MB | 878 B | 18.69 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.37 ms | 55.84 µs | 13.28 ms | 17.17 ms | 224.0 MB | 878 B | 19.26 ms as a whole process |  |
| Seal | Ceil.source | 109.71 ms | 560.90 µs | 108.37 ms | 111.10 ms | 7.8 MB | 992 B | 121.00 µs cold, 250 ns warm; 2.58 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 112.42 ms | 352.93 µs | 111.77 ms | 113.55 ms | 7.8 MB | 992 B | 131.42 µs cold, 291 ns warm; 2.72 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 91.74 ms | 201.93 µs | 91.44 ms | 92.60 ms | 7.6 MB | 992 B | 105.96 µs cold, 208 ns warm; 2.66 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 91.55 ms | 244.82 µs | 91.13 ms | 91.38 ms | 7.6 MB | 992 B | 104.54 µs cold, 208 ns warm; 2.61 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 128.76 ms | 914.65 µs | 126.73 ms | 129.14 ms | 39.1 MB | 799 B | 5.12 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.04 ms | 741.71 µs | 127.26 ms | 126.67 ms | 39.8 MB | 799 B | 2.35 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 81.28 ms | 1.17 ms | 79.89 ms | 80.73 ms | 44.2 MB | 799 B | 2.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 114.06 ms | 1.87 ms | 112.56 ms | 115.53 ms | 213.4 MB | 878 B | 18.74 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.29 ms | 916.24 µs | 13.33 ms | 17.22 ms | 129.8 MB | 878 B | 18.93 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.52 ms | 48.91 µs | 13.45 ms | 17.22 ms | 223.8 MB | 878 B | 18.32 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 49.48 µs (±6.23 µs, least 42.21 µs) | 19.42 µs (±2.90 µs, least 15.54 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.72 µs (±1.29 µs, least 8.54 µs) | 17.04 µs (±2.37 µs, least 14.46 µs) | 41 ns (±2 ns, least 39 ns) | 22 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 4.60 µs (±1.35 µs, least 3.00 µs) | 7.10 µs (±1.10 µs, least 6.00 µs) | 22 ns (±1 ns, least 21 ns) | 13 ns (±1 ns, least 12 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 4.40 µs (±966 ns, least 4.00 µs) | 8.50 µs (±1.27 µs, least 6.00 µs) | 13 ns (±0 ns, least 12 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 35.31 µs (±7.45 µs, least 21.83 µs) | 41.85 µs (±4.03 µs, least 38.46 µs) | 57 ns (±1 ns, least 55 ns) | 30 ns (±0 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 29.84 µs (±7.40 µs, least 19.46 µs) | 37.64 µs (±1.52 µs, least 35.67 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.2 MB | the engine's own heap |
| Seal | Normal | 4.84 µs (±39 ns, least 4.76 µs) | 147 ns (±9 ns, least 137 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.91 µs (±170 ns, least 4.69 µs) | 185 ns (±4 ns, least 179 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.78 µs (±20 ns, least 1.75 µs) | 182 ns (±6 ns, least 170 ns) | 28 ns (±1 ns, least 27 ns) | 14 ns (±2 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.80 µs (±13 ns, least 1.78 µs) | 362 ns (±16 ns, least 340 ns) | 94 ns (±3 ns, least 91 ns) | 106 ns (±11 ns, least 100 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±15 ns, least 1.11 µs) | 665 ns (±18 ns, least 643 ns) | 51 ns (±2 ns, least 48 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.12 µs (±19 ns, least 1.08 µs) | 684 ns (±23 ns, least 649 ns) | 22 ns (±1 ns, least 21 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.86 µs (±60 ns, least 4.75 µs) | 147 ns (±9 ns, least 136 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.84 µs (±31 ns, least 4.80 µs) | 148 ns (±9 ns, least 132 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.25 µs (±66 ns, least 4.16 µs) | 141 ns (±7 ns, least 133 ns) | 14 ns (±0 ns, least 13 ns) | 34 ns (±1 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.30 µs (±63 ns, least 4.22 µs) | 141 ns (±7 ns, least 133 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±0 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.87 µs (±98 ns, least 4.68 µs) | 190 ns (±8 ns, least 181 ns) | 34 ns (±2 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.78 µs (±20 ns, least 1.75 µs) | 180 ns (±6 ns, least 175 ns) | 28 ns (±1 ns, least 27 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.83 µs (±14 ns, least 1.80 µs) | 357 ns (±14 ns, least 336 ns) | 94 ns (±1 ns, least 91 ns) | 106 ns (±11 ns, least 101 ns) | 1.0 KB | 4.8 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.14 µs (±21 ns, least 1.11 µs) | 678 ns (±14 ns, least 652 ns) | 52 ns (±2 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±22 ns, least 1.06 µs) | 672 ns (±18 ns, least 643 ns) | 22 ns (±0 ns, least 21 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 49.58 ms | 58.63 ms (+18.3%) |
| n-body, 100,000 steps | 78.95 ms | 61.11 ms (-22.6%) |
| spectral-norm, size 200 | 19.69 ms | 19.30 ms (-2.0%) |
| fannkuch-redux, 9 | 113.15 ms | 114.25 ms (+1.0%) |
| word count, 500,000 words | 24.49 ms | 24.16 ms (-1.3%) |
| text building, 20,000 joins | 343.86 µs | 338.97 µs (-1.4%) |
| binary-trees, depth 14 | 118.46 ms | 117.15 ms (-1.1%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 178.96 ms | 1.87 ms | 176.20 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.70 ms | 87.80 µs | 10.62 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.44 ms | 15.21 µs | 4.43 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.41 ms | 10.92 µs | 1.39 ms | 3.6 MB |
