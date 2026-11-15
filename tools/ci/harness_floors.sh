#!/usr/bin/env bash
# Exact selection and the execution count prove every required floor test ran. Harness writes
# bypass libtest capture, so concurrent tests can splice their reports into test status lines.
set -euo pipefail
: "${SM_CACHE:?SM_CACHE is required}" "${SM_ROM:?SM_ROM is required}"
receipt=$(mktemp)
trap 'rm -f -- "$receipt"' EXIT
for test in the_clock_holds_its movement_holds_its the_camera_holds_its \
  progress_holds_its appearance_holds_its the_hud_holds_its the_blocks_hold_their \
  the_ceres_doors_hold_their the_door_passage_holds_its weapons_hold_their contact_holds_its; do
  name="harness::tests::${test}_floors_against_the_cache"
  "${CARGO:-cargo}" test --locked -p verify --profile verify --lib "$name" -- --exact \
    | tee "$receipt"
  grep -Eq '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out; finished in ' "$receipt"
  printf 'floor test passed: %s\n' "$name"
done
