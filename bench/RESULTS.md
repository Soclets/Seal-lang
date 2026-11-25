# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.74 2.30 2.05 }` before, `{ 2.56 2.31 2.09 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 48.15 ms | 70.03 ms | **met**: Seal takes 69% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.45 ms | 70.19 ms | **met**: Seal takes 95% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.86 ms | 28.56 ms | **met**: Seal takes 70% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 113.46 ms | 219.05 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 24.66 ms | 30.61 ms | **met**: Seal takes 81% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 342.41 µs | 166.87 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 111.43 ms | 140.05 ms | **met**: Seal takes 80% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.86 ms | 3.57 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 34 ns | **met**: Seal takes 51% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 154 ns | 186 ns | **met**: Seal takes 83% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.88 µs | 4.99 µs | **met**: Seal takes 98% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 48.15 ms | 95.48 ms | **met**: Seal takes 50% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.45 ms | 185.82 ms | **met**: Seal takes 36% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 24.66 ms | 52.93 ms | **met**: Seal takes 47% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 342.41 µs | 517.20 µs | **met**: Seal takes 66% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 132.41 ms | 169.76 µs | 132.20 ms | 151.51 ms | 2.8 MB | 395 B | 642.25 µs cold, 417 ns warm; 2.54 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 197.76 ms | 737.89 µs | 196.83 ms | 197.05 ms | 4.8 MB | 339 B | 5.09 ms as a whole process |  |
| Node `--jitless` | Purpose | 321.75 ms | 426.00 µs | 321.27 ms | 322.08 ms | 43.3 MB | 362 B | 19.29 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.97 ms | 892.04 µs | 122.30 ms | 123.72 ms | 49.2 MB | 362 B | 18.95 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 124.33 ms | 783.34 µs | 122.29 ms | 123.50 ms | 49.2 MB | 362 B | 19.08 ms as a whole process |  |
| Seal | Normal | 48.15 ms | 211.43 µs | 47.77 ms | 47.74 ms | 2.7 MB | 169 B | 78.88 µs cold, 333 ns warm; 2.67 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 70.03 ms | 342.54 µs | 69.45 ms | 71.08 ms | 4.8 MB | 143 B | 5.17 ms as a whole process |  |
| Node `--jitless` | Normal | 95.48 ms | 560.28 µs | 94.62 ms | 96.31 ms | 43.4 MB | 137 B | 19.49 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.93 ms | 657.86 µs | 9.63 ms | 10.07 ms | 48.5 MB | 137 B | 19.09 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.50 ms | 32.21 µs | 9.46 ms | 9.94 ms | 48.7 MB | 137 B | 19.13 ms as a whole process |  |
| Seal | Ceil.source | 48.08 ms | 205.75 µs | 47.81 ms | 47.81 ms | 2.7 MB | 169 B | 73.08 µs cold, 250 ns warm; 2.70 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 50.52 ms | 566.28 µs | 49.90 ms | 74.45 ms | 2.7 MB | 169 B | 942.75 µs cold, 541 ns warm; 2.71 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 43.80 ms | 183.64 µs | 43.57 ms | 68.06 ms | 2.4 MB | 169 B | 708.50 µs cold, 541 ns warm; 2.57 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.26 ms | 253.99 µs | 42.85 ms | 66.32 ms | 2.5 MB | 169 B | 718.04 µs cold, 542 ns warm; 2.64 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 74.07 ms | 321.16 µs | 73.56 ms | 72.94 ms | 4.8 MB | 143 B | 5.28 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.63 ms | 1.68 ms | 92.95 ms | 95.92 ms | 43.3 MB | 137 B | 19.33 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.88 ms | 373.39 µs | 9.62 ms | 9.86 ms | 48.6 MB | 137 B | 18.67 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.63 ms | 53.99 µs | 9.55 ms | 10.39 ms | 48.6 MB | 137 B | 18.86 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.43 ms | 465.00 µs | 51.93 ms | 56.45 ms | 2.7 MB | 550 B | 100.79 µs cold, 250 ns warm; 2.70 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.85 ms | 327.93 µs | 92.32 ms | 91.74 ms | 4.8 MB | 433 B | 5.34 ms as a whole process |  |
| Node `--jitless` | Purpose | 246.43 ms | 888.04 µs | 245.20 ms | 247.69 ms | 46.4 MB | 460 B | 19.06 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.66 ms | 164.52 µs | 18.46 ms | 18.68 ms | 50.2 MB | 460 B | 18.65 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.87 ms | 262.57 µs | 18.65 ms | 18.57 ms | 50.2 MB | 460 B | 20.68 ms as a whole process |  |
| Seal | Normal | 66.45 ms | 485.45 µs | 65.89 ms | 66.02 ms | 3.3 MB | 3.5 KB | 304.92 µs cold, 375 ns warm; 3.08 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.19 ms | 263.47 µs | 69.72 ms | 70.85 ms | 4.8 MB | 3.0 KB | 5.47 ms as a whole process |  |
| Node `--jitless` | Normal | 185.82 ms | 2.77 ms | 180.37 ms | 182.88 ms | 48.1 MB | 3.1 KB | 19.57 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.95 ms | 57.97 µs | 3.88 ms | 5.74 ms | 52.0 MB | 3.1 KB | 21.08 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 4.00 ms | 55.93 µs | 3.93 ms | 5.86 ms | 51.8 MB | 3.1 KB | 20.18 ms as a whole process |  |
| Seal | Ceil.source | 60.31 ms | 11.08 ms | 54.32 ms | 55.54 ms | 3.2 MB | 3.4 KB | 266.21 µs cold, 291 ns warm; 2.94 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 56.76 ms | 823.65 µs | 55.43 ms | 59.60 ms | 3.2 MB | 3.4 KB | 255.96 µs cold, 292 ns warm; 3.29 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.17 ms | 433.31 µs | 49.50 ms | 50.50 ms | 3.0 MB | 3.4 KB | 287.08 µs cold, 208 ns warm; 3.26 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.89 ms | 451.76 µs | 50.19 ms | 52.80 ms | 3.0 MB | 3.4 KB | 271.00 µs cold, 250 ns warm; 3.06 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 70.78 ms | 230.12 µs | 70.51 ms | 71.91 ms | 4.8 MB | 3.0 KB | 5.44 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 180.50 ms | 4.47 ms | 176.14 ms | 179.78 ms | 48.1 MB | 3.1 KB | 19.69 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.91 ms | 20.62 µs | 3.88 ms | 5.81 ms | 52.0 MB | 3.1 KB | 19.49 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 4.00 ms | 39.06 µs | 3.94 ms | 5.74 ms | 51.9 MB | 3.1 KB | 19.70 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 56.60 ms | 1.17 ms | 55.27 ms | 56.25 ms | 2.9 MB | 429 B | 95.92 µs cold, 292 ns warm; 2.78 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 46.13 ms | 288.02 µs | 45.72 ms | 46.04 ms | 4.8 MB | 366 B | 5.25 ms as a whole process |  |
| Node `--jitless` | Purpose | 381.36 ms | 6.49 ms | 370.69 ms | 385.24 ms | 48.5 MB | 398 B | 21.03 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 35.37 ms | 37.75 ms | 16.17 ms | 20.38 ms | 52.7 MB | 398 B | 20.01 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.69 ms | 181.44 µs | 14.48 ms | 22.03 ms | 55.3 MB | 398 B | 19.63 ms as a whole process |  |
| Seal | Normal | 19.86 ms | 106.75 µs | 19.77 ms | 19.90 ms | 3.0 MB | 1.2 KB | 142.12 µs cold, 250 ns warm; 2.96 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 28.56 ms | 353.71 µs | 28.13 ms | 28.07 ms | 4.8 MB | 981 B | 5.29 ms as a whole process |  |
| Node `--jitless` | Normal | 65.04 ms | 360.05 µs | 64.61 ms | 64.43 ms | 48.4 MB | 1.0 KB | 19.42 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.65 ms | 249.35 µs | 1.49 ms | 3.62 ms | 52.9 MB | 1.0 KB | 18.47 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.70 ms | 13.84 µs | 1.67 ms | 3.17 ms | 52.8 MB | 1.0 KB | 18.65 ms as a whole process |  |
| Seal | Ceil.source | 19.63 ms | 55.91 µs | 19.55 ms | 19.73 ms | 3.0 MB | 1.2 KB | 136.75 µs cold, 250 ns warm; 2.87 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.85 ms | 207.29 µs | 19.65 ms | 19.64 ms | 3.0 MB | 1.2 KB | 146.79 µs cold, 375 ns warm; 2.80 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.33 ms | 81.56 µs | 19.24 ms | 19.36 ms | 2.8 MB | 1.2 KB | 120.58 µs cold, 209 ns warm; 2.53 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.83 ms | 49.35 µs | 18.75 ms | 18.77 ms | 2.8 MB | 1.2 KB | 125.83 µs cold, 209 ns warm; 2.57 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.58 ms | 347.44 µs | 27.03 ms | 27.25 ms | 4.8 MB | 981 B | 5.22 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.48 ms | 189.10 µs | 64.27 ms | 65.11 ms | 48.5 MB | 1.0 KB | 19.33 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.63 ms | 209.24 µs | 1.47 ms | 3.26 ms | 52.8 MB | 1.0 KB | 18.90 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.63 ms | 12.64 µs | 1.62 ms | 3.21 ms | 52.9 MB | 1.0 KB | 19.10 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.24 ms | 545.97 µs | 96.34 ms | 97.02 ms | 2.7 MB | 632 B | 90.54 µs cold, 250 ns warm; 2.70 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.30 ms | 715.25 µs | 87.09 ms | 89.79 ms | 4.8 MB | 523 B | 5.27 ms as a whole process |  |
| Node `--jitless` | Purpose | 147.53 ms | 827.11 µs | 146.72 ms | 151.59 ms | 43.4 MB | 568 B | 18.98 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.70 ms | 42.91 µs | 3.63 ms | 4.46 ms | 49.2 MB | 568 B | 19.26 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.68 ms | 21.13 µs | 3.65 ms | 4.47 ms | 49.3 MB | 568 B | 19.03 ms as a whole process |  |
| Seal | Normal | 113.46 ms | 193.61 µs | 113.24 ms | 113.46 ms | 3.0 MB | 1.8 KB | 143.21 µs cold, 250 ns warm; 2.79 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 219.05 ms | 2.65 ms | 211.96 ms | 215.70 ms | 4.8 MB | 1.7 KB | 5.31 ms as a whole process |  |
| Node `--jitless` | Normal | 158.65 ms | 902.70 µs | 157.12 ms | 159.75 ms | 45.5 MB | 1.7 KB | 19.41 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.31 ms | 188.29 µs | 14.04 ms | 16.00 ms | 52.0 MB | 1.7 KB | 19.68 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.69 ms | 79.99 µs | 14.59 ms | 17.48 ms | 52.1 MB | 1.7 KB | 19.73 ms as a whole process |  |
| Seal | Ceil.source | 114.32 ms | 582.54 µs | 113.59 ms | 116.55 ms | 3.0 MB | 1.8 KB | 148.79 µs cold, 375 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 111.75 ms | 1.28 ms | 109.98 ms | 110.50 ms | 3.0 MB | 1.8 KB | 150.08 µs cold, 250 ns warm; 2.85 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 103.63 ms | 97.17 µs | 103.55 ms | 156.53 ms | 2.8 MB | 1.8 KB | 141.71 µs cold, 292 ns warm; 2.82 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.20 ms | 502.69 µs | 102.56 ms | 104.16 ms | 2.8 MB | 1.8 KB | 142.08 µs cold, 250 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 218.72 ms | 1.89 ms | 213.64 ms | 219.71 ms | 4.8 MB | 1.7 KB | 5.32 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 156.57 ms | 1.10 ms | 154.79 ms | 156.23 ms | 45.3 MB | 1.7 KB | 19.43 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.65 ms | 276.72 µs | 14.42 ms | 15.81 ms | 52.0 MB | 1.7 KB | 20.19 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.41 ms | 180.14 µs | 14.26 ms | 16.02 ms | 52.3 MB | 1.7 KB | 19.98 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.96 ms | 1.01 ms | 94.61 ms | 95.18 ms | 4.0 MB | 569 B | 99.33 µs cold, 333 ns warm; 2.87 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.24 ms | 290.26 µs | 62.81 ms | 63.62 ms | 4.8 MB | 540 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.20 ms | 2.20 ms | 241.27 ms | 246.46 ms | 48.6 MB | 514 B | 19.69 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 84.98 ms | 24.24 ms | 63.32 ms | 87.71 ms | 52.5 MB | 514 B | 19.02 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 70.43 ms | 860.69 µs | 69.72 ms | 96.33 ms | 52.7 MB | 514 B | 21.02 ms as a whole process |  |
| Seal | Normal | 24.66 ms | 333.25 µs | 24.29 ms | 24.80 ms | 4.0 MB | 538 B | 101.46 µs cold, 458 ns warm; 2.81 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.61 ms | 312.74 µs | 30.17 ms | 30.68 ms | 5.9 MB | 478 B | 5.22 ms as a whole process |  |
| Node `--jitless` | Normal | 52.93 ms | 425.35 µs | 52.09 ms | 54.90 ms | 56.5 MB | 477 B | 19.12 ms as a whole process |  |
| Node (JIT), from cold | Normal | 36.29 ms | 339.55 µs | 35.83 ms | 36.69 ms | 62.3 MB | 477 B | 19.22 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.69 ms | 210.87 µs | 35.46 ms | 36.72 ms | 88.3 MB | 477 B | 19.65 ms as a whole process |  |
| Seal | Ceil.source | 24.23 ms | 164.95 µs | 23.91 ms | 24.33 ms | 3.9 MB | 538 B | 95.96 µs cold, 333 ns warm; 2.88 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 24.44 ms | 242.34 µs | 24.17 ms | 24.24 ms | 3.9 MB | 538 B | 108.88 µs cold, 417 ns warm; 2.84 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.09 ms | 210.39 µs | 22.83 ms | 23.08 ms | 3.9 MB | 538 B | 94.42 µs cold, 291 ns warm; 2.78 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.55 ms | 133.22 µs | 24.39 ms | 24.60 ms | 3.9 MB | 538 B | 91.88 µs cold, 208 ns warm; 2.79 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.52 ms | 467.15 µs | 29.91 ms | 30.88 ms | 6.0 MB | 478 B | 5.43 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 53.09 ms | 373.94 µs | 52.61 ms | 54.84 ms | 56.4 MB | 477 B | 19.73 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 36.13 ms | 436.66 µs | 35.53 ms | 36.49 ms | 62.5 MB | 477 B | 19.63 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.95 ms | 567.49 µs | 35.06 ms | 36.72 ms | 88.2 MB | 477 B | 19.80 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 306.93 µs | 6.90 µs | 298.21 µs | 374.58 µs | 3.1 MB | 233 B | 73.62 µs cold, 375 ns warm; 2.77 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 147.48 ms | 1.02 ms | 145.68 ms | 147.76 ms | 11.1 MB | 195 B | 5.34 ms as a whole process |  |
| Node `--jitless` | Purpose | 386.37 µs | 140.17 µs | 256.75 µs | 1.01 ms | 52.5 MB | 223 B | 19.32 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 214.17 µs | 150.04 µs | 81.75 µs | 924.62 µs | 57.8 MB | 223 B | 19.41 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 180.57 µs | 180.82 µs | 63.04 µs | 965.46 µs | 58.5 MB | 223 B | 20.19 ms as a whole process |  |
| Seal | Normal | 342.41 µs | 4.95 µs | 336.75 µs | 388.04 µs | 3.2 MB | 182 B | 76.71 µs cold, 375 ns warm; 2.73 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 166.87 ms | 2.76 ms | 163.95 ms | 166.43 ms | 12.1 MB | 138 B | 5.66 ms as a whole process |  |
| Node `--jitless` | Normal | 517.20 µs | 361.77 µs | 274.88 µs | 1.19 ms | 61.4 MB | 155 B | 21.18 ms as a whole process |  |
| Node (JIT), from cold | Normal | 513.91 µs | 471.77 µs | 126.08 µs | 1.10 ms | 65.1 MB | 155 B | 20.14 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 289.31 µs | 343.89 µs | 126.62 µs | 1.07 ms | 69.2 MB | 155 B | 20.42 ms as a whole process |  |
| Seal | Ceil.source | 347.11 µs | 7.51 µs | 340.46 µs | 364.25 µs | 3.2 MB | 182 B | 74.71 µs cold, 333 ns warm; 2.67 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 355.61 µs | 9.66 µs | 344.21 µs | 378.96 µs | 3.2 MB | 182 B | 82.79 µs cold, 292 ns warm; 2.63 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 298.40 µs | 7.90 µs | 285.62 µs | 331.79 µs | 3.0 MB | 182 B | 60.75 µs cold, 209 ns warm; 2.68 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 292.62 µs | 12.60 µs | 277.79 µs | 320.42 µs | 3.0 MB | 182 B | 64.83 µs cold, 167 ns warm; 2.75 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 163.80 ms | 568.84 µs | 163.12 ms | 169.69 ms | 11.5 MB | 138 B | 5.21 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 521.81 µs | 337.69 µs | 272.54 µs | 1.12 ms | 61.4 MB | 155 B | 18.95 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 495.18 µs | 477.47 µs | 128.88 µs | 1.09 ms | 65.1 MB | 155 B | 18.94 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 285.51 µs | 322.40 µs | 129.75 µs | 1.05 ms | 69.2 MB | 155 B | 19.11 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 78.83 ms | 1.42 ms | 76.81 ms | 80.79 ms | 2.9 MB | 537 B | 96.71 µs cold, 334 ns warm; 2.87 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 102.33 ms | 15.41 ms | 94.71 ms | 97.46 ms | 4.8 MB | 411 B | 5.57 ms as a whole process |  |
| Node `--jitless` | Purpose | 78.04 ms | 808.49 µs | 76.96 ms | 82.12 ms | 59.7 MB | 444 B | 27.05 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 11.91 ms | 2.08 ms | 10.78 ms | 14.06 ms | 65.8 MB | 444 B | 19.68 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.81 ms | 588.69 µs | 10.26 ms | 13.37 ms | 114.6 MB | 444 B | 19.85 ms as a whole process |  |
| Seal | Normal | 111.43 ms | 624.53 µs | 110.73 ms | 114.18 ms | 7.8 MB | 992 B | 125.46 µs cold, 333 ns warm; 2.90 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 140.05 ms | 4.93 ms | 136.14 ms | 136.91 ms | 38.0 MB | 799 B | 5.54 ms as a whole process |  |
| Node `--jitless` | Normal | 115.52 ms | 2.42 ms | 113.48 ms | 173.11 ms | 213.4 MB | 878 B | 19.69 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.46 ms | 936.32 µs | 13.56 ms | 17.35 ms | 129.5 MB | 878 B | 21.85 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.73 ms | 169.54 µs | 13.51 ms | 18.43 ms | 224.2 MB | 878 B | 20.23 ms as a whole process |  |
| Seal | Ceil.source | 112.05 ms | 1.21 ms | 110.09 ms | 112.08 ms | 7.8 MB | 992 B | 120.42 µs cold, 333 ns warm; 2.65 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.72 ms | 1.15 ms | 112.12 ms | 115.52 ms | 7.8 MB | 992 B | 124.71 µs cold, 208 ns warm; 2.82 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 91.66 ms | 355.13 µs | 90.96 ms | 93.12 ms | 7.6 MB | 992 B | 121.25 µs cold, 250 ns warm; 2.67 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 92.51 ms | 930.62 µs | 91.70 ms | 93.82 ms | 7.6 MB | 992 B | 103.12 µs cold, 209 ns warm; 2.88 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 155.35 ms | 21.18 ms | 134.36 ms | 133.97 ms | 41.0 MB | 799 B | 5.33 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 115.72 ms | 1.80 ms | 113.78 ms | 120.58 ms | 213.0 MB | 878 B | 19.81 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.24 ms | 917.91 µs | 13.32 ms | 17.04 ms | 129.5 MB | 878 B | 18.98 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.37 ms | 82.35 µs | 13.20 ms | 17.32 ms | 223.8 MB | 878 B | 18.93 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.88 µs (±7.30 µs, least 45.71 µs) | 19.43 µs (±3.60 µs, least 14.62 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±1 ns, least 37 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.80 µs (±898 ns, least 8.67 µs) | 16.49 µs (±1.15 µs, least 15.04 µs) | 42 ns (±2 ns, least 40 ns) | 21 ns (±1 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 36.30 µs (±20.62 µs, least 20.58 µs) | 40.44 µs (±1.61 µs, least 37.62 µs) | 58 ns (±2 ns, least 54 ns) | 30 ns (±0 ns, least 30 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 29.87 µs (±9.00 µs, least 20.67 µs) | 44.18 µs (±9.41 µs, least 36.96 µs) | 9 ns (±0 ns, least 9 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 4.88 µs (±40 ns, least 4.81 µs) | 154 ns (±8 ns, least 139 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 4.4 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.99 µs (±109 ns, least 4.78 µs) | 186 ns (±10 ns, least 172 ns) | 34 ns (±0 ns, least 33 ns) | 18 ns (±1 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.16 µs (±27 ns, least 1.13 µs) | 693 ns (±22 ns, least 667 ns) | 52 ns (±1 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±22 ns, least 1.06 µs) | 681 ns (±22 ns, least 644 ns) | 22 ns (±0 ns, least 22 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.8 MB | the engine's own heap |
| Seal | Ceil.source | 4.73 µs (±33 ns, least 4.67 µs) | 151 ns (±8 ns, least 140 ns) | 19 ns (±4 ns, least 17 ns) | 37 ns (±1 ns, least 37 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 4.87 µs (±43 ns, least 4.78 µs) | 152 ns (±8 ns, least 143 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.24 µs (±27 ns, least 4.19 µs) | 140 ns (±5 ns, least 131 ns) | 14 ns (±0 ns, least 14 ns) | 35 ns (±1 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.29 µs (±40 ns, least 4.21 µs) | 141 ns (±8 ns, least 133 ns) | 14 ns (±1 ns, least 14 ns) | 34 ns (±0 ns, least 34 ns) | 512 B | 4.4 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.91 µs (±86 ns, least 4.80 µs) | 196 ns (±9 ns, least 182 ns) | 34 ns (±2 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.16 µs (±34 ns, least 1.12 µs) | 685 ns (±16 ns, least 668 ns) | 52 ns (±1 ns, least 50 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.43 µs (±628 ns, least 1.09 µs) | 800 ns (±272 ns, least 665 ns) | 28 ns (±12 ns, least 22 ns) | 8 ns (±2 ns, least 7 ns) | 643 B | 56.1 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 54.35 ms | 48.11 ms (-11.5%) |
| n-body, 100,000 steps | 66.00 ms | 61.33 ms (-7.1%) |
| spectral-norm, size 200 | 19.68 ms | 18.94 ms (-3.8%) |
| fannkuch-redux, 9 | 112.88 ms | 113.48 ms (+0.5%) |
| word count, 500,000 words | 24.26 ms | 23.87 ms (-1.6%) |
| text building, 20,000 joins | 345.72 µs | 344.80 µs (-0.3%) |
| binary-trees, depth 14 | 110.64 ms | 110.92 ms (+0.3%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 171.71 ms | 8.60 ms | 166.32 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.75 ms | 59.61 µs | 10.67 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.43 ms | 17.58 µs | 4.41 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.42 ms | 14.66 µs | 1.40 ms | 3.6 MB |
