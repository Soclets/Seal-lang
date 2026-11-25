# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.26 1.91 1.90 }` before, `{ 2.02 1.93 1.90 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 47.82 ms | 72.54 ms | **met**: Seal takes 66% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.50 ms | 69.77 ms | **met**: Seal takes 94% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.76 ms | 27.31 ms | **met**: Seal takes 72% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 113.10 ms | 216.85 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 23.87 ms | 30.68 ms | **met**: Seal takes 78% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 348.98 µs | 161.44 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 111.30 ms | 147.72 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.85 ms | 3.54 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 34 ns | **met**: Seal takes 51% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 149 ns | 189 ns | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 5.50 µs | 4.89 µs | **not met**: Seal takes 112% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 47.82 ms | 95.67 ms | **met**: Seal takes 50% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.50 ms | 177.97 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 23.87 ms | 52.61 ms | **met**: Seal takes 45% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 348.98 µs | 518.60 µs | **met**: Seal takes 67% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 132.58 ms | 887.00 µs | 131.23 ms | 150.10 ms | 2.8 MB | 395 B | 690.83 µs cold, 666 ns warm; 2.59 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 198.07 ms | 500.81 µs | 197.58 ms | 202.46 ms | 4.8 MB | 339 B | 5.15 ms as a whole process |  |
| Node `--jitless` | Purpose | 335.20 ms | 28.40 ms | 322.56 ms | 322.00 ms | 43.4 MB | 362 B | 20.48 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 132.40 ms | 4.16 ms | 126.01 ms | 126.94 ms | 49.2 MB | 362 B | 21.50 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 125.41 ms | 666.95 µs | 124.34 ms | 132.40 ms | 49.1 MB | 362 B | 19.52 ms as a whole process |  |
| Seal | Normal | 47.82 ms | 171.77 µs | 47.54 ms | 48.17 ms | 2.7 MB | 169 B | 77.83 µs cold, 333 ns warm; 2.57 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 72.54 ms | 502.58 µs | 71.51 ms | 73.51 ms | 4.8 MB | 143 B | 5.15 ms as a whole process |  |
| Node `--jitless` | Normal | 95.67 ms | 1.64 ms | 92.63 ms | 101.88 ms | 43.3 MB | 137 B | 19.68 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.57 ms | 279.95 µs | 9.43 ms | 10.51 ms | 48.6 MB | 137 B | 18.73 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.19 ms | 313.28 µs | 9.53 ms | 10.87 ms | 48.5 MB | 137 B | 18.42 ms as a whole process |  |
| Seal | Ceil.source | 49.56 ms | 4.46 ms | 47.61 ms | 47.71 ms | 2.7 MB | 169 B | 76.71 µs cold, 250 ns warm; 2.56 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 51.50 ms | 4.00 ms | 49.91 ms | 75.73 ms | 2.7 MB | 169 B | 1.18 ms cold, 791 ns warm; 2.62 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 43.77 ms | 74.04 µs | 43.67 ms | 67.48 ms | 2.5 MB | 169 B | 659.38 µs cold, 458 ns warm; 2.62 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.12 ms | 158.70 µs | 42.92 ms | 67.08 ms | 2.5 MB | 169 B | 776.92 µs cold, 1.38 µs warm; 2.68 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.21 ms | 441.68 µs | 70.70 ms | 71.91 ms | 4.8 MB | 143 B | 5.19 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 93.83 ms | 1.44 ms | 90.82 ms | 91.89 ms | 43.3 MB | 137 B | 18.71 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.52 ms | 46.86 µs | 9.44 ms | 11.51 ms | 48.6 MB | 137 B | 19.28 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 11.00 ms | 1.03 ms | 9.41 ms | 11.08 ms | 48.6 MB | 137 B | 19.15 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.77 ms | 97.19 µs | 52.65 ms | 52.51 ms | 2.7 MB | 550 B | 101.12 µs cold, 250 ns warm; 2.59 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.36 ms | 315.87 µs | 91.98 ms | 91.49 ms | 4.8 MB | 433 B | 5.18 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.25 ms | 906.82 µs | 243.80 ms | 246.93 ms | 46.4 MB | 460 B | 18.30 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.97 ms | 254.48 µs | 18.58 ms | 18.70 ms | 50.2 MB | 460 B | 19.15 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.91 ms | 273.71 µs | 18.36 ms | 19.16 ms | 50.4 MB | 460 B | 18.54 ms as a whole process |  |
| Seal | Normal | 65.50 ms | 1.11 ms | 64.26 ms | 66.18 ms | 3.2 MB | 3.5 KB | 276.83 µs cold, 250 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.77 ms | 207.87 µs | 69.27 ms | 70.01 ms | 4.9 MB | 3.0 KB | 5.24 ms as a whole process |  |
| Node `--jitless` | Normal | 177.97 ms | 3.40 ms | 173.47 ms | 181.85 ms | 48.0 MB | 3.1 KB | 18.86 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.89 ms | 13.70 µs | 3.87 ms | 5.68 ms | 52.0 MB | 3.1 KB | 18.80 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.88 ms | 16.40 µs | 3.86 ms | 5.73 ms | 52.0 MB | 3.1 KB | 19.02 ms as a whole process |  |
| Seal | Ceil.source | 55.26 ms | 673.42 µs | 53.76 ms | 55.03 ms | 3.1 MB | 3.4 KB | 238.54 µs cold, 250 ns warm; 2.76 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 53.62 ms | 390.35 µs | 53.11 ms | 53.72 ms | 3.2 MB | 3.4 KB | 270.88 µs cold, 292 ns warm; 2.91 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.93 ms | 266.59 µs | 49.62 ms | 49.63 ms | 2.9 MB | 3.4 KB | 279.12 µs cold, 292 ns warm; 2.75 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 51.57 ms | 1.01 ms | 50.22 ms | 50.44 ms | 2.9 MB | 3.4 KB | 259.88 µs cold, 250 ns warm; 2.88 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 71.15 ms | 802.19 µs | 70.00 ms | 69.77 ms | 4.9 MB | 3.0 KB | 5.73 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 180.22 ms | 5.39 ms | 174.12 ms | 185.46 ms | 48.1 MB | 3.1 KB | 19.26 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.92 ms | 19.95 µs | 3.89 ms | 5.75 ms | 52.0 MB | 3.1 KB | 19.94 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.90 ms | 24.10 µs | 3.86 ms | 5.67 ms | 52.1 MB | 3.1 KB | 19.03 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.35 ms | 1.50 ms | 54.54 ms | 59.01 ms | 2.8 MB | 429 B | 95.83 µs cold, 208 ns warm; 2.67 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.96 ms | 341.58 µs | 45.51 ms | 45.75 ms | 4.8 MB | 366 B | 5.11 ms as a whole process |  |
| Node `--jitless` | Purpose | 378.89 ms | 17.34 ms | 367.49 ms | 374.89 ms | 48.7 MB | 398 B | 19.41 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.51 ms | 37.47 ms | 15.65 ms | 19.73 ms | 52.7 MB | 398 B | 18.88 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.18 ms | 460.08 µs | 14.34 ms | 19.50 ms | 55.4 MB | 398 B | 19.77 ms as a whole process |  |
| Seal | Normal | 19.76 ms | 56.38 µs | 19.69 ms | 19.96 ms | 3.0 MB | 1.2 KB | 141.92 µs cold, 250 ns warm; 2.59 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.31 ms | 701.25 µs | 26.62 ms | 26.91 ms | 4.8 MB | 981 B | 5.19 ms as a whole process |  |
| Node `--jitless` | Normal | 62.90 ms | 308.97 µs | 62.50 ms | 62.74 ms | 48.4 MB | 1.0 KB | 18.46 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.65 ms | 159.31 µs | 1.49 ms | 3.60 ms | 52.9 MB | 1.0 KB | 18.93 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 12.01 µs | 1.65 ms | 3.25 ms | 52.7 MB | 1.0 KB | 18.75 ms as a whole process |  |
| Seal | Ceil.source | 19.78 ms | 47.61 µs | 19.68 ms | 19.84 ms | 3.0 MB | 1.2 KB | 148.83 µs cold, 250 ns warm; 2.77 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.00 ms | 149.69 µs | 19.73 ms | 19.93 ms | 3.0 MB | 1.2 KB | 153.25 µs cold, 208 ns warm; 2.64 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.25 ms | 67.79 µs | 19.09 ms | 19.25 ms | 2.8 MB | 1.2 KB | 126.38 µs cold, 209 ns warm; 2.72 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.88 ms | 92.58 µs | 18.70 ms | 18.75 ms | 2.8 MB | 1.2 KB | 126.25 µs cold, 167 ns warm; 2.56 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.39 ms | 904.22 µs | 26.37 ms | 27.01 ms | 4.8 MB | 981 B | 5.15 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.26 ms | 624.06 µs | 63.49 ms | 63.94 ms | 48.6 MB | 1.0 KB | 18.44 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.66 ms | 202.07 µs | 1.54 ms | 3.28 ms | 52.8 MB | 1.0 KB | 19.10 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.64 ms | 17.60 µs | 1.62 ms | 3.23 ms | 52.8 MB | 1.0 KB | 18.68 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.93 ms | 588.66 µs | 96.75 ms | 97.35 ms | 2.7 MB | 632 B | 95.92 µs cold, 250 ns warm; 2.95 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.90 ms | 1.20 ms | 87.05 ms | 90.97 ms | 4.8 MB | 523 B | 5.36 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.12 ms | 936.60 µs | 146.41 ms | 150.46 ms | 43.3 MB | 568 B | 19.37 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.69 ms | 40.46 µs | 3.64 ms | 4.52 ms | 49.3 MB | 568 B | 18.65 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.71 ms | 55.27 µs | 3.63 ms | 4.48 ms | 49.3 MB | 568 B | 19.38 ms as a whole process |  |
| Seal | Normal | 113.10 ms | 426.24 µs | 112.38 ms | 114.50 ms | 3.0 MB | 1.8 KB | 156.92 µs cold, 250 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 216.85 ms | 2.85 ms | 209.12 ms | 218.67 ms | 4.8 MB | 1.7 KB | 5.19 ms as a whole process |  |
| Node `--jitless` | Normal | 164.34 ms | 6.80 ms | 155.30 ms | 152.66 ms | 45.4 MB | 1.7 KB | 19.32 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.07 ms | 118.38 µs | 13.84 ms | 15.64 ms | 52.0 MB | 1.7 KB | 18.87 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.72 ms | 93.96 µs | 14.55 ms | 16.29 ms | 52.1 MB | 1.7 KB | 19.09 ms as a whole process |  |
| Seal | Ceil.source | 111.65 ms | 84.95 µs | 111.54 ms | 111.86 ms | 2.9 MB | 1.8 KB | 146.25 µs cold, 334 ns warm; 2.81 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 109.46 ms | 132.39 µs | 109.17 ms | 110.26 ms | 3.0 MB | 1.8 KB | 149.88 µs cold, 250 ns warm; 2.55 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 103.68 ms | 134.51 µs | 103.48 ms | 103.13 ms | 2.7 MB | 1.8 KB | 119.67 µs cold, 209 ns warm; 2.79 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.86 ms | 125.78 µs | 102.66 ms | 102.87 ms | 2.8 MB | 1.8 KB | 132.75 µs cold, 209 ns warm; 2.63 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 211.09 ms | 2.13 ms | 205.05 ms | 211.86 ms | 4.8 MB | 1.7 KB | 5.06 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 154.62 ms | 422.72 µs | 154.03 ms | 155.48 ms | 45.4 MB | 1.7 KB | 19.19 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.17 ms | 37.75 µs | 14.11 ms | 15.76 ms | 51.9 MB | 1.7 KB | 18.84 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.12 ms | 43.15 µs | 14.02 ms | 15.75 ms | 52.2 MB | 1.7 KB | 18.90 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.09 ms | 1.44 ms | 93.95 ms | 94.08 ms | 4.0 MB | 569 B | 98.54 µs cold, 417 ns warm; 2.67 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.99 ms | 92.75 µs | 62.80 ms | 63.06 ms | 4.8 MB | 540 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.74 ms | 1.71 ms | 240.83 ms | 242.77 ms | 48.7 MB | 514 B | 19.92 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 87.97 ms | 24.62 ms | 65.79 ms | 90.08 ms | 52.6 MB | 514 B | 19.61 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 65.13 ms | 362.37 µs | 64.62 ms | 87.68 ms | 52.6 MB | 514 B | 18.53 ms as a whole process |  |
| Seal | Normal | 23.87 ms | 161.47 µs | 23.63 ms | 24.08 ms | 3.9 MB | 538 B | 100.38 µs cold, 292 ns warm; 2.67 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.68 ms | 243.33 µs | 30.08 ms | 30.39 ms | 6.0 MB | 478 B | 5.17 ms as a whole process |  |
| Node `--jitless` | Normal | 52.61 ms | 382.38 µs | 51.98 ms | 54.23 ms | 56.4 MB | 477 B | 19.37 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.86 ms | 278.08 µs | 35.42 ms | 36.81 ms | 62.3 MB | 477 B | 19.32 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 34.72 ms | 222.54 µs | 34.47 ms | 36.53 ms | 87.9 MB | 477 B | 19.67 ms as a whole process |  |
| Seal | Ceil.source | 23.87 ms | 135.38 µs | 23.74 ms | 24.22 ms | 3.9 MB | 538 B | 101.00 µs cold, 291 ns warm; 2.62 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 24.15 ms | 121.11 µs | 23.93 ms | 24.00 ms | 4.0 MB | 538 B | 101.33 µs cold, 292 ns warm; 2.73 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 22.95 ms | 163.12 µs | 22.70 ms | 23.28 ms | 3.8 MB | 538 B | 106.71 µs cold, 291 ns warm; 2.70 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.28 ms | 130.47 µs | 23.98 ms | 24.25 ms | 3.9 MB | 538 B | 84.62 µs cold, 250 ns warm; 2.64 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.93 ms | 421.19 µs | 30.45 ms | 30.26 ms | 6.0 MB | 478 B | 5.25 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.46 ms | 639.28 µs | 51.74 ms | 52.96 ms | 56.6 MB | 477 B | 19.40 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.59 ms | 271.58 µs | 35.14 ms | 36.14 ms | 62.5 MB | 477 B | 19.55 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.75 ms | 714.99 µs | 35.08 ms | 36.59 ms | 88.0 MB | 477 B | 19.15 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 310.68 µs | 8.85 µs | 301.08 µs | 332.46 µs | 3.0 MB | 233 B | 71.62 µs cold, 375 ns warm; 2.60 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.05 ms | 879.26 µs | 142.94 ms | 145.28 ms | 10.3 MB | 195 B | 5.15 ms as a whole process |  |
| Node `--jitless` | Purpose | 366.05 µs | 142.70 µs | 239.17 µs | 1.01 ms | 52.4 MB | 223 B | 19.13 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 217.51 µs | 149.52 µs | 88.38 µs | 904.88 µs | 57.7 MB | 223 B | 18.94 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 175.40 µs | 169.09 µs | 62.88 µs | 914.62 µs | 58.5 MB | 223 B | 19.22 ms as a whole process |  |
| Seal | Normal | 348.98 µs | 5.54 µs | 343.04 µs | 357.50 µs | 3.2 MB | 182 B | 73.04 µs cold, 292 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 161.44 ms | 930.51 µs | 160.22 ms | 165.92 ms | 12.1 MB | 138 B | 5.16 ms as a whole process |  |
| Node `--jitless` | Normal | 518.60 µs | 337.71 µs | 272.79 µs | 1.14 ms | 61.3 MB | 155 B | 19.40 ms as a whole process |  |
| Node (JIT), from cold | Normal | 493.63 µs | 477.36 µs | 136.50 µs | 1.10 ms | 65.2 MB | 155 B | 18.52 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 275.82 µs | 331.93 µs | 123.08 µs | 1.09 ms | 69.4 MB | 155 B | 19.45 ms as a whole process |  |
| Seal | Ceil.source | 342.61 µs | 4.31 µs | 337.29 µs | 351.00 µs | 3.2 MB | 182 B | 74.67 µs cold, 333 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 349.05 µs | 8.54 µs | 336.79 µs | 375.12 µs | 3.2 MB | 182 B | 74.38 µs cold, 250 ns warm; 2.57 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 291.77 µs | 11.59 µs | 280.08 µs | 319.92 µs | 3.0 MB | 182 B | 55.88 µs cold, 291 ns warm; 2.46 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 288.22 µs | 5.47 µs | 280.29 µs | 319.75 µs | 3.0 MB | 182 B | 61.75 µs cold, 250 ns warm; 2.59 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.78 ms | 4.92 ms | 159.59 ms | 162.98 ms | 12.2 MB | 138 B | 5.47 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 547.29 µs | 395.12 µs | 286.62 µs | 1.09 ms | 61.5 MB | 155 B | 19.65 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 496.06 µs | 495.58 µs | 121.08 µs | 1.05 ms | 65.1 MB | 155 B | 19.93 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 292.32 µs | 348.65 µs | 125.38 µs | 1.08 ms | 69.3 MB | 155 B | 20.44 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.64 ms | 471.55 µs | 74.82 ms | 75.82 ms | 2.9 MB | 537 B | 94.21 µs cold, 166 ns warm; 2.72 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 97.24 ms | 1.88 ms | 95.34 ms | 95.47 ms | 4.8 MB | 411 B | 5.25 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.31 ms | 396.71 µs | 75.63 ms | 79.87 ms | 59.5 MB | 444 B | 19.26 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.54 ms | 278.69 µs | 10.22 ms | 13.19 ms | 65.7 MB | 444 B | 20.07 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.66 ms | 623.65 µs | 10.22 ms | 13.22 ms | 114.8 MB | 444 B | 20.13 ms as a whole process |  |
| Seal | Normal | 111.30 ms | 1.01 ms | 109.86 ms | 112.38 ms | 7.8 MB | 992 B | 120.38 µs cold, 208 ns warm; 2.83 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 147.72 ms | 21.98 ms | 132.35 ms | 134.82 ms | 39.7 MB | 799 B | 5.18 ms as a whole process |  |
| Node `--jitless` | Normal | 114.62 ms | 2.16 ms | 112.45 ms | 115.22 ms | 213.3 MB | 878 B | 19.36 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.32 ms | 910.13 µs | 13.33 ms | 17.05 ms | 129.5 MB | 878 B | 19.00 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.82 ms | 127.74 µs | 13.66 ms | 16.99 ms | 224.0 MB | 878 B | 19.19 ms as a whole process |  |
| Seal | Ceil.source | 110.95 ms | 1.08 ms | 109.40 ms | 111.79 ms | 7.8 MB | 992 B | 155.00 µs cold, 208 ns warm; 2.70 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 111.76 ms | 546.89 µs | 110.74 ms | 114.72 ms | 7.8 MB | 992 B | 123.25 µs cold, 208 ns warm; 2.86 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 92.04 ms | 654.66 µs | 90.94 ms | 92.54 ms | 7.6 MB | 992 B | 108.29 µs cold, 209 ns warm; 2.81 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 91.85 ms | 492.94 µs | 90.91 ms | 92.26 ms | 7.6 MB | 992 B | 114.88 µs cold, 167 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 133.00 ms | 3.41 ms | 126.26 ms | 135.35 ms | 40.2 MB | 799 B | 5.26 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 112.94 ms | 1.47 ms | 111.24 ms | 115.90 ms | 213.3 MB | 878 B | 18.95 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.33 ms | 960.13 µs | 13.41 ms | 17.10 ms | 129.4 MB | 878 B | 18.77 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.51 ms | 169.95 µs | 13.31 ms | 17.16 ms | 224.0 MB | 878 B | 19.23 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 64.01 µs (±6.49 µs, least 55.67 µs) | 22.97 µs (±4.36 µs, least 17.96 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 2.8 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.45 µs (±704 ns, least 9.46 µs) | 17.32 µs (±864 ns, least 15.54 µs) | 41 ns (±1 ns, least 40 ns) | 22 ns (±2 ns, least 20 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 32.83 µs (±6.69 µs, least 20.54 µs) | 38.80 µs (±1.87 µs, least 35.21 µs) | 57 ns (±1 ns, least 56 ns) | 30 ns (±0 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 35.35 µs (±6.31 µs, least 21.38 µs) | 38.97 µs (±5.03 µs, least 35.21 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 5.50 µs (±49 ns, least 5.45 µs) | 149 ns (±4 ns, least 142 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 4.6 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.89 µs (±102 ns, least 4.77 µs) | 189 ns (±16 ns, least 173 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±21 ns, least 1.12 µs) | 658 ns (±18 ns, least 628 ns) | 51 ns (±1 ns, least 49 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±26 ns, least 1.07 µs) | 667 ns (±29 ns, least 631 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 5.48 µs (±61 ns, least 5.41 µs) | 143 ns (±6 ns, least 135 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 4.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 5.55 µs (±56 ns, least 5.48 µs) | 148 ns (±6 ns, least 139 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 4.6 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 4.91 µs (±56 ns, least 4.84 µs) | 142 ns (±11 ns, least 130 ns) | 14 ns (±0 ns, least 14 ns) | 35 ns (±0 ns, least 34 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 4.94 µs (±58 ns, least 4.86 µs) | 140 ns (±7 ns, least 128 ns) | 14 ns (±0 ns, least 14 ns) | 34 ns (±0 ns, least 34 ns) | 512 B | 4.5 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.87 µs (±116 ns, least 4.67 µs) | 190 ns (±9 ns, least 180 ns) | 33 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.14 µs (±15 ns, least 1.12 µs) | 668 ns (±22 ns, least 637 ns) | 50 ns (±1 ns, least 48 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±15 ns, least 1.08 µs) | 676 ns (±32 ns, least 643 ns) | 22 ns (±0 ns, least 22 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 47.85 ms | 47.81 ms (-0.1%) |
| n-body, 100,000 steps | 67.13 ms | 63.06 ms (-6.1%) |
| spectral-norm, size 200 | 19.91 ms | 18.82 ms (-5.5%) |
| fannkuch-redux, 9 | 113.15 ms | 112.66 ms (-0.4%) |
| word count, 500,000 words | 23.53 ms | 23.82 ms (+1.2%) |
| text building, 20,000 joins | 350.47 µs | 337.49 µs (-3.7%) |
| binary-trees, depth 14 | 110.71 ms | 108.56 ms (-1.9%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 171.97 ms | 812.25 µs | 170.47 ms | 2.8 MB |
| the same sums in `Float`s, as a yardstick | 10.67 ms | 55.90 µs | 10.58 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.43 ms | 12.99 µs | 4.42 ms | 3.3 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.42 ms | 51.86 µs | 1.40 ms | 3.3 MB |
