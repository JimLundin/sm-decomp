#!/usr/bin/env bash
# Invented receipts prove a missing arm, skipped test or wrong publication count fails CI.
set -euo pipefail
cd "$(dirname "$0")/../.."
scratch=$(mktemp -d)
scratch_resolved=$(cd "$scratch" && pwd -P)
cleanup() {
  "${SM_PY:-python3}" - "$scratch" "$scratch_resolved" <<'PY'
import pathlib
import sys

scratch = pathlib.Path(sys.argv[1])
if scratch.is_symlink() or scratch.resolve(strict=True) != pathlib.Path(sys.argv[2]).absolute():
    raise SystemExit("audio receipt scratch path changed")
output = scratch / "output"
if output.is_symlink():
    raise SystemExit("audio receipt scratch output is a symlink")
output.unlink(missing_ok=True)
scratch.rmdir()
PY
}
trap cleanup EXIT
fake_cargo() {
  [ "$#" -eq 12 ] || return 80
  [ "$1 $2 $3 $4 $5 $6 $7" = 'test --locked -p verify --profile verify --lib' ] || return 81
  [ "$8" = harness::audio::native::tests::native_paired_arms_are_exact ] || return 82
  [ "$9 ${10} ${11} ${12}" = '-- --ignored --exact --nocapture' ] || return 83
  for arm in control expire; do
    [ "$scenario" != missing_arm ] || [ "$arm" != expire ] || continue
    publications=764
    [ "$scenario" != wrong_count ] || publications=763
    printf 'NATIVE audio-transport %s compared184 publications%s replies736 fields7157 null_cuts0 null_fields3000 restorations184 skipped0 defects0 cpu_timing0/140\n' "$arm" "$publications"
    boundaries=573
    restorations=573
    [ "$scenario" != wrong_pending_boundaries ] || boundaries=572
    [ "$scenario" != wrong_pending_restorations ] || restorations=572
    if [ "$scenario" != missing_pending_arm ] || [ "$arm" != expire ]; then
      printf 'PENDING audio-transport %s boundaries%s restorations%s zero_elapsed_exact committed_retained pending_retained publication_once\n' "$arm" "$boundaries" "$restorations"
    fi
  done
  case "$scenario" in
    renamed) printf 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.00s\n' ;;
    ignored) printf 'test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 98 filtered out; finished in 0.00s\n' ;;
    *) printf 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s\n' ;;
  esac
  [ "$scenario" != bad_exit ]
}
export -f fake_cargo
export CARGO=fake_cargo SM_AUDIO_NATIVE=invented-fixtures
for scenario in complete missing_arm wrong_count missing_pending_arm wrong_pending_boundaries wrong_pending_restorations renamed ignored bad_exit; do
  export scenario
  if bash tools/ci/audio_native.sh > "$scratch/output" 2>&1; then result=0; else result=$?; fi
  if [ "$scenario" = complete ]; then [ "$result" -eq 0 ]; else [ "$result" -ne 0 ]; fi
  printf 'native receipt fixture: %s OK\n' "$scenario"
done
if (unset SM_AUDIO_NATIVE; bash tools/ci/audio_native.sh) > "$scratch/output" 2>&1; then exit 1; fi
echo 'NATIVE AUDIO RECEIPT FIXTURES OK'
