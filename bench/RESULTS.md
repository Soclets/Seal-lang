# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.48 2.02 1.78 }` before, `{ 1.78 1.86 1.74 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 52.12 ms | 73.22 ms | **met**: Seal takes 71% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.03 ms | 69.61 ms | **met**: Seal takes 95% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.22 ms | 27.65 ms | **met**: Seal takes 70% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 113.28 ms | 211.39 ms | **met**: Seal takes 54% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 24.39 ms | 29.89 ms | **met**: Seal takes 82% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 334.68 µs | 160.17 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 114.51 ms | 126.50 ms | **met**: Seal takes 91% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.10 ms | 3.47 ms | **met**: Seal takes 61% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 19 ns | 33 ns | **met**: Seal takes 57% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 212 ns | 183 ns | **not met**: Seal takes 115% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 10.47 µs | 4.85 µs | **not met**: Seal takes 216% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 52.12 ms | 93.73 ms | **met**: Seal takes 56% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.03 ms | 177.44 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 24.39 ms | 50.92 ms | **met**: Seal takes 48% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 334.68 µs | 513.12 µs | **met**: Seal takes 65% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 19 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 145.21 ms | 661.49 µs | 144.34 ms | 148.72 ms | 2.6 MB | 395 B | 388.50 µs cold, 375 ns warm; 2.77 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 196.07 ms | 1.07 ms | 194.67 ms | 197.67 ms | 4.8 MB | 339 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Purpose | 319.66 ms | 2.11 ms | 317.45 ms | 319.64 ms | 43.5 MB | 362 B | 19.54 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.17 ms | 754.24 µs | 123.01 ms | 125.32 ms | 49.2 MB | 362 B | 18.81 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 122.64 ms | 546.60 µs | 121.97 ms | 123.89 ms | 49.1 MB | 362 B | 19.27 ms as a whole process |  |
| Seal | Normal | 52.12 ms | 206.14 µs | 51.87 ms | 52.04 ms | 2.5 MB | 169 B | 78.25 µs cold, 250 ns warm; 2.78 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 73.22 ms | 545.49 µs | 72.33 ms | 72.24 ms | 4.8 MB | 143 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Normal | 93.73 ms | 721.55 µs | 92.82 ms | 94.98 ms | 43.4 MB | 137 B | 18.96 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.66 ms | 433.98 µs | 9.40 ms | 9.95 ms | 48.6 MB | 137 B | 18.52 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.89 ms | 398.43 µs | 9.39 ms | 10.80 ms | 48.6 MB | 137 B | 18.80 ms as a whole process |  |
| Seal | Ceil.source | 51.95 ms | 101.81 µs | 51.77 ms | 51.92 ms | 2.5 MB | 169 B | 79.75 µs cold, 250 ns warm; 2.67 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 55.07 ms | 119.56 µs | 54.83 ms | 82.49 ms | 2.5 MB | 169 B | 811.88 µs cold, 667 ns warm; 2.79 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 52.66 ms | 1.20 ms | 50.13 ms | 77.18 ms | 2.4 MB | 169 B | 627.83 µs cold, 542 ns warm; 2.59 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 50.82 ms | 1.67 ms | 49.02 ms | 70.42 ms | 2.4 MB | 169 B | 524.71 µs cold, 375 ns warm; 2.69 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 74.07 ms | 1.28 ms | 72.40 ms | 71.98 ms | 4.8 MB | 143 B | 5.17 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.12 ms | 662.94 µs | 95.23 ms | 96.69 ms | 43.4 MB | 137 B | 19.06 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.66 ms | 202.66 µs | 9.50 ms | 11.95 ms | 48.7 MB | 137 B | 18.89 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.46 ms | 50.65 µs | 9.40 ms | 10.27 ms | 48.5 MB | 137 B | 19.32 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.35 ms | 177.35 µs | 51.93 ms | 67.58 ms | 2.6 MB | 550 B | 146.21 µs cold, 292 ns warm; 2.77 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.08 ms | 500.28 µs | 91.35 ms | 90.76 ms | 4.8 MB | 433 B | 5.30 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.07 ms | 1.26 ms | 243.63 ms | 245.80 ms | 46.5 MB | 460 B | 19.03 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.77 ms | 161.43 µs | 18.51 ms | 18.77 ms | 50.2 MB | 460 B | 18.92 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.73 ms | 202.16 µs | 18.41 ms | 18.51 ms | 50.2 MB | 460 B | 18.74 ms as a whole process |  |
| Seal | Normal | 66.03 ms | 645.35 µs | 64.91 ms | 97.28 ms | 3.1 MB | 3.5 KB | 311.46 µs cold, 375 ns warm; 3.01 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.61 ms | 128.35 µs | 69.38 ms | 69.68 ms | 4.8 MB | 3.0 KB | 5.17 ms as a whole process |  |
| Node `--jitless` | Normal | 177.44 ms | 4.47 ms | 172.44 ms | 179.08 ms | 48.1 MB | 3.1 KB | 19.02 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 35.43 µs | 3.82 ms | 5.65 ms | 52.0 MB | 3.1 KB | 18.74 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 27.62 µs | 3.83 ms | 5.70 ms | 51.8 MB | 3.1 KB | 18.85 ms as a whole process |  |
| Seal | Ceil.source | 57.03 ms | 466.36 µs | 56.49 ms | 58.54 ms | 3.0 MB | 3.5 KB | 293.88 µs cold, 333 ns warm; 2.92 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 54.50 ms | 733.13 µs | 53.17 ms | 55.15 ms | 3.0 MB | 3.5 KB | 300.54 µs cold, 334 ns warm; 2.73 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.68 ms | 337.10 µs | 49.28 ms | 49.38 ms | 2.9 MB | 3.5 KB | 286.96 µs cold, 292 ns warm; 2.78 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 46.56 ms | 416.35 µs | 45.82 ms | 46.04 ms | 2.9 MB | 3.5 KB | 308.46 µs cold, 208 ns warm; 2.87 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.41 ms | 261.20 µs | 68.93 ms | 69.67 ms | 4.8 MB | 3.0 KB | 5.30 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 178.69 ms | 3.49 ms | 173.76 ms | 184.88 ms | 48.0 MB | 3.1 KB | 19.99 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.90 ms | 39.65 µs | 3.85 ms | 5.79 ms | 52.0 MB | 3.1 KB | 19.22 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.89 ms | 37.69 µs | 3.84 ms | 7.81 ms | 52.2 MB | 3.1 KB | 19.18 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.16 ms | 1.87 ms | 51.64 ms | 54.95 ms | 2.7 MB | 429 B | 89.54 µs cold, 250 ns warm; 2.69 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.65 ms | 279.89 µs | 45.33 ms | 45.31 ms | 4.8 MB | 366 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Purpose | 367.40 ms | 6.47 ms | 361.44 ms | 372.50 ms | 48.6 MB | 398 B | 19.22 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.77 ms | 36.60 ms | 15.60 ms | 21.57 ms | 52.6 MB | 398 B | 18.91 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.62 ms | 400.17 µs | 14.29 ms | 21.43 ms | 55.2 MB | 398 B | 19.51 ms as a whole process |  |
| Seal | Normal | 19.22 ms | 36.63 µs | 19.16 ms | 19.38 ms | 2.9 MB | 1.2 KB | 178.79 µs cold, 292 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.65 ms | 445.03 µs | 27.29 ms | 27.63 ms | 4.8 MB | 981 B | 5.20 ms as a whole process |  |
| Node `--jitless` | Normal | 64.04 ms | 339.39 µs | 63.47 ms | 64.95 ms | 48.5 MB | 1.0 KB | 19.06 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.64 ms | 202.76 µs | 1.52 ms | 3.36 ms | 52.8 MB | 1.0 KB | 18.75 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 42.06 µs | 1.63 ms | 3.58 ms | 52.8 MB | 1.0 KB | 18.89 ms as a whole process |  |
| Seal | Ceil.source | 19.36 ms | 123.54 µs | 19.11 ms | 19.53 ms | 2.9 MB | 1.2 KB | 181.29 µs cold, 250 ns warm; 2.77 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.83 ms | 55.51 µs | 20.77 ms | 21.24 ms | 2.9 MB | 1.2 KB | 173.17 µs cold, 292 ns warm; 2.74 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.19 ms | 698.91 µs | 17.12 ms | 18.80 ms | 2.7 MB | 1.2 KB | 154.83 µs cold, 250 ns warm; 2.65 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 16.37 ms | 78.30 µs | 16.28 ms | 16.35 ms | 2.8 MB | 1.2 KB | 162.54 µs cold, 250 ns warm; 2.69 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.10 ms | 372.19 µs | 26.56 ms | 27.07 ms | 4.8 MB | 981 B | 5.11 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.25 ms | 538.87 µs | 63.43 ms | 64.21 ms | 48.3 MB | 1.0 KB | 18.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.64 ms | 150.42 µs | 1.51 ms | 3.21 ms | 52.8 MB | 1.0 KB | 18.86 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.62 ms | 31.74 µs | 1.57 ms | 3.34 ms | 52.7 MB | 1.0 KB | 18.66 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 114.19 ms | 2.56 ms | 110.58 ms | 115.53 ms | 2.6 MB | 632 B | 105.67 µs cold, 250 ns warm; 2.57 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.25 ms | 560.03 µs | 87.13 ms | 91.18 ms | 4.9 MB | 523 B | 5.25 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.31 ms | 5.01 ms | 145.09 ms | 147.20 ms | 43.4 MB | 568 B | 19.76 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.66 ms | 39.65 µs | 3.61 ms | 4.46 ms | 49.3 MB | 568 B | 19.54 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.65 ms | 38.79 µs | 3.61 ms | 4.46 ms | 49.4 MB | 568 B | 18.52 ms as a whole process |  |
| Seal | Normal | 113.28 ms | 650.87 µs | 112.47 ms | 114.70 ms | 2.8 MB | 1.8 KB | 169.46 µs cold, 291 ns warm; 2.79 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 211.39 ms | 2.02 ms | 206.75 ms | 213.56 ms | 4.8 MB | 1.7 KB | 5.08 ms as a whole process |  |
| Node `--jitless` | Normal | 154.55 ms | 1.07 ms | 153.40 ms | 155.87 ms | 45.3 MB | 1.7 KB | 18.14 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.05 ms | 94.08 µs | 13.91 ms | 15.47 ms | 52.0 MB | 1.7 KB | 18.58 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.97 ms | 67.23 µs | 14.88 ms | 16.59 ms | 52.1 MB | 1.7 KB | 18.63 ms as a whole process |  |
| Seal | Ceil.source | 113.11 ms | 449.41 µs | 112.36 ms | 113.12 ms | 2.8 MB | 1.8 KB | 162.46 µs cold, 333 ns warm; 2.53 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 116.54 ms | 236.87 µs | 116.18 ms | 116.85 ms | 2.9 MB | 1.8 KB | 167.71 µs cold, 375 ns warm; 2.72 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 101.52 ms | 271.98 µs | 101.21 ms | 101.54 ms | 2.7 MB | 1.8 KB | 153.21 µs cold, 292 ns warm; 2.72 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.35 ms | 317.83 µs | 102.76 ms | 103.39 ms | 2.8 MB | 1.8 KB | 158.21 µs cold, 292 ns warm; 2.76 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 217.39 ms | 5.82 ms | 205.73 ms | 212.69 ms | 4.9 MB | 1.7 KB | 5.08 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 154.53 ms | 395.38 µs | 153.86 ms | 156.90 ms | 45.4 MB | 1.7 KB | 18.77 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 15.98 ms | 663.72 µs | 14.82 ms | 16.30 ms | 52.0 MB | 1.7 KB | 18.54 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.02 ms | 98.61 µs | 13.85 ms | 15.59 ms | 52.2 MB | 1.7 KB | 18.65 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.47 ms | 1.72 ms | 93.55 ms | 92.58 ms | 3.8 MB | 569 B | 101.58 µs cold, 375 ns warm; 2.47 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.42 ms | 127.25 µs | 62.22 ms | 62.43 ms | 4.8 MB | 540 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Purpose | 238.84 ms | 1.64 ms | 236.76 ms | 241.91 ms | 48.6 MB | 514 B | 18.28 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 86.20 ms | 24.31 ms | 63.84 ms | 89.61 ms | 52.7 MB | 514 B | 18.31 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 62.57 ms | 739.20 µs | 61.42 ms | 90.22 ms | 52.6 MB | 514 B | 18.75 ms as a whole process |  |
| Seal | Normal | 24.39 ms | 41.33 µs | 24.34 ms | 24.42 ms | 3.7 MB | 538 B | 101.25 µs cold, 375 ns warm; 2.54 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.89 ms | 118.32 µs | 29.69 ms | 29.86 ms | 6.0 MB | 478 B | 4.99 ms as a whole process |  |
| Node `--jitless` | Normal | 50.92 ms | 155.49 µs | 50.69 ms | 52.27 ms | 56.6 MB | 477 B | 18.26 ms as a whole process |  |
| Node (JIT), from cold | Normal | 34.56 ms | 320.67 µs | 34.23 ms | 35.68 ms | 62.5 MB | 477 B | 18.29 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 34.78 ms | 273.16 µs | 34.47 ms | 35.73 ms | 87.9 MB | 477 B | 18.34 ms as a whole process |  |
| Seal | Ceil.source | 24.30 ms | 164.46 µs | 24.11 ms | 24.21 ms | 3.8 MB | 538 B | 106.29 µs cold, 333 ns warm; 2.59 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.54 ms | 72.74 µs | 25.45 ms | 25.34 ms | 3.7 MB | 538 B | 107.12 µs cold, 375 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.50 ms | 93.02 µs | 23.35 ms | 23.53 ms | 3.7 MB | 538 B | 98.58 µs cold, 334 ns warm; 2.51 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.26 ms | 52.54 µs | 23.20 ms | 23.31 ms | 3.8 MB | 538 B | 100.92 µs cold, 333 ns warm; 2.49 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.74 ms | 132.27 µs | 29.55 ms | 29.88 ms | 6.0 MB | 478 B | 5.09 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.48 ms | 216.91 µs | 51.18 ms | 52.37 ms | 56.5 MB | 477 B | 18.70 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.62 ms | 364.76 µs | 34.26 ms | 35.47 ms | 62.5 MB | 477 B | 18.29 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 34.86 ms | 329.55 µs | 34.47 ms | 35.49 ms | 88.3 MB | 477 B | 18.75 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 286.73 µs | 2.50 µs | 284.62 µs | 299.67 µs | 2.9 MB | 233 B | 67.00 µs cold, 375 ns warm; 2.56 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 142.94 ms | 260.46 µs | 142.55 ms | 144.83 ms | 9.7 MB | 195 B | 4.95 ms as a whole process |  |
| Node `--jitless` | Purpose | 361.32 µs | 141.00 µs | 241.88 µs | 1.06 ms | 52.5 MB | 223 B | 18.73 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 225.87 µs | 146.19 µs | 100.71 µs | 912.54 µs | 57.6 MB | 223 B | 18.68 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 167.45 µs | 169.11 µs | 56.00 µs | 938.58 µs | 58.5 MB | 223 B | 18.34 ms as a whole process |  |
| Seal | Normal | 334.68 µs | 6.05 µs | 329.29 µs | 363.33 µs | 3.0 MB | 182 B | 69.17 µs cold, 375 ns warm; 2.58 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.17 ms | 228.61 µs | 159.72 ms | 162.23 ms | 11.3 MB | 138 B | 4.90 ms as a whole process |  |
| Node `--jitless` | Normal | 513.12 µs | 344.62 µs | 270.33 µs | 1.07 ms | 61.4 MB | 155 B | 18.14 ms as a whole process |  |
| Node (JIT), from cold | Normal | 502.58 µs | 484.95 µs | 126.08 µs | 1.05 ms | 65.2 MB | 155 B | 18.04 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 276.69 µs | 327.63 µs | 124.54 µs | 1.07 ms | 69.4 MB | 155 B | 18.08 ms as a whole process |  |
| Seal | Ceil.source | 338.99 µs | 1.93 µs | 336.88 µs | 348.58 µs | 3.0 MB | 182 B | 73.46 µs cold, 375 ns warm; 2.50 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 341.42 µs | 5.46 µs | 334.17 µs | 359.92 µs | 3.0 MB | 182 B | 69.88 µs cold, 291 ns warm; 2.47 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 275.37 µs | 2.40 µs | 271.67 µs | 297.96 µs | 2.9 MB | 182 B | 61.21 µs cold, 292 ns warm; 2.50 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 264.02 µs | 2.87 µs | 261.83 µs | 284.21 µs | 3.0 MB | 182 B | 69.08 µs cold, 334 ns warm; 2.43 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 159.74 ms | 307.02 µs | 159.27 ms | 161.95 ms | 11.3 MB | 138 B | 4.93 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 510.61 µs | 342.60 µs | 283.25 µs | 1.14 ms | 61.4 MB | 155 B | 18.05 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 496.09 µs | 474.95 µs | 127.62 µs | 1.10 ms | 65.3 MB | 155 B | 18.25 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 283.76 µs | 331.93 µs | 124.83 µs | 1.11 ms | 69.4 MB | 155 B | 18.70 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.28 ms | 573.05 µs | 74.54 ms | 75.82 ms | 2.8 MB | 537 B | 106.17 µs cold, 250 ns warm; 2.62 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 93.79 ms | 368.54 µs | 92.91 ms | 93.56 ms | 4.8 MB | 411 B | 4.80 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.97 ms | 892.14 µs | 76.35 ms | 80.62 ms | 59.6 MB | 444 B | 18.72 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.39 ms | 269.17 µs | 10.10 ms | 12.87 ms | 65.9 MB | 444 B | 17.94 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.46 ms | 827.44 µs | 9.92 ms | 12.86 ms | 114.8 MB | 444 B | 18.17 ms as a whole process |  |
| Seal | Normal | 114.51 ms | 1.15 ms | 112.24 ms | 117.38 ms | 7.7 MB | 992 B | 149.25 µs cold, 250 ns warm; 2.67 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 126.50 ms | 1.60 ms | 124.79 ms | 127.00 ms | 42.7 MB | 799 B | 5.08 ms as a whole process |  |
| Node `--jitless` | Normal | 114.87 ms | 2.66 ms | 112.46 ms | 121.14 ms | 213.2 MB | 878 B | 17.98 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.17 ms | 977.91 µs | 13.16 ms | 16.90 ms | 129.7 MB | 878 B | 18.78 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.29 ms | 126.97 µs | 13.04 ms | 16.87 ms | 224.0 MB | 878 B | 17.97 ms as a whole process |  |
| Seal | Ceil.source | 112.56 ms | 1.33 ms | 110.68 ms | 114.87 ms | 7.7 MB | 992 B | 148.29 µs cold, 250 ns warm; 2.50 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.22 ms | 773.37 µs | 113.01 ms | 117.30 ms | 7.6 MB | 992 B | 145.04 µs cold, 292 ns warm; 2.77 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 95.49 ms | 745.33 µs | 94.34 ms | 100.15 ms | 7.6 MB | 992 B | 129.96 µs cold, 250 ns warm; 2.60 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 95.06 ms | 551.93 µs | 94.40 ms | 95.63 ms | 7.6 MB | 992 B | 128.17 µs cold, 250 ns warm; 2.67 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 126.68 ms | 1.14 ms | 124.54 ms | 128.15 ms | 42.2 MB | 799 B | 5.00 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 112.98 ms | 1.68 ms | 111.38 ms | 116.58 ms | 213.4 MB | 878 B | 18.03 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.25 ms | 917.57 µs | 13.41 ms | 16.88 ms | 129.6 MB | 878 B | 18.45 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.19 ms | 52.03 µs | 13.12 ms | 16.88 ms | 223.9 MB | 878 B | 18.58 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 59.26 µs (±5.82 µs, least 53.50 µs) | 32.82 µs (±7.01 µs, least 25.46 µs) | 17 ns (±0 ns, least 16 ns) | 36 ns (±1 ns, least 36 ns) | 512 B | 2.6 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.53 µs (±677 ns, least 8.67 µs) | 16.55 µs (±1.51 µs, least 15.08 µs) | 41 ns (±1 ns, least 39 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 33.32 µs (±6.78 µs, least 20.38 µs) | 38.16 µs (±2.30 µs, least 35.54 µs) | 56 ns (±2 ns, least 53 ns) | 29 ns (±1 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 30.17 µs (±7.92 µs, least 18.38 µs) | 38.11 µs (±2.37 µs, least 35.12 µs) | 9 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 10.47 µs (±122 ns, least 10.34 µs) | 212 ns (±5 ns, least 205 ns) | 19 ns (±3 ns, least 18 ns) | 38 ns (±2 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.85 µs (±166 ns, least 4.66 µs) | 183 ns (±6 ns, least 178 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±20 ns, least 1.12 µs) | 669 ns (±18 ns, least 643 ns) | 51 ns (±1 ns, least 50 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±24 ns, least 1.07 µs) | 675 ns (±12 ns, least 659 ns) | 22 ns (±0 ns, least 21 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 10.90 µs (±470 ns, least 10.40 µs) | 211 ns (±10 ns, least 195 ns) | 18 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 10.77 µs (±192 ns, least 10.50 µs) | 211 ns (±9 ns, least 194 ns) | 18 ns (±0 ns, least 17 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 9.61 µs (±324 ns, least 9.33 µs) | 201 ns (±11 ns, least 184 ns) | 14 ns (±0 ns, least 14 ns) | 35 ns (±1 ns, least 34 ns) | 512 B | 4.9 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 9.88 µs (±282 ns, least 9.69 µs) | 198 ns (±8 ns, least 188 ns) | 15 ns (±0 ns, least 14 ns) | 35 ns (±0 ns, least 34 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.89 µs (±199 ns, least 4.58 µs) | 179 ns (±7 ns, least 166 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.14 µs (±26 ns, least 1.08 µs) | 663 ns (±18 ns, least 630 ns) | 50 ns (±2 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±20 ns, least 1.06 µs) | 665 ns (±15 ns, least 634 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 51.65 ms | 51.60 ms (-0.1%) |
| n-body, 100,000 steps | 65.16 ms | 61.26 ms (-6.0%) |
| spectral-norm, size 200 | 19.29 ms | 18.78 ms (-2.6%) |
| fannkuch-redux, 9 | 111.66 ms | 113.56 ms (+1.7%) |
| word count, 500,000 words | 24.39 ms | 24.07 ms (-1.3%) |
| text building, 20,000 joins | 328.09 µs | 330.64 µs (+0.8%) |
| binary-trees, depth 14 | 114.73 ms | 114.64 ms (-0.1%) |
