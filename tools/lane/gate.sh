#!/usr/bin/env bash
# Local source gate: the formatting, lint, dependency, crate-rule, doc-link and report checks of
# Quality, and the tools' self-tests when the branch changes their subject. Only the report check
# links (verify's lib test, tools/check_report.sh); CI runs the other tests.
# The checks run at once, each into its own log, and report in a fixed order when all are done.
# They don't wait on each other's build locks: each clippy pass has its own target directory, and
# the report check links its test for an explicit target triple (tools/lane/test_dll.py).
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
checks=(fmt clippy clean-clippy deny crate-rules self-tests doc-links report)
cleanup() {
  "${SM_PY:-python3}" - "$O" "$O_resolved" "${checks[@]}" <<'PY'
import pathlib
import sys

directory = pathlib.Path(sys.argv[1])
if directory.is_symlink() or directory.resolve(strict=True) != pathlib.Path(sys.argv[2]).absolute():
    raise SystemExit("gate scratch path changed")
for check in sys.argv[3:]:
    for suffix in (".log", ".status"):
        path = directory / (check + suffix)
        if path.is_symlink():
            raise SystemExit("gate scratch file is a symlink")
        path.unlink(missing_ok=True)
try:
    directory.rmdir()
except OSError:
    pass  # Preserve other files; the gate's cleanup is best effort.
PY
}
trap 'cleanup || true' EXIT
fail() { echo "$1"; ok=0; }
# Starts check $1 in the background; its output goes to $O/$1.log and its status to $O/$1.status.
start() { local name=$1; shift; { "$@" >"$O/$name.log" 2>&1; echo $? >"$O/$name.status"; } & }
passed() { [ "$(cat "$O/$1.status" 2>/dev/null)" = 0 ]; }

start fmt "$R/cargo" fmt --all --check
start clippy "$R/cargo" clippy --locked --workspace --all-targets --no-deps -- -D warnings
# The clean crates under the clean room's ceilings (G5): tools/cleanroom/clippy.toml, the export's.
# Clippy fingerprints CLIPPY_CONF_DIR, so this pass keeps its own target directory; sharing one
# would re-check `types` and everything that depends on it in both passes on every run.
CLIPPY_CONF_DIR="$(git rev-parse --show-toplevel)/tools/cleanroom" start clean-clippy "$R/cargo" clippy --locked   --target-dir "${CARGO_TARGET_DIR:-target}/clean-clippy" -p types -p samus_movement -p world_blocks -p world_camera -p interface_hud -p run_progress -p samus_appearance -p boss_ceres_ridley -p world_doors -p weapons_projectiles -p enemy_interactions -p audio_transport -p game_time -p game_random -p samus_health -p world_quake -p world_haze --all-targets --no-deps -- -D warnings
# The dependency rules (deny.toml). cargo-deny is a prebuilt binary, found on PATH or in cargo's
# bin directory; where it is missing the gate says so and CI's Quality holds the rules.
deny=$(command -v cargo-deny || echo "${CARGO_HOME:-$HOME/.cargo}/bin/cargo-deny")
if [ -x "$deny" ] || [ -x "$deny.exe" ]; then
  PATH="$R:$PATH" start deny "$deny" --locked check bans
else
  deny=; echo 0 >"$O/deny.status"
fi
start crate-rules bash tools/check_crate_rules.sh
# The self-tests of the clean-room and lane tools, only for what this branch changes since main.
CARGO="$R/cargo" start self-tests bash tools/lane/self_tests.sh origin/main
start doc-links sh tools/check_doc_links.sh
CARGO="$R/cargo" SM_RUST_BIN="$R" start report bash tools/check_report.sh
wait

passed fmt || fail "FMT FAIL"
if ! passed clippy; then
  fail "CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/clippy.log" | head -10
fi
if ! passed clean-clippy; then
  fail "CLEAN CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/clean-clippy.log" | head -10
fi
[ -n "$deny" ] || echo "DENY SKIPPED: no cargo-deny here; CI's Quality runs cargo deny check bans"
if ! passed deny; then
  fail "DENY FAIL"
  grep -E -A6 '^error' "$O/deny.log" | head -20
fi
if ! passed crate-rules; then
  fail "CRATE RULES FAIL"
  cat "$O/crate-rules.log"
fi
if ! passed self-tests; then
  fail "SELF TESTS FAIL"
  cat "$O/self-tests.log"
fi
passed doc-links || fail "DOC LINKS FAIL"
passed report || fail "REPORT OR README STATUS STALE: run cargo run -p tools --bin tools -- report --write"
if [ "$ok" -eq 1 ]; then
  echo "GATE OK $(git rev-parse --short HEAD) $(git branch --show-current)"
else
  echo "GATE FAILED $(git branch --show-current)"
  exit 1
fi
