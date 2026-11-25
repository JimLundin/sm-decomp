#!/usr/bin/env bash
# Play the reviewed runs of scenarios/replay/played.toml in one test process, one build and
# one load of verify's lib tests: tools/lane/played.sh [<row> | <row>/<start>-<end>]...
# With no names it plays every run. Each run prints its held cuts and stop beside the file's,
# and the exit is nonzero when any differs, naming it. Needs SM_CACHE and SM_ROM; SM_LOCK
# defaults to the lsnes lock, and the build is optimised as replay tests are (run_tests.sh).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
for name in SM_CACHE SM_ROM; do
  [ -n "${!name:-}" ] || { echo "played: set $name" >&2; exit 2; }
done
export SM_LOCK="${SM_LOCK:-$PWD/scenarios/locks/lsnes.toml}"
export CARGO_PROFILE_TEST_OPT_LEVEL="${CARGO_PROFILE_TEST_OPT_LEVEL:-3}"
# The exact name selects nothing once the test is renamed, which libtest passes: as
# tools/ci/required_witnesses.py does, the run counts only when the test itself reports ok and is
# the one test run.
test=replay::played::tests::reviewed_runs_hold
log=$(mktemp)
trap 'rm -f "$log"' EXIT
status=0
SM_PLAYED="$*" bash tools/lane/run_tests.sh --suite lib verify -- --ignored --exact "$test" \
  --nocapture 2>&1 | tee "$log" || status=$?
[ "$status" -eq 0 ] || exit "$status"
grep -q "^test $test \.\.\. ok$" "$log" && grep -q '^test result: ok\. 1 passed; 0 failed; 0 ignored' "$log" || {
  echo "played: $test did not run as the one test" >&2
  exit 4
}
