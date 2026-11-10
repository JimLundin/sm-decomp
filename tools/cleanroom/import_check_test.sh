#!/usr/bin/env bash
# import_check_test.sh: import.sh refuses a patch that adds std::fs::read to a test, leaving the
# branch where it was; imports a harmless one, which passes both clippy runs; and fails an import
# whose clean_types change breaks another clean crate, naming that crate. Builds throwaway exports
# of the dummy domain `import-test` and a throwaway worktree; run from a checkout of this
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
mkexport "$tmp/leaky"
implementer_commit "$tmp/leaky" crates/clean_model/import-test/tests/leak.rs \
  '#[test] fn leak() { let _ = std::fs::read("C:/secret"); }'
mkexport "$tmp/fine"
implementer_commit "$tmp/fine" crates/clean_model/import-test/src/lib.rs \
  '//! Harmless.
pub fn id(x: u8) -> u8 { x }'
# clean_types emptied: it lints clean itself, but every model crate that uses it stops compiling,
# and the export holds none of them.
mkexport "$tmp/breaks"
implementer_commit "$tmp/breaks" crates/clean_types/src/lib.rs \
  '//! Emptied, so the crates that use a clean type no longer build.'

before=$(git -C "$tmp/wt" rev-parse HEAD)
if out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/leaky" "$branch" 2>&1); then
  fail "import accepted std::fs::read: $out"
else
  grep -q ': std-fs: ' <<<"$out" || fail "refusal did not name std-fs: $out"
  grep -q 'nothing applied' <<<"$out" || fail "refusal did not say nothing applied: $out"
fi
[ "$(git -C "$tmp/wt" rev-parse HEAD)" = "$before" ] || fail "the refused import moved the branch"

out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/fine" "$branch" 2>&1) || fail "harmless import refused: $out"
grep -q 'IMPORT OK' <<<"$out" || fail "harmless import did not finish: $out"
grep -q 'import: workspace clippy clean' <<<"$out" || fail "harmless import ran no workspace clippy: $out"
grep -q 'import: clean-room clippy clean' <<<"$out" || fail "harmless import ran no clean-room clippy: $out"

if out=$(cd "$tmp/wt" && bash tools/cleanroom/import.sh "$tmp/breaks" "$branch" 2>&1); then
  fail "import accepted a clean_types change that breaks the other clean crates: $out"
else
  grep -q 'IMPORT CLIPPY FAIL (workspace):.*clean_model_blocks' <<<"$out" \
    || fail "the workspace run did not name clean_model_blocks: $out"
  grep -q 'IMPORT CLIPPY FAIL (clean-room):.*clean_model_blocks' <<<"$out" \
    || fail "the clean-room run did not name clean_model_blocks: $out"
  ! grep -q 'IMPORT OK' <<<"$out" || fail "the failing import said IMPORT OK: $out"
fi
[ "$bad" -eq 0 ] && echo "IMPORT CHECK OK: std::fs::read refused with the branch untouched, a harmless patch imported, a clean_types change that breaks another clean crate failed naming it"
exit $bad
