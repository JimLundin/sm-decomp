#!/usr/bin/env bash
# Require every named domain exactly once, and prove the selected test actually ran.
set -euo pipefail
: "${SM_CACHE:?SM_CACHE is required}" "${SM_ROM:?SM_ROM is required}"
receipt=$(mktemp)
receipt_resolved=$(cd "$(dirname "$receipt")" && printf '%s/%s' "$(pwd -P)" "$(basename "$receipt")")
cleanup() {
  "${SM_PY:-python3}" - "$receipt" "$receipt_resolved" <<'PY'
import pathlib
import sys

receipt = pathlib.Path(sys.argv[1])
if receipt.is_symlink() or receipt.resolve(strict=True) != pathlib.Path(sys.argv[2]).absolute():
    raise SystemExit("free receipt path changed")
receipt.unlink()
PY
}
trap cleanup EXIT
"${CARGO:-cargo}" test --locked -p verify --profile verify --lib \
  harness::free::tests::every_domain_runs_free_against_the_cache -- --exact --nocapture \
  | tee "$receipt"
grep -Eq '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out; finished in ' "$receipt"
domains=('game time' 'samus movement' world-camera interface-hud world-blocks
  run-progress samus-appearance boss-ceres-ridley world-doors weapons-projectiles
  enemy-interactions audio-transport)
test "$(grep -c '^all rows: ' "$receipt")" -eq "${#domains[@]}"
for domain in "${domains[@]}"; do
  test "$(grep -c "^all rows: $domain free run: " "$receipt")" -eq 1
done
