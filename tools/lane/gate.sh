#!/usr/bin/env bash
# Local source gate: the formatting, lint, crate-rule, doc-link and report checks of Quality. Only
# the report check links (verify's lib test, tools/check_report.sh); CI runs the other tests.
# Overrides: SM_RUST_BIN (toolchain bin), SM_PY (Python), SM_GATE_DIR. Off Windows the toolchain is the cargo on
# PATH, which rustup points at rust-toolchain.toml's channel.
CHANNEL=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/$CHANNEL-x86_64-pc-windows-gnu/bin}"
    export SM_PY="${SM_PY:-$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe}"
    ;;
  *) R="${SM_RUST_BIN:-$(dirname "$(command -v cargo)")}" ;;
esac
O="${SM_GATE_DIR:-${TEMP:-${TMPDIR:-/tmp}}/sm-gate}/$(git rev-parse --short HEAD)-$$"
mkdir -p "$O"; ok=1
O_resolved=$(cd "$O" && pwd -P)
cleanup() {
  "${SM_PY:-python3}" - "$O" "$O_resolved" <<'PY'
import pathlib
import sys

directory = pathlib.Path(sys.argv[1])
if directory.is_symlink() or directory.resolve(strict=True) != pathlib.Path(sys.argv[2]).absolute():
    raise SystemExit("gate scratch path changed")
log = directory / "check.log"
if log.is_symlink():
    raise SystemExit("gate scratch log is a symlink")
log.unlink(missing_ok=True)
try:
    directory.rmdir()
except OSError:
    pass  # Preserve other files; the gate's cleanup is best effort.
PY
}
trap 'cleanup || true' EXIT
fail() { echo "$1"; ok=0; }


"$R/cargo" fmt --all --check >/dev/null 2>&1 || fail "FMT FAIL"

if ! "$R/cargo" clippy --locked --workspace --all-targets --no-deps -- -D warnings >"$O/check.log" 2>&1; then
  fail "CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/check.log" | head -10
fi
# The clean crates under the clean room's ceilings (G5): tools/cleanroom/clippy.toml, the export's.
# Clippy fingerprints CLIPPY_CONF_DIR, so this pass keeps its own target directory; sharing one
# would re-check `types` and everything that depends on it in both passes on every run.
if ! CLIPPY_CONF_DIR="$(git rev-parse --show-toplevel)/tools/cleanroom" "$R/cargo" clippy --locked   --target-dir "${CARGO_TARGET_DIR:-target}/clean-clippy" -p types -p samus_movement -p world_blocks -p world_camera -p interface_hud -p run_progress -p samus_appearance -p boss_ceres_ridley -p world_doors -p weapons_projectiles -p enemy_interactions -p audio_transport -p game_time -p game_random -p samus_health -p world_quake -p world_haze --all-targets --no-deps -- -D warnings >"$O/check.log" 2>&1; then
  fail "CLEAN CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/check.log" | head -10
fi
if ! CARGO="$R/cargo" bash tools/check_crate_rules.sh >"$O/check.log" 2>&1; then
  fail "CRATE RULES FAIL"
  cat "$O/check.log"
fi
sh tools/check_doc_links.sh >/dev/null 2>&1 || fail "DOC LINKS FAIL"
CARGO="$R/cargo" SM_RUST_BIN="$R" bash tools/check_report.sh >/dev/null 2>&1 || fail "REPORT OR README STATUS STALE: run cargo run -p tools --bin tools -- report --write"
if [ "$ok" -eq 1 ]; then
  echo "GATE OK $(git rev-parse --short HEAD) $(git branch --show-current)"
else
  echo "GATE FAILED $(git branch --show-current)"
  exit 1
fi
