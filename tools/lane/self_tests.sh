#!/usr/bin/env bash
# Run the lane and adjusted-vector self-tests in parallel. Without BASE every suite runs.
# With BASE each suite runs for changes to its own script or any of its inputs.
set -u
base=${1:-}
. tools/lane/changes.sh || exit 1
if [[ -n $base ]] && ! git merge-base "$base" HEAD >/dev/null; then
  echo "self_tests: cannot compare $base with HEAD" >&2
  exit 2
fi
clean=tools/cleanroom
lane=tools/lane
all=()
tests=()
known=()
suite() {
  local test=$1 path
  all+=("$test")
  if [[ -z $base ]] || changed_since "$base" "$@"; then tests+=("$test"); fi
  for path in "$@"; do known+=(":(exclude)$path"); done
}

suite "$lane/test_worktree.py" "$lane/worktree.sh"
suite "$lane/test_gate.py" "$lane/gate.sh"
suite "$clean/adjusted_vectors_test.sh" "$clean"/{adjusted_vectors.sh,adjusted_vectors_fixtures}
suite "$lane/test_self_tests.py"

# These operator entrypoints have no self-test in this suite. New or unclassified tooling
# files (including this selector and its change detector) run every suite. The landing tool,
# the adjusted-vector prover, the box's relay and the `cleanroom` binary's inputs and fixtures
# have their own Python and Rust tests in Quality.
for path in "$lane"/{played,run_tests}.sh "$lane"/{land,test_land,prereview,test_prereview,__init__}.py "$clean/__init__.py" \
    "$clean"/{adjusted_vectors_run.py,test_adjusted_vectors_run.py,adjusted_probe_tests.toml,adjusted_rule_owners} \
    "$clean"/{box_relay.py,test_box_relay.py,scan.sh,scan_tool.sh,scan_lexicon.sh,scan_lexicon,scan_allow.txt} \
    "$clean"/{allowed_deps.txt,scan_fixtures,rule_fixtures}; do
  known+=(":(exclude)$path")
done
if [[ -n $base ]] && changed_since "$base" "$lane" "$clean" "${known[@]}"; then tests=("${all[@]}"); fi
if [ ${#tests[@]} -eq 0 ]; then
  echo "SELF TESTS SKIPPED: no changes to self-test inputs since $base"
  exit 0
fi

out=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/self-tests-XXXXXX") || exit 1
trap 'rm -rf "$out"' EXIT
slots=$(nproc 2>/dev/null || echo 2)
for t in "${tests[@]}"; do
  while [ "$(jobs -rp | wc -l)" -ge "$slots" ]; do wait -n; done
  name=${t//\//_}
  case $t in *.py) run=("${SM_PY:-python3}" "$t") ;; *) run=(bash "$t") ;; esac
  { "${run[@]}" >"$out/$name.out" 2>&1; echo $? >"$out/$name.status"; } &
done
wait
bad=0
for t in "${tests[@]}"; do
  name=${t//\//_}
  [ "$(cat "$out/$name.status")" = 0 ] || { echo "SELF TEST FAIL $t"; cat "$out/$name.out"; bad=1; }
done
[ "$bad" -eq 0 ] && echo "SELF TESTS OK: ${tests[*]}"
exit $bad
