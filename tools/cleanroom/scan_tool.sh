#!/usr/bin/env bash
# scan_tool.sh ARGS...: runs the `cleanroom` scanner (crates/tools/src/bin/cleanroom) from the
# repository root. Elsewhere it is `cargo run`; on Windows, where Defender quarantines freshly
# linked executables, the bin is linked as a DLL and its `main` called through
# tools/lane/test_dll.py, as run_tests.sh does. Overrides: SM_RUST_BIN, SM_PY, SM_SCAN_DLL_DIR.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    CHANNEL=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
    R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/$CHANNEL-x86_64-pc-windows-gnu/bin}"
    PY="${SM_PY:-$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe}"
    DIR="${SM_SCAN_DLL_DIR:-$TEMP/sm-cleanroom}"
    mkdir -p "$DIR"
    DLL="$DIR/cleanroom.dll"
    "$R/cargo.exe" rustc -q --release -p tools --bin cleanroom -- \
      -C link-arg=-shared -C link-arg=-o -C "link-arg=$(cygpath -m "$DLL" 2>/dev/null || echo "$DLL")" >&2
    exec "$PY" -I tools/lane/test_dll.py --run "$DLL" "$@"
    ;;
  *)
    exec cargo run -q --release -p tools --bin cleanroom -- "$@"
    ;;
esac
