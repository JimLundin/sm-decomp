#!/usr/bin/env bash
# land.sh <pr> [expected-sha]: fast-forward main to a landing PR's head once every gate passed on
# that exact head. Gate: the PR's CI run (Quality and Verify jobs) on that exact head. With
# expected-sha (full or abbreviated), it lands only that head: a just-pushed head GitHub hasn't
# shown yet is never mistaken for the old one's green run.
# Exit 0 landed; 1 the push failed; 2 not ready yet (no run on the head, or it is still going);
# 3 main moved (rebase the slice); 4 the run on the head failed or was cancelled (read its log);
# 6 the PR's head is not expected-sha (yet).
set -u
pr=$1 expected=${2:-}
head=$(gh pr view "$pr" --json headRefOid --jq .headRefOid)
branch=$(gh pr view "$pr" --json headRefName --jq .headRefName)
if [ -n "$expected" ] && [[ $head != "$expected"* ]]; then
  echo "#$pr head is ${head:0:9}, not ${expected:0:9}"; exit 6
fi
run() { gh run list --branch "$branch" --workflow "$1" --json headSha,status,conclusion,event,databaseId --jq "$2"; }
on_pr="[.[] | select(.headSha==\"$head\" and .event==\"pull_request\")][0]"

ci=$(run ci.yml "$on_pr | if . == null then \"none\" elif .status != \"completed\" then \"pending\" else .conclusion end")
echo "#$pr ${head:0:9} ci=${ci:-none}"
case $ci in
  success) ;;
  none | pending | "") echo "not ready"; exit 2 ;;
  *) echo "the CI run on ${head:0:9} ended $ci"; exit 4 ;;
esac

git fetch -q origin
git merge-base --is-ancestor origin/main "$head" || { echo "not a fast-forward of main; rebase the slice"; exit 3; }
git push -q origin "$head:main" || exit 1
gh pr comment "$pr" --body "Landed on main as a fast-forward to \`${head:0:9}\`; CI passed on this head." >/dev/null
echo "landed #$pr"
