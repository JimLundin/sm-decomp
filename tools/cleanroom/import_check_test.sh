#!/usr/bin/env bash
# import_check_test.sh: import.sh refuses a patch that adds std::fs::read to a test, leaving the
# branch where it was, and still imports a harmless one. Builds a throwaway export of the dummy
# domain `import-test` and a throwaway worktree; run from a checkout of this repository.
set -u
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
[ "$bad" -eq 0 ] && echo "IMPORT CHECK OK: std::fs::read refused with the branch untouched, a harmless patch imported"
exit $bad
