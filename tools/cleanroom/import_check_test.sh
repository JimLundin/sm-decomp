#!/usr/bin/env bash
# import_check_test.sh: import.sh refuses a patch that adds a build script, leaving the branch
# where it was, and one that adds std::fs::read to a test, which the capability rule refuses after
# applying it, taking the branch back; refuses an export whose own configuration rewrites URLs;
# refuses one that adds a name scan.sh finds, naming it, with the branch untouched; imports a
# harmless one, which passes both clippy runs, though the caller's environment carries such a
# rewrite and it changes the shared types' file, whose hits on the trunk (scan.sh's names among
# them) stay; and fails an import whose change to the shared types breaks another clean crate,
# naming that crate. Builds throwaway
# exports of the dummy domain `import-test` and a throwaway worktree; run from a checkout of this
# repository. The clippy runs build in CARGO_TARGET_DIR if set, else in the throwaway worktree.
set -u
# Its own git identity, so it runs where none is configured (CI runners).
export GIT_AUTHOR_NAME=import-check GIT_AUTHOR_EMAIL=import-check@localhost
export GIT_COMMITTER_NAME=import-check GIT_COMMITTER_EMAIL=import-check@localhost
here=$(cd "$(dirname "$0")" && pwd)
repo=$(git rev-parse --show-toplevel) || exit 1
tmp=$(mktemp -d "${TEMP:-/tmp}/cc-import-XXXXXX")
branch=cc-import-test-$$
bad=0
fail() { echo "FAIL $*"; bad=1; }
cleanup() {
  git -C "$repo" worktree remove --force "$tmp/wt" 2>/dev/null
  git -C "$repo" branch -D "$branch" >/dev/null 2>&1
  rm -rf "$tmp"
}
trap cleanup EXIT

implementer_commit() { # EXPORT FILE CONTENT
  mkdir -p "$(dirname "$1/$2")"
  printf '%s\n' "$3" >"$1/$2"
  git -C "$1" add -A
  git -C "$1" -c user.name=implementer -c user.email=i@cleanroom.invalid \
    commit -q -m "Implement" -m "Role: implementer"
}

mkexport() { # DEST
  CLEANROOM_ALLOW_MISSING=1 bash "$here/export.sh" import-test "$1" >/dev/null 2>&1 || fail "export to $1"
}

git -C "$repo" worktree add -q -b "$branch" "$tmp/wt" HEAD || exit 1
mkexport "$tmp/builds"
implementer_commit "$tmp/builds" crates/clean/import_test/build.rs 'fn main() {}'
mkexport "$tmp/leaky"
implementer_commit "$tmp/leaky" crates/clean/import_test/tests/leak.rs \
  '//! Reads a file.
#[test]
fn leak() {
    let _ = std::fs::read("C:/secret");
}'
mkexport "$tmp/named"
implementer_commit "$tmp/named" crates/clean/import_test/src/lib.rs \
  '//! Names a sound by the name scan.sh knows it by.

/// The value it is given, as the jump sound.
#[must_use]
pub const fn id(x: u8) -> u8 {
    x
}'
mkexport "$tmp/fine"
# The shared types' file gains a doc paragraph above its hits on the trunk, so each moves down and
# none of them is new.
sed -i '1s|$|\n//!\n//! Each is named in player terms.|' "$tmp/fine/crates/clean/types/src/lib.rs"
grep -q 'SoundLibrary' "$tmp/fine/crates/clean/types/src/lib.rs" \
  || fail "the shared types no longer hold the trunk's scan hit this case keeps"
implementer_commit "$tmp/fine" crates/clean/import_test/src/lib.rs \
  '//! Harmless: the workspace lints it, as it does every folder of crates/clean.

/// The value it is given.
#[must_use]
pub const fn id(x: u8) -> u8 {
    x
}'
# The shared types emptied: they lint clean themselves, but every model crate that uses them stops
# compiling, and the export holds none of them.
mkexport "$tmp/breaks"
implementer_commit "$tmp/breaks" crates/clean/types/src/lib.rs \
  '//! Emptied, so the crates that use a clean type no longer build.'

before=$(git -C "$tmp/wt" rev-parse HEAD)
if out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/builds" "$branch" 2>&1); then
  fail "import accepted a build script: $out"
else
  grep -q ': extra-file: ' <<<"$out" || fail "refusal did not name extra-file: $out"
  grep -q 'nothing applied' <<<"$out" || fail "refusal did not say nothing applied: $out"
fi
[ "$(git -C "$tmp/wt" rev-parse HEAD)" = "$before" ] || fail "the refused import moved the branch"
if out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/leaky" "$branch" 2>&1); then
  fail "import accepted std::fs::read: $out"
else
  grep -q 'std::fs::read' <<<"$out" || fail "refusal did not name std::fs::read: $out"
  grep -q 'nothing applied' <<<"$out" || fail "refusal did not say nothing applied: $out"
fi
[ "$(git -C "$tmp/wt" rev-parse HEAD)" = "$before" ] || fail "the refused std::fs::read moved the branch"
[ -z "$(git -C "$tmp/wt" status --porcelain)" ] || fail "the refused std::fs::read left changes"

# A URL rewrite in the export's own configuration is refused; the same rewrite in the caller's
# environment is the caller's, and a clean export imports under it.
mkexport "$tmp/rewired"
git -C "$tmp/rewired" config url.https://elsewhere.invalid/.insteadOf cleanroom-test:
if out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/rewired" "$branch" 2>&1); then
  fail "import accepted an export that rewrites URLs: $out"
else
  grep -q 'remote or url config present' <<<"$out" || fail "refusal did not name the url config: $out"
fi
[ "$(git -C "$tmp/wt" rev-parse HEAD)" = "$before" ] || fail "the refused export moved the branch"

if out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/named" "$branch" 2>&1); then
  fail "import accepted a name scan.sh finds: $out"
else
  grep -q '^scan: crates/clean/import_test/src/lib.rs: name: jump sound$' <<<"$out" \
    || fail "refusal did not name the scan's hit: $out"
  grep -q 'nothing applied' <<<"$out" || fail "refusal did not say nothing applied: $out"
fi
[ "$(git -C "$tmp/wt" rev-parse HEAD)" = "$before" ] || fail "the refused name moved the branch"

out=$(cd "$tmp/wt" && GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=url.https://elsewhere.invalid/.insteadOf \
  GIT_CONFIG_VALUE_0=cleanroom-test: bash tools/cleanroom/import.sh "$tmp/fine" "$branch" 2>&1) \
  || fail "harmless import refused: $out"
grep -q 'IMPORT OK' <<<"$out" || fail "harmless import did not finish: $out"
grep -q 'import: workspace clippy clean' <<<"$out" || fail "harmless import ran no workspace clippy: $out"
grep -q 'import: clean-room clippy clean' <<<"$out" || fail "harmless import ran no clean-room clippy: $out"
grep -q 'import: scan.sh finds nothing new in 2 changed files' <<<"$out" \
  || fail "harmless import did not scan the shared types and its crate: $out"

# The harmless import changed the shared types' file this export replaces whole: start it where
# the others started.
git -C "$tmp/wt" reset -q --hard "$before"
if out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/breaks" "$branch" 2>&1); then
  fail "import accepted a change to the shared types that breaks the other clean crates: $out"
else
  grep -q 'IMPORT CLIPPY FAIL (workspace):.*world_blocks' <<<"$out" \
    || fail "the workspace run did not name world_blocks: $out"
  grep -q 'IMPORT CLIPPY FAIL (clean-room):.*world_blocks' <<<"$out" \
    || fail "the clean-room run did not name world_blocks: $out"
  ! grep -q 'IMPORT OK' <<<"$out" || fail "the failing import said IMPORT OK: $out"
fi
[ "$bad" -eq 0 ] && echo "IMPORT CHECK OK: a build script, std::fs::read and an export that rewrites URLs and a name scan.sh finds refused with the branch untouched, a harmless patch imported under a caller whose environment rewrites them with the trunk's scan hits kept, a change to the shared types that breaks another clean crate failed naming it"
exit $bad
