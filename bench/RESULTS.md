# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.29 2.40 2.35 }` before, `{ 1.45 2.05 2.21 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 48.77 ms | 72.23 ms | **met**: Seal takes 68% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.99 ms | 70.04 ms | **met**: Seal takes 96% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.42 ms | 27.29 ms | **met**: Seal takes 75% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 122.96 ms | 217.02 ms | **met**: Seal takes 57% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 23.97 ms | 31.53 ms | **met**: Seal takes 76% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 352.55 µs | 163.40 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 113.69 ms | 133.54 ms | **met**: Seal takes 85% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.92 ms | 3.76 ms | **met**: Seal takes 51% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 36 ns | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 192 ns | 203 ns | **met**: Seal takes 95% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 8.58 µs | 5.11 µs | **not met**: Seal takes 168% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 48.77 ms | 96.62 ms | **met**: Seal takes 50% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.99 ms | 184.77 ms | **met**: Seal takes 36% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 23.97 ms | 53.14 ms | **met**: Seal takes 45% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 352.55 µs | 523.93 µs | **met**: Seal takes 67% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 37 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 136.97 ms | 3.14 ms | 135.23 ms | 155.01 ms | 2.8 MB | 395 B | 614.96 µs cold, 458 ns warm; 2.96 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 204.50 ms | 16.10 ms | 196.19 ms | 205.45 ms | 4.8 MB | 339 B | 5.27 ms as a whole process |  |
| Node `--jitless` | Purpose | 328.55 ms | 4.22 ms | 325.11 ms | 366.09 ms | 43.4 MB | 362 B | 19.78 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 138.66 ms | 17.00 ms | 127.91 ms | 130.40 ms | 49.5 MB | 362 B | 21.56 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 126.96 ms | 323.52 µs | 126.53 ms | 129.11 ms | 49.1 MB | 362 B | 19.27 ms as a whole process |  |
| Seal | Normal | 48.77 ms | 278.30 µs | 48.27 ms | 48.51 ms | 2.7 MB | 169 B | 85.33 µs cold, 250 ns warm; 2.71 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 72.23 ms | 239.78 µs | 71.93 ms | 73.00 ms | 4.8 MB | 143 B | 5.25 ms as a whole process |  |
| Node `--jitless` | Normal | 96.62 ms | 2.92 ms | 94.44 ms | 96.68 ms | 43.3 MB | 137 B | 20.76 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.90 ms | 286.07 µs | 9.61 ms | 10.61 ms | 48.5 MB | 137 B | 18.80 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.85 ms | 98.68 µs | 9.75 ms | 10.84 ms | 48.6 MB | 137 B | 18.83 ms as a whole process |  |
| Seal | Ceil.source | 48.77 ms | 270.19 µs | 48.40 ms | 48.46 ms | 2.7 MB | 169 B | 85.33 µs cold, 334 ns warm; 2.71 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 53.30 ms | 799.74 µs | 52.46 ms | 76.85 ms | 2.7 MB | 169 B | 950.04 µs cold, 708 ns warm; 2.63 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 44.08 ms | 223.00 µs | 43.83 ms | 68.18 ms | 2.5 MB | 169 B | 719.29 µs cold, 542 ns warm; 2.85 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.66 ms | 142.42 µs | 43.43 ms | 66.67 ms | 2.5 MB | 169 B | 742.71 µs cold, 542 ns warm; 2.75 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 73.60 ms | 1.07 ms | 72.16 ms | 73.46 ms | 4.8 MB | 143 B | 5.11 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 101.11 ms | 4.72 ms | 96.18 ms | 96.22 ms | 43.4 MB | 137 B | 19.79 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.63 ms | 273.28 µs | 9.43 ms | 10.02 ms | 48.5 MB | 137 B | 18.86 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 10.36 ms | 190.71 µs | 9.88 ms | 11.74 ms | 48.7 MB | 137 B | 18.95 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 55.35 ms | 317.16 µs | 54.97 ms | 56.47 ms | 2.8 MB | 550 B | 128.46 µs cold, 208 ns warm; 2.75 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.75 ms | 633.34 µs | 92.12 ms | 91.93 ms | 4.9 MB | 433 B | 5.34 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.95 ms | 1.17 ms | 243.81 ms | 247.24 ms | 46.4 MB | 460 B | 19.46 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 19.26 ms | 139.61 µs | 19.09 ms | 19.40 ms | 50.4 MB | 460 B | 19.13 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.10 ms | 144.50 µs | 18.95 ms | 19.04 ms | 50.2 MB | 460 B | 19.72 ms as a whole process |  |
| Seal | Normal | 66.99 ms | 531.38 µs | 66.38 ms | 67.28 ms | 3.3 MB | 3.5 KB | 315.42 µs cold, 291 ns warm; 3.06 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.04 ms | 311.30 µs | 69.45 ms | 70.34 ms | 4.8 MB | 3.0 KB | 5.30 ms as a whole process |  |
| Node `--jitless` | Normal | 184.77 ms | 19.41 ms | 172.67 ms | 181.59 ms | 48.2 MB | 3.1 KB | 19.43 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.89 ms | 22.48 µs | 3.85 ms | 5.98 ms | 51.9 MB | 3.1 KB | 20.04 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.89 ms | 21.31 µs | 3.86 ms | 5.71 ms | 52.0 MB | 3.1 KB | 19.30 ms as a whole process |  |
| Seal | Ceil.source | 56.07 ms | 505.98 µs | 55.43 ms | 57.19 ms | 3.2 MB | 3.4 KB | 308.79 µs cold, 291 ns warm; 3.04 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 58.20 ms | 272.93 µs | 57.71 ms | 58.84 ms | 3.2 MB | 3.4 KB | 292.42 µs cold, 375 ns warm; 3.07 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.11 ms | 292.27 µs | 49.40 ms | 50.50 ms | 3.0 MB | 3.4 KB | 287.04 µs cold, 208 ns warm; 2.80 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 50.30 ms | 315.03 µs | 49.70 ms | 49.66 ms | 3.0 MB | 3.4 KB | 294.00 µs cold, 250 ns warm; 2.72 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 70.01 ms | 826.48 µs | 69.06 ms | 70.11 ms | 4.8 MB | 3.0 KB | 5.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 177.83 ms | 3.88 ms | 172.93 ms | 179.26 ms | 48.1 MB | 3.1 KB | 19.24 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.89 ms | 14.04 µs | 3.87 ms | 5.69 ms | 52.0 MB | 3.1 KB | 18.96 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.89 ms | 21.82 µs | 3.86 ms | 5.68 ms | 51.9 MB | 3.1 KB | 19.20 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.11 ms | 230.53 µs | 53.76 ms | 53.66 ms | 2.8 MB | 429 B | 164.46 µs cold, 209 ns warm; 2.59 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 46.52 ms | 237.26 µs | 46.24 ms | 46.79 ms | 4.8 MB | 366 B | 5.08 ms as a whole process |  |
| Node `--jitless` | Purpose | 375.01 ms | 6.77 ms | 365.59 ms | 377.64 ms | 48.6 MB | 398 B | 20.50 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 35.28 ms | 37.62 ms | 15.76 ms | 21.66 ms | 52.9 MB | 398 B | 19.59 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.68 ms | 100.82 µs | 14.59 ms | 18.98 ms | 55.3 MB | 398 B | 21.19 ms as a whole process |  |
| Seal | Normal | 20.42 ms | 117.02 µs | 20.30 ms | 20.58 ms | 3.1 MB | 1.2 KB | 190.12 µs cold, 250 ns warm; 2.99 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.29 ms | 382.47 µs | 26.89 ms | 27.35 ms | 4.8 MB | 981 B | 5.45 ms as a whole process |  |
| Node `--jitless` | Normal | 66.14 ms | 517.93 µs | 65.60 ms | 66.80 ms | 48.4 MB | 1.0 KB | 19.71 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.69 ms | 200.04 µs | 1.52 ms | 3.29 ms | 52.8 MB | 1.0 KB | 21.20 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.71 ms | 79.29 µs | 1.64 ms | 3.30 ms | 52.9 MB | 1.0 KB | 20.97 ms as a whole process |  |
| Seal | Ceil.source | 20.63 ms | 224.59 µs | 20.27 ms | 23.85 ms | 3.1 MB | 1.2 KB | 199.00 µs cold, 250 ns warm; 2.96 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.67 ms | 421.35 µs | 20.43 ms | 20.31 ms | 3.1 MB | 1.2 KB | 171.00 µs cold, 250 ns warm; 3.15 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.71 ms | 204.54 µs | 19.48 ms | 19.92 ms | 2.8 MB | 1.2 KB | 164.79 µs cold, 208 ns warm; 2.82 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 18.98 ms | 129.84 µs | 18.81 ms | 18.94 ms | 2.9 MB | 1.2 KB | 144.92 µs cold, 250 ns warm; 2.73 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.18 ms | 336.31 µs | 26.60 ms | 27.39 ms | 4.8 MB | 981 B | 5.22 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.45 ms | 410.21 µs | 63.74 ms | 64.90 ms | 48.4 MB | 1.0 KB | 19.58 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.66 ms | 187.27 µs | 1.51 ms | 3.81 ms | 52.9 MB | 1.0 KB | 19.59 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.71 ms | 28.48 µs | 1.65 ms | 3.68 ms | 52.9 MB | 1.0 KB | 19.66 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 98.43 ms | 705.62 µs | 97.45 ms | 98.51 ms | 2.7 MB | 632 B | 130.04 µs cold, 292 ns warm; 2.68 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 88.65 ms | 1.07 ms | 87.47 ms | 90.72 ms | 4.8 MB | 523 B | 5.21 ms as a whole process |  |
| Node `--jitless` | Purpose | 146.83 ms | 909.30 µs | 145.98 ms | 149.63 ms | 43.4 MB | 568 B | 19.26 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.75 ms | 31.17 µs | 3.70 ms | 4.75 ms | 49.3 MB | 568 B | 20.32 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.72 ms | 36.37 µs | 3.67 ms | 4.56 ms | 49.3 MB | 568 B | 19.85 ms as a whole process |  |
| Seal | Normal | 122.96 ms | 6.77 ms | 116.65 ms | 115.41 ms | 3.0 MB | 1.8 KB | 189.75 µs cold, 333 ns warm; 2.73 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 217.02 ms | 3.11 ms | 209.64 ms | 217.82 ms | 4.8 MB | 1.7 KB | 5.29 ms as a whole process |  |
| Node `--jitless` | Normal | 166.94 ms | 14.83 ms | 157.68 ms | 161.26 ms | 45.5 MB | 1.7 KB | 20.00 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.18 ms | 154.24 µs | 13.83 ms | 15.82 ms | 52.0 MB | 1.7 KB | 20.19 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.98 ms | 212.13 µs | 14.85 ms | 16.50 ms | 52.2 MB | 1.7 KB | 18.77 ms as a whole process |  |
| Seal | Ceil.source | 116.83 ms | 742.77 µs | 116.02 ms | 116.52 ms | 3.0 MB | 1.8 KB | 163.17 µs cold, 209 ns warm; 2.91 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 113.41 ms | 369.63 µs | 112.93 ms | 114.42 ms | 3.0 MB | 1.8 KB | 166.62 µs cold, 250 ns warm; 2.89 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 104.56 ms | 201.55 µs | 104.22 ms | 104.62 ms | 2.8 MB | 1.8 KB | 137.96 µs cold, 208 ns warm; 2.76 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 102.40 ms | 107.97 µs | 102.24 ms | 102.42 ms | 2.8 MB | 1.8 KB | 145.92 µs cold, 209 ns warm; 2.76 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 212.70 ms | 2.54 ms | 206.24 ms | 212.03 ms | 4.8 MB | 1.7 KB | 5.60 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 158.32 ms | 1.23 ms | 156.25 ms | 162.08 ms | 45.4 MB | 1.7 KB | 19.00 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.78 ms | 142.11 µs | 14.59 ms | 16.12 ms | 52.0 MB | 1.7 KB | 19.15 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.34 ms | 104.93 µs | 14.24 ms | 15.87 ms | 52.2 MB | 1.7 KB | 19.84 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 97.09 ms | 899.35 µs | 96.05 ms | 96.41 ms | 3.9 MB | 569 B | 189.29 µs cold, 375 ns warm; 2.72 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.77 ms | 338.99 µs | 63.29 ms | 64.12 ms | 4.8 MB | 540 B | 5.25 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.07 ms | 1.20 ms | 242.15 ms | 247.37 ms | 48.6 MB | 514 B | 19.37 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 91.10 ms | 25.25 ms | 67.46 ms | 92.28 ms | 52.5 MB | 514 B | 19.53 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 69.05 ms | 921.09 µs | 67.11 ms | 90.58 ms | 52.8 MB | 514 B | 19.24 ms as a whole process |  |
| Seal | Normal | 23.97 ms | 161.38 µs | 23.70 ms | 24.21 ms | 4.0 MB | 538 B | 118.42 µs cold, 333 ns warm; 2.79 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 31.53 ms | 733.84 µs | 30.10 ms | 29.99 ms | 6.0 MB | 478 B | 6.02 ms as a whole process |  |
| Node `--jitless` | Normal | 53.14 ms | 968.70 µs | 52.30 ms | 55.38 ms | 56.4 MB | 477 B | 20.29 ms as a whole process |  |
| Node (JIT), from cold | Normal | 37.89 ms | 3.36 ms | 35.26 ms | 36.71 ms | 62.5 MB | 477 B | 20.58 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.69 ms | 299.23 µs | 35.22 ms | 36.17 ms | 88.7 MB | 477 B | 19.40 ms as a whole process |  |
| Seal | Ceil.source | 24.34 ms | 147.64 µs | 24.15 ms | 24.29 ms | 4.0 MB | 538 B | 118.00 µs cold, 291 ns warm; 2.70 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.21 ms | 309.68 µs | 24.85 ms | 25.91 ms | 4.0 MB | 538 B | 111.58 µs cold, 291 ns warm; 3.04 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.70 ms | 332.54 µs | 23.34 ms | 23.22 ms | 3.8 MB | 538 B | 101.62 µs cold, 125 ns warm; 2.84 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.26 ms | 249.21 µs | 22.92 ms | 23.24 ms | 4.0 MB | 538 B | 108.79 µs cold, 250 ns warm; 2.76 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.50 ms | 288.13 µs | 30.02 ms | 30.87 ms | 5.9 MB | 478 B | 5.30 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.01 ms | 511.20 µs | 50.94 ms | 53.45 ms | 56.5 MB | 477 B | 20.17 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.93 ms | 477.19 µs | 35.15 ms | 36.24 ms | 62.4 MB | 477 B | 19.57 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.99 ms | 285.31 µs | 35.59 ms | 55.94 ms | 88.3 MB | 477 B | 20.52 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 325.85 µs | 7.02 µs | 317.71 µs | 316.00 µs | 3.1 MB | 233 B | 104.88 µs cold, 333 ns warm; 2.72 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 146.02 ms | 873.45 µs | 145.13 ms | 147.76 ms | 10.1 MB | 195 B | 5.32 ms as a whole process |  |
| Node `--jitless` | Purpose | 368.85 µs | 145.06 µs | 245.79 µs | 1.04 ms | 52.5 MB | 223 B | 19.23 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 219.25 µs | 146.74 µs | 90.38 µs | 899.62 µs | 57.6 MB | 223 B | 19.06 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 132.85 µs | 117.17 µs | 55.67 µs | 912.33 µs | 57.8 MB | 223 B | 19.27 ms as a whole process |  |
| Seal | Normal | 352.55 µs | 14.06 µs | 340.58 µs | 373.04 µs | 3.2 MB | 182 B | 79.21 µs cold, 334 ns warm; 2.66 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 163.40 ms | 530.38 µs | 162.58 ms | 165.75 ms | 11.9 MB | 138 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Normal | 523.93 µs | 352.03 µs | 269.62 µs | 1.07 ms | 61.4 MB | 155 B | 18.94 ms as a whole process |  |
| Node (JIT), from cold | Normal | 496.61 µs | 473.21 µs | 137.21 µs | 1.06 ms | 65.1 MB | 155 B | 18.96 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 311.85 µs | 343.54 µs | 127.08 µs | 1.07 ms | 69.2 MB | 155 B | 19.36 ms as a whole process |  |
| Seal | Ceil.source | 349.21 µs | 7.61 µs | 341.08 µs | 374.42 µs | 3.2 MB | 182 B | 80.42 µs cold, 291 ns warm; 2.76 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 354.86 µs | 5.13 µs | 346.79 µs | 369.42 µs | 3.2 MB | 182 B | 84.83 µs cold, 292 ns warm; 2.67 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 292.73 µs | 8.98 µs | 285.00 µs | 309.38 µs | 3.0 MB | 182 B | 60.42 µs cold, 250 ns warm; 2.50 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 278.62 µs | 3.97 µs | 274.33 µs | 312.04 µs | 3.0 MB | 182 B | 66.54 µs cold, 291 ns warm; 2.61 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 163.76 ms | 518.77 µs | 162.88 ms | 166.03 ms | 12.0 MB | 138 B | 5.15 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 512.35 µs | 338.39 µs | 274.92 µs | 1.11 ms | 61.3 MB | 155 B | 18.71 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 477.83 µs | 458.23 µs | 126.75 µs | 1.03 ms | 65.2 MB | 155 B | 18.68 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 295.35 µs | 334.17 µs | 125.58 µs | 1.09 ms | 69.2 MB | 155 B | 18.87 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 75.12 ms | 721.06 µs | 74.08 ms | 75.54 ms | 3.0 MB | 537 B | 115.04 µs cold, 208 ns warm; 2.79 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 97.66 ms | 1.48 ms | 95.61 ms | 95.67 ms | 4.8 MB | 411 B | 5.97 ms as a whole process |  |
| Node `--jitless` | Purpose | 77.70 ms | 991.30 µs | 75.87 ms | 81.42 ms | 59.7 MB | 444 B | 19.97 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 11.37 ms | 947.64 µs | 10.73 ms | 13.16 ms | 65.8 MB | 444 B | 21.57 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.66 ms | 674.72 µs | 10.35 ms | 13.37 ms | 114.7 MB | 444 B | 20.09 ms as a whole process |  |
| Seal | Normal | 113.69 ms | 1.09 ms | 111.76 ms | 113.21 ms | 7.8 MB | 992 B | 148.42 µs cold, 250 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 133.54 ms | 2.90 ms | 129.24 ms | 134.25 ms | 43.8 MB | 799 B | 5.26 ms as a whole process |  |
| Node `--jitless` | Normal | 116.32 ms | 1.85 ms | 114.42 ms | 119.20 ms | 213.5 MB | 878 B | 19.70 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.23 ms | 860.97 µs | 13.33 ms | 17.01 ms | 129.4 MB | 878 B | 19.24 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.60 ms | 91.75 µs | 13.40 ms | 17.08 ms | 224.2 MB | 878 B | 19.20 ms as a whole process |  |
| Seal | Ceil.source | 112.23 ms | 871.72 µs | 110.84 ms | 113.13 ms | 7.8 MB | 992 B | 155.71 µs cold, 250 ns warm; 2.80 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 123.72 ms | 1.04 ms | 122.80 ms | 126.37 ms | 7.8 MB | 992 B | 163.38 µs cold, 208 ns warm; 2.78 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 91.24 ms | 344.98 µs | 90.82 ms | 91.77 ms | 7.6 MB | 992 B | 128.29 µs cold, 167 ns warm; 2.85 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 90.95 ms | 543.66 µs | 90.09 ms | 92.52 ms | 7.6 MB | 992 B | 126.29 µs cold, 125 ns warm; 2.76 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 134.67 ms | 1.46 ms | 132.48 ms | 132.92 ms | 39.7 MB | 799 B | 5.17 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 116.75 ms | 1.65 ms | 115.36 ms | 119.77 ms | 213.2 MB | 878 B | 18.58 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.39 ms | 1.05 ms | 13.46 ms | 17.24 ms | 129.5 MB | 878 B | 18.90 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.65 ms | 1.20 ms | 13.51 ms | 17.20 ms | 223.7 MB | 878 B | 19.35 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 79.66 µs (±12.18 µs, least 59.50 µs) | 37.37 µs (±9.28 µs, least 23.38 µs) | 16 ns (±0 ns, least 16 ns) | 36 ns (±1 ns, least 36 ns) | 512 B | 2.8 MB | counted by Seal |
| Luau (interpreter) | Purpose | 12.13 µs (±3.75 µs, least 9.33 µs) | 21.52 µs (±7.74 µs, least 17.17 µs) | 45 ns (±10 ns, least 40 ns) | 23 ns (±3 ns, least 20 ns) | 3.0 KB | 4.9 MB | the engine's own heap |
| Node `--jitless` | Purpose | 54.74 µs (±25.42 µs, least 35.67 µs) | 73.52 µs (±77.79 µs, least 36.04 µs) | 61 ns (±3 ns, least 58 ns) | 31 ns (±3 ns, least 30 ns) | 0 B | 45.8 MB | the engine's own heap |
| Node (JIT) | Purpose | 42.77 µs (±27.50 µs, least 20.42 µs) | 39.71 µs (±2.38 µs, least 37.67 µs) | 9 ns (±0 ns, least 9 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 8.58 µs (±186 ns, least 8.21 µs) | 192 ns (±18 ns, least 163 ns) | 17 ns (±0 ns, least 17 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 4.9 MB | counted by Seal |
| Luau (interpreter) | Normal | 5.11 µs (±190 ns, least 4.86 µs) | 203 ns (±23 ns, least 178 ns) | 36 ns (±2 ns, least 33 ns) | 19 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.17 µs (±29 ns, least 1.11 µs) | 684 ns (±23 ns, least 658 ns) | 51 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.13 µs (±34 ns, least 1.09 µs) | 697 ns (±22 ns, least 675 ns) | 23 ns (±1 ns, least 22 ns) | 7 ns (±1 ns, least 6 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 8.29 µs (±80 ns, least 8.16 µs) | 175 ns (±5 ns, least 167 ns) | 17 ns (±0 ns, least 17 ns) | 36 ns (±0 ns, least 35 ns) | 512 B | 4.9 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 8.27 µs (±53 ns, least 8.19 µs) | 178 ns (±12 ns, least 163 ns) | 18 ns (±0 ns, least 18 ns) | 37 ns (±1 ns, least 36 ns) | 512 B | 4.9 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 7.43 µs (±112 ns, least 7.26 µs) | 158 ns (±6 ns, least 147 ns) | 14 ns (±1 ns, least 14 ns) | 35 ns (±1 ns, least 34 ns) | 512 B | 4.7 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 7.72 µs (±190 ns, least 7.55 µs) | 155 ns (±5 ns, least 148 ns) | 14 ns (±0 ns, least 14 ns) | 35 ns (±1 ns, least 34 ns) | 512 B | 4.8 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.06 µs (±198 ns, least 4.73 µs) | 187 ns (±7 ns, least 172 ns) | 34 ns (±1 ns, least 33 ns) | 18 ns (±1 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.30 µs (±173 ns, least 1.16 µs) | 852 ns (±181 ns, least 673 ns) | 57 ns (±5 ns, least 50 ns) | 26 ns (±2 ns, least 24 ns) | 643 B | 48.8 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.14 µs (±31 ns, least 1.09 µs) | 716 ns (±55 ns, least 634 ns) | 23 ns (±1 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 50.94 ms | 48.97 ms (-3.9%) |
| n-body, 100,000 steps | 65.89 ms | 61.23 ms (-7.1%) |
| spectral-norm, size 200 | 20.21 ms | 19.47 ms (-3.7%) |
| fannkuch-redux, 9 | 114.96 ms | 112.26 ms (-2.4%) |
| word count, 500,000 words | 24.33 ms | 28.64 ms (+17.7%) |
| text building, 20,000 joins | 500.70 µs | 348.86 µs (-30.3%) |
| binary-trees, depth 14 | 111.27 ms | 112.11 ms (+0.8%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 229.45 ms | 4.02 ms | 225.87 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 11.30 ms | 53.72 µs | 11.25 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.55 ms | 41.39 µs | 4.50 ms | 3.7 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.43 ms | 14.34 µs | 1.40 ms | 3.5 MB |
