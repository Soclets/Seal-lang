# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.42 1.91 1.76 }` before, `{ 1.51 1.77 1.73 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 32.33 ms | 71.29 ms | **met**: Seal takes 45% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.55 ms | 69.57 ms | **met**: Seal takes 94% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.62 ms | 28.19 ms | **met**: Seal takes 66% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 103.07 ms | 210.88 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.26 ms | 29.90 ms | **met**: Seal takes 88% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 342.04 µs | 159.72 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 97.90 ms | 130.97 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.19 ms | 3.46 ms | **met**: Seal takes 63% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 33 ns | **met**: Seal takes 62% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 149 ns | 180 ns | **met**: Seal takes 82% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.43 µs | 4.84 µs | **met**: Seal takes 92% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 32.33 ms | 33.87 ms | **met**: Seal takes 95% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 65.55 ms | 71.77 ms | **met**: Seal takes 91% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.62 ms | 46.21 ms | **met**: Seal takes 40% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 103.07 ms | 146.51 ms | **met**: Seal takes 70% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.26 ms | 30.82 ms | **met**: Seal takes 85% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 342.04 µs | 42.21 ms | **met**: Seal takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 97.90 ms | 128.61 ms | **met**: Seal takes 76% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.19 ms | 2.95 ms | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 28 ns | **met**: Seal takes 74% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 149 ns | 182 ns | **met**: Seal takes 82% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.43 µs | 1.79 µs | **not met**: Seal takes 247% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 32.33 ms | 94.96 ms | **met**: Seal takes 34% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.55 ms | 177.58 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.26 ms | 51.63 ms | **met**: Seal takes 51% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 342.04 µs | 497.56 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.36 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 83.40 ms | 389.56 µs | 83.08 ms | 102.29 ms | 2.8 MB | 386 B | 720.83 µs cold, 833 ns warm; 2.60 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 195.39 ms | 270.51 µs | 195.10 ms | 194.99 ms | 4.8 MB | 339 B | 5.14 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.15 ms | 960.65 µs | 88.82 ms | 88.55 ms | 1.9 MB | 339 B | 2.37 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.10 ms | 154.47 µs | 43.86 ms | 45.93 ms | 2.2 MB | 339 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Purpose | 321.20 ms | 257.31 µs | 320.60 ms | 319.76 ms | 43.3 MB | 362 B | 18.88 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.30 ms | 491.10 µs | 123.57 ms | 124.65 ms | 49.1 MB | 362 B | 18.30 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 122.74 ms | 365.17 µs | 122.30 ms | 124.90 ms | 49.1 MB | 362 B | 18.62 ms as a whole process |  |
| Seal | Normal | 32.33 ms | 685.81 µs | 31.36 ms | 30.68 ms | 2.7 MB | 160 B | 75.79 µs cold, 208 ns warm; 2.60 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 71.29 ms | 148.49 µs | 71.02 ms | 71.04 ms | 4.8 MB | 143 B | 4.99 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.87 ms | 96.81 µs | 33.78 ms | 33.88 ms | 1.9 MB | 143 B | 2.32 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 6.97 ms | 11.93 µs | 6.96 ms | 7.08 ms | 2.1 MB | 143 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 94.96 ms | 274.40 µs | 94.63 ms | 95.40 ms | 43.4 MB | 137 B | 18.93 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.53 ms | 386.36 µs | 9.39 ms | 10.22 ms | 48.6 MB | 137 B | 18.92 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.82 ms | 455.19 µs | 9.51 ms | 9.87 ms | 48.6 MB | 137 B | 19.18 ms as a whole process |  |
| Seal | Ceil.source | 33.19 ms | 567.29 µs | 32.57 ms | 33.26 ms | 2.7 MB | 160 B | 82.33 µs cold, 208 ns warm; 2.54 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 33.70 ms | 787.67 µs | 32.81 ms | 54.07 ms | 2.7 MB | 160 B | 890.33 µs cold, 458 ns warm; 2.72 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 34.54 ms | 1.10 ms | 32.73 ms | 52.93 ms | 2.5 MB | 160 B | 628.12 µs cold, 459 ns warm; 2.43 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 34.94 ms | 91.24 µs | 34.77 ms | 56.73 ms | 2.5 MB | 160 B | 678.12 µs cold, 417 ns warm; 2.62 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.21 ms | 209.05 µs | 70.89 ms | 72.33 ms | 4.8 MB | 143 B | 5.05 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.89 ms | 38.95 µs | 33.83 ms | 33.91 ms | 1.9 MB | 143 B | 2.36 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.41 ms | 30.01 µs | 7.35 ms | 7.53 ms | 2.1 MB | 143 B | 2.23 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 92.81 ms | 512.65 µs | 92.08 ms | 94.37 ms | 43.4 MB | 137 B | 18.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 10.01 ms | 785.84 µs | 9.49 ms | 10.53 ms | 48.5 MB | 137 B | 18.62 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.43 ms | 54.47 µs | 9.39 ms | 10.25 ms | 48.5 MB | 137 B | 19.18 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 49.90 ms | 469.51 µs | 49.14 ms | 53.41 ms | 2.7 MB | 547 B | 106.42 µs cold, 291 ns warm; 2.63 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.98 ms | 402.80 µs | 91.55 ms | 91.13 ms | 4.8 MB | 433 B | 4.95 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 76.82 ms | 366.99 µs | 76.17 ms | 76.16 ms | 1.9 MB | 433 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.54 ms | 85.05 µs | 6.41 ms | 6.78 ms | 2.1 MB | 433 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.06 ms | 668.29 µs | 242.20 ms | 245.39 ms | 46.4 MB | 460 B | 18.11 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.69 ms | 233.63 µs | 18.24 ms | 18.86 ms | 50.3 MB | 460 B | 18.34 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.58 ms | 230.45 µs | 18.17 ms | 18.69 ms | 50.3 MB | 460 B | 18.35 ms as a whole process |  |
| Seal | Normal | 65.55 ms | 681.63 µs | 64.36 ms | 65.40 ms | 3.2 MB | 3.5 KB | 268.04 µs cold, 250 ns warm; 2.75 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.57 ms | 173.40 µs | 69.32 ms | 70.03 ms | 4.8 MB | 3.0 KB | 5.16 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.77 ms | 268.09 µs | 71.32 ms | 71.83 ms | 1.9 MB | 3.0 KB | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.26 ms | 19.03 µs | 5.24 ms | 5.47 ms | 2.2 MB | 3.0 KB | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 177.58 ms | 3.84 ms | 173.39 ms | 181.78 ms | 48.1 MB | 3.1 KB | 19.01 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.89 ms | 15.58 µs | 3.86 ms | 5.66 ms | 51.9 MB | 3.1 KB | 18.26 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.88 ms | 26.36 µs | 3.84 ms | 5.68 ms | 51.8 MB | 3.1 KB | 18.46 ms as a whole process |  |
| Seal | Ceil.source | 54.31 ms | 548.11 µs | 53.56 ms | 55.02 ms | 3.2 MB | 3.4 KB | 254.67 µs cold, 334 ns warm; 2.72 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.72 ms | 634.78 µs | 54.79 ms | 55.23 ms | 3.2 MB | 3.4 KB | 260.38 µs cold, 333 ns warm; 2.93 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.54 ms | 455.79 µs | 48.81 ms | 49.97 ms | 3.0 MB | 3.4 KB | 266.92 µs cold, 291 ns warm; 2.91 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.27 ms | 519.90 µs | 48.47 ms | 49.87 ms | 3.0 MB | 3.4 KB | 262.04 µs cold, 291 ns warm; 2.76 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.62 ms | 120.20 µs | 69.39 ms | 69.95 ms | 4.8 MB | 3.0 KB | 5.15 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 71.91 ms | 251.93 µs | 71.59 ms | 71.21 ms | 1.9 MB | 3.0 KB | 2.59 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.20 ms | 28.39 µs | 5.17 ms | 5.40 ms | 2.2 MB | 3.0 KB | 2.44 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.10 ms | 3.72 ms | 174.00 ms | 180.41 ms | 48.2 MB | 3.1 KB | 19.12 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.87 ms | 19.70 µs | 3.84 ms | 5.71 ms | 51.9 MB | 3.1 KB | 19.33 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 24.38 µs | 3.84 ms | 5.68 ms | 52.0 MB | 3.1 KB | 19.24 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.17 ms | 946.38 µs | 55.48 ms | 58.47 ms | 2.9 MB | 426 B | 85.96 µs cold, 208 ns warm; 2.62 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.53 ms | 251.36 µs | 45.21 ms | 45.07 ms | 4.8 MB | 366 B | 4.85 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.10 ms | 115.93 µs | 76.95 ms | 76.98 ms | 1.9 MB | 366 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 22.64 ms | 280.22 µs | 22.19 ms | 22.38 ms | 2.1 MB | 366 B | 2.54 ms as a whole process |  |
| Node `--jitless` | Purpose | 363.50 ms | 3.41 ms | 360.37 ms | 368.70 ms | 48.6 MB | 398 B | 18.82 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.02 ms | 36.93 ms | 15.57 ms | 19.42 ms | 52.8 MB | 398 B | 18.58 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.48 ms | 264.13 µs | 14.30 ms | 20.23 ms | 55.4 MB | 398 B | 19.10 ms as a whole process |  |
| Seal | Normal | 18.62 ms | 220.08 µs | 18.37 ms | 19.21 ms | 3.0 MB | 1.2 KB | 150.75 µs cold, 208 ns warm; 2.52 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 28.19 ms | 815.24 µs | 27.27 ms | 28.55 ms | 4.8 MB | 981 B | 4.99 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 46.21 ms | 1.02 ms | 45.11 ms | 46.11 ms | 2.0 MB | 981 B | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.54 ms | 32.49 µs | 1.51 ms | 1.80 ms | 2.2 MB | 981 B | 2.37 ms as a whole process |  |
| Node `--jitless` | Normal | 65.25 ms | 225.56 µs | 65.00 ms | 64.96 ms | 48.4 MB | 1.0 KB | 18.79 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.61 ms | 190.62 µs | 1.49 ms | 3.32 ms | 52.7 MB | 1.0 KB | 18.94 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.62 ms | 20.53 µs | 1.58 ms | 3.66 ms | 52.8 MB | 1.0 KB | 19.15 ms as a whole process |  |
| Seal | Ceil.source | 18.60 ms | 290.45 µs | 18.20 ms | 19.04 ms | 3.0 MB | 1.2 KB | 146.25 µs cold, 291 ns warm; 2.68 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.78 ms | 107.13 µs | 18.59 ms | 18.85 ms | 3.1 MB | 1.2 KB | 147.54 µs cold, 250 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.59 ms | 32.09 µs | 18.50 ms | 18.54 ms | 2.8 MB | 1.2 KB | 119.67 µs cold, 250 ns warm; 2.64 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.92 ms | 48.00 µs | 18.85 ms | 18.87 ms | 2.8 MB | 1.2 KB | 121.00 µs cold, 208 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.29 ms | 481.82 µs | 26.72 ms | 27.01 ms | 4.9 MB | 981 B | 5.11 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 44.37 ms | 338.70 µs | 43.70 ms | 43.93 ms | 2.0 MB | 981 B | 2.44 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.55 ms | 52.82 µs | 1.50 ms | 1.77 ms | 2.2 MB | 981 B | 2.42 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.01 ms | 346.19 µs | 63.62 ms | 63.88 ms | 48.6 MB | 1.0 KB | 17.92 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.63 ms | 240.56 µs | 1.48 ms | 3.17 ms | 52.8 MB | 1.0 KB | 18.95 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.66 ms | 40.05 µs | 1.61 ms | 3.20 ms | 53.1 MB | 1.0 KB | 18.00 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 84.19 ms | 188.15 µs | 83.96 ms | 84.44 ms | 2.8 MB | 629 B | 94.50 µs cold, 250 ns warm; 2.56 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 89.08 ms | 1.38 ms | 86.76 ms | 89.33 ms | 4.8 MB | 523 B | 5.10 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.01 ms | 1.12 ms | 52.29 ms | 52.61 ms | 1.9 MB | 523 B | 2.41 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.25 ms | 27.68 µs | 6.19 ms | 7.03 ms | 2.1 MB | 523 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Purpose | 144.60 ms | 933.43 µs | 143.42 ms | 147.86 ms | 43.4 MB | 568 B | 18.63 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 45.43 µs | 3.62 ms | 4.46 ms | 49.2 MB | 568 B | 19.24 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.67 ms | 11.60 µs | 3.66 ms | 4.45 ms | 49.3 MB | 568 B | 18.83 ms as a whole process |  |
| Seal | Normal | 103.07 ms | 204.67 µs | 102.79 ms | 103.73 ms | 3.0 MB | 1.8 KB | 150.12 µs cold, 292 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 210.88 ms | 1.85 ms | 205.64 ms | 212.04 ms | 4.8 MB | 1.7 KB | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 146.51 ms | 853.22 µs | 145.86 ms | 144.25 ms | 1.9 MB | 1.7 KB | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 48.73 ms | 345.96 µs | 48.15 ms | 49.19 ms | 2.3 MB | 1.7 KB | 2.58 ms as a whole process |  |
| Node `--jitless` | Normal | 157.12 ms | 595.78 µs | 156.17 ms | 160.44 ms | 45.4 MB | 1.7 KB | 18.34 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.48 ms | 77.27 µs | 14.33 ms | 16.02 ms | 52.0 MB | 1.7 KB | 18.93 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.45 ms | 75.16 µs | 14.36 ms | 16.06 ms | 52.2 MB | 1.7 KB | 18.62 ms as a whole process |  |
| Seal | Ceil.source | 103.69 ms | 487.48 µs | 103.06 ms | 104.76 ms | 3.0 MB | 1.8 KB | 135.58 µs cold, 250 ns warm; 2.66 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 105.05 ms | 317.92 µs | 104.65 ms | 104.85 ms | 3.0 MB | 1.8 KB | 135.00 µs cold, 208 ns warm; 2.64 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 99.25 ms | 158.34 µs | 98.92 ms | 99.47 ms | 2.8 MB | 1.8 KB | 123.75 µs cold, 209 ns warm; 2.65 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.32 ms | 446.56 µs | 102.84 ms | 103.39 ms | 2.8 MB | 1.8 KB | 117.79 µs cold, 209 ns warm; 2.59 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 210.34 ms | 1.40 ms | 206.46 ms | 211.39 ms | 4.8 MB | 1.7 KB | 5.07 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 148.53 ms | 540.28 µs | 147.66 ms | 146.27 ms | 1.9 MB | 1.7 KB | 2.25 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 48.14 ms | 355.00 µs | 47.60 ms | 48.31 ms | 2.3 MB | 1.7 KB | 2.41 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 154.11 ms | 342.39 µs | 153.56 ms | 155.50 ms | 45.3 MB | 1.7 KB | 18.96 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.17 ms | 110.80 µs | 13.95 ms | 15.73 ms | 52.1 MB | 1.7 KB | 18.82 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.95 ms | 78.78 µs | 14.84 ms | 16.27 ms | 52.1 MB | 1.7 KB | 18.24 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 126.92 ms | 795.84 µs | 126.00 ms | 127.00 ms | 4.0 MB | 566 B | 105.17 µs cold, 375 ns warm; 2.53 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.70 ms | 412.30 µs | 62.03 ms | 62.97 ms | 4.8 MB | 540 B | 5.03 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 34.17 ms | 2.73 ms | 28.50 ms | 31.96 ms | 2.1 MB | 540 B | 2.47 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 5.11 ms | 19.04 µs | 5.08 ms | 5.21 ms | 2.3 MB | 540 B | 2.32 ms as a whole process |  |
| Node `--jitless` | Purpose | 240.63 ms | 1.75 ms | 238.78 ms | 245.50 ms | 48.7 MB | 514 B | 18.37 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 88.53 ms | 24.58 ms | 65.38 ms | 92.63 ms | 52.5 MB | 514 B | 18.36 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 65.66 ms | 385.25 µs | 65.22 ms | 89.85 ms | 52.7 MB | 514 B | 18.25 ms as a whole process |  |
| Seal | Normal | 26.26 ms | 114.81 µs | 26.09 ms | 26.34 ms | 3.9 MB | 535 B | 110.79 µs cold, 334 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.90 ms | 128.87 µs | 29.69 ms | 30.09 ms | 5.9 MB | 478 B | 5.04 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 30.82 ms | 159.40 µs | 30.43 ms | 29.15 ms | 3.0 MB | 478 B | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.53 ms | 541.30 µs | 11.71 ms | 12.09 ms | 3.3 MB | 478 B | 2.54 ms as a whole process |  |
| Node `--jitless` | Normal | 51.63 ms | 175.75 µs | 51.35 ms | 52.79 ms | 56.5 MB | 477 B | 18.41 ms as a whole process |  |
| Node (JIT), from cold | Normal | 34.79 ms | 172.07 µs | 34.61 ms | 35.36 ms | 62.3 MB | 477 B | 18.08 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.31 ms | 220.86 µs | 34.92 ms | 35.70 ms | 88.0 MB | 477 B | 18.41 ms as a whole process |  |
| Seal | Ceil.source | 26.27 ms | 118.87 µs | 26.09 ms | 26.51 ms | 4.0 MB | 535 B | 99.25 µs cold, 250 ns warm; 2.74 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 27.39 ms | 125.13 µs | 27.17 ms | 27.36 ms | 4.0 MB | 535 B | 97.67 µs cold, 292 ns warm; 2.47 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.79 ms | 71.30 µs | 23.65 ms | 23.75 ms | 3.9 MB | 535 B | 89.33 µs cold, 291 ns warm; 2.50 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.43 ms | 128.18 µs | 23.19 ms | 23.50 ms | 3.9 MB | 535 B | 87.79 µs cold, 250 ns warm; 2.45 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.91 ms | 123.07 µs | 29.64 ms | 30.10 ms | 6.0 MB | 478 B | 4.93 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 32.36 ms | 503.15 µs | 31.68 ms | 32.97 ms | 3.0 MB | 478 B | 2.43 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 11.99 ms | 770.15 µs | 10.78 ms | 12.17 ms | 3.3 MB | 478 B | 2.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.03 ms | 157.13 µs | 50.76 ms | 52.63 ms | 56.5 MB | 477 B | 18.27 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.25 ms | 174.77 µs | 35.11 ms | 35.89 ms | 62.5 MB | 477 B | 18.45 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.45 ms | 215.18 µs | 35.14 ms | 35.36 ms | 88.2 MB | 477 B | 18.95 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 308.50 µs | 3.37 µs | 305.79 µs | 322.25 µs | 3.1 MB | 230 B | 66.46 µs cold, 292 ns warm; 2.73 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.14 ms | 491.99 µs | 143.43 ms | 145.45 ms | 10.4 MB | 195 B | 4.96 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.93 ms | 101.91 µs | 32.78 ms | 31.40 ms | 3.5 MB | 195 B | 2.44 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.94 ms | 129.64 µs | 20.78 ms | 20.84 ms | 3.7 MB | 195 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Purpose | 360.60 µs | 135.16 µs | 249.50 µs | 1.02 ms | 52.4 MB | 223 B | 18.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 220.00 µs | 155.13 µs | 89.00 µs | 892.67 µs | 57.8 MB | 223 B | 18.52 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 136.10 µs | 121.30 µs | 55.92 µs | 881.42 µs | 57.9 MB | 223 B | 17.89 ms as a whole process |  |
| Seal | Normal | 342.04 µs | 3.23 µs | 338.12 µs | 366.79 µs | 3.2 MB | 179 B | 73.62 µs cold, 375 ns warm; 2.61 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 159.72 ms | 994.38 µs | 158.81 ms | 163.27 ms | 11.8 MB | 138 B | 4.88 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 42.21 ms | 199.58 µs | 41.83 ms | 41.71 ms | 4.9 MB | 138 B | 2.40 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.61 ms | 218.35 µs | 19.14 ms | 19.48 ms | 5.2 MB | 138 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Normal | 497.56 µs | 335.57 µs | 261.17 µs | 1.11 ms | 61.4 MB | 155 B | 18.17 ms as a whole process |  |
| Node (JIT), from cold | Normal | 489.05 µs | 466.95 µs | 129.17 µs | 1.05 ms | 65.2 MB | 155 B | 18.41 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 277.61 µs | 326.32 µs | 124.58 µs | 1.05 ms | 69.2 MB | 155 B | 18.50 ms as a whole process |  |
| Seal | Ceil.source | 343.60 µs | 5.69 µs | 338.33 µs | 350.46 µs | 3.2 MB | 179 B | 75.21 µs cold, 291 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 352.02 µs | 5.02 µs | 345.75 µs | 398.67 µs | 3.3 MB | 179 B | 75.62 µs cold, 416 ns warm; 2.53 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 286.96 µs | 5.22 µs | 282.00 µs | 322.12 µs | 3.0 MB | 179 B | 57.67 µs cold, 292 ns warm; 2.42 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 292.20 µs | 3.33 µs | 287.67 µs | 311.46 µs | 3.0 MB | 179 B | 59.29 µs cold, 333 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 159.46 ms | 1.47 ms | 158.17 ms | 163.06 ms | 11.3 MB | 138 B | 4.93 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 42.33 ms | 308.29 µs | 41.55 ms | 41.18 ms | 5.0 MB | 138 B | 2.55 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.46 ms | 180.77 µs | 19.24 ms | 19.96 ms | 5.0 MB | 138 B | 2.47 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 504.32 µs | 344.52 µs | 261.62 µs | 1.13 ms | 61.5 MB | 155 B | 18.42 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 496.82 µs | 488.82 µs | 121.38 µs | 1.08 ms | 65.2 MB | 155 B | 18.57 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 280.56 µs | 327.10 µs | 125.54 µs | 1.07 ms | 69.4 MB | 155 B | 18.29 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 72.41 ms | 496.12 µs | 71.62 ms | 76.62 ms | 3.0 MB | 528 B | 95.92 µs cold, 209 ns warm; 2.63 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 109.12 ms | 2.95 ms | 103.96 ms | 95.72 ms | 4.8 MB | 411 B | 4.94 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 118.65 ms | 656.36 µs | 117.28 ms | 117.69 ms | 2.5 MB | 411 B | 2.45 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 95.88 ms | 824.78 µs | 94.67 ms | 94.31 ms | 2.9 MB | 411 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.46 ms | 190.32 µs | 76.18 ms | 80.25 ms | 59.6 MB | 444 B | 19.18 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.45 ms | 295.19 µs | 10.10 ms | 13.01 ms | 65.7 MB | 444 B | 18.43 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.42 ms | 594.37 µs | 10.07 ms | 12.98 ms | 114.6 MB | 444 B | 18.78 ms as a whole process |  |
| Seal | Normal | 97.90 ms | 488.69 µs | 97.17 ms | 98.39 ms | 7.8 MB | 977 B | 126.58 µs cold, 209 ns warm; 2.47 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 130.97 ms | 1.39 ms | 128.79 ms | 130.53 ms | 42.0 MB | 799 B | 5.09 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 128.61 ms | 1.31 ms | 126.68 ms | 127.23 ms | 39.0 MB | 799 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.27 ms | 1.27 ms | 79.36 ms | 80.93 ms | 44.1 MB | 799 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Normal | 115.01 ms | 1.56 ms | 114.03 ms | 120.03 ms | 213.2 MB | 878 B | 18.40 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.15 ms | 936.37 µs | 13.29 ms | 17.15 ms | 129.5 MB | 878 B | 19.49 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.38 ms | 45.19 µs | 13.31 ms | 17.13 ms | 223.9 MB | 878 B | 19.02 ms as a whole process |  |
| Seal | Ceil.source | 99.99 ms | 1.34 ms | 98.17 ms | 103.68 ms | 7.8 MB | 977 B | 125.83 µs cold, 334 ns warm; 2.62 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 100.54 ms | 620.46 µs | 99.69 ms | 101.88 ms | 7.8 MB | 977 B | 118.62 µs cold, 250 ns warm; 2.49 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 83.90 ms | 445.27 µs | 83.25 ms | 84.52 ms | 7.7 MB | 977 B | 105.17 µs cold, 208 ns warm; 2.70 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 90.45 ms | 339.14 µs | 89.94 ms | 91.00 ms | 7.6 MB | 977 B | 103.25 µs cold, 125 ns warm; 2.54 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 132.04 ms | 950.81 µs | 130.65 ms | 131.47 ms | 43.4 MB | 799 B | 5.05 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 127.24 ms | 1.12 ms | 125.18 ms | 126.39 ms | 41.5 MB | 799 B | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 80.91 ms | 860.47 µs | 80.01 ms | 80.27 ms | 42.6 MB | 799 B | 2.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 113.37 ms | 1.67 ms | 111.67 ms | 114.49 ms | 213.2 MB | 878 B | 18.85 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.20 ms | 897.29 µs | 13.29 ms | 17.05 ms | 129.3 MB | 878 B | 18.63 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.45 ms | 74.66 µs | 13.34 ms | 17.04 ms | 224.0 MB | 878 B | 19.09 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 48.29 µs (±6.51 µs, least 42.50 µs) | 14.65 µs (±2.11 µs, least 12.38 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.37 µs (±3.00 µs, least 8.21 µs) | 16.69 µs (±3.30 µs, least 14.67 µs) | 42 ns (±1 ns, least 40 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.80 µs (±632 ns, least 3.00 µs) | 6.60 µs (±966 ns, least 5.00 µs) | 22 ns (±1 ns, least 21 ns) | 14 ns (±1 ns, least 12 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.70 µs (±823 ns, least 3.00 µs) | 6.90 µs (±2.13 µs, least 4.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 37.82 µs (±10.41 µs, least 21.08 µs) | 38.22 µs (±1.38 µs, least 36.21 µs) | 56 ns (±2 ns, least 53 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 33.62 µs (±5.23 µs, least 18.96 µs) | 38.05 µs (±2.32 µs, least 35.25 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 4.43 µs (±44 ns, least 4.36 µs) | 149 ns (±5 ns, least 141 ns) | 20 ns (±1 ns, least 20 ns) | 40 ns (±0 ns, least 39 ns) | 512 B | 4.4 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.84 µs (±129 ns, least 4.70 µs) | 180 ns (±7 ns, least 168 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.79 µs (±24 ns, least 1.75 µs) | 182 ns (±10 ns, least 173 ns) | 28 ns (±0 ns, least 27 ns) | 15 ns (±1 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.81 µs (±33 ns, least 1.76 µs) | 369 ns (±35 ns, least 338 ns) | 93 ns (±2 ns, least 91 ns) | 108 ns (±14 ns, least 98 ns) | 1.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±16 ns, least 1.13 µs) | 665 ns (±16 ns, least 652 ns) | 50 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±27 ns, least 1.07 µs) | 657 ns (±12 ns, least 632 ns) | 22 ns (±0 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.8 MB | the engine's own heap |
| Seal | Ceil.source | 4.44 µs (±62 ns, least 4.37 µs) | 150 ns (±4 ns, least 144 ns) | 20 ns (±0 ns, least 20 ns) | 39 ns (±1 ns, least 39 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.57 µs (±51 ns, least 4.50 µs) | 149 ns (±8 ns, least 139 ns) | 21 ns (±0 ns, least 20 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 3.91 µs (±18 ns, least 3.89 µs) | 140 ns (±8 ns, least 130 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 3.96 µs (±50 ns, least 3.91 µs) | 139 ns (±5 ns, least 132 ns) | 18 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.02 µs (±169 ns, least 4.69 µs) | 185 ns (±6 ns, least 175 ns) | 33 ns (±0 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.80 µs (±19 ns, least 1.77 µs) | 185 ns (±13 ns, least 177 ns) | 28 ns (±0 ns, least 27 ns) | 15 ns (±2 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.82 µs (±20 ns, least 1.79 µs) | 356 ns (±11 ns, least 345 ns) | 95 ns (±3 ns, least 92 ns) | 107 ns (±8 ns, least 100 ns) | 1.0 KB | 4.7 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.14 µs (±23 ns, least 1.12 µs) | 671 ns (±22 ns, least 645 ns) | 51 ns (±2 ns, least 49 ns) | 24 ns (±2 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±21 ns, least 1.07 µs) | 664 ns (±16 ns, least 642 ns) | 22 ns (±1 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 32.58 ms | 32.28 ms (-0.9%) |
| n-body, 100,000 steps | 65.52 ms | 58.39 ms (-10.9%) |
| spectral-norm, size 200 | 18.40 ms | 19.24 ms (+4.6%) |
| fannkuch-redux, 9 | 104.95 ms | 104.63 ms (-0.3%) |
| word count, 500,000 words | 27.04 ms | 33.45 ms (+23.7%) |
| text building, 20,000 joins | 348.26 µs | 344.98 µs (-0.9%) |
| binary-trees, depth 14 | 99.26 ms | 99.76 ms (+0.5%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 177.60 ms | 5.76 ms | 171.01 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.29 ms | 170.26 µs | 9.95 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.40 ms | 47.54 µs | 4.33 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.42 ms | 11.28 µs | 1.40 ms | 3.6 MB |
