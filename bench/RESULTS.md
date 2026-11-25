# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.25 2.43 2.21 }` before, `{ 2.10 2.26 2.17 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 40.66 ms | 72.40 ms | **met**: Seal takes 56% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 64.55 ms | 69.76 ms | **met**: Seal takes 93% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.57 ms | 26.75 ms | **met**: Seal takes 73% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 114.83 ms | 213.76 ms | **met**: Seal takes 54% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 29.28 ms | 30.45 ms | **met**: Seal takes 96% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 341.85 µs | 161.40 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 106.74 ms | 127.71 ms | **met**: Seal takes 84% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.86 ms | 3.62 ms | **met**: Seal takes 51% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 34 ns | **met**: Seal takes 50% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 142 ns | 187 ns | **met**: Seal takes 76% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.87 µs | 4.92 µs | **met**: Seal takes 99% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 40.66 ms | 33.84 ms | **not met**: Seal takes 120% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 64.55 ms | 71.95 ms | **met**: Seal takes 90% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 19.57 ms | 45.83 ms | **met**: Seal takes 43% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 114.83 ms | 148.12 ms | **met**: Seal takes 78% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 29.28 ms | 31.11 ms | **met**: Seal takes 94% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 341.85 µs | 42.12 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 106.74 ms | 128.08 ms | **met**: Seal takes 83% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.86 ms | 3.00 ms | **met**: Seal takes 62% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 17 ns | 28 ns | **met**: Seal takes 61% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 142 ns | 180 ns | **met**: Seal takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.87 µs | 1.80 µs | **not met**: Seal takes 270% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 40.66 ms | 94.84 ms | **met**: Seal takes 43% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 64.55 ms | 179.07 ms | **met**: Seal takes 36% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 29.28 ms | 51.80 ms | **met**: Seal takes 57% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 341.85 µs | 512.39 µs | **met**: Seal takes 67% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.46 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 111.30 ms | 1.17 ms | 109.94 ms | 116.00 ms | 2.8 MB | 386 B | 485.25 µs cold, 291 ns warm; 2.90 ms as a whole process | 32.0 KB counted by Seal; 184 B compiled |
| Luau (interpreter) | Purpose | 202.73 ms | 15.85 ms | 195.63 ms | 199.34 ms | 4.8 MB | 339 B | 5.32 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.23 ms | 682.34 µs | 89.09 ms | 91.01 ms | 1.9 MB | 339 B | 2.57 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.22 ms | 192.43 µs | 44.03 ms | 45.92 ms | 2.2 MB | 339 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Purpose | 337.96 ms | 5.11 ms | 332.74 ms | 378.04 ms | 43.4 MB | 362 B | 19.58 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.67 ms | 843.33 µs | 123.79 ms | 125.00 ms | 49.1 MB | 362 B | 19.55 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 126.17 ms | 3.66 ms | 121.90 ms | 124.45 ms | 49.1 MB | 362 B | 18.83 ms as a whole process |  |
| Seal | Normal | 40.66 ms | 92.42 µs | 40.50 ms | 40.66 ms | 2.7 MB | 160 B | 74.29 µs cold, 166 ns warm; 2.69 ms as a whole process | 2.0 KB counted by Seal; 128 B compiled |
| Luau (interpreter) | Normal | 72.40 ms | 151.74 µs | 72.11 ms | 72.65 ms | 4.8 MB | 143 B | 5.14 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.84 ms | 160.15 µs | 33.68 ms | 33.69 ms | 1.9 MB | 143 B | 2.41 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 6.94 ms | 14.40 µs | 6.91 ms | 7.22 ms | 2.2 MB | 143 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Normal | 94.84 ms | 1.10 ms | 93.18 ms | 92.07 ms | 43.3 MB | 137 B | 18.89 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.57 ms | 48.08 µs | 9.49 ms | 10.04 ms | 48.7 MB | 137 B | 18.57 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.84 ms | 267.28 µs | 9.59 ms | 10.28 ms | 48.6 MB | 137 B | 19.73 ms as a whole process |  |
| Seal | Ceil.source | 40.62 ms | 328.74 µs | 40.33 ms | 40.53 ms | 2.7 MB | 160 B | 73.17 µs cold, 208 ns warm; 2.84 ms as a whole process | 2.0 KB counted by Seal; 128 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 41.19 ms | 511.42 µs | 40.49 ms | 65.93 ms | 2.7 MB | 160 B | 983.33 µs cold, 584 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 128 B compiled; the Normal program stands in |
| Seal | Ceil.program | 38.02 ms | 228.12 µs | 37.84 ms | 61.62 ms | 2.5 MB | 160 B | 704.71 µs cold, 458 ns warm; 2.68 ms as a whole process | 2.0 KB counted by Seal; 128 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 37.41 ms | 180.09 µs | 37.20 ms | 50.97 ms | 2.5 MB | 160 B | 696.58 µs cold, 334 ns warm; 2.62 ms as a whole process | 2.0 KB counted by Seal; 128 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.09 ms | 771.61 µs | 71.48 ms | 70.95 ms | 4.8 MB | 143 B | 5.14 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.96 ms | 170.39 µs | 33.79 ms | 34.35 ms | 1.9 MB | 143 B | 2.54 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.25 ms | 90.96 µs | 7.20 ms | 7.45 ms | 2.2 MB | 143 B | 2.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.38 ms | 1.26 ms | 94.67 ms | 97.20 ms | 43.4 MB | 137 B | 19.53 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.69 ms | 191.20 µs | 9.51 ms | 9.94 ms | 48.5 MB | 137 B | 20.05 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.94 ms | 304.28 µs | 9.60 ms | 10.53 ms | 48.7 MB | 137 B | 19.19 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.99 ms | 540.27 µs | 52.34 ms | 53.41 ms | 2.7 MB | 547 B | 118.21 µs cold, 292 ns warm; 2.79 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.16 ms | 327.81 µs | 91.67 ms | 91.00 ms | 4.8 MB | 433 B | 5.34 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.19 ms | 400.91 µs | 76.46 ms | 77.18 ms | 1.9 MB | 433 B | 2.52 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.62 ms | 82.47 µs | 6.51 ms | 6.68 ms | 2.1 MB | 433 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Purpose | 247.12 ms | 1.35 ms | 244.69 ms | 249.06 ms | 46.3 MB | 460 B | 20.72 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.73 ms | 204.74 µs | 18.52 ms | 19.25 ms | 50.3 MB | 460 B | 18.88 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.45 ms | 135.23 µs | 18.24 ms | 18.55 ms | 50.2 MB | 460 B | 18.45 ms as a whole process |  |
| Seal | Normal | 64.55 ms | 378.30 µs | 63.95 ms | 64.71 ms | 3.3 MB | 3.5 KB | 280.04 µs cold, 292 ns warm; 2.89 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.76 ms | 180.13 µs | 69.38 ms | 69.57 ms | 4.8 MB | 3.0 KB | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.95 ms | 428.26 µs | 71.38 ms | 71.51 ms | 1.9 MB | 3.0 KB | 2.63 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.29 ms | 36.19 µs | 5.24 ms | 5.49 ms | 2.2 MB | 3.0 KB | 2.46 ms as a whole process |  |
| Node `--jitless` | Normal | 179.07 ms | 3.23 ms | 175.34 ms | 185.23 ms | 48.2 MB | 3.1 KB | 19.56 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.91 ms | 25.48 µs | 3.87 ms | 5.68 ms | 51.8 MB | 3.1 KB | 18.95 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.89 ms | 20.14 µs | 3.85 ms | 5.72 ms | 52.0 MB | 3.1 KB | 18.77 ms as a whole process |  |
| Seal | Ceil.source | 58.33 ms | 393.65 µs | 57.73 ms | 58.54 ms | 3.2 MB | 3.4 KB | 283.79 µs cold, 291 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 56.87 ms | 436.65 µs | 56.30 ms | 57.33 ms | 3.2 MB | 3.4 KB | 266.42 µs cold, 291 ns warm; 2.89 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.45 ms | 212.93 µs | 50.16 ms | 50.39 ms | 2.9 MB | 3.4 KB | 257.75 µs cold, 250 ns warm; 3.02 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.40 ms | 152.10 µs | 50.20 ms | 50.30 ms | 3.0 MB | 3.4 KB | 265.29 µs cold, 208 ns warm; 2.71 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.78 ms | 240.28 µs | 69.47 ms | 69.82 ms | 4.8 MB | 3.0 KB | 5.25 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 72.14 ms | 305.81 µs | 71.57 ms | 72.25 ms | 1.9 MB | 3.0 KB | 2.48 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.27 ms | 70.99 µs | 5.19 ms | 5.40 ms | 2.2 MB | 3.0 KB | 2.54 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.66 ms | 3.42 ms | 174.62 ms | 181.44 ms | 48.2 MB | 3.1 KB | 19.63 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 15.69 µs | 3.86 ms | 5.66 ms | 52.0 MB | 3.1 KB | 19.47 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.88 ms | 30.92 µs | 3.85 ms | 5.71 ms | 52.0 MB | 3.1 KB | 19.41 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.18 ms | 878.07 µs | 52.10 ms | 54.16 ms | 2.9 MB | 426 B | 87.54 µs cold, 250 ns warm; 2.70 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.67 ms | 284.35 µs | 45.25 ms | 45.20 ms | 4.8 MB | 366 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.27 ms | 89.17 µs | 77.14 ms | 77.05 ms | 1.9 MB | 366 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.34 ms | 129.86 µs | 22.11 ms | 22.43 ms | 2.1 MB | 366 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.20 ms | 5.50 ms | 363.63 ms | 374.59 ms | 48.6 MB | 398 B | 19.16 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.85 ms | 36.69 ms | 16.80 ms | 19.15 ms | 52.7 MB | 398 B | 19.17 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.37 ms | 54.07 µs | 14.29 ms | 19.68 ms | 55.2 MB | 398 B | 18.97 ms as a whole process |  |
| Seal | Normal | 19.57 ms | 52.55 µs | 19.51 ms | 19.53 ms | 3.0 MB | 1.2 KB | 151.29 µs cold, 291 ns warm; 2.63 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 26.75 ms | 264.21 µs | 26.27 ms | 27.29 ms | 4.8 MB | 981 B | 4.95 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.83 ms | 518.78 µs | 45.40 ms | 45.41 ms | 2.0 MB | 981 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.53 ms | 11.21 µs | 1.51 ms | 1.70 ms | 2.2 MB | 981 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Normal | 63.03 ms | 171.13 µs | 62.80 ms | 63.06 ms | 48.4 MB | 1.0 KB | 19.11 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.68 ms | 212.94 µs | 1.54 ms | 3.34 ms | 52.8 MB | 1.0 KB | 18.61 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.63 ms | 24.45 µs | 1.59 ms | 3.61 ms | 52.9 MB | 1.0 KB | 19.23 ms as a whole process |  |
| Seal | Ceil.source | 19.65 ms | 66.49 µs | 19.52 ms | 19.68 ms | 3.0 MB | 1.2 KB | 143.58 µs cold, 208 ns warm; 2.75 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.65 ms | 75.69 µs | 19.51 ms | 19.62 ms | 3.0 MB | 1.2 KB | 147.79 µs cold, 250 ns warm; 2.67 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.18 ms | 44.23 µs | 19.11 ms | 19.21 ms | 2.8 MB | 1.2 KB | 125.75 µs cold, 250 ns warm; 2.56 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.70 ms | 50.23 µs | 18.64 ms | 18.61 ms | 2.8 MB | 1.2 KB | 132.71 µs cold, 167 ns warm; 2.64 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.15 ms | 578.09 µs | 26.31 ms | 26.85 ms | 4.8 MB | 981 B | 5.15 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 43.74 ms | 860.20 µs | 43.23 ms | 43.62 ms | 2.0 MB | 981 B | 2.51 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.54 ms | 18.12 µs | 1.51 ms | 1.70 ms | 2.2 MB | 981 B | 2.58 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.72 ms | 281.48 µs | 62.50 ms | 63.35 ms | 48.4 MB | 1.0 KB | 19.21 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.59 ms | 202.42 µs | 1.46 ms | 3.40 ms | 52.7 MB | 1.0 KB | 19.30 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.69 ms | 12.29 µs | 1.67 ms | 3.29 ms | 52.8 MB | 1.0 KB | 18.29 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.54 ms | 674.50 µs | 96.60 ms | 97.35 ms | 2.8 MB | 629 B | 90.46 µs cold, 167 ns warm; 2.59 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.94 ms | 844.39 µs | 87.60 ms | 89.34 ms | 4.8 MB | 523 B | 5.27 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 54.16 ms | 417.99 µs | 53.63 ms | 52.93 ms | 1.9 MB | 523 B | 2.41 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.18 ms | 31.32 µs | 6.13 ms | 6.25 ms | 2.1 MB | 523 B | 2.44 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.19 ms | 1.21 ms | 146.31 ms | 152.67 ms | 43.3 MB | 568 B | 19.20 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.71 ms | 88.54 µs | 3.64 ms | 4.45 ms | 49.2 MB | 568 B | 19.11 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.67 ms | 12.77 µs | 3.65 ms | 4.51 ms | 49.3 MB | 568 B | 18.78 ms as a whole process |  |
| Seal | Normal | 114.83 ms | 1.22 ms | 113.96 ms | 114.07 ms | 3.0 MB | 1.8 KB | 144.46 µs cold, 209 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 213.76 ms | 2.75 ms | 206.92 ms | 213.52 ms | 4.8 MB | 1.7 KB | 5.60 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 148.12 ms | 350.70 µs | 147.72 ms | 147.90 ms | 1.9 MB | 1.7 KB | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 51.69 ms | 3.35 ms | 49.07 ms | 49.79 ms | 2.3 MB | 1.7 KB | 2.76 ms as a whole process |  |
| Node `--jitless` | Normal | 157.36 ms | 1.54 ms | 155.51 ms | 157.97 ms | 45.4 MB | 1.7 KB | 18.69 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.81 ms | 128.48 µs | 14.58 ms | 16.03 ms | 52.0 MB | 1.7 KB | 18.86 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.74 ms | 64.49 µs | 14.68 ms | 16.00 ms | 52.2 MB | 1.7 KB | 19.22 ms as a whole process |  |
| Seal | Ceil.source | 112.85 ms | 129.21 µs | 112.63 ms | 113.17 ms | 3.0 MB | 1.8 KB | 146.33 µs cold, 208 ns warm; 2.89 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 112.58 ms | 879.65 µs | 111.76 ms | 113.54 ms | 3.0 MB | 1.8 KB | 150.29 µs cold, 209 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 104.96 ms | 295.60 µs | 104.40 ms | 104.70 ms | 2.8 MB | 1.8 KB | 122.21 µs cold, 208 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.83 ms | 323.81 µs | 102.39 ms | 103.18 ms | 2.8 MB | 1.8 KB | 134.12 µs cold, 166 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 212.16 ms | 1.96 ms | 206.70 ms | 214.71 ms | 4.8 MB | 1.7 KB | 5.28 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 147.88 ms | 393.57 µs | 147.39 ms | 146.46 ms | 1.9 MB | 1.7 KB | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.97 ms | 443.66 µs | 48.05 ms | 49.30 ms | 2.3 MB | 1.7 KB | 2.55 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.70 ms | 345.75 µs | 157.19 ms | 159.73 ms | 45.4 MB | 1.7 KB | 18.35 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 15.06 ms | 194.77 µs | 14.91 ms | 16.11 ms | 52.0 MB | 1.7 KB | 19.06 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.23 ms | 116.44 µs | 14.05 ms | 16.02 ms | 52.0 MB | 1.7 KB | 18.65 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 96.47 ms | 2.03 ms | 94.06 ms | 93.42 ms | 4.0 MB | 566 B | 97.67 µs cold, 334 ns warm; 2.74 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.56 ms | 241.81 µs | 63.06 ms | 63.93 ms | 4.8 MB | 540 B | 5.31 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 31.27 ms | 1.78 ms | 28.04 ms | 31.07 ms | 2.1 MB | 540 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.79 ms | 37.97 µs | 4.73 ms | 4.85 ms | 2.3 MB | 540 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Purpose | 240.33 ms | 2.03 ms | 237.76 ms | 244.55 ms | 48.8 MB | 514 B | 19.08 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 91.16 ms | 24.78 ms | 68.48 ms | 92.36 ms | 52.7 MB | 514 B | 19.89 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 71.69 ms | 2.39 ms | 67.62 ms | 92.41 ms | 52.8 MB | 514 B | 20.53 ms as a whole process |  |
| Seal | Normal | 29.28 ms | 9.05 ms | 24.06 ms | 23.92 ms | 4.2 MB | 535 B | 98.54 µs cold, 417 ns warm; 2.80 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.45 ms | 268.97 µs | 30.13 ms | 30.77 ms | 5.9 MB | 478 B | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.11 ms | 510.85 µs | 29.95 ms | 30.47 ms | 3.0 MB | 478 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.36 ms | 498.57 µs | 11.61 ms | 13.09 ms | 3.3 MB | 478 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Normal | 51.80 ms | 442.52 µs | 51.29 ms | 53.02 ms | 56.5 MB | 477 B | 18.87 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.85 ms | 297.93 µs | 35.22 ms | 36.25 ms | 62.5 MB | 477 B | 19.42 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.24 ms | 248.71 µs | 34.95 ms | 36.40 ms | 88.7 MB | 477 B | 19.08 ms as a whole process |  |
| Seal | Ceil.source | 23.82 ms | 89.80 µs | 23.67 ms | 23.77 ms | 4.0 MB | 535 B | 102.12 µs cold, 375 ns warm; 2.79 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 23.84 ms | 98.00 µs | 23.70 ms | 23.92 ms | 4.0 MB | 535 B | 101.21 µs cold, 333 ns warm; 2.60 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.41 ms | 402.38 µs | 22.85 ms | 23.30 ms | 3.8 MB | 535 B | 93.42 µs cold, 208 ns warm; 2.72 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 25.22 ms | 341.84 µs | 24.72 ms | 24.33 ms | 3.9 MB | 535 B | 84.04 µs cold, 208 ns warm; 2.75 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.22 ms | 155.22 µs | 29.90 ms | 30.08 ms | 6.0 MB | 478 B | 5.05 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.30 ms | 205.56 µs | 32.90 ms | 31.62 ms | 3.0 MB | 478 B | 2.36 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.33 ms | 412.10 µs | 11.89 ms | 12.53 ms | 3.4 MB | 478 B | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.01 ms | 397.16 µs | 51.58 ms | 52.55 ms | 56.4 MB | 477 B | 18.96 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.49 ms | 287.88 µs | 35.01 ms | 36.21 ms | 62.3 MB | 477 B | 19.02 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.68 ms | 250.97 µs | 35.33 ms | 36.28 ms | 88.3 MB | 477 B | 18.92 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 303.16 µs | 7.64 µs | 298.04 µs | 320.25 µs | 3.1 MB | 230 B | 69.38 µs cold, 333 ns warm; 2.68 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 143.61 ms | 321.02 µs | 143.15 ms | 144.91 ms | 9.7 MB | 195 B | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.37 ms | 131.07 µs | 32.18 ms | 31.49 ms | 3.5 MB | 195 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 21.01 ms | 255.37 µs | 20.44 ms | 20.91 ms | 3.8 MB | 195 B | 2.33 ms as a whole process |  |
| Node `--jitless` | Purpose | 365.20 µs | 147.86 µs | 234.83 µs | 1.03 ms | 52.5 MB | 223 B | 18.73 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 210.08 µs | 145.12 µs | 85.21 µs | 949.50 µs | 57.8 MB | 223 B | 18.79 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 167.75 µs | 167.69 µs | 55.92 µs | 955.54 µs | 58.4 MB | 223 B | 18.84 ms as a whole process |  |
| Seal | Normal | 341.85 µs | 8.30 µs | 334.00 µs | 351.08 µs | 3.2 MB | 179 B | 74.75 µs cold, 292 ns warm; 2.69 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 161.40 ms | 224.48 µs | 161.10 ms | 163.72 ms | 11.3 MB | 138 B | 5.06 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 42.12 ms | 179.06 µs | 41.76 ms | 41.73 ms | 5.1 MB | 138 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.47 ms | 139.22 µs | 19.25 ms | 20.02 ms | 5.3 MB | 138 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 512.39 µs | 358.23 µs | 277.75 µs | 1.10 ms | 61.4 MB | 155 B | 19.41 ms as a whole process |  |
| Node (JIT), from cold | Normal | 507.55 µs | 489.33 µs | 133.38 µs | 1.05 ms | 65.3 MB | 155 B | 18.72 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 288.88 µs | 324.58 µs | 130.46 µs | 1.04 ms | 69.3 MB | 155 B | 18.61 ms as a whole process |  |
| Seal | Ceil.source | 337.14 µs | 4.11 µs | 332.42 µs | 365.79 µs | 3.2 MB | 179 B | 74.54 µs cold, 291 ns warm; 2.60 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 341.44 µs | 3.76 µs | 337.25 µs | 402.29 µs | 3.2 MB | 179 B | 76.62 µs cold, 292 ns warm; 2.57 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 280.88 µs | 4.43 µs | 276.04 µs | 308.38 µs | 3.0 MB | 179 B | 58.42 µs cold, 250 ns warm; 2.51 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 280.72 µs | 3.19 µs | 276.04 µs | 329.17 µs | 3.0 MB | 179 B | 63.04 µs cold, 208 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.82 ms | 298.53 µs | 161.29 ms | 163.71 ms | 11.3 MB | 138 B | 5.08 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.37 ms | 169.78 µs | 41.10 ms | 41.31 ms | 4.9 MB | 138 B | 2.54 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 20.14 ms | 291.38 µs | 19.53 ms | 20.54 ms | 5.2 MB | 138 B | 2.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 505.48 µs | 343.03 µs | 265.71 µs | 1.15 ms | 61.4 MB | 155 B | 19.02 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 490.65 µs | 481.66 µs | 122.92 µs | 1.08 ms | 65.2 MB | 155 B | 18.66 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 275.17 µs | 326.87 µs | 124.29 µs | 1.06 ms | 69.2 MB | 155 B | 19.16 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.47 ms | 1.13 ms | 74.09 ms | 77.59 ms | 3.0 MB | 528 B | 92.67 µs cold, 208 ns warm; 2.61 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 95.37 ms | 279.86 µs | 94.64 ms | 95.28 ms | 4.8 MB | 411 B | 4.94 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 118.47 ms | 1.17 ms | 116.52 ms | 119.09 ms | 2.5 MB | 411 B | 2.39 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 95.88 ms | 903.07 µs | 94.54 ms | 95.87 ms | 2.9 MB | 411 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.02 ms | 232.44 µs | 74.80 ms | 78.40 ms | 59.5 MB | 444 B | 18.67 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.50 ms | 304.68 µs | 10.22 ms | 12.98 ms | 65.8 MB | 444 B | 18.63 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.45 ms | 649.09 µs | 10.11 ms | 13.02 ms | 114.8 MB | 444 B | 19.07 ms as a whole process |  |
| Seal | Normal | 106.74 ms | 535.54 µs | 106.05 ms | 107.77 ms | 7.8 MB | 977 B | 139.04 µs cold, 208 ns warm; 2.66 ms as a whole process | 4.0 MB counted by Seal; 632 B compiled |
| Luau (interpreter) | Normal | 127.71 ms | 1.72 ms | 125.41 ms | 129.31 ms | 37.6 MB | 799 B | 5.14 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 128.08 ms | 1.05 ms | 126.15 ms | 127.70 ms | 41.5 MB | 799 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.12 ms | 1.12 ms | 79.84 ms | 81.14 ms | 43.8 MB | 799 B | 2.54 ms as a whole process |  |
| Node `--jitless` | Normal | 116.78 ms | 1.52 ms | 114.85 ms | 118.23 ms | 213.3 MB | 878 B | 19.32 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.46 ms | 904.80 µs | 13.50 ms | 17.71 ms | 129.9 MB | 878 B | 18.74 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.56 ms | 71.86 µs | 13.44 ms | 17.35 ms | 224.0 MB | 878 B | 19.27 ms as a whole process |  |
| Seal | Ceil.source | 107.75 ms | 1.01 ms | 106.65 ms | 109.19 ms | 7.8 MB | 977 B | 121.88 µs cold, 209 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 632 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 108.00 ms | 731.76 µs | 107.01 ms | 108.25 ms | 7.8 MB | 977 B | 119.75 µs cold, 125 ns warm; 2.72 ms as a whole process | 4.0 MB counted by Seal; 632 B compiled; the Normal program stands in |
| Seal | Ceil.program | 87.91 ms | 625.41 µs | 87.08 ms | 91.60 ms | 7.6 MB | 977 B | 102.54 µs cold, 209 ns warm; 2.72 ms as a whole process | 4.0 MB counted by Seal; 632 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 87.15 ms | 481.97 µs | 86.43 ms | 87.75 ms | 7.6 MB | 977 B | 109.46 µs cold, 209 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 632 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 140.77 ms | 6.25 ms | 129.58 ms | 132.56 ms | 44.4 MB | 799 B | 5.21 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 131.27 ms | 2.03 ms | 127.38 ms | 129.02 ms | 39.8 MB | 799 B | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 82.72 ms | 686.51 µs | 81.60 ms | 81.73 ms | 40.5 MB | 799 B | 2.47 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 116.83 ms | 2.09 ms | 114.70 ms | 120.53 ms | 213.2 MB | 878 B | 19.63 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.55 ms | 867.19 µs | 13.77 ms | 17.35 ms | 129.6 MB | 878 B | 19.86 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.74 ms | 590.68 µs | 13.46 ms | 18.13 ms | 224.2 MB | 878 B | 19.03 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 81.75 µs (±26.09 µs, least 65.50 µs) | 31.95 µs (±16.91 µs, least 17.46 µs) | 16 ns (±0 ns, least 16 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.54 µs (±1.14 µs, least 9.04 µs) | 19.02 µs (±2.09 µs, least 17.17 µs) | 42 ns (±2 ns, least 40 ns) | 20 ns (±1 ns, least 19 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 4.20 µs (±632 ns, least 3.00 µs) | 6.60 µs (±843 ns, least 5.00 µs) | 23 ns (±2 ns, least 21 ns) | 14 ns (±1 ns, least 12 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 4.10 µs (±568 ns, least 3.00 µs) | 6.10 µs (±1.73 µs, least 4.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 29.30 µs (±8.15 µs, least 20.08 µs) | 41.06 µs (±10.00 µs, least 35.50 µs) | 58 ns (±1 ns, least 55 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 29.53 µs (±6.83 µs, least 18.08 µs) | 39.10 µs (±2.53 µs, least 36.46 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.87 µs (±64 ns, least 4.74 µs) | 142 ns (±7 ns, least 133 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.92 µs (±156 ns, least 4.77 µs) | 187 ns (±10 ns, least 167 ns) | 34 ns (±2 ns, least 32 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.80 µs (±26 ns, least 1.77 µs) | 180 ns (±8 ns, least 172 ns) | 28 ns (±1 ns, least 27 ns) | 14 ns (±1 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.81 µs (±28 ns, least 1.77 µs) | 357 ns (±11 ns, least 333 ns) | 93 ns (±1 ns, least 91 ns) | 113 ns (±16 ns, least 100 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.16 µs (±28 ns, least 1.12 µs) | 684 ns (±9 ns, least 673 ns) | 51 ns (±2 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.12 µs (±13 ns, least 1.10 µs) | 678 ns (±34 ns, least 646 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.82 µs (±39 ns, least 4.75 µs) | 143 ns (±5 ns, least 133 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.84 µs (±61 ns, least 4.75 µs) | 146 ns (±6 ns, least 138 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 4.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.25 µs (±55 ns, least 4.16 µs) | 137 ns (±9 ns, least 123 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±0 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.29 µs (±42 ns, least 4.23 µs) | 141 ns (±9 ns, least 129 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±0 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.01 µs (±196 ns, least 4.70 µs) | 183 ns (±7 ns, least 172 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.80 µs (±25 ns, least 1.77 µs) | 185 ns (±7 ns, least 174 ns) | 28 ns (±1 ns, least 27 ns) | 15 ns (±2 ns, least 12 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.83 µs (±23 ns, least 1.79 µs) | 373 ns (±39 ns, least 333 ns) | 96 ns (±3 ns, least 91 ns) | 107 ns (±14 ns, least 99 ns) | 1.0 KB | 4.8 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.16 µs (±28 ns, least 1.10 µs) | 665 ns (±20 ns, least 639 ns) | 51 ns (±1 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.5 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.12 µs (±26 ns, least 1.08 µs) | 666 ns (±12 ns, least 642 ns) | 22 ns (±0 ns, least 22 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 40.43 ms | 40.32 ms (-0.3%) |
| n-body, 100,000 steps | 65.20 ms | 61.63 ms (-5.5%) |
| spectral-norm, size 200 | 19.77 ms | 19.27 ms (-2.5%) |
| fannkuch-redux, 9 | 112.85 ms | 112.92 ms (+0.1%) |
| word count, 500,000 words | 23.88 ms | 23.93 ms (+0.2%) |
| text building, 20,000 joins | 343.76 µs | 344.46 µs (+0.2%) |
| binary-trees, depth 14 | 106.48 ms | 105.43 ms (-1.0%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 174.63 ms | 1.55 ms | 173.30 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.66 ms | 46.66 µs | 10.59 ms | 2.8 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.41 ms | 33.96 µs | 4.33 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.41 ms | 19.31 µs | 1.36 ms | 3.7 MB |
