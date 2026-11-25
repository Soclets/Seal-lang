# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.19 2.25 2.08 }` before, `{ 2.18 2.26 2.11 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 33.71 ms | 71.03 ms | **met**: Seal takes 47% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.39 ms | 79.72 ms | **met**: Seal takes 83% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.77 ms | 28.77 ms | **met**: Seal takes 65% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 103.15 ms | 210.63 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.59 ms | 30.57 ms | **met**: Seal takes 87% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 354.86 µs | 162.71 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 101.22 ms | 140.93 ms | **met**: Seal takes 72% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.24 ms | 3.58 ms | **met**: Seal takes 63% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 21 ns | 34 ns | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 155 ns | 184 ns | **met**: Seal takes 84% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.60 µs | 4.97 µs | **met**: Seal takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 33.71 ms | 34.04 ms | **met**: Seal takes 99% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 66.39 ms | 72.23 ms | **met**: Seal takes 92% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.77 ms | 44.50 ms | **met**: Seal takes 42% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 103.15 ms | 147.08 ms | **met**: Seal takes 70% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.59 ms | 31.69 ms | **met**: Seal takes 84% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 354.86 µs | 42.27 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 101.22 ms | 132.17 ms | **met**: Seal takes 77% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.24 ms | 3.04 ms | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 21 ns | 28 ns | **met**: Seal takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 155 ns | 186 ns | **met**: Seal takes 83% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.60 µs | 1.85 µs | for reference, no target (D150) |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 33.71 ms | 95.67 ms | **met**: Seal takes 35% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.39 ms | 181.53 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.59 ms | 53.55 ms | **met**: Seal takes 50% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 354.86 µs | 521.79 µs | **met**: Seal takes 68% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 21 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.58 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 85.45 ms | 927.44 µs | 84.46 ms | 100.64 ms | 2.8 MB | 386 B | 675.75 µs cold, 500 ns warm; 2.67 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 196.73 ms | 723.68 µs | 195.94 ms | 198.09 ms | 4.8 MB | 339 B | 5.10 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 89.82 ms | 1.02 ms | 88.61 ms | 89.43 ms | 1.9 MB | 339 B | 2.57 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.68 ms | 161.44 µs | 44.46 ms | 45.88 ms | 2.2 MB | 339 B | 2.62 ms as a whole process |  |
| Node `--jitless` | Purpose | 322.44 ms | 753.95 µs | 321.58 ms | 324.35 ms | 43.3 MB | 362 B | 18.81 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.97 ms | 930.13 µs | 122.08 ms | 125.05 ms | 49.2 MB | 362 B | 18.90 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 122.34 ms | 545.22 µs | 121.62 ms | 122.79 ms | 49.1 MB | 362 B | 19.07 ms as a whole process |  |
| Seal | Normal | 33.71 ms | 655.33 µs | 32.35 ms | 33.86 ms | 2.7 MB | 160 B | 74.88 µs cold, 209 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 71.03 ms | 281.11 µs | 70.61 ms | 72.21 ms | 4.8 MB | 143 B | 5.04 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 34.04 ms | 328.22 µs | 33.59 ms | 34.03 ms | 1.9 MB | 143 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.09 ms | 21.58 µs | 7.05 ms | 7.40 ms | 2.2 MB | 143 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Normal | 95.67 ms | 372.18 µs | 95.19 ms | 95.49 ms | 43.4 MB | 137 B | 18.63 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.55 ms | 80.32 µs | 9.47 ms | 11.36 ms | 48.5 MB | 137 B | 19.15 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.89 ms | 210.20 µs | 9.63 ms | 11.70 ms | 48.6 MB | 137 B | 19.43 ms as a whole process |  |
| Seal | Ceil.source | 33.79 ms | 556.49 µs | 32.52 ms | 33.24 ms | 2.7 MB | 160 B | 73.08 µs cold, 125 ns warm; 2.64 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 34.52 ms | 812.62 µs | 33.09 ms | 59.11 ms | 2.7 MB | 160 B | 1.10 ms cold, 583 ns warm; 2.67 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 33.86 ms | 669.70 µs | 32.69 ms | 57.38 ms | 2.4 MB | 160 B | 658.17 µs cold, 500 ns warm; 2.67 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 31.54 ms | 1.27 ms | 29.63 ms | 52.01 ms | 2.5 MB | 160 B | 779.50 µs cold, 417 ns warm; 2.73 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.41 ms | 244.38 µs | 71.14 ms | 71.67 ms | 4.8 MB | 143 B | 5.19 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.90 ms | 112.18 µs | 33.71 ms | 33.93 ms | 1.9 MB | 143 B | 2.54 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.44 ms | 94.79 µs | 7.36 ms | 7.49 ms | 2.1 MB | 143 B | 2.53 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.42 ms | 565.34 µs | 94.29 ms | 96.71 ms | 43.4 MB | 137 B | 18.81 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.55 ms | 219.14 µs | 9.44 ms | 10.33 ms | 48.6 MB | 137 B | 19.23 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.88 ms | 705.46 µs | 9.51 ms | 10.68 ms | 48.6 MB | 137 B | 20.28 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.16 ms | 666.81 µs | 56.40 ms | 57.34 ms | 2.7 MB | 547 B | 108.46 µs cold, 209 ns warm; 2.76 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.01 ms | 272.94 µs | 91.54 ms | 91.75 ms | 4.8 MB | 433 B | 5.33 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.17 ms | 331.89 µs | 76.79 ms | 76.23 ms | 1.8 MB | 433 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.74 ms | 64.27 µs | 6.62 ms | 6.77 ms | 2.1 MB | 433 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Purpose | 246.71 ms | 1.24 ms | 245.13 ms | 245.17 ms | 46.4 MB | 460 B | 19.92 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.95 ms | 226.28 µs | 18.59 ms | 19.00 ms | 50.3 MB | 460 B | 20.37 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.93 ms | 254.64 µs | 18.67 ms | 19.03 ms | 50.2 MB | 460 B | 19.96 ms as a whole process |  |
| Seal | Normal | 66.39 ms | 1.05 ms | 65.04 ms | 70.15 ms | 3.3 MB | 3.5 KB | 292.25 µs cold, 250 ns warm; 3.02 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 79.72 ms | 24.53 ms | 70.56 ms | 70.72 ms | 4.8 MB | 3.0 KB | 5.27 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 72.23 ms | 327.31 µs | 71.70 ms | 72.24 ms | 1.9 MB | 3.0 KB | 2.57 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.32 ms | 24.14 µs | 5.28 ms | 5.56 ms | 2.2 MB | 3.0 KB | 2.72 ms as a whole process |  |
| Node `--jitless` | Normal | 181.53 ms | 2.03 ms | 176.96 ms | 186.28 ms | 48.2 MB | 3.1 KB | 20.62 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.93 ms | 27.68 µs | 3.89 ms | 5.80 ms | 51.8 MB | 3.1 KB | 19.17 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.97 ms | 24.11 µs | 3.94 ms | 5.88 ms | 51.9 MB | 3.1 KB | 21.32 ms as a whole process |  |
| Seal | Ceil.source | 57.01 ms | 1.11 ms | 55.17 ms | 60.48 ms | 3.2 MB | 3.4 KB | 273.67 µs cold, 417 ns warm; 2.91 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.77 ms | 791.53 µs | 54.71 ms | 57.55 ms | 3.1 MB | 3.4 KB | 263.96 µs cold, 292 ns warm; 2.88 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.05 ms | 406.42 µs | 49.63 ms | 50.65 ms | 2.9 MB | 3.4 KB | 293.92 µs cold, 208 ns warm; 2.97 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.15 ms | 366.50 µs | 49.52 ms | 50.89 ms | 3.0 MB | 3.4 KB | 276.62 µs cold, 208 ns warm; 2.97 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 70.48 ms | 451.27 µs | 69.71 ms | 70.49 ms | 4.8 MB | 3.0 KB | 5.44 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 72.24 ms | 636.48 µs | 71.71 ms | 71.96 ms | 1.9 MB | 3.0 KB | 2.71 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.22 ms | 25.56 µs | 5.17 ms | 5.46 ms | 2.2 MB | 3.0 KB | 2.55 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.63 ms | 3.64 ms | 171.99 ms | 179.18 ms | 48.1 MB | 3.1 KB | 19.80 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.95 ms | 40.61 µs | 3.87 ms | 5.99 ms | 52.1 MB | 3.1 KB | 20.47 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.92 ms | 25.19 µs | 3.89 ms | 5.87 ms | 51.9 MB | 3.1 KB | 19.73 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.68 ms | 898.86 µs | 53.29 ms | 55.72 ms | 2.9 MB | 426 B | 107.25 µs cold, 250 ns warm; 2.63 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.79 ms | 414.66 µs | 45.35 ms | 45.82 ms | 4.8 MB | 366 B | 5.22 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.83 ms | 455.40 µs | 77.42 ms | 77.88 ms | 1.9 MB | 366 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.85 ms | 308.23 µs | 22.53 ms | 23.03 ms | 2.1 MB | 366 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Purpose | 374.40 ms | 6.82 ms | 365.09 ms | 380.91 ms | 48.5 MB | 398 B | 19.86 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.22 ms | 37.48 ms | 15.63 ms | 19.33 ms | 52.8 MB | 398 B | 19.06 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.43 ms | 52.32 µs | 14.38 ms | 23.66 ms | 55.3 MB | 398 B | 18.99 ms as a whole process |  |
| Seal | Normal | 18.77 ms | 121.53 µs | 18.59 ms | 19.13 ms | 3.0 MB | 1.2 KB | 138.67 µs cold, 167 ns warm; 2.76 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 28.77 ms | 498.16 µs | 27.86 ms | 29.14 ms | 4.8 MB | 981 B | 5.24 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 44.50 ms | 224.51 µs | 44.15 ms | 44.41 ms | 2.0 MB | 981 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.53 ms | 20.19 µs | 1.51 ms | 1.72 ms | 2.2 MB | 981 B | 2.37 ms as a whole process |  |
| Node `--jitless` | Normal | 63.87 ms | 247.07 µs | 63.58 ms | 64.15 ms | 48.6 MB | 1.0 KB | 18.76 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.61 ms | 169.26 µs | 1.48 ms | 3.11 ms | 52.7 MB | 1.0 KB | 18.88 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.61 ms | 13.57 µs | 1.59 ms | 3.18 ms | 52.8 MB | 1.0 KB | 19.30 ms as a whole process |  |
| Seal | Ceil.source | 19.03 ms | 535.46 µs | 18.12 ms | 17.98 ms | 3.0 MB | 1.2 KB | 142.88 µs cold, 250 ns warm; 2.88 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.90 ms | 151.95 µs | 18.63 ms | 19.05 ms | 3.0 MB | 1.2 KB | 161.42 µs cold, 291 ns warm; 2.90 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.68 ms | 92.73 µs | 18.57 ms | 18.68 ms | 2.8 MB | 1.2 KB | 119.17 µs cold, 166 ns warm; 2.79 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.93 ms | 57.30 µs | 18.86 ms | 18.95 ms | 2.9 MB | 1.2 KB | 134.75 µs cold, 209 ns warm; 2.79 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.68 ms | 334.32 µs | 27.37 ms | 27.51 ms | 4.8 MB | 981 B | 5.23 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 43.51 ms | 488.27 µs | 42.93 ms | 43.57 ms | 2.0 MB | 981 B | 2.57 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.55 ms | 26.90 µs | 1.51 ms | 1.69 ms | 2.2 MB | 981 B | 2.48 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.24 ms | 349.90 µs | 63.74 ms | 64.56 ms | 48.3 MB | 1.0 KB | 19.76 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.65 ms | 224.99 µs | 1.48 ms | 3.35 ms | 52.8 MB | 1.0 KB | 18.98 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.66 ms | 43.69 µs | 1.60 ms | 4.16 ms | 52.9 MB | 1.0 KB | 18.91 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 85.09 ms | 265.17 µs | 84.64 ms | 86.52 ms | 2.8 MB | 629 B | 100.38 µs cold, 250 ns warm; 2.78 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 90.30 ms | 1.37 ms | 88.25 ms | 88.05 ms | 4.8 MB | 523 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.93 ms | 472.18 µs | 52.25 ms | 52.69 ms | 1.9 MB | 523 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.18 ms | 18.48 µs | 6.15 ms | 6.24 ms | 2.1 MB | 523 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Purpose | 147.24 ms | 949.99 µs | 145.90 ms | 145.70 ms | 43.4 MB | 568 B | 18.84 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.66 ms | 32.68 µs | 3.63 ms | 4.48 ms | 49.4 MB | 568 B | 18.86 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.65 ms | 28.41 µs | 3.62 ms | 4.44 ms | 49.4 MB | 568 B | 19.22 ms as a whole process |  |
| Seal | Normal | 103.15 ms | 260.95 µs | 102.73 ms | 103.40 ms | 3.0 MB | 1.8 KB | 149.00 µs cold, 291 ns warm; 2.82 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 210.63 ms | 1.69 ms | 206.03 ms | 211.54 ms | 4.8 MB | 1.7 KB | 5.09 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.08 ms | 216.44 µs | 146.68 ms | 145.52 ms | 1.9 MB | 1.7 KB | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 49.27 ms | 518.86 µs | 48.02 ms | 47.98 ms | 2.3 MB | 1.7 KB | 2.49 ms as a whole process |  |
| Node `--jitless` | Normal | 156.62 ms | 782.98 µs | 155.92 ms | 156.37 ms | 45.4 MB | 1.7 KB | 18.76 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.09 ms | 155.79 µs | 13.83 ms | 15.69 ms | 52.1 MB | 1.7 KB | 19.31 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.87 ms | 56.05 µs | 14.78 ms | 16.36 ms | 52.1 MB | 1.7 KB | 18.61 ms as a whole process |  |
| Seal | Ceil.source | 103.04 ms | 1.68 ms | 101.93 ms | 104.05 ms | 3.0 MB | 1.8 KB | 145.12 µs cold, 208 ns warm; 2.84 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 106.61 ms | 702.02 µs | 105.36 ms | 105.62 ms | 3.0 MB | 1.8 KB | 140.58 µs cold, 292 ns warm; 2.80 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 99.37 ms | 213.50 µs | 98.99 ms | 99.15 ms | 2.7 MB | 1.8 KB | 119.92 µs cold, 250 ns warm; 2.65 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.99 ms | 3.10 ms | 102.83 ms | 102.89 ms | 2.8 MB | 1.8 KB | 123.75 µs cold, 250 ns warm; 2.91 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 213.50 ms | 2.04 ms | 208.49 ms | 214.97 ms | 4.8 MB | 1.7 KB | 5.42 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 147.49 ms | 336.30 µs | 147.02 ms | 145.89 ms | 1.9 MB | 1.7 KB | 2.67 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.76 ms | 217.91 µs | 48.30 ms | 49.14 ms | 2.3 MB | 1.7 KB | 2.53 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.27 ms | 1.47 ms | 155.31 ms | 158.60 ms | 45.4 MB | 1.7 KB | 19.62 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 15.07 ms | 409.89 µs | 14.36 ms | 15.67 ms | 51.9 MB | 1.7 KB | 19.17 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.25 ms | 61.20 µs | 14.15 ms | 15.71 ms | 52.2 MB | 1.7 KB | 18.77 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 125.55 ms | 1.04 ms | 123.82 ms | 126.15 ms | 4.0 MB | 566 B | 100.46 µs cold, 333 ns warm; 2.77 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.02 ms | 1.22 ms | 62.24 ms | 63.03 ms | 4.8 MB | 540 B | 5.19 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 34.42 ms | 1.95 ms | 31.86 ms | 32.13 ms | 2.1 MB | 540 B | 2.38 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.75 ms | 60.25 µs | 4.68 ms | 4.90 ms | 2.3 MB | 540 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.26 ms | 1.36 ms | 241.74 ms | 244.37 ms | 48.6 MB | 514 B | 20.57 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 91.74 ms | 24.20 ms | 68.75 ms | 95.24 ms | 52.6 MB | 514 B | 18.99 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 65.04 ms | 780.06 µs | 63.47 ms | 88.57 ms | 52.7 MB | 514 B | 19.91 ms as a whole process |  |
| Seal | Normal | 26.59 ms | 148.82 µs | 26.33 ms | 26.79 ms | 4.0 MB | 535 B | 106.38 µs cold, 292 ns warm; 2.68 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.57 ms | 228.05 µs | 30.25 ms | 30.32 ms | 6.0 MB | 478 B | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.69 ms | 451.81 µs | 30.95 ms | 29.75 ms | 3.0 MB | 478 B | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.43 ms | 409.95 µs | 11.85 ms | 12.28 ms | 3.3 MB | 478 B | 3.43 ms as a whole process |  |
| Node `--jitless` | Normal | 53.55 ms | 454.74 µs | 52.59 ms | 114.33 ms | 56.5 MB | 477 B | 19.63 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.50 ms | 421.48 µs | 35.06 ms | 36.40 ms | 62.5 MB | 477 B | 18.82 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 34.65 ms | 228.77 µs | 34.31 ms | 35.97 ms | 88.1 MB | 477 B | 18.90 ms as a whole process |  |
| Seal | Ceil.source | 25.96 ms | 199.42 µs | 25.58 ms | 25.63 ms | 4.0 MB | 535 B | 102.42 µs cold, 334 ns warm; 2.56 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.01 ms | 75.64 µs | 26.88 ms | 27.08 ms | 4.0 MB | 535 B | 108.46 µs cold, 292 ns warm; 2.60 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.03 ms | 177.34 µs | 23.83 ms | 23.89 ms | 3.9 MB | 535 B | 101.04 µs cold, 250 ns warm; 2.74 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.66 ms | 72.65 µs | 23.53 ms | 23.72 ms | 4.0 MB | 535 B | 85.46 µs cold, 250 ns warm; 2.61 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.27 ms | 244.98 µs | 29.85 ms | 29.95 ms | 6.0 MB | 478 B | 4.94 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.68 ms | 576.11 µs | 31.03 ms | 31.54 ms | 3.0 MB | 478 B | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.09 ms | 364.23 µs | 11.66 ms | 12.40 ms | 3.3 MB | 478 B | 2.46 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.16 ms | 266.76 µs | 51.79 ms | 52.96 ms | 56.4 MB | 477 B | 18.86 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.42 ms | 445.32 µs | 34.86 ms | 35.81 ms | 62.3 MB | 477 B | 19.22 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 34.87 ms | 253.86 µs | 34.50 ms | 35.66 ms | 88.3 MB | 477 B | 18.37 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 319.01 µs | 10.82 µs | 312.38 µs | 338.96 µs | 3.0 MB | 230 B | 68.12 µs cold, 334 ns warm; 2.39 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.26 ms | 382.44 µs | 143.68 ms | 146.20 ms | 10.5 MB | 195 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.57 ms | 148.01 µs | 32.33 ms | 31.54 ms | 3.5 MB | 195 B | 2.43 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.76 ms | 155.21 µs | 20.62 ms | 20.87 ms | 3.7 MB | 195 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 361.70 µs | 145.15 µs | 239.58 µs | 1.03 ms | 52.5 MB | 223 B | 18.69 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 221.35 µs | 158.92 µs | 82.79 µs | 922.12 µs | 57.7 MB | 223 B | 18.24 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 134.43 µs | 120.29 µs | 55.25 µs | 898.71 µs | 57.8 MB | 223 B | 19.13 ms as a whole process |  |
| Seal | Normal | 354.86 µs | 5.71 µs | 350.58 µs | 396.38 µs | 3.2 MB | 179 B | 78.21 µs cold, 292 ns warm; 2.59 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 162.71 ms | 786.77 µs | 161.85 ms | 163.99 ms | 11.7 MB | 138 B | 4.96 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 42.27 ms | 417.54 µs | 41.61 ms | 41.85 ms | 4.9 MB | 138 B | 2.43 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.86 ms | 205.60 µs | 19.55 ms | 20.51 ms | 5.3 MB | 138 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Normal | 521.79 µs | 332.09 µs | 277.38 µs | 1.16 ms | 61.4 MB | 155 B | 18.63 ms as a whole process |  |
| Node (JIT), from cold | Normal | 528.41 µs | 505.78 µs | 137.46 µs | 1.16 ms | 65.1 MB | 155 B | 20.84 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 293.45 µs | 336.60 µs | 132.58 µs | 1.16 ms | 69.4 MB | 155 B | 19.73 ms as a whole process |  |
| Seal | Ceil.source | 356.50 µs | 8.62 µs | 349.33 µs | 373.29 µs | 3.2 MB | 179 B | 75.62 µs cold, 375 ns warm; 2.58 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 347.84 µs | 6.56 µs | 338.71 µs | 373.58 µs | 3.2 MB | 179 B | 77.17 µs cold, 292 ns warm; 2.69 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 293.87 µs | 4.59 µs | 287.12 µs | 322.12 µs | 3.0 MB | 179 B | 70.25 µs cold, 375 ns warm; 2.68 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 295.80 µs | 8.32 µs | 284.79 µs | 309.25 µs | 3.1 MB | 179 B | 57.08 µs cold, 208 ns warm; 2.50 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 166.34 ms | 3.74 ms | 162.97 ms | 164.61 ms | 12.4 MB | 138 B | 5.42 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.53 ms | 168.91 µs | 41.26 ms | 42.87 ms | 4.9 MB | 138 B | 2.62 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.87 ms | 184.77 µs | 19.43 ms | 20.43 ms | 5.2 MB | 138 B | 2.59 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 512.73 µs | 341.55 µs | 271.92 µs | 1.14 ms | 61.3 MB | 155 B | 19.14 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 500.58 µs | 488.12 µs | 125.00 µs | 1.08 ms | 65.1 MB | 155 B | 19.37 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 272.82 µs | 316.59 µs | 122.92 µs | 1.11 ms | 69.4 MB | 155 B | 19.19 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 71.91 ms | 576.90 µs | 70.85 ms | 74.10 ms | 2.9 MB | 528 B | 88.25 µs cold, 333 ns warm; 2.57 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 96.49 ms | 1.34 ms | 95.12 ms | 95.62 ms | 4.8 MB | 411 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 117.95 ms | 1.85 ms | 113.68 ms | 120.35 ms | 2.5 MB | 411 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 99.23 ms | 9.73 ms | 94.92 ms | 94.44 ms | 2.9 MB | 411 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Purpose | 79.42 ms | 1.97 ms | 77.18 ms | 79.74 ms | 59.5 MB | 444 B | 19.81 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.91 ms | 460.06 µs | 10.51 ms | 13.18 ms | 65.7 MB | 444 B | 20.80 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 11.19 ms | 1.17 ms | 10.53 ms | 13.31 ms | 114.7 MB | 444 B | 21.57 ms as a whole process |  |
| Seal | Normal | 101.22 ms | 1.71 ms | 99.59 ms | 103.59 ms | 7.8 MB | 977 B | 124.12 µs cold, 250 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 140.93 ms | 2.37 ms | 135.71 ms | 134.93 ms | 38.7 MB | 799 B | 5.46 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 132.17 ms | 1.45 ms | 130.72 ms | 129.54 ms | 39.1 MB | 799 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 83.91 ms | 914.57 µs | 82.52 ms | 84.92 ms | 39.8 MB | 799 B | 2.66 ms as a whole process |  |
| Node `--jitless` | Normal | 116.39 ms | 3.01 ms | 111.86 ms | 119.65 ms | 213.2 MB | 878 B | 19.90 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.37 ms | 998.29 µs | 13.32 ms | 17.02 ms | 129.7 MB | 878 B | 18.70 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.63 ms | 70.06 µs | 13.54 ms | 17.15 ms | 223.6 MB | 878 B | 19.09 ms as a whole process |  |
| Seal | Ceil.source | 102.29 ms | 8.09 ms | 97.35 ms | 100.95 ms | 7.8 MB | 977 B | 119.75 µs cold, 208 ns warm; 3.50 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 104.52 ms | 1.57 ms | 102.60 ms | 112.92 ms | 7.8 MB | 977 B | 134.54 µs cold, 250 ns warm; 2.59 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 85.54 ms | 847.65 µs | 84.48 ms | 85.19 ms | 7.6 MB | 977 B | 101.29 µs cold, 208 ns warm; 2.80 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 85.08 ms | 994.71 µs | 83.01 ms | 84.35 ms | 7.6 MB | 977 B | 102.00 µs cold, 167 ns warm; 2.86 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 136.18 ms | 2.11 ms | 130.80 ms | 131.83 ms | 41.7 MB | 799 B | 5.38 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 127.61 ms | 1.09 ms | 126.00 ms | 128.71 ms | 42.7 MB | 799 B | 2.50 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 81.58 ms | 770.14 µs | 80.17 ms | 81.37 ms | 40.4 MB | 799 B | 2.60 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.97 ms | 2.00 ms | 114.19 ms | 120.12 ms | 213.3 MB | 878 B | 19.34 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.29 ms | 947.13 µs | 13.41 ms | 17.05 ms | 129.7 MB | 878 B | 19.07 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.46 ms | 75.51 µs | 13.27 ms | 17.01 ms | 224.0 MB | 878 B | 19.13 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 50.22 µs (±6.09 µs, least 44.79 µs) | 12.82 µs (±2.09 µs, least 10.29 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.69 µs (±997 ns, least 8.83 µs) | 16.73 µs (±1.22 µs, least 14.88 µs) | 41 ns (±1 ns, least 39 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 4.10 µs (±876 ns, least 3.00 µs) | 6.70 µs (±1.25 µs, least 5.00 µs) | 22 ns (±1 ns, least 20 ns) | 14 ns (±1 ns, least 12 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 4.10 µs (±876 ns, least 3.00 µs) | 6.70 µs (±1.25 µs, least 5.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 35.48 µs (±18.96 µs, least 19.67 µs) | 42.42 µs (±9.22 µs, least 35.88 µs) | 57 ns (±2 ns, least 54 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 32.70 µs (±4.99 µs, least 21.62 µs) | 40.92 µs (±4.41 µs, least 36.08 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 4.60 µs (±128 ns, least 4.46 µs) | 155 ns (±9 ns, least 148 ns) | 21 ns (±1 ns, least 20 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.97 µs (±131 ns, least 4.78 µs) | 184 ns (±9 ns, least 171 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.85 µs (±27 ns, least 1.81 µs) | 186 ns (±11 ns, least 175 ns) | 28 ns (±1 ns, least 28 ns) | 15 ns (±1 ns, least 14 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.84 µs (±27 ns, least 1.79 µs) | 383 ns (±40 ns, least 359 ns) | 101 ns (±3 ns, least 96 ns) | 110 ns (±13 ns, least 103 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±20 ns, least 1.14 µs) | 672 ns (±15 ns, least 652 ns) | 52 ns (±4 ns, least 49 ns) | 24 ns (±0 ns, least 24 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.10 µs (±28 ns, least 1.05 µs) | 665 ns (±20 ns, least 636 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.51 µs (±56 ns, least 4.45 µs) | 156 ns (±7 ns, least 147 ns) | 21 ns (±1 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.55 µs (±45 ns, least 4.45 µs) | 150 ns (±7 ns, least 141 ns) | 21 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 3.91 µs (±64 ns, least 3.84 µs) | 144 ns (±11 ns, least 133 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±1 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 3.99 µs (±54 ns, least 3.95 µs) | 145 ns (±5 ns, least 138 ns) | 18 ns (±0 ns, least 18 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.3 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.92 µs (±136 ns, least 4.69 µs) | 184 ns (±7 ns, least 172 ns) | 33 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.80 µs (±20 ns, least 1.76 µs) | 182 ns (±6 ns, least 173 ns) | 28 ns (±1 ns, least 27 ns) | 15 ns (±2 ns, least 14 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.83 µs (±41 ns, least 1.78 µs) | 364 ns (±19 ns, least 346 ns) | 95 ns (±3 ns, least 92 ns) | 110 ns (±14 ns, least 101 ns) | 1.0 KB | 4.8 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.16 µs (±20 ns, least 1.13 µs) | 674 ns (±20 ns, least 643 ns) | 51 ns (±1 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.14 µs (±65 ns, least 1.10 µs) | 674 ns (±34 ns, least 629 ns) | 22 ns (±0 ns, least 22 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 32.99 ms | 34.12 ms (+3.4%) |
| n-body, 100,000 steps | 68.70 ms | 60.29 ms (-12.2%) |
| spectral-norm, size 200 | 19.53 ms | 18.13 ms (-7.1%) |
| fannkuch-redux, 9 | 104.33 ms | 105.68 ms (+1.3%) |
| word count, 500,000 words | 26.65 ms | 26.68 ms (+0.1%) |
| text building, 20,000 joins | 358.09 µs | 353.35 µs (-1.3%) |
| binary-trees, depth 14 | 99.74 ms | 98.65 ms (-1.1%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 169.10 ms | 1.39 ms | 167.01 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.42 ms | 375.68 µs | 10.12 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 3.92 ms | 169.65 µs | 3.71 ms | 3.9 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.33 ms | 41.86 µs | 1.28 ms | 3.6 MB |
