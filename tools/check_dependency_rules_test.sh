#!/usr/bin/env bash
# Exercise the actual dependency guards with invented graphs, without the source/full gate.
set -euo pipefail
cd "$(dirname "$0")/.."
fake_cargo() {
  local crate='' depth='' previous='' arg
  for arg in "$@"; do
    [ "$arg" != n ] || return 9
    case $previous in -p) crate=$arg ;; --depth) depth=$arg ;; esac
    previous=$arg
  done
  [ "$1" = tree ] && [ "$previous" = '{p}' ] || return 10
  if [ "$scenario" = failure ] && [ -z "$depth" ]; then return 7; fi
  printf '%s v0.1.0 (/invented/%s)\n' "$crate" "$crate"
  [ -z "$depth" ] || return 0
  printf 'intermediate v0.1.0\n'
  case $scenario in
    import) printf 'import v0.1.0\n' ;;
    forbidden) printf 'oracle v0.1.0\nverify v0.1.0\ntools v0.1.0\nsm v0.1.0\nsm_example v0.1.0\n' ;;
    similar) printf 'import_helper v0.1.0\noracle_helper v0.1.0\nsmother v0.1.0\n' ;;
    clean|failure) ;;
  esac
}
. "${DEPENDENCY_RULES_SOURCE:-tools/check_dependency_rules.sh}"
for scenario in clean similar import forbidden failure; do
  cargo=fake_cargo bad=0 violations=0
  fail() { bad=1; violations=$((violations + 1)); }
  check_dependency_reachability
  case $scenario in
    clean|similar) expected=0 ;;
    import) expected=3 ;;
    forbidden) expected=20 ;;
    failure) expected=7 ;;
  esac
  if [ "$violations" -ne "$expected" ]; then
    echo "FAIL dependency scenario $scenario: $violations violations, expected $expected"
    exit 1
  fi
done
echo 'DEPENDENCY FIXTURES OK'
