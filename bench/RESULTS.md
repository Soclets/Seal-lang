# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.97 2.98 2.66 }` before, `{ 2.02 2.60 2.56 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 55.71 ms | 77.69 ms | **met**: Seal takes 72% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 68.90 ms | 70.07 ms | **met**: Seal takes 98% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.08 ms | 27.21 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 116.45 ms | 212.46 ms | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 24.25 ms | 30.73 ms | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 358.71 µs | 165.01 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 115.18 ms | 130.70 ms | **met**: Seal takes 88% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.98 ms | 3.54 ms | **met**: Seal takes 56% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 18 ns | 34 ns | **met**: Seal takes 52% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 229 ns | 183 ns | **not met**: Seal takes 125% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 12.56 µs | 4.96 µs | **not met**: Seal takes 253% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 55.71 ms | 95.92 ms | **met**: Seal takes 58% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 68.90 ms | 178.27 ms | **met**: Seal takes 39% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 24.25 ms | 52.77 ms | **met**: Seal takes 46% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 358.71 µs | 518.56 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 18 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 166.69 ms | 1.28 ms | 165.27 ms | 184.02 ms | 2.8 MB | 395 B | 671.17 µs cold, 458 ns warm; 2.87 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 204.84 ms | 11.83 ms | 197.56 ms | 198.21 ms | 4.8 MB | 339 B | 5.23 ms as a whole process |  |
| Node `--jitless` | Purpose | 326.19 ms | 2.77 ms | 323.94 ms | 325.92 ms | 43.4 MB | 362 B | 20.87 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 127.75 ms | 2.33 ms | 125.30 ms | 127.89 ms | 49.2 MB | 362 B | 20.04 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 128.07 ms | 9.27 ms | 123.56 ms | 125.96 ms | 49.2 MB | 362 B | 20.25 ms as a whole process |  |
| Seal | Normal | 55.71 ms | 848.48 µs | 54.62 ms | 53.94 ms | 2.7 MB | 169 B | 91.71 µs cold, 292 ns warm; 3.18 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 77.69 ms | 7.57 ms | 73.78 ms | 74.19 ms | 4.8 MB | 143 B | 5.60 ms as a whole process |  |
| Node `--jitless` | Normal | 95.92 ms | 998.49 µs | 94.61 ms | 97.75 ms | 43.3 MB | 137 B | 20.10 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.53 ms | 234.57 µs | 9.41 ms | 11.51 ms | 48.5 MB | 137 B | 19.89 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.49 ms | 49.69 µs | 9.44 ms | 10.70 ms | 48.6 MB | 137 B | 19.93 ms as a whole process |  |
| Seal | Ceil.source | 62.41 ms | 14.64 ms | 53.43 ms | 54.48 ms | 2.8 MB | 169 B | 102.62 µs cold, 292 ns warm; 2.91 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 56.06 ms | 1.33 ms | 54.24 ms | 79.07 ms | 2.8 MB | 169 B | 1.01 ms cold, 458 ns warm; 2.66 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 47.12 ms | 238.61 µs | 46.60 ms | 71.96 ms | 2.5 MB | 169 B | 729.08 µs cold, 625 ns warm; 2.74 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.24 ms | 146.71 µs | 42.99 ms | 49.99 ms | 2.6 MB | 169 B | 481.62 µs cold, 375 ns warm; 2.86 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 72.24 ms | 766.01 µs | 71.21 ms | 72.88 ms | 4.8 MB | 143 B | 5.23 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 94.80 ms | 879.73 µs | 93.29 ms | 95.18 ms | 43.3 MB | 137 B | 19.85 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 10.06 ms | 485.30 µs | 9.48 ms | 12.13 ms | 48.5 MB | 137 B | 19.93 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.99 ms | 193.41 µs | 9.72 ms | 10.43 ms | 48.6 MB | 137 B | 20.33 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 53.14 ms | 153.22 µs | 52.97 ms | 55.49 ms | 2.8 MB | 550 B | 129.71 µs cold, 250 ns warm; 2.71 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 94.32 ms | 4.28 ms | 91.74 ms | 91.21 ms | 4.8 MB | 433 B | 5.30 ms as a whole process |  |
| Node `--jitless` | Purpose | 255.86 ms | 19.21 ms | 244.48 ms | 253.50 ms | 46.5 MB | 460 B | 20.34 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.37 ms | 143.45 µs | 19.23 ms | 19.38 ms | 50.2 MB | 460 B | 20.37 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.08 ms | 104.54 µs | 18.92 ms | 19.30 ms | 50.3 MB | 460 B | 21.30 ms as a whole process |  |
| Seal | Normal | 68.90 ms | 737.14 µs | 67.86 ms | 69.34 ms | 3.4 MB | 3.5 KB | 414.08 µs cold, 375 ns warm; 2.99 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.07 ms | 508.19 µs | 69.46 ms | 69.83 ms | 4.8 MB | 3.0 KB | 5.47 ms as a whole process |  |
| Node `--jitless` | Normal | 178.27 ms | 5.47 ms | 170.77 ms | 182.64 ms | 48.1 MB | 3.1 KB | 21.79 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.95 ms | 58.94 µs | 3.90 ms | 5.81 ms | 52.0 MB | 3.1 KB | 20.55 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.98 ms | 82.79 µs | 3.88 ms | 5.74 ms | 52.0 MB | 3.1 KB | 21.08 ms as a whole process |  |
| Seal | Ceil.source | 56.89 ms | 524.11 µs | 55.96 ms | 57.10 ms | 3.4 MB | 3.4 KB | 325.46 µs cold, 458 ns warm; 2.90 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 57.83 ms | 371.82 µs | 57.30 ms | 57.54 ms | 3.3 MB | 3.4 KB | 330.46 µs cold, 375 ns warm; 2.92 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 51.11 ms | 656.93 µs | 49.99 ms | 51.23 ms | 3.1 MB | 3.4 KB | 349.79 µs cold, 333 ns warm; 3.05 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.49 ms | 545.53 µs | 49.93 ms | 49.95 ms | 3.1 MB | 3.4 KB | 321.17 µs cold, 292 ns warm; 3.07 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.75 ms | 468.43 µs | 68.93 ms | 70.55 ms | 4.8 MB | 3.0 KB | 5.39 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.96 ms | 3.53 ms | 172.46 ms | 180.60 ms | 48.1 MB | 3.1 KB | 20.14 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 21.15 µs | 3.85 ms | 5.59 ms | 51.9 MB | 3.1 KB | 19.48 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.89 ms | 22.33 µs | 3.84 ms | 5.71 ms | 52.0 MB | 3.1 KB | 20.49 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.90 ms | 984.74 µs | 56.53 ms | 58.39 ms | 2.8 MB | 429 B | 104.67 µs cold, 250 ns warm; 2.79 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.62 ms | 435.11 µs | 44.99 ms | 45.14 ms | 4.8 MB | 366 B | 5.37 ms as a whole process |  |
| Node `--jitless` | Purpose | 366.25 ms | 7.60 ms | 358.54 ms | 382.94 ms | 48.6 MB | 398 B | 20.19 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 35.09 ms | 37.15 ms | 16.07 ms | 20.89 ms | 52.7 MB | 398 B | 20.20 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.40 ms | 75.14 µs | 14.32 ms | 20.31 ms | 55.4 MB | 398 B | 19.67 ms as a whole process |  |
| Seal | Normal | 20.08 ms | 45.88 µs | 20.04 ms | 20.05 ms | 3.2 MB | 1.2 KB | 200.46 µs cold, 333 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.21 ms | 955.62 µs | 26.51 ms | 27.17 ms | 4.8 MB | 981 B | 5.11 ms as a whole process |  |
| Node `--jitless` | Normal | 63.60 ms | 307.65 µs | 63.31 ms | 63.76 ms | 48.4 MB | 1.0 KB | 20.06 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.60 ms | 154.44 µs | 1.50 ms | 3.61 ms | 52.8 MB | 1.0 KB | 20.06 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.66 ms | 39.28 µs | 1.62 ms | 3.19 ms | 52.9 MB | 1.0 KB | 20.15 ms as a whole process |  |
| Seal | Ceil.source | 20.09 ms | 49.61 µs | 19.97 ms | 20.15 ms | 3.1 MB | 1.2 KB | 199.83 µs cold, 292 ns warm; 2.84 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.21 ms | 50.82 µs | 20.13 ms | 20.17 ms | 3.2 MB | 1.2 KB | 195.38 µs cold, 333 ns warm; 2.83 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.05 ms | 50.30 µs | 18.99 ms | 19.05 ms | 2.9 MB | 1.2 KB | 170.83 µs cold, 333 ns warm; 2.74 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.84 ms | 73.46 µs | 18.72 ms | 18.88 ms | 2.9 MB | 1.2 KB | 184.04 µs cold, 209 ns warm; 2.74 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.54 ms | 613.08 µs | 26.60 ms | 27.52 ms | 4.8 MB | 981 B | 5.20 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.94 ms | 582.33 µs | 63.18 ms | 65.39 ms | 48.5 MB | 1.0 KB | 19.98 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.65 ms | 219.83 µs | 1.52 ms | 3.61 ms | 52.9 MB | 1.0 KB | 20.63 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.62 ms | 38.18 µs | 1.58 ms | 3.66 ms | 52.9 MB | 1.0 KB | 19.71 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 96.25 ms | 475.43 µs | 95.61 ms | 95.86 ms | 2.8 MB | 632 B | 115.71 µs cold, 291 ns warm; 2.98 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 90.42 ms | 1.11 ms | 89.00 ms | 89.02 ms | 4.8 MB | 523 B | 5.66 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.98 ms | 806.36 µs | 147.38 ms | 151.47 ms | 43.5 MB | 568 B | 20.24 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 31.15 µs | 3.64 ms | 4.45 ms | 49.3 MB | 568 B | 20.76 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.77 ms | 61.18 µs | 3.71 ms | 4.62 ms | 49.5 MB | 568 B | 20.90 ms as a whole process |  |
| Seal | Normal | 116.45 ms | 781.76 µs | 115.49 ms | 116.73 ms | 3.1 MB | 1.8 KB | 201.00 µs cold, 417 ns warm; 2.89 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 212.46 ms | 2.08 ms | 207.07 ms | 215.46 ms | 4.8 MB | 1.7 KB | 5.42 ms as a whole process |  |
| Node `--jitless` | Normal | 160.71 ms | 4.21 ms | 156.54 ms | 159.53 ms | 45.3 MB | 1.7 KB | 21.34 ms as a whole process |  |
| Node (JIT), from cold | Normal | 15.08 ms | 233.18 µs | 14.69 ms | 16.73 ms | 51.9 MB | 1.7 KB | 22.21 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.60 ms | 202.03 µs | 14.36 ms | 16.24 ms | 52.1 MB | 1.7 KB | 23.53 ms as a whole process |  |
| Seal | Ceil.source | 115.50 ms | 917.99 µs | 113.88 ms | 115.54 ms | 3.1 MB | 1.8 KB | 215.71 µs cold, 417 ns warm; 2.93 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 115.81 ms | 1.16 ms | 114.42 ms | 114.92 ms | 3.1 MB | 1.8 KB | 186.12 µs cold, 292 ns warm; 3.08 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 106.42 ms | 1.07 ms | 105.29 ms | 105.78 ms | 2.8 MB | 1.8 KB | 167.42 µs cold, 291 ns warm; 3.05 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.89 ms | 445.27 µs | 103.32 ms | 103.20 ms | 2.9 MB | 1.8 KB | 193.79 µs cold, 333 ns warm; 2.71 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 214.09 ms | 2.20 ms | 208.71 ms | 214.58 ms | 4.9 MB | 1.7 KB | 5.29 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 160.36 ms | 888.22 µs | 159.14 ms | 159.76 ms | 45.3 MB | 1.7 KB | 21.13 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.39 ms | 131.82 µs | 14.14 ms | 15.71 ms | 52.0 MB | 1.7 KB | 19.68 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.14 ms | 192.80 µs | 13.80 ms | 15.55 ms | 52.1 MB | 1.7 KB | 21.15 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.50 ms | 1.85 ms | 95.37 ms | 95.40 ms | 4.0 MB | 569 B | 130.71 µs cold, 375 ns warm; 2.76 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.68 ms | 661.91 µs | 62.64 ms | 63.02 ms | 4.8 MB | 540 B | 5.16 ms as a whole process |  |
| Node `--jitless` | Purpose | 276.45 ms | 64.95 ms | 242.87 ms | 243.41 ms | 48.6 MB | 514 B | 20.85 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 96.68 ms | 31.82 ms | 66.73 ms | 93.12 ms | 52.5 MB | 514 B | 21.25 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.95 ms | 695.80 µs | 66.51 ms | 94.02 ms | 52.5 MB | 514 B | 20.77 ms as a whole process |  |
| Seal | Normal | 24.25 ms | 357.16 µs | 23.98 ms | 24.37 ms | 4.0 MB | 538 B | 130.33 µs cold, 375 ns warm; 3.70 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.73 ms | 207.32 µs | 30.30 ms | 51.36 ms | 6.0 MB | 478 B | 5.34 ms as a whole process |  |
| Node `--jitless` | Normal | 52.77 ms | 1.35 ms | 51.19 ms | 53.50 ms | 56.5 MB | 477 B | 20.87 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.25 ms | 381.10 µs | 34.79 ms | 37.31 ms | 62.5 MB | 477 B | 20.54 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 36.14 ms | 732.01 µs | 35.21 ms | 36.82 ms | 88.2 MB | 477 B | 20.12 ms as a whole process |  |
| Seal | Ceil.source | 24.43 ms | 111.67 µs | 24.28 ms | 24.08 ms | 4.1 MB | 538 B | 129.92 µs cold, 292 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.32 ms | 366.23 µs | 24.98 ms | 24.85 ms | 4.1 MB | 538 B | 142.38 µs cold, 334 ns warm; 2.87 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.40 ms | 235.37 µs | 23.93 ms | 25.33 ms | 3.9 MB | 538 B | 129.88 µs cold, 291 ns warm; 2.81 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.71 ms | 239.51 µs | 23.32 ms | 22.98 ms | 4.0 MB | 538 B | 124.50 µs cold, 333 ns warm; 2.92 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.59 ms | 180.15 µs | 30.39 ms | 31.01 ms | 5.9 MB | 478 B | 5.20 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 57.47 ms | 5.42 ms | 51.90 ms | 53.59 ms | 56.5 MB | 477 B | 21.50 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.68 ms | 258.63 µs | 35.19 ms | 37.25 ms | 62.5 MB | 477 B | 19.84 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.68 ms | 316.74 µs | 35.26 ms | 36.91 ms | 87.9 MB | 477 B | 20.65 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 316.28 µs | 6.71 µs | 308.83 µs | 348.62 µs | 3.1 MB | 233 B | 80.08 µs cold, 3.12 µs warm; 2.65 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 147.75 ms | 1.12 ms | 146.34 ms | 147.67 ms | 10.1 MB | 195 B | 5.56 ms as a whole process |  |
| Node `--jitless` | Purpose | 381.90 µs | 146.75 µs | 247.75 µs | 1.20 ms | 52.5 MB | 223 B | 22.21 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 222.27 µs | 157.04 µs | 86.92 µs | 1.03 ms | 57.7 MB | 223 B | 21.21 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 167.69 µs | 165.13 µs | 56.00 µs | 912.79 µs | 58.4 MB | 223 B | 20.90 ms as a whole process |  |
| Seal | Normal | 358.71 µs | 6.89 µs | 350.25 µs | 377.17 µs | 3.2 MB | 182 B | 86.79 µs cold, 416 ns warm; 2.69 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 165.01 ms | 1.12 ms | 163.12 ms | 165.58 ms | 11.5 MB | 138 B | 5.34 ms as a whole process |  |
| Node `--jitless` | Normal | 518.56 µs | 331.34 µs | 259.71 µs | 1.13 ms | 61.3 MB | 155 B | 19.90 ms as a whole process |  |
| Node (JIT), from cold | Normal | 490.46 µs | 472.21 µs | 124.25 µs | 1.07 ms | 65.2 MB | 155 B | 19.49 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 279.93 µs | 333.77 µs | 123.50 µs | 1.08 ms | 69.2 MB | 155 B | 20.79 ms as a whole process |  |
| Seal | Ceil.source | 359.35 µs | 9.28 µs | 347.04 µs | 392.00 µs | 3.2 MB | 182 B | 84.38 µs cold, 333 ns warm; 2.71 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 354.50 µs | 5.44 µs | 348.29 µs | 384.29 µs | 3.2 MB | 182 B | 83.33 µs cold, 416 ns warm; 2.51 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 277.06 µs | 4.00 µs | 272.46 µs | 295.42 µs | 3.0 MB | 182 B | 68.42 µs cold, 417 ns warm; 2.51 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 280.91 µs | 4.98 µs | 276.12 µs | 309.25 µs | 3.1 MB | 182 B | 71.25 µs cold, 333 ns warm; 2.64 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 163.67 ms | 656.16 µs | 162.90 ms | 165.20 ms | 12.5 MB | 138 B | 5.29 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 516.78 µs | 348.34 µs | 272.88 µs | 1.13 ms | 61.4 MB | 155 B | 20.96 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 498.57 µs | 491.17 µs | 129.92 µs | 1.10 ms | 65.2 MB | 155 B | 20.26 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 275.73 µs | 329.49 µs | 122.67 µs | 1.06 ms | 69.2 MB | 155 B | 20.38 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 74.04 ms | 1.28 ms | 72.97 ms | 76.73 ms | 3.0 MB | 537 B | 115.92 µs cold, 208 ns warm; 2.99 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 94.83 ms | 423.28 µs | 93.87 ms | 95.22 ms | 4.8 MB | 411 B | 5.21 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.16 ms | 470.93 µs | 75.64 ms | 81.24 ms | 59.6 MB | 444 B | 19.77 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.47 ms | 263.95 µs | 10.22 ms | 13.05 ms | 65.7 MB | 444 B | 19.73 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.71 ms | 706.40 µs | 10.24 ms | 13.06 ms | 114.8 MB | 444 B | 20.71 ms as a whole process |  |
| Seal | Normal | 115.18 ms | 893.94 µs | 113.76 ms | 116.34 ms | 8.0 MB | 992 B | 176.29 µs cold, 375 ns warm; 2.87 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 130.70 ms | 1.31 ms | 128.86 ms | 132.74 ms | 41.3 MB | 799 B | 5.17 ms as a whole process |  |
| Node `--jitless` | Normal | 117.63 ms | 1.95 ms | 115.76 ms | 119.06 ms | 213.4 MB | 878 B | 19.62 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.19 ms | 928.80 µs | 13.22 ms | 17.26 ms | 129.4 MB | 878 B | 20.05 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.42 ms | 44.77 µs | 13.36 ms | 17.18 ms | 223.5 MB | 878 B | 20.02 ms as a whole process |  |
| Seal | Ceil.source | 114.52 ms | 1.00 ms | 112.83 ms | 114.69 ms | 8.0 MB | 992 B | 187.38 µs cold, 375 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 113.72 ms | 786.67 µs | 112.41 ms | 115.75 ms | 8.0 MB | 992 B | 165.83 µs cold, 334 ns warm; 2.80 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 94.75 ms | 940.61 µs | 93.97 ms | 95.58 ms | 7.7 MB | 992 B | 141.04 µs cold, 250 ns warm; 2.75 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 95.05 ms | 1.44 ms | 92.74 ms | 99.05 ms | 7.8 MB | 992 B | 145.75 µs cold, 208 ns warm; 2.79 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 131.49 ms | 2.74 ms | 127.80 ms | 130.25 ms | 42.5 MB | 799 B | 5.32 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 119.01 ms | 1.77 ms | 117.72 ms | 124.01 ms | 213.4 MB | 878 B | 20.01 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.27 ms | 898.56 µs | 13.41 ms | 17.10 ms | 129.7 MB | 878 B | 19.97 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.46 ms | 71.83 µs | 13.35 ms | 17.16 ms | 224.1 MB | 878 B | 20.61 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 78.17 µs (±8.79 µs, least 67.58 µs) | 41.40 µs (±3.52 µs, least 37.67 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 2.9 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.71 µs (±762 ns, least 9.08 µs) | 17.03 µs (±1.20 µs, least 15.54 µs) | 41 ns (±1 ns, least 40 ns) | 22 ns (±1 ns, least 20 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 31.48 µs (±7.97 µs, least 19.54 µs) | 38.95 µs (±1.82 µs, least 36.17 µs) | 57 ns (±1 ns, least 56 ns) | 30 ns (±0 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 34.22 µs (±3.50 µs, least 24.54 µs) | 39.87 µs (±3.89 µs, least 35.50 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 12.56 µs (±342 ns, least 12.12 µs) | 229 ns (±14 ns, least 213 ns) | 18 ns (±1 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.3 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.96 µs (±193 ns, least 4.72 µs) | 183 ns (±7 ns, least 173 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±24 ns, least 1.14 µs) | 676 ns (±13 ns, least 657 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap |
| Node (JIT) | Normal | 1.12 µs (±29 ns, least 1.08 µs) | 679 ns (±23 ns, least 650 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 12.58 µs (±431 ns, least 12.18 µs) | 222 ns (±10 ns, least 212 ns) | 18 ns (±1 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 12.73 µs (±196 ns, least 12.51 µs) | 225 ns (±7 ns, least 212 ns) | 19 ns (±0 ns, least 18 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 5.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 11.43 µs (±300 ns, least 11.20 µs) | 206 ns (±6 ns, least 199 ns) | 15 ns (±0 ns, least 15 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 5.3 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 11.71 µs (±164 ns, least 11.38 µs) | 205 ns (±8 ns, least 195 ns) | 15 ns (±0 ns, least 15 ns) | 36 ns (±0 ns, least 35 ns) | 512 B | 5.3 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.05 µs (±147 ns, least 4.87 µs) | 194 ns (±16 ns, least 177 ns) | 34 ns (±2 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.16 µs (±39 ns, least 1.13 µs) | 672 ns (±19 ns, least 645 ns) | 51 ns (±1 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±18 ns, least 1.09 µs) | 670 ns (±10 ns, least 656 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 53.60 ms | 51.05 ms (-4.7%) |
| n-body, 100,000 steps | 67.13 ms | 64.06 ms (-4.6%) |
| spectral-norm, size 200 | 20.13 ms | 19.46 ms (-3.3%) |
| fannkuch-redux, 9 | 113.77 ms | 115.06 ms (+1.1%) |
| word count, 500,000 words | 23.98 ms | 24.23 ms (+1.1%) |
| text building, 20,000 joins | 361.71 µs | 350.91 µs (-3.0%) |
| binary-trees, depth 14 | 112.44 ms | 112.92 ms (+0.4%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 225.66 ms | 1.04 ms | 223.84 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 11.04 ms | 54.58 µs | 10.92 ms | 2.8 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.45 ms | 17.74 µs | 4.42 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.43 ms | 11.50 µs | 1.41 ms | 3.6 MB |
