# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.17 1.83 1.72 }` before, `{ 1.63 1.74 1.70 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 33.03 ms | 70.54 ms | **met**: Seal takes 47% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 64.09 ms | 70.18 ms | **met**: Seal takes 91% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.61 ms | 27.34 ms | **met**: Seal takes 68% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 103.44 ms | 210.98 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.48 ms | 30.45 ms | **met**: Seal takes 87% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 343.26 µs | 161.17 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 99.18 ms | 131.84 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.15 ms | 3.54 ms | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 34 ns | **met**: Seal takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 155 ns | 189 ns | **met**: Seal takes 82% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.79 µs | 4.89 µs | **met**: Seal takes 98% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 33.03 ms | 33.85 ms | **met**: Seal takes 98% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 64.09 ms | 71.99 ms | **met**: Seal takes 89% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.61 ms | 46.11 ms | **met**: Seal takes 40% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 103.44 ms | 147.37 ms | **met**: Seal takes 70% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.48 ms | 31.42 ms | **met**: Seal takes 84% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 343.26 µs | 41.83 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 99.18 ms | 130.09 ms | **met**: Seal takes 76% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.15 ms | 2.91 ms | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 27 ns | **met**: Seal takes 73% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 155 ns | 177 ns | **met**: Seal takes 87% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.79 µs | 1.78 µs | **not met**: Seal takes 269% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 33.03 ms | 96.83 ms | **met**: Seal takes 34% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 64.09 ms | 177.46 ms | **met**: Seal takes 36% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.48 ms | 51.21 ms | **met**: Seal takes 52% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 343.26 µs | 509.55 µs | **met**: Seal takes 67% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 39 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.33 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.41 ms | 500.57 µs | 84.05 ms | 103.66 ms | 2.8 MB | 386 B | 748.62 µs cold, 375 ns warm; 2.77 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 195.25 ms | 142.44 µs | 195.07 ms | 196.30 ms | 4.8 MB | 339 B | 5.10 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.50 ms | 801.00 µs | 88.84 ms | 90.89 ms | 1.9 MB | 339 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 43.52 ms | 234.10 µs | 43.11 ms | 44.99 ms | 2.2 MB | 339 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 323.35 ms | 413.80 µs | 322.54 ms | 322.57 ms | 43.4 MB | 362 B | 18.98 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 123.73 ms | 939.31 µs | 121.92 ms | 124.91 ms | 49.2 MB | 362 B | 18.55 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 122.59 ms | 1.20 ms | 120.90 ms | 125.46 ms | 49.2 MB | 362 B | 18.95 ms as a whole process |  |
| Seal | Normal | 33.03 ms | 1.20 ms | 32.02 ms | 33.87 ms | 2.7 MB | 160 B | 73.00 µs cold, 208 ns warm; 2.67 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 70.54 ms | 78.81 µs | 70.44 ms | 71.57 ms | 4.8 MB | 143 B | 4.89 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.85 ms | 74.59 µs | 33.70 ms | 33.92 ms | 1.9 MB | 143 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.76 ms | 155.28 µs | 7.65 ms | 7.83 ms | 2.1 MB | 143 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Normal | 96.83 ms | 206.16 µs | 96.37 ms | 94.39 ms | 43.3 MB | 137 B | 18.96 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.78 ms | 525.62 µs | 9.45 ms | 9.98 ms | 48.5 MB | 137 B | 18.92 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.11 ms | 214.73 µs | 9.68 ms | 11.15 ms | 48.6 MB | 137 B | 18.33 ms as a whole process |  |
| Seal | Ceil.source | 36.70 ms | 1.46 ms | 34.54 ms | 32.61 ms | 2.7 MB | 160 B | 78.67 µs cold, 208 ns warm; 2.50 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 34.28 ms | 483.53 µs | 33.73 ms | 53.07 ms | 2.7 MB | 160 B | 878.33 µs cold, 583 ns warm; 2.58 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 32.06 ms | 143.92 µs | 31.88 ms | 54.11 ms | 2.5 MB | 160 B | 656.54 µs cold, 458 ns warm; 2.54 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 34.88 ms | 1.83 ms | 29.68 ms | 49.77 ms | 2.5 MB | 160 B | 682.08 µs cold, 500 ns warm; 2.54 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 70.80 ms | 357.75 µs | 70.18 ms | 71.24 ms | 4.8 MB | 143 B | 4.98 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.84 ms | 45.98 µs | 33.75 ms | 33.78 ms | 1.9 MB | 143 B | 2.50 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.08 ms | 25.82 µs | 7.04 ms | 7.25 ms | 2.2 MB | 143 B | 2.42 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 93.78 ms | 621.06 µs | 92.90 ms | 94.21 ms | 43.4 MB | 137 B | 18.55 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.53 ms | 185.19 µs | 9.39 ms | 10.79 ms | 48.7 MB | 137 B | 18.86 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.10 ms | 164.97 µs | 9.67 ms | 10.83 ms | 48.5 MB | 137 B | 18.64 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.11 ms | 349.34 µs | 47.60 ms | 46.95 ms | 2.7 MB | 547 B | 106.08 µs cold, 208 ns warm; 2.61 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.08 ms | 328.71 µs | 91.54 ms | 90.98 ms | 4.8 MB | 433 B | 5.04 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.74 ms | 371.03 µs | 76.05 ms | 76.24 ms | 1.9 MB | 433 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.73 ms | 65.55 µs | 6.56 ms | 6.69 ms | 2.0 MB | 433 B | 2.16 ms as a whole process |  |
| Node `--jitless` | Purpose | 246.29 ms | 498.20 µs | 245.74 ms | 243.58 ms | 46.4 MB | 460 B | 18.16 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.72 ms | 275.07 µs | 18.24 ms | 19.27 ms | 50.3 MB | 460 B | 18.66 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.22 ms | 105.51 µs | 18.11 ms | 19.04 ms | 50.2 MB | 460 B | 18.65 ms as a whole process |  |
| Seal | Normal | 64.09 ms | 359.41 µs | 63.79 ms | 64.76 ms | 3.3 MB | 3.5 KB | 278.96 µs cold, 250 ns warm; 2.84 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.18 ms | 148.82 µs | 69.91 ms | 70.06 ms | 4.8 MB | 3.0 KB | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.99 ms | 148.36 µs | 71.77 ms | 72.10 ms | 1.9 MB | 3.0 KB | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.22 ms | 18.21 µs | 5.20 ms | 5.41 ms | 2.2 MB | 3.0 KB | 2.42 ms as a whole process |  |
| Node `--jitless` | Normal | 177.46 ms | 3.67 ms | 172.94 ms | 182.38 ms | 48.2 MB | 3.1 KB | 19.67 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.89 ms | 20.98 µs | 3.86 ms | 5.63 ms | 52.0 MB | 3.1 KB | 18.61 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 18.81 µs | 3.84 ms | 5.73 ms | 51.9 MB | 3.1 KB | 18.86 ms as a whole process |  |
| Seal | Ceil.source | 54.92 ms | 255.14 µs | 54.53 ms | 59.06 ms | 3.2 MB | 3.4 KB | 259.29 µs cold, 250 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 57.57 ms | 378.17 µs | 56.97 ms | 58.52 ms | 3.1 MB | 3.4 KB | 251.92 µs cold, 250 ns warm; 2.79 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.31 ms | 259.36 µs | 48.76 ms | 49.23 ms | 3.0 MB | 3.4 KB | 264.88 µs cold, 250 ns warm; 2.74 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.70 ms | 305.60 µs | 49.24 ms | 49.90 ms | 3.0 MB | 3.4 KB | 297.33 µs cold, 250 ns warm; 2.80 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.59 ms | 197.13 µs | 69.24 ms | 69.83 ms | 4.8 MB | 3.0 KB | 5.12 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.76 ms | 275.21 µs | 71.43 ms | 71.37 ms | 1.9 MB | 3.0 KB | 2.57 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.19 ms | 21.23 µs | 5.16 ms | 5.37 ms | 2.2 MB | 3.0 KB | 2.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.59 ms | 3.65 ms | 172.61 ms | 180.31 ms | 48.1 MB | 3.1 KB | 18.81 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 16.39 µs | 3.85 ms | 5.66 ms | 52.0 MB | 3.1 KB | 18.36 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 18.66 µs | 3.85 ms | 5.70 ms | 52.0 MB | 3.1 KB | 18.19 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 50.48 ms | 618.91 µs | 49.83 ms | 50.46 ms | 2.9 MB | 426 B | 88.62 µs cold, 208 ns warm; 2.48 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.53 ms | 245.75 µs | 45.26 ms | 45.09 ms | 4.8 MB | 366 B | 4.85 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.04 ms | 55.48 µs | 76.97 ms | 76.95 ms | 1.9 MB | 366 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.10 ms | 94.96 µs | 21.95 ms | 22.45 ms | 2.1 MB | 366 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.87 ms | 6.53 ms | 363.25 ms | 378.25 ms | 48.7 MB | 398 B | 18.88 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.40 ms | 36.73 ms | 15.69 ms | 20.97 ms | 52.8 MB | 398 B | 18.38 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.41 ms | 49.61 µs | 14.33 ms | 20.45 ms | 55.2 MB | 398 B | 18.32 ms as a whole process |  |
| Seal | Normal | 18.61 ms | 412.85 µs | 17.61 ms | 18.62 ms | 3.0 MB | 1.2 KB | 139.21 µs cold, 250 ns warm; 2.64 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.34 ms | 201.96 µs | 26.98 ms | 27.42 ms | 4.8 MB | 981 B | 5.12 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 46.11 ms | 638.94 µs | 45.55 ms | 46.43 ms | 2.0 MB | 981 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.53 ms | 10.54 µs | 1.52 ms | 1.68 ms | 2.2 MB | 981 B | 2.37 ms as a whole process |  |
| Node `--jitless` | Normal | 67.04 ms | 1.24 ms | 65.44 ms | 65.19 ms | 48.4 MB | 1.0 KB | 18.74 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.62 ms | 198.81 µs | 1.50 ms | 3.52 ms | 52.9 MB | 1.0 KB | 18.83 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.66 ms | 25.81 µs | 1.61 ms | 3.60 ms | 52.8 MB | 1.0 KB | 18.60 ms as a whole process |  |
| Seal | Ceil.source | 18.54 ms | 237.27 µs | 18.27 ms | 19.29 ms | 3.0 MB | 1.2 KB | 140.96 µs cold, 208 ns warm; 2.77 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.14 ms | 168.94 µs | 18.85 ms | 19.18 ms | 3.1 MB | 1.2 KB | 152.12 µs cold, 250 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.72 ms | 50.50 µs | 18.62 ms | 18.72 ms | 2.8 MB | 1.2 KB | 125.00 µs cold, 167 ns warm; 2.52 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.93 ms | 65.21 µs | 18.81 ms | 18.83 ms | 2.8 MB | 1.2 KB | 126.17 µs cold, 250 ns warm; 2.68 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 26.96 ms | 393.36 µs | 26.43 ms | 27.55 ms | 4.8 MB | 981 B | 5.06 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 46.71 ms | 417.58 µs | 46.15 ms | 47.03 ms | 2.0 MB | 981 B | 2.50 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.53 ms | 30.16 µs | 1.50 ms | 1.71 ms | 2.2 MB | 981 B | 2.44 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.59 ms | 488.82 µs | 63.87 ms | 63.95 ms | 48.4 MB | 1.0 KB | 18.72 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.63 ms | 158.38 µs | 1.51 ms | 3.64 ms | 52.9 MB | 1.0 KB | 18.81 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.68 ms | 20.38 µs | 1.65 ms | 3.26 ms | 52.9 MB | 1.0 KB | 18.54 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 87.41 ms | 554.97 µs | 86.61 ms | 85.10 ms | 2.8 MB | 629 B | 98.92 µs cold, 209 ns warm; 2.72 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 88.50 ms | 569.63 µs | 87.72 ms | 88.78 ms | 4.8 MB | 523 B | 5.08 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.01 ms | 655.31 µs | 51.75 ms | 53.37 ms | 1.9 MB | 523 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.21 ms | 52.44 µs | 6.12 ms | 6.57 ms | 2.1 MB | 523 B | 2.53 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.80 ms | 584.25 µs | 147.74 ms | 151.26 ms | 43.4 MB | 568 B | 18.93 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.69 ms | 95.78 µs | 3.63 ms | 4.49 ms | 49.4 MB | 568 B | 18.67 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.66 ms | 20.59 µs | 3.63 ms | 4.47 ms | 49.4 MB | 568 B | 18.45 ms as a whole process |  |
| Seal | Normal | 103.44 ms | 158.03 µs | 103.24 ms | 103.10 ms | 3.0 MB | 1.8 KB | 158.96 µs cold, 209 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 210.98 ms | 2.18 ms | 205.18 ms | 211.50 ms | 4.8 MB | 1.7 KB | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.37 ms | 407.20 µs | 146.82 ms | 146.41 ms | 1.9 MB | 1.7 KB | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.18 ms | 340.55 µs | 47.71 ms | 48.50 ms | 2.3 MB | 1.7 KB | 2.53 ms as a whole process |  |
| Node `--jitless` | Normal | 155.02 ms | 921.85 µs | 154.23 ms | 158.47 ms | 45.3 MB | 1.7 KB | 18.29 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.47 ms | 114.47 µs | 14.31 ms | 15.79 ms | 52.0 MB | 1.7 KB | 18.88 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.12 ms | 120.96 µs | 13.96 ms | 15.74 ms | 52.1 MB | 1.7 KB | 18.65 ms as a whole process |  |
| Seal | Ceil.source | 102.59 ms | 318.21 µs | 102.20 ms | 103.80 ms | 3.0 MB | 1.8 KB | 140.92 µs cold, 250 ns warm; 2.82 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 105.21 ms | 572.31 µs | 104.60 ms | 106.55 ms | 3.0 MB | 1.8 KB | 149.58 µs cold, 208 ns warm; 2.71 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 102.64 ms | 163.50 µs | 102.31 ms | 102.51 ms | 2.8 MB | 1.8 KB | 137.88 µs cold, 208 ns warm; 2.59 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 104.00 ms | 94.19 µs | 103.82 ms | 103.88 ms | 2.8 MB | 1.8 KB | 129.29 µs cold, 250 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 210.45 ms | 1.92 ms | 205.09 ms | 211.67 ms | 4.8 MB | 1.7 KB | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 146.67 ms | 381.35 µs | 146.21 ms | 144.09 ms | 1.9 MB | 1.7 KB | 2.42 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 47.99 ms | 288.97 µs | 47.60 ms | 48.90 ms | 2.3 MB | 1.7 KB | 2.46 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.86 ms | 431.70 µs | 157.34 ms | 158.89 ms | 45.4 MB | 1.7 KB | 18.83 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 13.94 ms | 71.82 µs | 13.82 ms | 15.68 ms | 52.0 MB | 1.7 KB | 18.48 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.72 ms | 152.66 µs | 14.46 ms | 16.19 ms | 52.0 MB | 1.7 KB | 18.29 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 124.78 ms | 1.49 ms | 123.06 ms | 125.64 ms | 4.0 MB | 566 B | 95.79 µs cold, 292 ns warm; 2.79 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.94 ms | 264.85 µs | 62.50 ms | 62.85 ms | 4.8 MB | 540 B | 5.02 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 28.59 ms | 127.47 µs | 28.40 ms | 28.94 ms | 2.1 MB | 540 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.94 ms | 20.69 µs | 4.91 ms | 5.21 ms | 2.3 MB | 540 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Purpose | 240.55 ms | 1.07 ms | 239.22 ms | 243.76 ms | 48.4 MB | 514 B | 19.55 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 85.67 ms | 23.30 ms | 64.67 ms | 87.76 ms | 52.5 MB | 514 B | 18.37 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 64.51 ms | 164.43 µs | 64.26 ms | 90.18 ms | 52.5 MB | 514 B | 18.53 ms as a whole process |  |
| Seal | Normal | 26.48 ms | 110.13 µs | 26.26 ms | 26.44 ms | 4.0 MB | 535 B | 102.92 µs cold, 291 ns warm; 2.55 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.45 ms | 291.38 µs | 29.83 ms | 29.93 ms | 6.0 MB | 478 B | 4.96 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.42 ms | 404.97 µs | 30.38 ms | 31.25 ms | 3.0 MB | 478 B | 2.38 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.86 ms | 628.46 µs | 12.40 ms | 13.24 ms | 3.3 MB | 478 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Normal | 51.21 ms | 184.17 µs | 50.93 ms | 52.49 ms | 56.5 MB | 477 B | 18.42 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.33 ms | 271.89 µs | 34.92 ms | 35.98 ms | 62.6 MB | 477 B | 18.18 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.32 ms | 245.42 µs | 34.89 ms | 35.47 ms | 88.0 MB | 477 B | 18.73 ms as a whole process |  |
| Seal | Ceil.source | 26.43 ms | 125.52 µs | 26.17 ms | 26.62 ms | 3.9 MB | 535 B | 103.29 µs cold, 375 ns warm; 2.65 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.53 ms | 76.94 µs | 27.43 ms | 27.67 ms | 4.0 MB | 535 B | 114.00 µs cold, 333 ns warm; 2.54 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.89 ms | 206.43 µs | 23.60 ms | 23.76 ms | 4.0 MB | 535 B | 109.79 µs cold, 250 ns warm; 2.74 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.06 ms | 97.49 µs | 23.92 ms | 24.10 ms | 3.9 MB | 535 B | 88.38 µs cold, 250 ns warm; 2.72 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.41 ms | 205.27 µs | 29.91 ms | 30.16 ms | 6.0 MB | 478 B | 5.10 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 31.54 ms | 473.30 µs | 30.89 ms | 29.58 ms | 3.0 MB | 478 B | 2.48 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.12 ms | 603.83 µs | 11.27 ms | 12.25 ms | 3.3 MB | 478 B | 2.41 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.36 ms | 309.57 µs | 51.85 ms | 53.60 ms | 56.4 MB | 477 B | 18.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.01 ms | 367.47 µs | 34.67 ms | 35.39 ms | 62.4 MB | 477 B | 18.89 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.17 ms | 236.11 µs | 34.86 ms | 35.81 ms | 88.4 MB | 477 B | 18.20 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 305.48 µs | 4.94 µs | 300.33 µs | 308.58 µs | 3.1 MB | 230 B | 87.79 µs cold, 333 ns warm; 2.54 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 142.90 ms | 512.29 µs | 142.33 ms | 146.33 ms | 9.7 MB | 195 B | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.97 ms | 94.73 µs | 32.83 ms | 31.28 ms | 3.5 MB | 195 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.80 ms | 63.47 µs | 20.71 ms | 20.95 ms | 3.7 MB | 195 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.09 µs | 141.61 µs | 247.08 µs | 1.04 ms | 52.5 MB | 223 B | 18.44 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 218.83 µs | 149.00 µs | 91.21 µs | 881.00 µs | 57.7 MB | 223 B | 18.83 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 171.32 µs | 163.93 µs | 62.92 µs | 893.79 µs | 58.4 MB | 223 B | 18.43 ms as a whole process |  |
| Seal | Normal | 343.26 µs | 5.50 µs | 338.21 µs | 376.79 µs | 3.2 MB | 179 B | 81.71 µs cold, 291 ns warm; 2.58 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 161.17 ms | 992.71 µs | 158.94 ms | 161.80 ms | 11.4 MB | 138 B | 5.01 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.83 ms | 225.10 µs | 41.50 ms | 42.06 ms | 5.0 MB | 138 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.61 ms | 95.10 µs | 19.51 ms | 19.51 ms | 5.2 MB | 138 B | 2.45 ms as a whole process |  |
| Node `--jitless` | Normal | 509.55 µs | 341.12 µs | 281.17 µs | 1.09 ms | 61.4 MB | 155 B | 18.81 ms as a whole process |  |
| Node (JIT), from cold | Normal | 489.41 µs | 476.28 µs | 136.08 µs | 1.04 ms | 65.2 MB | 155 B | 18.74 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 274.32 µs | 324.40 µs | 122.92 µs | 1.09 ms | 69.4 MB | 155 B | 19.24 ms as a whole process |  |
| Seal | Ceil.source | 341.64 µs | 5.87 µs | 336.50 µs | 355.46 µs | 3.2 MB | 179 B | 72.25 µs cold, 333 ns warm; 2.60 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 343.16 µs | 5.23 µs | 338.12 µs | 380.50 µs | 3.2 MB | 179 B | 80.50 µs cold, 458 ns warm; 2.61 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 294.15 µs | 4.22 µs | 289.96 µs | 311.33 µs | 3.1 MB | 179 B | 59.58 µs cold, 292 ns warm; 2.56 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 284.69 µs | 5.79 µs | 279.71 µs | 307.50 µs | 3.1 MB | 179 B | 59.25 µs cold, 167 ns warm; 2.44 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 158.80 ms | 222.83 µs | 158.41 ms | 163.41 ms | 11.3 MB | 138 B | 5.09 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.74 ms | 157.60 µs | 41.56 ms | 40.87 ms | 4.9 MB | 138 B | 2.44 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.52 ms | 323.85 µs | 19.33 ms | 20.24 ms | 5.2 MB | 138 B | 2.48 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 520.87 µs | 347.03 µs | 282.04 µs | 1.11 ms | 61.4 MB | 155 B | 18.44 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 504.73 µs | 489.44 µs | 118.79 µs | 1.07 ms | 65.3 MB | 155 B | 18.51 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 281.75 µs | 328.77 µs | 124.62 µs | 1.06 ms | 69.4 MB | 155 B | 18.32 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 73.71 ms | 667.42 µs | 72.41 ms | 74.99 ms | 2.9 MB | 528 B | 94.04 µs cold, 208 ns warm; 2.57 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 95.24 ms | 199.55 µs | 94.76 ms | 94.96 ms | 4.8 MB | 411 B | 5.02 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 117.91 ms | 1.87 ms | 113.73 ms | 118.30 ms | 2.5 MB | 411 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.51 ms | 378.77 µs | 94.02 ms | 95.30 ms | 2.9 MB | 411 B | 2.38 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.16 ms | 270.87 µs | 74.79 ms | 78.46 ms | 59.7 MB | 444 B | 18.28 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.46 ms | 310.61 µs | 10.09 ms | 12.87 ms | 65.8 MB | 444 B | 19.13 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.39 ms | 612.77 µs | 10.06 ms | 12.97 ms | 114.6 MB | 444 B | 18.59 ms as a whole process |  |
| Seal | Normal | 99.18 ms | 734.49 µs | 97.82 ms | 99.15 ms | 7.8 MB | 977 B | 138.33 µs cold, 250 ns warm; 2.64 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 131.84 ms | 1.00 ms | 130.02 ms | 130.64 ms | 42.6 MB | 799 B | 4.92 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 130.09 ms | 1.06 ms | 127.68 ms | 128.69 ms | 41.5 MB | 799 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 80.97 ms | 1.07 ms | 79.53 ms | 80.32 ms | 39.9 MB | 799 B | 2.42 ms as a whole process |  |
| Node `--jitless` | Normal | 114.30 ms | 1.58 ms | 111.12 ms | 117.89 ms | 213.3 MB | 878 B | 18.62 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.18 ms | 925.14 µs | 13.20 ms | 17.09 ms | 129.7 MB | 878 B | 18.76 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.32 ms | 120.68 µs | 13.08 ms | 16.87 ms | 224.0 MB | 878 B | 19.13 ms as a whole process |  |
| Seal | Ceil.source | 99.68 ms | 533.92 µs | 99.03 ms | 101.78 ms | 7.8 MB | 977 B | 124.67 µs cold, 250 ns warm; 2.64 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 99.47 ms | 498.00 µs | 98.66 ms | 100.30 ms | 7.8 MB | 977 B | 122.71 µs cold, 292 ns warm; 2.64 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 85.19 ms | 1.11 ms | 83.76 ms | 86.62 ms | 7.6 MB | 977 B | 103.58 µs cold, 250 ns warm; 2.45 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 85.28 ms | 260.50 µs | 84.98 ms | 85.71 ms | 7.7 MB | 977 B | 102.88 µs cold, 209 ns warm; 2.65 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 127.89 ms | 1.56 ms | 125.31 ms | 128.58 ms | 40.0 MB | 799 B | 5.06 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.48 ms | 980.66 µs | 126.82 ms | 128.00 ms | 41.5 MB | 799 B | 2.36 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 81.81 ms | 1.06 ms | 79.66 ms | 80.56 ms | 43.7 MB | 799 B | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 113.84 ms | 2.80 ms | 109.30 ms | 125.67 ms | 213.4 MB | 878 B | 18.74 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.13 ms | 856.09 µs | 13.30 ms | 17.08 ms | 129.4 MB | 878 B | 18.77 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.27 ms | 37.95 µs | 13.23 ms | 16.84 ms | 224.2 MB | 878 B | 19.46 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.59 µs (±6.70 µs, least 44.42 µs) | 18.09 µs (±2.95 µs, least 13.04 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±1 ns, least 40 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.47 µs (±3.03 µs, least 8.62 µs) | 16.32 µs (±1.22 µs, least 14.71 µs) | 42 ns (±2 ns, least 40 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.90 µs (±316 ns, least 3.00 µs) | 7.90 µs (±1.60 µs, least 6.00 µs) | 21 ns (±2 ns, least 19 ns) | 15 ns (±1 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.60 µs (±516 ns, least 3.00 µs) | 7.60 µs (±966 ns, least 6.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 37.20 µs (±18.07 µs, least 20.62 µs) | 38.31 µs (±2.21 µs, least 35.75 µs) | 57 ns (±1 ns, least 56 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 33.14 µs (±13.53 µs, least 18.17 µs) | 38.55 µs (±3.49 µs, least 34.54 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.79 µs (±43 ns, least 4.72 µs) | 155 ns (±6 ns, least 147 ns) | 20 ns (±0 ns, least 20 ns) | 39 ns (±1 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.89 µs (±106 ns, least 4.71 µs) | 189 ns (±9 ns, least 174 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.78 µs (±28 ns, least 1.75 µs) | 177 ns (±6 ns, least 168 ns) | 27 ns (±1 ns, least 26 ns) | 15 ns (±1 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.81 µs (±22 ns, least 1.78 µs) | 366 ns (±35 ns, least 327 ns) | 92 ns (±2 ns, least 89 ns) | 102 ns (±2 ns, least 100 ns) | 1.0 KB | 4.6 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±31 ns, least 1.10 µs) | 665 ns (±15 ns, least 641 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.09 µs (±14 ns, least 1.07 µs) | 665 ns (±18 ns, least 640 ns) | 22 ns (±1 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.79 µs (±44 ns, least 4.71 µs) | 150 ns (±11 ns, least 137 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.85 µs (±60 ns, least 4.76 µs) | 153 ns (±6 ns, least 144 ns) | 21 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.21 µs (±43 ns, least 4.16 µs) | 142 ns (±5 ns, least 134 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.26 µs (±47 ns, least 4.18 µs) | 141 ns (±7 ns, least 134 ns) | 18 ns (±0 ns, least 18 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.87 µs (±156 ns, least 4.64 µs) | 193 ns (±11 ns, least 180 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.80 µs (±13 ns, least 1.78 µs) | 177 ns (±5 ns, least 171 ns) | 28 ns (±0 ns, least 27 ns) | 14 ns (±1 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.82 µs (±25 ns, least 1.79 µs) | 357 ns (±11 ns, least 347 ns) | 93 ns (±3 ns, least 91 ns) | 105 ns (±11 ns, least 100 ns) | 1.0 KB | 4.8 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±18 ns, least 1.11 µs) | 671 ns (±18 ns, least 645 ns) | 50 ns (±1 ns, least 49 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±27 ns, least 1.05 µs) | 672 ns (±12 ns, least 655 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 36.13 ms | 30.91 ms (-14.4%) |
| n-body, 100,000 steps | 65.77 ms | 59.91 ms (-8.9%) |
| spectral-norm, size 200 | 18.43 ms | 18.22 ms (-1.2%) |
| fannkuch-redux, 9 | 103.56 ms | 101.83 ms (-1.7%) |
| word count, 500,000 words | 26.44 ms | 26.53 ms (+0.3%) |
| text building, 20,000 joins | 340.25 µs | 339.85 µs (-0.1%) |
| binary-trees, depth 14 | 99.18 ms | 99.17 ms (-0.0%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 175.96 ms | 330.61 µs | 175.52 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.37 ms | 271.78 µs | 10.03 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.40 ms | 48.21 µs | 4.34 ms | 3.4 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.40 ms | 22.63 µs | 1.37 ms | 3.3 MB |
