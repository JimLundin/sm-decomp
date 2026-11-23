#!/usr/bin/env bash
# worktree_test.sh: worktree.sh add gives a new lane worktree the seed's build state, hard-linked
# except for the dep-info files cargo rewrites in place, so the lane's rewrites leave the seed as
# it was; it leaves newer than that state exactly the files that differ from the seed's commit;
# SM_SEED_COPY=1 copies instead; without a seed the lane starts cold; remove deletes the worktree,
# its target/ and its test-DLL folder. Builds a throwaway repository.
set -u
export GIT_AUTHOR_NAME=worktree-check GIT_AUTHOR_EMAIL=worktree-check@localhost
export GIT_COMMITTER_NAME=worktree-check GIT_COMMITTER_EMAIL=worktree-check@localhost
worktree="$(cd "$(dirname "$0")" && pwd)/worktree.sh"
tmp=$(mktemp -d "${TEMP:-/tmp}/worktree-test-XXXXXX")
trap 'rm -rf "$tmp"' EXIT
bad=0
fail() { echo "FAIL $*"; bad=1; }
mtime() { stat -c %.9Y "$1"; }
newer() { awk -v a="$(mtime "$1")" -v b="$(mtime "$2")" 'BEGIN { exit !(a > b) }'; }
linked() { [ "$(stat -c %i "$1")" = "$(stat -c %i "$2")" ]; }
cd "$tmp" || exit 1
mkdir repo && cd repo || exit 1
git init -q -b main . && git config commit.gpgsign false && git config core.autocrlf false
printf 'target/\n' >.gitignore
echo same >same.rs && echo old >changed.rs && echo gone >deleted.rs
git add -A && git commit -q -m seed
git worktree add -q --detach "$tmp/seed" HEAD
built="$tmp/seed/target/debug"
mkdir -p "$built/deps" "$built/.fingerprint/x-1" "$tmp/seed/target/release"
echo built >"$built/deps/x.rlib" && echo info >"$built/deps/x.d"
echo info >"$built/.fingerprint/x-1/dep-lib-x" && echo shipped >"$tmp/seed/target/release/out"

export SM_SEED_DIR="$tmp/seed" SM_TEST_DLL_DIR="$tmp/dll"
bash "$worktree" add cold "$tmp/cold" HEAD | grep -q "first build is cold" || fail "an unbuilt seed is used"
[ -e "$tmp/cold/target" ] && fail "an unbuilt seed is copied"
touch "$tmp/seed/target/.seeded"

echo new >changed.rs && git rm -q deleted.rs && echo added >added.rs
git add -A && git commit -q -m base
for lane in linked copied; do
  if [ "$lane" = copied ]; then export SM_SEED_COPY=1; fi
  bash "$worktree" add "$lane" "$tmp/$lane" main | grep -q "$lane the build" || fail "$lane: add failed"
  lane_built="$tmp/$lane/target/debug"
  [ "$(git -C "$tmp/$lane" rev-parse HEAD)" = "$(git rev-parse main)" ] || fail "$lane: not at its base"
  [ "$(cat "$lane_built/deps/x.rlib" 2>/dev/null)" = built ] || fail "$lane: the build state is missing"
  [ -e "$tmp/$lane/target/release" ] && fail "$lane: a release build is taken"
  if [ "$lane" = linked ]; then
    linked "$lane_built/deps/x.rlib" "$built/deps/x.rlib" || fail "linked: an artifact is copied"
  else
    linked "$lane_built/deps/x.rlib" "$built/deps/x.rlib" && fail "copied: an artifact is linked"
  fi
  for info in deps/x.d .fingerprint/x-1/dep-lib-x; do
    [ "$(cat "$lane_built/$info" 2>/dev/null)" = info ] || fail "$lane: $info is missing"
    linked "$lane_built/$info" "$built/$info" && fail "$lane: $info is shared"
    echo rewritten >"$lane_built/$info"
    [ "$(cat "$built/$info")" = info ] || fail "$lane: rewriting $info changed the seed's"
  done
  newer "$tmp/$lane/changed.rs" "$lane_built/deps/x.rlib" || fail "$lane: a changed source looks built"
  newer "$tmp/$lane/added.rs" "$lane_built/deps/x.rlib" || fail "$lane: an added source looks built"
  newer "$tmp/$lane/same.rs" "$lane_built/deps/x.rlib" && fail "$lane: an unchanged source looks changed"
  [ -z "$(git -C "$tmp/$lane" status --porcelain)" ] || fail "$lane: the worktree is not clean"
done

mkdir -p "$tmp/dll" && echo dll >"$tmp/dll/x.dll"
bash "$worktree" remove "$tmp/linked" || fail "remove failed"
[ -e "$tmp/linked" ] && fail "the worktree remains"
[ -e "$tmp/dll" ] && fail "the test-DLL folder remains"
[ "$(cat "$built/deps/x.rlib")" = built ] || fail "remove touched the seed"
exit "$bad"
