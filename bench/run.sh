#!/bin/sh
# Builds Seal's benchmark host in each way D58's kinds need, then runs every benchmark and writes bench/RESULTS.md.
# Run from the repository's folder: sh bench/run.sh
set -e
cores=3

# trace: D159. Python with `gmpy2`, for the big number types' bar and reference rows, in a virtual environment of the
# benchmarks' own, made once.
if [ ! -x target/bench-python/bin/python ]; then
    python3 -m venv target/bench-python
    target/bench-python/bin/pip install -q gmpy2
fi

# The usual build: the baseline CPU, no profile.
cargo build -q -j "$cores" --release -p seal-bench

# Ceil.machine: the VM built for this exact CPU.
RUSTFLAGS="-C target-cpu=native" cargo build -q -j "$cores" --release -p seal-bench --target-dir target/variants/machine
variants="--variant Ceil.machine=target/variants/machine/release/seal-bench"

# What the float check costs (D58): the VM built without it, for measuring only.
RUSTFLAGS="--cfg seal_measure_without_float_checks" cargo build -q -j "$cores" --release -p seal-bench --target-dir target/variants/no-float-checks
variants="$variants --variant Checks.the-float-check=target/variants/no-float-checks/release/seal-bench"

# Ceil.program and Ceil.machine.program: the VM built with a profile of the benchmarks' own run. This needs
# `llvm-profdata`: Rust's own when it's installed, or the one that comes with Xcode, which reads the same profiles.
# Without either, these two kinds are left out.
profdata=$(find "$(rustc --print sysroot)" -name llvm-profdata 2>/dev/null | head -1)
if [ -z "$profdata" ]; then
    profdata=$(xcrun --find llvm-profdata 2>/dev/null || true)
fi
if [ -n "$profdata" ]; then
    for kind in program machine.program; do
        flags=""
        if [ "$kind" = "machine.program" ]; then flags="-C target-cpu=native"; fi
        profiles="$PWD/target/variants/$kind-profiles"
        rm -rf "$profiles"
        RUSTFLAGS="$flags -C profile-generate=$profiles" cargo build -q -j "$cores" --release -p seal-bench --target-dir "target/variants/$kind-measuring"
        measuring="target/variants/$kind-measuring/release/seal-bench"
        "$measuring" seal bench/programs/fib.normal.seal 32 1 3 > /dev/null
        "$measuring" seal bench/programs/nbody.normal.seal 100000 1 3 > /dev/null
        "$measuring" seal bench/programs/spectral.normal.seal 200 1 3 > /dev/null
        "$measuring" seal bench/programs/fannkuch.normal.seal 9 1 3 > /dev/null
        "$measuring" seal bench/programs/wordcount.normal.seal 500000 1 3 > /dev/null
        "$measuring" seal bench/programs/textbuild.normal.seal 20000 1 3 > /dev/null
        "$measuring" seal bench/programs/trees.normal.seal 14 1 3 > /dev/null
        "$measuring" patrol bench/programs/patrol.normal.seal 1000 100 > /dev/null
        "$profdata" merge -o "$profiles/merged.profdata" "$profiles"
        RUSTFLAGS="$flags -C profile-use=$profiles/merged.profdata" cargo build -q -j "$cores" --release -p seal-bench --target-dir "target/variants/$kind"
        variants="$variants --variant Ceil.$kind=target/variants/$kind/release/seal-bench"
    done
else
    echo "llvm-profdata isn't installed: Ceil.program and Ceil.machine.program are left out"
fi

# shellcheck disable=SC2086 # the variants are separate words on purpose
./target/release/seal-bench report $variants "$@"
