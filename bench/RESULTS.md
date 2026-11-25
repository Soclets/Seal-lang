# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.42 2.40 2.23 }` before, `{ 1.97 2.18 2.16 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 52.46 ms | 71.39 ms | **met**: Seal takes 73% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.14 ms | 69.67 ms | **met**: Seal takes 95% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.27 ms | 27.91 ms | **met**: Seal takes 69% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 115.54 ms | 222.57 ms | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 24.81 ms | 30.80 ms | **met**: Seal takes 81% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 348.03 µs | 160.78 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 114.69 ms | 128.30 ms | **met**: Seal takes 89% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.02 ms | 3.47 ms | **met**: Seal takes 58% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 18 ns | 33 ns | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 222 ns | 184 ns | **not met**: Seal takes 121% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 11.09 µs | 4.87 µs | **not met**: Seal takes 228% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 52.46 ms | 97.24 ms | **met**: Seal takes 54% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.14 ms | 178.07 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 24.81 ms | 52.31 ms | **met**: Seal takes 47% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 348.03 µs | 497.75 µs | **met**: Seal takes 70% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 18 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 148.20 ms | 134.62 µs | 147.96 ms | 165.26 ms | 2.7 MB | 395 B | 525.33 µs cold, 542 ns warm; 2.71 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 198.07 ms | 231.95 µs | 197.69 ms | 197.20 ms | 4.8 MB | 339 B | 5.20 ms as a whole process |  |
| Node `--jitless` | Purpose | 324.33 ms | 2.71 ms | 322.07 ms | 323.51 ms | 43.2 MB | 362 B | 19.33 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.78 ms | 751.08 µs | 123.96 ms | 129.41 ms | 49.5 MB | 362 B | 19.05 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 124.83 ms | 660.77 µs | 124.12 ms | 124.85 ms | 49.2 MB | 362 B | 18.60 ms as a whole process |  |
| Seal | Normal | 52.46 ms | 81.47 µs | 52.34 ms | 52.56 ms | 2.6 MB | 169 B | 76.88 µs cold, 333 ns warm; 2.65 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 71.39 ms | 263.60 µs | 71.03 ms | 70.93 ms | 4.8 MB | 143 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Normal | 97.24 ms | 1.11 ms | 95.37 ms | 97.00 ms | 43.5 MB | 137 B | 20.03 ms as a whole process |  |
| Node (JIT), from cold | Normal | 10.47 ms | 179.35 µs | 10.27 ms | 11.05 ms | 48.5 MB | 137 B | 19.57 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.02 ms | 318.68 µs | 9.45 ms | 10.34 ms | 48.5 MB | 137 B | 19.43 ms as a whole process |  |
| Seal | Ceil.source | 52.84 ms | 206.01 µs | 52.58 ms | 52.70 ms | 2.6 MB | 169 B | 78.17 µs cold, 208 ns warm; 2.81 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 57.84 ms | 531.00 µs | 56.82 ms | 76.32 ms | 2.6 MB | 169 B | 652.08 µs cold, 542 ns warm; 2.79 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 54.09 ms | 748.14 µs | 52.18 ms | 77.59 ms | 2.5 MB | 169 B | 749.29 µs cold, 500 ns warm; 2.74 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 51.61 ms | 1.09 ms | 50.37 ms | 75.72 ms | 2.5 MB | 169 B | 759.83 µs cold, 583 ns warm; 2.55 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 78.22 ms | 3.28 ms | 73.20 ms | 75.37 ms | 4.8 MB | 143 B | 5.22 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 96.06 ms | 437.84 µs | 95.48 ms | 94.44 ms | 43.5 MB | 137 B | 19.43 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.81 ms | 523.42 µs | 9.42 ms | 10.00 ms | 48.7 MB | 137 B | 18.41 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.33 ms | 149.47 µs | 10.09 ms | 10.56 ms | 48.5 MB | 137 B | 18.93 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.56 ms | 283.80 µs | 53.09 ms | 53.66 ms | 2.6 MB | 550 B | 126.46 µs cold, 292 ns warm; 2.63 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.06 ms | 404.15 µs | 91.53 ms | 91.34 ms | 4.8 MB | 433 B | 5.05 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.03 ms | 527.67 µs | 244.65 ms | 248.11 ms | 46.5 MB | 460 B | 19.25 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.93 ms | 222.39 µs | 18.41 ms | 18.89 ms | 50.5 MB | 460 B | 18.68 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.49 ms | 121.81 µs | 18.29 ms | 18.59 ms | 50.2 MB | 460 B | 18.24 ms as a whole process |  |
| Seal | Normal | 66.14 ms | 797.54 µs | 65.44 ms | 69.97 ms | 3.2 MB | 3.5 KB | 332.79 µs cold, 292 ns warm; 2.87 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.67 ms | 231.62 µs | 69.22 ms | 69.89 ms | 4.8 MB | 3.0 KB | 5.21 ms as a whole process |  |
| Node `--jitless` | Normal | 178.07 ms | 4.02 ms | 173.40 ms | 181.95 ms | 48.1 MB | 3.1 KB | 18.54 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.88 ms | 25.35 µs | 3.84 ms | 5.96 ms | 52.0 MB | 3.1 KB | 18.67 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.89 ms | 35.94 µs | 3.84 ms | 5.66 ms | 52.0 MB | 3.1 KB | 18.81 ms as a whole process |  |
| Seal | Ceil.source | 59.38 ms | 253.62 µs | 59.05 ms | 59.44 ms | 3.2 MB | 3.5 KB | 315.79 µs cold, 375 ns warm; 2.66 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 55.37 ms | 422.90 µs | 54.84 ms | 55.86 ms | 3.1 MB | 3.5 KB | 314.92 µs cold, 417 ns warm; 2.95 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 46.76 ms | 328.41 µs | 46.28 ms | 47.10 ms | 3.0 MB | 3.5 KB | 305.62 µs cold, 333 ns warm; 2.75 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 47.78 ms | 324.95 µs | 47.35 ms | 48.13 ms | 3.0 MB | 3.5 KB | 295.83 µs cold, 292 ns warm; 2.88 ms as a whole process | 2.1 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.87 ms | 230.10 µs | 69.56 ms | 69.95 ms | 4.8 MB | 3.0 KB | 5.16 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.23 ms | 3.89 ms | 171.57 ms | 180.10 ms | 48.1 MB | 3.1 KB | 18.74 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.87 ms | 19.63 µs | 3.84 ms | 5.71 ms | 51.9 MB | 3.1 KB | 18.20 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.86 ms | 16.93 µs | 3.84 ms | 5.73 ms | 52.0 MB | 3.1 KB | 19.37 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.92 ms | 1.10 ms | 52.49 ms | 54.68 ms | 2.7 MB | 429 B | 153.79 µs cold, 292 ns warm; 2.76 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.65 ms | 283.12 µs | 45.31 ms | 45.29 ms | 4.8 MB | 366 B | 5.18 ms as a whole process |  |
| Node `--jitless` | Purpose | 365.81 ms | 6.58 ms | 359.00 ms | 374.33 ms | 48.7 MB | 398 B | 19.03 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 33.81 ms | 36.66 ms | 15.65 ms | 19.24 ms | 52.7 MB | 398 B | 18.76 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.61 ms | 56.78 µs | 15.52 ms | 19.32 ms | 55.4 MB | 398 B | 18.77 ms as a whole process |  |
| Seal | Normal | 19.27 ms | 77.54 µs | 19.14 ms | 19.51 ms | 2.9 MB | 1.2 KB | 178.17 µs cold, 333 ns warm; 2.65 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.91 ms | 338.16 µs | 27.45 ms | 27.70 ms | 4.8 MB | 981 B | 5.09 ms as a whole process |  |
| Node `--jitless` | Normal | 63.46 ms | 559.56 µs | 62.90 ms | 63.86 ms | 48.5 MB | 1.0 KB | 18.68 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.62 ms | 199.82 µs | 1.48 ms | 3.96 ms | 52.8 MB | 1.0 KB | 18.82 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.68 ms | 18.46 µs | 1.63 ms | 3.18 ms | 52.9 MB | 1.0 KB | 18.91 ms as a whole process |  |
| Seal | Ceil.source | 19.67 ms | 84.95 µs | 19.55 ms | 19.70 ms | 3.0 MB | 1.2 KB | 178.67 µs cold, 292 ns warm; 2.74 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.76 ms | 61.73 µs | 20.60 ms | 20.85 ms | 2.9 MB | 1.2 KB | 169.58 µs cold, 333 ns warm; 2.77 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 16.32 ms | 86.91 µs | 16.21 ms | 16.18 ms | 2.9 MB | 1.2 KB | 164.71 µs cold, 208 ns warm; 2.77 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.35 ms | 285.00 µs | 17.83 ms | 17.93 ms | 2.8 MB | 1.2 KB | 159.62 µs cold, 250 ns warm; 2.75 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.21 ms | 219.99 µs | 26.87 ms | 27.52 ms | 4.8 MB | 981 B | 5.10 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 62.79 ms | 199.10 µs | 62.47 ms | 63.19 ms | 48.4 MB | 1.0 KB | 19.42 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.61 ms | 177.96 µs | 1.45 ms | 3.31 ms | 52.8 MB | 1.0 KB | 18.79 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.73 ms | 26.25 µs | 1.69 ms | 3.29 ms | 52.8 MB | 1.0 KB | 18.92 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 111.77 ms | 1.33 ms | 110.40 ms | 115.07 ms | 2.7 MB | 632 B | 103.83 µs cold, 291 ns warm; 2.66 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 87.68 ms | 1.28 ms | 86.21 ms | 89.87 ms | 4.8 MB | 523 B | 5.09 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.08 ms | 941.69 µs | 144.80 ms | 145.12 ms | 43.4 MB | 568 B | 18.75 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 46.72 µs | 3.63 ms | 4.77 ms | 49.2 MB | 568 B | 18.26 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.69 ms | 35.92 µs | 3.64 ms | 4.49 ms | 49.3 MB | 568 B | 18.69 ms as a whole process |  |
| Seal | Normal | 115.54 ms | 217.86 µs | 115.21 ms | 115.32 ms | 2.9 MB | 1.8 KB | 164.58 µs cold, 250 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 222.57 ms | 3.12 ms | 214.31 ms | 222.86 ms | 4.9 MB | 1.7 KB | 5.09 ms as a whole process |  |
| Node `--jitless` | Normal | 157.05 ms | 1.30 ms | 154.49 ms | 158.05 ms | 45.4 MB | 1.7 KB | 18.74 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.22 ms | 89.85 µs | 14.03 ms | 15.60 ms | 51.9 MB | 1.7 KB | 18.34 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.89 ms | 44.39 µs | 14.82 ms | 16.64 ms | 52.1 MB | 1.7 KB | 19.02 ms as a whole process |  |
| Seal | Ceil.source | 118.36 ms | 731.08 µs | 117.45 ms | 117.95 ms | 2.9 MB | 1.8 KB | 168.75 µs cold, 292 ns warm; 2.80 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 125.41 ms | 14.60 ms | 120.52 ms | 121.20 ms | 2.9 MB | 1.8 KB | 167.38 µs cold, 416 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 101.34 ms | 284.11 µs | 100.99 ms | 102.41 ms | 2.8 MB | 1.8 KB | 152.04 µs cold, 250 ns warm; 2.62 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.17 ms | 85.36 µs | 102.05 ms | 102.25 ms | 2.8 MB | 1.8 KB | 161.71 µs cold, 250 ns warm; 2.64 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 216.22 ms | 2.06 ms | 210.41 ms | 217.35 ms | 4.8 MB | 1.7 KB | 5.21 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 156.25 ms | 1.23 ms | 153.70 ms | 159.43 ms | 45.4 MB | 1.7 KB | 19.35 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 13.97 ms | 102.51 µs | 13.83 ms | 15.68 ms | 51.9 MB | 1.7 KB | 18.52 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.83 ms | 116.42 µs | 14.66 ms | 16.17 ms | 52.1 MB | 1.7 KB | 18.62 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.52 ms | 383.61 µs | 95.00 ms | 93.55 ms | 3.8 MB | 569 B | 186.38 µs cold, 334 ns warm; 2.65 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.80 ms | 225.01 µs | 63.32 ms | 63.99 ms | 4.8 MB | 540 B | 5.05 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.88 ms | 1.10 ms | 242.26 ms | 246.63 ms | 48.6 MB | 514 B | 18.57 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 85.63 ms | 24.42 ms | 63.77 ms | 87.08 ms | 52.6 MB | 514 B | 18.65 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 64.23 ms | 518.45 µs | 63.48 ms | 92.07 ms | 52.7 MB | 514 B | 18.94 ms as a whole process |  |
| Seal | Normal | 24.81 ms | 136.79 µs | 24.55 ms | 24.93 ms | 3.8 MB | 538 B | 109.08 µs cold, 375 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.80 ms | 319.40 µs | 30.27 ms | 30.59 ms | 5.9 MB | 478 B | 5.11 ms as a whole process |  |
| Node `--jitless` | Normal | 52.31 ms | 397.09 µs | 51.46 ms | 53.36 ms | 56.5 MB | 477 B | 18.73 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.43 ms | 301.17 µs | 35.05 ms | 36.17 ms | 62.4 MB | 477 B | 18.37 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.38 ms | 269.14 µs | 35.02 ms | 36.26 ms | 88.0 MB | 477 B | 19.14 ms as a whole process |  |
| Seal | Ceil.source | 24.64 ms | 170.48 µs | 24.39 ms | 24.69 ms | 3.8 MB | 538 B | 108.12 µs cold, 375 ns warm; 2.62 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.84 ms | 166.50 µs | 25.57 ms | 25.86 ms | 3.9 MB | 538 B | 103.42 µs cold, 1.25 µs warm; 2.63 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.44 ms | 150.68 µs | 24.22 ms | 24.54 ms | 3.8 MB | 538 B | 103.71 µs cold, 292 ns warm; 2.54 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 24.27 ms | 206.93 µs | 23.76 ms | 24.43 ms | 3.8 MB | 538 B | 107.88 µs cold, 292 ns warm; 2.55 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.46 ms | 297.45 µs | 29.84 ms | 30.32 ms | 6.0 MB | 478 B | 4.98 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 51.89 ms | 222.03 µs | 51.53 ms | 52.80 ms | 56.5 MB | 477 B | 19.07 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.39 ms | 280.37 µs | 35.04 ms | 35.98 ms | 62.5 MB | 477 B | 19.12 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.47 ms | 211.92 µs | 35.02 ms | 35.83 ms | 87.9 MB | 477 B | 18.89 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 290.14 µs | 6.49 µs | 278.75 µs | 303.50 µs | 2.9 MB | 233 B | 97.21 µs cold, 333 ns warm; 2.51 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 143.99 ms | 697.40 µs | 142.72 ms | 144.64 ms | 9.7 MB | 195 B | 5.06 ms as a whole process |  |
| Node `--jitless` | Purpose | 364.13 µs | 143.68 µs | 239.71 µs | 1.03 ms | 52.5 MB | 223 B | 19.28 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 214.30 µs | 154.71 µs | 86.62 µs | 949.00 µs | 57.8 MB | 223 B | 17.94 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 131.87 µs | 120.67 µs | 54.96 µs | 900.12 µs | 57.8 MB | 223 B | 18.98 ms as a whole process |  |
| Seal | Normal | 348.03 µs | 10.66 µs | 333.00 µs | 369.75 µs | 3.1 MB | 182 B | 68.71 µs cold, 417 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.78 ms | 820.47 µs | 159.59 ms | 162.28 ms | 11.3 MB | 138 B | 5.07 ms as a whole process |  |
| Node `--jitless` | Normal | 497.75 µs | 325.83 µs | 268.88 µs | 1.12 ms | 61.4 MB | 155 B | 18.48 ms as a whole process |  |
| Node (JIT), from cold | Normal | 492.45 µs | 484.03 µs | 124.08 µs | 1.07 ms | 65.3 MB | 155 B | 19.03 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 280.08 µs | 332.02 µs | 123.17 µs | 1.09 ms | 69.5 MB | 155 B | 18.67 ms as a whole process |  |
| Seal | Ceil.source | 338.36 µs | 4.59 µs | 331.96 µs | 352.67 µs | 3.0 MB | 182 B | 72.12 µs cold, 375 ns warm; 2.64 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 335.85 µs | 4.82 µs | 330.54 µs | 349.75 µs | 3.1 MB | 182 B | 74.17 µs cold, 375 ns warm; 2.54 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 264.57 µs | 3.75 µs | 260.96 µs | 309.96 µs | 3.0 MB | 182 B | 64.12 µs cold, 458 ns warm; 2.56 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 267.39 µs | 5.47 µs | 262.12 µs | 291.29 µs | 3.0 MB | 182 B | 62.75 µs cold, 1.04 µs warm; 2.44 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 161.31 ms | 165.56 µs | 161.02 ms | 163.28 ms | 11.3 MB | 138 B | 4.93 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 504.86 µs | 342.46 µs | 267.38 µs | 1.10 ms | 61.4 MB | 155 B | 18.50 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 495.57 µs | 477.52 µs | 127.71 µs | 1.06 ms | 65.2 MB | 155 B | 18.73 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 281.29 µs | 343.01 µs | 122.58 µs | 1.08 ms | 69.2 MB | 155 B | 18.16 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 79.26 ms | 1.01 ms | 77.92 ms | 82.56 ms | 2.8 MB | 537 B | 167.71 µs cold, 208 ns warm; 2.70 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 94.57 ms | 234.89 µs | 94.07 ms | 94.04 ms | 4.8 MB | 411 B | 5.06 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.06 ms | 291.82 µs | 75.58 ms | 79.76 ms | 59.7 MB | 444 B | 18.87 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.48 ms | 281.08 µs | 10.21 ms | 12.96 ms | 65.8 MB | 444 B | 19.14 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.50 ms | 603.98 µs | 10.12 ms | 13.17 ms | 114.8 MB | 444 B | 18.66 ms as a whole process |  |
| Seal | Normal | 114.69 ms | 998.99 µs | 113.38 ms | 118.52 ms | 7.7 MB | 992 B | 147.21 µs cold, 250 ns warm; 2.68 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 128.30 ms | 2.10 ms | 125.56 ms | 130.24 ms | 38.6 MB | 799 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Normal | 113.67 ms | 2.30 ms | 111.22 ms | 115.99 ms | 213.4 MB | 878 B | 19.11 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.28 ms | 877.74 µs | 13.43 ms | 17.14 ms | 129.7 MB | 878 B | 18.94 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.30 ms | 39.80 µs | 13.23 ms | 17.14 ms | 224.0 MB | 878 B | 18.65 ms as a whole process |  |
| Seal | Ceil.source | 113.99 ms | 364.08 µs | 113.12 ms | 116.37 ms | 7.7 MB | 992 B | 144.04 µs cold, 458 ns warm; 2.60 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.55 ms | 595.90 µs | 113.32 ms | 117.28 ms | 7.7 MB | 992 B | 160.54 µs cold, 291 ns warm; 2.66 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 93.11 ms | 555.24 µs | 92.27 ms | 93.93 ms | 7.7 MB | 992 B | 135.00 µs cold, 250 ns warm; 2.70 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 97.54 ms | 1.75 ms | 95.51 ms | 102.28 ms | 7.6 MB | 992 B | 136.50 µs cold, 209 ns warm; 2.63 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 126.35 ms | 739.51 µs | 125.46 ms | 127.13 ms | 43.2 MB | 799 B | 5.14 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 114.42 ms | 1.97 ms | 112.94 ms | 117.58 ms | 213.3 MB | 878 B | 18.80 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.15 ms | 850.10 µs | 13.33 ms | 17.12 ms | 129.5 MB | 878 B | 18.78 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.32 ms | 79.22 µs | 13.21 ms | 17.20 ms | 225.1 MB | 878 B | 19.30 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 64.55 µs (±9.72 µs, least 54.46 µs) | 35.66 µs (±5.31 µs, least 30.12 µs) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.64 µs (±1.15 µs, least 8.25 µs) | 16.32 µs (±1.92 µs, least 14.29 µs) | 42 ns (±1 ns, least 39 ns) | 22 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 32.42 µs (±12.21 µs, least 19.71 µs) | 61.50 µs (±74.19 µs, least 35.92 µs) | 57 ns (±1 ns, least 55 ns) | 29 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 32.63 µs (±7.04 µs, least 18.75 µs) | 38.09 µs (±2.03 µs, least 35.50 µs) | 8 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 11.09 µs (±311 ns, least 10.69 µs) | 222 ns (±12 ns, least 206 ns) | 18 ns (±0 ns, least 18 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.87 µs (±208 ns, least 4.68 µs) | 184 ns (±7 ns, least 175 ns) | 33 ns (±1 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.14 µs (±26 ns, least 1.10 µs) | 677 ns (±12 ns, least 657 ns) | 50 ns (±1 ns, least 49 ns) | 23 ns (±0 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.09 µs (±23 ns, least 1.07 µs) | 667 ns (±20 ns, least 631 ns) | 22 ns (±1 ns, least 21 ns) | 6 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 11.03 µs (±234 ns, least 10.72 µs) | 217 ns (±13 ns, least 195 ns) | 18 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 11.22 µs (±291 ns, least 10.89 µs) | 218 ns (±6 ns, least 207 ns) | 18 ns (±0 ns, least 18 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 9.97 µs (±175 ns, least 9.72 µs) | 200 ns (±10 ns, least 188 ns) | 14 ns (±1 ns, least 14 ns) | 35 ns (±0 ns, least 35 ns) | 512 B | 4.9 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 10.35 µs (±326 ns, least 10.00 µs) | 205 ns (±8 ns, least 193 ns) | 15 ns (±0 ns, least 14 ns) | 35 ns (±0 ns, least 35 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.87 µs (±101 ns, least 4.68 µs) | 186 ns (±10 ns, least 175 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.15 µs (±30 ns, least 1.10 µs) | 678 ns (±15 ns, least 652 ns) | 51 ns (±1 ns, least 49 ns) | 23 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±17 ns, least 1.06 µs) | 667 ns (±12 ns, least 647 ns) | 22 ns (±0 ns, least 21 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 52.38 ms | 52.13 ms (-0.5%) |
| n-body, 100,000 steps | 67.63 ms | 60.60 ms (-10.4%) |
| spectral-norm, size 200 | 19.52 ms | 19.07 ms (-2.3%) |
| fannkuch-redux, 9 | 115.07 ms | 113.96 ms (-1.0%) |
| word count, 500,000 words | 24.67 ms | 24.95 ms (+1.1%) |
| text building, 20,000 joins | 345.32 µs | 342.72 µs (-0.8%) |
| binary-trees, depth 14 | 113.66 ms | 114.52 ms (+0.8%) |
