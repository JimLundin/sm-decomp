#!/usr/bin/env bash
# Run the clean-room and lane self-tests in parallel. Without BASE every suite runs.
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

# Shared inputs of the Rust scanner, the source checks and the confined test runner.
cargo=(Cargo.toml Cargo.lock rust-toolchain.toml .cargo/config .cargo/config.toml 'crates/*/Cargo.toml')
scanner=(crates/tools/src/bin/cleanroom "$clean"/{scan.sh,scan_tool.sh,scan_lexicon,scan_allow.txt} "${cargo[@]}")
brief=("$clean/scan_brief.sh" "${scanner[@]}")
source_checks=("$clean"/{code_check.sh,allowed_deps.txt,check_manifest.py}
  "$clean"/{source_scan,lint_scan,shape_scan,strip_rust}.awk Cargo.toml)
runner=("$clean"/{vector_runner.py,shape_scan.awk,strip_rust.awk})
launch=("$clean"/{launch.sh,launch_stream.py,box_relay.py,lib.sh,allowlist.txt,canary.sh}
  "$clean"/{launch_probe.txt,launch_probe_checks.toml,probe_check.py} "${runner[@]}")

suite "$clean/launch_test.sh" "${launch[@]}" "${brief[@]}" "${source_checks[@]}" \
  "$clean"/{launch_responses.sh,responses_controller.py,responses_box.py,responses_probe.txt} \
  "$clean"/{launch_stream,responses_controller,box_relay,probe_check}_test.py
suite "$clean/launch_config_test.sh" "${launch[@]}" "${brief[@]}" "$clean/export.sh" "$lane/worktree.sh"
suite "$clean/code_check_test.sh" "${source_checks[@]}" "${runner[@]}" "$clean"/{code_check_fixtures,vector_runner_test.py}
suite "$clean/canary_test.sh" "$clean"/{canary.sh,canary_plant_test.sh,lib.sh,allowlist.txt}
suite "$lane/test_worktree.py" "$lane/worktree.sh"
suite "$lane/test_gate.py" "$lane/gate.sh" "$clean/lib.sh"
suite "$clean/blind_copy_test.sh" "$clean"/{blind_copy.sh,lib.sh,allowlist.txt} .agents/skills/blind-reviewer/SKILL.md
suite "$clean/adjusted_vectors_test.sh" "$clean"/{adjusted_vectors.sh,adjusted_vectors_fixtures}
# The fixture changes shared types and requires the importer to name world_blocks when it breaks.
suite "$clean/import_check_test.sh" "${scanner[@]}" "${source_checks[@]}" \
  "$clean"/{import.sh,export.sh,export_workspace.py,lib.sh,allowlist.txt} crates/clean/.clippy.toml clippy.toml \
  crates/clean/{types,world_blocks,import_test} docs/cleanroom/DESIGN.md .agents/skills/implementer/SKILL.md \
  docs/behaviours/import-test.md docs/behaviours/import-test.decode.md scenarios/vectors/import-test.toml
suite "$lane/test_self_tests.py"

# These operator entrypoints have no self-test in this suite. New or unclassified tooling
# files (including this selector and its change detector) run every suite. The landing tool,
# the adjusted-vector prover and scanner fixtures have their own Python and Rust tests in Quality.
for path in "$lane"/{played,run_tests}.sh "$lane"/{land,test_land}.py \
    "$clean"/{adjusted_vectors_run.py,test_adjusted_vectors_run.py,adjusted_probe_tests.toml,adjusted_rule_owners,scan_fixtures}; do
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
