#!/usr/bin/env bash
# Local source gate: the formatting, lint, dependency, crate-rule and doc-link checks of Quality,
# and, each only when the branch (committed since its base, or in the working tree) changes its
# subject: the tools' self-tests; verify's lib tests, which hold the README, the report, the
# played runs and the vectors' pins to the tree; the planted faults' anchors and coverage
# (tools/check_mutants.py plan, which runs no fault); and the adjusted vectors' rule owners
# (tools/cleanroom/adjusted_vectors_run.py --owners, which runs no test). CI runs the other tests.
# The checks run at once, each into its own log, and report in a fixed order when all are done.
# They don't wait on each other's build locks: each clippy pass has its own target directory, and
# verify's lib tests link for an explicit target triple (tools/lane/test_dll.py).
# A gate that passed with nothing uncommitted records its key: its tree, its base's tree (the tree
# of the commit where the branch left BASE) and what it ran with: the toolchain, cargo-deny and
# shellcheck or their absence, Python, the environment variables cargo, rustc, the gate and the
# self-tests read, and the cargo and rustfmt configuration files outside the tree (below). Those
# decide what every check reads, which checks run and how, so a later gate of the same key with
# nothing uncommitted passed already: the root's gate of a squashed or picked commit whose tree is
# the lane's, on the base the lane was gated on, says whose pass it reuses and runs nothing. A file
# git is told to ignore changes to (skip-worktree, assume-unchanged), a file hidden by ignore rules
# from outside the tree, or a cargo configuration that includes another turns this off.
# SM_GATE_FRESH=1 runs the checks anyway.
# Overrides: SM_RUST_BIN (toolchain bin), SM_PY (Python), SM_GATE_DIR, SM_GATE_BASE (the base,
# default origin/main; a stacked branch names the branch it builds on), SM_GATE_PASSED (where the
# passes are recorded). Off Windows the toolchain is the cargo on PATH, which rustup points at
# rust-toolchain.toml's channel.
BASE=${SM_GATE_BASE:-origin/main}
# Every subject check compares with BASE, so one that doesn't resolve stops the gate.
git rev-parse --verify -q "$BASE^{commit}" >/dev/null || {
  echo "GATE: the base $BASE is not a commit here; set SM_GATE_BASE to the one the branch builds on"
  exit 2
}
CHANNEL=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/$CHANNEL-x86_64-pc-windows-gnu/bin}"
    export SM_PY="${SM_PY:-$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe}"
    ;;
  *) R="${SM_RUST_BIN:-$(dirname "$(command -v cargo)")}" ;;
esac
# The dependency rules (deny.toml). cargo-deny is a prebuilt binary, found on PATH or in cargo's
# bin directory; where it is missing the gate says so and CI's Quality holds the rules.
deny=$(command -v cargo-deny || echo "${CARGO_HOME:-$HOME/.cargo}/bin/cargo-deny")
[ -x "$deny" ] || [ -x "$deny.exe" ] || deny=
PASSED="${SM_GATE_PASSED:-${TEMP:-${TMPDIR:-/tmp}}/sm-gate-passed}"
key=
# The configuration files cargo and rustfmt read from outside the tree: in CARGO_HOME, rustfmt's
# home and user folders, and the checkout's folder, TEMP's, the folder the clean-room self-tests
# build in (lib.sh's outside_home_temp: /tmp when TEMP is in the home) and every folder above them.
configs=("${CARGO_HOME:-$HOME/.cargo}"/config{,.toml} "$HOME"/{rustfmt,.rustfmt}.toml
  "${XDG_CONFIG_HOME:-$HOME/.config}"/rustfmt/{rustfmt,.rustfmt}.toml)
[ -z "${USERPROFILE:-}" ] || configs+=("$USERPROFILE"/{rustfmt,.rustfmt}.toml)
[ -z "${APPDATA:-}" ] || configs+=("$APPDATA"/rustfmt/{rustfmt,.rustfmt}.toml)
build_temp=$(. tools/cleanroom/lib.sh && outside_home_temp) || build_temp=
for dir in "$(git rev-parse --show-toplevel)" "${TEMP:-${TMPDIR:-/tmp}}" ${build_temp:+"$build_temp"}; do
  up=
  while [ "$dir" != "$up" ]; do
    configs+=("$dir"/.cargo/config{,.toml} "$dir"/{rustfmt,.rustfmt}.toml)
    up=$dir dir=$(dirname "$dir")
  done
done
# A cargo configuration that includes another names a file this list doesn't hold: an `include`
# key, bare or quoted, or an `[[include]]` table.
included() {
  local config
  for config in "${configs[@]}"; do
    case $config in */config | */config.toml)
      [ ! -f "$config" ] ||
        ! grep -qE "^[[:space:]]*(\[\[)?[[:space:]]*[\"']?include[\"']?[[:space:]]*(=|\]\])" \
          "$config" || return 0 ;;
    esac
  done
  return 1
}
# Nothing uncommitted, nothing git is told to skip, and no file hidden only by ignore rules from
# outside the tree (core.excludesFile, info/exclude), which status doesn't show.
if [ -n "$build_temp" ] && [ -z "$(git status --porcelain --untracked-files=all)" ] &&
  ! git ls-files -v | grep -q '^[a-zS]' &&
  [ -z "$(git -C "$(git rev-parse --show-toplevel)" ls-files -o --exclude-per-directory=.gitignore)" ] &&
  ! included &&
  left=$(git merge-base "$BASE" HEAD); then
  ran_with=$({
    "$R/rustc" --version 2>&1
    if [ -n "$deny" ]; then echo "cargo-deny $deny"; cksum <"$deny" 2>/dev/null || cksum <"$deny.exe"
    else echo "no cargo-deny"; fi
    if shellcheck=$(command -v shellcheck); then echo "shellcheck $shellcheck"; cksum <"$shellcheck"
    else echo "no shellcheck"; fi
    "${SM_PY:-python3}" --version 2>&1
    env | LC_ALL=C grep -E '^(RUST|CARGO|CLEANROOM_|CANARY_|CLIPPY_CONF_DIR=|SM_RUST_BIN=|SM_PY=)' |
      LC_ALL=C sort
    # A link at a configuration's name counts even when it leads nowhere readable.
    for config in "${configs[@]}"; do
      [ ! -f "$config" ] && [ ! -L "$config" ] || { echo "$config"; { cksum <"$config"; } 2>&1; }
    done
  } | git hash-object --stdin)
  key=$(git rev-parse HEAD^{tree})-$(git rev-parse "$left^{tree}")-$ran_with
fi
if [ -n "$key" ] && [ "${SM_GATE_FRESH:-0}" != 1 ] && [ -s "$PASSED/$key" ]; then
  echo "GATE REUSED: $(cat "$PASSED/$key") passed this tree over the same base, with the same tools"
  [ -n "$deny" ] || echo "DENY SKIPPED: no cargo-deny here; CI's Quality runs cargo deny check bans"
  echo "GATE OK $(git rev-parse --short HEAD) $(git branch --show-current)"
  exit 0
fi
O="${SM_GATE_DIR:-${TEMP:-${TMPDIR:-/tmp}}/sm-gate}/$(git rev-parse --short HEAD)-$$"
mkdir -p "$O"; ok=1
O_resolved=$(cd "$O" && pwd -P)
checks=(fmt clippy clean-clippy deny crate-rules self-tests doc-links verify-lib faults owners)
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
# Prints the lines on stdin, which name a failing check $1's finding, or when there are none the
# log's last lines, so a failure always shows a reason.
reason() { head -20 | grep . || tail -n 5 "$O/$1.log"; }
# Starts check $1 in the background; its output goes to $O/$1.log and its status to $O/$1.status.
start() { local name=$1; shift; { "$@" >"$O/$name.log" 2>&1; echo $? >"$O/$name.status"; } & }
passed() { [ "$(cat "$O/$1.status" 2>/dev/null)" = 0 ]; }
# Whether the branch changes any of the paths: committed since it left BASE, or not yet
# committed (untracked files included).
changed() {
  ! git diff --quiet "$BASE...HEAD" -- "$@" ||
    [ -n "$(git status --porcelain --untracked-files=all -- "$@")" ]
}
# Records check $1 as passed without running it, its subject unchanged.
skip() { echo skipped >"$O/$1.log"; echo 0 >"$O/$1.status"; }
skipped() { [ "$(cat "$O/$1.log" 2>/dev/null)" = skipped ]; }

start fmt "$R/cargo" fmt --all --check
start clippy "$R/cargo" clippy --locked --workspace --all-targets --no-deps -- -D warnings
# The clean crates under the clean room's ceilings (G5): tools/cleanroom/clippy.toml, the export's.
# Clippy fingerprints CLIPPY_CONF_DIR, so this pass keeps its own target directory; sharing one
# would re-check `types` and everything that depends on it in both passes on every run.
CLIPPY_CONF_DIR="$(git rev-parse --show-toplevel)/tools/cleanroom" start clean-clippy "$R/cargo" clippy --locked   --target-dir "${CARGO_TARGET_DIR:-target}/clean-clippy" -p types -p samus_movement -p world_blocks -p world_camera -p interface_hud -p run_progress -p samus_appearance -p boss_ceres_ridley -p world_doors -p weapons_projectiles -p enemy_interactions -p audio_transport -p game_time -p game_random -p samus_health -p world_quake -p world_haze -p screens_boot -p screens_arrival -p screens_blackout -p screens_flight -p screens_intro -p screens_discovery -p screens_flashback -p screens_playback -p screens_title -p screens_file_select -p screens_options -p save_files --all-targets --no-deps -- -D warnings
if [ -n "$deny" ]; then
  PATH="$R:$PATH" start deny "$deny" --locked check bans
else
  deny=; echo 0 >"$O/deny.status"
fi
start crate-rules bash tools/check_crate_rules.sh
# The self-tests of the clean-room and lane tools, only for what this branch changes since main.
# They run beside the builds, not after them: most of their load is their own (the import test's
# cold build), so waiting for the builds only adds the builds' time; their timeouts allow for it.
CARGO="$R/cargo" start self-tests bash tools/lane/self_tests.sh "$BASE"
start doc-links sh tools/check_doc_links.sh
# What verify's lib tests read: the README and the report it measures (the crates, the specs and
# their vectors, the domain plan), the replay index and the played runs, and the build's inputs.
if changed crates scenarios docs/behaviours docs/architecture/domains.toml README.md Cargo.toml \
  Cargo.lock rust-toolchain.toml; then
  SM_RUST_BIN="$R" start verify-lib bash tools/lane/run_tests.sh --suite lib verify
else skip verify-lib; fi
# The planted faults' manifest and runner, the sources check_mutants.py lets a fault patch, and the
# replay cases whose responsibilities each fault witnesses.
if changed tools/mutants.toml tools/check_mutants.py crates/game/src/production \
  crates/game/src/production.rs crates/verify/src/replay/played.rs scenarios/replay/index.toml \
  scenarios/replay/baseline.toml; then
  start faults "${SM_PY:-python3}" -I tools/check_mutants.py plan
else skip faults; fi
# The rule owners, the specs' Parameter entries, the vectors' overrides and the clean crates that
# make a domain probed.
if changed tools/cleanroom/adjusted_rule_owners tools/cleanroom/adjusted_probe_tests.toml \
  tools/cleanroom/adjusted_vectors_run.py scenarios/vectors docs/behaviours crates/clean; then
  start owners "${SM_PY:-python3}" -I tools/cleanroom/adjusted_vectors_run.py --owners
else skip owners; fi
wait

if ! passed fmt; then
  fail "FMT FAIL"
  grep '^Diff in' "$O/fmt.log" | reason fmt
fi
if ! passed clippy; then
  fail "CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/clippy.log" | head -10 | reason clippy
fi
if ! passed clean-clippy; then
  fail "CLEAN CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/clean-clippy.log" | head -10 | reason clean-clippy
fi
[ -n "$deny" ] || echo "DENY SKIPPED: no cargo-deny here; CI's Quality runs cargo deny check bans"
if ! passed deny; then
  fail "DENY FAIL"
  grep -E -A6 '^error' "$O/deny.log" | reason deny
fi
if ! passed crate-rules; then
  fail "CRATE RULES FAIL"
  cat "$O/crate-rules.log"
fi
if ! passed self-tests; then
  fail "SELF TESTS FAIL"
  cat "$O/self-tests.log"
fi
if ! passed doc-links; then
  fail "DOC LINKS FAIL"
  reason doc-links <"$O/doc-links.log"
fi
if ! passed verify-lib; then
  fail "VERIFY LIB FAIL"
  # Each failing test, then where each panicked with the panic's first line.
  { grep -E '^test .* FAILED$|^test_dll:' "$O/verify-lib.log"
    grep -A1 'panicked at' "$O/verify-lib.log" | grep -v '^--$'; } | reason verify-lib
  ! grep -q 'committed_readme_is_current .* FAILED' "$O/verify-lib.log" ||
    echo "README STALE: run cargo run -p tools --bin tools -- report --write"
fi
if ! passed faults; then
  fail "FAULTS FAIL: tools/check_mutants.py plan"
  tail -n 1 "$O/faults.log"
fi
if ! passed owners; then
  fail "OWNERS FAIL: tools/cleanroom/adjusted_vectors_run.py --owners"
  grep '^adjusted_vectors_run: ' "$O/owners.log" | reason owners
fi
for check in verify-lib faults owners; do
  ! skipped "$check" || echo "${check^^} SKIPPED: no change to its subject since $BASE"
done
if [ "$ok" -eq 1 ]; then
  # Recorded only when nothing is uncommitted at the end and HEAD's tree is the one gated.
  if [ -n "$key" ] && [ -z "$(git status --porcelain --untracked-files=all)" ] &&
    [ "$(git rev-parse HEAD^{tree})" = "${key%%-*}" ]; then
    mkdir -p "$PASSED" &&
      echo "$(git rev-parse --short HEAD) $(git branch --show-current) $(date -u +%FT%TZ)" >"$PASSED/$key"
  fi
  echo "GATE OK $(git rev-parse --short HEAD) $(git branch --show-current)"
else
  echo "GATE FAILED $(git branch --show-current)"
  exit 1
fi
