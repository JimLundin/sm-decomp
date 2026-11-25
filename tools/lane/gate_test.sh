#!/usr/bin/env bash
# gate_test.sh: gate.sh, run in a scratch repository whose tools and toolchain are fakes that
# record that they ran, stops before any check on a base that doesn't resolve; passes a clean
# branch; reuses that pass for a new commit of the same tree over the same base, running nothing,
# unless SM_GATE_FRESH=1, other compiler flags, cargo-deny's presence or a skip-worktree file
# says otherwise; never reuses a failed gate; runs a check for an untracked file in its subject
# even where git status hides untracked files; and shows a failing check's last lines when its tool prints none of the lines the gate
# looks for. Run from anywhere; exit 1 lists what went wrong.
here=$(cd "$(dirname "$0")" && pwd)
bad=0
fail() { echo "FAIL $*"; bad=1; }
scratch=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/gate-test-XXXXXX")
trap 'rm -rf "$scratch"' EXIT
repo=$scratch/repo bin=$scratch/bin ran=$scratch/ran
mkdir -p "$repo/tools/lane" "$repo/tools/cleanroom" "$repo/scenarios" "$bin" "$ran"
cp "$here/gate.sh" "$repo/tools/lane/gate.sh"
echo 'channel = "0.0.0"' >"$repo/rust-toolchain.toml"
echo 'Invented vectors.' >"$repo/scenarios/demo.toml"
# Each fake records its name, prints $GATE_TEST_OUT_<name> and exits $GATE_TEST_STATUS_<name>.
# The gate runs some tools with `sh`, which is dash on Linux, so the fakes are POSIX sh.
fake() { # NAME PATH
  printf '#!/bin/sh\necho >"%s/%s"\n[ -z "${GATE_TEST_OUT_%s-}" ] || echo "$GATE_TEST_OUT_%s"\nexit "${GATE_TEST_STATUS_%s:-0}"\n' \
    "$ran" "$1" "$1" "$1" "$1" >"$2"
  chmod +x "$2"
}
# Where dash is installed, `sh` is dash here as on Linux, so a bash-only fake fails on both.
if dash=$(command -v dash); then
  printf '#!/bin/sh\nexec "%s" "$@"\n' "$dash" >"$bin/sh"
  chmod +x "$bin/sh"
fi
fake cargo "$bin/cargo"
fake deny "$bin/cargo-deny"
fake self_tests "$repo/tools/lane/self_tests.sh"
fake verify_lib "$repo/tools/lane/run_tests.sh"
fake crate_rules "$repo/tools/check_crate_rules.sh"
fake doc_links "$repo/tools/check_doc_links.sh"
for name in faults owners; do
  path=$repo/tools/check_mutants.py
  [ "$name" = owners ] && path=$repo/tools/cleanroom/adjusted_vectors_run.py
  printf 'import os, pathlib, sys\npathlib.Path(%s, "%s").write_text("")\nprint(os.environ.get("GATE_TEST_OUT_%s", ""))\nsys.exit(int(os.environ.get("GATE_TEST_STATUS_%s", "0")))\n' \
    "r'$(cygpath -w "$ran" 2>/dev/null || echo "$ran")'" "$name" "$name" "$name" >"$path"
done
git -C "$repo" init -q -b lane/test
git -C "$repo" -c core.autocrlf=false add -A
git -C "$repo" -c user.name=fixture -c user.email=fixture@invalid commit -qm "Invented tree"

# Runs the gate in the scratch repository with the environment given as arguments.
gate() {
  rm -f "$ran"/*
  (cd "$repo" && env PATH="$bin:$PATH" SM_RUST_BIN="$bin" SM_GATE_DIR="$scratch/gate" \
    SM_GATE_PASSED="$scratch/passed" CARGO_HOME="$scratch/no-cargo-home" SM_GATE_BASE=HEAD "$@" \
    bash tools/lane/gate.sh 2>&1)
}
ran() { [ -e "$ran/$1" ]; }

out=$(gate SM_GATE_BASE=no-such-base); status=$?
[ "$status" -eq 2 ] || fail "a base that doesn't resolve exited $status: $out"
[ -z "$(ls "$ran")" ] || fail "a base that doesn't resolve ran $(ls "$ran"): $out"

out=$(gate) || fail "a clean branch failed: $out"
grep -q '^GATE OK ' <<<"$out" || fail "a clean branch printed no GATE OK: $out"
! ran verify_lib || fail "a clean branch ran verify's lib tests: $out"

commit() { git -C "$repo" -c user.name=fixture -c user.email=fixture@invalid commit -q "$@"; }
# A new commit of the same tree over the same base, as a squash or a pick makes.
commit --allow-empty -m "Same tree"
out=$(gate) || fail "the same tree's second gate failed: $out"
grep -q '^GATE REUSED: ' <<<"$out" || fail "the same tree's second gate reused nothing: $out"
grep -q "^GATE OK $(git -C "$repo" rev-parse --short HEAD) " <<<"$out" || fail "the reused gate named another head: $out"
[ -z "$(ls "$ran")" ] || fail "the reused gate ran $(ls "$ran"): $out"
out=$(gate SM_GATE_FRESH=1) || fail "a fresh gate of a passed tree failed: $out"
ran cargo || fail "SM_GATE_FRESH=1 ran no check: $out"
# What the gate ran with is part of the key: other compiler flags, or cargo-deny installed or gone.
out=$(gate RUSTFLAGS=--cap-lints=allow) || fail "a gate with other flags failed: $out"
! grep -q '^GATE REUSED: ' <<<"$out" || fail "a gate with other flags reused a pass: $out"
# No cargo-deny: the fake moved away, and no directory on PATH holding a real one (CI's Quality
# installs one); CARGO_HOME, the gate's other place to look, is empty for every gate here.
mv "$bin/cargo-deny" "$bin/no-cargo-deny"
no_deny=$bin
while IFS= read -r dir; do
  [ -z "$dir" ] || [ -e "$dir/cargo-deny" ] || [ -e "$dir/cargo-deny.exe" ] || no_deny=$no_deny:$dir
done < <(tr : '\n' <<<"$PATH")
out=$(gate PATH="$no_deny") || fail "a gate without cargo-deny failed: $out"
! grep -q '^GATE REUSED: ' <<<"$out" || fail "a gate without cargo-deny reused a pass that ran it: $out"
out=$(gate PATH="$no_deny") || fail "the second gate without cargo-deny failed: $out"
grep -q '^DENY SKIPPED: ' <<<"$out" || fail "a reused gate without cargo-deny didn't say deny was skipped: $out"
mv "$bin/no-cargo-deny" "$bin/cargo-deny"
# A file whose changes git is told to ignore turns reuse off.
git -C "$repo" update-index --skip-worktree scenarios/demo.toml
out=$(gate) || fail "a gate with a skip-worktree file failed: $out"
! grep -q '^GATE REUSED: ' <<<"$out" || fail "a gate with a skip-worktree file reused a pass: $out"
git -C "$repo" update-index --no-skip-worktree scenarios/demo.toml

# A failed gate is not reused: the same tree's next gate runs the checks again.
echo 'Invented crate rules.' >"$repo/scenarios/rules.toml"
git -C "$repo" add -A && commit -m "Changed tree"
out=$(gate GATE_TEST_STATUS_crate_rules=1) && fail "a failing crate-rules check passed: $out"
out=$(gate) || fail "the failed tree's next gate failed: $out"
! grep -q '^GATE REUSED: ' <<<"$out" || fail "a failed gate was reused: $out"
ran crate_rules || fail "the failed tree's next gate ran no crate rules: $out"

mkdir "$repo/scenarios/new"
echo 'Invented untracked vectors.' >"$repo/scenarios/new/demo.toml"
out=$(gate GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=status.showUntrackedFiles GIT_CONFIG_VALUE_0=no) ||
  fail "an untracked subject file failed the fakes: $out"
ran verify_lib || fail "an untracked subject file hidden by status.showUntrackedFiles=no skipped verify's lib tests: $out"

# Tools that fail with nothing the gate looks for: a link error and a TOML error's traceback.
mkdir -p "$repo/tools/cleanroom/adjusted_rule_owners"
echo 'Invented owners.' >"$repo/tools/cleanroom/adjusted_rule_owners/demo.toml"
out=$(gate GATE_TEST_STATUS_verify_lib=1 GATE_TEST_OUT_verify_lib='invented link failure' \
  GATE_TEST_STATUS_owners=1 GATE_TEST_OUT_owners='invented decode error') && fail "failing checks passed: $out"
grep -qx 'invented link failure' <<<"$out" || fail "verify's lib tests failed with no reason: $out"
grep -qx 'invented decode error' <<<"$out" || fail "the owners check failed with no reason: $out"

[ "$bad" -eq 0 ] && echo "GATE TEST OK: a bad base stops it, a clean branch passes and its tree's next gate reuses it, a failed gate isn't reused, an untracked subject runs its check, a failure shows its log"
exit $bad
