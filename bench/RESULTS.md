# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 1.99 1.70 1.65 }` before, `{ 1.70 1.65 1.63 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 50.60 ms | 71.43 ms | **met**: Seal takes 71% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.98 ms | 69.16 ms | **met**: Seal takes 97% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.86 ms | 26.96 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 110.90 ms | 208.24 ms | **met**: Seal takes 53% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 23.69 ms | 29.79 ms | **met**: Seal takes 80% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 347.00 µs | 159.32 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 110.88 ms | 128.95 ms | **met**: Seal takes 86% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.93 ms | 3.50 ms | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 33 ns | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 218 ns | 185 ns | **not met**: Seal takes 118% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 11.77 µs | 4.77 µs | **not met**: Seal takes 247% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 50.60 ms | 94.38 ms | **met**: Seal takes 54% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.98 ms | 175.30 ms | **met**: Seal takes 38% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 23.69 ms | 50.98 ms | **met**: Seal takes 46% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 347.00 µs | 503.26 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 36 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 155.58 ms | 1.25 ms | 153.79 ms | 178.49 ms | 2.8 MB | 395 B | 707.75 µs cold, 417 ns warm; 2.65 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 196.06 ms | 58.25 µs | 195.97 ms | 195.94 ms | 4.8 MB | 339 B | 5.11 ms as a whole process |  |
| Node `--jitless` | Purpose | 320.89 ms | 201.85 µs | 320.59 ms | 320.02 ms | 43.5 MB | 362 B | 18.10 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 121.68 ms | 157.91 µs | 121.43 ms | 122.82 ms | 49.2 MB | 362 B | 18.72 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 120.54 ms | 67.05 µs | 120.42 ms | 121.89 ms | 49.1 MB | 362 B | 18.39 ms as a whole process |  |
| Seal | Normal | 50.60 ms | 250.87 µs | 50.19 ms | 50.13 ms | 2.7 MB | 169 B | 82.67 µs cold, 333 ns warm; 2.65 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 71.43 ms | 200.77 µs | 71.06 ms | 71.63 ms | 4.8 MB | 143 B | 4.89 ms as a whole process |  |
| Node `--jitless` | Normal | 94.38 ms | 267.15 µs | 93.98 ms | 93.34 ms | 43.4 MB | 137 B | 18.57 ms as a whole process |  |
| Node (JIT), from cold | Normal | 10.23 ms | 922.45 µs | 9.33 ms | 11.66 ms | 48.5 MB | 137 B | 18.16 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.48 ms | 201.46 µs | 9.32 ms | 10.75 ms | 48.6 MB | 137 B | 18.24 ms as a whole process |  |
| Seal | Ceil.source | 50.17 ms | 356.65 µs | 49.55 ms | 50.46 ms | 2.7 MB | 169 B | 86.29 µs cold, 208 ns warm; 2.60 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 52.25 ms | 127.26 µs | 52.07 ms | 79.18 ms | 2.7 MB | 169 B | 1.02 ms cold, 625 ns warm; 2.59 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 45.63 ms | 92.54 µs | 45.49 ms | 71.67 ms | 2.5 MB | 169 B | 736.71 µs cold, 459 ns warm; 2.37 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.76 ms | 47.88 µs | 43.67 ms | 69.48 ms | 2.5 MB | 169 B | 776.08 µs cold, 500 ns warm; 2.41 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.40 ms | 136.83 µs | 71.14 ms | 72.05 ms | 4.8 MB | 143 B | 4.98 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.04 ms | 207.72 µs | 95.73 ms | 90.55 ms | 43.4 MB | 137 B | 18.87 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.86 ms | 483.68 µs | 9.47 ms | 9.95 ms | 48.5 MB | 137 B | 18.41 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.65 ms | 266.88 µs | 9.48 ms | 10.25 ms | 48.7 MB | 137 B | 18.35 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.96 ms | 422.54 µs | 53.45 ms | 56.82 ms | 2.7 MB | 550 B | 131.08 µs cold, 208 ns warm; 2.69 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.56 ms | 304.34 µs | 91.26 ms | 90.73 ms | 4.8 MB | 433 B | 5.02 ms as a whole process |  |
| Node `--jitless` | Purpose | 241.81 ms | 696.65 µs | 240.86 ms | 242.30 ms | 46.4 MB | 460 B | 18.36 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.34 ms | 158.31 µs | 18.12 ms | 18.24 ms | 50.3 MB | 460 B | 18.43 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.19 ms | 115.19 µs | 18.07 ms | 18.60 ms | 50.4 MB | 460 B | 18.28 ms as a whole process |  |
| Seal | Normal | 66.98 ms | 510.58 µs | 66.58 ms | 67.18 ms | 3.4 MB | 3.5 KB | 343.62 µs cold, 1.08 µs warm; 2.82 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.16 ms | 207.66 µs | 68.85 ms | 69.46 ms | 4.8 MB | 3.0 KB | 5.07 ms as a whole process |  |
| Node `--jitless` | Normal | 175.30 ms | 3.31 ms | 171.59 ms | 178.68 ms | 48.1 MB | 3.1 KB | 18.20 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.85 ms | 11.43 µs | 3.83 ms | 5.64 ms | 52.0 MB | 3.1 KB | 18.17 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.84 ms | 12.07 µs | 3.81 ms | 5.65 ms | 52.0 MB | 3.1 KB | 18.50 ms as a whole process |  |
| Seal | Ceil.source | 60.68 ms | 1.05 ms | 58.84 ms | 57.93 ms | 3.3 MB | 3.4 KB | 304.21 µs cold, 292 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 57.12 ms | 422.32 µs | 56.20 ms | 57.48 ms | 3.3 MB | 3.4 KB | 336.92 µs cold, 334 ns warm; 2.78 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.86 ms | 414.93 µs | 50.29 ms | 50.74 ms | 3.2 MB | 3.4 KB | 325.67 µs cold, 250 ns warm; 2.77 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.03 ms | 439.77 µs | 48.30 ms | 48.83 ms | 3.2 MB | 3.4 KB | 349.00 µs cold, 292 ns warm; 2.65 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.05 ms | 142.25 µs | 68.74 ms | 69.47 ms | 4.8 MB | 3.0 KB | 5.09 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 174.27 ms | 3.33 ms | 170.53 ms | 177.96 ms | 48.1 MB | 3.1 KB | 19.58 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.85 ms | 23.00 µs | 3.81 ms | 5.62 ms | 52.1 MB | 3.1 KB | 18.72 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.84 ms | 15.10 µs | 3.82 ms | 5.64 ms | 51.8 MB | 3.1 KB | 18.39 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.04 ms | 1.29 ms | 52.79 ms | 52.64 ms | 2.8 MB | 429 B | 98.67 µs cold, 250 ns warm; 2.54 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.27 ms | 227.72 µs | 45.02 ms | 44.96 ms | 4.8 MB | 366 B | 4.99 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.68 ms | 7.07 ms | 362.17 ms | 379.53 ms | 48.7 MB | 398 B | 18.22 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 33.85 ms | 36.46 ms | 15.47 ms | 19.25 ms | 52.6 MB | 398 B | 18.15 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.47 ms | 217.67 µs | 14.21 ms | 19.04 ms | 55.3 MB | 398 B | 18.80 ms as a whole process |  |
| Seal | Normal | 19.86 ms | 49.31 µs | 19.78 ms | 19.89 ms | 3.1 MB | 1.2 KB | 187.71 µs cold, 375 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 26.96 ms | 508.66 µs | 26.29 ms | 26.53 ms | 4.8 MB | 981 B | 4.88 ms as a whole process |  |
| Node `--jitless` | Normal | 64.70 ms | 653.59 µs | 63.53 ms | 63.66 ms | 48.5 MB | 1.0 KB | 17.95 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.61 ms | 143.46 µs | 1.51 ms | 3.53 ms | 52.8 MB | 1.0 KB | 18.71 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.63 ms | 13.66 µs | 1.60 ms | 3.15 ms | 52.8 MB | 1.0 KB | 18.21 ms as a whole process |  |
| Seal | Ceil.source | 19.80 ms | 46.45 µs | 19.74 ms | 19.88 ms | 3.1 MB | 1.2 KB | 180.88 µs cold, 250 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.07 ms | 58.16 µs | 20.01 ms | 20.00 ms | 3.1 MB | 1.2 KB | 192.62 µs cold, 292 ns warm; 2.68 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.08 ms | 77.76 µs | 18.97 ms | 19.04 ms | 2.9 MB | 1.2 KB | 167.67 µs cold, 250 ns warm; 2.63 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.76 ms | 16.56 µs | 18.74 ms | 18.76 ms | 3.0 MB | 1.2 KB | 169.08 µs cold, 250 ns warm; 2.50 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.12 ms | 422.35 µs | 26.59 ms | 26.77 ms | 4.8 MB | 981 B | 4.98 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.65 ms | 279.67 µs | 63.28 ms | 63.28 ms | 48.4 MB | 1.0 KB | 18.19 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.59 ms | 239.75 µs | 1.46 ms | 3.15 ms | 52.8 MB | 1.0 KB | 18.46 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.68 ms | 43.53 µs | 1.63 ms | 3.21 ms | 53.0 MB | 1.0 KB | 18.63 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.83 ms | 664.14 µs | 95.12 ms | 95.79 ms | 2.7 MB | 632 B | 112.50 µs cold, 292 ns warm; 2.55 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 87.09 ms | 750.03 µs | 86.13 ms | 89.14 ms | 4.8 MB | 523 B | 4.92 ms as a whole process |  |
| Node `--jitless` | Purpose | 145.27 ms | 439.41 µs | 144.63 ms | 147.65 ms | 43.4 MB | 568 B | 18.16 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.64 ms | 34.51 µs | 3.61 ms | 4.42 ms | 49.4 MB | 568 B | 18.61 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.66 ms | 42.09 µs | 3.61 ms | 4.51 ms | 49.4 MB | 568 B | 18.85 ms as a whole process |  |
| Seal | Normal | 110.90 ms | 1.09 ms | 109.60 ms | 112.44 ms | 3.0 MB | 1.8 KB | 174.25 µs cold, 333 ns warm; 2.71 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 208.24 ms | 2.10 ms | 202.31 ms | 210.18 ms | 4.8 MB | 1.7 KB | 4.98 ms as a whole process |  |
| Node `--jitless` | Normal | 154.67 ms | 819.27 µs | 153.52 ms | 156.96 ms | 45.4 MB | 1.7 KB | 18.79 ms as a whole process |  |
| Node (JIT), from cold | Normal | 13.97 ms | 134.55 µs | 13.74 ms | 15.41 ms | 52.0 MB | 1.7 KB | 18.35 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.95 ms | 99.88 µs | 13.77 ms | 15.56 ms | 52.1 MB | 1.7 KB | 17.93 ms as a whole process |  |
| Seal | Ceil.source | 115.13 ms | 475.67 µs | 114.59 ms | 114.90 ms | 3.0 MB | 1.8 KB | 172.17 µs cold, 291 ns warm; 2.71 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 112.63 ms | 189.10 µs | 112.33 ms | 112.47 ms | 3.0 MB | 1.8 KB | 189.67 µs cold, 291 ns warm; 2.67 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 103.85 ms | 477.59 µs | 103.28 ms | 104.49 ms | 2.9 MB | 1.8 KB | 167.42 µs cold, 250 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 101.12 ms | 217.40 µs | 100.78 ms | 101.20 ms | 2.9 MB | 1.8 KB | 155.00 µs cold, 292 ns warm; 2.70 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 209.57 ms | 1.56 ms | 205.28 ms | 211.34 ms | 4.9 MB | 1.7 KB | 4.86 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 154.65 ms | 342.00 µs | 154.18 ms | 156.37 ms | 45.4 MB | 1.7 KB | 18.30 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.12 ms | 71.71 µs | 14.01 ms | 15.64 ms | 52.0 MB | 1.7 KB | 18.23 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.48 ms | 137.83 µs | 14.25 ms | 16.29 ms | 52.1 MB | 1.7 KB | 18.33 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.53 ms | 249.95 µs | 95.22 ms | 93.85 ms | 4.0 MB | 569 B | 110.29 µs cold, 333 ns warm; 2.52 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 61.81 ms | 94.54 µs | 61.71 ms | 62.29 ms | 4.8 MB | 540 B | 4.94 ms as a whole process |  |
| Node `--jitless` | Purpose | 237.77 ms | 947.06 µs | 236.11 ms | 239.92 ms | 48.6 MB | 514 B | 18.89 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 84.37 ms | 23.80 ms | 63.07 ms | 86.34 ms | 52.6 MB | 514 B | 17.90 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 65.03 ms | 266.49 µs | 64.66 ms | 93.61 ms | 52.5 MB | 514 B | 18.25 ms as a whole process |  |
| Seal | Normal | 23.69 ms | 198.70 µs | 23.36 ms | 23.88 ms | 4.0 MB | 538 B | 117.17 µs cold, 333 ns warm; 2.64 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.79 ms | 96.87 µs | 29.73 ms | 29.86 ms | 6.0 MB | 478 B | 5.03 ms as a whole process |  |
| Node `--jitless` | Normal | 50.98 ms | 233.50 µs | 50.66 ms | 52.05 ms | 56.5 MB | 477 B | 18.53 ms as a whole process |  |
| Node (JIT), from cold | Normal | 34.67 ms | 283.49 µs | 34.33 ms | 35.37 ms | 62.5 MB | 477 B | 18.79 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.46 ms | 348.83 µs | 35.03 ms | 35.54 ms | 88.3 MB | 477 B | 18.06 ms as a whole process |  |
| Seal | Ceil.source | 24.55 ms | 920.20 µs | 23.54 ms | 24.07 ms | 4.0 MB | 538 B | 118.71 µs cold, 375 ns warm; 2.59 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.42 ms | 106.46 µs | 25.32 ms | 25.34 ms | 4.0 MB | 538 B | 121.38 µs cold, 334 ns warm; 2.56 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.33 ms | 174.53 µs | 23.09 ms | 23.20 ms | 3.9 MB | 538 B | 122.50 µs cold, 250 ns warm; 2.47 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 22.92 ms | 408.89 µs | 22.22 ms | 23.36 ms | 4.0 MB | 538 B | 107.46 µs cold, 250 ns warm; 2.61 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.72 ms | 82.03 µs | 29.59 ms | 29.77 ms | 5.9 MB | 478 B | 4.98 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 50.59 ms | 237.03 µs | 50.35 ms | 51.99 ms | 56.4 MB | 477 B | 18.44 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.74 ms | 311.60 µs | 34.49 ms | 35.14 ms | 62.5 MB | 477 B | 18.44 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 36.14 ms | 273.46 µs | 35.73 ms | 35.29 ms | 88.3 MB | 477 B | 17.73 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 317.89 µs | 2.60 µs | 315.46 µs | 328.83 µs | 3.0 MB | 233 B | 74.67 µs cold, 458 ns warm; 2.59 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 142.66 ms | 1.01 ms | 141.88 ms | 143.84 ms | 9.7 MB | 195 B | 4.99 ms as a whole process |  |
| Node `--jitless` | Purpose | 365.53 µs | 144.57 µs | 232.83 µs | 1.03 ms | 52.5 MB | 223 B | 18.79 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 215.40 µs | 146.47 µs | 91.38 µs | 908.46 µs | 57.7 MB | 223 B | 18.66 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 145.20 µs | 118.19 µs | 69.79 µs | 895.21 µs | 57.8 MB | 223 B | 19.10 ms as a whole process |  |
| Seal | Normal | 347.00 µs | 4.68 µs | 343.46 µs | 362.46 µs | 3.1 MB | 182 B | 76.42 µs cold, 334 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 159.32 ms | 189.33 µs | 159.06 ms | 161.30 ms | 11.8 MB | 138 B | 4.92 ms as a whole process |  |
| Node `--jitless` | Normal | 503.26 µs | 326.57 µs | 273.29 µs | 1.09 ms | 61.6 MB | 155 B | 18.49 ms as a whole process |  |
| Node (JIT), from cold | Normal | 498.80 µs | 494.41 µs | 122.79 µs | 1.04 ms | 65.2 MB | 155 B | 18.22 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 275.63 µs | 325.37 µs | 122.63 µs | 1.05 ms | 69.3 MB | 155 B | 18.50 ms as a whole process |  |
| Seal | Ceil.source | 341.73 µs | 2.41 µs | 339.17 µs | 365.42 µs | 3.1 MB | 182 B | 79.17 µs cold, 417 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 339.00 µs | 3.53 µs | 334.71 µs | 372.29 µs | 3.2 MB | 182 B | 78.54 µs cold, 375 ns warm; 2.44 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 274.76 µs | 1.89 µs | 271.17 µs | 299.88 µs | 3.0 MB | 182 B | 65.46 µs cold, 375 ns warm; 2.47 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 276.44 µs | 2.74 µs | 271.75 µs | 304.67 µs | 3.1 MB | 182 B | 65.67 µs cold, 292 ns warm; 2.36 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 159.17 ms | 194.75 µs | 158.95 ms | 161.26 ms | 11.3 MB | 138 B | 4.94 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 497.95 µs | 338.20 µs | 263.25 µs | 1.09 ms | 61.5 MB | 155 B | 18.22 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 481.99 µs | 469.53 µs | 119.33 µs | 1.08 ms | 65.2 MB | 155 B | 18.25 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 277.94 µs | 322.52 µs | 125.75 µs | 1.05 ms | 69.4 MB | 155 B | 18.35 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.82 ms | 841.32 µs | 74.75 ms | 75.24 ms | 3.0 MB | 537 B | 107.79 µs cold, 208 ns warm; 2.67 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 93.48 ms | 368.05 µs | 92.80 ms | 93.04 ms | 4.8 MB | 411 B | 4.92 ms as a whole process |  |
| Node `--jitless` | Purpose | 74.84 ms | 211.71 µs | 74.38 ms | 78.47 ms | 59.6 MB | 444 B | 18.66 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.38 ms | 279.37 µs | 10.11 ms | 12.89 ms | 65.8 MB | 444 B | 17.77 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.35 ms | 633.45 µs | 9.96 ms | 12.85 ms | 114.7 MB | 444 B | 18.78 ms as a whole process |  |
| Seal | Normal | 110.88 ms | 1.18 ms | 109.08 ms | 113.54 ms | 8.0 MB | 992 B | 160.50 µs cold, 250 ns warm; 2.54 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 128.95 ms | 1.34 ms | 126.92 ms | 128.24 ms | 41.8 MB | 799 B | 4.92 ms as a whole process |  |
| Node `--jitless` | Normal | 111.27 ms | 2.24 ms | 109.23 ms | 115.52 ms | 213.2 MB | 878 B | 18.55 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.13 ms | 878.56 µs | 13.42 ms | 16.74 ms | 129.6 MB | 878 B | 18.78 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.26 ms | 71.36 µs | 13.17 ms | 17.01 ms | 224.1 MB | 878 B | 18.51 ms as a whole process |  |
| Seal | Ceil.source | 111.64 ms | 1.44 ms | 109.78 ms | 112.98 ms | 8.0 MB | 992 B | 168.83 µs cold, 292 ns warm; 2.49 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.16 ms | 581.52 µs | 113.48 ms | 115.29 ms | 7.9 MB | 992 B | 159.38 µs cold, 292 ns warm; 2.65 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 90.98 ms | 382.30 µs | 90.43 ms | 91.57 ms | 7.7 MB | 992 B | 138.38 µs cold, 208 ns warm; 2.72 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 90.13 ms | 375.90 µs | 89.51 ms | 89.14 ms | 7.7 MB | 992 B | 142.17 µs cold, 208 ns warm; 2.64 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 127.94 ms | 1.06 ms | 125.90 ms | 128.70 ms | 39.2 MB | 799 B | 5.00 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 113.03 ms | 2.28 ms | 109.87 ms | 113.67 ms | 213.2 MB | 878 B | 18.30 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.15 ms | 880.15 µs | 13.31 ms | 16.78 ms | 129.7 MB | 878 B | 18.09 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.41 ms | 54.01 µs | 13.34 ms | 16.93 ms | 224.0 MB | 878 B | 17.99 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 64.37 µs (±5.98 µs, least 59.79 µs) | 37.39 µs (±3.89 µs, least 31.67 µs) | 16 ns (±0 ns, least 16 ns) | 36 ns (±1 ns, least 36 ns) | 512 B | 2.9 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.55 µs (±695 ns, least 9.17 µs) | 16.58 µs (±1.20 µs, least 15.25 µs) | 40 ns (±1 ns, least 38 ns) | 20 ns (±2 ns, least 18 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 36.16 µs (±11.76 µs, least 20.67 µs) | 37.44 µs (±1.25 µs, least 35.29 µs) | 56 ns (±1 ns, least 54 ns) | 30 ns (±1 ns, least 28 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 28.93 µs (±8.53 µs, least 18.67 µs) | 38.27 µs (±3.23 µs, least 35.29 µs) | 8 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 11.77 µs (±118 ns, least 11.53 µs) | 218 ns (±14 ns, least 201 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 5.2 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.77 µs (±162 ns, least 4.59 µs) | 185 ns (±7 ns, least 174 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.14 µs (±22 ns, least 1.11 µs) | 676 ns (±28 ns, least 639 ns) | 51 ns (±0 ns, least 50 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.10 µs (±20 ns, least 1.07 µs) | 682 ns (±21 ns, least 648 ns) | 22 ns (±0 ns, least 21 ns) | 6 ns (±0 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 11.95 µs (±274 ns, least 11.69 µs) | 208 ns (±6 ns, least 202 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 5.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 11.98 µs (±138 ns, least 11.81 µs) | 212 ns (±11 ns, least 198 ns) | 19 ns (±0 ns, least 18 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 5.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 10.82 µs (±294 ns, least 10.48 µs) | 196 ns (±13 ns, least 175 ns) | 15 ns (±0 ns, least 15 ns) | 35 ns (±0 ns, least 34 ns) | 512 B | 5.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 10.93 µs (±80 ns, least 10.84 µs) | 195 ns (±6 ns, least 189 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±1 ns, least 34 ns) | 512 B | 5.2 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.76 µs (±94 ns, least 4.64 µs) | 186 ns (±6 ns, least 178 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±10 ns, least 1.12 µs) | 670 ns (±24 ns, least 629 ns) | 50 ns (±1 ns, least 49 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.09 µs (±24 ns, least 1.06 µs) | 663 ns (±15 ns, least 645 ns) | 22 ns (±0 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 50.15 ms | 51.42 ms (+2.5%) |
| n-body, 100,000 steps | 65.59 ms | 60.56 ms (-7.7%) |
| spectral-norm, size 200 | 19.88 ms | 19.25 ms (-3.2%) |
| fannkuch-redux, 9 | 112.63 ms | 110.52 ms (-1.9%) |
| word count, 500,000 words | 23.71 ms | 23.89 ms (+0.7%) |
| text building, 20,000 joins | 340.57 µs | 342.90 µs (+0.7%) |
| binary-trees, depth 14 | 110.17 ms | 112.95 ms (+2.5%) |
