# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 1.89 2.16 2.15 }` before, `{ 1.23 1.79 1.99 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 51.64 ms | 70.20 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 64.63 ms | 69.39 ms | **met**: Seal takes 93% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 19.94 ms | 26.75 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 114.64 ms | 216.48 ms | **met**: Seal takes 53% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 23.46 ms | 29.66 ms | **met**: Seal takes 79% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 343.35 µs | 160.85 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 111.28 ms | 127.41 ms | **met**: Seal takes 87% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.07 ms | 3.58 ms | **met**: Seal takes 58% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 19 ns | 34 ns | **met**: Seal takes 55% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 224 ns | 188 ns | **not met**: Seal takes 119% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 12.47 µs | 4.96 µs | **not met**: Seal takes 251% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 51.64 ms | 91.45 ms | **met**: Seal takes 56% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 64.63 ms | 175.29 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 23.46 ms | 51.47 ms | **met**: Seal takes 46% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 343.35 µs | 497.02 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 19 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 141.58 ms | 659.76 µs | 140.70 ms | 147.90 ms | 2.8 MB | 395 B | 582.42 µs cold, 375 ns warm; 2.71 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 196.10 ms | 150.44 µs | 195.95 ms | 196.05 ms | 4.8 MB | 339 B | 5.16 ms as a whole process |  |
| Node `--jitless` | Purpose | 320.08 ms | 686.57 µs | 318.98 ms | 318.60 ms | 43.4 MB | 362 B | 19.10 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.34 ms | 706.46 µs | 123.44 ms | 124.42 ms | 49.2 MB | 362 B | 19.50 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 123.38 ms | 300.79 µs | 122.97 ms | 124.07 ms | 49.1 MB | 362 B | 19.99 ms as a whole process |  |
| Seal | Normal | 51.64 ms | 153.19 µs | 51.36 ms | 51.64 ms | 2.7 MB | 169 B | 92.38 µs cold, 250 ns warm; 2.59 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 70.20 ms | 94.57 µs | 70.07 ms | 70.34 ms | 4.8 MB | 143 B | 4.96 ms as a whole process |  |
| Node `--jitless` | Normal | 91.45 ms | 403.49 µs | 90.97 ms | 92.11 ms | 43.3 MB | 137 B | 19.93 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.43 ms | 81.65 µs | 9.36 ms | 11.65 ms | 48.5 MB | 137 B | 20.66 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 10.41 ms | 1.40 ms | 9.46 ms | 11.53 ms | 48.5 MB | 137 B | 19.73 ms as a whole process |  |
| Seal | Ceil.source | 51.67 ms | 209.35 µs | 51.28 ms | 51.71 ms | 2.7 MB | 169 B | 95.62 µs cold, 250 ns warm; 2.62 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 54.20 ms | 643.43 µs | 53.71 ms | 79.96 ms | 2.8 MB | 169 B | 1.01 ms cold, 750 ns warm; 2.68 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 46.58 ms | 127.80 µs | 46.36 ms | 72.14 ms | 2.5 MB | 169 B | 721.96 µs cold, 709 ns warm; 2.57 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 42.80 ms | 123.43 µs | 42.63 ms | 69.08 ms | 2.6 MB | 169 B | 799.04 µs cold, 666 ns warm; 2.57 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.42 ms | 190.98 µs | 73.03 ms | 73.10 ms | 4.8 MB | 143 B | 4.94 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 95.01 ms | 129.36 µs | 94.69 ms | 95.03 ms | 43.4 MB | 137 B | 19.94 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.84 ms | 729.80 µs | 9.37 ms | 10.62 ms | 48.6 MB | 137 B | 19.20 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.31 ms | 363.94 µs | 9.44 ms | 11.22 ms | 48.6 MB | 137 B | 19.50 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.41 ms | 87.39 µs | 52.27 ms | 52.32 ms | 2.8 MB | 550 B | 128.71 µs cold, 250 ns warm; 2.67 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 91.52 ms | 259.75 µs | 91.11 ms | 90.79 ms | 4.9 MB | 433 B | 5.02 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.09 ms | 941.74 µs | 241.99 ms | 243.67 ms | 46.5 MB | 460 B | 19.67 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.86 ms | 180.49 µs | 18.38 ms | 18.85 ms | 50.3 MB | 460 B | 19.52 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.68 ms | 197.70 µs | 18.36 ms | 18.99 ms | 50.3 MB | 460 B | 19.55 ms as a whole process |  |
| Seal | Normal | 64.63 ms | 734.86 µs | 63.54 ms | 63.80 ms | 3.4 MB | 3.5 KB | 351.42 µs cold, 333 ns warm; 2.82 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.39 ms | 1.02 ms | 68.72 ms | 69.21 ms | 4.8 MB | 3.0 KB | 5.00 ms as a whole process |  |
| Node `--jitless` | Normal | 175.29 ms | 4.91 ms | 169.39 ms | 181.70 ms | 48.1 MB | 3.1 KB | 19.65 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 26.85 µs | 3.83 ms | 5.65 ms | 52.0 MB | 3.1 KB | 19.44 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.85 ms | 12.81 µs | 3.83 ms | 5.69 ms | 52.0 MB | 3.1 KB | 19.08 ms as a whole process |  |
| Seal | Ceil.source | 56.19 ms | 378.46 µs | 55.55 ms | 58.06 ms | 3.3 MB | 3.4 KB | 330.50 µs cold, 333 ns warm; 2.82 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 58.05 ms | 567.30 µs | 56.91 ms | 57.59 ms | 3.4 MB | 3.4 KB | 356.58 µs cold, 375 ns warm; 2.95 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 49.65 ms | 302.89 µs | 49.16 ms | 49.88 ms | 3.1 MB | 3.4 KB | 336.12 µs cold, 292 ns warm; 2.88 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.27 ms | 175.18 µs | 49.01 ms | 49.38 ms | 3.2 MB | 3.4 KB | 346.46 µs cold, 875 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.03 ms | 188.93 µs | 68.76 ms | 69.10 ms | 4.8 MB | 3.0 KB | 5.07 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 175.29 ms | 3.21 ms | 171.81 ms | 178.56 ms | 48.2 MB | 3.1 KB | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.85 ms | 26.33 µs | 3.81 ms | 5.61 ms | 52.0 MB | 3.1 KB | 19.52 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.84 ms | 14.05 µs | 3.82 ms | 5.64 ms | 51.8 MB | 3.1 KB | 19.61 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 56.10 ms | 1.53 ms | 53.26 ms | 55.75 ms | 2.9 MB | 429 B | 105.50 µs cold, 292 ns warm; 2.60 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.22 ms | 276.97 µs | 44.83 ms | 44.97 ms | 4.8 MB | 366 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Purpose | 358.12 ms | 4.92 ms | 353.09 ms | 366.40 ms | 48.6 MB | 398 B | 19.99 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 33.74 ms | 36.64 ms | 15.53 ms | 19.14 ms | 52.7 MB | 398 B | 18.96 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.26 ms | 19.22 µs | 14.23 ms | 19.26 ms | 55.3 MB | 398 B | 19.59 ms as a whole process |  |
| Seal | Normal | 19.94 ms | 18.53 µs | 19.91 ms | 19.96 ms | 3.1 MB | 1.2 KB | 190.79 µs cold, 292 ns warm; 2.86 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 26.75 ms | 231.76 µs | 26.44 ms | 27.02 ms | 4.8 MB | 981 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Normal | 63.72 ms | 382.99 µs | 63.23 ms | 63.36 ms | 48.5 MB | 1.0 KB | 19.92 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.61 ms | 218.82 µs | 1.47 ms | 3.48 ms | 52.8 MB | 1.0 KB | 19.71 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.65 ms | 19.39 µs | 1.61 ms | 3.18 ms | 53.0 MB | 1.0 KB | 19.89 ms as a whole process |  |
| Seal | Ceil.source | 19.97 ms | 33.12 µs | 19.92 ms | 19.91 ms | 3.2 MB | 1.2 KB | 190.88 µs cold, 292 ns warm; 2.72 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.03 ms | 39.11 µs | 19.98 ms | 19.99 ms | 3.1 MB | 1.2 KB | 192.33 µs cold, 292 ns warm; 2.78 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 18.97 ms | 59.84 µs | 18.89 ms | 18.98 ms | 2.9 MB | 1.2 KB | 168.21 µs cold, 292 ns warm; 2.58 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.58 ms | 14.57 µs | 18.56 ms | 18.52 ms | 2.9 MB | 1.2 KB | 165.33 µs cold, 250 ns warm; 2.73 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.20 ms | 324.87 µs | 26.87 ms | 28.18 ms | 4.8 MB | 981 B | 5.07 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.92 ms | 581.76 µs | 63.75 ms | 65.15 ms | 48.4 MB | 1.0 KB | 19.33 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.63 ms | 199.91 µs | 1.49 ms | 3.21 ms | 52.8 MB | 1.0 KB | 19.09 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.63 ms | 10.31 µs | 1.61 ms | 3.52 ms | 52.8 MB | 1.0 KB | 20.06 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 94.88 ms | 615.96 µs | 94.04 ms | 94.09 ms | 2.8 MB | 632 B | 120.88 µs cold, 250 ns warm; 2.53 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 86.82 ms | 1.18 ms | 85.44 ms | 87.39 ms | 4.9 MB | 523 B | 5.08 ms as a whole process |  |
| Node `--jitless` | Purpose | 143.88 ms | 551.68 µs | 143.09 ms | 147.63 ms | 43.3 MB | 568 B | 19.01 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.66 ms | 42.28 µs | 3.61 ms | 4.46 ms | 49.3 MB | 568 B | 19.61 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.63 ms | 24.14 µs | 3.61 ms | 4.44 ms | 49.3 MB | 568 B | 20.53 ms as a whole process |  |
| Seal | Normal | 114.64 ms | 183.78 µs | 114.34 ms | 114.71 ms | 3.0 MB | 1.8 KB | 205.58 µs cold, 459 ns warm; 2.64 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 216.48 ms | 1.76 ms | 211.51 ms | 217.20 ms | 4.8 MB | 1.7 KB | 5.16 ms as a whole process |  |
| Node `--jitless` | Normal | 158.66 ms | 445.65 µs | 157.91 ms | 158.76 ms | 45.4 MB | 1.7 KB | 19.58 ms as a whole process |  |
| Node (JIT), from cold | Normal | 13.97 ms | 90.83 µs | 13.86 ms | 15.60 ms | 51.9 MB | 1.7 KB | 19.19 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.62 ms | 122.73 µs | 14.45 ms | 16.14 ms | 52.0 MB | 1.7 KB | 19.19 ms as a whole process |  |
| Seal | Ceil.source | 111.04 ms | 491.87 µs | 110.28 ms | 111.74 ms | 3.0 MB | 1.8 KB | 190.62 µs cold, 292 ns warm; 2.76 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 112.00 ms | 123.83 µs | 111.83 ms | 112.16 ms | 3.0 MB | 1.8 KB | 188.08 µs cold, 417 ns warm; 2.72 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 102.68 ms | 97.85 µs | 102.53 ms | 102.87 ms | 2.8 MB | 1.8 KB | 163.08 µs cold, 250 ns warm; 2.74 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 104.70 ms | 174.43 µs | 104.46 ms | 104.70 ms | 2.9 MB | 1.8 KB | 163.58 µs cold, 292 ns warm; 2.90 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 213.39 ms | 2.14 ms | 207.61 ms | 214.10 ms | 4.8 MB | 1.7 KB | 5.15 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 155.29 ms | 348.13 µs | 154.87 ms | 157.82 ms | 45.3 MB | 1.7 KB | 19.61 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 13.79 ms | 83.82 µs | 13.64 ms | 15.51 ms | 51.9 MB | 1.7 KB | 18.90 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.77 ms | 72.49 µs | 14.64 ms | 16.33 ms | 52.2 MB | 1.7 KB | 19.90 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.68 ms | 947.47 µs | 94.81 ms | 94.79 ms | 4.0 MB | 569 B | 120.42 µs cold, 334 ns warm; 2.77 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 62.58 ms | 149.47 µs | 62.34 ms | 62.53 ms | 4.8 MB | 540 B | 5.06 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.33 ms | 3.02 ms | 240.30 ms | 245.07 ms | 48.7 MB | 514 B | 19.88 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 87.37 ms | 24.14 ms | 66.34 ms | 90.54 ms | 52.6 MB | 514 B | 19.40 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.59 ms | 273.79 µs | 67.25 ms | 89.96 ms | 52.6 MB | 514 B | 19.52 ms as a whole process |  |
| Seal | Normal | 23.46 ms | 83.09 µs | 23.35 ms | 23.39 ms | 4.0 MB | 538 B | 125.58 µs cold, 333 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 29.66 ms | 107.08 µs | 29.53 ms | 29.82 ms | 6.0 MB | 478 B | 5.07 ms as a whole process |  |
| Node `--jitless` | Normal | 51.47 ms | 194.64 µs | 51.04 ms | 52.39 ms | 56.5 MB | 477 B | 19.90 ms as a whole process |  |
| Node (JIT), from cold | Normal | 34.85 ms | 254.68 µs | 34.56 ms | 35.66 ms | 62.3 MB | 477 B | 19.37 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 34.83 ms | 190.11 µs | 34.61 ms | 35.35 ms | 88.3 MB | 477 B | 19.65 ms as a whole process |  |
| Seal | Ceil.source | 23.52 ms | 61.48 µs | 23.40 ms | 23.60 ms | 4.0 MB | 538 B | 120.79 µs cold, 333 ns warm; 2.69 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 24.78 ms | 139.56 µs | 24.59 ms | 25.19 ms | 4.0 MB | 538 B | 124.00 µs cold, 459 ns warm; 2.68 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.78 ms | 173.10 µs | 23.57 ms | 24.36 ms | 3.9 MB | 538 B | 122.75 µs cold, 292 ns warm; 2.45 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.23 ms | 379.44 µs | 22.43 ms | 23.31 ms | 4.0 MB | 538 B | 125.17 µs cold, 333 ns warm; 2.52 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 29.83 ms | 100.81 µs | 29.73 ms | 29.94 ms | 5.9 MB | 478 B | 5.07 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 50.68 ms | 312.03 µs | 50.41 ms | 52.02 ms | 56.5 MB | 477 B | 19.88 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 34.69 ms | 228.23 µs | 34.38 ms | 35.74 ms | 62.5 MB | 477 B | 19.03 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 36.16 ms | 470.54 µs | 35.02 ms | 35.08 ms | 88.6 MB | 477 B | 19.37 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 308.44 µs | 2.15 µs | 305.71 µs | 316.54 µs | 3.0 MB | 233 B | 77.08 µs cold, 417 ns warm; 2.57 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 143.79 ms | 327.67 µs | 143.40 ms | 145.26 ms | 9.7 MB | 195 B | 5.05 ms as a whole process |  |
| Node `--jitless` | Purpose | 360.94 µs | 140.16 µs | 234.50 µs | 1.06 ms | 52.6 MB | 223 B | 19.77 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 214.20 µs | 153.93 µs | 82.38 µs | 875.33 µs | 57.8 MB | 223 B | 19.54 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 132.62 µs | 115.49 µs | 55.92 µs | 912.50 µs | 57.9 MB | 223 B | 19.57 ms as a whole process |  |
| Seal | Normal | 343.35 µs | 1.92 µs | 341.25 µs | 368.42 µs | 3.2 MB | 182 B | 82.08 µs cold, 375 ns warm; 2.62 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 160.85 ms | 230.62 µs | 160.54 ms | 162.77 ms | 11.8 MB | 138 B | 4.99 ms as a whole process |  |
| Node `--jitless` | Normal | 497.02 µs | 330.89 µs | 266.92 µs | 1.13 ms | 61.5 MB | 155 B | 19.84 ms as a whole process |  |
| Node (JIT), from cold | Normal | 513.11 µs | 470.76 µs | 141.00 µs | 1.03 ms | 65.1 MB | 155 B | 19.89 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 295.97 µs | 328.97 µs | 141.88 µs | 1.04 ms | 69.4 MB | 155 B | 19.46 ms as a whole process |  |
| Seal | Ceil.source | 342.75 µs | 2.58 µs | 339.92 µs | 365.58 µs | 3.2 MB | 182 B | 85.50 µs cold, 417 ns warm; 2.64 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 341.50 µs | 2.65 µs | 338.75 µs | 370.58 µs | 3.2 MB | 182 B | 77.83 µs cold, 375 ns warm; 2.55 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 278.76 µs | 2.36 µs | 275.75 µs | 306.71 µs | 3.0 MB | 182 B | 66.75 µs cold, 291 ns warm; 2.53 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 275.79 µs | 4.96 µs | 272.71 µs | 307.88 µs | 3.1 MB | 182 B | 70.29 µs cold, 333 ns warm; 2.39 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 160.79 ms | 158.89 µs | 160.55 ms | 162.77 ms | 11.3 MB | 138 B | 5.01 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 505.55 µs | 345.71 µs | 266.21 µs | 1.12 ms | 61.4 MB | 155 B | 18.94 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 490.22 µs | 471.74 µs | 121.67 µs | 1.07 ms | 65.2 MB | 155 B | 19.32 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 282.00 µs | 325.37 µs | 127.96 µs | 1.05 ms | 69.4 MB | 155 B | 19.28 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.09 ms | 480.50 µs | 74.30 ms | 75.76 ms | 3.0 MB | 537 B | 118.88 µs cold, 250 ns warm; 2.60 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 94.33 ms | 198.12 µs | 93.87 ms | 94.35 ms | 4.8 MB | 411 B | 4.98 ms as a whole process |  |
| Node `--jitless` | Purpose | 74.17 ms | 177.76 µs | 73.85 ms | 78.01 ms | 59.7 MB | 444 B | 19.64 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.49 ms | 308.52 µs | 10.17 ms | 13.68 ms | 65.8 MB | 444 B | 20.11 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.40 ms | 750.22 µs | 9.90 ms | 12.92 ms | 114.8 MB | 444 B | 19.63 ms as a whole process |  |
| Seal | Normal | 111.28 ms | 1.91 ms | 109.30 ms | 115.40 ms | 7.9 MB | 992 B | 167.08 µs cold, 292 ns warm; 2.65 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 127.41 ms | 882.22 µs | 126.50 ms | 128.21 ms | 37.5 MB | 799 B | 5.11 ms as a whole process |  |
| Node `--jitless` | Normal | 111.00 ms | 1.73 ms | 109.45 ms | 113.57 ms | 213.0 MB | 878 B | 19.45 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.10 ms | 873.32 µs | 13.20 ms | 16.87 ms | 129.7 MB | 878 B | 19.55 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.34 ms | 43.99 µs | 13.25 ms | 16.89 ms | 224.1 MB | 878 B | 19.59 ms as a whole process |  |
| Seal | Ceil.source | 110.10 ms | 428.36 µs | 109.19 ms | 111.17 ms | 7.9 MB | 992 B | 163.67 µs cold, 250 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 112.04 ms | 497.24 µs | 111.27 ms | 113.51 ms | 8.0 MB | 992 B | 187.21 µs cold, 292 ns warm; 2.64 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 94.62 ms | 848.87 µs | 93.46 ms | 94.34 ms | 7.7 MB | 992 B | 138.54 µs cold, 167 ns warm; 2.65 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 90.78 ms | 415.89 µs | 89.91 ms | 92.17 ms | 7.7 MB | 992 B | 136.75 µs cold, 291 ns warm; 2.71 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 132.64 ms | 705.76 µs | 131.77 ms | 132.39 ms | 44.6 MB | 799 B | 5.05 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 112.34 ms | 1.85 ms | 110.69 ms | 114.98 ms | 213.2 MB | 878 B | 19.72 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.10 ms | 863.19 µs | 13.23 ms | 16.79 ms | 129.7 MB | 878 B | 19.91 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.21 ms | 81.48 µs | 13.10 ms | 16.93 ms | 223.9 MB | 878 B | 20.24 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 68.38 µs (±8.18 µs, least 62.29 µs) | 39.67 µs (±5.29 µs, least 34.08 µs) | 16 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 2.9 MB | counted by Seal |
| Luau (interpreter) | Purpose | 10.45 µs (±1.37 µs, least 9.12 µs) | 16.58 µs (±987 ns, least 14.67 µs) | 41 ns (±1 ns, least 39 ns) | 20 ns (±1 ns, least 18 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 35.44 µs (±4.36 µs, least 23.12 µs) | 39.26 µs (±2.05 µs, least 36.92 µs) | 58 ns (±2 ns, least 56 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 30.69 µs (±7.78 µs, least 19.17 µs) | 36.96 µs (±1.53 µs, least 34.67 µs) | 8 ns (±1 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 12.47 µs (±464 ns, least 12.06 µs) | 224 ns (±15 ns, least 206 ns) | 19 ns (±4 ns, least 17 ns) | 37 ns (±2 ns, least 36 ns) | 512 B | 5.4 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.96 µs (±280 ns, least 4.64 µs) | 188 ns (±5 ns, least 178 ns) | 34 ns (±3 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.15 µs (±23 ns, least 1.11 µs) | 658 ns (±17 ns, least 630 ns) | 50 ns (±1 ns, least 48 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.10 µs (±26 ns, least 1.08 µs) | 660 ns (±13 ns, least 644 ns) | 22 ns (±0 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 12.25 µs (±149 ns, least 12.06 µs) | 218 ns (±11 ns, least 204 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±0 ns, least 36 ns) | 512 B | 5.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 12.57 µs (±274 ns, least 12.34 µs) | 221 ns (±8 ns, least 211 ns) | 18 ns (±1 ns, least 18 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.4 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 11.33 µs (±399 ns, least 11.03 µs) | 202 ns (±3 ns, least 197 ns) | 15 ns (±0 ns, least 14 ns) | 36 ns (±0 ns, least 36 ns) | 512 B | 5.2 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 11.68 µs (±249 ns, least 11.49 µs) | 203 ns (±9 ns, least 187 ns) | 15 ns (±0 ns, least 14 ns) | 36 ns (±1 ns, least 35 ns) | 512 B | 5.3 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.92 µs (±210 ns, least 4.71 µs) | 187 ns (±12 ns, least 172 ns) | 34 ns (±2 ns, least 32 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.14 µs (±14 ns, least 1.12 µs) | 669 ns (±14 ns, least 637 ns) | 50 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.10 µs (±14 ns, least 1.08 µs) | 673 ns (±15 ns, least 648 ns) | 22 ns (±0 ns, least 21 ns) | 6 ns (±0 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 51.35 ms | 48.64 ms (-5.3%) |
| n-body, 100,000 steps | 66.24 ms | 61.79 ms (-6.7%) |
| spectral-norm, size 200 | 19.79 ms | 20.55 ms (+3.9%) |
| fannkuch-redux, 9 | 111.82 ms | 113.41 ms (+1.4%) |
| word count, 500,000 words | 23.68 ms | 24.03 ms (+1.5%) |
| text building, 20,000 joins | 344.95 µs | 346.42 µs (+0.4%) |
| binary-trees, depth 14 | 110.92 ms | 110.73 ms (-0.2%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 226.13 ms | 1.46 ms | 225.13 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 10.90 ms | 44.42 µs | 10.84 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.45 ms | 49.01 µs | 4.35 ms | 3.8 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.41 ms | 11.42 µs | 1.40 ms | 3.6 MB |
