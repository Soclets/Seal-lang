# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.84 2.49 2.49 }` before, `{ 2.13 2.34 2.43 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 61.99 ms | 74.75 ms | **met**: Seal takes 83% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 67.74 ms | 70.07 ms | **met**: Seal takes 97% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 21.47 ms | 26.99 ms | **met**: Seal takes 80% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 123.67 ms | 219.93 ms | **met**: Seal takes 56% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.16 ms | 30.41 ms | **met**: Seal takes 86% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 281.90 µs | 161.68 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 115.25 ms | 148.29 ms | **met**: Seal takes 78% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.09 ms | 3.55 ms | **met**: Seal takes 59% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 19 ns | 33 ns | **met**: Seal takes 56% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 224 ns | 196 ns | **not met**: Seal takes 114% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 9.63 µs | 4.86 µs | **not met**: Seal takes 198% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 61.99 ms | 94.80 ms | **met**: Seal takes 65% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 67.74 ms | 175.68 ms | **met**: Seal takes 39% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.16 ms | 51.94 ms | **met**: Seal takes 50% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 281.90 µs | 505.92 µs | **met**: Seal takes 56% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 19 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 8.04 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 156.53 ms | 526.93 µs | 155.79 ms | 177.25 ms | 2.6 MB | 388 B | 621.54 µs cold, 625 ns warm; 2.85 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 198.83 ms | 1.71 ms | 196.24 ms | 200.79 ms | 4.9 MB | 339 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Purpose | 330.45 ms | 15.72 ms | 323.64 ms | 329.56 ms | 43.6 MB | 362 B | 19.84 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 123.96 ms | 1.72 ms | 121.78 ms | 126.29 ms | 49.3 MB | 362 B | 20.67 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 124.92 ms | 2.64 ms | 122.47 ms | 127.23 ms | 49.3 MB | 362 B | 20.27 ms as a whole process |  |
| Seal | Normal | 61.99 ms | 11.92 ms | 56.22 ms | 57.18 ms | 2.5 MB | 162 B | 69.50 µs cold, 208 ns warm; 2.92 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 74.75 ms | 1.18 ms | 73.39 ms | 74.22 ms | 4.9 MB | 143 B | 5.09 ms as a whole process |  |
| Node `--jitless` | Normal | 94.80 ms | 1.02 ms | 93.47 ms | 94.98 ms | 43.5 MB | 137 B | 19.34 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.42 ms | 13.34 µs | 9.41 ms | 11.05 ms | 48.7 MB | 137 B | 18.76 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.01 ms | 2.81 ms | 9.54 ms | 12.33 ms | 48.6 MB | 137 B | 18.80 ms as a whole process |  |
| Seal | Ceil.source | 56.31 ms | 272.07 µs | 55.97 ms | 56.27 ms | 2.5 MB | 162 B | 68.29 µs cold, 292 ns warm; 2.67 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 52.80 ms | 94.58 µs | 52.70 ms | 78.56 ms | 2.5 MB | 162 B | 747.71 µs cold, 667 ns warm; 2.69 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.10 ms | 266.94 µs | 72.84 ms | 72.91 ms | 4.9 MB | 143 B | 5.20 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.52 ms | 927.13 µs | 94.34 ms | 95.31 ms | 43.3 MB | 137 B | 19.45 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 10.15 ms | 757.47 µs | 9.57 ms | 11.43 ms | 48.7 MB | 137 B | 19.15 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.99 ms | 121.54 µs | 9.86 ms | 11.73 ms | 48.8 MB | 137 B | 18.55 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.27 ms | 141.06 µs | 54.05 ms | 54.15 ms | 2.5 MB | 543 B | 90.92 µs cold, 250 ns warm; 2.58 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.44 ms | 309.32 µs | 91.96 ms | 91.91 ms | 4.8 MB | 433 B | 5.13 ms as a whole process |  |
| Node `--jitless` | Purpose | 246.08 ms | 1.34 ms | 243.72 ms | 244.74 ms | 46.6 MB | 460 B | 19.29 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.98 ms | 276.41 µs | 18.60 ms | 18.85 ms | 50.5 MB | 460 B | 19.61 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.67 ms | 87.28 µs | 18.56 ms | 18.78 ms | 50.4 MB | 460 B | 19.34 ms as a whole process |  |
| Seal | Normal | 67.74 ms | 1.53 ms | 66.09 ms | 68.13 ms | 3.1 MB | 3.5 KB | 318.25 µs cold, 417 ns warm; 2.90 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.07 ms | 212.30 µs | 69.69 ms | 70.24 ms | 4.8 MB | 3.0 KB | 5.11 ms as a whole process |  |
| Node `--jitless` | Normal | 175.68 ms | 2.76 ms | 171.12 ms | 178.62 ms | 48.3 MB | 3.1 KB | 19.22 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 23.95 µs | 3.84 ms | 5.77 ms | 52.1 MB | 3.1 KB | 18.47 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 13.20 µs | 3.84 ms | 5.65 ms | 52.0 MB | 3.1 KB | 18.64 ms as a whole process |  |
| Seal | Ceil.source | 55.35 ms | 175.02 µs | 55.10 ms | 55.43 ms | 3.0 MB | 3.5 KB | 286.04 µs cold, 417 ns warm; 2.93 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.81 ms | 383.51 µs | 55.12 ms | 56.27 ms | 3.0 MB | 3.5 KB | 323.29 µs cold, 417 ns warm; 2.84 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.70 ms | 178.03 µs | 69.41 ms | 69.70 ms | 4.9 MB | 3.0 KB | 5.06 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 175.59 ms | 3.39 ms | 171.87 ms | 180.10 ms | 48.3 MB | 3.1 KB | 18.57 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 24.12 µs | 3.85 ms | 5.65 ms | 52.2 MB | 3.1 KB | 18.67 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.88 ms | 22.08 µs | 3.85 ms | 5.64 ms | 52.1 MB | 3.1 KB | 18.76 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 58.45 ms | 564.70 µs | 57.74 ms | 57.85 ms | 2.6 MB | 422 B | 78.42 µs cold, 375 ns warm; 2.65 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.71 ms | 314.76 µs | 45.35 ms | 45.82 ms | 4.9 MB | 366 B | 5.02 ms as a whole process |  |
| Node `--jitless` | Purpose | 370.56 ms | 6.20 ms | 360.07 ms | 377.77 ms | 48.8 MB | 398 B | 18.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.83 ms | 37.33 ms | 15.69 ms | 21.65 ms | 52.8 MB | 398 B | 19.28 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.87 ms | 652.99 µs | 14.41 ms | 20.98 ms | 55.4 MB | 398 B | 19.46 ms as a whole process |  |
| Seal | Normal | 21.47 ms | 170.56 µs | 21.26 ms | 21.20 ms | 2.8 MB | 1.2 KB | 165.92 µs cold, 458 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 26.99 ms | 191.25 µs | 26.67 ms | 27.57 ms | 4.8 MB | 981 B | 5.10 ms as a whole process |  |
| Node `--jitless` | Normal | 63.40 ms | 382.33 µs | 62.81 ms | 64.30 ms | 48.5 MB | 1.0 KB | 19.37 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.68 ms | 186.84 µs | 1.49 ms | 3.28 ms | 52.9 MB | 1.0 KB | 19.03 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 23.47 µs | 1.63 ms | 3.17 ms | 52.9 MB | 1.0 KB | 18.52 ms as a whole process |  |
| Seal | Ceil.source | 21.38 ms | 108.84 µs | 21.25 ms | 21.39 ms | 2.8 MB | 1.2 KB | 163.04 µs cold, 250 ns warm; 2.67 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.67 ms | 115.88 µs | 19.46 ms | 19.71 ms | 2.9 MB | 1.2 KB | 161.83 µs cold, 375 ns warm; 2.77 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.71 ms | 503.09 µs | 27.19 ms | 27.60 ms | 4.9 MB | 981 B | 5.21 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.88 ms | 528.61 µs | 63.25 ms | 64.60 ms | 48.5 MB | 1.0 KB | 18.81 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.62 ms | 172.18 µs | 1.49 ms | 3.65 ms | 53.1 MB | 1.0 KB | 18.89 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.68 ms | 55.18 µs | 1.63 ms | 3.33 ms | 53.1 MB | 1.0 KB | 18.58 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 108.53 ms | 356.11 µs | 107.80 ms | 108.56 ms | 2.5 MB | 625 B | 92.50 µs cold, 250 ns warm; 2.56 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.95 ms | 510.57 µs | 88.23 ms | 89.34 ms | 4.9 MB | 523 B | 5.01 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.05 ms | 706.96 µs | 147.19 ms | 148.71 ms | 43.4 MB | 568 B | 20.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.70 ms | 18.44 µs | 3.68 ms | 4.61 ms | 49.4 MB | 568 B | 20.19 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.70 ms | 24.81 µs | 3.67 ms | 4.58 ms | 49.4 MB | 568 B | 20.32 ms as a whole process |  |
| Seal | Normal | 123.67 ms | 3.45 ms | 121.06 ms | 123.06 ms | 2.8 MB | 1.8 KB | 171.29 µs cold, 291 ns warm; 2.80 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 219.93 ms | 2.45 ms | 213.61 ms | 220.52 ms | 4.9 MB | 1.7 KB | 5.57 ms as a whole process |  |
| Node `--jitless` | Normal | 155.67 ms | 1.74 ms | 153.46 ms | 161.53 ms | 45.5 MB | 1.7 KB | 18.74 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.80 ms | 98.15 µs | 14.59 ms | 16.31 ms | 52.2 MB | 1.7 KB | 18.37 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.25 ms | 204.06 µs | 13.92 ms | 15.76 ms | 52.3 MB | 1.7 KB | 18.47 ms as a whole process |  |
| Seal | Ceil.source | 117.13 ms | 168.72 µs | 116.82 ms | 118.15 ms | 2.8 MB | 1.8 KB | 153.00 µs cold, 375 ns warm; 2.72 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 120.40 ms | 848.41 µs | 118.83 ms | 120.90 ms | 2.8 MB | 1.8 KB | 161.21 µs cold, 333 ns warm; 2.80 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 212.37 ms | 2.75 ms | 205.50 ms | 210.62 ms | 4.9 MB | 1.7 KB | 5.24 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 154.23 ms | 284.33 µs | 153.64 ms | 155.72 ms | 45.6 MB | 1.7 KB | 19.04 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.53 ms | 79.57 µs | 14.42 ms | 16.06 ms | 52.2 MB | 1.7 KB | 18.67 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.81 ms | 157.80 µs | 14.59 ms | 16.40 ms | 52.2 MB | 1.7 KB | 18.51 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 98.66 ms | 1.09 ms | 97.51 ms | 96.71 ms | 3.7 MB | 564 B | 92.83 µs cold, 333 ns warm; 2.74 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.45 ms | 121.60 µs | 62.25 ms | 62.48 ms | 4.8 MB | 540 B | 5.09 ms as a whole process |  |
| Node `--jitless` | Purpose | 239.71 ms | 1.52 ms | 238.26 ms | 243.13 ms | 48.9 MB | 514 B | 18.62 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.93 ms | 24.15 ms | 65.15 ms | 90.39 ms | 52.7 MB | 514 B | 19.25 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 68.30 ms | 773.72 µs | 67.21 ms | 89.84 ms | 52.7 MB | 514 B | 18.49 ms as a whole process |  |
| Seal | Normal | 26.16 ms | 118.06 µs | 25.93 ms | 25.89 ms | 3.7 MB | 510 B | 94.42 µs cold, 334 ns warm; 2.52 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.41 ms | 291.11 µs | 29.87 ms | 29.74 ms | 6.0 MB | 478 B | 4.87 ms as a whole process |  |
| Node `--jitless` | Normal | 51.94 ms | 297.57 µs | 51.50 ms | 53.12 ms | 56.7 MB | 477 B | 20.30 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.67 ms | 300.33 µs | 35.20 ms | 36.86 ms | 62.6 MB | 477 B | 19.37 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.82 ms | 245.53 µs | 35.50 ms | 36.09 ms | 88.0 MB | 477 B | 18.90 ms as a whole process |  |
| Seal | Ceil.source | 26.28 ms | 569.73 µs | 25.85 ms | 25.93 ms | 3.7 MB | 510 B | 96.46 µs cold, 417 ns warm; 2.79 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 24.57 ms | 240.33 µs | 24.27 ms | 24.98 ms | 3.7 MB | 510 B | 152.96 µs cold, 375 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.70 ms | 353.94 µs | 30.27 ms | 30.72 ms | 6.0 MB | 478 B | 5.03 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.41 ms | 287.99 µs | 52.01 ms | 53.59 ms | 56.7 MB | 477 B | 19.77 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.57 ms | 449.48 µs | 34.88 ms | 36.15 ms | 62.7 MB | 477 B | 19.14 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.37 ms | 282.97 µs | 35.05 ms | 36.13 ms | 88.3 MB | 477 B | 18.67 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 220.15 µs | 8.17 µs | 209.04 µs | 222.88 µs | 2.8 MB | 227 B | 62.63 µs cold, 375 ns warm; 2.64 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.72 ms | 246.75 µs | 144.39 ms | 146.10 ms | 10.6 MB | 195 B | 5.07 ms as a whole process |  |
| Node `--jitless` | Purpose | 369.55 µs | 144.20 µs | 248.29 µs | 1.05 ms | 52.6 MB | 223 B | 18.85 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 221.03 µs | 152.83 µs | 89.62 µs | 929.00 µs | 57.9 MB | 223 B | 18.20 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 167.18 µs | 164.48 µs | 55.88 µs | 871.04 µs | 58.6 MB | 223 B | 18.10 ms as a whole process |  |
| Seal | Normal | 281.90 µs | 6.66 µs | 271.79 µs | 295.50 µs | 3.0 MB | 177 B | 64.12 µs cold, 458 ns warm; 2.57 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 161.68 ms | 489.90 µs | 161.01 ms | 163.45 ms | 12.2 MB | 138 B | 4.87 ms as a whole process |  |
| Node `--jitless` | Normal | 505.92 µs | 352.58 µs | 265.67 µs | 1.09 ms | 61.7 MB | 155 B | 18.52 ms as a whole process |  |
| Node (JIT), from cold | Normal | 497.00 µs | 487.43 µs | 125.08 µs | 1.01 ms | 65.4 MB | 155 B | 18.33 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 286.00 µs | 330.37 µs | 132.83 µs | 1.09 ms | 69.6 MB | 155 B | 18.73 ms as a whole process |  |
| Seal | Ceil.source | 279.91 µs | 3.52 µs | 276.12 µs | 306.92 µs | 3.0 MB | 177 B | 79.79 µs cold, 417 ns warm; 2.66 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 282.90 µs | 5.44 µs | 275.33 µs | 322.54 µs | 3.0 MB | 177 B | 64.62 µs cold, 292 ns warm; 2.60 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.77 ms | 506.59 µs | 160.73 ms | 165.17 ms | 12.1 MB | 138 B | 5.07 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 516.56 µs | 345.74 µs | 263.58 µs | 1.16 ms | 61.6 MB | 155 B | 18.57 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 499.00 µs | 457.55 µs | 142.08 µs | 1.05 ms | 65.5 MB | 155 B | 18.47 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 288.30 µs | 331.72 µs | 132.96 µs | 1.07 ms | 69.5 MB | 155 B | 18.71 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 77.30 ms | 536.43 µs | 76.38 ms | 82.41 ms | 2.7 MB | 530 B | 95.79 µs cold, 375 ns warm; 2.49 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 96.41 ms | 397.44 µs | 95.43 ms | 95.31 ms | 4.9 MB | 411 B | 5.07 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.01 ms | 557.76 µs | 74.48 ms | 79.27 ms | 59.7 MB | 444 B | 18.48 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.58 ms | 437.41 µs | 10.25 ms | 13.04 ms | 66.0 MB | 444 B | 18.81 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.50 ms | 742.39 µs | 10.10 ms | 13.08 ms | 114.9 MB | 444 B | 19.23 ms as a whole process |  |
| Seal | Normal | 115.25 ms | 850.26 µs | 114.04 ms | 117.81 ms | 7.7 MB | 985 B | 134.92 µs cold, 208 ns warm; 2.96 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 148.29 ms | 3.32 ms | 142.71 ms | 151.66 ms | 41.3 MB | 799 B | 5.52 ms as a whole process |  |
| Node `--jitless` | Normal | 118.33 ms | 3.24 ms | 114.57 ms | 128.23 ms | 213.5 MB | 878 B | 19.34 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.38 ms | 954.84 µs | 13.45 ms | 17.21 ms | 129.9 MB | 878 B | 18.82 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.51 ms | 227.34 µs | 13.31 ms | 17.65 ms | 224.4 MB | 878 B | 19.30 ms as a whole process |  |
| Seal | Ceil.source | 122.84 ms | 12.59 ms | 114.71 ms | 119.28 ms | 7.6 MB | 985 B | 137.33 µs cold, 250 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.16 ms | 1.24 ms | 113.01 ms | 116.56 ms | 7.6 MB | 985 B | 150.62 µs cold, 250 ns warm; 2.75 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 139.48 ms | 3.04 ms | 135.13 ms | 137.19 ms | 38.8 MB | 799 B | 5.37 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 116.10 ms | 2.16 ms | 113.65 ms | 121.51 ms | 213.5 MB | 878 B | 20.01 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.75 ms | 1.22 ms | 13.51 ms | 18.30 ms | 129.6 MB | 878 B | 19.74 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.59 ms | 156.32 µs | 13.31 ms | 17.38 ms | 223.5 MB | 878 B | 19.24 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 62.53 µs (±6.55 µs, least 53.38 µs) | 29.47 µs (±3.76 µs, least 25.21 µs) | 17 ns (±0 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 2.6 MB | counted by Seal |
| Luau (interpreter) | Purpose | 11.98 µs (±3.94 µs, least 8.79 µs) | 18.67 µs (±4.53 µs, least 16.38 µs) | 42 ns (±1 ns, least 39 ns) | 22 ns (±1 ns, least 20 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| Node `--jitless` | Purpose | 33.43 µs (±6.29 µs, least 20.08 µs) | 41.46 µs (±4.71 µs, least 37.75 µs) | 58 ns (±2 ns, least 55 ns) | 30 ns (±0 ns, least 29 ns) | 0 B | 45.9 MB | the engine's own heap |
| Node (JIT) | Purpose | 34.60 µs (±11.26 µs, least 18.75 µs) | 41.50 µs (±4.38 µs, least 37.25 µs) | 9 ns (±0 ns, least 9 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.3 MB | the engine's own heap |
| Seal | Normal | 9.63 µs (±113 ns, least 9.49 µs) | 224 ns (±10 ns, least 210 ns) | 19 ns (±0 ns, least 18 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 5.0 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.86 µs (±140 ns, least 4.73 µs) | 196 ns (±9 ns, least 183 ns) | 33 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.2 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±24 ns, least 1.12 µs) | 702 ns (±20 ns, least 679 ns) | 51 ns (±1 ns, least 50 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap |
| Node (JIT) | Normal | 1.15 µs (±59 ns, least 1.09 µs) | 694 ns (±24 ns, least 667 ns) | 23 ns (±1 ns, least 22 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 56.0 MB | the engine's own heap |
| Seal | Ceil.source | 9.62 µs (±185 ns, least 9.42 µs) | 223 ns (±12 ns, least 210 ns) | 18 ns (±1 ns, least 18 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 9.92 µs (±340 ns, least 9.47 µs) | 219 ns (±12 ns, least 204 ns) | 18 ns (±0 ns, least 18 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.89 µs (±167 ns, least 4.66 µs) | 190 ns (±9 ns, least 173 ns) | 34 ns (±1 ns, least 32 ns) | 18 ns (±1 ns, least 18 ns) | 2.0 KB | 6.2 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±43 ns, least 1.11 µs) | 682 ns (±23 ns, least 652 ns) | 51 ns (±2 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.13 µs (±20 ns, least 1.09 µs) | 678 ns (±20 ns, least 642 ns) | 22 ns (±1 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 57.39 ms | 55.56 ms (-3.2%) |
| n-body, 100,000 steps | 66.48 ms | 62.60 ms (-5.8%) |
| spectral-norm, size 200 | 21.30 ms | 21.19 ms (-0.5%) |
| fannkuch-redux, 9 | 119.64 ms | 118.80 ms (-0.7%) |
| word count, 500,000 words | 26.01 ms | 25.89 ms (-0.5%) |
| text building, 20,000 joins | 282.26 µs | 279.44 µs (-1.0%) |
| binary-trees, depth 14 | 114.58 ms | 116.70 ms (+1.9%) |
