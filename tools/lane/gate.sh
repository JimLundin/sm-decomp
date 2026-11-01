#!/usr/bin/env bash
# The local gate for a lane commit, run from the worktree root (bash tools/lane/gate.sh).
# Checks fmt, mutation anchors, clippy, the target crates, the detached host, both sm
# configurations, the verifier DLL, the source inventory and the ROM import test.
# Prints "GATE OK <hash> <branch>" or "GATE FAILED <branch>"; nothing in it needs a test exe to run.
# Overrides: SM_RUST_BIN (toolchain bin), SM_PY (Python 3.11+), SM_ROM, SM_GATE_DIR.
R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-gnu/bin}"
PY="${SM_PY:-/c/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe}"
ROM="${SM_ROM:-$(git rev-parse --path-format=absolute --git-common-dir)/../rom/sm.smc}"
O="${SM_GATE_DIR:-$TEMP/sm-gate}/$(git rev-parse --short HEAD)-$$"; mkdir -p "$O/v"; ok=1
rustc() { "$R/rustc" --edition=2024 -D warnings "$@"; }
fail() { echo "$1"; [ -n "$2" ] && grep -E "^(error|warning)|-->" "$2" | head -10; ok=0; }

"$R/cargo" fmt --all --check >/dev/null 2>&1 || fail "FMT FAIL"
for f in sm/src/verification/player_detached/mutations.py sm/src/verification/session_mutations.py; do
  "$PY" -c "import sys,runpy,os; sys.path.insert(0, os.path.dirname('$f')); sys.argv=['$f','--check-anchors']; runpy.run_path('$f', run_name='__main__')" >/dev/null 2>&1 || fail "ANCHORS FAIL $f"
done
"$PY" -I tools/lane/anchors_gate.py . >/dev/null || fail "MUTATION ANCHORS FAIL"
tools_pkg=""; [ -f crates/sm_tools/Cargo.toml ] && tools_pkg="-p sm_tools"
"$R/cargo" clippy --locked -p sm_data -p sm_game -p sm_runtime $tools_pkg --all-targets --no-deps -q >/dev/null 2>"$O/clippy.log" || fail "CLIPPY FAIL" "$O/clippy.log"

# Target crates, plain and with the verifier cfg.
rustc --crate-name sm_data --crate-type rlib crates/sm_data/src/lib.rs -o "$O/libsm_data.rlib" || ok=0
ext="--extern sm_data=$O/libsm_data.rlib"
rustc --crate-name sm_game --crate-type rlib crates/sm_game/src/lib.rs $ext -L "dependency=$O" -o "$O/libsm_game.rlib" || ok=0
rustc --crate-name sm_runtime --crate-type rlib crates/sm_runtime/src/lib.rs $ext -L "dependency=$O" -o "$O/libsm_runtime.rlib" || ok=0
gx="--extern sm_game=$O/libsm_game.rlib -L dependency=$O --extern sm_runtime=$O/libsm_runtime.rlib"
rustc --cfg sm_verify_dll --crate-name sm_game --crate-type rlib crates/sm_game/src/lib.rs $ext -L "dependency=$O" -o "$O/v/libsm_game.rlib" || ok=0
gxv="-L dependency=$O/v ${gx/libsm_game.rlib/v/libsm_game.rlib}"
gt=""; gtv=""
if [ -f crates/sm_tools/src/lib.rs ]; then
  rustc --crate-name sm_tools --crate-type rlib crates/sm_tools/src/lib.rs $ext --extern sm_game=$O/libsm_game.rlib -L "dependency=$O" -o "$O/libsm_tools.rlib" || ok=0
  rustc --crate-name sm_tools --crate-type rlib crates/sm_tools/src/lib.rs $ext --extern sm_game=$O/v/libsm_game.rlib -L "dependency=$O/v" -L "dependency=$O" -o "$O/v/libsm_tools.rlib" || ok=0
  gt="--extern sm_tools=$O/libsm_tools.rlib"; gtv="--extern sm_tools=$O/v/libsm_tools.rlib"
fi

# The detached host; --test has two known unused imports in main.rs.
rustc sm/src/verification/player_detached/main.rs $gx $ext --emit=metadata -o "$O/detached.rmeta" >"$O/det.log" 2>&1 || fail "DETACHED FAIL" "$O/det.log"
if ! rustc --test sm/src/verification/player_detached/main.rs $gx $ext --emit=metadata -o "$O/detached.rmeta" >"$O/dett.log" 2>&1; then
  all=$(grep -c "^error" "$O/dett.log"); unused=$(grep -c "^error: unused import" "$O/dett.log")
  { [ "$all" -le 3 ] && [ "$unused" -le 2 ]; } || fail "DETACHED --test FAIL" "$O/dett.log"
fi

# sm in both configurations, then the verifier cdylib against the verify-cfg library.
rustc --crate-name sm --crate-type rlib sm/src/lib.rs $gx $gt $ext --emit=metadata -o "$O/libsm.rmeta" >"$O/sm.log" 2>&1 || fail "SM FAIL" "$O/sm.log"
rustc --crate-name sm --crate-type rlib --cfg sm_verify_dll sm/src/lib.rs $gxv $gtv $ext --emit=metadata -o "$O/libsmv.rmeta" >"$O/smv.log" 2>&1 || fail "SM [verify] FAIL" "$O/smv.log"
# Check the ordinary host against the library exposing checkpoint support, as
# Cargo's frontend library does; the host itself still has no sm_verify_dll cfg.
rustc --crate-name sm_verify --crate-type cdylib sm/src/verify_dll.rs --extern sm="$O/libsmv.rmeta" $gxv $gtv $ext --emit=metadata -o "$O/plain-verify.rmeta" >"$O/plain-v.log" 2>&1 || fail "VERIFY_DLL [plain] FAIL" "$O/plain-v.log"
for t in "" "--test"; do
  SM_VERIFY_SOURCE_HASH=$(printf '0%.0s' $(seq 64)) rustc $t --crate-name sm_verify --crate-type cdylib --cfg sm_verify_dll sm/src/verify_dll.rs --extern sm="$O/libsmv.rmeta" $gxv $gtv $ext --emit=metadata -o "$O/verify.rmeta" >"$O/v.log" 2>&1 || fail "VERIFY_DLL [$t] FAIL" "$O/v.log"
done
"$PY" -I -c "import sys; sys.path.insert(0,'tools'); import pathlib, verify_dll; verify_dll.check_source_inventory(pathlib.Path('.'))" || ok=0

# The ROM import test; test executables may be blocked locally (exit 126), which is not a failure.
if [ -f crates/sm_data/tests/rom_import.rs ] && [ -f "$ROM" ]; then
  X="$O/rom_import.exe"
  if rustc -O --test crates/sm_data/tests/rom_import.rs --extern sm_data=$O/libsm_data.rlib -o "$X" >"$O/ri.log" 2>&1; then
    SM_ROM="$ROM" "$X" >>"$O/ri.log" 2>&1; st=$?
    if [ $st -eq 126 ] || grep -q "Access is denied" "$O/ri.log"; then echo "(rom_import built; execution blocked locally)"
    elif [ $st -ne 0 ]; then fail "ROM IMPORT TEST FAIL" "$O/ri.log"; fi
  else fail "ROM IMPORT BUILD FAIL" "$O/ri.log"; fi
fi
rm -rf "$O"
[ $ok -eq 1 ] && echo "GATE OK $(git rev-parse --short HEAD) $(git branch --show-current)" || echo "GATE FAILED $(git branch --show-current)"
