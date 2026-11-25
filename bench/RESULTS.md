# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.81 2.51 2.47 }` before, `{ 2.09 2.33 2.40 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 54.01 ms | 72.14 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 67.51 ms | 69.97 ms | **met**: Seal takes 96% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.07 ms | 27.84 ms | **met**: Seal takes 72% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 113.74 ms | 210.71 ms | **met**: Seal takes 54% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 24.41 ms | 30.89 ms | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 349.70 µs | 164.43 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 114.81 ms | 133.60 ms | **met**: Seal takes 86% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.99 ms | 3.58 ms | **met**: Seal takes 56% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 18 ns | 34 ns | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 231 ns | 190 ns | **not met**: Seal takes 121% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 12.60 µs | 4.99 µs | **not met**: Seal takes 253% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 54.01 ms | 95.81 ms | **met**: Seal takes 56% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 67.51 ms | 195.43 ms | **met**: Seal takes 35% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 24.41 ms | 52.55 ms | **met**: Seal takes 46% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 349.70 µs | 521.64 µs | **met**: Seal takes 67% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 18 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 167.15 ms | 1.64 ms | 165.28 ms | 180.45 ms | 2.8 MB | 395 B | 720.83 µs cold, 458 ns warm; 2.73 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 199.40 ms | 956.24 µs | 197.89 ms | 199.82 ms | 4.8 MB | 339 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Purpose | 328.50 ms | 6.17 ms | 323.77 ms | 329.22 ms | 43.5 MB | 362 B | 20.62 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 126.68 ms | 287.60 µs | 126.47 ms | 126.93 ms | 49.1 MB | 362 B | 19.62 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 127.57 ms | 700.08 µs | 126.51 ms | 127.18 ms | 49.0 MB | 362 B | 21.55 ms as a whole process |  |
| Seal | Normal | 54.01 ms | 408.95 µs | 53.40 ms | 53.26 ms | 2.7 MB | 169 B | 99.62 µs cold, 250 ns warm; 2.81 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 72.14 ms | 768.02 µs | 71.15 ms | 72.43 ms | 4.9 MB | 143 B | 5.32 ms as a whole process |  |
| Node `--jitless` | Normal | 95.81 ms | 457.15 µs | 95.26 ms | 98.48 ms | 43.4 MB | 137 B | 20.39 ms as a whole process |  |
| Node (JIT), from cold | Normal | 10.14 ms | 663.40 µs | 9.64 ms | 9.99 ms | 48.6 MB | 137 B | 20.35 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.63 ms | 73.40 µs | 9.54 ms | 11.01 ms | 48.6 MB | 137 B | 19.93 ms as a whole process |  |
| Seal | Ceil.source | 53.66 ms | 332.66 µs | 53.18 ms | 53.90 ms | 2.7 MB | 169 B | 94.58 µs cold, 333 ns warm; 2.80 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 53.95 ms | 229.10 µs | 53.62 ms | 78.18 ms | 2.8 MB | 169 B | 1.01 ms cold, 667 ns warm; 2.82 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 49.58 ms | 332.04 µs | 49.13 ms | 73.73 ms | 2.5 MB | 169 B | 861.25 µs cold, 667 ns warm; 3.08 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.69 ms | 184.28 µs | 43.44 ms | 68.21 ms | 2.6 MB | 169 B | 886.04 µs cold, 625 ns warm; 2.84 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 71.04 ms | 328.00 µs | 70.69 ms | 71.52 ms | 4.8 MB | 143 B | 5.11 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.40 ms | 893.56 µs | 94.38 ms | 106.49 ms | 43.4 MB | 137 B | 20.22 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.90 ms | 280.51 µs | 9.66 ms | 10.87 ms | 48.6 MB | 137 B | 20.65 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.92 ms | 294.40 µs | 9.46 ms | 10.06 ms | 48.7 MB | 137 B | 19.92 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.65 ms | 375.31 µs | 52.99 ms | 53.36 ms | 2.7 MB | 550 B | 132.21 µs cold, 250 ns warm; 2.76 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 93.43 ms | 1.27 ms | 92.27 ms | 91.86 ms | 4.8 MB | 433 B | 5.32 ms as a whole process |  |
| Node `--jitless` | Purpose | 252.81 ms | 12.29 ms | 244.44 ms | 258.40 ms | 46.4 MB | 460 B | 21.37 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.28 ms | 91.12 µs | 19.17 ms | 19.33 ms | 50.3 MB | 460 B | 19.68 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.19 ms | 274.82 µs | 18.99 ms | 19.26 ms | 50.2 MB | 460 B | 21.36 ms as a whole process |  |
| Seal | Normal | 67.51 ms | 951.66 µs | 65.47 ms | 68.90 ms | 3.4 MB | 3.5 KB | 394.00 µs cold, 417 ns warm; 3.22 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.97 ms | 796.17 µs | 69.25 ms | 70.99 ms | 4.8 MB | 3.0 KB | 5.40 ms as a whole process |  |
| Node `--jitless` | Normal | 195.43 ms | 32.02 ms | 172.99 ms | 181.15 ms | 48.1 MB | 3.1 KB | 20.40 ms as a whole process |  |
| Node (JIT), from cold | Normal | 4.13 ms | 567.80 µs | 3.91 ms | 5.72 ms | 52.0 MB | 3.1 KB | 20.19 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.90 ms | 21.86 µs | 3.86 ms | 5.71 ms | 51.8 MB | 3.1 KB | 21.08 ms as a whole process |  |
| Seal | Ceil.source | 56.78 ms | 668.62 µs | 55.92 ms | 57.48 ms | 3.3 MB | 3.4 KB | 316.04 µs cold, 375 ns warm; 3.02 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.70 ms | 418.67 µs | 55.08 ms | 56.94 ms | 3.4 MB | 3.4 KB | 342.58 µs cold, 292 ns warm; 3.06 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.66 ms | 345.44 µs | 49.94 ms | 50.19 ms | 3.2 MB | 3.4 KB | 338.54 µs cold, 292 ns warm; 2.79 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 51.47 ms | 266.35 µs | 50.93 ms | 52.09 ms | 3.1 MB | 3.4 KB | 332.58 µs cold, 292 ns warm; 3.10 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 70.67 ms | 2.81 ms | 69.53 ms | 70.63 ms | 4.8 MB | 3.0 KB | 5.07 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.76 ms | 4.18 ms | 171.02 ms | 179.72 ms | 48.2 MB | 3.1 KB | 20.54 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.98 ms | 131.63 µs | 3.87 ms | 5.76 ms | 51.9 MB | 3.1 KB | 19.99 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.89 ms | 22.90 µs | 3.86 ms | 5.71 ms | 51.9 MB | 3.1 KB | 20.30 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.50 ms | 1.21 ms | 55.46 ms | 58.98 ms | 2.8 MB | 429 B | 107.17 µs cold, 292 ns warm; 2.51 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.70 ms | 269.67 µs | 45.28 ms | 45.38 ms | 4.8 MB | 366 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Purpose | 366.05 ms | 6.45 ms | 360.36 ms | 372.12 ms | 48.5 MB | 398 B | 20.85 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.05 ms | 37.03 ms | 15.80 ms | 19.42 ms | 52.7 MB | 398 B | 20.69 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.71 ms | 299.40 µs | 14.32 ms | 21.18 ms | 55.4 MB | 398 B | 19.60 ms as a whole process |  |
| Seal | Normal | 20.07 ms | 59.55 µs | 19.97 ms | 20.01 ms | 3.1 MB | 1.2 KB | 200.00 µs cold, 292 ns warm; 2.85 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.84 ms | 532.12 µs | 27.02 ms | 28.47 ms | 4.8 MB | 981 B | 5.23 ms as a whole process |  |
| Node `--jitless` | Normal | 64.72 ms | 443.20 µs | 64.16 ms | 64.39 ms | 48.6 MB | 1.0 KB | 19.92 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.65 ms | 201.16 µs | 1.48 ms | 3.25 ms | 52.9 MB | 1.0 KB | 20.18 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 16.17 µs | 1.64 ms | 3.53 ms | 53.0 MB | 1.0 KB | 20.27 ms as a whole process |  |
| Seal | Ceil.source | 20.00 ms | 40.71 µs | 19.94 ms | 19.97 ms | 3.2 MB | 1.2 KB | 194.08 µs cold, 292 ns warm; 2.75 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.10 ms | 46.96 µs | 20.01 ms | 20.15 ms | 3.2 MB | 1.2 KB | 203.12 µs cold, 333 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.22 ms | 59.65 µs | 19.14 ms | 19.12 ms | 2.9 MB | 1.2 KB | 170.08 µs cold, 291 ns warm; 2.87 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 19.26 ms | 153.21 µs | 19.03 ms | 19.44 ms | 2.9 MB | 1.2 KB | 180.54 µs cold, 250 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.53 ms | 661.51 µs | 26.91 ms | 27.22 ms | 4.8 MB | 981 B | 5.31 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 65.06 ms | 587.93 µs | 64.12 ms | 66.03 ms | 48.4 MB | 1.0 KB | 20.27 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.64 ms | 161.16 µs | 1.52 ms | 3.68 ms | 52.9 MB | 1.0 KB | 19.69 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.66 ms | 21.17 µs | 1.63 ms | 3.19 ms | 53.0 MB | 1.0 KB | 20.20 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.57 ms | 494.91 µs | 96.76 ms | 96.54 ms | 2.8 MB | 632 B | 118.08 µs cold, 250 ns warm; 2.55 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 89.45 ms | 992.26 µs | 87.94 ms | 89.40 ms | 4.8 MB | 523 B | 5.37 ms as a whole process |  |
| Node `--jitless` | Purpose | 145.70 ms | 711.49 µs | 144.92 ms | 149.93 ms | 43.5 MB | 568 B | 19.62 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 28.66 µs | 3.64 ms | 4.49 ms | 49.3 MB | 568 B | 20.24 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.66 ms | 20.88 µs | 3.63 ms | 4.49 ms | 49.4 MB | 568 B | 20.33 ms as a whole process |  |
| Seal | Normal | 113.74 ms | 515.43 µs | 113.16 ms | 114.85 ms | 3.0 MB | 1.8 KB | 207.50 µs cold, 292 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 210.71 ms | 1.71 ms | 205.90 ms | 212.32 ms | 4.8 MB | 1.7 KB | 5.22 ms as a whole process |  |
| Node `--jitless` | Normal | 153.86 ms | 509.98 µs | 152.75 ms | 154.36 ms | 45.4 MB | 1.7 KB | 20.21 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.21 ms | 64.53 µs | 14.14 ms | 15.82 ms | 52.1 MB | 1.7 KB | 19.81 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.75 ms | 64.51 µs | 14.63 ms | 16.44 ms | 52.1 MB | 1.7 KB | 19.84 ms as a whole process |  |
| Seal | Ceil.source | 112.48 ms | 334.66 µs | 111.99 ms | 112.26 ms | 3.0 MB | 1.8 KB | 185.46 µs cold, 292 ns warm; 2.79 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 115.25 ms | 667.90 µs | 114.24 ms | 112.31 ms | 3.1 MB | 1.8 KB | 183.62 µs cold, 291 ns warm; 3.04 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 105.06 ms | 823.79 µs | 104.07 ms | 105.92 ms | 2.9 MB | 1.8 KB | 170.71 µs cold, 250 ns warm; 2.77 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 105.57 ms | 755.38 µs | 104.32 ms | 105.72 ms | 2.9 MB | 1.8 KB | 172.29 µs cold, 209 ns warm; 2.65 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 211.91 ms | 2.06 ms | 206.26 ms | 212.61 ms | 4.8 MB | 1.7 KB | 5.26 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 160.25 ms | 1.28 ms | 158.61 ms | 159.10 ms | 45.6 MB | 1.7 KB | 20.72 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.92 ms | 164.33 µs | 14.59 ms | 16.26 ms | 51.9 MB | 1.7 KB | 19.74 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.67 ms | 199.60 µs | 14.41 ms | 16.16 ms | 52.2 MB | 1.7 KB | 19.86 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.76 ms | 1.25 ms | 95.94 ms | 95.17 ms | 4.0 MB | 569 B | 122.67 µs cold, 375 ns warm; 2.75 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.23 ms | 368.60 µs | 62.71 ms | 63.27 ms | 4.8 MB | 540 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.39 ms | 3.23 ms | 241.73 ms | 243.15 ms | 48.8 MB | 514 B | 20.47 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 91.48 ms | 24.19 ms | 69.44 ms | 92.88 ms | 52.6 MB | 514 B | 20.29 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 63.78 ms | 494.54 µs | 63.21 ms | 89.52 ms | 52.6 MB | 514 B | 21.24 ms as a whole process |  |
| Seal | Normal | 24.41 ms | 215.25 µs | 24.15 ms | 24.13 ms | 4.0 MB | 538 B | 131.33 µs cold, 416 ns warm; 2.93 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.89 ms | 661.98 µs | 30.28 ms | 31.31 ms | 6.0 MB | 478 B | 5.67 ms as a whole process |  |
| Node `--jitless` | Normal | 52.55 ms | 538.33 µs | 51.45 ms | 55.65 ms | 56.5 MB | 477 B | 21.80 ms as a whole process |  |
| Node (JIT), from cold | Normal | 36.10 ms | 567.72 µs | 35.40 ms | 37.77 ms | 62.5 MB | 477 B | 21.35 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 36.25 ms | 349.10 µs | 35.61 ms | 35.93 ms | 87.9 MB | 477 B | 21.28 ms as a whole process |  |
| Seal | Ceil.source | 24.61 ms | 191.76 µs | 24.39 ms | 24.34 ms | 4.0 MB | 538 B | 139.67 µs cold, 583 ns warm; 3.03 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.78 ms | 539.21 µs | 25.02 ms | 25.31 ms | 4.1 MB | 538 B | 137.21 µs cold, 417 ns warm; 3.04 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.16 ms | 221.58 µs | 23.85 ms | 24.61 ms | 3.9 MB | 538 B | 126.12 µs cold, 333 ns warm; 3.04 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 25.83 ms | 1.95 ms | 24.55 ms | 24.09 ms | 3.9 MB | 538 B | 133.17 µs cold, 333 ns warm; 2.84 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.83 ms | 369.31 µs | 30.15 ms | 30.98 ms | 5.9 MB | 478 B | 5.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 53.02 ms | 576.94 µs | 52.00 ms | 54.32 ms | 56.5 MB | 477 B | 22.44 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.06 ms | 258.53 µs | 34.64 ms | 35.80 ms | 62.5 MB | 477 B | 20.55 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 36.16 ms | 522.99 µs | 35.60 ms | 36.70 ms | 87.9 MB | 477 B | 20.01 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 308.64 µs | 4.09 µs | 304.96 µs | 326.58 µs | 3.0 MB | 233 B | 81.12 µs cold, 375 ns warm; 2.72 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 146.18 ms | 272.24 µs | 145.76 ms | 146.86 ms | 9.7 MB | 195 B | 5.23 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.11 µs | 147.04 µs | 240.96 µs | 1.03 ms | 52.5 MB | 223 B | 20.34 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 219.83 µs | 153.59 µs | 93.17 µs | 925.42 µs | 57.7 MB | 223 B | 20.00 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 137.68 µs | 125.42 µs | 55.96 µs | 906.00 µs | 57.9 MB | 223 B | 19.86 ms as a whole process |  |
| Seal | Normal | 349.70 µs | 6.60 µs | 342.21 µs | 368.92 µs | 3.2 MB | 182 B | 82.83 µs cold, 334 ns warm; 2.65 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 164.43 ms | 968.51 µs | 163.03 ms | 165.06 ms | 11.8 MB | 138 B | 5.17 ms as a whole process |  |
| Node `--jitless` | Normal | 521.64 µs | 354.78 µs | 271.46 µs | 1.12 ms | 61.4 MB | 155 B | 22.21 ms as a whole process |  |
| Node (JIT), from cold | Normal | 505.90 µs | 491.54 µs | 126.08 µs | 1.04 ms | 65.3 MB | 155 B | 20.18 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 292.72 µs | 331.04 µs | 136.92 µs | 1.08 ms | 69.2 MB | 155 B | 19.86 ms as a whole process |  |
| Seal | Ceil.source | 372.14 µs | 9.94 µs | 363.38 µs | 405.58 µs | 3.2 MB | 182 B | 84.21 µs cold, 375 ns warm; 2.59 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 351.47 µs | 5.24 µs | 345.50 µs | 373.17 µs | 3.2 MB | 182 B | 85.79 µs cold, 375 ns warm; 2.65 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 283.57 µs | 4.02 µs | 278.96 µs | 313.96 µs | 3.0 MB | 182 B | 75.58 µs cold, 333 ns warm; 2.42 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 283.49 µs | 9.35 µs | 273.50 µs | 314.75 µs | 3.1 MB | 182 B | 73.46 µs cold, 292 ns warm; 2.63 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 163.35 ms | 262.32 µs | 162.87 ms | 165.25 ms | 11.3 MB | 138 B | 5.21 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 503.47 µs | 344.71 µs | 260.00 µs | 1.10 ms | 61.4 MB | 155 B | 20.41 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 506.90 µs | 477.46 µs | 137.96 µs | 1.03 ms | 65.2 MB | 155 B | 20.20 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 278.39 µs | 334.22 µs | 124.50 µs | 1.10 ms | 69.4 MB | 155 B | 19.42 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 76.44 ms | 745.12 µs | 75.30 ms | 75.82 ms | 2.9 MB | 537 B | 116.42 µs cold, 292 ns warm; 2.74 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 97.07 ms | 872.72 µs | 96.24 ms | 96.25 ms | 4.8 MB | 411 B | 5.33 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.07 ms | 202.59 µs | 76.77 ms | 80.13 ms | 59.6 MB | 444 B | 21.67 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.84 ms | 291.61 µs | 10.34 ms | 13.52 ms | 65.7 MB | 444 B | 20.51 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.59 ms | 665.58 µs | 10.23 ms | 13.24 ms | 114.7 MB | 444 B | 21.38 ms as a whole process |  |
| Seal | Normal | 114.81 ms | 923.90 µs | 114.00 ms | 115.67 ms | 8.0 MB | 992 B | 167.62 µs cold, 292 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 133.60 ms | 3.60 ms | 130.07 ms | 131.65 ms | 43.2 MB | 799 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Normal | 113.35 ms | 2.31 ms | 111.72 ms | 117.09 ms | 213.2 MB | 878 B | 19.69 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.23 ms | 877.86 µs | 13.45 ms | 17.09 ms | 129.6 MB | 878 B | 20.18 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.47 ms | 84.76 µs | 13.31 ms | 17.16 ms | 224.0 MB | 878 B | 20.18 ms as a whole process |  |
| Seal | Ceil.source | 112.84 ms | 334.98 µs | 112.09 ms | 113.51 ms | 7.9 MB | 992 B | 165.25 µs cold, 292 ns warm; 2.73 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.98 ms | 1.34 ms | 112.85 ms | 114.57 ms | 8.0 MB | 992 B | 164.96 µs cold, 292 ns warm; 2.94 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 92.60 ms | 483.73 µs | 92.07 ms | 94.45 ms | 7.7 MB | 992 B | 139.71 µs cold, 250 ns warm; 2.79 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 93.66 ms | 1.38 ms | 91.58 ms | 93.26 ms | 7.8 MB | 992 B | 151.71 µs cold, 166 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 140.65 ms | 4.05 ms | 136.14 ms | 134.75 ms | 40.8 MB | 799 B | 5.33 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 119.60 ms | 15.69 ms | 112.63 ms | 120.43 ms | 214.0 MB | 878 B | 21.37 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.25 ms | 891.66 µs | 13.35 ms | 17.16 ms | 129.7 MB | 878 B | 20.59 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.27 ms | 81.14 µs | 13.15 ms | 17.29 ms | 223.7 MB | 878 B | 19.58 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 77.12 µs (±10.78 µs, least 65.33 µs) | 43.07 µs (±4.32 µs, least 34.88 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±1 ns, least 37 ns) | 512 B | 2.8 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.02 µs (±994 ns, least 9.04 µs) | 19.43 µs (±2.90 µs, least 17.25 µs) | 42 ns (±2 ns, least 39 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 36.39 µs (±12.22 µs, least 19.67 µs) | 40.96 µs (±3.78 µs, least 36.92 µs) | 58 ns (±1 ns, least 55 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 28.37 µs (±8.02 µs, least 18.83 µs) | 42.29 µs (±7.84 µs, least 36.96 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 12.60 µs (±282 ns, least 12.26 µs) | 231 ns (±15 ns, least 214 ns) | 18 ns (±1 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 5.3 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.99 µs (±157 ns, least 4.70 µs) | 190 ns (±14 ns, least 175 ns) | 34 ns (±1 ns, least 33 ns) | 19 ns (±1 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.18 µs (±48 ns, least 1.13 µs) | 715 ns (±41 ns, least 678 ns) | 53 ns (±2 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.12 µs (±30 ns, least 1.07 µs) | 704 ns (±22 ns, least 662 ns) | 22 ns (±0 ns, least 21 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.8 MB | the engine's own heap |
| Seal | Ceil.source | 12.54 µs (±183 ns, least 12.08 µs) | 237 ns (±13 ns, least 213 ns) | 18 ns (±0 ns, least 17 ns) | 37 ns (±1 ns, least 37 ns) | 512 B | 5.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 13.05 µs (±127 ns, least 12.87 µs) | 264 ns (±20 ns, least 230 ns) | 19 ns (±0 ns, least 19 ns) | 39 ns (±1 ns, least 38 ns) | 512 B | 5.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 11.48 µs (±258 ns, least 11.24 µs) | 223 ns (±8 ns, least 207 ns) | 15 ns (±1 ns, least 15 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 5.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 12.01 µs (±300 ns, least 11.69 µs) | 226 ns (±21 ns, least 201 ns) | 15 ns (±0 ns, least 15 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 5.3 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.98 µs (±111 ns, least 4.87 µs) | 189 ns (±8 ns, least 176 ns) | 33 ns (±0 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.16 µs (±36 ns, least 1.11 µs) | 688 ns (±28 ns, least 651 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.13 µs (±31 ns, least 1.10 µs) | 692 ns (±32 ns, least 649 ns) | 23 ns (±1 ns, least 22 ns) | 7 ns (±0 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 53.70 ms | 51.82 ms (-3.5%) |
| n-body, 100,000 steps | 69.39 ms | 63.58 ms (-8.4%) |
| spectral-norm, size 200 | 20.63 ms | 19.96 ms (-3.2%) |
| fannkuch-redux, 9 | 117.66 ms | 112.56 ms (-4.3%) |
| word count, 500,000 words | 23.82 ms | 24.60 ms (+3.2%) |
| text building, 20,000 joins | 380.99 µs | 352.95 µs (-7.4%) |
| binary-trees, depth 14 | 112.53 ms | 113.03 ms (+0.4%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 232.59 ms | 750.38 µs | 231.66 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 11.00 ms | 133.10 µs | 10.92 ms | 2.8 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.46 ms | 13.57 µs | 4.44 ms | 3.3 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.44 ms | 20.70 µs | 1.41 ms | 3.5 MB |
