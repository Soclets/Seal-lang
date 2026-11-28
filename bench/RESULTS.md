# Wax's benchmark results

Written by `wax-bench report` (see `bench/run.sh`). The benchmarks are D57's, run by D58's rules: every benchmark in every kind, every number saying its kind, and every answer checked before its time counts.

## Setup

- **Machine:** Apple M5, 10 cores, 24 GB.
- **System:** macOS 27.2.1 (27.2.0).
- **Wax:** this repository, built by `rustc 1.96.1 (31fca3adb 2026-06-26)`, release profile (optimised, link-time optimisation, one code unit).
- **Luau:** `0.736.0`, its interpreter, with native code generation off (the default of the `luau` command).
- **LuaJIT:** `LuaJIT 2.1.1787165859 -- Copyright (C) 2005-2026 Mike Pall. https://luajit.org/`, with `-joff` (its interpreter alone, a bar for goal 1 by D142) and without (its JIT on, for reference). It runs the Luau programs as they are, which are plain Lua.
- **Node:** `v24.16.0`, with `--jitless` (only Ignition, V8's bytecode interpreter) and without (V8's JITs on).
- **Python:** `Python 3.14.7`, with `gmpy2` `2.3.2` (on GMP), in `target/bench-python`, which `bench/run.sh` makes; Python's `decimal` is on libmpdec `4.0.1`.
- **How busy the machine was** (load averages over 1, 5 and 15 minutes): `{ 2.23 2.20 2.10 }` before, `{ 2.47 2.29 2.15 }` after. Other sessions were working on this machine during the run.
- **Runs:** each row is one process; `run` is called once (the first run), twice more to warm up, then 10 times, timed. The warmed-up Node row warms up 30 times instead; the from-cold Node row not at all.
- **Peak memory** is the whole process's, as the system reports it. **Load** is how long loading the script takes: inside the process for Wax (cold, then warm, when Wax reuses what it compiled for the same text), and as a whole process that starts, loads and stops for every engine.
- **Kinds:** Purpose isolates one thing; Normal is written plainly; Ceil is tuned. `Ceil.source` runs the tuned program on the usual build. `Ceil.machine` builds Wax's VM for this exact CPU, `Ceil.program` builds it with a profile of the benchmarks' own run, and `Ceil.machine.program` does both; a script is never compiled for a CPU (D46), so those apply to the VM. Luau, LuaJIT and Node are the installed builds in every kind.

## D57's goals, judged Normal against Normal (D58)

| Goal | Benchmark | Wax | Against | Verdict |
|---|---|---|---|---|
| 1. Faster than Luau's interpreter | `fib(32)`, recursive | 30.81 ms | 71.12 ms | **met**: Wax takes 43% of its time |
| 1. Faster than Luau's interpreter | n-body, 100,000 steps | 66.65 ms | 69.75 ms | **met**: Wax takes 96% of its time |
| 1. Faster than Luau's interpreter | spectral-norm, size 200 | 18.61 ms | 27.32 ms | **met**: Wax takes 68% of its time |
| 1. Faster than Luau's interpreter | fannkuch-redux, 9 | 104.75 ms | 215.75 ms | **met**: Wax takes 49% of its time |
| 1. Faster than Luau's interpreter | word count, 500,000 words | 27.05 ms | 31.06 ms | **met**: Wax takes 87% of its time |
| 1. Faster than Luau's interpreter | text building, 20,000 joins | 342.61 µs | 162.21 ms | **met**: Wax takes 0.2% of its time |
| 1. Faster than Luau's interpreter | binary-trees, depth 14 | 99.73 ms | 134.39 ms | **met**: Wax takes 74% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.23 ms | 3.60 ms | **met**: Wax takes 62% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, resuming each | 21 ns | 34 ns | **met**: Wax takes 60% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from the same text | 172 ns | 187 ns | **met**: Wax takes 92% of its time |
| 1. Faster than Luau's interpreter | patrolling scripts, loading each from a text of its own | 4.83 µs | 5.00 µs | **met**: Wax takes 97% of its time |
| 1. Faster than LuaJIT's interpreter | `fib(32)`, recursive | 30.81 ms | 36.06 ms | **met**: Wax takes 85% of its time |
| 1. Faster than LuaJIT's interpreter | n-body, 100,000 steps | 66.65 ms | 71.67 ms | **met**: Wax takes 93% of its time |
| 1. Faster than LuaJIT's interpreter | spectral-norm, size 200 | 18.61 ms | 45.64 ms | **met**: Wax takes 41% of its time |
| 1. Faster than LuaJIT's interpreter | fannkuch-redux, 9 | 104.75 ms | 147.37 ms | **met**: Wax takes 71% of its time |
| 1. Faster than LuaJIT's interpreter | word count, 500,000 words | 27.05 ms | 32.50 ms | **met**: Wax takes 83% of its time |
| 1. Faster than LuaJIT's interpreter | text building, 20,000 joins | 342.61 µs | 41.58 ms | **met**: Wax takes 0.8% of its time |
| 1. Faster than LuaJIT's interpreter | binary-trees, depth 14 | 99.73 ms | 130.10 ms | **met**: Wax takes 77% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, the whole run: loading 1000 from one text, resuming each 100 times | 2.23 ms | 2.94 ms | **met**: Wax takes 76% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, resuming each | 21 ns | 28 ns | **met**: Wax takes 75% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from the same text | 172 ns | 182 ns | **met**: Wax takes 95% of its time |
| 1. Faster than LuaJIT's interpreter | patrolling scripts, loading each from a text of its own | 4.83 µs | 1.83 µs | for reference, no target (D150) |
| D171, D176. Within twice LuaJIT loading its own bytecode | patrolling scripts, loading each from a compiled file of its own | 532 ns | 285 ns | **met**: Wax takes 187% of its time |
| D176. Loading the same compiled file again | patrolling scripts, each loaded from one `.waxc` file, reusing what was verified | 137 ns | | for reference, no target |
| D171. Checking a `.waxc` file's signature, once per file | patrolling scripts, loading each signed file of its own, signature checked | 40.25 µs | | for reference, no target |
| D152. A door call through C as fast as a Rust door | 1,000,000 door calls from a loop | 12 ns | a Rust door: 8 ns; LuaJIT's C API: 7 ns | an aim, no target: 1.5× a Rust door's time |
| D155. Faster than Python's built-ins | `HugeWhole`: 3000 factorial, 9,131 digits | 1.37 ms | 1.59 ms | **met**: Wax takes 86% of its time |
| D155. Faster than Python's built-ins | `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.39 ms | 1.52 ms | **met**: Wax takes 92% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 prices summed with tax, then split three ways | 117.64 ms | 169.11 ms | **met**: Wax takes 70% of its time |
| D155. Faster than Python's built-ins | `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 71.04 ms | 83.47 ms | **met**: Wax takes 85% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 437.24 µs | 918.88 µs | **met**: Wax takes 48% of its time |
| D155. Faster than Python's built-ins | `HugeDecimal`: the money sums, kept exactly | 93.40 ms | 166.25 ms | **met**: Wax takes 56% of its time |
| 2. Faster than V8 `--jitless` | `fib(32)`, recursive | 30.81 ms | 91.14 ms | **met**: Wax takes 34% of its time |
| 2. Faster than V8 `--jitless` | n-body, 100,000 steps | 66.65 ms | 190.15 ms | **met**: Wax takes 35% of its time |
| 2. Within twice V8 `--jitless` | word count, 500,000 words | 27.05 ms | 51.76 ms | **met**: Wax takes 52% of its time |
| 2. Within twice V8 `--jitless` | text building, 20,000 joins | 342.61 µs | 518.00 µs | **met**: Wax takes 66% of its time |
| 6. Pausing costs about as much as a call | patrolling scripts | resume 21 ns | host call 41 ns | **met** |
| 5. Thousands of scripts, each costing kilobytes | patrolling scripts | 512 B each, counted | | **met** |

Goal 3, full V8 with its JITs, has no target, nor has LuaJIT with its JIT (D142): their rows are in every table for reference. Loading a script from a text of its own is judged against Luau's interpreter alone, with LuaJIT's shown beside it (D150). Goal 7 holds for every number here: every check was on, except in the table that measures what the checks cost.

## `fib(32)`, recursive

Tests: calls. Purpose isolates calls alone: a procedure calling itself 1,000 deep, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 108.08 ms | 1.88 ms | 103.90 ms | 125.87 ms | 2.9 MB | 377 B | 682.29 µs cold, 541 ns warm; 2.78 ms as a whole process | 32.0 KB counted by Wax; 176 B compiled |
| Luau (interpreter) | Purpose | 196.57 ms | 302.39 µs | 196.00 ms | 196.94 ms | 4.8 MB | 339 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 89.63 ms | 486.58 µs | 88.89 ms | 89.41 ms | 1.9 MB | 339 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 44.37 ms | 171.51 µs | 44.13 ms | 45.93 ms | 2.2 MB | 339 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Purpose | 321.64 ms | 466.43 µs | 320.76 ms | 361.05 ms | 43.6 MB | 362 B | 18.44 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 124.65 ms | 498.23 µs | 123.98 ms | 127.70 ms | 49.2 MB | 362 B | 18.79 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 120.71 ms | 208.35 µs | 120.46 ms | 122.28 ms | 49.2 MB | 362 B | 19.01 ms as a whole process |  |
| Wax | Normal | 30.81 ms | 451.50 µs | 29.94 ms | 30.52 ms | 2.8 MB | 169 B | 83.04 µs cold, 333 ns warm; 2.59 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled |
| Luau (interpreter) | Normal | 71.12 ms | 303.81 µs | 70.64 ms | 71.21 ms | 4.9 MB | 143 B | 5.15 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 36.06 ms | 1.75 ms | 33.74 ms | 33.74 ms | 1.9 MB | 143 B | 2.42 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 6.96 ms | 18.36 µs | 6.94 ms | 7.19 ms | 2.2 MB | 143 B | 2.41 ms as a whole process |  |
| Node `--jitless` | Normal | 91.14 ms | 1.37 ms | 89.96 ms | 94.54 ms | 43.5 MB | 137 B | 18.70 ms as a whole process |  |
| Node (JIT), from cold | Normal | 9.75 ms | 385.65 µs | 9.39 ms | 9.80 ms | 48.7 MB | 137 B | 19.00 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 9.44 ms | 30.85 µs | 9.39 ms | 10.02 ms | 48.7 MB | 137 B | 19.04 ms as a whole process |  |
| Wax | Ceil.source | 30.80 ms | 537.90 µs | 29.81 ms | 30.98 ms | 2.8 MB | 169 B | 90.29 µs cold, 333 ns warm; 2.69 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 32.45 ms | 88.53 µs | 32.33 ms | 53.72 ms | 2.8 MB | 169 B | 925.83 µs cold, 16.29 µs warm; 2.57 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Wax | Ceil.program | 32.53 ms | 584.35 µs | 31.46 ms | 53.31 ms | 2.7 MB | 169 B | 843.67 µs cold, 459 ns warm; 2.80 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 30.90 ms | 451.22 µs | 30.38 ms | 52.00 ms | 2.7 MB | 169 B | 946.08 µs cold, 625 ns warm; 2.56 ms as a whole process | 2.0 KB counted by Wax; 88 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 72.11 ms | 306.17 µs | 71.68 ms | 71.44 ms | 4.8 MB | 143 B | 5.06 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 33.63 ms | 159.40 µs | 33.39 ms | 33.66 ms | 1.9 MB | 143 B | 2.43 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 7.15 ms | 47.93 µs | 7.10 ms | 7.33 ms | 2.2 MB | 143 B | 2.44 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 93.76 ms | 4.62 ms | 90.16 ms | 93.73 ms | 43.5 MB | 137 B | 19.12 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 9.85 ms | 341.89 µs | 9.59 ms | 10.11 ms | 48.7 MB | 137 B | 19.11 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 9.45 ms | 43.76 µs | 9.41 ms | 9.83 ms | 48.7 MB | 137 B | 18.84 ms as a whole process | the Normal program stands in |

## n-body, 100,000 steps

Tests: float maths, records. Purpose isolates float maths on one record's fields, 5,000,000 times, with nothing called or allocated.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 47.53 ms | 228.74 µs | 47.23 ms | 48.62 ms | 2.8 MB | 532 B | 172.50 µs cold, 250 ns warm; 2.67 ms as a whole process | 512 B counted by Wax; 232 B compiled |
| Luau (interpreter) | Purpose | 91.87 ms | 612.97 µs | 91.16 ms | 91.49 ms | 4.8 MB | 433 B | 5.42 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.69 ms | 2.42 ms | 76.40 ms | 77.13 ms | 1.9 MB | 433 B | 2.51 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.86 ms | 115.81 µs | 6.60 ms | 6.70 ms | 2.1 MB | 433 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Purpose | 243.73 ms | 602.27 µs | 242.68 ms | 244.72 ms | 46.7 MB | 460 B | 18.54 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 18.38 ms | 100.58 µs | 18.25 ms | 18.56 ms | 50.5 MB | 460 B | 18.98 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 18.52 ms | 162.79 µs | 18.28 ms | 18.95 ms | 50.4 MB | 460 B | 19.47 ms as a whole process |  |
| Wax | Normal | 66.65 ms | 390.91 µs | 65.87 ms | 66.97 ms | 3.4 MB | 3.4 KB | 282.38 µs cold, 292 ns warm; 2.91 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Luau (interpreter) | Normal | 69.75 ms | 167.94 µs | 69.44 ms | 69.91 ms | 4.8 MB | 3.0 KB | 5.22 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 71.67 ms | 346.26 µs | 71.19 ms | 71.66 ms | 1.9 MB | 3.0 KB | 2.50 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 5.17 ms | 17.08 µs | 5.15 ms | 5.39 ms | 2.2 MB | 3.0 KB | 2.43 ms as a whole process |  |
| Node `--jitless` | Normal | 190.15 ms | 18.29 ms | 177.29 ms | 180.18 ms | 48.3 MB | 3.1 KB | 20.99 ms as a whole process |  |
| Node (JIT), from cold | Normal | 4.08 ms | 155.19 µs | 3.89 ms | 6.32 ms | 52.2 MB | 3.1 KB | 21.88 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 4.03 ms | 146.95 µs | 3.88 ms | 6.40 ms | 52.0 MB | 3.1 KB | 20.36 ms as a whole process |  |
| Wax | Ceil.source | 57.52 ms | 918.58 µs | 55.98 ms | 58.98 ms | 3.3 MB | 3.4 KB | 282.29 µs cold, 417 ns warm; 2.99 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Wax | Ceil.machine | 59.54 ms | 2.94 ms | 56.65 ms | 60.73 ms | 3.4 MB | 3.4 KB | 286.04 µs cold, 1.00 µs warm; 3.14 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Wax | Ceil.program | 49.61 ms | 5.69 ms | 45.83 ms | 47.94 ms | 3.2 MB | 3.4 KB | 280.71 µs cold, 250 ns warm; 3.35 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Wax | Ceil.machine.program | 50.07 ms | 1.04 ms | 49.38 ms | 51.67 ms | 3.2 MB | 3.4 KB | 295.58 µs cold, 208 ns warm; 3.19 ms as a whole process | 2.0 KB counted by Wax; 1.8 KB compiled |
| Luau (interpreter) | Ceil.source | 71.15 ms | 383.59 µs | 70.57 ms | 70.01 ms | 4.8 MB | 3.0 KB | 5.94 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 72.57 ms | 563.79 µs | 71.84 ms | 73.23 ms | 1.9 MB | 3.0 KB | 2.54 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 5.28 ms | 47.73 µs | 5.21 ms | 5.42 ms | 2.2 MB | 3.0 KB | 2.61 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 185.70 ms | 4.84 ms | 178.98 ms | 186.26 ms | 48.2 MB | 3.1 KB | 19.30 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 3.87 ms | 14.95 µs | 3.84 ms | 5.66 ms | 52.1 MB | 3.1 KB | 20.24 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 3.88 ms | 19.92 µs | 3.85 ms | 5.77 ms | 52.1 MB | 3.1 KB | 19.16 ms as a whole process | the Normal program stands in |

## spectral-norm, size 200

Tests: loops, arrays, floats. Purpose isolates a loop over a list of 1,000 floats, 20,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 56.06 ms | 1.30 ms | 53.45 ms | 56.55 ms | 3.0 MB | 393 B | 111.25 µs cold, 375 ns warm; 2.66 ms as a whole process | 8.5 KB counted by Wax; 224 B compiled |
| Luau (interpreter) | Purpose | 45.17 ms | 244.50 µs | 44.90 ms | 44.88 ms | 4.8 MB | 366 B | 5.21 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 77.02 ms | 141.74 µs | 76.85 ms | 76.91 ms | 2.0 MB | 366 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 23.12 ms | 271.68 µs | 22.58 ms | 23.17 ms | 2.2 MB | 366 B | 2.34 ms as a whole process |  |
| Node `--jitless` | Purpose | 374.23 ms | 3.45 ms | 370.61 ms | 374.81 ms | 48.9 MB | 398 B | 19.07 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 34.15 ms | 36.93 ms | 15.64 ms | 20.01 ms | 52.8 MB | 398 B | 19.09 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 15.61 ms | 108.45 µs | 15.52 ms | 20.57 ms | 55.5 MB | 398 B | 19.26 ms as a whole process |  |
| Wax | Normal | 18.61 ms | 308.35 µs | 18.08 ms | 18.03 ms | 3.2 MB | 1.2 KB | 163.58 µs cold, 292 ns warm; 2.85 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled |
| Luau (interpreter) | Normal | 27.32 ms | 267.38 µs | 27.00 ms | 27.55 ms | 4.8 MB | 981 B | 5.31 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 45.64 ms | 512.86 µs | 44.66 ms | 47.23 ms | 2.0 MB | 981 B | 2.56 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 1.55 ms | 18.80 µs | 1.52 ms | 1.82 ms | 2.3 MB | 981 B | 2.48 ms as a whole process |  |
| Node `--jitless` | Normal | 63.39 ms | 378.79 µs | 62.93 ms | 63.56 ms | 48.6 MB | 1.0 KB | 19.67 ms as a whole process |  |
| Node (JIT), from cold | Normal | 1.63 ms | 227.50 µs | 1.48 ms | 3.15 ms | 52.8 MB | 1.0 KB | 19.71 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 1.66 ms | 60.68 µs | 1.59 ms | 3.24 ms | 52.8 MB | 1.0 KB | 19.29 ms as a whole process |  |
| Wax | Ceil.source | 18.55 ms | 272.08 µs | 18.18 ms | 19.28 ms | 3.2 MB | 1.2 KB | 152.08 µs cold, 333 ns warm; 2.79 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 18.77 ms | 136.58 µs | 18.63 ms | 18.82 ms | 3.2 MB | 1.2 KB | 142.92 µs cold, 959 ns warm; 2.90 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Wax | Ceil.program | 17.43 ms | 139.95 µs | 17.20 ms | 17.53 ms | 3.0 MB | 1.2 KB | 136.29 µs cold, 250 ns warm; 2.70 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 19.06 ms | 182.77 µs | 18.81 ms | 19.89 ms | 3.1 MB | 1.2 KB | 138.96 µs cold, 208 ns warm; 2.65 ms as a whole process | 9.2 KB counted by Wax; 992 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 27.08 ms | 296.41 µs | 26.57 ms | 26.85 ms | 4.8 MB | 981 B | 5.15 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 46.18 ms | 678.69 µs | 44.87 ms | 44.90 ms | 2.0 MB | 981 B | 2.52 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.54 ms | 14.46 µs | 1.52 ms | 1.80 ms | 2.3 MB | 981 B | 2.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 64.11 ms | 233.06 µs | 63.71 ms | 64.94 ms | 48.6 MB | 1.0 KB | 19.19 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 1.56 ms | 184.41 µs | 1.40 ms | 3.65 ms | 53.0 MB | 1.0 KB | 19.65 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 1.61 ms | 16.14 µs | 1.57 ms | 4.09 ms | 53.0 MB | 1.0 KB | 19.63 ms as a whole process | the Normal program stands in |

## fannkuch-redux, 9

Tests: array shuffling, `Whole` maths. Purpose isolates reversing a list of 16 whole numbers in place, 1,000,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 88.41 ms | 825.47 µs | 87.17 ms | 88.97 ms | 2.9 MB | 596 B | 123.83 µs cold, 333 ns warm; 2.66 ms as a whole process | 688 B counted by Wax; 288 B compiled |
| Luau (interpreter) | Purpose | 89.09 ms | 624.35 µs | 88.45 ms | 89.14 ms | 4.8 MB | 523 B | 5.26 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 52.74 ms | 565.07 µs | 51.92 ms | 53.64 ms | 1.9 MB | 523 B | 2.40 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 6.20 ms | 60.41 µs | 6.14 ms | 6.22 ms | 2.2 MB | 523 B | 2.43 ms as a whole process |  |
| Node `--jitless` | Purpose | 147.61 ms | 695.68 µs | 146.05 ms | 147.43 ms | 43.5 MB | 568 B | 18.78 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 3.67 ms | 32.62 µs | 3.64 ms | 4.78 ms | 49.4 MB | 568 B | 19.12 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 3.68 ms | 36.97 µs | 3.62 ms | 4.55 ms | 49.5 MB | 568 B | 18.99 ms as a whole process |  |
| Wax | Normal | 104.75 ms | 375.34 µs | 104.14 ms | 103.61 ms | 3.2 MB | 1.8 KB | 164.92 µs cold, 375 ns warm; 2.82 ms as a whole process | 984 B counted by Wax; 688 B compiled |
| Luau (interpreter) | Normal | 215.75 ms | 1.99 ms | 210.36 ms | 217.74 ms | 4.8 MB | 1.7 KB | 5.22 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 147.37 ms | 509.71 µs | 146.26 ms | 145.77 ms | 2.0 MB | 1.7 KB | 2.43 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 49.12 ms | 436.66 µs | 48.66 ms | 49.01 ms | 2.3 MB | 1.7 KB | 2.51 ms as a whole process |  |
| Node `--jitless` | Normal | 155.46 ms | 564.02 µs | 154.31 ms | 155.15 ms | 45.6 MB | 1.7 KB | 19.02 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.09 ms | 97.10 µs | 13.92 ms | 16.19 ms | 52.1 MB | 1.7 KB | 19.04 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 14.76 ms | 91.28 µs | 14.63 ms | 16.33 ms | 52.3 MB | 1.7 KB | 20.10 ms as a whole process |  |
| Wax | Ceil.source | 104.54 ms | 597.79 µs | 103.98 ms | 104.48 ms | 3.2 MB | 1.8 KB | 169.88 µs cold, 208 ns warm; 2.72 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 108.91 ms | 657.67 µs | 107.91 ms | 108.21 ms | 3.1 MB | 1.8 KB | 154.04 µs cold, 958 ns warm; 2.83 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Wax | Ceil.program | 100.07 ms | 634.63 µs | 98.98 ms | 99.82 ms | 2.9 MB | 1.8 KB | 139.17 µs cold, 250 ns warm; 2.85 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 103.63 ms | 567.45 µs | 102.89 ms | 104.83 ms | 3.0 MB | 1.8 KB | 142.46 µs cold, 250 ns warm; 2.84 ms as a whole process | 984 B counted by Wax; 688 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 216.27 ms | 2.70 ms | 209.97 ms | 214.12 ms | 4.9 MB | 1.7 KB | 5.29 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 149.81 ms | 985.05 µs | 148.21 ms | 145.94 ms | 2.0 MB | 1.7 KB | 2.64 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 49.31 ms | 537.88 µs | 48.09 ms | 49.30 ms | 2.3 MB | 1.7 KB | 2.65 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 159.55 ms | 13.80 ms | 151.01 ms | 159.10 ms | 45.5 MB | 1.7 KB | 18.65 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.93 ms | 115.43 µs | 14.73 ms | 16.72 ms | 52.0 MB | 1.7 KB | 19.51 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 14.19 ms | 293.37 µs | 13.92 ms | 15.67 ms | 52.2 MB | 1.7 KB | 19.74 ms as a whole process | the Normal program stands in |

## word count, 500,000 words

Tests: `Dict` and `Text`. Purpose isolates looking up and counting 1,000 text keys in a `Dict`, 5,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 126.68 ms | 1.08 ms | 125.19 ms | 125.12 ms | 4.2 MB | 533 B | 162.21 µs cold, 417 ns warm; 2.81 ms as a whole process | 793.6 KB counted by Wax; 336 B compiled |
| Luau (interpreter) | Purpose | 63.83 ms | 1.26 ms | 62.85 ms | 63.64 ms | 4.8 MB | 540 B | 5.26 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 35.18 ms | 1.27 ms | 32.26 ms | 32.02 ms | 2.2 MB | 540 B | 2.49 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 4.84 ms | 23.73 µs | 4.81 ms | 4.95 ms | 2.4 MB | 540 B | 2.52 ms as a whole process |  |
| Node `--jitless` | Purpose | 241.92 ms | 1.45 ms | 240.18 ms | 243.20 ms | 48.8 MB | 514 B | 19.35 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 90.42 ms | 24.56 ms | 68.69 ms | 89.87 ms | 52.7 MB | 514 B | 19.09 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 68.32 ms | 984.38 µs | 66.94 ms | 92.37 ms | 52.8 MB | 514 B | 19.72 ms as a whole process |  |
| Wax | Normal | 27.05 ms | 143.47 µs | 26.78 ms | 26.89 ms | 4.1 MB | 538 B | 113.21 µs cold, 333 ns warm; 2.69 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled |
| Luau (interpreter) | Normal | 31.06 ms | 411.30 µs | 30.35 ms | 30.61 ms | 6.0 MB | 478 B | 5.69 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 32.50 ms | 834.42 µs | 31.00 ms | 33.36 ms | 3.0 MB | 478 B | 2.53 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 12.24 ms | 529.12 µs | 11.52 ms | 13.72 ms | 3.4 MB | 478 B | 2.36 ms as a whole process |  |
| Node `--jitless` | Normal | 51.76 ms | 274.82 µs | 51.40 ms | 53.18 ms | 56.7 MB | 477 B | 20.23 ms as a whole process |  |
| Node (JIT), from cold | Normal | 35.59 ms | 284.48 µs | 35.13 ms | 37.04 ms | 62.6 MB | 477 B | 19.28 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 35.93 ms | 758.87 µs | 35.03 ms | 36.52 ms | 88.7 MB | 477 B | 18.94 ms as a whole process |  |
| Wax | Ceil.source | 27.51 ms | 136.60 µs | 27.33 ms | 26.91 ms | 4.1 MB | 538 B | 105.58 µs cold, 375 ns warm; 2.74 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 27.73 ms | 129.32 µs | 27.51 ms | 27.99 ms | 4.1 MB | 538 B | 121.42 µs cold, 1.00 µs warm; 2.56 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Wax | Ceil.program | 24.05 ms | 133.91 µs | 23.84 ms | 24.36 ms | 4.1 MB | 538 B | 105.88 µs cold, 292 ns warm; 2.70 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 24.54 ms | 181.57 µs | 24.19 ms | 24.42 ms | 4.1 MB | 538 B | 117.79 µs cold, 250 ns warm; 2.84 ms as a whole process | 625.5 KB counted by Wax; 296 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 30.75 ms | 417.02 µs | 30.22 ms | 30.77 ms | 6.0 MB | 478 B | 5.45 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 32.39 ms | 949.88 µs | 30.04 ms | 31.57 ms | 3.0 MB | 478 B | 2.63 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 12.27 ms | 643.21 µs | 11.51 ms | 13.70 ms | 3.4 MB | 478 B | 2.37 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 52.55 ms | 502.58 µs | 51.93 ms | 52.89 ms | 56.8 MB | 477 B | 19.18 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 36.05 ms | 407.57 µs | 35.22 ms | 36.31 ms | 62.6 MB | 477 B | 19.75 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 35.49 ms | 298.59 µs | 35.00 ms | 36.30 ms | 88.2 MB | 477 B | 19.52 ms as a whole process | the Normal program stands in |

## text building, 20,000 joins

Tests: joining text many times. Purpose isolates one text growing by two characters, 30,000 times.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 292.50 µs | 3.18 µs | 288.33 µs | 326.25 µs | 3.2 MB | 215 B | 114.12 µs cold, 292 ns warm; 2.56 ms as a whole process | 64.6 KB counted by Wax; 96 B compiled |
| Luau (interpreter) | Purpose | 144.09 ms | 728.00 µs | 143.27 ms | 145.36 ms | 11.0 MB | 195 B | 5.28 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 32.19 ms | 63.14 µs | 32.08 ms | 31.17 ms | 3.6 MB | 195 B | 2.48 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 20.60 ms | 64.39 µs | 20.49 ms | 20.87 ms | 3.7 MB | 195 B | 2.51 ms as a whole process |  |
| Node `--jitless` | Purpose | 374.68 µs | 147.84 µs | 246.00 µs | 1.04 ms | 52.5 MB | 223 B | 19.02 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 219.23 µs | 152.81 µs | 83.83 µs | 916.25 µs | 57.9 MB | 223 B | 18.99 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 140.09 µs | 121.17 µs | 62.83 µs | 920.04 µs | 58.0 MB | 223 B | 19.26 ms as a whole process |  |
| Wax | Normal | 342.61 µs | 6.30 µs | 335.54 µs | 379.83 µs | 3.4 MB | 182 B | 80.67 µs cold, 333 ns warm; 2.67 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled |
| Luau (interpreter) | Normal | 162.21 ms | 770.46 µs | 160.47 ms | 162.94 ms | 11.3 MB | 138 B | 5.09 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 41.58 ms | 291.35 µs | 41.26 ms | 41.83 ms | 5.1 MB | 138 B | 2.38 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 19.70 ms | 184.03 µs | 19.35 ms | 20.16 ms | 5.4 MB | 138 B | 2.40 ms as a whole process |  |
| Node `--jitless` | Normal | 518.00 µs | 338.59 µs | 283.96 µs | 1.12 ms | 61.4 MB | 155 B | 18.86 ms as a whole process |  |
| Node (JIT), from cold | Normal | 501.54 µs | 494.69 µs | 127.04 µs | 1.09 ms | 65.3 MB | 155 B | 19.02 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 271.19 µs | 318.65 µs | 122.46 µs | 1.13 ms | 69.4 MB | 155 B | 18.66 ms as a whole process |  |
| Wax | Ceil.source | 349.69 µs | 4.76 µs | 344.21 µs | 376.62 µs | 3.4 MB | 182 B | 81.08 µs cold, 417 ns warm; 2.45 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 347.35 µs | 3.55 µs | 343.17 µs | 380.67 µs | 3.4 MB | 182 B | 94.21 µs cold, 1.29 µs warm; 2.50 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Wax | Ceil.program | 284.05 µs | 4.12 µs | 280.17 µs | 325.96 µs | 3.2 MB | 182 B | 63.96 µs cold, 333 ns warm; 2.47 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 283.86 µs | 6.15 µs | 276.54 µs | 339.50 µs | 3.3 MB | 182 B | 66.54 µs cold, 333 ns warm; 2.48 ms as a whole process | 128.6 KB counted by Wax; 80 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 162.24 ms | 513.17 µs | 161.75 ms | 163.73 ms | 12.2 MB | 138 B | 5.06 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 42.07 ms | 249.89 µs | 41.65 ms | 41.12 ms | 4.9 MB | 138 B | 2.74 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 19.56 ms | 277.79 µs | 19.02 ms | 20.42 ms | 5.3 MB | 138 B | 2.38 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 518.52 µs | 343.53 µs | 279.54 µs | 1.15 ms | 61.5 MB | 155 B | 19.02 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 510.09 µs | 483.98 µs | 142.00 µs | 1.07 ms | 65.2 MB | 155 B | 18.87 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 279.80 µs | 330.98 µs | 123.25 µs | 1.08 ms | 69.4 MB | 155 B | 18.84 ms as a whole process | the Normal program stands in |

## binary-trees, depth 14

Tests: making and dropping heap values (joined by D72). Purpose isolates making and dropping a tree 10 deep, 2,000 times, never reading it.

| Engine | Kind | Mean | Deviation | Least | First run | Peak memory | Program | Load | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 71.82 ms | 398.61 µs | 71.18 ms | 72.56 ms | 3.1 MB | 519 B | 127.62 µs cold, 209 ns warm; 2.66 ms as a whole process | 65.9 KB counted by Wax; 216 B compiled |
| Luau (interpreter) | Purpose | 94.53 ms | 722.02 µs | 93.87 ms | 94.22 ms | 4.8 MB | 411 B | 5.13 ms as a whole process |  |
| LuaJIT `-joff` | Purpose | 117.22 ms | 2.48 ms | 111.99 ms | 118.36 ms | 2.6 MB | 411 B | 2.59 ms as a whole process |  |
| LuaJIT (JIT) | Purpose | 94.35 ms | 483.57 µs | 93.66 ms | 93.87 ms | 2.9 MB | 411 B | 2.42 ms as a whole process |  |
| Node `--jitless` | Purpose | 76.55 ms | 743.78 µs | 75.65 ms | 78.95 ms | 59.8 MB | 444 B | 18.58 ms as a whole process |  |
| Node (JIT), from cold | Purpose | 10.51 ms | 267.07 µs | 10.23 ms | 12.94 ms | 65.8 MB | 444 B | 19.09 ms as a whole process |  |
| Node (JIT), warmed up | Purpose | 10.67 ms | 784.83 µs | 10.17 ms | 13.04 ms | 114.9 MB | 444 B | 20.55 ms as a whole process |  |
| Wax | Normal | 99.73 ms | 1.29 ms | 97.93 ms | 101.47 ms | 7.9 MB | 956 B | 135.92 µs cold, 292 ns warm; 2.75 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled |
| Luau (interpreter) | Normal | 134.39 ms | 1.22 ms | 133.05 ms | 130.97 ms | 43.7 MB | 799 B | 5.31 ms as a whole process |  |
| LuaJIT `-joff` | Normal | 130.10 ms | 1.93 ms | 128.03 ms | 129.16 ms | 41.6 MB | 799 B | 2.61 ms as a whole process |  |
| LuaJIT (JIT) | Normal | 82.45 ms | 916.99 µs | 80.35 ms | 82.04 ms | 39.7 MB | 799 B | 2.62 ms as a whole process |  |
| Node `--jitless` | Normal | 116.00 ms | 2.77 ms | 112.40 ms | 118.08 ms | 213.3 MB | 878 B | 18.94 ms as a whole process |  |
| Node (JIT), from cold | Normal | 14.31 ms | 963.17 µs | 13.36 ms | 17.21 ms | 129.5 MB | 878 B | 20.26 ms as a whole process |  |
| Node (JIT), warmed up | Normal | 13.42 ms | 62.14 µs | 13.30 ms | 17.69 ms | 224.2 MB | 878 B | 19.52 ms as a whole process |  |
| Wax | Ceil.source | 100.84 ms | 791.65 µs | 99.52 ms | 102.88 ms | 8.0 MB | 956 B | 127.58 µs cold, 333 ns warm; 2.64 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Wax | Ceil.machine | 100.38 ms | 687.15 µs | 99.04 ms | 100.79 ms | 8.0 MB | 956 B | 150.17 µs cold, 1.04 µs warm; 2.80 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Wax | Ceil.program | 93.16 ms | 781.81 µs | 91.75 ms | 94.61 ms | 7.8 MB | 956 B | 114.42 µs cold, 209 ns warm; 2.63 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Wax | Ceil.machine.program | 99.18 ms | 971.48 µs | 97.47 ms | 102.19 ms | 7.9 MB | 956 B | 115.83 µs cold, 166 ns warm; 2.66 ms as a whole process | 4.0 MB counted by Wax; 592 B compiled; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 133.72 ms | 2.05 ms | 131.18 ms | 132.55 ms | 38.5 MB | 799 B | 5.29 ms as a whole process | the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 128.93 ms | 1.32 ms | 126.86 ms | 127.46 ms | 41.6 MB | 799 B | 2.62 ms as a whole process | the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 82.47 ms | 1.08 ms | 80.73 ms | 83.03 ms | 44.6 MB | 799 B | 2.60 ms as a whole process | the Normal program stands in |
| Node `--jitless` | Ceil.source | 114.34 ms | 2.18 ms | 112.01 ms | 118.80 ms | 213.6 MB | 878 B | 18.91 ms as a whole process | the Normal program stands in |
| Node (JIT), from cold | Ceil.source | 14.49 ms | 1.03 ms | 13.54 ms | 17.15 ms | 129.9 MB | 878 B | 18.63 ms as a whole process | the Normal program stands in |
| Node (JIT), warmed up | Ceil.source | 13.43 ms | 168.44 µs | 13.28 ms | 16.97 ms | 224.2 MB | 878 B | 19.72 ms as a whole process | the Normal program stands in |

## 1,000 patrolling scripts

Tests: loading, pausing, footprint. Each script is loaded from its own copy of the same text and paused in its patrol; then every one is resumed 100 times, and then a procedure of each is called 100 times. Purpose isolates pausing: one script, resumed 1000000 times. Each row is 10 whole processes. Loading is timed three ways: every script from a text of its own, so that nothing compiled is reused; every script from the same text, where Wax reuses what it compiled, and the other engines compile once and make each script from that; and every script from a compiled file of its own, a `.waxc` for Wax and `string.dump`'s bytecode, as `luajit -b` writes it, for LuaJIT (D171). Wax frees each script loaded from a text or a file of its own inside its timing, before loading the next.

| Engine | Kind | Load, each from its own text | Load, each from the same text | Load, each from a compiled file (D141) | Resume, each | Call, each | Memory, each script | Peak memory | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Wax | Purpose | 66.57 µs (±11.75 µs, least 51.33 µs) | 21.27 µs (±6.58 µs, least 13.38 µs) | 8.74 µs (±5.57 µs, least 5.83 µs) | 19 ns (±0 ns, least 19 ns) | 42 ns (±1 ns, least 40 ns) | 512 B | 4.2 MB | counted by Wax |
| Luau (interpreter) | Purpose | 10.67 µs (±718 ns, least 9.25 µs) | 17.82 µs (±1.30 µs, least 16.33 µs) | ? | 42 ns (±1 ns, least 40 ns) | 21 ns (±2 ns, least 19 ns) | 3.0 KB | 4.8 MB | the engine's own heap |
| LuaJIT `-joff` | Purpose | 3.90 µs (±876 ns, least 3.00 µs) | 7.60 µs (±1.96 µs, least 5.00 µs) | 1.40 µs (±699 ns, least 1.00 µs) | 22 ns (±1 ns, least 21 ns) | 14 ns (±1 ns, least 13 ns) | 1.2 KB | 1.9 MB | the engine's own heap |
| LuaJIT (JIT) | Purpose | 3.60 µs (±516 ns, least 3.00 µs) | 8.00 µs (±1.70 µs, least 5.00 µs) | 1.20 µs (±632 ns, least 0 ns) | 13 ns (±0 ns, least 13 ns) | 0 ns (±0 ns, least 0 ns) | 1.2 KB | 2.2 MB | the engine's own heap |
| Node `--jitless` | Purpose | 34.56 µs (±4.79 µs, least 21.50 µs) | 63.52 µs (±76.85 µs, least 36.08 µs) | ? | 57 ns (±1 ns, least 55 ns) | 30 ns (±1 ns, least 29 ns) | 0 B | 46.0 MB | the engine's own heap |
| Node (JIT) | Purpose | 28.64 µs (±8.04 µs, least 18.88 µs) | 38.55 µs (±2.64 µs, least 35.12 µs) | ? | 9 ns (±0 ns, least 8 ns) | 2 ns (±0 ns, least 2 ns) | 0 B | 53.2 MB | the engine's own heap |
| Wax | Normal | 4.83 µs (±64 ns, least 4.69 µs) | 172 ns (±14 ns, least 151 ns) | 532 ns (±8 ns, least 524 ns) | 21 ns (±1 ns, least 20 ns) | 41 ns (±0 ns, least 41 ns) | 512 B | 10.2 MB | counted by Wax |
| Luau (interpreter) | Normal | 5.00 µs (±224 ns, least 4.72 µs) | 187 ns (±6 ns, least 177 ns) | ? | 34 ns (±1 ns, least 33 ns) | 18 ns (±0 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap |
| LuaJIT `-joff` | Normal | 1.83 µs (±49 ns, least 1.79 µs) | 182 ns (±6 ns, least 174 ns) | 285 ns (±10 ns, least 275 ns) | 28 ns (±1 ns, least 27 ns) | 17 ns (±2 ns, least 14 ns) | 1.0 KB | 5.2 MB | the engine's own heap |
| LuaJIT (JIT) | Normal | 1.84 µs (±34 ns, least 1.78 µs) | 367 ns (±33 ns, least 341 ns) | 471 ns (±101 ns, least 243 ns) | 95 ns (±2 ns, least 93 ns) | 110 ns (±16 ns, least 101 ns) | 1.0 KB | 6.0 MB | the engine's own heap |
| Node `--jitless` | Normal | 1.20 µs (±62 ns, least 1.15 µs) | 709 ns (±52 ns, least 639 ns) | ? | 51 ns (±2 ns, least 49 ns) | 24 ns (±0 ns, least 23 ns) | 643 B | 48.8 MB | the engine's own heap |
| Node (JIT) | Normal | 1.11 µs (±22 ns, least 1.08 µs) | 697 ns (±35 ns, least 653 ns) | ? | 22 ns (±1 ns, least 22 ns) | 6 ns (±1 ns, least 5 ns) | 643 B | 56.1 MB | the engine's own heap |
| Wax | Ceil.source | 4.82 µs (±43 ns, least 4.76 µs) | 170 ns (±12 ns, least 155 ns) | 554 ns (±62 ns, least 522 ns) | 21 ns (±1 ns, least 20 ns) | 41 ns (±0 ns, least 40 ns) | 512 B | 10.2 MB | counted by Wax; the Normal program stands in |
| Wax | Ceil.machine | 4.71 µs (±44 ns, least 4.65 µs) | 159 ns (±12 ns, least 146 ns) | 525 ns (±14 ns, least 506 ns) | 20 ns (±1 ns, least 20 ns) | 42 ns (±1 ns, least 41 ns) | 512 B | 10.1 MB | counted by Wax; the Normal program stands in |
| Wax | Ceil.program | 4.05 µs (±44 ns, least 3.99 µs) | 149 ns (±12 ns, least 133 ns) | 470 ns (±10 ns, least 444 ns) | 17 ns (±0 ns, least 16 ns) | 37 ns (±0 ns, least 37 ns) | 512 B | 10.1 MB | counted by Wax; the Normal program stands in |
| Wax | Ceil.machine.program | 4.14 µs (±41 ns, least 4.08 µs) | 152 ns (±7 ns, least 143 ns) | 468 ns (±19 ns, least 441 ns) | 17 ns (±0 ns, least 17 ns) | 38 ns (±1 ns, least 37 ns) | 512 B | 10.1 MB | counted by Wax; the Normal program stands in |
| Luau (interpreter) | Ceil.source | 4.89 µs (±159 ns, least 4.71 µs) | 195 ns (±9 ns, least 186 ns) | ? | 35 ns (±1 ns, least 33 ns) | 18 ns (±1 ns, least 17 ns) | 2.0 KB | 6.3 MB | the engine's own heap; the Normal program stands in |
| LuaJIT `-joff` | Ceil.source | 1.83 µs (±33 ns, least 1.77 µs) | 189 ns (±10 ns, least 177 ns) | 281 ns (±8 ns, least 274 ns) | 28 ns (±1 ns, least 26 ns) | 15 ns (±3 ns, least 13 ns) | 1.0 KB | 5.2 MB | the engine's own heap; the Normal program stands in |
| LuaJIT (JIT) | Ceil.source | 1.85 µs (±34 ns, least 1.80 µs) | 353 ns (±17 ns, least 325 ns) | 464 ns (±106 ns, least 256 ns) | 97 ns (±4 ns, least 93 ns) | 110 ns (±14 ns, least 100 ns) | 1.0 KB | 6.0 MB | the engine's own heap; the Normal program stands in |
| Node `--jitless` | Ceil.source | 1.18 µs (±17 ns, least 1.15 µs) | 678 ns (±16 ns, least 651 ns) | ? | 50 ns (±1 ns, least 49 ns) | 24 ns (±1 ns, least 23 ns) | 643 B | 48.7 MB | the engine's own heap; the Normal program stands in |
| Node (JIT) | Ceil.source | 1.11 µs (±30 ns, least 1.06 µs) | 678 ns (±18 ns, least 661 ns) | ? | 23 ns (±2 ns, least 22 ns) | 7 ns (±1 ns, least 5 ns) | 643 B | 56.2 MB | the engine's own heap; the Normal program stands in |

## What Wax's checks cost (Purpose)

D58 asks for the cost of Wax's checks to be measured, not guessed. Each row runs a Normal program on a build of the VM with one kind of check left out. Those builds exist only for this table: a script never runs without its checks (D17).

| Benchmark | With every check | Without the-float-check |
|---|---|---|
| `fib(32)`, recursive | 30.84 ms | 31.53 ms (+2.2%) |
| n-body, 100,000 steps | 66.69 ms | 60.74 ms (-8.9%) |
| spectral-norm, size 200 | 18.87 ms | 18.23 ms (-3.4%) |
| fannkuch-redux, 9 | 106.98 ms | 103.73 ms (-3.0%) |
| word count, 500,000 words | 26.60 ms | 26.65 ms (+0.2%) |
| text building, 20,000 joins | 346.32 µs | 333.14 µs (-3.8%) |
| binary-trees, depth 14 | 98.88 ms | 99.43 ms (+0.6%) |

## The big number types against Python (D155, D159), Normal

Python's built-ins are the bar: `int` for `HugeWhole`, and `decimal` for `Decimal` (set to decimal128's rules) and `HugeDecimal` (with room for every digit, so it's exact). `gmpy2`, Python's binding to GMP, is shown for reference, with no target; it has no decimal type, so it runs the `HugeWhole` programs only. The `Float` row shows what exactness costs.

| Program | Wax mean | Wax deviation | Wax least | Wax peak memory | Python mean | Python least | `gmpy2` mean |
|---|---|---|---|---|---|---|---|
| `HugeWhole`: 3000 factorial, 9,131 digits | 1.37 ms | 12.58 µs | 1.35 ms | 3.8 MB | 1.59 ms | 1.56 ms | 304.71 µs |
| `HugeWhole`: an idle game's gold over 20,000 ticks, upgrades doubling it, 604 digits | 1.39 ms | 19.74 µs | 1.37 ms | 3.6 MB | 1.52 ms | 1.50 ms | 1.08 ms |
| `Decimal`: 1,000,000 prices summed with tax, then split three ways | 117.64 ms | 310.54 µs | 116.94 ms | 3.0 MB | 169.11 ms | 166.10 ms | – |
| `Decimal`: 1,000,000 days of interest compounding, a cent put in each day | 71.04 ms | 373.20 µs | 70.32 ms | 3.0 MB | 83.47 ms | 82.76 ms | – |
| `HugeDecimal`: 2,000 years of exact 5% interest, 4,048 digits | 437.24 µs | 9.98 µs | 416.75 µs | 3.6 MB | 918.88 µs | 900.33 µs | – |
| `HugeDecimal`: the money sums, kept exactly | 93.40 ms | 2.15 ms | 89.36 ms | 3.0 MB | 166.25 ms | 165.62 ms | – |
| the money sums in `Float`s, as a yardstick | 10.25 ms | 134.02 µs | 10.10 ms | 2.8 MB | – | – | – |

## Door calls through C (D136, D152), Purpose

A loop calling a host door 1,000,000 times, each call handing it two whole numbers it multiplies and adds to a total; the time is the best of seven runs, divided by the calls. Wax's door is written in C, through Wax's C API (`crates/wax-c/tests/api_test.c`), and in Rust, which the C API aims to match. `LuaJIT`'s is a C function registered with its C API, reading its numbers with `luaL_checkinteger`, as Wax's door checks their kinds (`bench/drivers/doors.luajit.c`). Luau's C library isn't installed here, so it has no row.

| Engine | Door written in | Each call | Notes |
|---|---|---|---|
| Wax | C | 12 ns | |
| Wax | Rust | 8 ns | |
| LuaJIT `-joff` | C | 7 ns | |
| LuaJIT (JIT) | C | 7 ns | |
