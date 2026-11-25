# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.47 2.54 2.46 }` before, `{ 2.00 2.32 2.38 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 50.39 ms | 73.44 ms | **met**: Seal takes 69% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.07 ms | 70.65 ms | **met**: Seal takes 94% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.03 ms | 26.93 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 111.63 ms | 216.26 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 23.89 ms | 30.25 ms | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 348.89 µs | 166.34 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 111.40 ms | 134.63 ms | **met**: Seal takes 83% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.97 ms | 3.56 ms | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 34 ns | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 230 ns | 189 ns | **not met**: Seal takes 122% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 8.91 µs | 4.93 µs | **not met**: Seal takes 181% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 50.39 ms | 95.99 ms | **met**: Seal takes 52% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.07 ms | 180.75 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 23.89 ms | 51.40 ms | **met**: Seal takes 46% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 348.89 µs | 507.96 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 150.23 ms | 3.77 ms | 146.82 ms | 166.14 ms | 2.8 MB | 395 B | 590.25 µs cold, 458 ns warm; 2.73 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 201.23 ms | 14.72 ms | 196.01 ms | 197.59 ms | 4.8 MB | 339 B | 5.28 ms as a whole process |  |
| Node `--jitless` | Purpose | 323.01 ms | 1.05 ms | 321.57 ms | 322.10 ms | 43.3 MB | 362 B | 19.77 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 127.46 ms | 355.99 µs | 126.97 ms | 131.32 ms | 49.5 MB | 362 B | 19.19 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 127.16 ms | 193.77 µs | 126.91 ms | 127.42 ms | 49.1 MB | 362 B | 19.19 ms as a whole process |  |
| Seal | Normal | 50.39 ms | 196.10 µs | 50.07 ms | 50.02 ms | 2.7 MB | 169 B | 85.12 µs cold, 250 ns warm; 2.61 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 73.44 ms | 517.21 µs | 72.57 ms | 71.81 ms | 4.8 MB | 143 B | 5.19 ms as a whole process |  |
| Node `--jitless` | Normal | 95.99 ms | 1.27 ms | 93.51 ms | 96.88 ms | 43.3 MB | 137 B | 19.11 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.57 ms | 130.11 µs | 9.47 ms | 11.04 ms | 48.6 MB | 137 B | 19.41 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.72 ms | 71.90 µs | 9.65 ms | 10.12 ms | 48.5 MB | 137 B | 19.21 ms as a whole process |  |
| Seal | Ceil.source | 50.84 ms | 253.42 µs | 50.57 ms | 50.53 ms | 2.7 MB | 169 B | 89.21 µs cold, 208 ns warm; 2.69 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 52.85 ms | 244.52 µs | 52.56 ms | 76.69 ms | 2.7 MB | 169 B | 927.88 µs cold, 583 ns warm; 2.80 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 47.46 ms | 231.58 µs | 47.12 ms | 71.55 ms | 2.5 MB | 169 B | 741.62 µs cold, 791 ns warm; 2.70 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.34 ms | 190.59 µs | 43.01 ms | 67.31 ms | 2.5 MB | 169 B | 724.29 µs cold, 667 ns warm; 2.67 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.02 ms | 217.73 µs | 72.69 ms | 73.47 ms | 4.8 MB | 143 B | 5.21 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.23 ms | 319.99 µs | 95.79 ms | 93.05 ms | 43.2 MB | 137 B | 19.08 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.76 ms | 331.28 µs | 9.57 ms | 10.13 ms | 48.6 MB | 137 B | 19.47 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.70 ms | 205.39 µs | 9.53 ms | 9.87 ms | 48.7 MB | 137 B | 19.22 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.93 ms | 316.31 µs | 53.48 ms | 54.16 ms | 2.7 MB | 550 B | 117.12 µs cold, 250 ns warm; 2.66 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.16 ms | 386.81 µs | 91.68 ms | 91.12 ms | 4.8 MB | 433 B | 5.25 ms as a whole process |  |
| Node `--jitless` | Purpose | 246.75 ms | 1.29 ms | 244.60 ms | 246.25 ms | 46.5 MB | 460 B | 19.54 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.40 ms | 38.64 µs | 19.32 ms | 19.58 ms | 50.4 MB | 460 B | 19.42 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.84 ms | 523.41 µs | 19.10 ms | 19.38 ms | 50.3 MB | 460 B | 20.13 ms as a whole process |  |
| Seal | Normal | 66.07 ms | 897.24 µs | 64.34 ms | 71.47 ms | 3.3 MB | 3.5 KB | 334.17 µs cold, 334 ns warm; 3.03 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.65 ms | 432.01 µs | 69.97 ms | 70.35 ms | 4.8 MB | 3.0 KB | 5.55 ms as a whole process |  |
| Node `--jitless` | Normal | 180.75 ms | 4.17 ms | 175.20 ms | 181.94 ms | 48.2 MB | 3.1 KB | 19.97 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.93 ms | 47.51 µs | 3.87 ms | 6.12 ms | 52.0 MB | 3.1 KB | 19.80 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.95 ms | 48.79 µs | 3.87 ms | 5.76 ms | 51.9 MB | 3.1 KB | 20.16 ms as a whole process |  |
| Seal | Ceil.source | 55.17 ms | 442.55 µs | 54.19 ms | 57.01 ms | 3.2 MB | 3.4 KB | 295.38 µs cold, 375 ns warm; 2.81 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 62.03 ms | 11.88 ms | 56.08 ms | 57.14 ms | 3.2 MB | 3.4 KB | 298.25 µs cold, 375 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 51.43 ms | 519.51 µs | 50.66 ms | 51.68 ms | 3.1 MB | 3.4 KB | 304.96 µs cold, 333 ns warm; 2.88 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.33 ms | 367.04 µs | 49.84 ms | 51.01 ms | 3.1 MB | 3.4 KB | 309.67 µs cold, 250 ns warm; 2.97 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.66 ms | 190.16 µs | 69.41 ms | 70.05 ms | 4.8 MB | 3.0 KB | 5.06 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.28 ms | 4.24 ms | 171.30 ms | 180.10 ms | 48.2 MB | 3.1 KB | 19.38 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 30.18 µs | 3.85 ms | 5.69 ms | 51.9 MB | 3.1 KB | 19.04 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 20.24 µs | 3.84 ms | 5.70 ms | 52.0 MB | 3.1 KB | 19.08 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.19 ms | 709.54 µs | 52.53 ms | 53.22 ms | 2.8 MB | 429 B | 99.58 µs cold, 250 ns warm; 2.61 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.60 ms | 236.90 µs | 45.36 ms | 45.27 ms | 4.8 MB | 366 B | 5.19 ms as a whole process |  |
| Node `--jitless` | Purpose | 365.30 ms | 5.00 ms | 359.33 ms | 371.27 ms | 48.6 MB | 398 B | 18.77 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.52 ms | 37.14 ms | 15.61 ms | 21.29 ms | 52.7 MB | 398 B | 18.77 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.40 ms | 53.84 µs | 14.29 ms | 19.41 ms | 55.3 MB | 398 B | 18.88 ms as a whole process |  |
| Seal | Normal | 20.03 ms | 60.76 µs | 19.92 ms | 20.07 ms | 3.1 MB | 1.2 KB | 177.21 µs cold, 292 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 26.93 ms | 155.90 µs | 26.70 ms | 27.40 ms | 4.8 MB | 981 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Normal | 64.21 ms | 462.34 µs | 63.71 ms | 64.62 ms | 48.4 MB | 1.0 KB | 18.82 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.62 ms | 188.85 µs | 1.47 ms | 3.56 ms | 52.8 MB | 1.0 KB | 18.68 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.68 ms | 12.61 µs | 1.67 ms | 3.23 ms | 53.0 MB | 1.0 KB | 18.81 ms as a whole process |  |
| Seal | Ceil.source | 20.05 ms | 64.91 µs | 19.96 ms | 20.11 ms | 3.1 MB | 1.2 KB | 180.04 µs cold, 250 ns warm; 2.64 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.18 ms | 23.86 µs | 20.14 ms | 20.25 ms | 3.1 MB | 1.2 KB | 170.04 µs cold, 333 ns warm; 2.72 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.12 ms | 45.04 µs | 19.05 ms | 19.07 ms | 2.9 MB | 1.2 KB | 150.58 µs cold, 250 ns warm; 2.75 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.91 ms | 73.10 µs | 18.79 ms | 19.00 ms | 2.8 MB | 1.2 KB | 153.88 µs cold, 250 ns warm; 2.79 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 28.16 ms | 834.47 µs | 26.52 ms | 27.15 ms | 4.8 MB | 981 B | 5.11 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.42 ms | 365.17 µs | 62.90 ms | 63.91 ms | 48.4 MB | 1.0 KB | 19.28 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.68 ms | 205.85 µs | 1.55 ms | 3.55 ms | 52.9 MB | 1.0 KB | 18.50 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.63 ms | 23.25 µs | 1.61 ms | 3.29 ms | 52.8 MB | 1.0 KB | 18.52 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 96.52 ms | 1.10 ms | 95.21 ms | 95.32 ms | 2.7 MB | 632 B | 103.96 µs cold, 250 ns warm; 2.66 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.52 ms | 1.24 ms | 86.56 ms | 89.62 ms | 4.8 MB | 523 B | 5.13 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.03 ms | 824.05 µs | 144.85 ms | 145.50 ms | 43.4 MB | 568 B | 18.89 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.66 ms | 34.15 µs | 3.63 ms | 4.78 ms | 49.4 MB | 568 B | 18.51 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.65 ms | 26.11 µs | 3.63 ms | 4.45 ms | 49.2 MB | 568 B | 19.34 ms as a whole process |  |
| Seal | Normal | 111.63 ms | 687.64 µs | 110.69 ms | 111.12 ms | 2.9 MB | 1.8 KB | 159.67 µs cold, 291 ns warm; 2.72 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 216.26 ms | 2.77 ms | 209.44 ms | 212.82 ms | 4.8 MB | 1.7 KB | 5.13 ms as a whole process |  |
| Node `--jitless` | Normal | 157.59 ms | 476.97 µs | 156.86 ms | 159.32 ms | 45.4 MB | 1.7 KB | 18.75 ms as a whole process |  |
| Node (JIT), from cold | Normal | 13.95 ms | 86.04 µs | 13.84 ms | 15.75 ms | 51.9 MB | 1.7 KB | 19.15 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.50 ms | 107.36 µs | 14.31 ms | 15.92 ms | 52.1 MB | 1.7 KB | 19.51 ms as a whole process |  |
| Seal | Ceil.source | 113.43 ms | 893.61 µs | 112.36 ms | 112.82 ms | 2.9 MB | 1.8 KB | 159.54 µs cold, 375 ns warm; 2.84 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.60 ms | 1.46 ms | 112.81 ms | 113.56 ms | 3.0 MB | 1.8 KB | 160.17 µs cold, 292 ns warm; 3.08 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 104.70 ms | 565.57 µs | 103.81 ms | 104.57 ms | 2.8 MB | 1.8 KB | 148.21 µs cold, 208 ns warm; 2.93 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.82 ms | 686.70 µs | 101.52 ms | 102.71 ms | 2.8 MB | 1.8 KB | 151.50 µs cold, 291 ns warm; 2.92 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 213.93 ms | 1.88 ms | 209.47 ms | 214.57 ms | 4.8 MB | 1.7 KB | 5.32 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 157.21 ms | 1.44 ms | 155.00 ms | 153.91 ms | 45.4 MB | 1.7 KB | 18.82 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.36 ms | 78.64 µs | 14.24 ms | 15.79 ms | 52.0 MB | 1.7 KB | 19.76 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.77 ms | 116.13 µs | 14.61 ms | 16.35 ms | 52.1 MB | 1.7 KB | 18.97 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 98.08 ms | 1.05 ms | 97.02 ms | 94.89 ms | 4.0 MB | 569 B | 112.21 µs cold, 375 ns warm; 2.87 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.67 ms | 423.14 µs | 62.12 ms | 63.28 ms | 4.8 MB | 540 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Purpose | 239.64 ms | 1.60 ms | 236.83 ms | 242.96 ms | 48.7 MB | 514 B | 19.08 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 90.29 ms | 24.81 ms | 66.70 ms | 93.61 ms | 52.6 MB | 514 B | 19.29 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 68.38 ms | 255.70 µs | 68.02 ms | 89.40 ms | 52.6 MB | 514 B | 19.52 ms as a whole process |  |
| Seal | Normal | 23.89 ms | 143.14 µs | 23.67 ms | 24.01 ms | 4.0 MB | 538 B | 111.17 µs cold, 334 ns warm; 2.75 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.25 ms | 342.03 µs | 29.99 ms | 30.19 ms | 6.0 MB | 478 B | 5.29 ms as a whole process |  |
| Node `--jitless` | Normal | 51.40 ms | 381.48 µs | 50.70 ms | 52.34 ms | 56.6 MB | 477 B | 19.26 ms as a whole process |  |
| Node (JIT), from cold | Normal | 36.89 ms | 1.80 ms | 35.81 ms | 36.26 ms | 62.4 MB | 477 B | 20.12 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.90 ms | 311.00 µs | 35.44 ms | 37.30 ms | 88.3 MB | 477 B | 19.80 ms as a whole process |  |
| Seal | Ceil.source | 24.18 ms | 66.01 µs | 24.01 ms | 24.21 ms | 3.9 MB | 538 B | 115.46 µs cold, 458 ns warm; 2.89 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 24.96 ms | 121.00 µs | 24.77 ms | 25.25 ms | 4.0 MB | 538 B | 114.88 µs cold, 416 ns warm; 2.62 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.13 ms | 210.43 µs | 23.84 ms | 23.88 ms | 3.9 MB | 538 B | 118.92 µs cold, 334 ns warm; 2.82 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.63 ms | 182.89 µs | 23.35 ms | 23.94 ms | 3.9 MB | 538 B | 128.83 µs cold, 292 ns warm; 2.60 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 31.02 ms | 94.05 µs | 30.85 ms | 30.98 ms | 5.9 MB | 478 B | 5.16 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 53.75 ms | 406.72 µs | 53.09 ms | 54.13 ms | 56.5 MB | 477 B | 20.71 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.89 ms | 316.88 µs | 35.27 ms | 36.92 ms | 62.4 MB | 477 B | 20.77 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.80 ms | 490.00 µs | 35.20 ms | 36.60 ms | 88.2 MB | 477 B | 19.24 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 313.29 µs | 6.95 µs | 307.58 µs | 339.08 µs | 3.0 MB | 233 B | 77.21 µs cold, 375 ns warm; 2.67 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 147.96 ms | 2.70 ms | 146.63 ms | 148.58 ms | 10.4 MB | 195 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Purpose | 371.56 µs | 140.62 µs | 248.54 µs | 1.07 ms | 52.4 MB | 223 B | 20.42 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 211.10 µs | 147.89 µs | 88.79 µs | 902.21 µs | 57.7 MB | 223 B | 20.25 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 167.03 µs | 166.19 µs | 55.08 µs | 971.92 µs | 58.5 MB | 223 B | 19.47 ms as a whole process |  |
| Seal | Normal | 348.89 µs | 6.05 µs | 343.79 µs | 368.92 µs | 3.2 MB | 182 B | 81.58 µs cold, 458 ns warm; 2.66 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 166.34 ms | 7.93 ms | 162.32 ms | 165.09 ms | 12.5 MB | 138 B | 5.27 ms as a whole process |  |
| Node `--jitless` | Normal | 507.96 µs | 349.25 µs | 268.04 µs | 1.12 ms | 61.3 MB | 155 B | 18.78 ms as a whole process |  |
| Node (JIT), from cold | Normal | 487.53 µs | 478.44 µs | 123.38 µs | 1.08 ms | 65.2 MB | 155 B | 18.80 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 276.63 µs | 329.00 µs | 122.25 µs | 1.08 ms | 69.3 MB | 155 B | 18.85 ms as a whole process |  |
| Seal | Ceil.source | 344.17 µs | 5.44 µs | 339.33 µs | 363.79 µs | 3.2 MB | 182 B | 80.62 µs cold, 333 ns warm; 2.65 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 347.57 µs | 3.82 µs | 343.33 µs | 368.46 µs | 3.2 MB | 182 B | 84.12 µs cold, 375 ns warm; 2.52 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 273.62 µs | 3.48 µs | 269.83 µs | 297.33 µs | 3.0 MB | 182 B | 69.79 µs cold, 250 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 279.98 µs | 3.75 µs | 275.71 µs | 306.42 µs | 3.0 MB | 182 B | 69.08 µs cold, 291 ns warm; 2.50 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 163.15 ms | 902.71 µs | 162.45 ms | 164.59 ms | 11.9 MB | 138 B | 5.09 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 512.07 µs | 338.83 µs | 274.83 µs | 1.16 ms | 61.5 MB | 155 B | 18.93 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 488.32 µs | 474.89 µs | 133.04 µs | 1.05 ms | 65.2 MB | 155 B | 18.80 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 274.18 µs | 321.50 µs | 123.67 µs | 1.07 ms | 69.4 MB | 155 B | 18.83 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 76.94 ms | 657.55 µs | 75.91 ms | 77.19 ms | 2.9 MB | 537 B | 105.46 µs cold, 292 ns warm; 2.65 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 94.31 ms | 246.15 µs | 93.92 ms | 93.90 ms | 4.8 MB | 411 B | 5.16 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.78 ms | 2.03 ms | 75.97 ms | 80.39 ms | 59.6 MB | 444 B | 19.47 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.54 ms | 276.66 µs | 10.30 ms | 13.18 ms | 65.8 MB | 444 B | 19.93 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.51 ms | 598.51 µs | 10.16 ms | 13.71 ms | 114.7 MB | 444 B | 18.52 ms as a whole process |  |
| Seal | Normal | 111.40 ms | 546.02 µs | 110.39 ms | 112.55 ms | 7.8 MB | 992 B | 145.71 µs cold, 250 ns warm; 2.81 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 134.63 ms | 1.14 ms | 133.28 ms | 130.91 ms | 38.1 MB | 799 B | 5.06 ms as a whole process |  |
| Node `--jitless` | Normal | 114.50 ms | 1.58 ms | 113.02 ms | 117.80 ms | 213.3 MB | 878 B | 19.19 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.15 ms | 880.92 µs | 13.34 ms | 17.14 ms | 129.6 MB | 878 B | 19.10 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.39 ms | 73.68 µs | 13.23 ms | 17.05 ms | 222.8 MB | 878 B | 18.81 ms as a whole process |  |
| Seal | Ceil.source | 112.76 ms | 1.32 ms | 110.85 ms | 114.52 ms | 7.8 MB | 992 B | 147.54 µs cold, 292 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 113.64 ms | 627.53 µs | 112.55 ms | 114.57 ms | 7.9 MB | 992 B | 143.71 µs cold, 333 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 92.21 ms | 411.39 µs | 91.61 ms | 93.74 ms | 7.6 MB | 992 B | 124.83 µs cold, 209 ns warm; 2.71 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 90.46 ms | 679.31 µs | 89.70 ms | 92.38 ms | 7.7 MB | 992 B | 127.58 µs cold, 250 ns warm; 2.77 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 130.93 ms | 1.13 ms | 129.18 ms | 132.45 ms | 44.3 MB | 799 B | 5.25 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 111.82 ms | 2.15 ms | 110.18 ms | 116.54 ms | 212.9 MB | 878 B | 18.62 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.21 ms | 850.31 µs | 13.47 ms | 16.99 ms | 129.5 MB | 878 B | 18.91 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.19 ms | 100.27 µs | 12.95 ms | 16.92 ms | 223.0 MB | 878 B | 19.05 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 64.45 µs (±8.39 µs, least 57.46 µs) | 32.63 µs (±4.58 µs, least 26.46 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 2.8 MB | counted by Seal |
| Luau (interpreter) | Purpose | 11.33 µs (±3.57 µs, least 9.38 µs) | 16.89 µs (±882 ns, least 15.12 µs) | 41 ns (±1 ns, least 39 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 32.21 µs (±7.45 µs, least 20.08 µs) | 65.44 µs (±86.66 µs, least 36.04 µs) | 57 ns (±1 ns, least 55 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 29.85 µs (±7.39 µs, least 19.12 µs) | 41.85 µs (±9.83 µs, least 35.46 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 8.91 µs (±161 ns, least 8.66 µs) | 230 ns (±8 ns, least 218 ns) | 17 ns (±1 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.1 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.93 µs (±219 ns, least 4.65 µs) | 189 ns (±10 ns, least 172 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±21 ns, least 1.12 µs) | 669 ns (±8 ns, least 654 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.12 µs (±18 ns, least 1.09 µs) | 673 ns (±15 ns, least 656 ns) | 22 ns (±0 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 9.13 µs (±120 ns, least 8.94 µs) | 215 ns (±7 ns, least 206 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 9.08 µs (±266 ns, least 8.89 µs) | 227 ns (±10 ns, least 214 ns) | 19 ns (±0 ns, least 18 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 5.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 8.23 µs (±391 ns, least 7.94 µs) | 198 ns (±10 ns, least 186 ns) | 15 ns (±0 ns, least 14 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 5.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 8.26 µs (±234 ns, least 8.04 µs) | 199 ns (±8 ns, least 185 ns) | 15 ns (±0 ns, least 14 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 5.1 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.90 µs (±161 ns, least 4.73 µs) | 191 ns (±10 ns, least 176 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±20 ns, least 1.13 µs) | 681 ns (±19 ns, least 651 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.12 µs (±20 ns, least 1.08 µs) | 685 ns (±24 ns, least 642 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 50.65 ms | 50.38 ms (-0.5%) |
| n-body, 100,000 steps | 68.03 ms | 61.84 ms (-9.1%) |
| spectral-norm, size 200 | 20.07 ms | 19.33 ms (-3.7%) |
| fannkuch-redux, 9 | 112.41 ms | 111.42 ms (-0.9%) |
| word count, 500,000 words | 23.97 ms | 24.15 ms (+0.8%) |
| text building, 20,000 joins | 347.48 µs | 346.90 µs (-0.2%) |
| binary-trees, depth 14 | 112.55 ms | 111.07 ms (-1.3%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 227.28 ms | 1.08 ms | 226.26 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.90 ms | 110.17 µs | 10.71 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.46 ms | 35.46 µs | 4.42 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.41 ms | 20.25 µs | 1.40 ms | 3.2 MB |
