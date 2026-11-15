#!/usr/bin/env bash
# Invented libtest transcripts exercise the CI invocation and receipt, without ROM or Cargo.
set -euo pipefail
cd "$(dirname "$0")/../.."
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
export calls="$scratch/calls"
fake_cargo() {
  [ "$#" -eq 10 ] || return 80
  [ "$1 $2 $3 $4 $5 $6 $7" = 'test --locked -p verify --profile verify --lib' ] || return 82
  [ "$9 ${10}" = '-- --exact' ] || return 83
  printf '%s\n' "$8" >> "$calls"
  if [ "$8" = harness::tests::the_hud_holds_its_floors_against_the_cache ]; then
    case "$scenario" in
      renamed) printf 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.00s\n'; return 0 ;;
      failed) printf 'test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s\n'; return 101 ;;
      ignored) printf 'test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 98 filtered out; finished in 0.00s\n'; return 0 ;;
      bad_exit) printf 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s\n'; return 101 ;;
      no_summary) return 0 ;;
    esac
  fi
  # The captured failure's splice, with invented report content. Only the summary is atomic
  # after the selected test completes; a test status line is not a reliable receipt.
  printf 'test %s ... okinvented-row: another harness report\n\n' "$8"
  printf 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s\n'
}
export -f fake_cargo
export CARGO=fake_cargo SM_CACHE=invented-cache SM_ROM=invented-rom
for scenario in splice renamed failed ignored bad_exit no_summary; do
  export scenario
  : > "$calls"
  if bash tools/ci/harness_floors.sh > "$scratch/output" 2>&1; then result=0; else result=$?; fi
  if [ "$scenario" = splice ]; then
    [ "$result" -eq 0 ]
    [ "$(wc -l < "$calls")" -eq 11 ]
    [ "$(sort -u "$calls" | wc -l)" -eq 11 ]
    for domain in the_clock_holds_its movement_holds_its the_camera_holds_its \
      progress_holds_its appearance_holds_its the_hud_holds_its the_blocks_hold_their \
      the_ceres_doors_hold_their the_door_passage_holds_its weapons_hold_their contact_holds_its; do
      grep -Fxq "harness::tests::${domain}_floors_against_the_cache" "$calls"
    done
    if grep -Fxq 'test harness::tests::the_hud_holds_its_floors_against_the_cache ... ok' "$scratch/output"; then exit 1; fi
  else
    [ "$result" -ne 0 ]
    [ "$(wc -l < "$calls")" -eq 6 ]
  fi
  printf 'floor receipt fixture: %s OK\n' "$scenario"
done
for missing in cache rom both; do
  : > "$calls"
  if (
    case "$missing" in
      cache) export SM_CACHE='' ;;
      rom) unset SM_ROM ;;
      both) unset SM_CACHE SM_ROM ;;
    esac
    bash tools/ci/harness_floors.sh
  ) > "$scratch/output" 2>&1; then exit 1; fi
  [ ! -s "$calls" ]
done
echo 'FLOOR RECEIPTS OK'
