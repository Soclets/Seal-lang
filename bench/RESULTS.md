# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.05 1.82 1.83 }` before, `{ 1.64 1.76 1.81 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 41.55 ms | 71.41 ms | **met**: Seal takes 58% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.12 ms | 69.77 ms | **met**: Seal takes 95% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.11 ms | 37.52 ms | **met**: Seal takes 54% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 114.42 ms | 214.83 ms | **met**: Seal takes 53% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.71 ms | 30.16 ms | **met**: Seal takes 89% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 345.04 µs | 158.90 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 103.71 ms | 129.87 ms | **met**: Seal takes 80% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.86 ms | 3.51 ms | **met**: Seal takes 53% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 33 ns | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 144 ns | 182 ns | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.78 µs | 4.90 µs | **met**: Seal takes 98% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 41.55 ms | 33.87 ms | **not met**: Seal takes 123% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 66.12 ms | 71.64 ms | **met**: Seal takes 92% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 20.11 ms | 45.45 ms | **met**: Seal takes 44% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 114.42 ms | 146.56 ms | **met**: Seal takes 78% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.71 ms | 31.39 ms | **met**: Seal takes 85% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 345.04 µs | 42.38 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 103.71 ms | 127.37 ms | **met**: Seal takes 81% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.86 ms | 2.91 ms | **met**: Seal takes 64% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 17 ns | 27 ns | **met**: Seal takes 63% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 144 ns | 182 ns | **met**: Seal takes 79% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.78 µs | 1.78 µs | **not met**: Seal takes 269% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 41.55 ms | 91.76 ms | **met**: Seal takes 45% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.12 ms | 177.02 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.71 ms | 51.88 ms | **met**: Seal takes 51% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 345.04 µs | 500.09 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.44 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 99.89 ms | 173.09 µs | 99.66 ms | 119.88 ms | 2.8 MB | 386 B | 749.08 µs cold, 417 ns warm; 2.79 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 197.44 ms | 266.95 µs | 196.77 ms | 196.83 ms | 4.8 MB | 339 B | 5.17 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 87.83 ms | 418.31 µs | 87.24 ms | 87.93 ms | 1.9 MB | 339 B | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.23 ms | 648.54 µs | 43.78 ms | 45.60 ms | 2.2 MB | 339 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Purpose | 327.77 ms | 4.52 ms | 321.76 ms | 322.04 ms | 43.4 MB | 362 B | 18.65 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.68 ms | 310.61 µs | 122.15 ms | 123.20 ms | 49.1 MB | 362 B | 18.73 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 121.95 ms | 545.22 µs | 121.27 ms | 122.61 ms | 49.1 MB | 362 B | 19.01 ms as a whole process |  |
| Seal | Normal | 41.55 ms | 128.55 µs | 41.33 ms | 42.57 ms | 2.7 MB | 160 B | 77.04 µs cold, 583 ns warm; 2.69 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 71.41 ms | 328.67 µs | 71.03 ms | 71.53 ms | 4.8 MB | 143 B | 5.01 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.87 ms | 58.28 µs | 33.76 ms | 33.73 ms | 1.9 MB | 143 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 6.95 ms | 21.33 µs | 6.92 ms | 7.13 ms | 2.1 MB | 143 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Normal | 91.76 ms | 240.39 µs | 91.44 ms | 93.25 ms | 43.2 MB | 137 B | 19.09 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.82 ms | 433.12 µs | 9.52 ms | 10.00 ms | 48.5 MB | 137 B | 18.39 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.51 ms | 133.55 µs | 9.41 ms | 10.63 ms | 48.6 MB | 137 B | 18.68 ms as a whole process |  |
| Seal | Ceil.source | 40.87 ms | 523.49 µs | 39.54 ms | 38.32 ms | 2.7 MB | 160 B | 76.00 µs cold, 166 ns warm; 2.50 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 36.24 ms | 438.37 µs | 35.55 ms | 57.22 ms | 2.7 MB | 160 B | 853.12 µs cold, 500 ns warm; 2.65 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 36.99 ms | 674.81 µs | 35.84 ms | 59.58 ms | 2.5 MB | 160 B | 656.04 µs cold, 417 ns warm; 2.51 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 32.28 ms | 150.18 µs | 32.04 ms | 55.89 ms | 2.5 MB | 160 B | 898.92 µs cold, 667 ns warm; 2.45 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 70.93 ms | 279.89 µs | 70.43 ms | 71.54 ms | 4.8 MB | 143 B | 5.03 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.87 ms | 103.29 µs | 33.64 ms | 33.87 ms | 1.9 MB | 143 B | 2.49 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.41 ms | 26.74 µs | 7.37 ms | 7.54 ms | 2.1 MB | 143 B | 2.31 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.52 ms | 277.05 µs | 96.13 ms | 97.58 ms | 43.3 MB | 137 B | 18.42 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.81 ms | 578.11 µs | 9.48 ms | 9.86 ms | 48.5 MB | 137 B | 18.41 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.67 ms | 430.20 µs | 9.49 ms | 12.00 ms | 48.6 MB | 137 B | 18.47 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.40 ms | 191.94 µs | 53.17 ms | 53.65 ms | 2.7 MB | 547 B | 107.12 µs cold, 208 ns warm; 2.57 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.15 ms | 368.05 µs | 91.64 ms | 91.35 ms | 4.8 MB | 433 B | 5.04 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.81 ms | 339.87 µs | 76.22 ms | 76.18 ms | 1.9 MB | 433 B | 2.40 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.72 ms | 69.08 µs | 6.56 ms | 6.67 ms | 2.1 MB | 433 B | 2.33 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.93 ms | 1.19 ms | 244.33 ms | 244.72 ms | 46.4 MB | 460 B | 18.70 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.21 ms | 80.31 µs | 18.12 ms | 18.23 ms | 50.2 MB | 460 B | 17.87 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.21 ms | 59.26 µs | 18.14 ms | 18.32 ms | 50.3 MB | 460 B | 18.62 ms as a whole process |  |
| Seal | Normal | 66.12 ms | 934.20 µs | 64.63 ms | 65.56 ms | 3.3 MB | 3.5 KB | 290.00 µs cold, 208 ns warm; 2.87 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.77 ms | 262.63 µs | 69.27 ms | 69.91 ms | 4.8 MB | 3.0 KB | 4.96 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.64 ms | 126.11 µs | 71.48 ms | 71.44 ms | 1.9 MB | 3.0 KB | 2.36 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.22 ms | 21.91 µs | 5.20 ms | 5.41 ms | 2.2 MB | 3.0 KB | 2.36 ms as a whole process |  |
| Node `--jitless` | Normal | 177.02 ms | 4.42 ms | 172.17 ms | 180.14 ms | 48.1 MB | 3.1 KB | 18.62 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.88 ms | 12.88 µs | 3.86 ms | 5.71 ms | 52.0 MB | 3.1 KB | 18.74 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 20.84 µs | 3.84 ms | 5.67 ms | 52.1 MB | 3.1 KB | 18.81 ms as a whole process |  |
| Seal | Ceil.source | 54.71 ms | 393.46 µs | 54.12 ms | 55.06 ms | 3.2 MB | 3.4 KB | 254.08 µs cold, 250 ns warm; 2.70 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.67 ms | 515.40 µs | 54.76 ms | 57.28 ms | 3.2 MB | 3.4 KB | 250.75 µs cold, 250 ns warm; 2.78 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 51.04 ms | 229.12 µs | 50.75 ms | 50.72 ms | 3.0 MB | 3.4 KB | 269.54 µs cold, 250 ns warm; 2.89 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.93 ms | 365.90 µs | 50.28 ms | 50.97 ms | 2.9 MB | 3.4 KB | 272.75 µs cold, 209 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.39 ms | 234.22 µs | 68.93 ms | 69.70 ms | 4.8 MB | 3.0 KB | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.72 ms | 196.33 µs | 71.44 ms | 71.07 ms | 1.9 MB | 3.0 KB | 2.54 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.26 ms | 28.66 µs | 5.21 ms | 5.45 ms | 2.2 MB | 3.0 KB | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.54 ms | 4.10 ms | 171.95 ms | 181.31 ms | 48.3 MB | 3.1 KB | 18.88 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 20.11 µs | 3.86 ms | 5.66 ms | 52.0 MB | 3.1 KB | 18.40 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.88 ms | 18.29 µs | 3.84 ms | 5.67 ms | 51.9 MB | 3.1 KB | 18.49 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 58.58 ms | 1.10 ms | 56.90 ms | 59.92 ms | 2.9 MB | 426 B | 92.71 µs cold, 250 ns warm; 2.61 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 51.81 ms | 2.68 ms | 49.24 ms | 45.08 ms | 4.8 MB | 366 B | 5.07 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.01 ms | 69.29 µs | 76.91 ms | 77.04 ms | 1.9 MB | 366 B | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.08 ms | 82.85 µs | 21.95 ms | 22.48 ms | 2.1 MB | 366 B | 2.41 ms as a whole process |  |
| Node `--jitless` | Purpose | 366.19 ms | 6.13 ms | 360.81 ms | 379.72 ms | 48.6 MB | 398 B | 19.33 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.36 ms | 36.63 ms | 15.59 ms | 20.79 ms | 52.5 MB | 398 B | 18.82 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.44 ms | 110.68 µs | 14.29 ms | 21.18 ms | 55.3 MB | 398 B | 19.19 ms as a whole process |  |
| Seal | Normal | 20.11 ms | 69.22 µs | 19.99 ms | 20.14 ms | 3.0 MB | 1.2 KB | 150.08 µs cold, 250 ns warm; 2.74 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 37.52 ms | 352.00 µs | 36.87 ms | 30.88 ms | 4.8 MB | 981 B | 5.07 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.45 ms | 304.23 µs | 44.89 ms | 46.16 ms | 2.0 MB | 981 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.53 ms | 11.45 µs | 1.51 ms | 1.71 ms | 2.2 MB | 981 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 63.55 ms | 156.23 µs | 63.18 ms | 63.47 ms | 48.5 MB | 1.0 KB | 18.98 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.68 ms | 239.72 µs | 1.51 ms | 3.68 ms | 52.9 MB | 1.0 KB | 18.83 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.65 ms | 34.11 µs | 1.62 ms | 3.24 ms | 52.8 MB | 1.0 KB | 18.35 ms as a whole process |  |
| Seal | Ceil.source | 20.13 ms | 46.71 µs | 20.07 ms | 20.22 ms | 3.0 MB | 1.2 KB | 140.38 µs cold, 250 ns warm; 2.61 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.13 ms | 87.27 µs | 20.00 ms | 20.15 ms | 3.0 MB | 1.2 KB | 144.79 µs cold, 250 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.68 ms | 61.55 µs | 19.56 ms | 19.58 ms | 2.8 MB | 1.2 KB | 130.12 µs cold, 209 ns warm; 2.66 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 19.66 ms | 41.31 µs | 19.59 ms | 19.72 ms | 2.8 MB | 1.2 KB | 139.88 µs cold, 209 ns warm; 2.58 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.44 ms | 459.45 µs | 26.77 ms | 27.16 ms | 4.8 MB | 981 B | 5.05 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 47.14 ms | 333.06 µs | 46.61 ms | 47.43 ms | 2.0 MB | 981 B | 2.32 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.53 ms | 17.75 µs | 1.51 ms | 1.77 ms | 2.2 MB | 981 B | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.56 ms | 1.01 ms | 62.48 ms | 63.63 ms | 48.5 MB | 1.0 KB | 18.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.65 ms | 189.66 µs | 1.49 ms | 3.73 ms | 52.9 MB | 1.0 KB | 18.75 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.66 ms | 18.68 µs | 1.63 ms | 3.27 ms | 52.8 MB | 1.0 KB | 18.85 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.80 ms | 467.94 µs | 97.36 ms | 97.11 ms | 2.7 MB | 629 B | 115.88 µs cold, 208 ns warm; 2.58 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 88.29 ms | 1.18 ms | 86.89 ms | 87.40 ms | 4.8 MB | 523 B | 4.96 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.93 ms | 578.77 µs | 51.93 ms | 52.48 ms | 1.9 MB | 523 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.17 ms | 18.76 µs | 6.14 ms | 6.29 ms | 2.1 MB | 523 B | 2.32 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.74 ms | 580.24 µs | 146.09 ms | 146.41 ms | 43.4 MB | 568 B | 18.72 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.67 ms | 22.51 µs | 3.65 ms | 4.45 ms | 49.4 MB | 568 B | 18.71 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.68 ms | 41.15 µs | 3.64 ms | 4.48 ms | 49.3 MB | 568 B | 18.69 ms as a whole process |  |
| Seal | Normal | 114.42 ms | 401.31 µs | 113.72 ms | 115.52 ms | 3.0 MB | 1.8 KB | 146.42 µs cold, 250 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 214.83 ms | 2.60 ms | 207.71 ms | 212.82 ms | 4.8 MB | 1.7 KB | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 146.56 ms | 360.98 µs | 146.13 ms | 144.58 ms | 1.9 MB | 1.7 KB | 2.35 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.63 ms | 183.91 µs | 48.25 ms | 48.69 ms | 2.3 MB | 1.7 KB | 2.53 ms as a whole process |  |
| Node `--jitless` | Normal | 155.22 ms | 403.29 µs | 154.68 ms | 156.90 ms | 45.4 MB | 1.7 KB | 18.65 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.60 ms | 90.48 µs | 14.45 ms | 16.15 ms | 51.9 MB | 1.7 KB | 18.81 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.14 ms | 102.01 µs | 13.96 ms | 15.57 ms | 52.0 MB | 1.7 KB | 19.54 ms as a whole process |  |
| Seal | Ceil.source | 112.36 ms | 151.85 µs | 112.13 ms | 113.89 ms | 3.0 MB | 1.8 KB | 157.50 µs cold, 250 ns warm; 2.62 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 108.51 ms | 227.26 µs | 108.23 ms | 109.03 ms | 3.0 MB | 1.8 KB | 146.92 µs cold, 250 ns warm; 2.59 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 102.53 ms | 240.89 µs | 102.20 ms | 102.62 ms | 2.7 MB | 1.8 KB | 123.08 µs cold, 208 ns warm; 2.60 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 107.50 ms | 370.96 µs | 107.04 ms | 107.27 ms | 2.8 MB | 1.8 KB | 126.62 µs cold, 167 ns warm; 2.63 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 214.95 ms | 2.41 ms | 208.21 ms | 216.86 ms | 4.9 MB | 1.7 KB | 5.12 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 145.95 ms | 235.83 µs | 145.50 ms | 145.20 ms | 1.9 MB | 1.7 KB | 2.32 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 49.78 ms | 271.75 µs | 49.22 ms | 50.02 ms | 2.3 MB | 1.7 KB | 2.47 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 179.68 ms | 12.08 ms | 162.79 ms | 160.83 ms | 45.6 MB | 1.7 KB | 18.36 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 15.24 ms | 555.69 µs | 14.56 ms | 16.19 ms | 51.9 MB | 1.7 KB | 18.64 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.60 ms | 30.64 µs | 14.55 ms | 16.15 ms | 52.1 MB | 1.7 KB | 18.44 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 129.81 ms | 331.72 µs | 129.37 ms | 129.65 ms | 4.0 MB | 566 B | 99.25 µs cold, 333 ns warm; 2.45 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.33 ms | 161.44 µs | 62.03 ms | 62.24 ms | 4.8 MB | 540 B | 4.94 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 31.01 ms | 1.23 ms | 29.18 ms | 31.32 ms | 2.1 MB | 540 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 5.31 ms | 59.33 µs | 5.15 ms | 5.33 ms | 2.3 MB | 540 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Purpose | 238.99 ms | 1.18 ms | 237.72 ms | 240.47 ms | 48.7 MB | 514 B | 18.61 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.77 ms | 24.86 ms | 64.84 ms | 87.80 ms | 52.7 MB | 514 B | 18.79 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 65.38 ms | 223.40 µs | 65.01 ms | 88.71 ms | 52.6 MB | 514 B | 19.02 ms as a whole process |  |
| Seal | Normal | 26.71 ms | 90.95 µs | 26.60 ms | 26.86 ms | 4.0 MB | 535 B | 101.04 µs cold, 292 ns warm; 2.65 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.16 ms | 167.91 µs | 29.82 ms | 30.30 ms | 5.9 MB | 478 B | 4.96 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 31.39 ms | 242.32 µs | 30.98 ms | 30.34 ms | 3.0 MB | 478 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.44 ms | 551.66 µs | 11.55 ms | 12.57 ms | 3.3 MB | 478 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Normal | 51.88 ms | 320.94 µs | 51.41 ms | 52.59 ms | 56.6 MB | 477 B | 19.00 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.33 ms | 228.57 µs | 34.88 ms | 36.08 ms | 62.4 MB | 477 B | 18.62 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.81 ms | 505.91 µs | 34.99 ms | 36.07 ms | 87.9 MB | 477 B | 18.13 ms as a whole process |  |
| Seal | Ceil.source | 26.79 ms | 157.40 µs | 26.50 ms | 27.12 ms | 4.0 MB | 535 B | 105.54 µs cold, 292 ns warm; 2.55 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 26.96 ms | 208.79 µs | 26.68 ms | 26.57 ms | 4.0 MB | 535 B | 103.54 µs cold, 292 ns warm; 2.59 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 25.55 ms | 153.11 µs | 25.31 ms | 25.19 ms | 3.9 MB | 535 B | 89.00 µs cold, 208 ns warm; 2.61 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 25.06 ms | 538.52 µs | 24.65 ms | 24.80 ms | 3.9 MB | 535 B | 84.12 µs cold, 250 ns warm; 2.45 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.24 ms | 172.78 µs | 29.84 ms | 29.90 ms | 6.0 MB | 478 B | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 30.38 ms | 340.23 µs | 29.71 ms | 30.96 ms | 3.0 MB | 478 B | 2.41 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.16 ms | 544.60 µs | 11.04 ms | 12.21 ms | 3.3 MB | 478 B | 2.27 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.53 ms | 328.41 µs | 51.17 ms | 52.76 ms | 56.5 MB | 477 B | 18.62 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.36 ms | 396.57 µs | 35.00 ms | 35.56 ms | 62.3 MB | 477 B | 18.96 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.06 ms | 312.30 µs | 34.68 ms | 36.11 ms | 87.9 MB | 477 B | 18.57 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 307.94 µs | 7.09 µs | 299.17 µs | 308.21 µs | 3.0 MB | 230 B | 69.71 µs cold, 291 ns warm; 2.64 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 141.64 ms | 408.72 µs | 141.13 ms | 143.52 ms | 9.7 MB | 195 B | 5.05 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.36 ms | 77.85 µs | 32.21 ms | 31.34 ms | 3.5 MB | 195 B | 2.29 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.57 ms | 69.33 µs | 20.45 ms | 20.79 ms | 3.7 MB | 195 B | 2.56 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.69 µs | 141.56 µs | 249.58 µs | 1.01 ms | 52.5 MB | 223 B | 18.45 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 210.37 µs | 149.75 µs | 86.12 µs | 979.54 µs | 57.8 MB | 223 B | 18.75 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 165.66 µs | 165.18 µs | 55.92 µs | 893.83 µs | 58.5 MB | 223 B | 18.91 ms as a whole process |  |
| Seal | Normal | 345.04 µs | 7.67 µs | 337.96 µs | 372.54 µs | 3.2 MB | 179 B | 76.62 µs cold, 333 ns warm; 2.51 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 158.90 ms | 706.65 µs | 158.09 ms | 163.00 ms | 11.3 MB | 138 B | 4.90 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 42.38 ms | 266.03 µs | 41.85 ms | 40.98 ms | 5.0 MB | 138 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.29 ms | 162.57 µs | 19.10 ms | 19.99 ms | 5.2 MB | 138 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Normal | 500.09 µs | 338.86 µs | 277.42 µs | 1.08 ms | 61.4 MB | 155 B | 18.70 ms as a whole process |  |
| Node (JIT), from cold | Normal | 489.75 µs | 470.44 µs | 120.58 µs | 1.06 ms | 65.2 MB | 155 B | 19.36 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 282.85 µs | 323.74 µs | 132.79 µs | 1.06 ms | 69.4 MB | 155 B | 18.08 ms as a whole process |  |
| Seal | Ceil.source | 347.02 µs | 7.33 µs | 334.67 µs | 361.54 µs | 3.2 MB | 179 B | 75.79 µs cold, 291 ns warm; 2.61 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 350.60 µs | 4.22 µs | 346.54 µs | 372.67 µs | 3.2 MB | 179 B | 72.08 µs cold, 292 ns warm; 2.58 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 293.50 µs | 5.45 µs | 288.54 µs | 327.71 µs | 3.0 MB | 179 B | 60.62 µs cold, 250 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 287.92 µs | 2.74 µs | 284.54 µs | 317.46 µs | 3.0 MB | 179 B | 60.25 µs cold, 250 ns warm; 2.38 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 158.49 ms | 357.06 µs | 157.82 ms | 160.64 ms | 11.3 MB | 138 B | 4.84 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 42.37 ms | 371.42 µs | 41.56 ms | 41.95 ms | 4.9 MB | 138 B | 2.46 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.40 ms | 148.66 µs | 19.22 ms | 20.15 ms | 5.2 MB | 138 B | 2.35 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 505.16 µs | 346.39 µs | 268.04 µs | 1.12 ms | 61.4 MB | 155 B | 18.57 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 502.18 µs | 482.45 µs | 124.96 µs | 1.11 ms | 65.2 MB | 155 B | 18.63 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 290.41 µs | 322.89 µs | 124.08 µs | 1.06 ms | 69.4 MB | 155 B | 18.55 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 76.10 ms | 742.53 µs | 75.20 ms | 78.61 ms | 2.9 MB | 528 B | 93.88 µs cold, 208 ns warm; 2.65 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 94.79 ms | 284.23 µs | 94.35 ms | 94.46 ms | 4.8 MB | 411 B | 4.95 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 118.22 ms | 753.21 µs | 116.52 ms | 118.11 ms | 2.6 MB | 411 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 93.54 ms | 955.79 µs | 91.07 ms | 95.27 ms | 2.9 MB | 411 B | 2.33 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.48 ms | 446.75 µs | 74.83 ms | 78.25 ms | 59.7 MB | 444 B | 18.30 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.53 ms | 250.79 µs | 10.24 ms | 13.08 ms | 65.7 MB | 444 B | 18.06 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.47 ms | 590.07 µs | 10.16 ms | 13.02 ms | 114.8 MB | 444 B | 18.57 ms as a whole process |  |
| Seal | Normal | 103.71 ms | 692.33 µs | 102.36 ms | 106.06 ms | 7.8 MB | 977 B | 128.33 µs cold, 14.54 µs warm; 2.70 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 129.87 ms | 1.47 ms | 128.22 ms | 129.93 ms | 42.8 MB | 799 B | 5.11 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 127.37 ms | 1.03 ms | 125.63 ms | 127.23 ms | 41.5 MB | 799 B | 2.54 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 80.71 ms | 677.34 µs | 79.77 ms | 80.86 ms | 39.8 MB | 799 B | 2.41 ms as a whole process |  |
| Node `--jitless` | Normal | 117.90 ms | 1.74 ms | 116.41 ms | 119.26 ms | 213.2 MB | 878 B | 17.89 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.22 ms | 857.55 µs | 13.44 ms | 17.05 ms | 129.4 MB | 878 B | 18.70 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.31 ms | 61.46 µs | 13.17 ms | 16.86 ms | 224.2 MB | 878 B | 18.24 ms as a whole process |  |
| Seal | Ceil.source | 103.64 ms | 507.88 µs | 103.14 ms | 104.39 ms | 7.8 MB | 977 B | 122.00 µs cold, 166 ns warm; 2.73 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 104.47 ms | 402.18 µs | 103.92 ms | 105.83 ms | 7.8 MB | 977 B | 126.67 µs cold, 250 ns warm; 2.58 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 86.49 ms | 341.44 µs | 86.01 ms | 86.34 ms | 7.6 MB | 977 B | 104.67 µs cold, 208 ns warm; 2.60 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 88.04 ms | 298.32 µs | 87.63 ms | 87.80 ms | 7.6 MB | 977 B | 111.58 µs cold, 167 ns warm; 2.59 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 130.45 ms | 592.44 µs | 129.67 ms | 130.71 ms | 37.4 MB | 799 B | 4.94 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.23 ms | 1.10 ms | 126.04 ms | 126.46 ms | 42.7 MB | 799 B | 2.56 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 81.07 ms | 526.55 µs | 80.21 ms | 80.44 ms | 39.8 MB | 799 B | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 113.62 ms | 2.02 ms | 110.81 ms | 115.92 ms | 213.3 MB | 878 B | 18.66 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.28 ms | 878.43 µs | 13.49 ms | 17.23 ms | 129.8 MB | 878 B | 18.44 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.36 ms | 54.32 µs | 13.26 ms | 17.11 ms | 223.9 MB | 878 B | 18.32 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 51.52 µs (±5.72 µs, least 46.38 µs) | 15.20 µs (±2.75 µs, least 11.75 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.53 µs (±921 ns, least 8.46 µs) | 16.74 µs (±1.94 µs, least 14.79 µs) | 41 ns (±1 ns, least 40 ns) | 21 ns (±1 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.80 µs (±632 ns, least 3.00 µs) | 6.80 µs (±1.69 µs, least 5.00 µs) | 21 ns (±1 ns, least 20 ns) | 15 ns (±1 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 4.10 µs (±568 ns, least 3.00 µs) | 6.40 µs (±2.07 µs, least 4.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 30.26 µs (±8.45 µs, least 19.71 µs) | 38.47 µs (±1.83 µs, least 36.04 µs) | 57 ns (±1 ns, least 54 ns) | 30 ns (±0 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 30.55 µs (±7.84 µs, least 18.58 µs) | 37.14 µs (±1.66 µs, least 34.92 µs) | 8 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 4.78 µs (±33 ns, least 4.73 µs) | 144 ns (±5 ns, least 135 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.90 µs (±130 ns, least 4.68 µs) | 182 ns (±13 ns, least 167 ns) | 33 ns (±0 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.78 µs (±22 ns, least 1.75 µs) | 182 ns (±10 ns, least 168 ns) | 27 ns (±0 ns, least 27 ns) | 15 ns (±1 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.80 µs (±21 ns, least 1.76 µs) | 355 ns (±12 ns, least 328 ns) | 94 ns (±3 ns, least 90 ns) | 108 ns (±13 ns, least 98 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.16 µs (±15 ns, least 1.13 µs) | 658 ns (±18 ns, least 631 ns) | 50 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±24 ns, least 1.08 µs) | 660 ns (±16 ns, least 631 ns) | 22 ns (±1 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.8 MB | the engine's own heap |
| Seal | Ceil.source | 4.84 µs (±77 ns, least 4.73 µs) | 152 ns (±14 ns, least 139 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.88 µs (±117 ns, least 4.78 µs) | 151 ns (±6 ns, least 144 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 4.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.18 µs (±37 ns, least 4.14 µs) | 140 ns (±10 ns, least 125 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±1 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.19 µs (±41 ns, least 4.14 µs) | 143 ns (±4 ns, least 138 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±0 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.78 µs (±143 ns, least 4.60 µs) | 189 ns (±9 ns, least 174 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.79 µs (±21 ns, least 1.75 µs) | 182 ns (±8 ns, least 167 ns) | 27 ns (±0 ns, least 27 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.80 µs (±34 ns, least 1.74 µs) | 359 ns (±18 ns, least 329 ns) | 94 ns (±3 ns, least 91 ns) | 104 ns (±7 ns, least 99 ns) | 1.0 KB | 4.7 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.14 µs (±24 ns, least 1.10 µs) | 673 ns (±19 ns, least 642 ns) | 51 ns (±2 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±25 ns, least 1.06 µs) | 668 ns (±24 ns, least 625 ns) | 22 ns (±0 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 36.33 ms | 37.75 ms (+3.9%) |
| n-body, 100,000 steps | 65.96 ms | 61.51 ms (-6.7%) |
| spectral-norm, size 200 | 20.03 ms | 19.38 ms (-3.2%) |
| fannkuch-redux, 9 | 112.92 ms | 113.83 ms (+0.8%) |
| word count, 500,000 words | 27.01 ms | 26.74 ms (-1.0%) |
| text building, 20,000 joins | 341.30 µs | 355.45 µs (+4.1%) |
| binary-trees, depth 14 | 103.65 ms | 104.25 ms (+0.6%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 174.45 ms | 794.06 µs | 173.39 ms | 2.8 MB |
| the same sums in `Float`s, as a yardstick | 11.18 ms | 80.21 µs | 11.09 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.38 ms | 50.57 µs | 4.31 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.40 ms | 21.03 µs | 1.36 ms | 3.6 MB |
