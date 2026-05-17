#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULT_DIR="$ROOT/results"
mkdir -p "$RESULT_DIR"

cases=(
  "itertools safe crates/itertools-safe bypasser_target"
  "itertools unsafe crates/itertools-unsafe bypasser_target"
  "rand safe crates/rand-safe bypasser_target"
  "rand unsafe crates/rand-unsafe bypasser_target"
  "ring safe crates/ring-safe/bench bypasser_hkdf"
  "ring unsafe crates/ring-unsafe/bench bypasser_hkdf"
  "arrayvec safe crates/arrayvec-safe bypasser_target"
  "arrayvec unsafe crates/arrayvec-unsafe bypasser_target"
  "bit-vec safe crates/bit-vec-safe bypasser_target"
  "bit-vec unsafe crates/bit-vec-unsafe bypasser_target"
)

for opt in 1 2 3; do
  for entry in "${cases[@]}"; do
    read -r crate variant relpath bench_name <<< "$entry"
    workdir="$ROOT/$relpath"
    log="$RESULT_DIR/${crate}_${variant}_O${opt}.log"
    target_dir="$ROOT/target/${crate}-${variant}-O${opt}"

    echo "==> ${crate} ${variant} opt-level=${opt}"
    (
      cd "$workdir"
      CARGO_TARGET_DIR="$target_dir" \
      CARGO_PROFILE_BENCH_OPT_LEVEL="$opt" \
      cargo bench --bench "$bench_name" -- --noplot
    ) | tee "$log"
  done
done
