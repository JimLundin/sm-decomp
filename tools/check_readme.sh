#!/usr/bin/env bash
# check_readme.sh: README.md is what `tools report --write` generates for this tree
# (verify::report), and the report measured beside it still generates.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
exec "${CARGO:-cargo}" run --locked -q -p tools --bin tools -- report --check
