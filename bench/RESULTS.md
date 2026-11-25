# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 1.99 1.90 1.80 }` before, `{ 2.21 2.03 1.87 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 35.50 ms | 72.96 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.77 ms | 70.02 ms | **met**: Seal takes 94% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.95 ms | 28.48 ms | **met**: Seal takes 67% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 104.38 ms | 212.56 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.29 ms | 30.15 ms | **met**: Seal takes 87% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 581.31 µs | 160.27 ms | **met**: Seal takes 0.4% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 100.49 ms | 142.05 ms | **met**: Seal takes 71% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.17 ms | 3.55 ms | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 34 ns | **met**: Seal takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 153 ns | 187 ns | **met**: Seal takes 82% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.63 µs | 5.00 µs | **met**: Seal takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 35.50 ms | 33.82 ms | **not met**: Seal takes 105% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 65.77 ms | 72.02 ms | **met**: Seal takes 91% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.95 ms | 43.68 ms | **met**: Seal takes 43% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 104.38 ms | 147.76 ms | **met**: Seal takes 71% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 26.29 ms | 32.54 ms | **met**: Seal takes 81% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 581.31 µs | 41.18 ms | **met**: Seal takes 1.4% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 100.49 ms | 131.73 ms | **met**: Seal takes 76% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.17 ms | 3.00 ms | **met**: Seal takes 72% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 20 ns | 28 ns | **met**: Seal takes 72% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 153 ns | 184 ns | **met**: Seal takes 83% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.63 µs | 1.81 µs | for reference, no target (D150) |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 35.50 ms | 96.16 ms | **met**: Seal takes 37% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.77 ms | 180.74 ms | **met**: Seal takes 36% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.29 ms | 51.49 ms | **met**: Seal takes 51% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 581.31 µs | 509.76 µs | **met**: Seal takes 114% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 40 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 5.43 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 83.98 ms | 312.20 µs | 83.61 ms | 101.83 ms | 2.8 MB | 386 B | 696.00 µs cold, 916 ns warm; 2.49 ms as a whole process | 32.0 KB counted by Seal; 176 B compiled |
| Luau (interpreter) | Purpose | 197.60 ms | 250.08 µs | 197.30 ms | 197.33 ms | 4.8 MB | 339 B | 5.17 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 90.87 ms | 622.19 µs | 89.48 ms | 89.79 ms | 1.9 MB | 339 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 43.94 ms | 195.36 µs | 43.74 ms | 45.64 ms | 2.2 MB | 339 B | 2.46 ms as a whole process |  |
| Node `--jitless` | Purpose | 320.50 ms | 457.41 µs | 320.04 ms | 320.10 ms | 43.4 MB | 362 B | 19.16 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.72 ms | 437.53 µs | 124.25 ms | 124.91 ms | 49.3 MB | 362 B | 18.21 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 123.79 ms | 905.58 µs | 122.68 ms | 124.35 ms | 49.2 MB | 362 B | 19.34 ms as a whole process |  |
| Seal | Normal | 35.50 ms | 735.31 µs | 34.49 ms | 30.55 ms | 2.7 MB | 160 B | 77.83 µs cold, 166 ns warm; 2.70 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled |
| Luau (interpreter) | Normal | 72.96 ms | 440.14 µs | 72.41 ms | 73.22 ms | 4.8 MB | 143 B | 5.07 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 33.82 ms | 86.83 µs | 33.75 ms | 33.85 ms | 1.9 MB | 143 B | 2.34 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 7.64 ms | 28.34 µs | 7.59 ms | 8.03 ms | 2.2 MB | 143 B | 2.38 ms as a whole process |  |
| Node `--jitless` | Normal | 96.16 ms | 1.08 ms | 93.76 ms | 95.25 ms | 43.3 MB | 137 B | 20.15 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.61 ms | 187.15 µs | 9.42 ms | 9.85 ms | 48.6 MB | 137 B | 18.69 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.48 ms | 33.86 µs | 9.44 ms | 10.55 ms | 48.5 MB | 137 B | 18.95 ms as a whole process |  |
| Seal | Ceil.source | 35.94 ms | 750.81 µs | 34.08 ms | 37.61 ms | 2.7 MB | 160 B | 73.21 µs cold, 167 ns warm; 2.65 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 33.01 ms | 225.20 µs | 32.65 ms | 55.03 ms | 2.7 MB | 160 B | 849.88 µs cold, 542 ns warm; 2.80 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.program | 32.91 ms | 799.14 µs | 31.94 ms | 34.48 ms | 2.5 MB | 160 B | 299.75 µs cold, 208 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 32.95 ms | 249.46 µs | 32.42 ms | 55.15 ms | 2.5 MB | 160 B | 589.17 µs cold, 333 ns warm; 2.72 ms as a whole process | 2.0 KB counted by Seal; 104 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 74.55 ms | 352.88 µs | 74.19 ms | 74.17 ms | 4.8 MB | 143 B | 5.61 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 38.99 ms | 10.92 ms | 33.76 ms | 34.55 ms | 1.9 MB | 143 B | 2.46 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.39 ms | 38.74 µs | 7.32 ms | 7.55 ms | 2.1 MB | 143 B | 2.49 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 97.38 ms | 1.17 ms | 95.39 ms | 96.01 ms | 43.4 MB | 137 B | 19.62 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 10.10 ms | 215.29 µs | 9.75 ms | 10.02 ms | 48.7 MB | 137 B | 19.19 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.61 ms | 57.00 µs | 9.54 ms | 10.21 ms | 48.5 MB | 137 B | 20.01 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.85 ms | 753.22 µs | 53.67 ms | 55.44 ms | 2.8 MB | 547 B | 108.71 µs cold, 166 ns warm; 2.77 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.84 ms | 509.63 µs | 92.06 ms | 92.46 ms | 4.8 MB | 433 B | 5.07 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.79 ms | 390.61 µs | 76.99 ms | 77.55 ms | 1.9 MB | 433 B | 2.39 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.76 ms | 44.05 µs | 6.70 ms | 6.79 ms | 2.1 MB | 433 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.40 ms | 664.64 µs | 244.43 ms | 246.13 ms | 46.3 MB | 460 B | 19.54 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.65 ms | 172.72 µs | 18.44 ms | 18.54 ms | 50.3 MB | 460 B | 19.37 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.82 ms | 292.86 µs | 18.32 ms | 18.85 ms | 50.3 MB | 460 B | 19.44 ms as a whole process |  |
| Seal | Normal | 65.77 ms | 518.00 µs | 64.88 ms | 66.12 ms | 3.3 MB | 3.5 KB | 290.08 µs cold, 250 ns warm; 2.93 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.02 ms | 150.46 µs | 69.83 ms | 69.96 ms | 4.8 MB | 3.0 KB | 5.26 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 72.02 ms | 373.28 µs | 71.55 ms | 72.06 ms | 1.9 MB | 3.0 KB | 2.55 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.26 ms | 20.41 µs | 5.23 ms | 5.45 ms | 2.2 MB | 3.0 KB | 2.61 ms as a whole process |  |
| Node `--jitless` | Normal | 180.74 ms | 3.89 ms | 176.48 ms | 182.21 ms | 48.1 MB | 3.1 KB | 19.59 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.90 ms | 32.68 µs | 3.86 ms | 5.71 ms | 52.0 MB | 3.1 KB | 18.97 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.91 ms | 25.03 µs | 3.88 ms | 5.67 ms | 52.0 MB | 3.1 KB | 18.91 ms as a whole process |  |
| Seal | Ceil.source | 54.25 ms | 414.59 µs | 53.43 ms | 55.04 ms | 3.2 MB | 3.4 KB | 256.21 µs cold, 333 ns warm; 2.87 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.47 ms | 1.16 ms | 54.31 ms | 58.83 ms | 3.2 MB | 3.4 KB | 269.33 µs cold, 292 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.49 ms | 160.12 µs | 50.25 ms | 50.30 ms | 2.9 MB | 3.4 KB | 254.83 µs cold, 250 ns warm; 2.92 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.58 ms | 306.32 µs | 49.15 ms | 50.01 ms | 3.0 MB | 3.4 KB | 256.38 µs cold, 250 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.89 ms | 177.17 µs | 69.64 ms | 69.87 ms | 4.8 MB | 3.0 KB | 5.30 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 72.04 ms | 178.60 µs | 71.74 ms | 71.99 ms | 1.9 MB | 3.0 KB | 2.59 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.26 ms | 75.82 µs | 5.20 ms | 5.40 ms | 2.2 MB | 3.0 KB | 2.61 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.36 ms | 3.63 ms | 172.37 ms | 181.48 ms | 48.1 MB | 3.1 KB | 19.57 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.90 ms | 28.11 µs | 3.87 ms | 5.72 ms | 52.0 MB | 3.1 KB | 19.86 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.90 ms | 29.40 µs | 3.87 ms | 5.69 ms | 51.8 MB | 3.1 KB | 18.57 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.08 ms | 1.11 ms | 50.36 ms | 52.62 ms | 2.9 MB | 426 B | 97.79 µs cold, 208 ns warm; 2.83 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 46.70 ms | 435.33 µs | 45.94 ms | 46.05 ms | 4.8 MB | 366 B | 5.23 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.97 ms | 311.61 µs | 77.58 ms | 78.01 ms | 1.9 MB | 366 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 23.00 ms | 153.67 µs | 22.76 ms | 23.12 ms | 2.2 MB | 366 B | 2.47 ms as a whole process |  |
| Node `--jitless` | Purpose | 374.08 ms | 7.24 ms | 366.08 ms | 385.99 ms | 48.7 MB | 398 B | 18.93 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.59 ms | 36.91 ms | 15.77 ms | 21.28 ms | 52.8 MB | 398 B | 19.77 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.32 ms | 305.99 µs | 14.82 ms | 19.48 ms | 55.3 MB | 398 B | 18.92 ms as a whole process |  |
| Seal | Normal | 18.95 ms | 289.26 µs | 18.30 ms | 19.89 ms | 3.1 MB | 1.2 KB | 148.50 µs cold, 208 ns warm; 3.05 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 28.48 ms | 503.27 µs | 27.96 ms | 27.94 ms | 4.8 MB | 981 B | 5.82 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 43.68 ms | 652.17 µs | 42.65 ms | 47.20 ms | 2.0 MB | 981 B | 2.69 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.56 ms | 33.20 µs | 1.53 ms | 1.71 ms | 2.2 MB | 981 B | 2.38 ms as a whole process |  |
| Node `--jitless` | Normal | 65.47 ms | 689.17 µs | 64.50 ms | 64.58 ms | 48.4 MB | 1.0 KB | 19.34 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.67 ms | 239.06 µs | 1.48 ms | 3.41 ms | 52.9 MB | 1.0 KB | 19.45 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.68 ms | 40.21 µs | 1.62 ms | 3.34 ms | 52.9 MB | 1.0 KB | 20.44 ms as a whole process |  |
| Seal | Ceil.source | 19.39 ms | 126.97 µs | 19.17 ms | 20.04 ms | 3.1 MB | 1.2 KB | 145.88 µs cold, 333 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 18.92 ms | 111.73 µs | 18.79 ms | 19.10 ms | 3.0 MB | 1.2 KB | 144.29 µs cold, 208 ns warm; 2.87 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.85 ms | 111.83 µs | 18.57 ms | 18.79 ms | 2.8 MB | 1.2 KB | 126.21 µs cold, 208 ns warm; 2.92 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 19.05 ms | 71.84 µs | 18.93 ms | 19.23 ms | 2.9 MB | 1.2 KB | 137.62 µs cold, 209 ns warm; 2.91 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.93 ms | 319.56 µs | 27.26 ms | 28.86 ms | 4.8 MB | 981 B | 5.47 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 46.48 ms | 744.75 µs | 45.36 ms | 47.45 ms | 2.0 MB | 981 B | 2.50 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.54 ms | 17.72 µs | 1.52 ms | 1.70 ms | 2.2 MB | 981 B | 2.63 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 65.76 ms | 653.08 µs | 64.81 ms | 64.60 ms | 48.4 MB | 1.0 KB | 19.69 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.63 ms | 161.05 µs | 1.49 ms | 3.20 ms | 52.8 MB | 1.0 KB | 19.81 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.67 ms | 33.98 µs | 1.61 ms | 3.37 ms | 52.9 MB | 1.0 KB | 20.38 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 87.61 ms | 824.35 µs | 85.88 ms | 86.07 ms | 2.8 MB | 629 B | 102.67 µs cold, 209 ns warm; 2.75 ms as a whole process | 688 B counted by Seal; 288 B compiled |
| Luau (interpreter) | Purpose | 89.68 ms | 1.40 ms | 87.24 ms | 89.63 ms | 4.8 MB | 523 B | 5.69 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 53.70 ms | 1.26 ms | 52.21 ms | 53.21 ms | 1.9 MB | 523 B | 2.60 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.30 ms | 64.38 µs | 6.21 ms | 6.25 ms | 2.1 MB | 523 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Purpose | 149.89 ms | 867.48 µs | 148.62 ms | 151.31 ms | 43.4 MB | 568 B | 19.51 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.69 ms | 34.19 µs | 3.64 ms | 4.65 ms | 49.4 MB | 568 B | 19.61 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.70 ms | 31.85 µs | 3.66 ms | 4.48 ms | 49.3 MB | 568 B | 19.56 ms as a whole process |  |
| Seal | Normal | 104.38 ms | 381.60 µs | 103.63 ms | 105.43 ms | 3.0 MB | 1.8 KB | 140.92 µs cold, 208 ns warm; 2.74 ms as a whole process | 984 B counted by Seal; 688 B compiled |
| Luau (interpreter) | Normal | 212.56 ms | 2.09 ms | 206.69 ms | 213.37 ms | 4.8 MB | 1.7 KB | 5.29 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.76 ms | 283.18 µs | 147.31 ms | 146.18 ms | 1.9 MB | 1.7 KB | 2.66 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 49.97 ms | 349.89 µs | 49.49 ms | 50.11 ms | 2.3 MB | 1.7 KB | 2.69 ms as a whole process |  |
| Node `--jitless` | Normal | 156.67 ms | 1.04 ms | 154.54 ms | 158.61 ms | 45.5 MB | 1.7 KB | 20.17 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.74 ms | 127.85 µs | 14.49 ms | 16.03 ms | 52.1 MB | 1.7 KB | 19.86 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.89 ms | 87.68 µs | 14.78 ms | 15.99 ms | 52.1 MB | 1.7 KB | 19.62 ms as a whole process |  |
| Seal | Ceil.source | 106.09 ms | 200.99 µs | 105.68 ms | 105.13 ms | 3.0 MB | 1.8 KB | 152.38 µs cold, 209 ns warm; 2.76 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 107.45 ms | 377.86 µs | 106.76 ms | 107.31 ms | 3.0 MB | 1.8 KB | 142.33 µs cold, 250 ns warm; 2.91 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.program | 100.23 ms | 233.34 µs | 99.85 ms | 100.11 ms | 2.8 MB | 1.8 KB | 143.58 µs cold, 250 ns warm; 2.61 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 109.73 ms | 15.50 ms | 103.56 ms | 102.97 ms | 2.8 MB | 1.8 KB | 130.33 µs cold, 208 ns warm; 3.16 ms as a whole process | 984 B counted by Seal; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 218.81 ms | 14.48 ms | 208.67 ms | 268.17 ms | 4.9 MB | 1.7 KB | 5.24 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 148.55 ms | 594.87 µs | 147.62 ms | 146.41 ms | 1.9 MB | 1.7 KB | 2.52 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 49.39 ms | 419.73 µs | 48.52 ms | 48.93 ms | 2.3 MB | 1.7 KB | 2.56 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 160.62 ms | 1.15 ms | 158.73 ms | 159.59 ms | 45.3 MB | 1.7 KB | 19.14 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.84 ms | 157.43 µs | 14.65 ms | 16.30 ms | 52.1 MB | 1.7 KB | 19.05 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.86 ms | 98.46 µs | 14.72 ms | 16.37 ms | 52.0 MB | 1.7 KB | 18.55 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 127.01 ms | 776.29 µs | 126.22 ms | 127.82 ms | 4.0 MB | 566 B | 95.04 µs cold, 292 ns warm; 2.71 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 65.01 ms | 1.97 ms | 63.07 ms | 64.24 ms | 4.8 MB | 540 B | 5.26 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 33.76 ms | 1.22 ms | 31.65 ms | 33.58 ms | 2.1 MB | 540 B | 2.46 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 5.17 ms | 35.19 µs | 5.09 ms | 4.78 ms | 2.3 MB | 540 B | 2.56 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.48 ms | 1.95 ms | 241.73 ms | 248.69 ms | 48.7 MB | 514 B | 19.22 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 91.82 ms | 25.16 ms | 67.59 ms | 92.70 ms | 52.5 MB | 514 B | 18.99 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.84 ms | 223.20 µs | 67.53 ms | 90.99 ms | 52.5 MB | 514 B | 18.76 ms as a whole process |  |
| Seal | Normal | 26.29 ms | 152.69 µs | 26.05 ms | 26.31 ms | 4.0 MB | 535 B | 108.75 µs cold, 292 ns warm; 2.83 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.15 ms | 324.88 µs | 29.70 ms | 30.42 ms | 5.9 MB | 478 B | 5.18 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 32.54 ms | 553.96 µs | 31.77 ms | 31.60 ms | 3.0 MB | 478 B | 2.65 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.47 ms | 715.26 µs | 11.50 ms | 11.81 ms | 3.3 MB | 478 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 51.49 ms | 425.91 µs | 51.04 ms | 52.36 ms | 56.5 MB | 477 B | 18.49 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.43 ms | 298.35 µs | 34.93 ms | 36.34 ms | 62.4 MB | 477 B | 18.71 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.40 ms | 368.61 µs | 34.94 ms | 36.26 ms | 88.1 MB | 477 B | 19.26 ms as a whole process |  |
| Seal | Ceil.source | 26.50 ms | 168.56 µs | 26.21 ms | 26.68 ms | 4.0 MB | 535 B | 100.79 µs cold, 333 ns warm; 2.81 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 28.34 ms | 401.57 µs | 27.75 ms | 27.75 ms | 4.0 MB | 535 B | 127.08 µs cold, 292 ns warm; 2.81 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.73 ms | 151.27 µs | 23.46 ms | 24.14 ms | 3.9 MB | 535 B | 86.71 µs cold, 333 ns warm; 2.64 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.11 ms | 109.73 µs | 23.89 ms | 24.08 ms | 3.9 MB | 535 B | 95.08 µs cold, 250 ns warm; 2.77 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.71 ms | 255.14 µs | 30.32 ms | 30.98 ms | 6.0 MB | 478 B | 5.35 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 30.81 ms | 589.16 µs | 29.81 ms | 28.55 ms | 3.0 MB | 478 B | 2.45 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.74 ms | 1.00 ms | 11.54 ms | 15.47 ms | 3.3 MB | 478 B | 2.50 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.78 ms | 268.86 µs | 51.38 ms | 52.42 ms | 56.4 MB | 477 B | 18.89 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.36 ms | 354.91 µs | 34.94 ms | 36.25 ms | 62.5 MB | 477 B | 18.82 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.36 ms | 322.19 µs | 34.90 ms | 35.85 ms | 88.0 MB | 477 B | 18.55 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 318.97 µs | 7.28 µs | 309.71 µs | 333.75 µs | 3.1 MB | 230 B | 73.00 µs cold, 292 ns warm; 2.61 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 143.65 ms | 636.59 µs | 142.67 ms | 145.18 ms | 10.3 MB | 195 B | 4.95 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.36 ms | 101.79 µs | 32.23 ms | 31.45 ms | 3.6 MB | 195 B | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.70 ms | 82.57 µs | 20.63 ms | 21.09 ms | 3.7 MB | 195 B | 2.50 ms as a whole process |  |
| Node `--jitless` | Purpose | 361.30 µs | 141.77 µs | 240.17 µs | 1.04 ms | 52.5 MB | 223 B | 18.82 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 222.39 µs | 153.72 µs | 93.00 µs | 907.42 µs | 57.8 MB | 223 B | 19.64 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 176.75 µs | 165.59 µs | 62.83 µs | 879.42 µs | 58.4 MB | 223 B | 19.05 ms as a whole process |  |
| Seal | Normal | 581.31 µs | 10.73 µs | 568.67 µs | 610.38 µs | 3.2 MB | 179 B | 75.50 µs cold, 375 ns warm; 2.60 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.27 ms | 244.99 µs | 159.82 ms | 162.42 ms | 11.3 MB | 138 B | 5.08 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.18 ms | 180.62 µs | 40.86 ms | 41.67 ms | 5.0 MB | 138 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.51 ms | 308.65 µs | 18.80 ms | 20.24 ms | 5.2 MB | 138 B | 2.49 ms as a whole process |  |
| Node `--jitless` | Normal | 509.76 µs | 336.53 µs | 280.25 µs | 1.11 ms | 61.5 MB | 155 B | 19.09 ms as a whole process |  |
| Node (JIT), from cold | Normal | 504.25 µs | 493.47 µs | 124.33 µs | 1.07 ms | 65.3 MB | 155 B | 19.57 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 277.24 µs | 330.49 µs | 124.42 µs | 1.03 ms | 69.4 MB | 155 B | 18.87 ms as a whole process |  |
| Seal | Ceil.source | 357.71 µs | 4.07 µs | 353.38 µs | 378.21 µs | 3.2 MB | 179 B | 78.79 µs cold, 334 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 341.38 µs | 5.34 µs | 338.12 µs | 361.79 µs | 3.2 MB | 179 B | 73.29 µs cold, 292 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 294.01 µs | 4.29 µs | 287.75 µs | 321.83 µs | 3.0 MB | 179 B | 57.92 µs cold, 250 ns warm; 2.57 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 287.90 µs | 3.28 µs | 283.88 µs | 316.79 µs | 3.0 MB | 179 B | 60.46 µs cold, 292 ns warm; 2.33 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.28 ms | 284.57 µs | 160.90 ms | 163.60 ms | 12.3 MB | 138 B | 5.12 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 41.68 ms | 228.77 µs | 41.50 ms | 41.63 ms | 4.8 MB | 138 B | 2.46 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.53 ms | 131.84 µs | 19.35 ms | 20.65 ms | 5.1 MB | 138 B | 2.41 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 503.98 µs | 344.90 µs | 260.33 µs | 1.11 ms | 61.3 MB | 155 B | 18.60 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 491.73 µs | 477.21 µs | 120.71 µs | 1.09 ms | 65.3 MB | 155 B | 18.70 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 296.91 µs | 341.22 µs | 136.04 µs | 1.12 ms | 69.2 MB | 155 B | 18.65 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 72.66 ms | 670.58 µs | 71.98 ms | 73.31 ms | 3.0 MB | 528 B | 90.62 µs cold, 166 ns warm; 2.51 ms as a whole process | 65.9 KB counted by Seal; 216 B compiled |
| Luau (interpreter) | Purpose | 95.76 ms | 376.79 µs | 95.35 ms | 96.29 ms | 4.8 MB | 411 B | 5.27 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 120.55 ms | 3.68 ms | 115.74 ms | 118.09 ms | 2.5 MB | 411 B | 2.41 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.59 ms | 2.18 ms | 90.69 ms | 91.59 ms | 2.9 MB | 411 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.66 ms | 329.70 µs | 75.11 ms | 79.41 ms | 59.7 MB | 444 B | 19.11 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.58 ms | 306.86 µs | 10.27 ms | 12.99 ms | 65.7 MB | 444 B | 19.64 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.60 ms | 742.03 µs | 10.13 ms | 13.25 ms | 114.7 MB | 444 B | 19.02 ms as a whole process |  |
| Seal | Normal | 100.49 ms | 458.03 µs | 99.81 ms | 100.86 ms | 7.9 MB | 977 B | 126.67 µs cold, 209 ns warm; 2.80 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled |
| Luau (interpreter) | Normal | 142.05 ms | 5.31 ms | 135.89 ms | 132.68 ms | 43.6 MB | 799 B | 5.50 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 131.73 ms | 1.42 ms | 129.66 ms | 131.54 ms | 41.5 MB | 799 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 81.89 ms | 1.33 ms | 80.27 ms | 83.36 ms | 39.2 MB | 799 B | 2.58 ms as a whole process |  |
| Node `--jitless` | Normal | 115.79 ms | 1.95 ms | 113.37 ms | 122.05 ms | 213.2 MB | 878 B | 19.40 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.43 ms | 986.34 µs | 13.50 ms | 17.23 ms | 129.7 MB | 878 B | 18.96 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.46 ms | 71.99 µs | 13.34 ms | 17.00 ms | 223.6 MB | 878 B | 19.48 ms as a whole process |  |
| Seal | Ceil.source | 101.84 ms | 474.97 µs | 101.11 ms | 100.70 ms | 7.8 MB | 977 B | 124.38 µs cold, 208 ns warm; 2.83 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 102.10 ms | 314.26 µs | 101.70 ms | 102.25 ms | 7.8 MB | 977 B | 133.29 µs cold, 250 ns warm; 2.86 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.program | 84.11 ms | 433.42 µs | 83.25 ms | 84.68 ms | 7.6 MB | 977 B | 109.92 µs cold, 250 ns warm; 2.76 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 86.85 ms | 528.00 µs | 86.08 ms | 87.51 ms | 7.7 MB | 977 B | 103.00 µs cold, 167 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 132.89 ms | 1.37 ms | 130.16 ms | 129.19 ms | 41.3 MB | 799 B | 5.39 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 129.48 ms | 1.29 ms | 127.81 ms | 129.31 ms | 39.4 MB | 799 B | 2.48 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 83.48 ms | 1.81 ms | 80.51 ms | 82.72 ms | 42.8 MB | 799 B | 2.68 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 119.13 ms | 3.28 ms | 114.32 ms | 119.53 ms | 213.3 MB | 878 B | 19.71 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.44 ms | 963.39 µs | 13.43 ms | 17.64 ms | 129.7 MB | 878 B | 19.42 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.47 ms | 68.66 µs | 13.35 ms | 17.46 ms | 224.0 MB | 878 B | 19.98 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and the other engines compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 62.21 µs (±3.00 µs, least 57.08 µs) | 20.21 µs (±3.66 µs, least 16.33 µs) | 19 ns (±0 ns, least 19 ns) | 40 ns (±0 ns, least 40 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.75 µs (±1.61 µs, least 8.46 µs) | 20.24 µs (±4.92 µs, least 16.42 µs) | 42 ns (±2 ns, least 40 ns) | 22 ns (±2 ns, least 20 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 4.60 µs (±2.01 µs, least 3.00 µs) | 7.40 µs (±1.51 µs, least 6.00 µs) | 22 ns (±1 ns, least 21 ns) | 15 ns (±1 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.60 µs (±516 ns, least 3.00 µs) | 7.10 µs (±1.97 µs, least 5.00 µs) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.1 MB | the engine's own heap |
| Node `--jitless` | Purpose | 32.75 µs (±9.56 µs, least 19.83 µs) | 43.08 µs (±5.02 µs, least 37.00 µs) | 58 ns (±1 ns, least 56 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 32.47 µs (±7.02 µs, least 18.96 µs) | 40.22 µs (±2.08 µs, least 37.08 µs) | 9 ns (±0 ns, least 9 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 4.63 µs (±27 ns, least 4.59 µs) | 153 ns (±9 ns, least 141 ns) | 20 ns (±0 ns, least 20 ns) | 40 ns (±1 ns, least 39 ns) | 512 B | 4.5 MB | counted by Seal |
| Luau (interpreter) | Normal | 5.00 µs (±149 ns, least 4.72 µs) | 187 ns (±9 ns, least 172 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.81 µs (±31 ns, least 1.75 µs) | 184 ns (±9 ns, least 176 ns) | 28 ns (±1 ns, least 27 ns) | 16 ns (±2 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.82 µs (±25 ns, least 1.78 µs) | 365 ns (±10 ns, least 344 ns) | 99 ns (±4 ns, least 93 ns) | 108 ns (±6 ns, least 103 ns) | 1.0 KB | 4.7 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±31 ns, least 1.13 µs) | 699 ns (±31 ns, least 645 ns) | 52 ns (±2 ns, least 49 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.12 µs (±18 ns, least 1.07 µs) | 681 ns (±25 ns, least 648 ns) | 22 ns (±0 ns, least 21 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 4.61 µs (±86 ns, least 4.46 µs) | 156 ns (±7 ns, least 144 ns) | 20 ns (±0 ns, least 20 ns) | 41 ns (±1 ns, least 40 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.57 µs (±55 ns, least 4.43 µs) | 157 ns (±8 ns, least 140 ns) | 21 ns (±1 ns, least 20 ns) | 41 ns (±1 ns, least 40 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 3.94 µs (±47 ns, least 3.84 µs) | 147 ns (±4 ns, least 139 ns) | 18 ns (±1 ns, least 17 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.00 µs (±50 ns, least 3.94 µs) | 143 ns (±6 ns, least 131 ns) | 18 ns (±0 ns, least 18 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.94 µs (±186 ns, least 4.72 µs) | 189 ns (±14 ns, least 170 ns) | 34 ns (±0 ns, least 34 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.83 µs (±32 ns, least 1.79 µs) | 189 ns (±9 ns, least 175 ns) | 28 ns (±1 ns, least 26 ns) | 15 ns (±1 ns, least 13 ns) | 1.0 KB | 3.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.83 µs (±26 ns, least 1.79 µs) | 364 ns (±15 ns, least 341 ns) | 99 ns (±3 ns, least 93 ns) | 114 ns (±17 ns, least 101 ns) | 1.0 KB | 4.9 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.16 µs (±27 ns, least 1.11 µs) | 682 ns (±39 ns, least 632 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.14 µs (±32 ns, least 1.09 µs) | 680 ns (±17 ns, least 645 ns) | 22 ns (±1 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 35.97 ms | 32.21 ms (-10.5%) |
| n-body, 100,000 steps | 66.29 ms | 62.56 ms (-5.6%) |
| spectral-norm, size 200 | 19.21 ms | 19.24 ms (+0.2%) |
| fannkuch-redux, 9 | 107.90 ms | 107.67 ms (-0.2%) |
| word count, 500,000 words | 26.82 ms | 26.81 ms (-0.0%) |
| text building, 20,000 joins | 364.90 µs | 357.66 µs (-2.0%) |
| binary-trees, depth 14 | 101.38 ms | 103.48 ms (+2.1%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 170.27 ms | 1.32 ms | 168.67 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.56 ms | 206.86 µs | 10.30 ms | 2.8 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 3.77 ms | 35.05 µs | 3.72 ms | 3.9 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.31 ms | 32.39 µs | 1.28 ms | 3.6 MB |
