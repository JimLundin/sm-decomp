#!/usr/bin/env bash
# Run Clippy with the ceilings used for clean-room exports.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
packages=()
for manifest in crates/clean/*/Cargo.toml; do
  crate=${manifest%/Cargo.toml}
  packages+=(-p "${crate##*/}")
done
export CLIPPY_CONF_DIR="$PWD/tools/cleanroom"
exec "${CARGO:-cargo}" clippy --locked "${packages[@]}" --all-targets --no-deps "$@"
