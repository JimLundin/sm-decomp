#!/usr/bin/env bash
# Run the clean-room scanner from the repository root.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
exec cargo run -q --profile verify -p tools --no-default-features --bin cleanroom -- "$@"
