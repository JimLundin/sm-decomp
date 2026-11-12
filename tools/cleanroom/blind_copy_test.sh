#!/usr/bin/env bash
# blind_copy_test.sh: blind_copy.sh turns a made-up export into one commit holding the crates,
# the vector file and the blind-reviewer skill alone: no docs/, CLEANROOM.md or other skill, and
# without the test that reads the spec, while the test that reads the vectors stays. It refuses a
# source outside tests/ that reads docs/. CARGO is unset for it: the made-up crates don't build.
# Run from anywhere; exit 1 lists what went wrong.
set -u
here=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp -d "${TEMP:-/tmp}/blind-copy-XXXXXX")
trap 'rm -rf "$tmp"' EXIT
bad=0
fail() { echo "FAIL $*"; bad=1; }

put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "$2" >"$1"; }
mkexport() { # DIR: an export of the dummy domain `demo`, as export.sh lays one out
  put "$1/docs/behaviours/demo.md" "# Demo spec"
  put "$1/docs/cleanroom/DESIGN.md" "# Design"
  put "$1/CLEANROOM.md" "# Clean room: demo"
  put "$1/.agents/skills/implementer/SKILL.md" "# Implement"
  put "$1/Cargo.toml" '[workspace]'
  put "$1/scenarios/vectors/demo.toml" '[[vector]]'
  put "$1/crates/clean/types/src/lib.rs" '//! Shared types.'
  put "$1/crates/clean/demo/Cargo.toml" '[package]'
  put "$1/crates/clean/demo/src/lib.rs" '//! The model.'
  put "$1/crates/clean/demo/tests/spec.rs" \
    'const SPEC: &str = include_str!("../../../../docs/behaviours/demo.md");'
  put "$1/crates/clean/demo/tests/vectors.rs" \
    'const VECTORS: &str = include_str!("../../../../scenarios/vectors/demo.toml");'
  git -c core.autocrlf=false -C "$1" init -q -b main
  git -c core.autocrlf=false -C "$1" add -A
  git -c core.autocrlf=false -C "$1" -c user.name=cleanroom -c user.email=cleanroom@invalid commit -q -m "Clean-room export: demo"
}

mkexport "$tmp/export"
out=$(env -u CARGO bash "$here/blind_copy.sh" "$tmp/export" "$tmp/blind" 2>&1) || fail "the copy failed: $out"
files=$(git -C "$tmp/blind" ls-files | sort)
want=".agents/skills/blind-reviewer/SKILL.md
Cargo.toml
crates/clean/demo/Cargo.toml
crates/clean/demo/src/lib.rs
crates/clean/demo/tests/vectors.rs
crates/clean/types/src/lib.rs
scenarios/vectors/demo.toml"
[ "$files" = "$want" ] || fail "the copy holds: $files"
[ "$(git -C "$tmp/blind" rev-list --count HEAD)" -eq 1 ] || fail "not one commit"
[ -z "$(git -C "$tmp/blind" status --porcelain)" ] || fail "the copy has uncommitted files"
[ -z "$(git -C "$tmp/blind" remote)" ] || fail "the copy has a remote"
cmp -s "$tmp/blind/.agents/skills/blind-reviewer/SKILL.md" "$here/../../.agents/skills/blind-reviewer/SKILL.md" ||
  fail "the skill is not this repository's"
grep -q 'left out crates/clean/demo/tests/spec.rs' <<<"$out" || fail "the left-out test not named: $out"

mkexport "$tmp/leaky"
put "$tmp/leaky/crates/clean/demo/src/spec.rs" 'const SPEC: &str = include_str!("../../../../docs/behaviours/demo.md");'
git -c core.autocrlf=false -C "$tmp/leaky" add -A
git -c core.autocrlf=false -C "$tmp/leaky" -c user.name=cleanroom -c user.email=cleanroom@invalid commit -q -m "Implement"
env -u CARGO bash "$here/blind_copy.sh" "$tmp/leaky" "$tmp/leaky-blind" >/dev/null 2>&1 &&
  fail "a source outside tests/ that reads docs/ was copied"

mkdir -p "$tmp/full" && touch "$tmp/full/x"
env -u CARGO bash "$here/blind_copy.sh" "$tmp/export" "$tmp/full" >/dev/null 2>&1 && fail "a non-empty DEST was used"

[ "$bad" -eq 0 ] && echo "BLIND COPY OK: crates, vectors and the reviewer's skill alone, in one commit"
exit $bad
