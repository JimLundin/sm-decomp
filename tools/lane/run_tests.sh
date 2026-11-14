#!/usr/bin/env bash
# Run crates' unit and integration tests: tools/lane/run_tests.sh <crate>... [-- <libtest args>]
# On Windows each test target links as a DLL and runs through Python (Defender blocks
# test executables); elsewhere this is `cargo test`. Doc tests are not run on Windows.
# Optional --suite lib|test:NAME selects one enabled suite in one package.
# Overrides: SM_RUST_BIN (toolchain bin), SM_PY (Python 3.11+), SM_TEST_DLL_DIR.
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
  selector=$2
  shift 2
fi
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) ;;
  *)
    args=()
    for a in "$@"; do [ "$a" = "--" ] && break; args+=(-p "$a"); shift; done
    if [ "${#suite[@]}" -gt 0 ] && [ "${#args[@]}" -ne 2 ]; then
      echo "run_tests: a selected suite requires one package" >&2; exit 2
    fi
    exec cargo test "${args[@]}" "${suite[@]}" "$@"
    ;;
esac
CHANNEL=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/$CHANNEL-x86_64-pc-windows-gnu/bin}"
PY="${SM_PY:-$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe}"
OUT="${SM_TEST_DLL_DIR:-$TEMP/sm-test-dll/$(basename "$PWD")}"
selected=()
if [ "${#suite[@]}" -gt 0 ]; then selected=(--suite "$selector"); fi
exec "$PY" -I tools/lane/test_dll.py "$R/cargo.exe" "$OUT" "${selected[@]}" "$@"
