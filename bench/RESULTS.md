# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.19 2.04 1.86 }` before, `{ 2.25 1.99 1.86 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 30.44 ms | 72.57 ms | **met**: Seal takes 42% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 63.91 ms | 69.60 ms | **met**: Seal takes 92% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 17.77 ms | 27.12 ms | **met**: Seal takes 66% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 102.07 ms | 210.75 ms | **met**: Seal takes 48% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.47 ms | 32.46 ms | **met**: Seal takes 82% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 342.64 µs | 161.55 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 99.32 ms | 130.62 ms | **met**: Seal takes 76% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.18 ms | 3.49 ms | **met**: Seal takes 62% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 33 ns | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 157 ns | 185 ns | **met**: Seal takes 85% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.48 µs | 4.91 µs | **met**: Seal takes 91% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 30.44 ms | 34.96 ms | **met**: Seal takes 87% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 63.91 ms | 72.09 ms | **met**: Seal takes 89% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 17.77 ms | 48.09 ms | **met**: Seal takes 37% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 102.07 ms | 147.16 ms | **met**: Seal takes 69% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.47 ms | 32.06 ms | **met**: Seal takes 83% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 342.64 µs | 41.49 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 99.32 ms | 127.73 ms | **met**: Seal takes 78% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.18 ms | 2.92 ms | **met**: Seal takes 75% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 27 ns | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 157 ns | 181 ns | **met**: Seal takes 87% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.48 µs | 1.79 µs | **not met**: Seal takes 250% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 30.44 ms | 96.67 ms | **met**: Seal takes 31% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 63.91 ms | 175.13 ms | **met**: Seal takes 36% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.47 ms | 51.71 ms | **met**: Seal takes 51% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 342.64 µs | 506.95 µs | **met**: Seal takes 68% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.46 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 83.49 ms | 582.99 µs | 82.84 ms | 101.77 ms | 2.8 MB | 386 B | 731.04 µs cold, 458 ns warm; 2.63 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 197.45 ms | 299.44 µs | 196.91 ms | 197.49 ms | 4.8 MB | 339 B | 5.07 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.53 ms | 484.45 µs | 89.75 ms | 89.76 ms | 1.9 MB | 339 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 43.97 ms | 714.90 µs | 42.06 ms | 45.99 ms | 2.2 MB | 339 B | 2.33 ms as a whole process |  |
| Node `--jitless` | Purpose | 320.12 ms | 1.02 ms | 318.14 ms | 320.41 ms | 43.4 MB | 362 B | 18.44 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 123.17 ms | 1.06 ms | 121.87 ms | 125.71 ms | 49.1 MB | 362 B | 18.50 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 121.40 ms | 1.40 ms | 120.18 ms | 121.90 ms | 49.1 MB | 362 B | 18.49 ms as a whole process |  |
| Seal | Normal | 30.44 ms | 384.44 µs | 29.70 ms | 31.02 ms | 2.7 MB | 160 B | 74.54 µs cold, 250 ns warm; 2.66 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 72.57 ms | 873.37 µs | 71.52 ms | 71.68 ms | 4.8 MB | 143 B | 5.04 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 34.96 ms | 1.06 ms | 33.58 ms | 33.65 ms | 1.9 MB | 143 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.09 ms | 19.15 µs | 7.05 ms | 7.27 ms | 2.2 MB | 143 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 96.67 ms | 169.66 µs | 96.33 ms | 96.15 ms | 43.3 MB | 137 B | 18.49 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.65 ms | 425.87 µs | 9.41 ms | 10.54 ms | 48.5 MB | 137 B | 18.98 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.01 ms | 184.92 µs | 9.78 ms | 10.77 ms | 48.6 MB | 137 B | 18.78 ms as a whole process |  |
| Seal | Ceil.source | 32.91 ms | 369.07 µs | 32.07 ms | 32.65 ms | 2.7 MB | 160 B | 75.42 µs cold, 250 ns warm; 2.51 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 38.90 ms | 1.22 ms | 36.58 ms | 57.42 ms | 2.7 MB | 160 B | 912.58 µs cold, 542 ns warm; 2.70 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 35.58 ms | 338.35 µs | 34.76 ms | 53.85 ms | 2.5 MB | 160 B | 632.33 µs cold, 417 ns warm; 2.54 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 32.97 ms | 861.70 µs | 31.44 ms | 47.39 ms | 2.5 MB | 160 B | 586.46 µs cold, 417 ns warm; 2.52 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 70.45 ms | 424.34 µs | 70.02 ms | 70.44 ms | 4.8 MB | 143 B | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.98 ms | 59.52 µs | 33.87 ms | 33.63 ms | 1.9 MB | 143 B | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.90 ms | 35.87 µs | 7.80 ms | 8.30 ms | 2.2 MB | 143 B | 2.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.53 ms | 475.47 µs | 95.87 ms | 96.33 ms | 43.4 MB | 137 B | 19.06 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.51 ms | 171.29 µs | 9.42 ms | 10.83 ms | 48.7 MB | 137 B | 18.34 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.53 ms | 40.02 µs | 9.45 ms | 10.91 ms | 48.6 MB | 137 B | 18.66 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 49.20 ms | 1.86 ms | 47.47 ms | 47.59 ms | 2.7 MB | 547 B | 102.21 µs cold, 167 ns warm; 2.64 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.02 ms | 302.36 µs | 91.60 ms | 91.16 ms | 4.8 MB | 433 B | 5.05 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.08 ms | 681.55 µs | 76.08 ms | 76.67 ms | 1.9 MB | 433 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.70 ms | 67.95 µs | 6.53 ms | 6.71 ms | 2.1 MB | 433 B | 2.34 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.43 ms | 733.21 µs | 242.80 ms | 243.42 ms | 46.5 MB | 460 B | 18.88 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.60 ms | 151.60 µs | 18.43 ms | 18.82 ms | 50.2 MB | 460 B | 18.40 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.45 ms | 68.02 µs | 18.36 ms | 18.24 ms | 50.2 MB | 460 B | 18.36 ms as a whole process |  |
| Seal | Normal | 63.91 ms | 641.14 µs | 62.53 ms | 65.55 ms | 3.3 MB | 3.5 KB | 288.42 µs cold, 292 ns warm; 2.86 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.60 ms | 237.18 µs | 69.19 ms | 69.68 ms | 4.8 MB | 3.0 KB | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 72.09 ms | 454.09 µs | 71.55 ms | 71.57 ms | 1.9 MB | 3.0 KB | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.20 ms | 26.08 µs | 5.17 ms | 5.39 ms | 2.2 MB | 3.0 KB | 2.40 ms as a whole process |  |
| Node `--jitless` | Normal | 175.13 ms | 3.77 ms | 170.85 ms | 179.44 ms | 48.1 MB | 3.1 KB | 18.38 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 21.49 µs | 3.84 ms | 5.67 ms | 52.0 MB | 3.1 KB | 18.44 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 19.90 µs | 3.84 ms | 5.67 ms | 52.0 MB | 3.1 KB | 18.38 ms as a whole process |  |
| Seal | Ceil.source | 53.14 ms | 337.35 µs | 52.63 ms | 55.24 ms | 3.1 MB | 3.4 KB | 247.58 µs cold, 250 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 56.20 ms | 434.93 µs | 55.67 ms | 56.78 ms | 3.2 MB | 3.4 KB | 246.62 µs cold, 292 ns warm; 2.89 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.66 ms | 322.70 µs | 49.02 ms | 49.87 ms | 3.0 MB | 3.4 KB | 253.04 µs cold, 250 ns warm; 2.72 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.87 ms | 437.68 µs | 49.12 ms | 49.94 ms | 3.0 MB | 3.4 KB | 251.83 µs cold, 292 ns warm; 2.89 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.45 ms | 251.53 µs | 68.94 ms | 69.72 ms | 4.8 MB | 3.0 KB | 5.15 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.37 ms | 281.54 µs | 70.97 ms | 71.52 ms | 1.9 MB | 3.0 KB | 2.53 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.21 ms | 30.26 µs | 5.16 ms | 5.43 ms | 2.2 MB | 3.0 KB | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 174.83 ms | 3.28 ms | 171.15 ms | 179.76 ms | 48.0 MB | 3.1 KB | 18.54 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.87 ms | 15.63 µs | 3.85 ms | 5.73 ms | 52.1 MB | 3.1 KB | 18.62 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 16.57 µs | 3.84 ms | 5.71 ms | 52.0 MB | 3.1 KB | 19.03 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 58.28 ms | 1.20 ms | 55.33 ms | 59.85 ms | 2.9 MB | 426 B | 84.50 µs cold, 250 ns warm; 2.69 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.67 ms | 232.21 µs | 45.35 ms | 45.34 ms | 4.8 MB | 366 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.09 ms | 57.58 µs | 77.01 ms | 77.01 ms | 1.9 MB | 366 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.62 ms | 191.11 µs | 22.31 ms | 22.45 ms | 2.1 MB | 366 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Purpose | 363.02 ms | 7.60 ms | 356.07 ms | 374.12 ms | 48.6 MB | 398 B | 19.03 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.01 ms | 37.29 ms | 15.54 ms | 21.34 ms | 52.8 MB | 398 B | 18.79 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.34 ms | 111.76 µs | 15.21 ms | 19.44 ms | 55.2 MB | 398 B | 19.43 ms as a whole process |  |
| Seal | Normal | 17.77 ms | 287.19 µs | 17.48 ms | 17.82 ms | 3.0 MB | 1.2 KB | 135.00 µs cold, 250 ns warm; 2.68 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.12 ms | 261.21 µs | 26.68 ms | 27.18 ms | 4.8 MB | 981 B | 5.19 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 48.09 ms | 2.50 ms | 45.40 ms | 46.35 ms | 2.0 MB | 981 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.54 ms | 22.02 µs | 1.52 ms | 1.67 ms | 2.2 MB | 981 B | 2.33 ms as a whole process |  |
| Node `--jitless` | Normal | 65.61 ms | 346.78 µs | 65.18 ms | 65.64 ms | 48.3 MB | 1.0 KB | 18.63 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.66 ms | 213.44 µs | 1.52 ms | 3.38 ms | 52.9 MB | 1.0 KB | 18.69 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.61 ms | 38.22 µs | 1.55 ms | 3.21 ms | 52.8 MB | 1.0 KB | 18.83 ms as a whole process |  |
| Seal | Ceil.source | 17.52 ms | 561.48 µs | 17.20 ms | 17.59 ms | 3.0 MB | 1.2 KB | 141.67 µs cold, 292 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.69 ms | 130.11 µs | 18.47 ms | 18.88 ms | 3.0 MB | 1.2 KB | 152.75 µs cold, 333 ns warm; 2.64 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.54 ms | 49.01 µs | 18.47 ms | 18.52 ms | 2.8 MB | 1.2 KB | 121.96 µs cold, 166 ns warm; 2.53 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.94 ms | 48.00 µs | 18.88 ms | 18.82 ms | 2.8 MB | 1.2 KB | 123.25 µs cold, 209 ns warm; 2.76 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.43 ms | 725.22 µs | 29.53 ms | 34.52 ms | 4.8 MB | 981 B | 5.10 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 45.18 ms | 584.90 µs | 44.43 ms | 44.85 ms | 2.0 MB | 981 B | 2.37 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.55 ms | 17.45 µs | 1.52 ms | 1.71 ms | 2.2 MB | 981 B | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 65.92 ms | 236.54 µs | 65.62 ms | 65.56 ms | 48.4 MB | 1.0 KB | 18.64 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.65 ms | 180.97 µs | 1.49 ms | 3.34 ms | 53.1 MB | 1.0 KB | 18.43 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.67 ms | 14.57 µs | 1.65 ms | 3.29 ms | 52.8 MB | 1.0 KB | 18.54 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.17 ms | 346.81 µs | 83.74 ms | 84.89 ms | 2.7 MB | 629 B | 95.25 µs cold, 250 ns warm; 2.65 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 88.38 ms | 1.12 ms | 87.29 ms | 88.80 ms | 4.8 MB | 523 B | 4.97 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.03 ms | 393.33 µs | 52.40 ms | 51.68 ms | 1.9 MB | 523 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.15 ms | 23.40 µs | 6.11 ms | 6.24 ms | 2.1 MB | 523 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.60 ms | 1.01 ms | 144.98 ms | 147.86 ms | 43.3 MB | 568 B | 18.88 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.67 ms | 19.26 µs | 3.65 ms | 4.44 ms | 49.4 MB | 568 B | 18.34 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.71 ms | 30.41 µs | 3.65 ms | 4.55 ms | 49.3 MB | 568 B | 18.15 ms as a whole process |  |
| Seal | Normal | 102.07 ms | 63.27 µs | 101.99 ms | 102.42 ms | 3.0 MB | 1.8 KB | 144.96 µs cold, 291 ns warm; 2.63 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 210.75 ms | 1.74 ms | 205.83 ms | 211.55 ms | 4.8 MB | 1.7 KB | 5.10 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.16 ms | 358.60 µs | 146.61 ms | 145.68 ms | 1.9 MB | 1.7 KB | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.25 ms | 229.04 µs | 47.89 ms | 47.57 ms | 2.3 MB | 1.7 KB | 2.49 ms as a whole process |  |
| Node `--jitless` | Normal | 156.13 ms | 625.52 µs | 155.45 ms | 158.09 ms | 45.4 MB | 1.7 KB | 18.83 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.50 ms | 81.57 µs | 14.35 ms | 16.15 ms | 51.9 MB | 1.7 KB | 18.65 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.70 ms | 89.55 µs | 14.59 ms | 16.18 ms | 52.1 MB | 1.7 KB | 18.73 ms as a whole process |  |
| Seal | Ceil.source | 102.61 ms | 146.38 µs | 102.40 ms | 102.81 ms | 3.0 MB | 1.8 KB | 142.75 µs cold, 209 ns warm; 2.68 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 105.50 ms | 188.24 µs | 105.11 ms | 105.35 ms | 3.0 MB | 1.8 KB | 166.83 µs cold, 333 ns warm; 2.59 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 99.87 ms | 1.03 ms | 98.85 ms | 99.47 ms | 2.7 MB | 1.8 KB | 123.75 µs cold, 250 ns warm; 2.68 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.87 ms | 189.01 µs | 102.54 ms | 102.58 ms | 2.8 MB | 1.8 KB | 129.83 µs cold, 250 ns warm; 2.76 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 210.92 ms | 2.27 ms | 204.49 ms | 212.32 ms | 4.8 MB | 1.7 KB | 5.23 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 145.27 ms | 376.63 µs | 144.70 ms | 145.17 ms | 1.9 MB | 1.7 KB | 2.41 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.33 ms | 228.04 µs | 48.00 ms | 47.94 ms | 2.3 MB | 1.7 KB | 2.53 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 155.79 ms | 653.08 µs | 154.88 ms | 156.09 ms | 45.3 MB | 1.7 KB | 18.72 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.10 ms | 92.94 µs | 13.96 ms | 15.67 ms | 52.0 MB | 1.7 KB | 18.89 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.05 ms | 47.29 µs | 13.99 ms | 15.60 ms | 52.0 MB | 1.7 KB | 18.68 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 124.06 ms | 448.89 µs | 123.29 ms | 123.48 ms | 4.0 MB | 566 B | 106.83 µs cold, 333 ns warm; 2.68 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.25 ms | 240.90 µs | 62.01 ms | 62.56 ms | 4.8 MB | 540 B | 5.15 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 29.12 ms | 167.84 µs | 28.85 ms | 29.96 ms | 2.1 MB | 540 B | 2.33 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.61 ms | 29.70 µs | 4.55 ms | 5.01 ms | 2.3 MB | 540 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Purpose | 237.50 ms | 1.35 ms | 236.22 ms | 247.46 ms | 48.6 MB | 514 B | 18.62 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 85.07 ms | 22.92 ms | 64.24 ms | 87.60 ms | 52.5 MB | 514 B | 18.95 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 64.14 ms | 336.78 µs | 63.84 ms | 88.35 ms | 52.6 MB | 514 B | 18.13 ms as a whole process |  |
| Seal | Normal | 26.47 ms | 199.02 µs | 26.13 ms | 26.85 ms | 3.9 MB | 535 B | 106.38 µs cold, 375 ns warm; 2.76 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 32.46 ms | 1.83 ms | 30.25 ms | 29.67 ms | 6.0 MB | 478 B | 5.06 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 32.06 ms | 494.36 µs | 31.47 ms | 30.02 ms | 3.0 MB | 478 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.27 ms | 284.84 µs | 11.91 ms | 12.57 ms | 3.3 MB | 478 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Normal | 51.71 ms | 227.75 µs | 51.39 ms | 52.91 ms | 56.5 MB | 477 B | 18.86 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.30 ms | 395.27 µs | 34.92 ms | 36.26 ms | 62.5 MB | 477 B | 18.85 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 34.77 ms | 165.27 µs | 34.58 ms | 35.90 ms | 88.2 MB | 477 B | 18.60 ms as a whole process |  |
| Seal | Ceil.source | 26.23 ms | 130.31 µs | 26.00 ms | 26.38 ms | 3.9 MB | 535 B | 101.29 µs cold, 292 ns warm; 2.54 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.38 ms | 98.43 µs | 27.23 ms | 27.44 ms | 4.0 MB | 535 B | 114.75 µs cold, 250 ns warm; 2.54 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.98 ms | 65.31 µs | 23.88 ms | 24.09 ms | 3.9 MB | 535 B | 93.92 µs cold, 250 ns warm; 2.61 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.94 ms | 631.02 µs | 23.37 ms | 23.51 ms | 4.0 MB | 535 B | 95.21 µs cold, 292 ns warm; 2.47 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.92 ms | 103.23 µs | 29.77 ms | 30.10 ms | 5.9 MB | 478 B | 4.83 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.19 ms | 551.14 µs | 29.79 ms | 30.74 ms | 3.0 MB | 478 B | 2.47 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.44 ms | 531.83 µs | 11.63 ms | 12.66 ms | 3.3 MB | 478 B | 2.59 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.38 ms | 408.45 µs | 51.12 ms | 52.70 ms | 56.4 MB | 477 B | 17.99 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.13 ms | 205.44 µs | 34.82 ms | 35.70 ms | 62.5 MB | 477 B | 19.28 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.43 ms | 161.63 µs | 35.22 ms | 35.77 ms | 88.2 MB | 477 B | 18.37 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 308.27 µs | 4.51 µs | 304.33 µs | 332.92 µs | 3.0 MB | 230 B | 67.38 µs cold, 333 ns warm; 2.51 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.36 ms | 357.23 µs | 143.59 ms | 145.39 ms | 9.7 MB | 195 B | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.30 ms | 80.61 µs | 32.20 ms | 31.46 ms | 3.6 MB | 195 B | 2.43 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.72 ms | 205.12 µs | 20.54 ms | 20.83 ms | 3.8 MB | 195 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Purpose | 365.88 µs | 141.37 µs | 245.38 µs | 1.02 ms | 52.4 MB | 223 B | 18.40 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 219.05 µs | 154.81 µs | 83.92 µs | 920.33 µs | 57.8 MB | 223 B | 18.54 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 135.35 µs | 123.88 µs | 55.96 µs | 925.17 µs | 57.8 MB | 223 B | 18.07 ms as a whole process |  |
| Seal | Normal | 342.64 µs | 4.39 µs | 337.50 µs | 352.08 µs | 3.2 MB | 179 B | 72.21 µs cold, 375 ns warm; 2.52 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 161.55 ms | 378.52 µs | 161.07 ms | 163.27 ms | 11.3 MB | 138 B | 4.94 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.49 ms | 261.87 µs | 41.12 ms | 41.64 ms | 4.9 MB | 138 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.73 ms | 82.04 µs | 19.67 ms | 20.82 ms | 5.0 MB | 138 B | 2.30 ms as a whole process |  |
| Node `--jitless` | Normal | 506.95 µs | 333.96 µs | 276.29 µs | 1.15 ms | 61.4 MB | 155 B | 18.19 ms as a whole process |  |
| Node (JIT), from cold | Normal | 486.22 µs | 470.21 µs | 124.04 µs | 1.04 ms | 65.3 MB | 155 B | 18.86 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 277.62 µs | 329.18 µs | 122.67 µs | 1.08 ms | 69.3 MB | 155 B | 18.47 ms as a whole process |  |
| Seal | Ceil.source | 347.37 µs | 4.80 µs | 343.29 µs | 350.92 µs | 3.2 MB | 179 B | 73.21 µs cold, 334 ns warm; 2.61 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 346.00 µs | 8.51 µs | 337.96 µs | 392.38 µs | 3.2 MB | 179 B | 72.58 µs cold, 292 ns warm; 2.39 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 292.73 µs | 6.77 µs | 286.79 µs | 316.00 µs | 3.0 MB | 179 B | 56.88 µs cold, 333 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 290.54 µs | 8.63 µs | 282.38 µs | 327.42 µs | 3.1 MB | 179 B | 60.54 µs cold, 333 ns warm; 2.42 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.30 ms | 191.89 µs | 161.01 ms | 163.17 ms | 11.3 MB | 138 B | 5.01 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 42.13 ms | 348.44 µs | 41.74 ms | 42.08 ms | 4.9 MB | 138 B | 2.43 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.61 ms | 108.90 µs | 19.42 ms | 20.35 ms | 5.2 MB | 138 B | 2.52 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 505.29 µs | 343.79 µs | 261.79 µs | 1.13 ms | 61.5 MB | 155 B | 19.31 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 492.57 µs | 484.52 µs | 123.08 µs | 1.05 ms | 65.2 MB | 155 B | 18.19 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 276.65 µs | 323.57 µs | 124.33 µs | 1.09 ms | 69.5 MB | 155 B | 18.65 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 74.95 ms | 988.96 µs | 73.63 ms | 76.42 ms | 2.9 MB | 528 B | 92.25 µs cold, 167 ns warm; 2.70 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 95.27 ms | 262.49 µs | 94.62 ms | 95.03 ms | 4.8 MB | 411 B | 5.06 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 119.41 ms | 1.48 ms | 115.52 ms | 118.47 ms | 2.5 MB | 411 B | 2.57 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.77 ms | 699.54 µs | 93.44 ms | 94.53 ms | 2.9 MB | 411 B | 2.22 ms as a whole process |  |
| Node `--jitless` | Purpose | 74.98 ms | 164.20 µs | 74.76 ms | 78.44 ms | 59.6 MB | 444 B | 18.91 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.50 ms | 309.64 µs | 10.19 ms | 13.04 ms | 65.7 MB | 444 B | 18.36 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.47 ms | 614.44 µs | 10.16 ms | 13.04 ms | 114.6 MB | 444 B | 18.81 ms as a whole process |  |
| Seal | Normal | 99.32 ms | 675.44 µs | 98.15 ms | 100.17 ms | 7.8 MB | 977 B | 120.46 µs cold, 209 ns warm; 2.72 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 130.62 ms | 1.01 ms | 129.18 ms | 129.03 ms | 37.7 MB | 799 B | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 127.73 ms | 1.16 ms | 125.74 ms | 127.43 ms | 41.5 MB | 799 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.08 ms | 783.91 µs | 79.98 ms | 80.51 ms | 40.6 MB | 799 B | 2.57 ms as a whole process |  |
| Node `--jitless` | Normal | 113.54 ms | 1.62 ms | 112.47 ms | 118.23 ms | 213.2 MB | 878 B | 19.08 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.21 ms | 891.78 µs | 13.37 ms | 16.94 ms | 129.7 MB | 878 B | 18.69 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.40 ms | 39.92 µs | 13.34 ms | 16.97 ms | 225.5 MB | 878 B | 18.54 ms as a whole process |  |
| Seal | Ceil.source | 101.87 ms | 1.44 ms | 99.25 ms | 101.21 ms | 7.8 MB | 977 B | 118.29 µs cold, 209 ns warm; 2.67 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 100.48 ms | 533.91 µs | 99.41 ms | 99.90 ms | 7.8 MB | 977 B | 121.17 µs cold, 292 ns warm; 2.75 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 84.38 ms | 640.19 µs | 83.69 ms | 86.83 ms | 7.6 MB | 977 B | 103.50 µs cold, 208 ns warm; 2.56 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 85.41 ms | 335.58 µs | 84.79 ms | 86.34 ms | 7.6 MB | 977 B | 102.08 µs cold, 208 ns warm; 2.66 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 128.30 ms | 1.07 ms | 126.82 ms | 129.52 ms | 44.8 MB | 799 B | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.56 ms | 1.13 ms | 126.82 ms | 127.98 ms | 41.5 MB | 799 B | 2.53 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 82.23 ms | 932.70 µs | 81.14 ms | 80.85 ms | 41.6 MB | 799 B | 2.42 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 113.08 ms | 2.04 ms | 110.91 ms | 117.12 ms | 213.5 MB | 878 B | 18.51 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.26 ms | 896.80 µs | 13.30 ms | 16.91 ms | 129.7 MB | 878 B | 18.86 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.30 ms | 42.44 µs | 13.26 ms | 16.97 ms | 224.1 MB | 878 B | 18.86 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 49.02 µs (±6.38 µs, least 43.92 µs) | 16.52 µs (±3.27 µs, least 12.08 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.66 µs (±1.08 µs, least 8.42 µs) | 16.80 µs (±2.24 µs, least 14.00 µs) | 41 ns (±1 ns, least 39 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.90 µs (±738 ns, least 3.00 µs) | 6.90 µs (±1.97 µs, least 4.00 µs) | 22 ns (±1 ns, least 20 ns) | 15 ns (±1 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 4.20 µs (±1.03 µs, least 3.00 µs) | 7.00 µs (±1.70 µs, least 5.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 34.90 µs (±5.42 µs, least 19.67 µs) | 38.36 µs (±2.26 µs, least 35.88 µs) | 57 ns (±2 ns, least 54 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 25.53 µs (±8.03 µs, least 18.67 µs) | 37.63 µs (±1.53 µs, least 35.83 µs) | 8 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 52.9 MB | the engine's own heap |
| Seal | Normal | 4.48 µs (±49 ns, least 4.41 µs) | 157 ns (±7 ns, least 143 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.91 µs (±184 ns, least 4.64 µs) | 185 ns (±8 ns, least 172 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.79 µs (±31 ns, least 1.74 µs) | 181 ns (±7 ns, least 170 ns) | 27 ns (±1 ns, least 27 ns) | 16 ns (±2 ns, least 14 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.81 µs (±25 ns, least 1.77 µs) | 358 ns (±12 ns, least 345 ns) | 93 ns (±2 ns, least 91 ns) | 109 ns (±15 ns, least 100 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±17 ns, least 1.13 µs) | 674 ns (±26 ns, least 641 ns) | 50 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.10 µs (±24 ns, least 1.07 µs) | 670 ns (±16 ns, least 649 ns) | 22 ns (±0 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.8 MB | the engine's own heap |
| Seal | Ceil.source | 4.55 µs (±48 ns, least 4.46 µs) | 150 ns (±10 ns, least 135 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.67 µs (±156 ns, least 4.57 µs) | 155 ns (±9 ns, least 145 ns) | 21 ns (±1 ns, least 20 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 3.97 µs (±21 ns, least 3.95 µs) | 142 ns (±9 ns, least 127 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.01 µs (±51 ns, least 3.93 µs) | 147 ns (±8 ns, least 130 ns) | 18 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.85 µs (±197 ns, least 4.63 µs) | 186 ns (±4 ns, least 181 ns) | 33 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.78 µs (±21 ns, least 1.75 µs) | 181 ns (±7 ns, least 170 ns) | 27 ns (±0 ns, least 26 ns) | 15 ns (±1 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.81 µs (±19 ns, least 1.77 µs) | 358 ns (±17 ns, least 330 ns) | 93 ns (±3 ns, least 89 ns) | 104 ns (±10 ns, least 99 ns) | 1.0 KB | 4.8 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±16 ns, least 1.11 µs) | 670 ns (±20 ns, least 642 ns) | 50 ns (±1 ns, least 48 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±16 ns, least 1.08 µs) | 658 ns (±22 ns, least 631 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 31.29 ms | 32.02 ms (+2.3%) |
| n-body, 100,000 steps | 65.40 ms | 59.69 ms (-8.7%) |
| spectral-norm, size 200 | 18.94 ms | 18.88 ms (-0.4%) |
| fannkuch-redux, 9 | 103.22 ms | 101.76 ms (-1.4%) |
| word count, 500,000 words | 26.40 ms | 26.27 ms (-0.5%) |
| text building, 20,000 joins | 341.75 µs | 344.76 µs (+0.9%) |
| binary-trees, depth 14 | 99.34 ms | 100.34 ms (+1.0%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 171.08 ms | 736.30 µs | 169.74 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.33 ms | 113.22 µs | 10.20 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.43 ms | 27.57 µs | 4.36 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.45 ms | 44.67 µs | 1.41 ms | 3.6 MB |
