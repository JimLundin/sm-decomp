#!/usr/bin/env bash
# Local source gate. CI builds/links the verifier executable and runs units/replays.
# Overrides: SM_RUST_BIN (toolchain bin), SM_PY (Python 3.11+), SM_GATE_DIR.
R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-gnu/bin}"
PY="${SM_PY:-/c/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe}"
O="${SM_GATE_DIR:-$TEMP/sm-gate}/$(git rev-parse --short HEAD)-$$"
mkdir -p "$O"; ok=1
trap 'rm -f -- "$O/check.log"; rmdir -- "$O" 2>/dev/null || true' EXIT
fail() { echo "$1"; ok=0; }

"$R/cargo" fmt --all --check >/dev/null 2>&1 || fail "FMT FAIL"
for f in sm/src/verification/player_detached/mutations.py sm/src/verification/session_mutations.py; do
  "$PY" -c "import sys,runpy,os; sys.path.insert(0, os.path.dirname('$f')); sys.argv=['$f','--check-anchors']; runpy.run_path('$f', run_name='__main__')" >/dev/null 2>&1 || fail "ANCHORS FAIL $f"
done
"$PY" -I -c "import sys; sys.path.insert(0,'tools'); import pathlib, verify_dll; verify_dll.check_source_inventory(pathlib.Path('.'))" || fail "SOURCE INVENTORY FAIL"

# Preserve existing compiler options, including Cargo's higher-precedence flags.
if [[ -v CARGO_ENCODED_RUSTFLAGS ]]; then
  export CARGO_ENCODED_RUSTFLAGS="$CARGO_ENCODED_RUSTFLAGS"$'\x1f-D\x1fwarnings'
else
  export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-D warnings"
fi
if ! "$R/cargo" check --locked --no-default-features -p sm -p sm_data -p sm_game -p sm_runtime -p sm_tools --lib >"$O/check.log" 2>&1; then
  fail "CARGO CHECK FAIL"
  rg '^(error|warning)|-->' "$O/check.log" | head -10
fi
if [ "$ok" -eq 1 ]; then
  echo "GATE OK $(git rev-parse --short HEAD) $(git branch --show-current)"
else
  echo "GATE FAILED $(git branch --show-current)"
  exit 1
fi
