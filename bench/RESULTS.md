# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.62 2.16 1.97 }` before, `{ 2.51 2.48 2.14 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 52.50 ms | 71.74 ms | **met**: Seal takes 73% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 67.50 ms | 74.22 ms | **met**: Seal takes 91% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.10 ms | 27.24 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 114.08 ms | 213.34 ms | **met**: Seal takes 53% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 25.03 ms | 30.27 ms | **met**: Seal takes 83% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 380.67 µs | 162.93 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 115.46 ms | 128.33 ms | **met**: Seal takes 90% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.12 ms | 3.78 ms | **met**: Seal takes 56% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 19 ns | 36 ns | **met**: Seal takes 53% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 242 ns | 200 ns | **not met**: Seal takes 121% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 11.67 µs | 4.94 µs | **not met**: Seal takes 236% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 52.50 ms | 102.23 ms | **met**: Seal takes 51% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 67.50 ms | 180.26 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 25.03 ms | 51.95 ms | **met**: Seal takes 48% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 380.67 µs | 509.38 µs | **met**: Seal takes 75% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 19 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 183.27 ms | 873.02 µs | 181.93 ms | 188.49 ms | 2.7 MB | 395 B | 442.00 µs cold, 333 ns warm; 3.05 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 200.84 ms | 1.72 ms | 198.53 ms | 203.42 ms | 4.8 MB | 339 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Purpose | 326.61 ms | 1.23 ms | 324.71 ms | 331.44 ms | 43.5 MB | 362 B | 20.26 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 128.12 ms | 1.32 ms | 125.94 ms | 127.32 ms | 49.2 MB | 362 B | 20.73 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 125.25 ms | 1.27 ms | 123.69 ms | 128.17 ms | 49.1 MB | 362 B | 20.13 ms as a whole process |  |
| Seal | Normal | 52.50 ms | 350.66 µs | 51.87 ms | 52.40 ms | 2.6 MB | 169 B | 83.50 µs cold, 250 ns warm; 2.69 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 71.74 ms | 136.91 µs | 71.53 ms | 72.32 ms | 4.8 MB | 143 B | 5.43 ms as a whole process |  |
| Node `--jitless` | Normal | 102.23 ms | 14.52 ms | 96.45 ms | 103.49 ms | 43.4 MB | 137 B | 19.83 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.66 ms | 153.45 µs | 9.49 ms | 11.03 ms | 48.6 MB | 137 B | 19.57 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.86 ms | 56.92 µs | 9.73 ms | 10.89 ms | 48.6 MB | 137 B | 19.27 ms as a whole process |  |
| Seal | Ceil.source | 52.72 ms | 569.05 µs | 51.81 ms | 52.01 ms | 2.6 MB | 169 B | 82.25 µs cold, 292 ns warm; 2.75 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 55.64 ms | 6.22 ms | 52.48 ms | 75.78 ms | 2.6 MB | 169 B | 876.54 µs cold, 667 ns warm; 2.65 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 47.39 ms | 1.12 ms | 46.04 ms | 51.65 ms | 2.4 MB | 169 B | 281.12 µs cold, 208 ns warm; 2.81 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 47.83 ms | 5.27 ms | 44.03 ms | 70.08 ms | 2.5 MB | 169 B | 632.50 µs cold, 416 ns warm; 3.15 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.67 ms | 1.06 ms | 72.00 ms | 75.40 ms | 4.8 MB | 143 B | 5.40 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 97.60 ms | 2.59 ms | 94.19 ms | 95.41 ms | 43.4 MB | 137 B | 19.59 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.67 ms | 238.87 µs | 9.52 ms | 12.04 ms | 48.6 MB | 137 B | 20.37 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 11.85 ms | 1.35 ms | 10.32 ms | 11.49 ms | 48.6 MB | 137 B | 19.32 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.31 ms | 237.04 µs | 53.98 ms | 54.70 ms | 2.7 MB | 550 B | 112.71 µs cold, 292 ns warm; 2.95 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.34 ms | 441.23 µs | 91.84 ms | 92.42 ms | 4.8 MB | 433 B | 5.29 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.86 ms | 857.73 µs | 242.77 ms | 246.50 ms | 46.4 MB | 460 B | 19.21 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.12 ms | 181.35 µs | 18.71 ms | 19.05 ms | 50.2 MB | 460 B | 19.44 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.99 ms | 208.30 µs | 18.58 ms | 19.00 ms | 50.3 MB | 460 B | 18.64 ms as a whole process |  |
| Seal | Normal | 67.50 ms | 315.11 µs | 67.01 ms | 66.88 ms | 3.2 MB | 3.5 KB | 338.67 µs cold, 334 ns warm; 2.96 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 74.22 ms | 11.16 ms | 69.36 ms | 69.84 ms | 4.8 MB | 3.0 KB | 5.34 ms as a whole process |  |
| Node `--jitless` | Normal | 180.26 ms | 4.25 ms | 174.20 ms | 180.38 ms | 48.0 MB | 3.1 KB | 20.59 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.99 ms | 69.53 µs | 3.92 ms | 5.80 ms | 52.0 MB | 3.1 KB | 19.97 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.96 ms | 33.51 µs | 3.91 ms | 5.77 ms | 52.0 MB | 3.1 KB | 20.93 ms as a whole process |  |
| Seal | Ceil.source | 56.15 ms | 686.48 µs | 55.25 ms | 58.22 ms | 3.2 MB | 3.4 KB | 420.79 µs cold, 333 ns warm; 3.24 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 58.69 ms | 2.12 ms | 57.16 ms | 58.69 ms | 3.2 MB | 3.4 KB | 318.67 µs cold, 333 ns warm; 3.21 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 51.83 ms | 326.78 µs | 51.28 ms | 52.18 ms | 2.9 MB | 3.4 KB | 309.42 µs cold, 250 ns warm; 2.80 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 51.67 ms | 321.36 µs | 51.16 ms | 52.20 ms | 3.0 MB | 3.4 KB | 318.25 µs cold, 250 ns warm; 2.94 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 69.47 ms | 210.90 µs | 69.22 ms | 69.84 ms | 4.8 MB | 3.0 KB | 5.28 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 177.70 ms | 3.28 ms | 173.02 ms | 179.10 ms | 48.1 MB | 3.1 KB | 19.53 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.89 ms | 20.60 µs | 3.86 ms | 5.78 ms | 52.0 MB | 3.1 KB | 19.92 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.89 ms | 24.04 µs | 3.87 ms | 5.78 ms | 52.0 MB | 3.1 KB | 19.67 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 57.39 ms | 350.16 µs | 56.92 ms | 58.15 ms | 2.7 MB | 429 B | 96.17 µs cold, 292 ns warm; 2.76 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.62 ms | 326.95 µs | 45.23 ms | 45.39 ms | 4.8 MB | 366 B | 5.35 ms as a whole process |  |
| Node `--jitless` | Purpose | 370.96 ms | 6.36 ms | 365.66 ms | 375.82 ms | 48.5 MB | 398 B | 18.65 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.45 ms | 37.79 ms | 15.79 ms | 19.31 ms | 52.7 MB | 398 B | 19.21 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.85 ms | 355.35 µs | 14.39 ms | 19.76 ms | 55.3 MB | 398 B | 18.74 ms as a whole process |  |
| Seal | Normal | 20.10 ms | 31.42 µs | 20.05 ms | 20.16 ms | 3.0 MB | 1.2 KB | 192.50 µs cold, 250 ns warm; 2.62 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.24 ms | 289.47 µs | 26.90 ms | 27.42 ms | 4.8 MB | 981 B | 5.12 ms as a whole process |  |
| Node `--jitless` | Normal | 64.49 ms | 171.13 µs | 64.17 ms | 64.31 ms | 48.4 MB | 1.0 KB | 19.40 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.62 ms | 197.05 µs | 1.49 ms | 3.23 ms | 52.8 MB | 1.0 KB | 19.19 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.67 ms | 30.80 µs | 1.60 ms | 3.11 ms | 52.8 MB | 1.0 KB | 19.09 ms as a whole process |  |
| Seal | Ceil.source | 20.11 ms | 79.08 µs | 19.95 ms | 20.23 ms | 3.0 MB | 1.2 KB | 179.83 µs cold, 250 ns warm; 2.81 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.27 ms | 39.15 µs | 20.20 ms | 20.36 ms | 3.0 MB | 1.2 KB | 183.50 µs cold, 291 ns warm; 2.73 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.24 ms | 55.02 µs | 19.15 ms | 19.20 ms | 2.8 MB | 1.2 KB | 158.79 µs cold, 208 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 19.02 ms | 39.27 µs | 18.96 ms | 18.96 ms | 2.9 MB | 1.2 KB | 163.92 µs cold, 208 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.15 ms | 370.15 µs | 26.55 ms | 26.89 ms | 4.9 MB | 981 B | 5.17 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 63.10 ms | 399.08 µs | 62.63 ms | 64.18 ms | 48.4 MB | 1.0 KB | 18.87 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.63 ms | 200.50 µs | 1.52 ms | 3.31 ms | 52.8 MB | 1.0 KB | 18.75 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.62 ms | 11.88 µs | 1.60 ms | 3.64 ms | 52.9 MB | 1.0 KB | 19.20 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 96.17 ms | 542.57 µs | 95.31 ms | 95.19 ms | 2.7 MB | 632 B | 110.29 µs cold, 250 ns warm; 2.95 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 89.24 ms | 845.60 µs | 87.75 ms | 91.35 ms | 4.8 MB | 523 B | 5.59 ms as a whole process |  |
| Node `--jitless` | Purpose | 147.66 ms | 1.09 ms | 145.33 ms | 149.89 ms | 43.3 MB | 568 B | 19.82 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.72 ms | 46.07 µs | 3.66 ms | 4.52 ms | 49.3 MB | 568 B | 20.15 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.72 ms | 45.04 µs | 3.66 ms | 4.54 ms | 49.3 MB | 568 B | 19.70 ms as a whole process |  |
| Seal | Normal | 114.08 ms | 783.30 µs | 112.66 ms | 115.72 ms | 2.9 MB | 1.8 KB | 177.58 µs cold, 292 ns warm; 2.90 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 213.34 ms | 2.23 ms | 208.72 ms | 214.64 ms | 4.8 MB | 1.7 KB | 5.25 ms as a whole process |  |
| Node `--jitless` | Normal | 156.51 ms | 699.80 µs | 155.45 ms | 158.08 ms | 45.5 MB | 1.7 KB | 19.60 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.57 ms | 302.30 µs | 14.10 ms | 15.57 ms | 51.9 MB | 1.7 KB | 18.90 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 16.86 ms | 4.02 ms | 14.00 ms | 15.89 ms | 52.2 MB | 1.7 KB | 21.17 ms as a whole process |  |
| Seal | Ceil.source | 113.36 ms | 695.45 µs | 112.32 ms | 114.18 ms | 3.0 MB | 1.8 KB | 174.12 µs cold, 333 ns warm; 2.61 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 113.50 ms | 1.09 ms | 112.03 ms | 112.68 ms | 3.0 MB | 1.8 KB | 177.46 µs cold, 375 ns warm; 2.87 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 105.64 ms | 1.49 ms | 104.07 ms | 105.41 ms | 2.8 MB | 1.8 KB | 158.71 µs cold, 250 ns warm; 2.75 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 103.59 ms | 801.44 µs | 102.79 ms | 103.55 ms | 2.8 MB | 1.8 KB | 166.75 µs cold, 292 ns warm; 2.99 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 227.76 ms | 23.79 ms | 211.03 ms | 217.30 ms | 4.9 MB | 1.7 KB | 5.88 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 163.02 ms | 1.49 ms | 159.70 ms | 174.40 ms | 45.4 MB | 1.7 KB | 19.27 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.31 ms | 273.84 µs | 14.05 ms | 15.91 ms | 52.0 MB | 1.7 KB | 21.48 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 16.28 ms | 1.33 ms | 14.59 ms | 16.40 ms | 52.2 MB | 1.7 KB | 21.51 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 107.06 ms | 11.75 ms | 95.73 ms | 112.16 ms | 3.9 MB | 569 B | 131.75 µs cold, 333 ns warm; 2.68 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.37 ms | 300.79 µs | 62.91 ms | 63.47 ms | 4.8 MB | 540 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Purpose | 237.57 ms | 1.15 ms | 235.67 ms | 241.87 ms | 48.7 MB | 514 B | 19.29 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 87.39 ms | 24.84 ms | 64.67 ms | 90.94 ms | 52.6 MB | 514 B | 20.97 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 67.06 ms | 1.22 ms | 66.17 ms | 92.26 ms | 52.7 MB | 514 B | 20.18 ms as a whole process |  |
| Seal | Normal | 25.03 ms | 145.22 µs | 24.84 ms | 24.96 ms | 3.9 MB | 538 B | 117.38 µs cold, 375 ns warm; 2.59 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 30.27 ms | 237.18 µs | 29.88 ms | 30.16 ms | 6.0 MB | 478 B | 5.09 ms as a whole process |  |
| Node `--jitless` | Normal | 51.95 ms | 273.12 µs | 51.59 ms | 53.31 ms | 56.5 MB | 477 B | 19.21 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.38 ms | 189.97 µs | 35.18 ms | 35.91 ms | 62.5 MB | 477 B | 18.58 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.04 ms | 274.41 µs | 34.68 ms | 35.94 ms | 88.1 MB | 477 B | 18.99 ms as a whole process |  |
| Seal | Ceil.source | 24.93 ms | 200.65 µs | 24.66 ms | 24.54 ms | 3.9 MB | 538 B | 111.04 µs cold, 417 ns warm; 2.67 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.37 ms | 258.15 µs | 25.11 ms | 25.31 ms | 3.9 MB | 538 B | 132.62 µs cold, 417 ns warm; 2.74 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 24.33 ms | 286.29 µs | 23.80 ms | 24.25 ms | 3.8 MB | 538 B | 111.29 µs cold, 250 ns warm; 2.66 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 22.52 ms | 274.76 µs | 22.18 ms | 23.01 ms | 3.9 MB | 538 B | 107.50 µs cold, 375 ns warm; 2.61 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.23 ms | 87.86 µs | 30.07 ms | 30.27 ms | 6.0 MB | 478 B | 4.97 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.15 ms | 248.30 µs | 51.84 ms | 53.27 ms | 56.4 MB | 477 B | 19.03 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.38 ms | 220.05 µs | 35.15 ms | 36.08 ms | 62.5 MB | 477 B | 19.30 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.75 ms | 433.22 µs | 35.06 ms | 36.16 ms | 88.0 MB | 477 B | 18.97 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 310.47 µs | 7.67 µs | 300.21 µs | 345.00 µs | 3.0 MB | 233 B | 70.96 µs cold, 375 ns warm; 2.69 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 144.76 ms | 1.56 ms | 143.86 ms | 146.31 ms | 10.8 MB | 195 B | 5.59 ms as a whole process |  |
| Node `--jitless` | Purpose | 374.84 µs | 141.15 µs | 246.21 µs | 1.13 ms | 52.5 MB | 223 B | 20.53 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 240.44 µs | 179.68 µs | 89.83 µs | 915.79 µs | 57.8 MB | 223 B | 20.62 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 143.89 µs | 133.40 µs | 56.79 µs | 892.62 µs | 57.8 MB | 223 B | 19.34 ms as a whole process |  |
| Seal | Normal | 380.67 µs | 7.76 µs | 369.88 µs | 397.67 µs | 3.1 MB | 182 B | 72.75 µs cold, 416 ns warm; 2.85 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 162.93 ms | 1.59 ms | 161.18 ms | 167.05 ms | 11.5 MB | 138 B | 5.42 ms as a whole process |  |
| Node `--jitless` | Normal | 509.38 µs | 353.37 µs | 270.88 µs | 1.33 ms | 61.5 MB | 155 B | 19.69 ms as a whole process |  |
| Node (JIT), from cold | Normal | 514.63 µs | 497.63 µs | 134.54 µs | 1.05 ms | 65.2 MB | 155 B | 19.21 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 286.79 µs | 322.52 µs | 130.62 µs | 1.08 ms | 69.4 MB | 155 B | 19.39 ms as a whole process |  |
| Seal | Ceil.source | 375.69 µs | 5.34 µs | 369.58 µs | 402.79 µs | 3.1 MB | 182 B | 94.25 µs cold, 417 ns warm; 2.85 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 375.10 µs | 16.48 µs | 359.67 µs | 405.00 µs | 3.1 MB | 182 B | 74.17 µs cold, 417 ns warm; 2.75 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 283.33 µs | 6.97 µs | 273.42 µs | 309.21 µs | 3.0 MB | 182 B | 65.54 µs cold, 292 ns warm; 2.76 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 289.77 µs | 10.44 µs | 277.50 µs | 332.12 µs | 3.0 MB | 182 B | 63.79 µs cold, 250 ns warm; 2.71 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 164.20 ms | 581.37 µs | 163.56 ms | 166.39 ms | 11.4 MB | 138 B | 5.09 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 532.81 µs | 342.26 µs | 276.71 µs | 1.09 ms | 61.5 MB | 155 B | 20.02 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 511.56 µs | 512.84 µs | 123.21 µs | 1.11 ms | 65.3 MB | 155 B | 20.21 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 282.40 µs | 327.61 µs | 124.96 µs | 1.08 ms | 69.4 MB | 155 B | 19.12 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 77.22 ms | 700.80 µs | 76.39 ms | 76.61 ms | 2.9 MB | 537 B | 104.00 µs cold, 208 ns warm; 2.55 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 95.40 ms | 640.68 µs | 94.49 ms | 95.65 ms | 4.8 MB | 411 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.30 ms | 368.04 µs | 76.66 ms | 79.16 ms | 59.6 MB | 444 B | 19.19 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.46 ms | 281.85 µs | 10.19 ms | 13.07 ms | 65.7 MB | 444 B | 19.21 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.43 ms | 673.77 µs | 9.99 ms | 13.04 ms | 114.7 MB | 444 B | 19.04 ms as a whole process |  |
| Seal | Normal | 115.46 ms | 606.72 µs | 114.50 ms | 116.42 ms | 7.8 MB | 992 B | 158.12 µs cold, 291 ns warm; 2.72 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 128.33 ms | 1.21 ms | 126.41 ms | 131.55 ms | 42.7 MB | 799 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Normal | 117.95 ms | 2.05 ms | 115.31 ms | 121.82 ms | 213.4 MB | 878 B | 18.52 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.30 ms | 947.94 µs | 13.46 ms | 17.02 ms | 130.0 MB | 878 B | 18.99 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.31 ms | 115.43 µs | 13.05 ms | 17.02 ms | 223.2 MB | 878 B | 19.11 ms as a whole process |  |
| Seal | Ceil.source | 112.28 ms | 764.85 µs | 111.12 ms | 113.58 ms | 7.8 MB | 992 B | 152.04 µs cold, 250 ns warm; 2.72 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.13 ms | 513.69 µs | 113.10 ms | 114.30 ms | 7.8 MB | 992 B | 149.25 µs cold, 292 ns warm; 2.92 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 92.00 ms | 296.13 µs | 91.28 ms | 93.29 ms | 7.6 MB | 992 B | 136.42 µs cold, 166 ns warm; 2.56 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 92.65 ms | 361.64 µs | 92.07 ms | 94.25 ms | 7.6 MB | 992 B | 141.04 µs cold, 209 ns warm; 2.69 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 133.11 ms | 5.83 ms | 127.55 ms | 130.60 ms | 41.6 MB | 799 B | 5.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 117.46 ms | 2.55 ms | 113.94 ms | 119.80 ms | 213.3 MB | 878 B | 20.52 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.42 ms | 842.27 µs | 13.47 ms | 17.36 ms | 129.7 MB | 878 B | 19.54 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.18 ms | 843.89 µs | 13.41 ms | 17.47 ms | 224.1 MB | 878 B | 19.57 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 86.53 µs (±9.72 µs, least 76.29 µs) | 43.95 µs (±4.90 µs, least 37.96 µs) | 17 ns (±0 ns, least 16 ns) | 38 ns (±0 ns, least 37 ns) | 512 B | 2.8 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.85 µs (±902 ns, least 9.04 µs) | 18.95 µs (±1.12 µs, least 17.58 µs) | 42 ns (±2 ns, least 40 ns) | 22 ns (±1 ns, least 20 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| Node `--jitless` | Purpose | 37.36 µs (±11.69 µs, least 25.04 µs) | 41.05 µs (±3.64 µs, least 37.29 µs) | 58 ns (±1 ns, least 56 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 44.08 µs (±17.93 µs, least 20.21 µs) | 52.15 µs (±16.06 µs, least 40.17 µs) | 10 ns (±4 ns, least 9 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.0 MB | the engine's own heap |
| Seal | Normal | 11.67 µs (±502 ns, least 11.21 µs) | 242 ns (±16 ns, least 224 ns) | 19 ns (±1 ns, least 18 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 5.1 MB | counted by Seal |
| Luau (interpreter) | Normal | 4.94 µs (±100 ns, least 4.79 µs) | 200 ns (±8 ns, least 185 ns) | 36 ns (±3 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±20 ns, least 1.14 µs) | 703 ns (±40 ns, least 649 ns) | 52 ns (±2 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.17 µs (±43 ns, least 1.11 µs) | 719 ns (±63 ns, least 666 ns) | 23 ns (±1 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 11.27 µs (±188 ns, least 10.94 µs) | 230 ns (±10 ns, least 219 ns) | 18 ns (±1 ns, least 17 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 5.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 11.68 µs (±364 ns, least 11.31 µs) | 237 ns (±15 ns, least 211 ns) | 19 ns (±1 ns, least 19 ns) | 37 ns (±1 ns, least 37 ns) | 512 B | 5.1 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 10.22 µs (±253 ns, least 9.93 µs) | 226 ns (±13 ns, least 211 ns) | 16 ns (±1 ns, least 15 ns) | 36 ns (±1 ns, least 35 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 10.44 µs (±370 ns, least 10.04 µs) | 215 ns (±14 ns, least 202 ns) | 15 ns (±0 ns, least 15 ns) | 35 ns (±0 ns, least 35 ns) | 512 B | 5.0 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.15 µs (±183 ns, least 4.88 µs) | 193 ns (±7 ns, least 181 ns) | 35 ns (±5 ns, least 33 ns) | 19 ns (±1 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.18 µs (±36 ns, least 1.13 µs) | 687 ns (±32 ns, least 650 ns) | 52 ns (±2 ns, least 50 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.13 µs (±39 ns, least 1.10 µs) | 698 ns (±25 ns, least 660 ns) | 23 ns (±1 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 52.51 ms | 51.00 ms (-2.9%) |
| n-body, 100,000 steps | 68.01 ms | 59.93 ms (-11.9%) |
| spectral-norm, size 200 | 20.27 ms | 19.73 ms (-2.7%) |
| fannkuch-redux, 9 | 114.33 ms | 114.51 ms (+0.2%) |
| word count, 500,000 words | 25.04 ms | 24.60 ms (-1.8%) |
| text building, 20,000 joins | 385.55 µs | 383.35 µs (-0.6%) |
| binary-trees, depth 14 | 116.49 ms | 113.29 ms (-2.7%) |
