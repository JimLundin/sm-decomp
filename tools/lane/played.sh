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
SM_PLAYED="$*" exec bash tools/lane/run_tests.sh --suite lib verify -- --ignored --exact \
  replay::played::tests::reviewed_runs_hold --nocapture
