#!/usr/bin/env bash
# gate_test.sh: gate.sh, run in a scratch repository whose tools and toolchain are fakes that
# record that they ran, stops before any check on a base that doesn't resolve; passes a clean
# branch; reuses that pass for a new commit of the same tree over the same base, running nothing,
# unless SM_GATE_FRESH=1, other compiler flags or overrides, cargo-deny's or shellcheck's presence,
# a cargo or rustfmt configuration outside the tree, a configuration that includes another, a
# file only an outside ignore rule hides, or a skip-worktree file says otherwise; never reuses a failed gate; runs a check for an untracked file in its subject
# even where git status hides untracked files; and shows a failing check's last lines when its tool prints none of the lines the gate
# looks for. Run from anywhere; exit 1 lists what went wrong.
here=$(cd "$(dirname "$0")" && pwd)
bad=0
fail() { echo "FAIL $*"; bad=1; }
scratch=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/gate-test-XXXXXX") || exit 1
trap 'rm -rf "$scratch"' EXIT
repo=$scratch/repo bin=$scratch/bin ran=$scratch/ran
mkdir -p "$repo/tools/ci" "$repo/tools/lane" "$repo/tools/cleanroom" "$repo/scenarios" "$bin" "$ran"
cp "$here/../ci/lint_clean.sh" "$repo/tools/ci/"
cp "$here/gate.sh" "$repo/tools/lane/gate.sh"
# Run the real selector, mapping its /tmp result to a private stand-in for the fallback case.
sed 's/^outside_home_temp() {/select_outside_home_temp() {/' "$here/../cleanroom/lib.sh" \
  >"$repo/tools/cleanroom/lib.sh" || exit 1
cat >>"$repo/tools/cleanroom/lib.sh" <<'EOF' || exit 1
outside_home_temp() {
  local selected
  selected=$(select_outside_home_temp) || return
  if [ "$selected" = /tmp ] && [ -n "${GATE_TEST_FALLBACK:-}" ]; then
    selected=$GATE_TEST_FALLBACK
  fi
  printf '%s\n' "$selected"
}
EOF
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
    "r'$ran'" "$name" "$name" "$name" >"$path"
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
  [ -z "$dir" ] || [ -e "$dir/cargo-deny" ] || no_deny=$no_deny:$dir
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
# A rustfmt configuration above the checkout, a self-test's interpreter override, and shellcheck
# installed each change what a gate runs: none reuses the pass recorded without them.
out=$(gate) || fail "the gate before the outside changes failed: $out"
out=$(gate) && grep -q '^GATE REUSED: ' <<<"$out" || fail "the same tree's gate reused nothing: $out"
echo 'hard_tabs = true' >"$scratch/rustfmt.toml"
out=$(gate) || fail "a gate under a rustfmt.toml failed: $out"
! grep -q '^GATE REUSED: ' <<<"$out" || fail "a gate under a rustfmt.toml reused a pass: $out"
rm "$scratch/rustfmt.toml"
out=$(gate CLEANROOM_PY=/elsewhere/python) || fail "a gate with CLEANROOM_PY failed: $out"
! grep -q '^GATE REUSED: ' <<<"$out" || fail "a gate with CLEANROOM_PY reused a pass: $out"
fake shellcheck "$bin/shellcheck"
out=$(gate) || fail "a gate with shellcheck failed: $out"
! grep -q '^GATE REUSED: ' <<<"$out" || fail "a gate with shellcheck installed reused a pass made without it: $out"
rm "$bin/shellcheck"
# Whether a gate run with these settings reused a pass; each case first records a pass with the
# same settings, then changes one thing outside the tree.
reuses() { out=$(gate "$@") || fail "a gate with ${*:-no settings} failed: $out"; grep -q '^GATE REUSED: ' <<<"$out"; }
settled() { reuses "$@" || reuses "$@" || fail "a second gate with ${*:-no settings} reused nothing: $out"; }
settled
out=$(gate CANARY_SALT=other) && ! grep -q '^GATE REUSED: ' <<<"$out" || fail "a gate with a CANARY_ variable reused a pass: $out"
mkdir -p "$scratch/.cargo" && echo '[build]' >"$scratch/.cargo/config.toml"
! reuses || fail "a gate under a parent .cargo/config.toml reused a pass: $out"
for include in 'include = [{ path = "more.toml" }]' '"include" = ["more.toml"]' \
  "'include' = [\"more.toml\"]" $'[[include]]\npath = "more.toml"'; do
  echo "$include" >"$scratch/.cargo/config.toml"
  ! reuses && ! reuses ||
    fail "a gate under a cargo configuration with $include reused a pass: $out"
done
rm -r "$scratch/.cargo"
settled
mkdir -p "$scratch/no-cargo-home" && echo '[build]' >"$scratch/no-cargo-home/config.toml"
! reuses || fail "a gate with a CARGO_HOME configuration reused a pass: $out"
rm "$scratch/no-cargo-home/config.toml"
settled XDG_CONFIG_HOME="$scratch/xdg" HOME="$scratch/home"
mkdir -p "$scratch/xdg/rustfmt" && echo 'hard_tabs = true' >"$scratch/xdg/rustfmt/.rustfmt.toml"
! reuses XDG_CONFIG_HOME="$scratch/xdg" HOME="$scratch/home" || fail "a gate with rustfmt's user configuration reused a pass: $out"
rm -r "$scratch/xdg"
settled XDG_CONFIG_HOME="$scratch/xdg" HOME="$scratch/home"
mkdir -p "$scratch/home" && echo 'hard_tabs = true' >"$scratch/home/rustfmt.toml"
! reuses XDG_CONFIG_HOME="$scratch/xdg" HOME="$scratch/home" || fail "a gate with a rustfmt.toml in the home reused a pass: $out"
rm "$scratch/home/rustfmt.toml"
settled
ln -s rustfmt.toml "$scratch/rustfmt.toml"
! reuses || fail "a gate under a rustfmt.toml that is a link loop reused a pass: $out"
rm "$scratch/rustfmt.toml"
# Cargo's configuration above TEMP, where the import self-test builds.
elsewhere=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/gate-test-temp-XXXXXX")
mkdir -p "$elsewhere/tmp"
settled TEMP="$elsewhere/tmp"
mkdir -p "$elsewhere/.cargo" && echo '[build]' >"$elsewhere/.cargo/config.toml"
! reuses TEMP="$elsewhere/tmp" || fail "a gate with a cargo configuration above TEMP reused a pass: $out"
rm -r "$elsewhere"
# With TEMP in the home, the real selector returns /tmp. Give that result a private
# stand-in beside the repo and home so only the selected build path can see its config.
home_temp=$scratch/home/tmp
fallback=$scratch/fallback
mkdir -p "$home_temp" "$fallback" || exit 1
selected=$(export HOME="$scratch/home" TEMP="$home_temp"; . "$here/../cleanroom/lib.sh"; outside_home_temp)
[ "$selected" = /tmp ] || fail "the self-tests selected '$selected' with TEMP inside the home"
settings=(HOME="$scratch/home" TEMP="$home_temp" GATE_TEST_FALLBACK="$fallback")
settled "${settings[@]}"
if ! (mkdir "$fallback/.cargo" && echo '[build]' >"$fallback/.cargo/config.toml"); then
  fail "could not install the cargo configuration above the self-tests' build directory"
  exit 1
fi
! reuses "${settings[@]}" ||
  fail "a gate with a cargo configuration above the self-tests' build directory reused a pass: $out"
# A file git ignores only by a rule from outside the tree is unseen by status, not by the gate.
settled
echo 'hard_tabs = true' >"$repo/rustfmt.toml" && echo rustfmt.toml >"$scratch/excludes"
! reuses GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.excludesFile GIT_CONFIG_VALUE_0="$scratch/excludes" &&
  ! reuses GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.excludesFile GIT_CONFIG_VALUE_0="$scratch/excludes" ||
  fail "a gate beside a file only an outside ignore rule hides reused a pass: $out"
rm "$repo/rustfmt.toml"

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
