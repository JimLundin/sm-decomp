#!/usr/bin/env bash
# land.sh <pr>: fast-forward main to a landing PR's head once every gate passed on that exact head.
# Gates: exact-head PR CI and fully qualified Verify (dispatch/full). Exit 0 landed, 2 not ready yet, 3 main moved (rebase the slice).
set -u
pr=$1
head=$(gh pr view "$pr" --json headRefOid --jq .headRefOid)
branch=$(gh pr view "$pr" --json headRefName --jq .headRefName)
run() { gh run list --branch "$branch" --workflow "$1" --json headSha,conclusion,event,databaseId --jq "$2"; }
on_pr="[.[] | select(.headSha==\"$head\" and .event==\"pull_request\")][0].conclusion"

ci=$(run ci.yml "$on_pr")
verify=""
ids=$(run verify.yml "[.[] | select(.headSha==\"$head\" and .conclusion==\"success\")][].databaseId")
for id in $ids; do
  qualified=$(gh api "repos/{owner}/{repo}/actions/runs/$id/jobs?per_page=100" --paginate --jq '.jobs[] | select(.name=="Verify full result" and .conclusion=="success") | .id')
  if [ -n "$qualified" ]; then verify=success; break; fi
done
echo "#$pr ${head:0:9} ci=${ci:-none} verify=${verify:-none}"
[ "$ci" = success ] && [ "$verify" = success ] || { echo "not ready"; exit 2; }

git fetch -q origin
git merge-base --is-ancestor origin/main "$head" || { echo "not a fast-forward of main; rebase the slice"; exit 3; }
git push -q origin "$head:main" || exit 1
gh pr comment "$pr" --body "Landed on main as a fast-forward to \`${head:0:9}\`; CI and Verify passed on this head." >/dev/null
echo "landed #$pr"
