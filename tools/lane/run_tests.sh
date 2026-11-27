#!/usr/bin/env bash
# Run crates' unit and integration tests: tools/lane/run_tests.sh <crate>... [-- <libtest args>]
# Optional --suite lib|test:NAME selects one enabled suite in one package.
# The reviewed played runs play in one call through tools/lane/played.sh.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
suite=()
if [ "${1:-}" = "--suite" ]; then
  [ "$#" -ge 3 ] || { echo "run_tests: --suite needs a selector and package" >&2; exit 2; }
  case "$2" in
    lib) suite=(--lib) ;;
    test:?*) suite=(--test "${2#test:}") ;;
    *) echo "run_tests: suite must be lib or test:NAME" >&2; exit 2 ;;
  esac
  shift 2
fi
args=()
for a in "$@"; do [ "$a" = "--" ] && break; args+=(-p "$a"); shift; done
if [ "${#suite[@]}" -gt 0 ] && [ "${#args[@]}" -ne 2 ]; then
  echo "run_tests: a selected suite requires one package" >&2; exit 2
fi
exec cargo test "${args[@]}" "${suite[@]}" "$@"
