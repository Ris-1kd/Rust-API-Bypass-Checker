#!/bin/bash

# This file is to build the target crate for mir_wrapper to debug.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  cdto.sh [DIRECTORY]

Description:
  Change into DIRECTORY. If DIRECTORY is omitted, change into $HOME.

Examples:
  source ./cdto.sh /var/log
  . ./cdto.sh ~/workspace
EOF
}

# Help
if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  return 0 2>/dev/null || exit 0
fi

# Default directory
target="${1:-$HOME}"

# Expand ~ manually (bash won't expand it inside quotes reliably in all cases)
if [[ "$target" == "~" ]]; then
  target="$HOME"
elif [[ "$target" == "~/"* ]]; then
  target="$HOME/${target#~/}"
fi

# Validate
if [[ ! -d "$target" ]]; then
  echo "Error: not a directory: $target" >&2
  return 2 2>/dev/null || exit 2
fi

# Change directory
cd "$target"

# Optional: show where we are
pwd

# build the target crate
rustup override set nightly-2025-01-10
rm -rf target
export RUSTC_WRAPPER="/home/yunlong/workspace/Bypassing/Rust-API-Bypass/target/debug/mir_wrapper"
export MIR_WRAPPER_DUMP="/var/tmp/inv.json"
export MIR_CHECKER_ARGS='["--show_entries"]'

# env | grep RUSTC

cargo +nightly-2025-01-10 build -vv