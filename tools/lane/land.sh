#!/usr/bin/env bash
# land.sh <pr>: fast-forward main to a landing PR's head once every gate passed on that exact head.
# Gates: PR CI, a dispatched corpus run (segments.yml), and, when the head has them, the full route
# and the oracle lock gate. Exit 0 landed, 2 not ready yet, 3 main moved (rebase the slice).
set -u
pr=$1
repo=$(gh repo view --json nameWithOwner --jq .nameWithOwner)
head=$(gh pr view "$pr" --json headRefOid --jq .headRefOid)
branch=$(gh pr view "$pr" --json headRefName --jq .headRefName)
run() { gh run list --branch "$branch" --workflow "$1" --json headSha,conclusion,event,databaseId --jq "$2"; }
on_pr="[.[] | select(.headSha==\"$head\" and .event==\"pull_request\")][0].conclusion"
has() { gh api "repos/$repo/contents/.github/workflows/$1?ref=$head" --jq .name >/dev/null 2>&1; }

ci=$(run ci.yml "$on_pr")
corpus=$(run segments.yml "[.[] | select(.headSha==\"$head\" and .conclusion==\"success\")][0].databaseId")
route=success; has full-route.yml && route=$(run full-route.yml "$on_pr")
lock=success; has lock-gate.yml && lock=$(run lock-gate.yml "$on_pr")
echo "#$pr ${head:0:9} ci=${ci:-none} corpus=${corpus:-none} route=${route:-none} lock=${lock:-none}"
[ "$ci" = success ] && [ -n "$corpus" ] && [ "$route" = success ] && [ "$lock" = success ] || { echo "not ready"; exit 2; }

git fetch -q origin
git merge-base --is-ancestor origin/main "$head" || { echo "not a fast-forward of main; rebase the slice"; exit 3; }
git push -q origin "$head:main" || exit 1
gh pr comment "$pr" --body "Landed on main as a fast-forward to \`${head:0:9}\`; CI, corpus, route and lock gate passed on this head." >/dev/null
echo "landed #$pr"
