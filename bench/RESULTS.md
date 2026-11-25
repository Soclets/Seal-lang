# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.99 2.50 2.11 }` before, `{ 1.99 2.35 2.11 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 55.71 ms | 72.83 ms | **met**: Seal takes 76% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.96 ms | 69.64 ms | **met**: Seal takes 95% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.26 ms | 27.47 ms | **met**: Seal takes 70% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 115.03 ms | 209.81 ms | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 25.00 ms | 30.64 ms | **met**: Seal takes 82% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 333.40 µs | 163.29 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 115.30 ms | 144.38 ms | **met**: Seal takes 80% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.04 ms | 3.56 ms | **met**: Seal takes 57% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 18 ns | 34 ns | **met**: Seal takes 54% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 221 ns | 183 ns | **not met**: Seal takes 121% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 10.35 µs | 4.89 µs | **not met**: Seal takes 212% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 55.71 ms | 97.41 ms | **met**: Seal takes 57% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.96 ms | 175.08 ms | **met**: Seal takes 38% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 25.00 ms | 51.90 ms | **met**: Seal takes 48% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 333.40 µs | 504.19 µs | **met**: Seal takes 66% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 18 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 146.36 ms | 688.19 µs | 145.70 ms | 166.63 ms | 2.7 MB | 395 B | 632.58 µs cold, 542 ns warm; 3.48 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 199.54 ms | 3.58 ms | 196.77 ms | 219.42 ms | 4.8 MB | 339 B | 5.00 ms as a whole process |  |
| Node `--jitless` | Purpose | 328.70 ms | 9.18 ms | 321.10 ms | 322.26 ms | 43.3 MB | 362 B | 18.78 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 129.37 ms | 5.50 ms | 125.79 ms | 129.85 ms | 49.5 MB | 362 B | 19.27 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 125.76 ms | 588.03 µs | 125.01 ms | 126.31 ms | 49.2 MB | 362 B | 19.45 ms as a whole process |  |
| Seal | Normal | 55.71 ms | 5.88 ms | 52.74 ms | 52.80 ms | 2.5 MB | 169 B | 76.62 µs cold, 250 ns warm; 2.76 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 72.83 ms | 270.54 µs | 72.49 ms | 73.30 ms | 4.8 MB | 143 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Normal | 97.41 ms | 4.88 ms | 94.24 ms | 96.14 ms | 43.5 MB | 137 B | 19.19 ms as a whole process |  |
| Node (JIT), from cold | Normal | 10.62 ms | 375.56 µs | 10.29 ms | 11.37 ms | 48.7 MB | 137 B | 18.64 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.05 ms | 234.37 µs | 9.72 ms | 9.97 ms | 48.6 MB | 137 B | 19.56 ms as a whole process |  |
| Seal | Ceil.source | 52.55 ms | 121.26 µs | 52.37 ms | 52.36 ms | 2.5 MB | 169 B | 77.08 µs cold, 208 ns warm; 2.69 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 56.34 ms | 114.01 µs | 56.16 ms | 83.80 ms | 2.5 MB | 169 B | 913.29 µs cold, 708 ns warm; 2.67 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 52.93 ms | 771.47 µs | 51.52 ms | 75.94 ms | 2.4 MB | 169 B | 538.46 µs cold, 583 ns warm; 2.76 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 51.09 ms | 1.65 ms | 48.58 ms | 76.99 ms | 2.4 MB | 169 B | 805.12 µs cold, 625 ns warm; 2.71 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.70 ms | 740.34 µs | 70.76 ms | 71.97 ms | 4.8 MB | 143 B | 5.13 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.98 ms | 1.17 ms | 94.09 ms | 94.96 ms | 43.4 MB | 137 B | 19.36 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.52 ms | 50.91 µs | 9.45 ms | 10.72 ms | 48.5 MB | 137 B | 19.42 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.87 ms | 66.68 µs | 9.78 ms | 9.90 ms | 48.6 MB | 137 B | 18.76 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 56.09 ms | 257.93 µs | 55.58 ms | 56.34 ms | 2.6 MB | 550 B | 163.62 µs cold, 375 ns warm; 2.69 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.46 ms | 577.54 µs | 91.48 ms | 91.34 ms | 4.8 MB | 433 B | 5.13 ms as a whole process |  |
| Node `--jitless` | Purpose | 247.14 ms | 6.19 ms | 243.28 ms | 245.96 ms | 46.4 MB | 460 B | 19.54 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.06 ms | 107.34 µs | 18.84 ms | 19.21 ms | 50.4 MB | 460 B | 19.01 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.09 ms | 86.41 µs | 18.98 ms | 19.15 ms | 50.3 MB | 460 B | 19.30 ms as a whole process |  |
| Seal | Normal | 65.96 ms | 1.28 ms | 63.64 ms | 66.85 ms | 3.1 MB | 3.5 KB | 327.54 µs cold, 375 ns warm; 3.05 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.64 ms | 247.82 µs | 69.27 ms | 70.07 ms | 4.8 MB | 3.0 KB | 5.21 ms as a whole process |  |
| Node `--jitless` | Normal | 175.08 ms | 3.85 ms | 170.87 ms | 179.69 ms | 48.2 MB | 3.1 KB | 19.39 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.88 ms | 47.90 µs | 3.83 ms | 6.01 ms | 52.0 MB | 3.1 KB | 19.14 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.86 ms | 20.66 µs | 3.83 ms | 5.73 ms | 52.0 MB | 3.1 KB | 19.17 ms as a whole process |  |
| Seal | Ceil.source | 54.12 ms | 634.15 µs | 52.86 ms | 55.88 ms | 3.1 MB | 3.5 KB | 325.79 µs cold, 334 ns warm; 3.04 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.53 ms | 1.22 ms | 54.15 ms | 57.29 ms | 3.1 MB | 3.5 KB | 298.38 µs cold, 375 ns warm; 2.84 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 46.61 ms | 353.44 µs | 46.16 ms | 45.62 ms | 2.9 MB | 3.5 KB | 291.71 µs cold, 250 ns warm; 2.80 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 47.58 ms | 392.22 µs | 46.93 ms | 47.49 ms | 2.9 MB | 3.5 KB | 271.33 µs cold, 333 ns warm; 2.87 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.74 ms | 418.81 µs | 69.27 ms | 71.65 ms | 4.8 MB | 3.0 KB | 5.25 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.32 ms | 3.87 ms | 173.28 ms | 182.23 ms | 48.1 MB | 3.1 KB | 19.97 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.89 ms | 26.76 µs | 3.85 ms | 5.75 ms | 52.0 MB | 3.1 KB | 19.22 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 24.35 µs | 3.83 ms | 5.66 ms | 51.8 MB | 3.1 KB | 19.37 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 56.74 ms | 955.81 µs | 55.12 ms | 49.56 ms | 2.7 MB | 429 B | 136.50 µs cold, 292 ns warm; 2.65 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.92 ms | 222.78 µs | 45.62 ms | 45.84 ms | 4.8 MB | 366 B | 4.93 ms as a whole process |  |
| Node `--jitless` | Purpose | 371.59 ms | 6.72 ms | 362.52 ms | 367.22 ms | 48.6 MB | 398 B | 19.60 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.66 ms | 36.89 ms | 15.75 ms | 22.26 ms | 52.9 MB | 398 B | 18.86 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.33 ms | 59.69 µs | 14.20 ms | 20.28 ms | 55.4 MB | 398 B | 19.34 ms as a whole process |  |
| Seal | Normal | 19.26 ms | 92.14 µs | 19.14 ms | 19.56 ms | 2.9 MB | 1.2 KB | 176.21 µs cold, 375 ns warm; 2.66 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.47 ms | 506.36 µs | 26.78 ms | 28.84 ms | 4.8 MB | 981 B | 5.05 ms as a whole process |  |
| Node `--jitless` | Normal | 65.05 ms | 1.38 ms | 63.13 ms | 62.32 ms | 48.5 MB | 1.0 KB | 19.60 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.74 ms | 245.42 µs | 1.58 ms | 3.61 ms | 52.9 MB | 1.0 KB | 20.27 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.88 ms | 276.53 µs | 1.67 ms | 3.54 ms | 52.9 MB | 1.0 KB | 20.82 ms as a whole process |  |
| Seal | Ceil.source | 19.87 ms | 794.81 µs | 19.32 ms | 19.63 ms | 2.9 MB | 1.2 KB | 178.42 µs cold, 292 ns warm; 2.68 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.87 ms | 298.36 µs | 20.61 ms | 20.56 ms | 2.9 MB | 1.2 KB | 168.21 µs cold, 250 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 16.24 ms | 75.55 µs | 16.14 ms | 16.20 ms | 2.7 MB | 1.2 KB | 155.21 µs cold, 209 ns warm; 2.83 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 17.40 ms | 538.91 µs | 16.77 ms | 18.25 ms | 2.8 MB | 1.2 KB | 154.67 µs cold, 250 ns warm; 2.89 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.20 ms | 502.62 µs | 26.36 ms | 27.30 ms | 4.8 MB | 981 B | 5.20 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.75 ms | 911.63 µs | 63.43 ms | 76.10 ms | 48.4 MB | 1.0 KB | 18.69 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.68 ms | 213.11 µs | 1.55 ms | 3.52 ms | 52.9 MB | 1.0 KB | 19.87 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.68 ms | 16.77 µs | 1.67 ms | 3.62 ms | 52.9 MB | 1.0 KB | 19.10 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 107.40 ms | 1.84 ms | 105.51 ms | 106.80 ms | 2.6 MB | 632 B | 157.67 µs cold, 292 ns warm; 2.75 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 91.05 ms | 1.85 ms | 88.52 ms | 90.55 ms | 4.8 MB | 523 B | 5.30 ms as a whole process |  |
| Node `--jitless` | Purpose | 149.61 ms | 1.70 ms | 147.03 ms | 149.63 ms | 43.2 MB | 568 B | 19.39 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.69 ms | 27.60 µs | 3.64 ms | 4.83 ms | 49.3 MB | 568 B | 19.21 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.70 ms | 35.80 µs | 3.62 ms | 4.52 ms | 49.4 MB | 568 B | 19.94 ms as a whole process |  |
| Seal | Normal | 115.03 ms | 463.20 µs | 114.44 ms | 115.50 ms | 2.9 MB | 1.8 KB | 175.38 µs cold, 250 ns warm; 3.07 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 209.81 ms | 2.35 ms | 203.33 ms | 212.26 ms | 4.8 MB | 1.7 KB | 5.38 ms as a whole process |  |
| Node `--jitless` | Normal | 156.48 ms | 627.68 µs | 155.56 ms | 155.27 ms | 45.4 MB | 1.7 KB | 19.38 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.65 ms | 114.37 µs | 14.52 ms | 16.42 ms | 52.0 MB | 1.7 KB | 19.27 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.53 ms | 80.77 µs | 14.40 ms | 16.01 ms | 52.1 MB | 1.7 KB | 19.66 ms as a whole process |  |
| Seal | Ceil.source | 115.49 ms | 843.12 µs | 114.61 ms | 114.37 ms | 2.9 MB | 1.8 KB | 167.29 µs cold, 250 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 119.31 ms | 209.31 µs | 119.06 ms | 121.51 ms | 2.9 MB | 1.8 KB | 158.62 µs cold, 292 ns warm; 2.84 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 100.38 ms | 277.35 µs | 99.97 ms | 100.29 ms | 2.7 MB | 1.8 KB | 149.12 µs cold, 209 ns warm; 2.63 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.93 ms | 329.66 µs | 102.48 ms | 103.32 ms | 2.8 MB | 1.8 KB | 148.58 µs cold, 250 ns warm; 2.69 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 218.75 ms | 12.01 ms | 208.42 ms | 210.17 ms | 4.8 MB | 1.7 KB | 5.57 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 163.76 ms | 6.72 ms | 154.60 ms | 162.22 ms | 45.4 MB | 1.7 KB | 19.44 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 15.00 ms | 209.33 µs | 14.69 ms | 16.61 ms | 51.9 MB | 1.7 KB | 19.25 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.13 ms | 110.11 µs | 13.89 ms | 15.64 ms | 52.2 MB | 1.7 KB | 19.94 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 98.53 ms | 3.74 ms | 95.51 ms | 96.22 ms | 3.8 MB | 569 B | 122.87 µs cold, 458 ns warm; 2.88 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.10 ms | 470.68 µs | 62.55 ms | 63.33 ms | 4.8 MB | 540 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.44 ms | 4.00 ms | 239.61 ms | 244.39 ms | 48.6 MB | 514 B | 19.61 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 89.20 ms | 25.59 ms | 66.28 ms | 92.41 ms | 52.5 MB | 514 B | 19.96 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.98 ms | 1.08 ms | 66.03 ms | 94.88 ms | 52.7 MB | 514 B | 19.29 ms as a whole process |  |
| Seal | Normal | 25.00 ms | 192.31 µs | 24.70 ms | 24.99 ms | 3.8 MB | 538 B | 104.92 µs cold, 333 ns warm; 2.68 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.64 ms | 190.68 µs | 30.28 ms | 30.57 ms | 6.0 MB | 478 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Normal | 51.90 ms | 207.09 µs | 51.60 ms | 53.72 ms | 56.3 MB | 477 B | 19.01 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.03 ms | 229.40 µs | 34.58 ms | 35.74 ms | 62.4 MB | 477 B | 19.65 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 38.85 ms | 6.40 ms | 36.29 ms | 36.49 ms | 88.3 MB | 477 B | 21.20 ms as a whole process |  |
| Seal | Ceil.source | 25.20 ms | 187.16 µs | 24.82 ms | 25.77 ms | 3.8 MB | 538 B | 113.88 µs cold, 334 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.72 ms | 144.31 µs | 25.59 ms | 25.85 ms | 3.8 MB | 538 B | 103.96 µs cold, 333 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.04 ms | 299.85 µs | 23.54 ms | 23.90 ms | 3.7 MB | 538 B | 99.29 µs cold, 375 ns warm; 2.55 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.32 ms | 100.91 µs | 24.17 ms | 24.92 ms | 3.8 MB | 538 B | 106.79 µs cold, 333 ns warm; 2.75 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.63 ms | 105.75 µs | 30.43 ms | 30.10 ms | 5.9 MB | 478 B | 5.16 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 56.72 ms | 7.06 ms | 51.46 ms | 52.78 ms | 56.6 MB | 477 B | 20.90 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.74 ms | 623.48 µs | 34.88 ms | 38.48 ms | 62.5 MB | 477 B | 19.46 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.78 ms | 339.13 µs | 35.26 ms | 36.76 ms | 88.0 MB | 477 B | 20.26 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 296.55 µs | 9.49 µs | 287.54 µs | 343.58 µs | 2.9 MB | 233 B | 82.33 µs cold, 291 ns warm; 2.71 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 145.85 ms | 751.75 µs | 144.78 ms | 146.83 ms | 10.5 MB | 195 B | 5.13 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.30 µs | 142.09 µs | 245.25 µs | 1.04 ms | 52.6 MB | 223 B | 19.66 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 213.46 µs | 151.21 µs | 81.21 µs | 924.96 µs | 57.8 MB | 223 B | 19.02 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 135.43 µs | 121.93 µs | 56.04 µs | 947.92 µs | 57.8 MB | 223 B | 19.31 ms as a whole process |  |
| Seal | Normal | 333.40 µs | 11.09 µs | 324.04 µs | 352.92 µs | 3.1 MB | 182 B | 73.96 µs cold, 334 ns warm; 2.72 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 163.29 ms | 629.35 µs | 162.37 ms | 164.36 ms | 11.9 MB | 138 B | 5.07 ms as a whole process |  |
| Node `--jitless` | Normal | 504.19 µs | 343.64 µs | 270.67 µs | 1.16 ms | 61.4 MB | 155 B | 19.20 ms as a whole process |  |
| Node (JIT), from cold | Normal | 496.33 µs | 468.27 µs | 130.75 µs | 1.09 ms | 65.1 MB | 155 B | 19.55 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 297.62 µs | 337.65 µs | 141.83 µs | 1.08 ms | 69.3 MB | 155 B | 20.67 ms as a whole process |  |
| Seal | Ceil.source | 349.51 µs | 14.63 µs | 337.92 µs | 380.46 µs | 3.1 MB | 182 B | 76.92 µs cold, 375 ns warm; 2.73 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 343.47 µs | 7.33 µs | 332.33 µs | 381.54 µs | 3.0 MB | 182 B | 76.71 µs cold, 417 ns warm; 2.62 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 283.92 µs | 34.24 µs | 265.96 µs | 295.12 µs | 2.9 MB | 182 B | 63.92 µs cold, 250 ns warm; 2.73 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 274.21 µs | 4.47 µs | 268.25 µs | 304.25 µs | 3.0 MB | 182 B | 65.25 µs cold, 375 ns warm; 2.84 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 164.30 ms | 816.31 µs | 163.04 ms | 169.15 ms | 12.6 MB | 138 B | 5.17 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 523.56 µs | 325.86 µs | 280.50 µs | 1.19 ms | 61.4 MB | 155 B | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 492.51 µs | 471.15 µs | 133.79 µs | 1.10 ms | 65.2 MB | 155 B | 18.81 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 273.76 µs | 320.13 µs | 122.79 µs | 1.08 ms | 69.4 MB | 155 B | 18.87 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 77.65 ms | 587.37 µs | 76.75 ms | 79.72 ms | 2.8 MB | 537 B | 136.17 µs cold, 250 ns warm; 2.72 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 96.29 ms | 615.98 µs | 95.60 ms | 95.33 ms | 4.9 MB | 411 B | 5.20 ms as a whole process |  |
| Node `--jitless` | Purpose | 79.32 ms | 1.43 ms | 76.62 ms | 80.58 ms | 59.6 MB | 444 B | 20.67 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 12.75 ms | 4.39 ms | 10.49 ms | 13.51 ms | 65.8 MB | 444 B | 20.87 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.50 ms | 789.99 µs | 10.10 ms | 13.34 ms | 114.8 MB | 444 B | 19.41 ms as a whole process |  |
| Seal | Normal | 115.30 ms | 6.35 ms | 111.77 ms | 114.81 ms | 7.7 MB | 992 B | 158.67 µs cold, 333 ns warm; 2.76 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 144.38 ms | 9.33 ms | 134.72 ms | 131.14 ms | 41.4 MB | 799 B | 5.31 ms as a whole process |  |
| Node `--jitless` | Normal | 116.46 ms | 5.06 ms | 111.97 ms | 117.41 ms | 213.3 MB | 878 B | 19.50 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.26 ms | 970.46 µs | 13.34 ms | 17.12 ms | 129.6 MB | 878 B | 18.91 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.29 ms | 131.97 µs | 13.18 ms | 17.20 ms | 222.9 MB | 878 B | 19.99 ms as a whole process |  |
| Seal | Ceil.source | 114.40 ms | 1.48 ms | 112.81 ms | 115.26 ms | 7.7 MB | 992 B | 150.79 µs cold, 292 ns warm; 2.80 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.10 ms | 749.32 µs | 112.89 ms | 114.78 ms | 7.7 MB | 992 B | 139.29 µs cold, 250 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 93.24 ms | 523.70 µs | 92.38 ms | 93.12 ms | 7.6 MB | 992 B | 128.46 µs cold, 125 ns warm; 2.58 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 99.00 ms | 640.46 µs | 97.89 ms | 100.05 ms | 7.6 MB | 992 B | 149.25 µs cold, 208 ns warm; 2.76 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 131.15 ms | 2.94 ms | 128.07 ms | 131.17 ms | 40.5 MB | 799 B | 5.17 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 111.57 ms | 1.74 ms | 110.27 ms | 114.13 ms | 213.3 MB | 878 B | 19.46 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.31 ms | 885.26 µs | 13.58 ms | 17.18 ms | 129.7 MB | 878 B | 18.96 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.39 ms | 59.34 µs | 13.32 ms | 17.30 ms | 223.2 MB | 878 B | 19.94 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 61.65 µs (±6.59 µs, least 56.12 µs) | 32.75 µs (±4.46 µs, least 28.42 µs) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 11.06 µs (±2.81 µs, least 9.17 µs) | 18.64 µs (±6.00 µs, least 16.33 µs) | 42 ns (±1 ns, least 39 ns) | 22 ns (±5 ns, least 17 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| Node `--jitless` | Purpose | 36.35 µs (±15.07 µs, least 19.21 µs) | 68.14 µs (±74.40 µs, least 36.50 µs) | 58 ns (±2 ns, least 55 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 46.0 MB | the engine's own heap |
| Node (JIT) | Purpose | 34.08 µs (±4.20 µs, least 22.38 µs) | 38.92 µs (±4.14 µs, least 35.79 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 10.35 µs (±121 ns, least 10.19 µs) | 221 ns (±10 ns, least 211 ns) | 18 ns (±0 ns, least 18 ns) | 37 ns (±1 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.89 µs (±152 ns, least 4.66 µs) | 183 ns (±9 ns, least 172 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.14 µs (±28 ns, least 1.10 µs) | 670 ns (±23 ns, least 638 ns) | 50 ns (±2 ns, least 49 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.12 µs (±35 ns, least 1.08 µs) | 670 ns (±22 ns, least 648 ns) | 22 ns (±0 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 10.85 µs (±374 ns, least 10.47 µs) | 221 ns (±15 ns, least 207 ns) | 18 ns (±0 ns, least 18 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 10.71 µs (±229 ns, least 10.51 µs) | 221 ns (±9 ns, least 207 ns) | 18 ns (±1 ns, least 18 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 9.53 µs (±274 ns, least 9.28 µs) | 204 ns (±8 ns, least 195 ns) | 14 ns (±1 ns, least 14 ns) | 36 ns (±4 ns, least 35 ns) | 512 B | 4.9 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 9.91 µs (±430 ns, least 9.51 µs) | 205 ns (±11 ns, least 194 ns) | 15 ns (±1 ns, least 14 ns) | 35 ns (±0 ns, least 35 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.09 µs (±165 ns, least 4.79 µs) | 195 ns (±13 ns, least 179 ns) | 33 ns (±2 ns, least 32 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.17 µs (±34 ns, least 1.12 µs) | 680 ns (±24 ns, least 654 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±41 ns, least 1.05 µs) | 658 ns (±12 ns, least 637 ns) | 22 ns (±0 ns, least 21 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 52.76 ms | 52.81 ms (+0.1%) |
| n-body, 100,000 steps | 64.89 ms | 61.92 ms (-4.6%) |
| spectral-norm, size 200 | 19.30 ms | 20.16 ms (+4.5%) |
| fannkuch-redux, 9 | 117.29 ms | 115.47 ms (-1.6%) |
| word count, 500,000 words | 24.33 ms | 24.28 ms (-0.2%) |
| text building, 20,000 joins | 330.51 µs | 338.83 µs (+2.5%) |
| binary-trees, depth 14 | 114.42 ms | 113.31 ms (-1.0%) |
