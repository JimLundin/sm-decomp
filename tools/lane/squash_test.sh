#!/usr/bin/env bash
# squash_test.sh: squash.sh makes one commit with the lane's tree on HEAD, files each commit
# under its role with its body, keeps every Role and Co-Authored-By line once at the end, gives a
# lane of one commit that commit's body, and refuses a lane that doesn't hold HEAD or a subject
# without its kind, changing nothing. Builds a throwaway repository.
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
short() { git rev-parse --short=9 "$1"; }
commit base "Base [exact]"
git checkout -q -b lane
commit spec "Spec [exact]" $'Why the spec.\nIts second line.' "Second paragraph." "Role: observer" \
  "Co-Authored-By: A <a@x>"
commit model "Model" $'Why the model.\nRole: implementer' "After a trailer."
commit glue "Glue [exact]" "Role: integrator" "Co-Authored-By: A <a@x>"
commit note "Note [exact]" "Why the note." "Co-Authored-By: B <b@x>"
git checkout -q -b landing main

out=$(bash "$squash" "Round [exact]" lane 2>&1) || fail "squash refused: $out"
[ "$(git rev-parse HEAD^{tree})" = "$(git rev-parse lane^{tree})" ] || fail "the tree isn't the lane's"
[ "$(git rev-parse HEAD^)" = "$(git rev-parse main)" ] || fail "the parent isn't the base"
[ "$(git rev-list --count main..HEAD)" = 1 ] || fail "not one commit"
roles=$(git log -1 --format='%(trailers:key=Role,valueonly,separator=%x2C)')
[ "$roles" = "observer,implementer,integrator" ] || fail "Role trailers: $roles"
authors=$(git log -1 --format='%(trailers:key=Co-Authored-By,valueonly,separator=%x2C)')
[ "$authors" = "A <a@x>,B <b@x>" ] || fail "Co-Authored-By trailers: $authors"
# Each body sits indented under its commit's line without its trailers, even one in the middle of
# a body; the trailers come once each, in the last paragraph only.
expected="Round [exact]

Spec (observer):
- $(short lane~3) Spec [exact]
  Why the spec.
  Its second line.

  Second paragraph.

Model (implementer):
- $(short lane~2) Model
  Why the model.

  After a trailer.

Integration (integrator):
- $(short lane~1) Glue [exact]

Other (no Role line):
- $(short lane) Note [exact]
  Why the note.

Squashed from lane at $(short lane): 4 commits on $(short main).

Role: observer
Role: implementer
Role: integrator
Co-Authored-By: A <a@x>
Co-Authored-By: B <b@x>"
message=$(git log -1 --format=%B)
[ "$message" = "$expected" ] || fail "the round's message is
$message"

# A lane of one commit has that commit's body as the message's own body, above its list line.
git checkout -q -b single main
commit fix "Fix [exact]" "Why the fix." "Its consequence." "Role: engineer" "Co-Authored-By: A <a@x>"
git checkout -q -b landing-single main
out=$(bash "$squash" "Fix [exact]" single 2>&1) || fail "squash refused one commit: $out"
expected="Fix [exact]

Why the fix.

Its consequence.

Engineering (engineer):
- $(short single) Fix [exact]

Squashed from single at $(short single): 1 commit on $(short main).

Role: engineer
Co-Authored-By: A <a@x>"
message=$(git log -1 --format=%B)
[ "$message" = "$expected" ] || fail "the one commit's message is
$message"

git checkout -q -b stale main
commit other "Other [exact]"
head=$(git rev-parse HEAD)
bash "$squash" "Round [exact]" lane >/dev/null 2>&1 && fail "a lane that doesn't hold HEAD was squashed"
bash "$squash" "Round" HEAD >/dev/null 2>&1 && fail "a subject without its kind was taken"
[ "$(git rev-parse HEAD)" = "$head" ] || fail "a refusal moved HEAD"

[ "$bad" -eq 0 ] && echo "SQUASH TEST OK"
exit "$bad"
