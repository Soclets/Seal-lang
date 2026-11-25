# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 1.98 1.88 1.76 }` before, `{ 1.60 1.83 1.75 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 35.83 ms | 71.11 ms | **met**: Seal takes 50% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 64.44 ms | 69.74 ms | **met**: Seal takes 92% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.42 ms | 28.17 ms | **met**: Seal takes 65% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 103.20 ms | 210.33 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.42 ms | 29.93 ms | **met**: Seal takes 88% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 336.27 µs | 160.88 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 98.63 ms | 131.62 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.15 ms | 3.54 ms | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 34 ns | **met**: Seal takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 155 ns | 180 ns | **met**: Seal takes 86% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.55 µs | 4.88 µs | **met**: Seal takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 35.83 ms | 33.99 ms | **not met**: Seal takes 105% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 64.44 ms | 71.77 ms | **met**: Seal takes 90% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.42 ms | 45.89 ms | **met**: Seal takes 40% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 103.20 ms | 147.35 ms | **met**: Seal takes 70% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.42 ms | 31.36 ms | **met**: Seal takes 84% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 336.27 µs | 41.36 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 98.63 ms | 128.29 ms | **met**: Seal takes 77% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.15 ms | 2.91 ms | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 27 ns | **met**: Seal takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 155 ns | 183 ns | **met**: Seal takes 84% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.55 µs | 1.77 µs | **not met**: Seal takes 257% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 35.83 ms | 96.16 ms | **met**: Seal takes 37% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 64.44 ms | 175.75 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.42 ms | 51.11 ms | **met**: Seal takes 52% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 336.27 µs | 504.40 µs | **met**: Seal takes 67% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.51 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.30 ms | 343.17 µs | 83.93 ms | 103.68 ms | 2.8 MB | 386 B | 715.42 µs cold, 458 ns warm; 2.74 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 195.90 ms | 784.82 µs | 195.24 ms | 195.08 ms | 4.8 MB | 339 B | 5.15 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 89.13 ms | 511.42 µs | 88.36 ms | 88.16 ms | 1.9 MB | 339 B | 2.34 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.11 ms | 80.96 µs | 43.96 ms | 45.82 ms | 2.2 MB | 339 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 320.53 ms | 1.32 ms | 318.24 ms | 321.63 ms | 43.4 MB | 362 B | 18.18 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.24 ms | 615.46 µs | 123.00 ms | 124.89 ms | 49.1 MB | 362 B | 18.83 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 123.43 ms | 515.38 µs | 122.59 ms | 125.00 ms | 49.1 MB | 362 B | 18.91 ms as a whole process |  |
| Seal | Normal | 35.83 ms | 1.45 ms | 34.16 ms | 33.82 ms | 2.7 MB | 160 B | 72.75 µs cold, 292 ns warm; 2.62 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 71.11 ms | 134.15 µs | 70.96 ms | 71.06 ms | 4.8 MB | 143 B | 4.96 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.99 ms | 122.64 µs | 33.74 ms | 33.78 ms | 1.9 MB | 143 B | 2.43 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.31 ms | 19.44 µs | 7.28 ms | 7.56 ms | 2.2 MB | 143 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Normal | 96.16 ms | 304.45 µs | 95.59 ms | 96.30 ms | 43.4 MB | 137 B | 19.04 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.54 ms | 175.77 µs | 9.46 ms | 9.74 ms | 48.6 MB | 137 B | 18.49 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 18.14 ms | 1.31 ms | 16.30 ms | 10.56 ms | 48.7 MB | 137 B | 18.14 ms as a whole process |  |
| Seal | Ceil.source | 36.23 ms | 1.06 ms | 34.56 ms | 34.37 ms | 2.7 MB | 160 B | 76.54 µs cold, 250 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 33.77 ms | 552.36 µs | 32.86 ms | 54.73 ms | 2.7 MB | 160 B | 1.01 ms cold, 417 ns warm; 2.70 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 34.62 ms | 579.59 µs | 33.17 ms | 55.44 ms | 2.5 MB | 160 B | 664.62 µs cold, 416 ns warm; 2.42 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 33.90 ms | 656.35 µs | 33.25 ms | 50.95 ms | 2.5 MB | 160 B | 914.08 µs cold, 458 ns warm; 2.57 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 72.34 ms | 283.79 µs | 71.90 ms | 72.92 ms | 4.8 MB | 143 B | 5.00 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.66 ms | 54.24 µs | 33.56 ms | 33.78 ms | 1.9 MB | 143 B | 2.48 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.07 ms | 17.55 µs | 7.04 ms | 7.32 ms | 2.2 MB | 143 B | 2.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.86 ms | 263.43 µs | 95.39 ms | 95.30 ms | 43.3 MB | 137 B | 18.74 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.82 ms | 611.71 µs | 9.40 ms | 11.03 ms | 48.5 MB | 137 B | 18.25 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.04 ms | 398.55 µs | 9.39 ms | 10.85 ms | 48.5 MB | 137 B | 18.69 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.75 ms | 453.07 µs | 47.92 ms | 51.33 ms | 2.7 MB | 547 B | 101.29 µs cold, 208 ns warm; 2.73 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.70 ms | 326.35 µs | 91.35 ms | 90.84 ms | 4.8 MB | 433 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.04 ms | 648.45 µs | 76.10 ms | 76.15 ms | 1.9 MB | 433 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.69 ms | 68.92 µs | 6.59 ms | 6.72 ms | 2.1 MB | 433 B | 2.28 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.62 ms | 366.41 µs | 242.97 ms | 244.80 ms | 46.4 MB | 460 B | 18.39 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.45 ms | 217.44 µs | 18.12 ms | 18.72 ms | 50.2 MB | 460 B | 18.38 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.55 ms | 182.95 µs | 18.33 ms | 18.57 ms | 50.4 MB | 460 B | 18.18 ms as a whole process |  |
| Seal | Normal | 64.44 ms | 645.21 µs | 63.68 ms | 65.02 ms | 3.2 MB | 3.5 KB | 271.88 µs cold, 250 ns warm; 2.82 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.74 ms | 234.39 µs | 69.39 ms | 69.88 ms | 4.8 MB | 3.0 KB | 5.14 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.77 ms | 305.09 µs | 71.32 ms | 71.58 ms | 1.9 MB | 3.0 KB | 2.33 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.19 ms | 20.77 µs | 5.16 ms | 5.41 ms | 2.2 MB | 3.0 KB | 2.40 ms as a whole process |  |
| Node `--jitless` | Normal | 175.75 ms | 3.53 ms | 171.51 ms | 180.41 ms | 48.2 MB | 3.1 KB | 18.44 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.88 ms | 15.27 µs | 3.86 ms | 5.66 ms | 51.9 MB | 3.1 KB | 18.97 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 16.82 µs | 3.84 ms | 5.62 ms | 51.9 MB | 3.1 KB | 18.85 ms as a whole process |  |
| Seal | Ceil.source | 55.55 ms | 542.80 µs | 54.69 ms | 55.30 ms | 3.2 MB | 3.4 KB | 247.88 µs cold, 416 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.23 ms | 476.46 µs | 54.53 ms | 55.89 ms | 3.2 MB | 3.4 KB | 268.92 µs cold, 250 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.64 ms | 363.64 µs | 49.07 ms | 49.40 ms | 3.0 MB | 3.4 KB | 254.83 µs cold, 292 ns warm; 2.84 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.63 ms | 399.91 µs | 48.97 ms | 49.27 ms | 3.0 MB | 3.4 KB | 262.29 µs cold, 208 ns warm; 2.86 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.83 ms | 168.05 µs | 69.61 ms | 69.98 ms | 4.8 MB | 3.0 KB | 5.19 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 72.01 ms | 282.34 µs | 71.39 ms | 71.42 ms | 1.9 MB | 3.0 KB | 2.54 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.19 ms | 19.14 µs | 5.16 ms | 5.36 ms | 2.2 MB | 3.0 KB | 2.46 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.81 ms | 4.37 ms | 171.05 ms | 181.84 ms | 48.1 MB | 3.1 KB | 18.68 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.90 ms | 34.13 µs | 3.85 ms | 5.70 ms | 51.9 MB | 3.1 KB | 18.47 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 13.40 µs | 3.84 ms | 5.64 ms | 52.0 MB | 3.1 KB | 18.42 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 47.14 ms | 127.04 µs | 46.89 ms | 50.37 ms | 2.8 MB | 426 B | 87.67 µs cold, 209 ns warm; 2.44 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.45 ms | 675.93 µs | 44.88 ms | 45.14 ms | 4.8 MB | 366 B | 4.92 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.05 ms | 82.57 µs | 76.95 ms | 76.98 ms | 1.9 MB | 366 B | 2.27 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.58 ms | 243.31 µs | 22.21 ms | 22.69 ms | 2.1 MB | 366 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Purpose | 370.60 ms | 6.21 ms | 365.35 ms | 378.53 ms | 48.5 MB | 398 B | 18.44 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.88 ms | 37.95 ms | 15.74 ms | 19.69 ms | 52.8 MB | 398 B | 18.52 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.25 ms | 307.50 µs | 14.76 ms | 20.12 ms | 55.4 MB | 398 B | 18.50 ms as a whole process |  |
| Seal | Normal | 18.42 ms | 196.72 µs | 18.22 ms | 18.70 ms | 3.0 MB | 1.2 KB | 136.96 µs cold, 291 ns warm; 2.57 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 28.17 ms | 473.79 µs | 27.26 ms | 27.99 ms | 4.9 MB | 981 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.89 ms | 323.53 µs | 45.33 ms | 45.28 ms | 2.0 MB | 981 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.55 ms | 37.18 µs | 1.52 ms | 1.68 ms | 2.2 MB | 981 B | 2.41 ms as a whole process |  |
| Node `--jitless` | Normal | 65.02 ms | 316.83 µs | 64.50 ms | 64.83 ms | 48.4 MB | 1.0 KB | 18.33 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.63 ms | 171.36 µs | 1.47 ms | 3.34 ms | 53.0 MB | 1.0 KB | 18.49 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.65 ms | 20.38 µs | 1.62 ms | 3.23 ms | 52.8 MB | 1.0 KB | 18.95 ms as a whole process |  |
| Seal | Ceil.source | 18.94 ms | 214.29 µs | 18.49 ms | 19.35 ms | 3.0 MB | 1.2 KB | 133.62 µs cold, 209 ns warm; 2.74 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.93 ms | 95.96 µs | 18.79 ms | 18.90 ms | 3.0 MB | 1.2 KB | 143.17 µs cold, 208 ns warm; 2.54 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.51 ms | 50.14 µs | 18.45 ms | 18.48 ms | 2.8 MB | 1.2 KB | 124.25 µs cold, 166 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.91 ms | 47.19 µs | 18.84 ms | 19.00 ms | 2.8 MB | 1.2 KB | 124.29 µs cold, 250 ns warm; 2.61 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.18 ms | 296.89 µs | 26.75 ms | 26.80 ms | 4.8 MB | 981 B | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 43.49 ms | 546.35 µs | 42.77 ms | 44.03 ms | 2.0 MB | 981 B | 2.55 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.55 ms | 47.78 µs | 1.51 ms | 1.71 ms | 2.2 MB | 981 B | 2.48 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.67 ms | 434.16 µs | 64.11 ms | 65.06 ms | 48.4 MB | 1.0 KB | 18.35 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.65 ms | 206.92 µs | 1.52 ms | 3.68 ms | 53.2 MB | 1.0 KB | 19.05 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.63 ms | 21.51 µs | 1.60 ms | 3.66 ms | 52.8 MB | 1.0 KB | 18.42 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 85.90 ms | 334.97 µs | 85.30 ms | 85.75 ms | 2.7 MB | 629 B | 88.46 µs cold, 209 ns warm; 2.62 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 87.74 ms | 696.33 µs | 86.61 ms | 92.37 ms | 4.8 MB | 523 B | 5.01 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.02 ms | 520.51 µs | 52.15 ms | 52.31 ms | 1.9 MB | 523 B | 2.33 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.15 ms | 13.34 µs | 6.12 ms | 6.94 ms | 2.1 MB | 523 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Purpose | 147.18 ms | 720.05 µs | 146.43 ms | 148.18 ms | 43.4 MB | 568 B | 18.55 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 38.87 µs | 3.63 ms | 4.52 ms | 49.4 MB | 568 B | 18.59 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.70 ms | 43.45 µs | 3.63 ms | 4.46 ms | 49.4 MB | 568 B | 18.59 ms as a whole process |  |
| Seal | Normal | 103.20 ms | 133.50 µs | 103.04 ms | 103.01 ms | 2.9 MB | 1.8 KB | 148.08 µs cold, 291 ns warm; 2.67 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 210.33 ms | 1.76 ms | 205.40 ms | 212.38 ms | 4.8 MB | 1.7 KB | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.35 ms | 342.61 µs | 146.75 ms | 145.47 ms | 1.9 MB | 1.7 KB | 2.39 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 49.13 ms | 137.42 µs | 48.84 ms | 48.49 ms | 2.3 MB | 1.7 KB | 2.46 ms as a whole process |  |
| Node `--jitless` | Normal | 157.40 ms | 965.32 µs | 156.59 ms | 156.82 ms | 45.5 MB | 1.7 KB | 18.71 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.57 ms | 71.57 µs | 14.46 ms | 16.13 ms | 52.0 MB | 1.7 KB | 18.56 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.59 ms | 57.13 µs | 14.50 ms | 16.07 ms | 52.0 MB | 1.7 KB | 19.04 ms as a whole process |  |
| Seal | Ceil.source | 103.69 ms | 653.10 µs | 102.49 ms | 103.79 ms | 2.9 MB | 1.8 KB | 140.42 µs cold, 292 ns warm; 2.84 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 108.61 ms | 60.88 µs | 108.48 ms | 107.47 ms | 3.0 MB | 1.8 KB | 146.42 µs cold, 250 ns warm; 2.68 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 99.92 ms | 121.17 µs | 99.73 ms | 100.09 ms | 2.8 MB | 1.8 KB | 149.29 µs cold, 209 ns warm; 2.65 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.80 ms | 128.38 µs | 103.63 ms | 103.43 ms | 2.8 MB | 1.8 KB | 125.04 µs cold, 208 ns warm; 2.49 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 214.90 ms | 2.08 ms | 209.02 ms | 216.19 ms | 4.8 MB | 1.7 KB | 5.03 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 148.04 ms | 508.79 µs | 147.47 ms | 145.59 ms | 1.9 MB | 1.7 KB | 2.53 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.72 ms | 331.49 µs | 48.16 ms | 49.03 ms | 2.3 MB | 1.7 KB | 2.55 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.27 ms | 1.32 ms | 155.56 ms | 158.93 ms | 45.5 MB | 1.7 KB | 18.40 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.54 ms | 81.15 µs | 14.41 ms | 16.21 ms | 52.0 MB | 1.7 KB | 18.52 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.71 ms | 55.27 µs | 14.58 ms | 16.36 ms | 52.1 MB | 1.7 KB | 18.85 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 127.57 ms | 644.77 µs | 126.69 ms | 125.24 ms | 4.0 MB | 566 B | 104.08 µs cold, 333 ns warm; 2.73 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.13 ms | 144.09 µs | 62.93 ms | 63.22 ms | 4.8 MB | 540 B | 4.99 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 28.72 ms | 778.47 µs | 28.12 ms | 30.30 ms | 2.1 MB | 540 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.73 ms | 29.55 µs | 4.66 ms | 4.77 ms | 2.3 MB | 540 B | 2.42 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.33 ms | 750.21 µs | 242.36 ms | 249.36 ms | 48.7 MB | 514 B | 18.60 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 89.43 ms | 23.03 ms | 67.72 ms | 91.42 ms | 52.6 MB | 514 B | 18.41 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 63.78 ms | 417.83 µs | 63.17 ms | 87.52 ms | 52.8 MB | 514 B | 18.45 ms as a whole process |  |
| Seal | Normal | 26.42 ms | 59.15 µs | 26.32 ms | 26.61 ms | 4.0 MB | 535 B | 100.25 µs cold, 250 ns warm; 2.40 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.93 ms | 91.66 µs | 29.73 ms | 30.02 ms | 5.9 MB | 478 B | 5.14 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.36 ms | 239.91 µs | 31.09 ms | 29.82 ms | 3.0 MB | 478 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 11.93 ms | 486.82 µs | 11.22 ms | 11.36 ms | 3.3 MB | 478 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 51.11 ms | 271.41 µs | 50.77 ms | 52.50 ms | 56.4 MB | 477 B | 19.08 ms as a whole process |  |
| Node (JIT), from cold | Normal | 34.78 ms | 205.67 µs | 34.46 ms | 35.66 ms | 62.5 MB | 477 B | 18.64 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.06 ms | 296.50 µs | 34.64 ms | 35.80 ms | 87.9 MB | 477 B | 19.12 ms as a whole process |  |
| Seal | Ceil.source | 26.62 ms | 114.52 µs | 26.40 ms | 26.74 ms | 3.9 MB | 535 B | 97.38 µs cold, 334 ns warm; 2.60 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.84 ms | 173.15 µs | 27.63 ms | 28.06 ms | 4.0 MB | 535 B | 123.58 µs cold, 333 ns warm; 2.60 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.84 ms | 103.27 µs | 23.66 ms | 24.08 ms | 4.0 MB | 535 B | 88.33 µs cold, 208 ns warm; 2.70 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.15 ms | 549.48 µs | 23.37 ms | 23.74 ms | 3.9 MB | 535 B | 92.17 µs cold, 292 ns warm; 2.70 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.29 ms | 326.14 µs | 29.87 ms | 30.42 ms | 6.0 MB | 478 B | 4.97 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.71 ms | 648.54 µs | 32.68 ms | 29.73 ms | 3.0 MB | 478 B | 2.32 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.36 ms | 315.66 µs | 11.96 ms | 12.45 ms | 3.3 MB | 478 B | 2.42 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.01 ms | 421.56 µs | 51.53 ms | 53.12 ms | 56.6 MB | 477 B | 19.04 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.79 ms | 219.61 µs | 34.56 ms | 35.94 ms | 62.3 MB | 477 B | 18.78 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 34.89 ms | 237.05 µs | 34.65 ms | 35.50 ms | 88.2 MB | 477 B | 18.81 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 308.03 µs | 4.98 µs | 302.25 µs | 326.54 µs | 3.1 MB | 230 B | 66.71 µs cold, 375 ns warm; 2.49 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.14 ms | 339.70 µs | 143.60 ms | 146.10 ms | 9.7 MB | 195 B | 5.06 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 33.00 ms | 67.85 µs | 32.88 ms | 31.10 ms | 3.5 MB | 195 B | 2.32 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.71 ms | 232.90 µs | 20.47 ms | 20.99 ms | 3.7 MB | 195 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Purpose | 372.00 µs | 149.94 µs | 246.83 µs | 1.02 ms | 52.4 MB | 223 B | 18.55 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 215.18 µs | 156.50 µs | 83.25 µs | 934.71 µs | 57.7 MB | 223 B | 18.50 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 141.28 µs | 117.64 µs | 62.83 µs | 914.58 µs | 57.8 MB | 223 B | 18.62 ms as a whole process |  |
| Seal | Normal | 336.27 µs | 6.16 µs | 329.88 µs | 368.21 µs | 3.2 MB | 179 B | 70.17 µs cold, 416 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.88 ms | 500.15 µs | 159.66 ms | 162.87 ms | 11.3 MB | 138 B | 4.90 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.36 ms | 302.99 µs | 40.88 ms | 40.35 ms | 5.0 MB | 138 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.24 ms | 81.21 µs | 19.16 ms | 20.20 ms | 5.2 MB | 138 B | 2.41 ms as a whole process |  |
| Node `--jitless` | Normal | 504.40 µs | 351.44 µs | 261.42 µs | 1.12 ms | 61.4 MB | 155 B | 18.40 ms as a whole process |  |
| Node (JIT), from cold | Normal | 492.96 µs | 482.27 µs | 125.08 µs | 1.03 ms | 65.3 MB | 155 B | 18.29 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 275.11 µs | 326.10 µs | 122.71 µs | 1.07 ms | 69.2 MB | 155 B | 18.09 ms as a whole process |  |
| Seal | Ceil.source | 337.91 µs | 3.21 µs | 333.71 µs | 351.21 µs | 3.2 MB | 179 B | 71.12 µs cold, 291 ns warm; 2.52 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 350.43 µs | 4.68 µs | 345.50 µs | 377.92 µs | 3.2 MB | 179 B | 74.04 µs cold, 334 ns warm; 2.64 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 288.83 µs | 4.31 µs | 285.17 µs | 302.12 µs | 3.0 MB | 179 B | 59.08 µs cold, 250 ns warm; 2.44 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 290.56 µs | 3.52 µs | 286.96 µs | 311.62 µs | 3.1 MB | 179 B | 64.75 µs cold, 292 ns warm; 2.62 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 160.27 ms | 752.49 µs | 159.50 ms | 163.08 ms | 11.8 MB | 138 B | 4.91 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.69 ms | 126.64 µs | 41.45 ms | 41.53 ms | 4.8 MB | 138 B | 2.31 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.18 ms | 136.08 µs | 19.11 ms | 19.37 ms | 5.1 MB | 138 B | 2.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 501.84 µs | 336.59 µs | 266.04 µs | 1.10 ms | 61.4 MB | 155 B | 18.54 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 498.16 µs | 483.12 µs | 121.96 µs | 1.09 ms | 65.3 MB | 155 B | 18.77 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 279.47 µs | 320.36 µs | 124.42 µs | 1.07 ms | 69.4 MB | 155 B | 18.73 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 73.13 ms | 683.79 µs | 71.47 ms | 73.49 ms | 2.9 MB | 528 B | 90.71 µs cold, 333 ns warm; 2.63 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 94.32 ms | 271.65 µs | 93.84 ms | 94.08 ms | 4.8 MB | 411 B | 5.06 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 118.90 ms | 1.23 ms | 116.69 ms | 117.60 ms | 2.5 MB | 411 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 93.84 ms | 718.18 µs | 92.22 ms | 91.92 ms | 2.9 MB | 411 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.64 ms | 250.97 µs | 75.31 ms | 79.43 ms | 59.5 MB | 444 B | 19.15 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.50 ms | 288.35 µs | 10.23 ms | 13.05 ms | 65.8 MB | 444 B | 18.35 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.50 ms | 640.81 µs | 10.11 ms | 13.07 ms | 114.8 MB | 444 B | 19.04 ms as a whole process |  |
| Seal | Normal | 98.63 ms | 272.80 µs | 98.08 ms | 99.96 ms | 7.8 MB | 977 B | 125.92 µs cold, 209 ns warm; 2.70 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 131.62 ms | 1.13 ms | 130.60 ms | 130.48 ms | 39.8 MB | 799 B | 5.19 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 128.29 ms | 1.01 ms | 126.45 ms | 128.07 ms | 41.5 MB | 799 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.20 ms | 847.41 µs | 79.55 ms | 79.94 ms | 43.4 MB | 799 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Normal | 114.58 ms | 2.07 ms | 112.82 ms | 117.31 ms | 213.4 MB | 878 B | 18.76 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.26 ms | 935.88 µs | 13.38 ms | 16.93 ms | 129.7 MB | 878 B | 18.58 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.34 ms | 58.18 µs | 13.27 ms | 17.00 ms | 224.0 MB | 878 B | 19.38 ms as a whole process |  |
| Seal | Ceil.source | 99.81 ms | 488.96 µs | 99.11 ms | 101.55 ms | 7.8 MB | 977 B | 115.42 µs cold, 250 ns warm; 2.54 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 100.26 ms | 421.96 µs | 99.69 ms | 101.29 ms | 7.8 MB | 977 B | 122.21 µs cold, 208 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 84.49 ms | 441.53 µs | 83.98 ms | 84.70 ms | 7.6 MB | 977 B | 111.46 µs cold, 209 ns warm; 2.66 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 85.04 ms | 262.81 µs | 84.46 ms | 84.77 ms | 7.6 MB | 977 B | 104.17 µs cold, 167 ns warm; 2.58 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 128.63 ms | 537.04 µs | 127.88 ms | 129.89 ms | 43.4 MB | 799 B | 5.06 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 127.17 ms | 565.38 µs | 126.48 ms | 125.92 ms | 40.5 MB | 799 B | 2.53 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 82.09 ms | 583.06 µs | 80.77 ms | 81.86 ms | 41.6 MB | 799 B | 2.54 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.03 ms | 2.26 ms | 113.05 ms | 119.03 ms | 213.3 MB | 878 B | 18.66 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.20 ms | 873.53 µs | 13.35 ms | 17.02 ms | 129.7 MB | 878 B | 18.16 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.39 ms | 93.86 µs | 13.23 ms | 16.89 ms | 224.2 MB | 878 B | 18.61 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 47.10 µs (±5.90 µs, least 41.96 µs) | 14.25 µs (±2.12 µs, least 11.38 µs) | 19 ns (±0 ns, least 19 ns) | 41 ns (±1 ns, least 40 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.18 µs (±1000 ns, least 8.62 µs) | 16.97 µs (±1.47 µs, least 15.12 µs) | 41 ns (±1 ns, least 40 ns) | 21 ns (±2 ns, least 18 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 4.00 µs (±0 ns, least 4.00 µs) | 8.30 µs (±1.49 µs, least 5.00 µs) | 22 ns (±1 ns, least 20 ns) | 14 ns (±2 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.80 µs (±632 ns, least 3.00 µs) | 7.40 µs (±1.78 µs, least 5.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 33.90 µs (±6.60 µs, least 21.04 µs) | 38.38 µs (±1.70 µs, least 36.04 µs) | 56 ns (±2 ns, least 54 ns) | 29 ns (±0 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 35.38 µs (±921 ns, least 34.12 µs) | 37.06 µs (±1.34 µs, least 34.58 µs) | 8 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 4.55 µs (±47 ns, least 4.49 µs) | 155 ns (±10 ns, least 141 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.88 µs (±122 ns, least 4.70 µs) | 180 ns (±8 ns, least 161 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.77 µs (±20 ns, least 1.73 µs) | 183 ns (±6 ns, least 174 ns) | 27 ns (±0 ns, least 27 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.79 µs (±24 ns, least 1.77 µs) | 356 ns (±14 ns, least 337 ns) | 92 ns (±2 ns, least 90 ns) | 108 ns (±16 ns, least 99 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.16 µs (±31 ns, least 1.12 µs) | 657 ns (±11 ns, least 636 ns) | 51 ns (±2 ns, least 49 ns) | 24 ns (±2 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.10 µs (±24 ns, least 1.07 µs) | 657 ns (±14 ns, least 640 ns) | 22 ns (±1 ns, least 21 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.49 µs (±50 ns, least 4.43 µs) | 149 ns (±6 ns, least 140 ns) | 20 ns (±0 ns, least 20 ns) | 39 ns (±1 ns, least 39 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.62 µs (±43 ns, least 4.52 µs) | 152 ns (±9 ns, least 142 ns) | 21 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 3.99 µs (±35 ns, least 3.92 µs) | 140 ns (±6 ns, least 127 ns) | 18 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.02 µs (±30 ns, least 3.96 µs) | 140 ns (±7 ns, least 133 ns) | 18 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.84 µs (±141 ns, least 4.68 µs) | 183 ns (±9 ns, least 172 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.79 µs (±21 ns, least 1.75 µs) | 180 ns (±6 ns, least 170 ns) | 28 ns (±0 ns, least 27 ns) | 16 ns (±4 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.80 µs (±15 ns, least 1.77 µs) | 360 ns (±12 ns, least 340 ns) | 94 ns (±2 ns, least 91 ns) | 109 ns (±12 ns, least 99 ns) | 1.0 KB | 4.8 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±22 ns, least 1.12 µs) | 657 ns (±15 ns, least 626 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±19 ns, least 1.07 µs) | 666 ns (±19 ns, least 637 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 33.98 ms | 31.80 ms (-6.4%) |
| n-body, 100,000 steps | 65.97 ms | 60.84 ms (-7.8%) |
| spectral-norm, size 200 | 18.51 ms | 17.87 ms (-3.5%) |
| fannkuch-redux, 9 | 104.30 ms | 102.55 ms (-1.7%) |
| word count, 500,000 words | 26.48 ms | 26.22 ms (-1.0%) |
| text building, 20,000 joins | 338.37 µs | 334.58 µs (-1.1%) |
| binary-trees, depth 14 | 98.81 ms | 99.13 ms (+0.3%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 171.34 ms | 2.55 ms | 169.19 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.38 ms | 166.38 µs | 10.14 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.45 ms | 31.00 µs | 4.41 ms | 3.4 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.43 ms | 5.97 µs | 1.42 ms | 3.3 MB |
