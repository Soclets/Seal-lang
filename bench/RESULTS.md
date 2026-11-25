# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 1.82 1.72 1.73 }` before, `{ 1.84 1.70 1.71 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 32.15 ms | 71.43 ms | **met**: Seal takes 45% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.14 ms | 69.43 ms | **met**: Seal takes 94% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.54 ms | 27.27 ms | **met**: Seal takes 68% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 101.93 ms | 210.11 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.29 ms | 30.37 ms | **met**: Seal takes 87% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 343.94 µs | 160.36 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 98.25 ms | 131.28 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.17 ms | 3.56 ms | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 34 ns | **met**: Seal takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 153 ns | 186 ns | **met**: Seal takes 82% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.50 µs | 4.85 µs | **met**: Seal takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 32.15 ms | 33.86 ms | **met**: Seal takes 95% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 65.14 ms | 71.74 ms | **met**: Seal takes 91% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.54 ms | 43.58 ms | **met**: Seal takes 43% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 101.93 ms | 148.01 ms | **met**: Seal takes 69% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.29 ms | 32.21 ms | **met**: Seal takes 82% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 343.94 µs | 41.71 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 98.25 ms | 127.96 ms | **met**: Seal takes 77% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.17 ms | 2.96 ms | **met**: Seal takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 28 ns | **met**: Seal takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 153 ns | 178 ns | **met**: Seal takes 86% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.50 µs | 1.79 µs | **not met**: Seal takes 251% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 32.15 ms | 96.05 ms | **met**: Seal takes 33% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.14 ms | 175.53 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.29 ms | 52.86 ms | **met**: Seal takes 50% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 343.94 µs | 517.78 µs | **met**: Seal takes 66% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.43 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 83.30 ms | 456.91 µs | 82.80 ms | 102.82 ms | 2.8 MB | 386 B | 761.17 µs cold, 500 ns warm; 2.61 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 209.24 ms | 2.82 ms | 204.47 ms | 200.02 ms | 4.8 MB | 339 B | 5.08 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 89.75 ms | 978.75 µs | 88.16 ms | 89.23 ms | 1.9 MB | 339 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.11 ms | 166.15 µs | 43.82 ms | 45.77 ms | 2.2 MB | 339 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Purpose | 320.45 ms | 486.80 µs | 319.69 ms | 320.43 ms | 43.4 MB | 362 B | 18.15 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 120.63 ms | 276.12 µs | 120.10 ms | 121.36 ms | 49.2 MB | 362 B | 18.86 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 121.27 ms | 265.66 µs | 120.95 ms | 121.35 ms | 49.2 MB | 362 B | 18.49 ms as a whole process |  |
| Seal | Normal | 32.15 ms | 835.93 µs | 30.63 ms | 30.60 ms | 2.7 MB | 160 B | 71.67 µs cold, 167 ns warm; 2.68 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 71.43 ms | 239.03 µs | 71.04 ms | 72.09 ms | 4.8 MB | 143 B | 5.09 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.86 ms | 93.05 µs | 33.69 ms | 33.92 ms | 1.9 MB | 143 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.06 ms | 15.69 µs | 7.04 ms | 7.27 ms | 2.2 MB | 143 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 96.05 ms | 589.94 µs | 95.33 ms | 93.87 ms | 43.4 MB | 137 B | 18.83 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.87 ms | 816.73 µs | 9.41 ms | 11.56 ms | 48.7 MB | 137 B | 18.39 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.52 ms | 48.41 µs | 9.41 ms | 9.90 ms | 48.5 MB | 137 B | 19.11 ms as a whole process |  |
| Seal | Ceil.source | 32.23 ms | 633.13 µs | 31.11 ms | 31.38 ms | 2.7 MB | 160 B | 74.88 µs cold, 208 ns warm; 2.55 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 34.09 ms | 1.29 ms | 33.02 ms | 53.54 ms | 2.7 MB | 160 B | 833.04 µs cold, 500 ns warm; 2.62 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 31.83 ms | 313.89 µs | 31.19 ms | 52.81 ms | 2.5 MB | 160 B | 658.71 µs cold, 375 ns warm; 2.63 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 30.18 ms | 634.53 µs | 29.59 ms | 51.32 ms | 2.5 MB | 160 B | 944.21 µs cold, 708 ns warm; 2.59 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 72.58 ms | 444.11 µs | 72.07 ms | 73.19 ms | 4.8 MB | 143 B | 5.05 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.89 ms | 53.76 µs | 33.81 ms | 33.91 ms | 1.9 MB | 143 B | 2.43 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.51 ms | 27.40 µs | 7.46 ms | 7.62 ms | 2.1 MB | 143 B | 2.28 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.59 ms | 623.59 µs | 94.89 ms | 94.86 ms | 43.3 MB | 137 B | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.80 ms | 427.99 µs | 9.52 ms | 10.41 ms | 48.5 MB | 137 B | 19.33 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.30 ms | 340.96 µs | 9.51 ms | 9.89 ms | 48.6 MB | 137 B | 18.78 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.98 ms | 1.26 ms | 47.92 ms | 53.69 ms | 2.7 MB | 547 B | 102.62 µs cold, 250 ns warm; 2.56 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.58 ms | 335.04 µs | 91.18 ms | 90.74 ms | 4.8 MB | 433 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.67 ms | 449.55 µs | 75.87 ms | 76.01 ms | 1.9 MB | 433 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.61 ms | 127.30 µs | 6.44 ms | 6.56 ms | 2.0 MB | 433 B | 2.27 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.89 ms | 550.89 µs | 244.20 ms | 244.53 ms | 46.4 MB | 460 B | 19.27 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.36 ms | 181.45 µs | 18.16 ms | 18.54 ms | 50.2 MB | 460 B | 18.98 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.33 ms | 236.55 µs | 18.11 ms | 18.27 ms | 50.3 MB | 460 B | 17.94 ms as a whole process |  |
| Seal | Normal | 65.14 ms | 697.24 µs | 64.13 ms | 67.22 ms | 3.2 MB | 3.5 KB | 271.12 µs cold, 250 ns warm; 2.87 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.43 ms | 229.35 µs | 68.90 ms | 70.07 ms | 4.8 MB | 3.0 KB | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.74 ms | 200.69 µs | 71.39 ms | 71.31 ms | 1.9 MB | 3.0 KB | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.20 ms | 25.52 µs | 5.17 ms | 5.41 ms | 2.2 MB | 3.0 KB | 2.44 ms as a whole process |  |
| Node `--jitless` | Normal | 175.53 ms | 3.95 ms | 171.53 ms | 181.12 ms | 48.3 MB | 3.1 KB | 19.20 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 17.55 µs | 3.84 ms | 5.56 ms | 51.9 MB | 3.1 KB | 18.99 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.86 ms | 15.07 µs | 3.83 ms | 5.73 ms | 51.8 MB | 3.1 KB | 18.59 ms as a whole process |  |
| Seal | Ceil.source | 54.75 ms | 363.65 µs | 54.29 ms | 56.18 ms | 3.1 MB | 3.4 KB | 261.54 µs cold, 333 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.92 ms | 257.72 µs | 55.42 ms | 56.79 ms | 3.2 MB | 3.4 KB | 251.21 µs cold, 250 ns warm; 2.88 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.94 ms | 612.85 µs | 49.43 ms | 49.71 ms | 3.0 MB | 3.4 KB | 257.00 µs cold, 250 ns warm; 2.63 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.65 ms | 190.39 µs | 49.45 ms | 49.49 ms | 3.0 MB | 3.4 KB | 263.17 µs cold, 208 ns warm; 2.73 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.89 ms | 132.66 µs | 69.68 ms | 69.94 ms | 4.8 MB | 3.0 KB | 5.11 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.52 ms | 307.53 µs | 71.11 ms | 70.96 ms | 1.9 MB | 3.0 KB | 2.38 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.25 ms | 28.28 µs | 5.22 ms | 5.46 ms | 2.2 MB | 3.0 KB | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 177.58 ms | 3.25 ms | 173.93 ms | 181.04 ms | 47.9 MB | 3.1 KB | 18.99 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.87 ms | 15.13 µs | 3.84 ms | 5.66 ms | 52.0 MB | 3.1 KB | 18.55 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 22.12 µs | 3.84 ms | 5.66 ms | 51.9 MB | 3.1 KB | 18.34 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 58.59 ms | 1.17 ms | 55.82 ms | 57.67 ms | 2.9 MB | 426 B | 95.50 µs cold, 167 ns warm; 2.60 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.61 ms | 253.61 µs | 45.34 ms | 45.39 ms | 4.9 MB | 366 B | 4.97 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.03 ms | 60.93 µs | 76.94 ms | 76.96 ms | 1.9 MB | 366 B | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.05 ms | 52.43 µs | 21.97 ms | 22.53 ms | 2.1 MB | 366 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.61 ms | 7.01 ms | 360.97 ms | 378.04 ms | 48.7 MB | 398 B | 18.54 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.13 ms | 36.44 ms | 16.72 ms | 19.37 ms | 52.7 MB | 398 B | 18.23 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.44 ms | 276.03 µs | 14.83 ms | 18.78 ms | 55.4 MB | 398 B | 18.61 ms as a whole process |  |
| Seal | Normal | 18.54 ms | 200.12 µs | 18.19 ms | 19.90 ms | 3.0 MB | 1.2 KB | 153.12 µs cold, 208 ns warm; 2.56 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.27 ms | 335.44 µs | 26.65 ms | 27.53 ms | 4.8 MB | 981 B | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 43.58 ms | 706.92 µs | 42.86 ms | 44.14 ms | 2.0 MB | 981 B | 2.33 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.54 ms | 16.68 µs | 1.52 ms | 1.70 ms | 2.2 MB | 981 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 65.48 ms | 383.54 µs | 64.68 ms | 64.52 ms | 48.4 MB | 1.0 KB | 18.86 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.60 ms | 171.62 µs | 1.47 ms | 3.27 ms | 52.8 MB | 1.0 KB | 19.21 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.62 ms | 14.19 µs | 1.59 ms | 3.27 ms | 52.8 MB | 1.0 KB | 18.77 ms as a whole process |  |
| Seal | Ceil.source | 18.91 ms | 181.80 µs | 18.69 ms | 18.99 ms | 3.0 MB | 1.2 KB | 144.04 µs cold, 292 ns warm; 2.72 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.68 ms | 144.69 µs | 18.50 ms | 18.64 ms | 3.0 MB | 1.2 KB | 149.29 µs cold, 208 ns warm; 2.65 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.51 ms | 23.62 µs | 18.48 ms | 18.48 ms | 2.8 MB | 1.2 KB | 124.08 µs cold, 209 ns warm; 2.73 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.91 ms | 100.99 µs | 18.80 ms | 18.81 ms | 2.8 MB | 1.2 KB | 120.46 µs cold, 83 ns warm; 2.67 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.38 ms | 372.64 µs | 26.97 ms | 27.01 ms | 4.8 MB | 981 B | 5.10 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 47.26 ms | 614.38 µs | 46.43 ms | 47.65 ms | 1.9 MB | 981 B | 2.42 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.53 ms | 14.96 µs | 1.50 ms | 1.69 ms | 2.2 MB | 981 B | 2.46 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.43 ms | 463.15 µs | 62.75 ms | 63.87 ms | 48.5 MB | 1.0 KB | 18.91 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.59 ms | 194.34 µs | 1.44 ms | 3.35 ms | 52.9 MB | 1.0 KB | 19.23 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.68 ms | 18.90 µs | 1.64 ms | 3.30 ms | 52.8 MB | 1.0 KB | 18.59 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.06 ms | 474.64 µs | 83.39 ms | 84.57 ms | 2.8 MB | 629 B | 96.79 µs cold, 167 ns warm; 2.70 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 88.22 ms | 812.68 µs | 86.93 ms | 88.88 ms | 4.8 MB | 523 B | 5.06 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.16 ms | 376.92 µs | 52.72 ms | 51.79 ms | 1.9 MB | 523 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.17 ms | 30.99 µs | 6.13 ms | 6.20 ms | 2.1 MB | 523 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Purpose | 145.36 ms | 450.04 µs | 144.60 ms | 144.60 ms | 43.3 MB | 568 B | 18.86 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 19.60 µs | 3.64 ms | 4.55 ms | 49.4 MB | 568 B | 18.81 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.68 ms | 36.34 µs | 3.64 ms | 4.47 ms | 49.3 MB | 568 B | 18.48 ms as a whole process |  |
| Seal | Normal | 101.93 ms | 79.91 µs | 101.79 ms | 102.28 ms | 3.0 MB | 1.8 KB | 144.38 µs cold, 291 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 210.11 ms | 1.83 ms | 205.02 ms | 211.89 ms | 4.8 MB | 1.7 KB | 5.05 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 148.01 ms | 836.51 µs | 146.55 ms | 146.41 ms | 1.9 MB | 1.7 KB | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 49.47 ms | 303.33 µs | 49.07 ms | 48.94 ms | 2.3 MB | 1.7 KB | 2.40 ms as a whole process |  |
| Node `--jitless` | Normal | 157.56 ms | 474.90 µs | 157.04 ms | 159.34 ms | 45.5 MB | 1.7 KB | 18.63 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.58 ms | 50.18 µs | 14.50 ms | 16.07 ms | 52.0 MB | 1.7 KB | 18.20 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.13 ms | 95.82 µs | 13.95 ms | 15.85 ms | 52.1 MB | 1.7 KB | 18.29 ms as a whole process |  |
| Seal | Ceil.source | 102.64 ms | 268.46 µs | 102.16 ms | 102.29 ms | 3.0 MB | 1.8 KB | 140.46 µs cold, 292 ns warm; 2.79 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 105.25 ms | 227.01 µs | 104.84 ms | 105.25 ms | 3.0 MB | 1.8 KB | 145.04 µs cold, 250 ns warm; 2.62 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 99.20 ms | 144.88 µs | 98.95 ms | 99.49 ms | 2.7 MB | 1.8 KB | 143.04 µs cold, 208 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 104.34 ms | 141.05 µs | 104.10 ms | 104.22 ms | 2.8 MB | 1.8 KB | 118.12 µs cold, 209 ns warm; 2.63 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 208.93 ms | 2.33 ms | 202.35 ms | 210.07 ms | 4.8 MB | 1.7 KB | 4.89 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 146.37 ms | 321.17 µs | 145.88 ms | 144.99 ms | 1.9 MB | 1.7 KB | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.56 ms | 303.15 µs | 48.00 ms | 48.66 ms | 2.3 MB | 1.7 KB | 2.50 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 156.50 ms | 514.19 µs | 155.63 ms | 157.51 ms | 45.4 MB | 1.7 KB | 18.34 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.63 ms | 51.01 µs | 14.53 ms | 16.26 ms | 52.0 MB | 1.7 KB | 18.96 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.27 ms | 78.00 µs | 14.14 ms | 15.74 ms | 52.1 MB | 1.7 KB | 18.79 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 126.05 ms | 571.68 µs | 125.37 ms | 125.79 ms | 4.0 MB | 566 B | 103.75 µs cold, 333 ns warm; 2.69 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.39 ms | 102.98 µs | 63.30 ms | 63.32 ms | 4.8 MB | 540 B | 4.97 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 28.64 ms | 100.55 µs | 28.48 ms | 28.77 ms | 2.1 MB | 540 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 5.21 ms | 15.04 µs | 5.19 ms | 5.31 ms | 2.3 MB | 540 B | 2.39 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.67 ms | 979.68 µs | 242.61 ms | 244.87 ms | 48.6 MB | 514 B | 19.42 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 87.16 ms | 23.85 ms | 65.29 ms | 90.08 ms | 52.6 MB | 514 B | 18.31 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 64.25 ms | 465.92 µs | 63.65 ms | 87.82 ms | 52.5 MB | 514 B | 18.72 ms as a whole process |  |
| Seal | Normal | 26.29 ms | 172.97 µs | 25.97 ms | 26.28 ms | 4.0 MB | 535 B | 99.58 µs cold, 292 ns warm; 2.59 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.37 ms | 190.30 µs | 29.95 ms | 30.39 ms | 5.9 MB | 478 B | 5.10 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 32.21 ms | 170.97 µs | 31.96 ms | 32.18 ms | 3.0 MB | 478 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.70 ms | 608.50 µs | 11.84 ms | 12.35 ms | 3.3 MB | 478 B | 2.32 ms as a whole process |  |
| Node `--jitless` | Normal | 52.86 ms | 428.08 µs | 52.24 ms | 54.17 ms | 56.3 MB | 477 B | 18.33 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.87 ms | 198.70 µs | 35.63 ms | 36.26 ms | 62.4 MB | 477 B | 18.67 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.15 ms | 630.06 µs | 34.40 ms | 36.27 ms | 88.3 MB | 477 B | 18.48 ms as a whole process |  |
| Seal | Ceil.source | 26.16 ms | 69.32 µs | 26.05 ms | 26.32 ms | 4.0 MB | 535 B | 103.50 µs cold, 334 ns warm; 2.68 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.39 ms | 89.01 µs | 27.28 ms | 26.90 ms | 4.0 MB | 535 B | 111.29 µs cold, 292 ns warm; 2.46 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.55 ms | 62.36 µs | 23.45 ms | 23.80 ms | 3.9 MB | 535 B | 90.46 µs cold, 291 ns warm; 2.60 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.30 ms | 67.99 µs | 23.22 ms | 23.34 ms | 3.9 MB | 535 B | 96.00 µs cold, 250 ns warm; 2.71 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.33 ms | 818.13 µs | 29.74 ms | 30.18 ms | 6.0 MB | 478 B | 4.86 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.89 ms | 273.06 µs | 33.45 ms | 33.68 ms | 3.0 MB | 478 B | 2.46 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.27 ms | 382.87 µs | 11.70 ms | 13.14 ms | 3.3 MB | 478 B | 2.40 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.60 ms | 506.78 µs | 50.78 ms | 52.85 ms | 56.5 MB | 477 B | 18.65 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.25 ms | 215.40 µs | 34.98 ms | 35.77 ms | 62.5 MB | 477 B | 18.52 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.53 ms | 253.07 µs | 35.12 ms | 36.33 ms | 88.2 MB | 477 B | 18.65 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 310.14 µs | 8.96 µs | 304.29 µs | 322.42 µs | 3.0 MB | 230 B | 70.75 µs cold, 333 ns warm; 2.56 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.28 ms | 316.73 µs | 143.78 ms | 145.89 ms | 9.7 MB | 195 B | 5.08 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.60 ms | 250.57 µs | 32.31 ms | 31.53 ms | 3.5 MB | 195 B | 2.39 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.80 ms | 289.24 µs | 20.58 ms | 20.90 ms | 3.7 MB | 195 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Purpose | 361.62 µs | 140.75 µs | 245.38 µs | 1.03 ms | 52.4 MB | 223 B | 18.45 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 213.69 µs | 150.94 µs | 85.62 µs | 868.92 µs | 57.7 MB | 223 B | 18.51 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 134.21 µs | 121.78 µs | 55.92 µs | 900.38 µs | 57.8 MB | 223 B | 18.89 ms as a whole process |  |
| Seal | Normal | 343.94 µs | 5.97 µs | 336.00 µs | 353.67 µs | 3.2 MB | 179 B | 71.38 µs cold, 291 ns warm; 2.64 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.36 ms | 999.77 µs | 158.90 ms | 161.08 ms | 11.3 MB | 138 B | 5.02 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.71 ms | 239.69 µs | 41.26 ms | 40.78 ms | 5.1 MB | 138 B | 2.40 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.79 ms | 185.78 µs | 19.43 ms | 20.10 ms | 5.2 MB | 138 B | 2.27 ms as a whole process |  |
| Node `--jitless` | Normal | 517.78 µs | 336.53 µs | 264.42 µs | 1.12 ms | 61.5 MB | 155 B | 18.33 ms as a whole process |  |
| Node (JIT), from cold | Normal | 484.77 µs | 479.00 µs | 124.96 µs | 1.05 ms | 65.2 MB | 155 B | 18.65 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 287.25 µs | 327.58 µs | 132.71 µs | 1.04 ms | 69.3 MB | 155 B | 18.84 ms as a whole process |  |
| Seal | Ceil.source | 345.52 µs | 8.53 µs | 336.42 µs | 377.83 µs | 3.2 MB | 179 B | 71.75 µs cold, 291 ns warm; 2.59 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 345.25 µs | 5.13 µs | 339.58 µs | 367.04 µs | 3.2 MB | 179 B | 73.25 µs cold, 333 ns warm; 2.47 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 290.60 µs | 3.77 µs | 286.42 µs | 317.38 µs | 3.0 MB | 179 B | 58.58 µs cold, 250 ns warm; 2.50 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 295.35 µs | 9.68 µs | 284.33 µs | 326.21 µs | 3.0 MB | 179 B | 61.08 µs cold, 250 ns warm; 2.33 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 160.37 ms | 938.98 µs | 159.02 ms | 161.82 ms | 11.3 MB | 138 B | 4.96 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.64 ms | 277.21 µs | 41.13 ms | 40.82 ms | 4.9 MB | 138 B | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.57 ms | 252.45 µs | 19.28 ms | 20.11 ms | 5.1 MB | 138 B | 2.52 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 506.01 µs | 335.39 µs | 259.54 µs | 1.14 ms | 61.4 MB | 155 B | 17.90 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 489.13 µs | 483.17 µs | 120.71 µs | 1.07 ms | 65.3 MB | 155 B | 18.66 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 293.93 µs | 342.63 µs | 134.17 µs | 1.06 ms | 69.2 MB | 155 B | 18.07 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 73.44 ms | 646.59 µs | 72.07 ms | 74.20 ms | 2.9 MB | 528 B | 108.21 µs cold, 250 ns warm; 2.63 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 95.49 ms | 187.62 µs | 95.08 ms | 95.07 ms | 4.8 MB | 411 B | 4.91 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 118.28 ms | 911.30 µs | 116.81 ms | 117.95 ms | 2.5 MB | 411 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.18 ms | 470.85 µs | 93.50 ms | 94.56 ms | 2.9 MB | 411 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.11 ms | 320.35 µs | 75.36 ms | 78.32 ms | 59.6 MB | 444 B | 18.33 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.48 ms | 269.16 µs | 10.20 ms | 13.02 ms | 65.7 MB | 444 B | 18.44 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.42 ms | 617.70 µs | 10.07 ms | 12.91 ms | 114.7 MB | 444 B | 18.59 ms as a whole process |  |
| Seal | Normal | 98.25 ms | 687.59 µs | 97.03 ms | 102.06 ms | 7.8 MB | 977 B | 127.00 µs cold, 209 ns warm; 2.61 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 131.28 ms | 1.76 ms | 129.49 ms | 130.39 ms | 38.7 MB | 799 B | 5.02 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 127.96 ms | 888.25 µs | 126.78 ms | 127.12 ms | 38.3 MB | 799 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 83.35 ms | 1.25 ms | 81.49 ms | 81.82 ms | 44.2 MB | 799 B | 2.54 ms as a whole process |  |
| Node `--jitless` | Normal | 116.60 ms | 1.67 ms | 115.38 ms | 117.17 ms | 213.2 MB | 878 B | 18.71 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.21 ms | 885.01 µs | 13.31 ms | 16.98 ms | 129.7 MB | 878 B | 18.42 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.40 ms | 87.54 µs | 13.23 ms | 17.05 ms | 224.1 MB | 878 B | 18.97 ms as a whole process |  |
| Seal | Ceil.source | 98.83 ms | 798.26 µs | 97.52 ms | 100.40 ms | 7.8 MB | 977 B | 120.08 µs cold, 250 ns warm; 2.71 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 100.88 ms | 397.84 µs | 100.23 ms | 100.33 ms | 7.8 MB | 977 B | 127.33 µs cold, 250 ns warm; 2.63 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 85.59 ms | 1.33 ms | 84.12 ms | 88.16 ms | 7.6 MB | 977 B | 96.33 µs cold, 208 ns warm; 2.67 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 85.21 ms | 219.49 µs | 84.95 ms | 85.25 ms | 7.6 MB | 977 B | 101.96 µs cold, 167 ns warm; 2.67 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 129.02 ms | 960.42 µs | 127.71 ms | 133.79 ms | 37.7 MB | 799 B | 4.88 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.10 ms | 1.15 ms | 126.24 ms | 127.22 ms | 41.5 MB | 799 B | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 82.65 ms | 975.11 µs | 81.16 ms | 83.21 ms | 44.4 MB | 799 B | 2.51 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.88 ms | 2.49 ms | 112.79 ms | 114.88 ms | 213.3 MB | 878 B | 18.87 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.15 ms | 858.42 µs | 13.39 ms | 16.89 ms | 129.6 MB | 878 B | 18.99 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.46 ms | 87.82 µs | 13.29 ms | 16.98 ms | 224.0 MB | 878 B | 19.24 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 50.33 µs (±6.82 µs, least 46.08 µs) | 16.78 µs (±4.30 µs, least 13.04 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.94 µs (±1.18 µs, least 8.79 µs) | 16.67 µs (±1.86 µs, least 14.42 µs) | 42 ns (±2 ns, least 40 ns) | 21 ns (±1 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.90 µs (±568 ns, least 3.00 µs) | 7.80 µs (±1.81 µs, least 5.00 µs) | 21 ns (±1 ns, least 20 ns) | 14 ns (±1 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.50 µs (±527 ns, least 3.00 µs) | 6.90 µs (±1.37 µs, least 5.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 41.02 µs (±14.09 µs, least 35.33 µs) | 38.02 µs (±1.50 µs, least 35.42 µs) | 57 ns (±1 ns, least 56 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 28.38 µs (±7.25 µs, least 18.62 µs) | 37.51 µs (±1.84 µs, least 34.62 µs) | 9 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.50 µs (±62 ns, least 4.41 µs) | 153 ns (±12 ns, least 133 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.85 µs (±118 ns, least 4.72 µs) | 186 ns (±10 ns, least 176 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.79 µs (±29 ns, least 1.77 µs) | 178 ns (±6 ns, least 171 ns) | 28 ns (±1 ns, least 27 ns) | 14 ns (±1 ns, least 12 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.83 µs (±18 ns, least 1.80 µs) | 348 ns (±9 ns, least 334 ns) | 94 ns (±3 ns, least 90 ns) | 109 ns (±12 ns, least 100 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±27 ns, least 1.12 µs) | 669 ns (±23 ns, least 640 ns) | 50 ns (±1 ns, least 48 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.09 µs (±34 ns, least 1.04 µs) | 666 ns (±20 ns, least 634 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 6 ns) | 643 B | 55.8 MB | the engine's own heap |
| Seal | Ceil.source | 4.46 µs (±39 ns, least 4.41 µs) | 152 ns (±8 ns, least 138 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.49 µs (±57 ns, least 4.42 µs) | 157 ns (±6 ns, least 149 ns) | 21 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 3.86 µs (±45 ns, least 3.81 µs) | 145 ns (±6 ns, least 129 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 3.93 µs (±42 ns, least 3.87 µs) | 141 ns (±8 ns, least 128 ns) | 18 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.80 µs (±98 ns, least 4.61 µs) | 182 ns (±9 ns, least 170 ns) | 33 ns (±0 ns, least 32 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.78 µs (±19 ns, least 1.74 µs) | 181 ns (±6 ns, least 174 ns) | 27 ns (±1 ns, least 27 ns) | 16 ns (±3 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.81 µs (±15 ns, least 1.79 µs) | 366 ns (±33 ns, least 337 ns) | 93 ns (±3 ns, least 90 ns) | 108 ns (±14 ns, least 100 ns) | 1.0 KB | 4.8 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±24 ns, least 1.10 µs) | 670 ns (±22 ns, least 624 ns) | 51 ns (±1 ns, least 50 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±18 ns, least 1.07 µs) | 679 ns (±18 ns, least 648 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.8 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 32.27 ms | 32.86 ms (+1.8%) |
| n-body, 100,000 steps | 65.58 ms | 61.68 ms (-5.9%) |
| spectral-norm, size 200 | 18.68 ms | 18.23 ms (-2.4%) |
| fannkuch-redux, 9 | 102.26 ms | 102.70 ms (+0.4%) |
| word count, 500,000 words | 26.24 ms | 26.52 ms (+1.1%) |
| text building, 20,000 joins | 341.84 µs | 339.30 µs (-0.7%) |
| binary-trees, depth 14 | 99.95 ms | 101.22 ms (+1.3%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 170.69 ms | 690.25 µs | 169.88 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.45 ms | 95.65 µs | 10.29 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.50 ms | 15.39 µs | 4.47 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.42 ms | 13.82 µs | 1.40 ms | 3.6 MB |
