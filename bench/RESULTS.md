# Seal's benchmark results

Written by `seal-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Seal:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.39 2.37 2.31 }` before, `{ 1.87 2.26 2.28 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Seal (cold, then warm, when Seal reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Seal's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Seal | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 49.17 ms | 72.87 ms | **met**: Seal takes 67% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 65.24 ms | 70.29 ms | **met**: Seal takes 93% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 20.00 ms | 27.29 ms | **met**: Seal takes 73% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 111.78 ms | 210.69 ms | **met**: Seal takes 53% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 24.02 ms | 32.60 ms | **met**: Seal takes 74% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 358.53 µs | 163.73 ms | **met**: Seal takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 111.88 ms | 131.31 ms | **met**: Seal takes 85% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 1.86 ms | 3.76 ms | **met**: Seal takes 49% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 17 ns | 36 ns | **met**: Seal takes 48% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 162 ns | 194 ns | **met**: Seal takes 84% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 6.87 µs | 5.00 µs | **not met**: Seal takes 137% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 49.17 ms | 94.17 ms | **met**: Seal takes 52% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 65.24 ms | 176.86 ms | **met**: Seal takes 37% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 24.02 ms | 52.60 ms | **met**: Seal takes 46% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 358.53 µs | 516.74 µs | **met**: Seal takes 69% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 17 ns | host call 38 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target: its rows are in every table for reference. Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 135.85 ms | 487.78 µs | 134.96 ms | 154.55 ms | 2.8 MB | 395 B | 707.17 µs cold, 458 ns warm; 2.68 ms as a whole process | 64.0 KB counted by Seal; 200 B compiled |
| Luau (interpreter) | Purpose | 200.15 ms | 2.23 ms | 198.10 ms | 196.62 ms | 4.8 MB | 339 B | 5.61 ms as a whole process |  |
| Node `--jitless` | Purpose | 331.60 ms | 14.41 ms | 323.46 ms | 326.71 ms | 43.4 MB | 362 B | 20.25 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 127.46 ms | 475.83 µs | 126.71 ms | 131.15 ms | 49.5 MB | 362 B | 20.96 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 125.39 ms | 1.07 ms | 123.80 ms | 127.48 ms | 49.1 MB | 362 B | 19.57 ms as a whole process |  |
| Seal | Normal | 49.17 ms | 136.24 µs | 48.86 ms | 49.86 ms | 2.7 MB | 169 B | 87.58 µs cold, 166 ns warm; 2.60 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled |
| Luau (interpreter) | Normal | 72.87 ms | 138.16 µs | 72.68 ms | 72.84 ms | 4.8 MB | 143 B | 5.13 ms as a whole process |  |
| Node `--jitless` | Normal | 94.17 ms | 777.53 µs | 93.20 ms | 94.64 ms | 43.4 MB | 137 B | 19.09 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.60 ms | 96.47 µs | 9.53 ms | 11.99 ms | 48.5 MB | 137 B | 18.61 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.74 ms | 249.00 µs | 9.40 ms | 10.22 ms | 48.7 MB | 137 B | 18.57 ms as a whole process |  |
| Seal | Ceil.source | 49.07 ms | 111.40 µs | 48.90 ms | 49.02 ms | 2.7 MB | 169 B | 79.38 µs cold, 167 ns warm; 2.59 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 52.39 ms | 113.90 µs | 52.20 ms | 76.89 ms | 2.7 MB | 169 B | 1.01 ms cold, 583 ns warm; 2.63 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.program | 44.23 ms | 155.86 µs | 43.99 ms | 68.14 ms | 2.5 MB | 169 B | 850.79 µs cold, 583 ns warm; 2.61 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 43.62 ms | 177.78 µs | 43.25 ms | 58.98 ms | 2.5 MB | 169 B | 648.42 µs cold, 334 ns warm; 2.75 ms as a whole process | 4.0 KB counted by Seal; 144 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 70.67 ms | 273.73 µs | 70.19 ms | 70.20 ms | 4.8 MB | 143 B | 5.00 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 94.25 ms | 448.05 µs | 93.53 ms | 95.07 ms | 43.3 MB | 137 B | 18.85 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.84 ms | 800.50 µs | 9.46 ms | 9.82 ms | 48.6 MB | 137 B | 19.01 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.42 ms | 14.67 µs | 9.39 ms | 10.86 ms | 48.5 MB | 137 B | 18.44 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 54.36 ms | 329.84 µs | 53.77 ms | 54.18 ms | 2.7 MB | 550 B | 104.54 µs cold, 208 ns warm; 2.72 ms as a whole process | 512 B counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 92.12 ms | 361.69 µs | 91.56 ms | 91.44 ms | 4.8 MB | 433 B | 4.88 ms as a whole process |  |
| Node `--jitless` | Purpose | 244.83 ms | 577.99 µs | 244.24 ms | 245.78 ms | 46.3 MB | 460 B | 18.50 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.67 ms | 166.14 µs | 18.42 ms | 19.09 ms | 50.5 MB | 460 B | 19.05 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 19.14 ms | 178.45 µs | 18.94 ms | 19.06 ms | 50.3 MB | 460 B | 20.42 ms as a whole process |  |
| Seal | Normal | 65.24 ms | 623.18 µs | 64.38 ms | 66.71 ms | 3.3 MB | 3.5 KB | 322.25 µs cold, 334 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Normal | 70.29 ms | 873.15 µs | 69.29 ms | 70.28 ms | 4.8 MB | 3.0 KB | 5.25 ms as a whole process |  |
| Node `--jitless` | Normal | 176.86 ms | 3.66 ms | 172.78 ms | 180.67 ms | 48.1 MB | 3.1 KB | 19.07 ms as a whole process |  |
| Node (JIT), from cold | Normal | 3.87 ms | 17.64 µs | 3.85 ms | 6.14 ms | 52.0 MB | 3.1 KB | 19.10 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 3.87 ms | 13.08 µs | 3.85 ms | 5.70 ms | 52.0 MB | 3.1 KB | 18.88 ms as a whole process |  |
| Seal | Ceil.source | 55.47 ms | 374.21 µs | 54.87 ms | 57.64 ms | 3.2 MB | 3.4 KB | 271.25 µs cold, 292 ns warm; 2.84 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine | 56.40 ms | 789.65 µs | 55.78 ms | 57.08 ms | 3.2 MB | 3.4 KB | 270.46 µs cold, 292 ns warm; 2.99 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.program | 50.16 ms | 383.81 µs | 49.62 ms | 50.40 ms | 3.0 MB | 3.4 KB | 279.75 µs cold, 250 ns warm; 2.85 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Seal | Ceil.machine.program | 49.98 ms | 247.18 µs | 49.53 ms | 50.20 ms | 3.0 MB | 3.4 KB | 275.08 µs cold, 250 ns warm; 2.78 ms as a whole process | 2.0 KB counted by Seal; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 70.22 ms | 247.51 µs | 69.68 ms | 70.29 ms | 4.8 MB | 3.0 KB | 5.08 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 176.14 ms | 1.48 ms | 174.30 ms | 176.98 ms | 48.1 MB | 3.1 KB | 19.37 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.88 ms | 16.23 µs | 3.85 ms | 5.74 ms | 51.8 MB | 3.1 KB | 18.80 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.87 ms | 26.08 µs | 3.84 ms | 5.65 ms | 52.0 MB | 3.1 KB | 18.88 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 52.63 ms | 450.82 µs | 52.00 ms | 52.67 ms | 2.8 MB | 429 B | 100.29 µs cold, 250 ns warm; 2.76 ms as a whole process | 8.5 KB counted by Seal; 224 B compiled |
| Luau (interpreter) | Purpose | 45.35 ms | 279.37 µs | 44.97 ms | 45.00 ms | 4.8 MB | 366 B | 5.14 ms as a whole process |  |
| Node `--jitless` | Purpose | 376.39 ms | 5.76 ms | 366.81 ms | 372.48 ms | 48.7 MB | 398 B | 20.09 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.05 ms | 36.83 ms | 15.56 ms | 19.77 ms | 52.7 MB | 398 B | 19.34 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 14.70 ms | 260.11 µs | 14.39 ms | 21.03 ms | 55.4 MB | 398 B | 18.62 ms as a whole process |  |
| Seal | Normal | 20.00 ms | 42.73 µs | 19.92 ms | 20.07 ms | 3.0 MB | 1.2 KB | 167.71 µs cold, 208 ns warm; 2.71 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled |
| Luau (interpreter) | Normal | 27.29 ms | 578.01 µs | 26.58 ms | 27.47 ms | 4.8 MB | 981 B | 5.19 ms as a whole process |  |
| Node `--jitless` | Normal | 67.30 ms | 1.26 ms | 64.92 ms | 64.10 ms | 48.5 MB | 1.0 KB | 21.17 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.62 ms | 246.11 µs | 1.44 ms | 3.40 ms | 53.0 MB | 1.0 KB | 20.34 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.69 ms | 27.19 µs | 1.64 ms | 3.52 ms | 52.9 MB | 1.0 KB | 19.30 ms as a whole process |  |
| Seal | Ceil.source | 20.35 ms | 87.77 µs | 20.26 ms | 20.17 ms | 3.1 MB | 1.2 KB | 169.38 µs cold, 167 ns warm; 2.88 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 20.45 ms | 183.40 µs | 20.26 ms | 20.47 ms | 3.1 MB | 1.2 KB | 169.62 µs cold, 250 ns warm; 2.96 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.program | 19.57 ms | 129.35 µs | 19.39 ms | 19.65 ms | 2.9 MB | 1.2 KB | 140.04 µs cold, 167 ns warm; 2.74 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 19.02 ms | 103.61 µs | 18.83 ms | 19.20 ms | 2.8 MB | 1.2 KB | 157.62 µs cold, 250 ns warm; 3.16 ms as a whole process | 9.2 KB counted by Seal; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.97 ms | 999.61 µs | 26.48 ms | 27.91 ms | 4.8 MB | 981 B | 5.32 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.98 ms | 437.74 µs | 64.07 ms | 63.64 ms | 48.4 MB | 1.0 KB | 19.38 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.66 ms | 230.98 µs | 1.50 ms | 3.54 ms | 53.0 MB | 1.0 KB | 18.92 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.64 ms | 21.79 µs | 1.61 ms | 3.30 ms | 52.8 MB | 1.0 KB | 19.68 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.72 ms | 439.02 µs | 95.20 ms | 96.55 ms | 2.7 MB | 632 B | 100.29 µs cold, 208 ns warm; 2.60 ms as a whole process | 688 B counted by Seal; 296 B compiled |
| Luau (interpreter) | Purpose | 89.03 ms | 957.47 µs | 87.72 ms | 89.58 ms | 4.9 MB | 523 B | 5.24 ms as a whole process |  |
| Node `--jitless` | Purpose | 148.43 ms | 1.08 ms | 146.67 ms | 150.69 ms | 43.4 MB | 568 B | 18.40 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.68 ms | 24.99 µs | 3.65 ms | 4.74 ms | 49.4 MB | 568 B | 19.13 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.69 ms | 28.91 µs | 3.64 ms | 4.52 ms | 49.3 MB | 568 B | 18.87 ms as a whole process |  |
| Seal | Normal | 111.78 ms | 617.30 µs | 110.46 ms | 113.09 ms | 3.0 MB | 1.8 KB | 167.75 µs cold, 208 ns warm; 2.82 ms as a whole process | 984 B counted by Seal; 696 B compiled |
| Luau (interpreter) | Normal | 210.69 ms | 1.24 ms | 208.26 ms | 211.69 ms | 4.8 MB | 1.7 KB | 5.29 ms as a whole process |  |
| Node `--jitless` | Normal | 155.00 ms | 687.14 µs | 154.11 ms | 156.95 ms | 45.3 MB | 1.7 KB | 18.96 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.96 ms | 86.67 µs | 14.82 ms | 16.57 ms | 52.0 MB | 1.7 KB | 19.14 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.85 ms | 53.79 µs | 14.76 ms | 18.53 ms | 52.1 MB | 1.7 KB | 18.62 ms as a whole process |  |
| Seal | Ceil.source | 112.69 ms | 995.78 µs | 111.82 ms | 113.03 ms | 3.0 MB | 1.8 KB | 191.00 µs cold, 250 ns warm; 2.70 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 114.20 ms | 452.07 µs | 113.23 ms | 116.53 ms | 3.0 MB | 1.8 KB | 161.75 µs cold, 291 ns warm; 3.09 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.program | 105.25 ms | 416.83 µs | 104.62 ms | 107.27 ms | 2.8 MB | 1.8 KB | 141.67 µs cold, 208 ns warm; 2.94 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 104.85 ms | 6.71 ms | 102.43 ms | 103.75 ms | 2.8 MB | 1.8 KB | 149.71 µs cold, 292 ns warm; 2.78 ms as a whole process | 984 B counted by Seal; 696 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 212.45 ms | 1.68 ms | 207.80 ms | 215.57 ms | 4.8 MB | 1.7 KB | 5.26 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 158.27 ms | 1.27 ms | 155.42 ms | 161.69 ms | 45.4 MB | 1.7 KB | 18.96 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.31 ms | 315.43 µs | 13.97 ms | 15.85 ms | 52.0 MB | 1.7 KB | 18.90 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.97 ms | 146.00 µs | 14.86 ms | 16.32 ms | 52.2 MB | 1.7 KB | 18.83 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 95.73 ms | 968.77 µs | 94.64 ms | 93.73 ms | 4.0 MB | 569 B | 102.38 µs cold, 333 ns warm; 2.56 ms as a whole process | 793.6 KB counted by Seal; 336 B compiled |
| Luau (interpreter) | Purpose | 63.01 ms | 158.80 µs | 62.82 ms | 62.95 ms | 4.8 MB | 540 B | 5.04 ms as a whole process |  |
| Node `--jitless` | Purpose | 245.10 ms | 2.27 ms | 241.71 ms | 245.07 ms | 48.6 MB | 514 B | 19.20 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 87.68 ms | 24.32 ms | 66.37 ms | 89.64 ms | 52.5 MB | 514 B | 19.28 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 65.67 ms | 497.70 µs | 64.91 ms | 91.40 ms | 52.8 MB | 514 B | 18.76 ms as a whole process |  |
| Seal | Normal | 24.02 ms | 152.40 µs | 23.80 ms | 24.19 ms | 3.9 MB | 538 B | 103.33 µs cold, 250 ns warm; 2.65 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled |
| Luau (interpreter) | Normal | 32.60 ms | 240.63 µs | 32.13 ms | 30.31 ms | 6.0 MB | 478 B | 5.09 ms as a whole process |  |
| Node `--jitless` | Normal | 52.60 ms | 215.46 µs | 52.32 ms | 53.90 ms | 56.5 MB | 477 B | 19.01 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.53 ms | 273.31 µs | 35.18 ms | 36.06 ms | 62.5 MB | 477 B | 19.45 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.13 ms | 221.99 µs | 34.80 ms | 36.53 ms | 88.3 MB | 477 B | 18.70 ms as a whole process |  |
| Seal | Ceil.source | 24.01 ms | 275.19 µs | 23.68 ms | 23.81 ms | 3.9 MB | 538 B | 108.25 µs cold, 375 ns warm; 2.73 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 25.25 ms | 272.52 µs | 24.82 ms | 24.83 ms | 4.0 MB | 538 B | 119.17 µs cold, 250 ns warm; 2.72 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.program | 23.32 ms | 240.87 µs | 22.93 ms | 23.41 ms | 3.9 MB | 538 B | 98.83 µs cold, 250 ns warm; 2.68 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 23.37 ms | 255.11 µs | 22.92 ms | 23.08 ms | 3.9 MB | 538 B | 88.54 µs cold, 250 ns warm; 2.62 ms as a whole process | 625.5 KB counted by Seal; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.40 ms | 153.08 µs | 30.15 ms | 30.80 ms | 6.0 MB | 478 B | 5.23 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.56 ms | 220.87 µs | 52.20 ms | 53.73 ms | 56.6 MB | 477 B | 19.27 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 35.34 ms | 234.52 µs | 35.01 ms | 36.05 ms | 62.4 MB | 477 B | 19.01 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 36.39 ms | 484.86 µs | 35.63 ms | 36.35 ms | 88.1 MB | 477 B | 21.07 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 315.19 µs | 18.61 µs | 303.50 µs | 315.08 µs | 3.0 MB | 233 B | 72.54 µs cold, 333 ns warm; 2.99 ms as a whole process | 64.6 KB counted by Seal; 96 B compiled |
| Luau (interpreter) | Purpose | 146.28 ms | 495.73 µs | 145.08 ms | 149.52 ms | 9.8 MB | 195 B | 5.05 ms as a whole process |  |
| Node `--jitless` | Purpose | 374.35 µs | 141.35 µs | 244.71 µs | 1.02 ms | 52.5 MB | 223 B | 18.44 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 216.38 µs | 149.48 µs | 90.58 µs | 887.25 µs | 57.8 MB | 223 B | 19.10 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 133.00 µs | 117.61 µs | 55.00 µs | 902.96 µs | 57.8 MB | 223 B | 18.88 ms as a whole process |  |
| Seal | Normal | 358.53 µs | 25.19 µs | 336.92 µs | 383.17 µs | 3.2 MB | 182 B | 78.75 µs cold, 334 ns warm; 2.75 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled |
| Luau (interpreter) | Normal | 163.73 ms | 1.39 ms | 162.36 ms | 168.17 ms | 12.1 MB | 138 B | 5.23 ms as a whole process |  |
| Node `--jitless` | Normal | 516.74 µs | 343.84 µs | 268.71 µs | 1.09 ms | 61.5 MB | 155 B | 18.69 ms as a whole process |  |
| Node (JIT), from cold | Normal | 501.40 µs | 483.11 µs | 123.71 µs | 1.06 ms | 65.1 MB | 155 B | 19.23 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 280.73 µs | 326.88 µs | 125.08 µs | 1.07 ms | 69.3 MB | 155 B | 18.20 ms as a whole process |  |
| Seal | Ceil.source | 342.58 µs | 5.02 µs | 337.25 µs | 363.17 µs | 3.2 MB | 182 B | 76.04 µs cold, 291 ns warm; 2.71 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 352.85 µs | 5.71 µs | 346.83 µs | 377.38 µs | 3.2 MB | 182 B | 78.08 µs cold, 291 ns warm; 2.51 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.program | 294.47 µs | 3.78 µs | 289.12 µs | 340.50 µs | 3.0 MB | 182 B | 62.42 µs cold, 209 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 288.93 µs | 3.72 µs | 285.04 µs | 314.62 µs | 3.0 MB | 182 B | 63.42 µs cold, 292 ns warm; 2.42 ms as a whole process | 128.6 KB counted by Seal; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 162.23 ms | 302.23 µs | 161.74 ms | 164.75 ms | 11.9 MB | 138 B | 5.16 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 512.10 µs | 348.25 µs | 262.62 µs | 1.11 ms | 61.5 MB | 155 B | 19.24 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 498.43 µs | 476.87 µs | 125.46 µs | 1.07 ms | 65.1 MB | 155 B | 18.34 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 281.74 µs | 327.01 µs | 129.25 µs | 1.07 ms | 69.5 MB | 155 B | 19.10 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 76.10 ms | 550.61 µs | 75.08 ms | 77.22 ms | 2.9 MB | 537 B | 100.25 µs cold, 208 ns warm; 2.55 ms as a whole process | 65.9 KB counted by Seal; 232 B compiled |
| Luau (interpreter) | Purpose | 95.81 ms | 343.67 µs | 95.31 ms | 95.63 ms | 4.8 MB | 411 B | 5.10 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.90 ms | 686.50 µs | 75.95 ms | 80.65 ms | 59.6 MB | 444 B | 18.81 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.50 ms | 272.14 µs | 10.24 ms | 13.03 ms | 65.8 MB | 444 B | 18.75 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.46 ms | 570.83 µs | 10.07 ms | 13.05 ms | 114.7 MB | 444 B | 19.24 ms as a whole process |  |
| Seal | Normal | 111.88 ms | 738.87 µs | 111.10 ms | 113.70 ms | 7.8 MB | 992 B | 149.88 µs cold, 291 ns warm; 2.91 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled |
| Luau (interpreter) | Normal | 131.31 ms | 1.48 ms | 128.88 ms | 129.43 ms | 43.2 MB | 799 B | 5.23 ms as a whole process |  |
| Node `--jitless` | Normal | 114.07 ms | 2.05 ms | 111.97 ms | 117.15 ms | 213.4 MB | 878 B | 19.61 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.23 ms | 850.21 µs | 13.44 ms | 17.15 ms | 129.7 MB | 878 B | 19.64 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.45 ms | 37.40 µs | 13.39 ms | 16.85 ms | 225.2 MB | 878 B | 19.13 ms as a whole process |  |
| Seal | Ceil.source | 112.83 ms | 1.62 ms | 110.63 ms | 112.67 ms | 7.8 MB | 992 B | 124.29 µs cold, 250 ns warm; 2.84 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine | 115.28 ms | 804.93 µs | 113.87 ms | 115.88 ms | 7.8 MB | 992 B | 128.50 µs cold, 208 ns warm; 2.81 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.program | 91.39 ms | 502.01 µs | 90.79 ms | 92.08 ms | 7.6 MB | 992 B | 124.17 µs cold, 208 ns warm; 2.74 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Seal | Ceil.machine.program | 91.81 ms | 500.85 µs | 90.99 ms | 92.56 ms | 7.6 MB | 992 B | 114.88 µs cold, 209 ns warm; 2.79 ms as a whole process | 4.0 MB counted by Seal; 672 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 139.62 ms | 6.25 ms | 132.55 ms | 134.47 ms | 42.4 MB | 799 B | 5.65 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 117.14 ms | 1.69 ms | 115.21 ms | 118.54 ms | 213.3 MB | 878 B | 19.18 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.42 ms | 901.88 µs | 13.58 ms | 17.08 ms | 130.0 MB | 878 B | 20.20 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.87 ms | 209.58 µs | 13.48 ms | 17.29 ms | 223.9 MB | 878 B | 20.08 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed two ways: every script from a text of its own, so that nothing compiled is reused, and every script from the same text, where Seal reuses what it compiled, and Luau and Node compile once and make each script from that.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|
| Seal | Purpose | 76.38 µs (±6.14 µs, least 69.79 µs) | 35.81 µs (±16.24 µs, least 25.67 µs) | 16 ns (±0 ns, least 16 ns) | 38 ns (±0 ns, least 38 ns) | 512 B | 2.7 MB | counted by Seal |
| Luau (interpreter) | Purpose | 9.84 µs (±773 ns, least 9.12 µs) | 18.55 µs (±692 ns, least 17.42 µs) | 46 ns (±12 ns, least 41 ns) | 22 ns (±3 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| Node `--jitless` | Purpose | 40.67 µs (±24.54 µs, least 26.04 µs) | 70.27 µs (±90.40 µs, least 38.88 µs) | 58 ns (±1 ns, least 57 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 45.7 MB | the engine's own heap |
| Node (JIT) | Purpose | 33.15 µs (±18.25 µs, least 18.58 µs) | 40.95 µs (±3.60 µs, least 37.00 µs) | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.1 MB | the engine's own heap |
| Seal | Normal | 6.87 µs (±163 ns, least 6.58 µs) | 162 ns (±19 ns, least 149 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 4.7 MB | counted by Seal |
| Luau (interpreter) | Normal | 5.00 µs (±192 ns, least 4.73 µs) | 194 ns (±10 ns, least 182 ns) | 36 ns (±2 ns, least 33 ns) | 18 ns (±0 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.20 µs (±53 ns, least 1.15 µs) | 720 ns (±43 ns, least 664 ns) | 52 ns (±2 ns, least 50 ns) | 25 ns (±1 ns, least 24 ns) | 643 B | 48.6 MB | the engine's own heap |
| Node (JIT) | Normal | 1.18 µs (±76 ns, least 1.12 µs) | 785 ns (±158 ns, least 652 ns) | 25 ns (±3 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap |
| Seal | Ceil.source | 6.87 µs (±245 ns, least 6.61 µs) | 155 ns (±8 ns, least 144 ns) | 17 ns (±1 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 4.8 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine | 6.83 µs (±175 ns, least 6.60 µs) | 170 ns (±19 ns, least 148 ns) | 18 ns (±1 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 4.8 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.program | 6.20 µs (±171 ns, least 5.93 µs) | 158 ns (±31 ns, least 140 ns) | 14 ns (±0 ns, least 14 ns) | 35 ns (±1 ns, least 35 ns) | 512 B | 4.7 MB | counted by Seal; the Normal program stands in |
| Seal | Ceil.machine.program | 6.66 µs (±321 ns, least 6.28 µs) | 166 ns (±27 ns, least 145 ns) | 14 ns (±1 ns, least 14 ns) | 36 ns (±2 ns, least 35 ns) | 512 B | 4.7 MB | counted by Seal; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 5.13 µs (±193 ns, least 4.85 µs) | 223 ns (±73 ns, least 187 ns) | 38 ns (±9 ns, least 34 ns) | 20 ns (±6 ns, least 18 ns) | 2.0 KB | 6.1 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.30 µs (±207 ns, least 1.15 µs) | 714 ns (±58 ns, least 652 ns) | 52 ns (±2 ns, least 49 ns) | 25 ns (±2 ns, least 23 ns) | 643 B | 48.6 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.13 µs (±47 ns, least 1.06 µs) | 709 ns (±42 ns, least 645 ns) | 23 ns (±1 ns, least 21 ns) | 7 ns (±2 ns, least 5 ns) | 643 B | 55.9 MB | the engine's own heap; the Normal program stands in |

## What Seal's checks cost (Purpose)

D58 asks for the cost of Seal's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 49.53 ms | 48.83 ms (-1.4%) |
| n-body, 100,000 steps | 64.54 ms | 60.59 ms (-6.1%) |
| spectral-norm, size 200 | 20.18 ms | 19.85 ms (-1.6%) |
| fannkuch-redux, 9 | 111.45 ms | 111.62 ms (+0.2%) |
| word count, 500,000 words | 24.10 ms | 24.95 ms (+3.5%) |
| text building, 20,000 joins | 343.55 µs | 346.05 µs (+0.7%) |
| binary-trees, depth 14 | 111.13 ms | 112.27 ms (+1.0%) |

## The new number types (D115, D116), Normal

Timed on Seal alone, since Luau has neither, and D57 sets no goal for them; the `Float` row shows what exactness costs.

| Program | Mean | Deviation | Least | Peak memory |
|---|---|---|---|---|
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 180.38 ms | 1.60 ms | 178.25 ms | 2.9 MB |
| the same sums in `Float`s, as a yardstick | 11.16 ms | 120.27 µs | 11.04 ms | 2.7 MB |
| `HugeWhole`: 3000 factorial, 9,131 digits | 4.47 ms | 48.81 µs | 4.43 ms | 3.3 MB |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 1.42 ms | 14.94 µs | 1.41 ms | 3.5 MB |
