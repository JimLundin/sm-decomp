#!/usr/bin/env bash
# land.sh <pr>: fast-forward main to a landing PR's head once every gate passed on that exact head.
# Gate: the PR's CI run (Quality and Verify jobs) on that exact head. Exit 0 landed, 2 not ready yet, 3 main moved (rebase the slice).
set -u
pr=$1
head=$(gh pr view "$pr" --json headRefOid --jq .headRefOid)
branch=$(gh pr view "$pr" --json headRefName --jq .headRefName)
run() { gh run list --branch "$branch" --workflow "$1" --json headSha,conclusion,event,databaseId --jq "$2"; }
on_pr="[.[] | select(.headSha==\"$head\" and .event==\"pull_request\")][0].conclusion"

ci=$(run ci.yml "$on_pr")
echo "#$pr ${head:0:9} ci=${ci:-none}"
[ "$ci" = success ] || { echo "not ready"; exit 2; }

git fetch -q origin
git merge-base --is-ancestor origin/main "$head" || { echo "not a fast-forward of main; rebase the slice"; exit 3; }
git push -q origin "$head:main" || exit 1
gh pr comment "$pr" --body "Landed on main as a fast-forward to \`${head:0:9}\`; CI passed on this head." >/dev/null
echo "landed #$pr"
