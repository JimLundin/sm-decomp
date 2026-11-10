#!/usr/bin/env bash
# squash_test.sh: squash.sh makes one commit with the lane's tree on HEAD, files each commit
# under its role, keeps every Role and Co-Authored-By line once, and refuses a lane that doesn't
# hold HEAD or a subject without its kind, changing nothing. Builds a throwaway repository.
set -u
export GIT_AUTHOR_NAME=squash-check GIT_AUTHOR_EMAIL=squash-check@localhost
export GIT_COMMITTER_NAME=squash-check GIT_COMMITTER_EMAIL=squash-check@localhost
squash="$(cd "$(dirname "$0")" && pwd)/squash.sh"
tmp=$(mktemp -d "${TEMP:-/tmp}/squash-test-XXXXXX")
trap 'rm -rf "$tmp"' EXIT
bad=0
fail() { echo "FAIL $*"; bad=1; }
cd "$tmp" || exit 1
git init -q -b main . && git config commit.gpgsign false && git config core.autocrlf false
commit() { # FILE MESSAGE...
  echo "$1" >>"$1"
  git add -A
  shift
  local args=()
  for part in "$@"; do args+=(-m "$part"); done
  git commit -q "${args[@]}"
}
commit base "Base [exact]"
git checkout -q -b lane
commit spec "Spec [exact]" "Role: observer" "Co-Authored-By: A <a@x>"
commit model "Model" "Role: implementer"
commit glue "Glue [exact]" "Role: integrator" "Co-Authored-By: A <a@x>"
commit note "Note [exact]" "Co-Authored-By: B <b@x>"
git checkout -q -b landing main

out=$(bash "$squash" "Round [exact]" lane 2>&1) || fail "squash refused: $out"
[ "$(git rev-parse HEAD^{tree})" = "$(git rev-parse lane^{tree})" ] || fail "the tree isn't the lane's"
[ "$(git rev-parse HEAD^)" = "$(git rev-parse main)" ] || fail "the parent isn't the base"
[ "$(git rev-list --count main..HEAD)" = 1 ] || fail "not one commit"
message=$(git log -1 --format=%B)
[ "$(git log -1 --format=%s)" = "Round [exact]" ] || fail "subject"
for heading in "Spec (observer):" "Model (implementer):" "Integration (integrator):" "Other (no Role line):"; do
  printf '%s\n' "$message" | grep -qxF "$heading" || fail "no heading $heading"
done
roles=$(git log -1 --format='%(trailers:key=Role,valueonly,separator=%x2C)')
[ "$roles" = "observer,implementer,integrator" ] || fail "Role trailers: $roles"
authors=$(git log -1 --format='%(trailers:key=Co-Authored-By,valueonly,separator=%x2C)')
[ "$authors" = "A <a@x>,B <b@x>" ] || fail "Co-Authored-By trailers: $authors"

git checkout -q -b stale main
commit other "Other [exact]"
head=$(git rev-parse HEAD)
bash "$squash" "Round [exact]" lane >/dev/null 2>&1 && fail "a lane that doesn't hold HEAD was squashed"
bash "$squash" "Round" HEAD >/dev/null 2>&1 && fail "a subject without its kind was taken"
[ "$(git rev-parse HEAD)" = "$head" ] || fail "a refusal moved HEAD"

[ "$bad" -eq 0 ] && echo "SQUASH TEST OK"
exit "$bad"
