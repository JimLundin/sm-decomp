#!/usr/bin/env bash
# self_tests.sh [BASE]: the clean-room and lane tools' self-tests (export, launch, canary, blind
# copy, import, squash, worktree, the gate, and the code check, score scan and adjusted vectors),
# at once, at most one per processor, each reported in a fixed order. With BASE, only when the branch
# changes their subject since BASE: tools/cleanroom/ or tools/lane/, and for the import's self-test
# also the toolchain, the workspace manifests and the shared types, whose clippy runs it holds,
# and the scanner whose report it reads.
# Without BASE (main pushes and scheduled runs) all run. Run from the repository root; exit 1
# prints each failing self-test's output.
set -u
base=${1:-}
# Whether the branch changes any of the paths since BASE (always, without BASE).
changed() { [ -z "$base" ] || ! git diff --quiet "$base...HEAD" -- "$@"; }

tests=()
if changed tools/cleanroom tools/lane; then
  tests=(tools/cleanroom/launch_test.sh tools/cleanroom/launch_config_test.sh tools/cleanroom/code_check_test.sh
    tools/cleanroom/canary_test.sh tools/cleanroom/score_scan_test.sh tools/lane/squash_test.sh
    tools/lane/worktree_test.sh tools/lane/gate_test.sh tools/cleanroom/blind_copy_test.sh
    tools/cleanroom/adjusted_vectors_test.sh)
fi
# The import self-test builds a throwaway worktree cold (about seven minutes).
if changed tools/cleanroom tools/lane rust-toolchain.toml Cargo.toml Cargo.lock crates/clean/types \
  crates/tools/src/bin/cleanroom; then
  tests+=(tools/cleanroom/import_check_test.sh)
fi
if [ ${#tests[@]} -eq 0 ]; then
  echo "SELF TESTS SKIPPED: no change to their subjects since $base"
  exit 0
fi

out=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/self-tests-XXXXXX")
trap 'rm -rf "$out"' EXIT
slots=$(nproc 2>/dev/null || echo 2)
for t in "${tests[@]}"; do
  while [ "$(jobs -rp | wc -l)" -ge "$slots" ]; do wait -n; done
  name=${t//\//_}
  { bash "$t" >"$out/$name.out" 2>&1; echo $? >"$out/$name.status"; } &
done
wait
bad=0
for t in "${tests[@]}"; do
  name=${t//\//_}
  [ "$(cat "$out/$name.status")" = 0 ] || { echo "SELF TEST FAIL $t"; cat "$out/$name.out"; bad=1; }
done
[ "$bad" -eq 0 ] && echo "SELF TESTS OK: ${#tests[@]} run"
exit $bad
