# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.24 2.38 2.33 }` before, `{ 2.13 2.23 2.27 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 49.46 ms | 71.94 ms | **met**: Seal takes 69% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 68.76 ms | 71.04 ms | **met**: Seal takes 97% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.08 ms | 27.30 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 109.86 ms | 210.96 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 24.41 ms | 29.99 ms | **met**: Seal takes 81% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 339.45 µs | 164.34 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 111.95 ms | 147.72 ms | **met**: Seal takes 76% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.15 ms | 3.61 ms | **met**: Seal takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 20 ns | 34 ns | **met**: Seal takes 58% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 171 ns | 185 ns | **met**: Seal takes 92% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 8.16 µs | 4.99 µs | **not met**: Seal takes 164% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 49.46 ms | 94.65 ms | **met**: Seal takes 52% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 68.76 ms | 180.74 ms | **met**: Seal takes 38% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 24.41 ms | 51.18 ms | **met**: Seal takes 48% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 339.45 µs | 548.11 µs | **met**: Seal takes 62% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 20 ns | host call 39 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 136.40 ms | 278.82 µs | 135.87 ms | 142.97 ms | 2.8 MB | 395 B | 533.88 µs cold, 292 ns warm; 2.67 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 201.17 ms | 5.12 ms | 196.32 ms | 197.84 ms | 4.8 MB | 339 B | 5.19 ms as a whole process |  |
| Node `--jitless` | Purpose | 328.87 ms | 1.05 ms | 327.50 ms | 328.79 ms | 43.4 MB | 362 B | 18.76 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 126.15 ms | 708.86 µs | 124.90 ms | 130.54 ms | 49.3 MB | 362 B | 19.35 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 125.94 ms | 323.41 µs | 125.36 ms | 125.53 ms | 49.2 MB | 362 B | 18.67 ms as a whole process |  |
| Seal | Normal | 49.46 ms | 266.18 µs | 49.04 ms | 49.06 ms | 2.7 MB | 169 B | 86.58 µs cold, 209 ns warm; 2.80 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 71.94 ms | 895.37 µs | 70.83 ms | 71.84 ms | 4.8 MB | 143 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Normal | 94.65 ms | 2.48 ms | 92.07 ms | 93.49 ms | 43.3 MB | 137 B | 20.14 ms as a whole process |  |
| Node (JIT), from cold | Normal | 10.16 ms | 374.83 µs | 9.69 ms | 12.36 ms | 48.6 MB | 137 B | 19.93 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.59 ms | 41.44 µs | 9.52 ms | 9.97 ms | 48.7 MB | 137 B | 19.06 ms as a whole process |  |
| Seal | Ceil.source | 49.46 ms | 330.39 µs | 48.99 ms | 49.29 ms | 2.7 MB | 169 B | 83.33 µs cold, 167 ns warm; 2.53 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 56.57 ms | 453.39 µs | 55.74 ms | 67.32 ms | 2.7 MB | 169 B | 633.50 µs cold, 333 ns warm; 3.16 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 44.50 ms | 196.34 µs | 44.27 ms | 53.30 ms | 2.5 MB | 169 B | 438.88 µs cold, 250 ns warm; 2.91 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 44.02 ms | 181.84 µs | 43.79 ms | 43.82 ms | 2.5 MB | 169 B | 360.54 µs cold, 167 ns warm; 2.73 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.67 ms | 365.95 µs | 71.26 ms | 72.91 ms | 4.8 MB | 143 B | 5.13 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.86 ms | 594.68 µs | 94.39 ms | 96.53 ms | 43.4 MB | 137 B | 19.63 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.77 ms | 189.63 µs | 9.57 ms | 10.38 ms | 48.6 MB | 137 B | 19.36 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.06 ms | 1.43 ms | 9.52 ms | 10.73 ms | 48.5 MB | 137 B | 19.90 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.16 ms | 201.90 µs | 53.73 ms | 54.46 ms | 2.8 MB | 550 B | 156.12 µs cold, 208 ns warm; 2.86 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.52 ms | 426.68 µs | 91.80 ms | 91.86 ms | 4.8 MB | 433 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.63 ms | 658.92 µs | 244.27 ms | 248.96 ms | 46.4 MB | 460 B | 20.17 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.30 ms | 217.68 µs | 19.05 ms | 19.46 ms | 50.4 MB | 460 B | 20.22 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.14 ms | 285.22 µs | 18.77 ms | 20.02 ms | 50.3 MB | 460 B | 21.56 ms as a whole process |  |
| Seal | Normal | 68.76 ms | 3.37 ms | 66.43 ms | 67.87 ms | 3.3 MB | 3.5 KB | 349.79 µs cold, 250 ns warm; 3.03 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 71.04 ms | 363.17 µs | 70.35 ms | 71.02 ms | 4.8 MB | 3.0 KB | 5.41 ms as a whole process |  |
| Node `--jitless` | Normal | 180.74 ms | 5.14 ms | 174.83 ms | 182.28 ms | 48.1 MB | 3.1 KB | 19.13 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.92 ms | 25.64 µs | 3.88 ms | 6.17 ms | 51.8 MB | 3.1 KB | 20.20 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.93 ms | 36.72 µs | 3.88 ms | 5.81 ms | 52.1 MB | 3.1 KB | 19.94 ms as a whole process |  |
| Seal | Ceil.source | 56.00 ms | 412.00 µs | 55.18 ms | 55.45 ms | 3.2 MB | 3.4 KB | 312.21 µs cold, 250 ns warm; 2.89 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 59.05 ms | 892.64 µs | 58.05 ms | 58.38 ms | 3.2 MB | 3.4 KB | 298.71 µs cold, 959 ns warm; 2.91 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.19 ms | 272.91 µs | 49.85 ms | 50.41 ms | 3.0 MB | 3.4 KB | 267.88 µs cold, 209 ns warm; 2.98 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.99 ms | 207.21 µs | 49.59 ms | 49.87 ms | 3.1 MB | 3.4 KB | 302.21 µs cold, 250 ns warm; 2.91 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.60 ms | 288.84 µs | 69.29 ms | 69.86 ms | 4.8 MB | 3.0 KB | 5.27 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 175.02 ms | 3.70 ms | 171.09 ms | 178.81 ms | 48.1 MB | 3.1 KB | 19.43 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 15.19 µs | 3.86 ms | 5.70 ms | 52.0 MB | 3.1 KB | 19.24 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.89 ms | 18.41 µs | 3.85 ms | 5.74 ms | 52.0 MB | 3.1 KB | 19.28 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.61 ms | 561.58 µs | 51.83 ms | 53.20 ms | 2.8 MB | 429 B | 119.29 µs cold, 208 ns warm; 2.72 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.90 ms | 339.91 µs | 45.33 ms | 45.27 ms | 4.8 MB | 366 B | 4.95 ms as a whole process |  |
| Node `--jitless` | Purpose | 369.57 ms | 4.85 ms | 364.73 ms | 373.04 ms | 48.7 MB | 398 B | 19.02 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.13 ms | 36.12 ms | 16.23 ms | 19.76 ms | 52.7 MB | 398 B | 19.09 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.51 ms | 36.03 µs | 15.45 ms | 21.42 ms | 55.2 MB | 398 B | 18.87 ms as a whole process |  |
| Seal | Normal | 20.08 ms | 48.85 µs | 20.00 ms | 20.10 ms | 3.0 MB | 1.2 KB | 176.96 µs cold, 208 ns warm; 2.77 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.30 ms | 375.14 µs | 26.84 ms | 26.94 ms | 4.8 MB | 981 B | 5.28 ms as a whole process |  |
| Node `--jitless` | Normal | 63.67 ms | 886.33 µs | 62.69 ms | 63.73 ms | 48.4 MB | 1.0 KB | 19.21 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.61 ms | 174.22 µs | 1.50 ms | 3.86 ms | 52.8 MB | 1.0 KB | 20.32 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 26.51 µs | 1.64 ms | 3.41 ms | 52.8 MB | 1.0 KB | 19.65 ms as a whole process |  |
| Seal | Ceil.source | 20.41 ms | 179.62 µs | 20.18 ms | 20.29 ms | 3.0 MB | 1.2 KB | 186.38 µs cold, 250 ns warm; 2.82 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.25 ms | 18.79 µs | 20.23 ms | 20.32 ms | 3.1 MB | 1.2 KB | 172.25 µs cold, 250 ns warm; 2.87 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.29 ms | 50.43 µs | 19.21 ms | 19.45 ms | 2.8 MB | 1.2 KB | 152.96 µs cold, 208 ns warm; 2.85 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 19.21 ms | 115.07 µs | 19.08 ms | 19.35 ms | 2.9 MB | 1.2 KB | 149.12 µs cold, 208 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.35 ms | 490.77 µs | 26.86 ms | 27.07 ms | 4.8 MB | 981 B | 5.18 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 67.78 ms | 580.38 µs | 66.89 ms | 67.22 ms | 48.4 MB | 1.0 KB | 20.43 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.71 ms | 236.40 µs | 1.54 ms | 3.52 ms | 52.8 MB | 1.0 KB | 20.07 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.69 ms | 34.65 µs | 1.63 ms | 4.10 ms | 52.8 MB | 1.0 KB | 19.03 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 98.33 ms | 1.39 ms | 96.21 ms | 96.27 ms | 2.7 MB | 632 B | 116.58 µs cold, 209 ns warm; 2.75 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.76 ms | 1.22 ms | 87.54 ms | 87.50 ms | 4.8 MB | 523 B | 5.18 ms as a whole process |  |
| Node `--jitless` | Purpose | 147.72 ms | 642.18 µs | 146.64 ms | 150.00 ms | 43.3 MB | 568 B | 18.79 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.71 ms | 62.86 µs | 3.63 ms | 4.83 ms | 49.5 MB | 568 B | 19.66 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.70 ms | 27.84 µs | 3.67 ms | 4.45 ms | 49.3 MB | 568 B | 19.11 ms as a whole process |  |
| Seal | Normal | 109.86 ms | 385.36 µs | 109.43 ms | 112.57 ms | 3.0 MB | 1.8 KB | 182.67 µs cold, 291 ns warm; 2.85 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 210.96 ms | 1.94 ms | 205.54 ms | 213.81 ms | 4.8 MB | 1.7 KB | 5.14 ms as a whole process |  |
| Node `--jitless` | Normal | 155.03 ms | 922.98 µs | 152.99 ms | 155.77 ms | 45.4 MB | 1.7 KB | 19.12 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.26 ms | 227.39 µs | 14.07 ms | 15.63 ms | 52.1 MB | 1.7 KB | 19.14 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.08 ms | 155.02 µs | 13.94 ms | 15.55 ms | 52.2 MB | 1.7 KB | 18.70 ms as a whole process |  |
| Seal | Ceil.source | 110.60 ms | 354.32 µs | 110.11 ms | 112.71 ms | 3.0 MB | 1.8 KB | 180.88 µs cold, 291 ns warm; 2.87 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 112.71 ms | 215.65 µs | 112.40 ms | 112.97 ms | 3.0 MB | 1.8 KB | 165.04 µs cold, 250 ns warm; 2.77 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 107.30 ms | 9.06 ms | 103.43 ms | 103.71 ms | 2.8 MB | 1.8 KB | 138.33 µs cold, 167 ns warm; 2.65 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 101.70 ms | 468.95 µs | 101.01 ms | 101.50 ms | 2.8 MB | 1.8 KB | 172.83 µs cold, 333 ns warm; 2.85 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 211.25 ms | 1.68 ms | 207.00 ms | 212.82 ms | 4.8 MB | 1.7 KB | 5.07 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.53 ms | 9.68 ms | 163.13 ms | 161.75 ms | 45.4 MB | 1.7 KB | 18.65 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.73 ms | 111.15 µs | 14.55 ms | 16.16 ms | 51.9 MB | 1.7 KB | 19.16 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.69 ms | 59.12 µs | 14.61 ms | 16.22 ms | 52.0 MB | 1.7 KB | 19.27 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.75 ms | 922.71 µs | 94.64 ms | 94.16 ms | 4.0 MB | 569 B | 147.62 µs cold, 333 ns warm; 2.68 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.75 ms | 291.45 µs | 62.20 ms | 62.61 ms | 4.8 MB | 540 B | 5.21 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.74 ms | 1.21 ms | 243.89 ms | 247.15 ms | 48.6 MB | 514 B | 18.84 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.49 ms | 24.75 ms | 64.04 ms | 89.52 ms | 52.5 MB | 514 B | 19.16 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 65.35 ms | 246.40 µs | 65.01 ms | 90.24 ms | 52.8 MB | 514 B | 19.30 ms as a whole process |  |
| Seal | Normal | 24.41 ms | 123.19 µs | 24.16 ms | 24.62 ms | 3.9 MB | 538 B | 108.79 µs cold, 250 ns warm; 2.53 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.99 ms | 78.06 µs | 29.90 ms | 30.03 ms | 6.0 MB | 478 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Normal | 51.18 ms | 247.04 µs | 50.86 ms | 52.69 ms | 56.5 MB | 477 B | 18.70 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.51 ms | 372.91 µs | 35.01 ms | 36.09 ms | 62.5 MB | 477 B | 19.27 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.22 ms | 200.98 µs | 34.94 ms | 36.37 ms | 88.0 MB | 477 B | 18.95 ms as a whole process |  |
| Seal | Ceil.source | 24.01 ms | 196.93 µs | 23.76 ms | 24.17 ms | 3.9 MB | 538 B | 108.42 µs cold, 292 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 24.46 ms | 82.92 µs | 24.34 ms | 24.79 ms | 4.0 MB | 538 B | 112.88 µs cold, 375 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.32 ms | 209.42 µs | 23.04 ms | 23.50 ms | 3.9 MB | 538 B | 103.33 µs cold, 209 ns warm; 2.72 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.21 ms | 162.08 µs | 22.89 ms | 23.18 ms | 3.9 MB | 538 B | 109.17 µs cold, 291 ns warm; 2.69 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.93 ms | 95.79 µs | 29.77 ms | 30.09 ms | 6.0 MB | 478 B | 5.12 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.01 ms | 298.49 µs | 51.59 ms | 53.08 ms | 56.4 MB | 477 B | 18.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.80 ms | 283.30 µs | 35.37 ms | 36.31 ms | 62.5 MB | 477 B | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 37.19 ms | 3.12 ms | 35.55 ms | 35.77 ms | 88.3 MB | 477 B | 19.56 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 303.97 µs | 5.97 µs | 299.08 µs | 309.25 µs | 3.0 MB | 233 B | 108.58 µs cold, 292 ns warm; 2.45 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 146.78 ms | 1.98 ms | 145.07 ms | 147.11 ms | 10.0 MB | 195 B | 5.19 ms as a whole process |  |
| Node `--jitless` | Purpose | 370.21 µs | 150.49 µs | 241.42 µs | 1.03 ms | 52.5 MB | 223 B | 18.70 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 215.33 µs | 148.67 µs | 91.75 µs | 876.62 µs | 57.8 MB | 223 B | 18.99 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 132.70 µs | 118.95 µs | 54.71 µs | 884.12 µs | 57.9 MB | 223 B | 19.26 ms as a whole process |  |
| Seal | Normal | 339.45 µs | 4.41 µs | 334.75 µs | 369.96 µs | 3.2 MB | 182 B | 76.58 µs cold, 292 ns warm; 2.63 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 164.34 ms | 5.72 ms | 161.70 ms | 164.18 ms | 12.3 MB | 138 B | 6.10 ms as a whole process |  |
| Node `--jitless` | Normal | 548.11 µs | 376.96 µs | 273.79 µs | 1.35 ms | 61.4 MB | 155 B | 22.20 ms as a whole process |  |
| Node (JIT), from cold | Normal | 498.88 µs | 468.59 µs | 129.92 µs | 1.04 ms | 65.1 MB | 155 B | 19.39 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 281.07 µs | 328.43 µs | 116.71 µs | 1.03 ms | 69.4 MB | 155 B | 19.56 ms as a whole process |  |
| Seal | Ceil.source | 345.72 µs | 6.55 µs | 335.75 µs | 360.58 µs | 3.2 MB | 182 B | 78.83 µs cold, 292 ns warm; 2.49 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 349.78 µs | 6.51 µs | 341.83 µs | 400.62 µs | 3.2 MB | 182 B | 82.12 µs cold, 292 ns warm; 2.67 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 284.15 µs | 6.54 µs | 279.25 µs | 307.88 µs | 3.0 MB | 182 B | 65.88 µs cold, 250 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 291.99 µs | 8.80 µs | 280.62 µs | 315.08 µs | 3.0 MB | 182 B | 62.29 µs cold, 250 ns warm; 2.43 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 169.18 ms | 7.02 ms | 163.02 ms | 173.18 ms | 12.5 MB | 138 B | 5.18 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 552.30 µs | 390.18 µs | 302.29 µs | 1.22 ms | 61.4 MB | 155 B | 20.82 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 529.91 µs | 511.79 µs | 132.46 µs | 1.15 ms | 65.2 MB | 155 B | 21.45 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 284.11 µs | 347.28 µs | 122.67 µs | 1.05 ms | 69.4 MB | 155 B | 19.31 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 76.93 ms | 2.32 ms | 75.03 ms | 77.27 ms | 2.9 MB | 537 B | 120.42 µs cold, 209 ns warm; 2.67 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 97.55 ms | 2.35 ms | 95.25 ms | 94.48 ms | 4.8 MB | 411 B | 5.62 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.04 ms | 3.71 ms | 74.39 ms | 80.77 ms | 59.6 MB | 444 B | 18.99 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.59 ms | 330.53 µs | 10.24 ms | 13.07 ms | 65.8 MB | 444 B | 18.48 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.43 ms | 778.31 µs | 10.03 ms | 12.97 ms | 114.7 MB | 444 B | 19.18 ms as a whole process |  |
| Seal | Normal | 111.95 ms | 1.96 ms | 109.90 ms | 110.47 ms | 7.8 MB | 992 B | 143.88 µs cold, 209 ns warm; 3.06 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 147.72 ms | 20.20 ms | 127.87 ms | 131.83 ms | 44.6 MB | 799 B | 5.70 ms as a whole process |  |
| Node `--jitless` | Normal | 115.45 ms | 1.83 ms | 114.17 ms | 120.85 ms | 213.2 MB | 878 B | 19.23 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.26 ms | 934.08 µs | 13.33 ms | 17.12 ms | 129.6 MB | 878 B | 18.15 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.74 ms | 534.88 µs | 13.40 ms | 17.02 ms | 224.2 MB | 878 B | 22.68 ms as a whole process |  |
| Seal | Ceil.source | 112.64 ms | 809.72 µs | 111.44 ms | 119.13 ms | 7.8 MB | 992 B | 142.21 µs cold, 208 ns warm; 2.77 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 119.71 ms | 2.45 ms | 114.35 ms | 113.62 ms | 7.8 MB | 992 B | 150.50 µs cold, 250 ns warm; 3.07 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 110.40 ms | 56.43 ms | 90.45 ms | 92.35 ms | 7.7 MB | 992 B | 129.46 µs cold, 125 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 92.33 ms | 1.04 ms | 90.45 ms | 92.81 ms | 7.6 MB | 992 B | 125.71 µs cold, 208 ns warm; 2.74 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 131.39 ms | 1.98 ms | 128.75 ms | 132.26 ms | 43.4 MB | 799 B | 5.15 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.45 ms | 2.67 ms | 112.84 ms | 116.40 ms | 213.1 MB | 878 B | 19.28 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.59 ms | 929.50 µs | 13.65 ms | 17.33 ms | 129.3 MB | 878 B | 20.10 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.73 ms | 443.61 µs | 13.30 ms | 17.41 ms | 224.1 MB | 878 B | 19.24 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 63.90 µs (±6.91 µs, least 57.54 µs) | 26.57 µs (±3.03 µs, least 22.17 µs) | 16 ns (±0 ns, least 15 ns) | 38 ns (±1 ns, least 38 ns) | 512 B | 2.8 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.12 µs (±1.04 µs, least 8.83 µs) | 18.16 µs (±2.86 µs, least 15.00 µs) | 41 ns (±2 ns, least 39 ns) | 21 ns (±1 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 33.27 µs (±7.02 µs, least 20.46 µs) | 66.64 µs (±86.04 µs, least 37.08 µs) | 57 ns (±1 ns, least 54 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 36.15 µs (±15.10 µs, least 21.88 µs) | 40.14 µs (±5.98 µs, least 35.92 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 8.16 µs (±105 ns, least 7.99 µs) | 171 ns (±24 ns, least 151 ns) | 20 ns (±9 ns, least 17 ns) | 39 ns (±6 ns, least 37 ns) | 512 B | 4.8 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.99 µs (±157 ns, least 4.79 µs) | 185 ns (±9 ns, least 170 ns) | 34 ns (±2 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.16 µs (±19 ns, least 1.13 µs) | 693 ns (±21 ns, least 667 ns) | 52 ns (±2 ns, least 49 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.13 µs (±23 ns, least 1.10 µs) | 710 ns (±114 ns, least 658 ns) | 22 ns (±0 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 8.17 µs (±86 ns, least 8.04 µs) | 152 ns (±8 ns, least 142 ns) | 18 ns (±1 ns, least 17 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 4.8 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 8.17 µs (±114 ns, least 7.92 µs) | 152 ns (±9 ns, least 142 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 4.8 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 7.39 µs (±176 ns, least 7.13 µs) | 141 ns (±9 ns, least 132 ns) | 14 ns (±0 ns, least 14 ns) | 35 ns (±0 ns, least 34 ns) | 512 B | 4.7 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 7.56 µs (±149 ns, least 7.30 µs) | 148 ns (±8 ns, least 139 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±1 ns, least 34 ns) | 512 B | 4.7 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.80 µs (±91 ns, least 4.64 µs) | 188 ns (±10 ns, least 173 ns) | 34 ns (±1 ns, least 32 ns) | 18 ns (±1 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.17 µs (±80 ns, least 1.11 µs) | 691 ns (±45 ns, least 651 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.13 µs (±20 ns, least 1.10 µs) | 677 ns (±12 ns, least 661 ns) | 22 ns (±0 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 49.47 ms | 48.31 ms (-2.3%) |
| n-body, 100,000 steps | 65.90 ms | 61.28 ms (-7.0%) |
| spectral-norm, size 200 | 20.27 ms | 19.75 ms (-2.6%) |
| fannkuch-redux, 9 | 111.21 ms | 115.64 ms (+4.0%) |
| word count, 500,000 words | 24.23 ms | 24.35 ms (+0.5%) |
| text building, 20,000 joins | 342.27 µs | 344.97 µs (+0.8%) |
| binary-trees, depth 14 | 113.18 ms | 113.23 ms (+0.0%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 176.74 ms | 883.06 µs | 175.04 ms | 2.8 MB |
| the same sums in `Float`s, as a yardstick | 11.12 ms | 86.92 µs | 11.02 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.49 ms | 30.57 µs | 4.45 ms | 3.4 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.42 ms | 10.31 µs | 1.41 ms | 3.3 MB |
