#!/usr/bin/env bash
# Invented transcripts prove complete domain coverage and missing/duplicate rejection.
set -euo pipefail
cd "$(dirname "$0")/../.."
fake_cargo() {
  [ "$#" -eq 11 ] || return 80
  [ "$1 $2 $3 $4 $5 $6 $7" = 'test --locked -p verify --profile verify --lib' ] || return 81
  [ "$8" = harness::free::tests::every_domain_runs_free_against_the_cache ] || return 82
  [ "$9 ${10} ${11}" = '-- --exact --nocapture' ] || return 83
  for domain in 'game time' 'samus movement' world-camera interface-hud world-blocks \
    run-progress samus-appearance boss-ceres-ridley world-doors weapons-projectiles \
    enemy-interactions audio-transport; do
    case "$scenario:$domain" in
      missing:audio-transport|replaced:audio-transport) continue ;;
      substituted:audio-transport) domain=world-camera ;;
    esac
    printf 'all rows: %s free run: invented totals\n' "$domain"
  done
  case "$scenario" in
    duplicate|replaced) printf 'all rows: world-camera free run: invented totals\n' ;;
    unexpected) printf 'all rows: invented-domain free run: invented totals\n' ;;
  esac
  case "$scenario" in
    renamed) printf 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s\n' ;;
    ignored) printf 'test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 98 filtered out; finished in 0.01s\n' ;;
    no_summary) return 0 ;;
    *) printf 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s\n' ;;
  esac
  [ "$scenario" != bad_exit ]
}
export -f fake_cargo
export CARGO=fake_cargo SM_CACHE=invented-cache SM_ROM=invented-rom
for scenario in complete missing duplicate replaced substituted unexpected renamed ignored no_summary bad_exit; do
  export scenario
  if bash tools/ci/domain_free.sh > /dev/null 2>&1; then result=0; else result=$?; fi
  if [ "$scenario" = complete ]; then [ "$result" -eq 0 ]; else [ "$result" -ne 0 ]; fi
  printf 'free receipt fixture: %s OK\n' "$scenario"
done
for missing in SM_CACHE SM_ROM; do
  if (unset "$missing"; bash tools/ci/domain_free.sh) > /dev/null 2>&1; then exit 1; fi
done
echo 'FREE RECEIPTS OK'
