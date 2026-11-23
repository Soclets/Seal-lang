# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 1.99 2.15 2.33 }` before, `{ 1.74 2.15 2.30 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 55.52 ms | 70.98 ms | **met**: Seal takes 78% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 68.45 ms | 69.67 ms | **met**: Seal takes 98% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 22.35 ms | 27.43 ms | **met**: Seal takes 81% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 124.69 ms | 219.51 ms | **met**: Seal takes 57% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 26.02 ms | 30.78 ms | **met**: Seal takes 85% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 295.39 µs | 165.11 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 117.59 ms | 136.64 ms | **met**: Seal takes 86% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.09 ms | 3.58 ms | **met**: Seal takes 58% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 19 ns | 34 ns | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 227 ns | 187 ns | **not met**: Seal takes 122% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 9.73 µs | 4.96 µs | **not met**: Seal takes 196% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 55.52 ms | 95.24 ms | **met**: Seal takes 58% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 68.45 ms | 181.03 ms | **met**: Seal takes 38% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 26.02 ms | 52.02 ms | **met**: Seal takes 50% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 295.39 µs | 530.85 µs | **met**: Seal takes 56% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 19 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |
| 4. A 10,000-line script loads in under 100 ms | a generated script of 9996 lines | 8.00 ms best of 10 | 100 ms | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 154.79 ms | 628.93 µs | 154.17 ms | 160.53 ms | 2.6 MB | 388 B | 397.08 µs cold, 375 ns warm; 2.61 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 196.84 ms | 344.80 µs | 196.46 ms | 197.28 ms | 4.9 MB | 339 B | 5.04 ms as a whole process |  |
| Node `--jitless` | Purpose | 321.50 ms | 807.39 µs | 319.92 ms | 319.71 ms | 43.5 MB | 362 B | 18.69 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 122.09 ms | 263.43 µs | 121.81 ms | 122.20 ms | 49.4 MB | 362 B | 18.68 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 119.79 ms | 253.83 µs | 119.49 ms | 122.23 ms | 49.3 MB | 362 B | 18.60 ms as a whole process |  |
| Seal | Normal | 55.52 ms | 119.19 µs | 55.36 ms | 55.97 ms | 2.5 MB | 162 B | 69.92 µs cold, 208 ns warm; 2.64 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 70.98 ms | 344.94 µs | 70.53 ms | 72.01 ms | 4.8 MB | 143 B | 4.91 ms as a whole process |  |
| Node `--jitless` | Normal | 95.24 ms | 1.00 ms | 94.00 ms | 95.42 ms | 43.6 MB | 137 B | 19.46 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.81 ms | 453.79 µs | 9.54 ms | 10.04 ms | 48.7 MB | 137 B | 20.20 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.36 ms | 155.68 µs | 10.23 ms | 11.65 ms | 48.8 MB | 137 B | 19.18 ms as a whole process |  |
| Seal | Ceil.source | 56.85 ms | 412.44 µs | 56.24 ms | 57.23 ms | 2.5 MB | 162 B | 71.00 µs cold, 250 ns warm; 2.55 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 54.27 ms | 474.18 µs | 53.50 ms | 82.22 ms | 2.5 MB | 162 B | 704.79 µs cold, 750 ns warm; 2.76 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 43.20 ms | 362.86 µs | 42.79 ms | 54.66 ms | 2.4 MB | 162 B | 471.46 µs cold, 416 ns warm; 2.84 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 60.89 ms | 15.85 ms | 51.47 ms | 79.22 ms | 2.4 MB | 162 B | 681.83 µs cold, 541 ns warm; 2.78 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.78 ms | 2.07 ms | 70.86 ms | 74.52 ms | 4.9 MB | 143 B | 5.01 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 97.38 ms | 4.70 ms | 93.02 ms | 93.73 ms | 43.5 MB | 137 B | 18.64 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.71 ms | 312.60 µs | 9.53 ms | 9.90 ms | 48.8 MB | 137 B | 19.08 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.53 ms | 87.77 µs | 9.41 ms | 10.24 ms | 48.8 MB | 137 B | 19.12 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.64 ms | 245.18 µs | 54.29 ms | 54.45 ms | 2.5 MB | 543 B | 97.67 µs cold, 250 ns warm; 2.64 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 93.59 ms | 3.92 ms | 91.73 ms | 91.17 ms | 4.8 MB | 433 B | 5.13 ms as a whole process |  |
| Node `--jitless` | Purpose | 251.60 ms | 8.89 ms | 241.53 ms | 246.82 ms | 46.5 MB | 460 B | 19.98 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.96 ms | 222.29 µs | 18.50 ms | 19.01 ms | 50.6 MB | 460 B | 19.13 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.78 ms | 213.95 µs | 18.40 ms | 18.40 ms | 50.3 MB | 460 B | 19.63 ms as a whole process |  |
| Seal | Normal | 68.45 ms | 937.36 µs | 67.19 ms | 68.76 ms | 3.0 MB | 3.5 KB | 313.83 µs cold, 334 ns warm; 2.69 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.67 ms | 204.87 µs | 69.42 ms | 70.65 ms | 4.8 MB | 3.0 KB | 5.13 ms as a whole process |  |
| Node `--jitless` | Normal | 181.03 ms | 7.66 ms | 172.42 ms | 203.18 ms | 48.3 MB | 3.1 KB | 18.54 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.88 ms | 27.20 µs | 3.85 ms | 5.73 ms | 52.2 MB | 3.1 KB | 18.81 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.88 ms | 22.63 µs | 3.85 ms | 5.77 ms | 52.1 MB | 3.1 KB | 18.26 ms as a whole process |  |
| Seal | Ceil.source | 55.20 ms | 505.26 µs | 54.80 ms | 55.80 ms | 3.0 MB | 3.5 KB | 291.83 µs cold, 333 ns warm; 2.82 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 56.45 ms | 466.86 µs | 55.56 ms | 55.54 ms | 3.0 MB | 3.5 KB | 276.21 µs cold, 459 ns warm; 2.61 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 48.10 ms | 3.64 ms | 46.07 ms | 47.23 ms | 2.9 MB | 3.5 KB | 262.58 µs cold, 334 ns warm; 2.77 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 48.79 ms | 3.57 ms | 46.77 ms | 47.53 ms | 2.8 MB | 3.5 KB | 256.75 µs cold, 292 ns warm; 2.82 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.60 ms | 169.88 µs | 69.34 ms | 69.88 ms | 4.9 MB | 3.0 KB | 5.17 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.53 ms | 3.09 ms | 175.22 ms | 180.68 ms | 48.3 MB | 3.1 KB | 19.20 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.91 ms | 32.36 µs | 3.86 ms | 5.86 ms | 52.1 MB | 3.1 KB | 20.05 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.88 ms | 24.74 µs | 3.86 ms | 5.75 ms | 52.1 MB | 3.1 KB | 19.04 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 60.88 ms | 699.13 µs | 59.60 ms | 59.77 ms | 2.6 MB | 422 B | 86.21 µs cold, 250 ns warm; 2.84 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.89 ms | 325.86 µs | 45.35 ms | 48.48 ms | 4.9 MB | 366 B | 5.29 ms as a whole process |  |
| Node `--jitless` | Purpose | 413.92 ms | 113.35 ms | 365.47 ms | 383.27 ms | 48.7 MB | 398 B | 19.43 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 35.52 ms | 37.72 ms | 15.71 ms | 19.61 ms | 53.1 MB | 398 B | 19.29 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.94 ms | 291.53 µs | 14.51 ms | 19.81 ms | 55.5 MB | 398 B | 19.62 ms as a whole process |  |
| Seal | Normal | 22.35 ms | 158.19 µs | 22.21 ms | 22.27 ms | 2.9 MB | 1.2 KB | 175.25 µs cold, 333 ns warm; 2.97 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.43 ms | 220.24 µs | 27.15 ms | 27.58 ms | 4.8 MB | 981 B | 5.33 ms as a whole process |  |
| Node `--jitless` | Normal | 64.30 ms | 498.69 µs | 63.38 ms | 64.08 ms | 48.6 MB | 1.0 KB | 19.33 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.62 ms | 201.74 µs | 1.46 ms | 3.40 ms | 53.0 MB | 1.0 KB | 19.68 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.65 ms | 16.83 µs | 1.63 ms | 3.28 ms | 52.9 MB | 1.0 KB | 19.23 ms as a whole process |  |
| Seal | Ceil.source | 21.15 ms | 69.18 µs | 20.99 ms | 21.24 ms | 2.9 MB | 1.2 KB | 160.88 µs cold, 250 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 19.20 ms | 28.87 µs | 19.14 ms | 19.26 ms | 2.9 MB | 1.2 KB | 163.54 µs cold, 292 ns warm; 2.84 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 16.98 ms | 121.57 µs | 16.83 ms | 16.91 ms | 2.7 MB | 1.2 KB | 147.54 µs cold, 208 ns warm; 2.63 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 17.18 ms | 46.26 µs | 17.12 ms | 17.48 ms | 2.7 MB | 1.2 KB | 144.04 µs cold, 250 ns warm; 2.83 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.10 ms | 340.61 µs | 26.53 ms | 28.52 ms | 4.8 MB | 981 B | 5.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.32 ms | 543.66 µs | 62.45 ms | 63.44 ms | 48.6 MB | 1.0 KB | 19.59 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.68 ms | 206.00 µs | 1.51 ms | 3.37 ms | 53.0 MB | 1.0 KB | 19.37 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.66 ms | 19.00 µs | 1.64 ms | 3.28 ms | 52.9 MB | 1.0 KB | 19.62 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 109.78 ms | 751.97 µs | 108.40 ms | 110.35 ms | 2.6 MB | 625 B | 94.12 µs cold, 292 ns warm; 2.81 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 90.87 ms | 1.46 ms | 87.96 ms | 91.28 ms | 4.8 MB | 523 B | 5.17 ms as a whole process |  |
| Node `--jitless` | Purpose | 150.74 ms | 931.24 µs | 148.86 ms | 151.42 ms | 43.5 MB | 568 B | 19.70 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.75 ms | 45.37 µs | 3.66 ms | 4.59 ms | 49.5 MB | 568 B | 19.68 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.77 ms | 52.88 µs | 3.69 ms | 4.71 ms | 49.5 MB | 568 B | 19.70 ms as a whole process |  |
| Seal | Normal | 124.69 ms | 409.61 µs | 124.18 ms | 124.00 ms | 2.8 MB | 1.8 KB | 170.08 µs cold, 375 ns warm; 3.11 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 219.51 ms | 1.95 ms | 214.09 ms | 223.64 ms | 4.8 MB | 1.7 KB | 5.26 ms as a whole process |  |
| Node `--jitless` | Normal | 160.42 ms | 1.51 ms | 158.32 ms | 159.46 ms | 45.5 MB | 1.7 KB | 18.78 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.26 ms | 119.01 µs | 14.08 ms | 15.74 ms | 52.1 MB | 1.7 KB | 19.67 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.39 ms | 178.52 µs | 14.20 ms | 15.95 ms | 52.2 MB | 1.7 KB | 19.40 ms as a whole process |  |
| Seal | Ceil.source | 120.14 ms | 882.84 µs | 118.99 ms | 118.78 ms | 2.8 MB | 1.8 KB | 159.08 µs cold, 250 ns warm; 2.83 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 125.91 ms | 1.45 ms | 123.51 ms | 124.53 ms | 2.8 MB | 1.8 KB | 167.54 µs cold, 334 ns warm; 2.83 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 99.30 ms | 307.73 µs | 98.81 ms | 98.12 ms | 2.7 MB | 1.8 KB | 146.12 µs cold, 292 ns warm; 2.85 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.89 ms | 503.34 µs | 102.22 ms | 103.23 ms | 2.7 MB | 1.8 KB | 155.46 µs cold, 292 ns warm; 2.87 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 219.24 ms | 2.84 ms | 211.62 ms | 221.58 ms | 4.9 MB | 1.7 KB | 5.27 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 161.42 ms | 1.36 ms | 159.39 ms | 164.01 ms | 45.6 MB | 1.7 KB | 19.63 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.27 ms | 84.50 µs | 14.16 ms | 15.66 ms | 52.0 MB | 1.7 KB | 19.34 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.27 ms | 79.85 µs | 14.14 ms | 15.97 ms | 52.2 MB | 1.7 KB | 19.27 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 100.09 ms | 2.11 ms | 98.40 ms | 96.21 ms | 3.8 MB | 564 B | 120.17 µs cold, 375 ns warm; 2.82 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.81 ms | 199.11 µs | 62.48 ms | 63.58 ms | 4.8 MB | 540 B | 5.08 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.12 ms | 2.29 ms | 240.74 ms | 244.57 ms | 48.9 MB | 514 B | 19.28 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 89.79 ms | 25.77 ms | 66.24 ms | 91.97 ms | 52.7 MB | 514 B | 19.19 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 69.97 ms | 963.76 µs | 68.18 ms | 93.22 ms | 52.8 MB | 514 B | 18.74 ms as a whole process |  |
| Seal | Normal | 26.02 ms | 170.18 µs | 25.76 ms | 26.16 ms | 3.7 MB | 510 B | 95.71 µs cold, 459 ns warm; 2.52 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.78 ms | 211.22 µs | 30.42 ms | 30.37 ms | 6.0 MB | 478 B | 5.15 ms as a whole process |  |
| Node `--jitless` | Normal | 52.02 ms | 544.31 µs | 51.53 ms | 53.47 ms | 56.8 MB | 477 B | 19.26 ms as a whole process |  |
| Node (JIT), from cold | Normal | 36.26 ms | 498.13 µs | 35.54 ms | 36.36 ms | 62.7 MB | 477 B | 19.46 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 36.10 ms | 415.09 µs | 35.48 ms | 36.44 ms | 88.7 MB | 477 B | 20.08 ms as a whole process |  |
| Seal | Ceil.source | 26.32 ms | 227.24 µs | 26.02 ms | 26.03 ms | 3.7 MB | 510 B | 99.62 µs cold, 333 ns warm; 2.56 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 24.89 ms | 150.22 µs | 24.70 ms | 24.71 ms | 3.8 MB | 510 B | 104.04 µs cold, 416 ns warm; 2.82 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 22.13 ms | 231.67 µs | 21.83 ms | 22.07 ms | 3.7 MB | 510 B | 91.25 µs cold, 416 ns warm; 2.92 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 22.17 ms | 155.58 µs | 21.87 ms | 22.25 ms | 3.7 MB | 510 B | 104.46 µs cold, 292 ns warm; 2.80 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 31.05 ms | 460.40 µs | 30.56 ms | 30.77 ms | 6.0 MB | 478 B | 5.15 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 53.36 ms | 703.28 µs | 52.37 ms | 54.22 ms | 56.6 MB | 477 B | 19.72 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.72 ms | 412.57 µs | 35.34 ms | 37.25 ms | 62.6 MB | 477 B | 19.67 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.96 ms | 420.81 µs | 35.33 ms | 36.55 ms | 88.4 MB | 477 B | 21.33 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 220.51 µs | 5.76 µs | 214.96 µs | 235.25 µs | 2.9 MB | 227 B | 63.79 µs cold, 417 ns warm; 2.59 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 146.77 ms | 1.07 ms | 145.72 ms | 149.33 ms | 10.6 MB | 195 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Purpose | 379.42 µs | 159.81 µs | 235.96 µs | 1.05 ms | 52.6 MB | 223 B | 19.63 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 226.92 µs | 157.87 µs | 82.88 µs | 925.88 µs | 57.8 MB | 223 B | 19.12 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 172.60 µs | 167.28 µs | 59.54 µs | 906.38 µs | 58.7 MB | 223 B | 19.30 ms as a whole process |  |
| Seal | Normal | 295.39 µs | 10.72 µs | 284.42 µs | 305.38 µs | 3.0 MB | 177 B | 63.50 µs cold, 292 ns warm; 2.60 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 165.11 ms | 2.25 ms | 163.37 ms | 164.92 ms | 12.8 MB | 138 B | 5.25 ms as a whole process |  |
| Node `--jitless` | Normal | 530.85 µs | 355.56 µs | 279.58 µs | 1.14 ms | 61.5 MB | 155 B | 19.25 ms as a whole process |  |
| Node (JIT), from cold | Normal | 496.95 µs | 490.92 µs | 124.29 µs | 1.13 ms | 65.3 MB | 155 B | 18.78 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 297.60 µs | 341.42 µs | 132.12 µs | 1.07 ms | 69.6 MB | 155 B | 19.67 ms as a whole process |  |
| Seal | Ceil.source | 293.90 µs | 6.96 µs | 284.67 µs | 312.67 µs | 3.0 MB | 177 B | 84.42 µs cold, 375 ns warm; 2.53 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 292.50 µs | 8.74 µs | 282.46 µs | 305.58 µs | 3.0 MB | 177 B | 62.33 µs cold, 292 ns warm; 2.63 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 252.59 µs | 7.54 µs | 242.50 µs | 268.42 µs | 2.9 MB | 177 B | 62.04 µs cold, 292 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 258.22 µs | 9.91 µs | 246.54 µs | 273.25 µs | 2.9 MB | 177 B | 56.04 µs cold, 292 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 163.58 ms | 899.49 µs | 162.44 ms | 166.22 ms | 12.6 MB | 138 B | 5.10 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 544.02 µs | 382.88 µs | 272.96 µs | 1.09 ms | 61.5 MB | 155 B | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 519.20 µs | 554.05 µs | 125.46 µs | 1.11 ms | 65.3 MB | 155 B | 19.46 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 279.08 µs | 325.83 µs | 125.12 µs | 1.10 ms | 69.5 MB | 155 B | 19.36 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 77.71 ms | 691.95 µs | 76.65 ms | 80.74 ms | 2.7 MB | 530 B | 94.79 µs cold, 250 ns warm; 2.86 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 95.40 ms | 635.35 µs | 94.08 ms | 95.20 ms | 4.9 MB | 411 B | 5.20 ms as a whole process |  |
| Node `--jitless` | Purpose | 75.87 ms | 664.23 µs | 74.71 ms | 78.96 ms | 59.8 MB | 444 B | 19.34 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.50 ms | 298.90 µs | 10.21 ms | 13.09 ms | 66.0 MB | 444 B | 19.92 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 11.18 ms | 821.38 µs | 10.46 ms | 13.06 ms | 115.0 MB | 444 B | 20.80 ms as a whole process |  |
| Seal | Normal | 117.59 ms | 712.90 µs | 116.04 ms | 141.73 ms | 7.7 MB | 985 B | 149.29 µs cold, 292 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 136.64 ms | 2.33 ms | 132.88 ms | 135.10 ms | 41.9 MB | 799 B | 5.29 ms as a whole process |  |
| Node `--jitless` | Normal | 125.94 ms | 22.52 ms | 115.32 ms | 121.97 ms | 213.5 MB | 878 B | 20.27 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.76 ms | 1.13 ms | 13.49 ms | 17.58 ms | 129.8 MB | 878 B | 19.52 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.46 ms | 105.58 µs | 13.30 ms | 17.58 ms | 224.0 MB | 878 B | 19.28 ms as a whole process |  |
| Seal | Ceil.source | 117.11 ms | 489.53 µs | 116.55 ms | 118.13 ms | 7.7 MB | 985 B | 140.00 µs cold, 292 ns warm; 2.73 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.42 ms | 1.09 ms | 112.75 ms | 115.06 ms | 7.6 MB | 985 B | 151.96 µs cold, 291 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 87.87 ms | 732.54 µs | 86.46 ms | 87.69 ms | 7.6 MB | 985 B | 121.88 µs cold, 208 ns warm; 2.82 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 94.21 ms | 409.73 µs | 93.58 ms | 95.41 ms | 7.5 MB | 985 B | 129.96 µs cold, 250 ns warm; 2.60 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 141.72 ms | 6.48 ms | 132.36 ms | 138.44 ms | 44.6 MB | 799 B | 5.21 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 117.40 ms | 3.51 ms | 112.29 ms | 121.14 ms | 213.5 MB | 878 B | 19.26 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.41 ms | 990.65 µs | 13.40 ms | 17.23 ms | 129.6 MB | 878 B | 20.03 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.48 ms | 113.44 µs | 13.37 ms | 17.42 ms | 224.5 MB | 878 B | 18.67 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.98 µs (±9.42 µs, least 50.17 µs) | 32.79 µs (±4.52 µs, least 28.50 µs) | 17 ns (±0 ns, least 17 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 2.6 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.57 µs (±2.13 µs, least 8.92 µs) | 16.88 µs (±762 ns, least 15.75 µs) | 41 ns (±2 ns, least 39 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| Node `--jitless` | Purpose | 27.78 µs (±7.99 µs, least 20.12 µs) | 38.85 µs (±1.63 µs, least 36.50 µs) | 57 ns (±1 ns, least 56 ns) | 30 ns (±0 ns, least 29 ns) | 0 B | 45.9 MB | the engine's own heap |
| Node (JIT) | Purpose | 36.57 µs (±12.03 µs, least 18.62 µs) | 38.01 µs (±1.58 µs, least 35.83 µs) | 9 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.2 MB | the engine's own heap |
| Seal | Normal | 9.73 µs (±309 ns, least 9.43 µs) | 227 ns (±12 ns, least 213 ns) | 19 ns (±1 ns, least 18 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.96 µs (±238 ns, least 4.65 µs) | 187 ns (±7 ns, least 177 ns) | 34 ns (±3 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.2 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.16 µs (±18 ns, least 1.14 µs) | 673 ns (±18 ns, least 653 ns) | 50 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±30 ns, least 1.06 µs) | 686 ns (±14 ns, least 662 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap |
| Seal | Ceil.source | 9.37 µs (±73 ns, least 9.28 µs) | 224 ns (±10 ns, least 209 ns) | 18 ns (±0 ns, least 18 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 9.81 µs (±327 ns, least 9.54 µs) | 224 ns (±10 ns, least 212 ns) | 18 ns (±1 ns, least 18 ns) | 38 ns (±1 ns, least 38 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 8.48 µs (±209 ns, least 8.19 µs) | 206 ns (±14 ns, least 189 ns) | 15 ns (±0 ns, least 14 ns) | 35 ns (±0 ns, least 35 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 8.93 µs (±278 ns, least 8.62 µs) | 217 ns (±12 ns, least 194 ns) | 15 ns (±1 ns, least 15 ns) | 35 ns (±1 ns, least 35 ns) | 512 B | 4.9 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.00 µs (±155 ns, least 4.72 µs) | 186 ns (±10 ns, least 170 ns) | 34 ns (±2 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.2 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.17 µs (±40 ns, least 1.12 µs) | 682 ns (±22 ns, least 645 ns) | 51 ns (±1 ns, least 50 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±23 ns, least 1.08 µs) | 692 ns (±20 ns, least 657 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.0 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 55.95 ms | 55.85 ms (-0.2%) |
| n-body, 100,000 steps | 68.43 ms | 68.13 ms (-0.4%) |
| spectral-norm, size 200 | 21.45 ms | 20.93 ms (-2.4%) |
| fannkuch-redux, 9 | 120.83 ms | 119.91 ms (-0.8%) |
| word count, 500,000 words | 25.89 ms | 25.76 ms (-0.5%) |
| text building, 20,000 joins | 279.59 µs | 290.56 µs (+3.9%) |
| binary-trees, depth 14 | 117.30 ms | 116.79 ms (-0.4%) |
