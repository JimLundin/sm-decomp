#!/usr/bin/env bash
# Run crates' unit and integration tests: tools/lane/run_tests.sh <crate>... [-- <libtest args>]
# On Windows each test target links as a DLL and runs through Python (Defender blocks
# test executables); elsewhere this is `cargo test`. Doc tests are not run on Windows.
# Overrides: SM_RUST_BIN (toolchain bin), SM_PY (Python 3.11+), SM_TEST_DLL_DIR.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) ;;
  *)
    args=()
    for a in "$@"; do [ "$a" = "--" ] && break; args+=(-p "$a"); shift; done
    exec cargo test "${args[@]}" "$@"
    ;;
esac
CHANNEL=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/$CHANNEL-x86_64-pc-windows-gnu/bin}"
PY="${SM_PY:-$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe}"
OUT="${SM_TEST_DLL_DIR:-$TEMP/sm-test-dll/$(basename "$PWD")}"
exec "$PY" -I tools/lane/test_dll.py "$R/cargo.exe" "$OUT" "$@"
