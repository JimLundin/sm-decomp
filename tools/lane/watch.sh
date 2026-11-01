#!/usr/bin/env bash
# watch.sh <pr>: poll a landing PR every 2 minutes and land it when green (tools/lane/land.sh).
# Exit 0 landed, 3 main moved (rebase), 4 a gating run failed on the head (read its log), 5 timed out.
# Run it in the background; it prints one line per poll.
set -u
pr=$1; here=$(dirname "$0")
branch=$(gh pr view "$pr" --json headRefName --jq .headRefName)
gates='["CI","Corpus replay","Full 100% route","Engine against the oracle lock"]'
for _ in $(seq 1 90); do
  out=$(bash "$here/land.sh" "$pr" 2>&1); rc=$?; echo "$out" | tail -1
  [ $rc = 0 ] && exit 0
  [ $rc = 3 ] && exit 3
  head=$(gh pr view "$pr" --json headRefOid --jq .headRefOid)
  failed=$(gh run list --branch "$branch" --json conclusion,headSha,workflowName \
    --jq "[.[] | select(.conclusion==\"failure\" and .headSha==\"$head\" and (.workflowName as \$w | $gates | index(\$w)))] | length")
  [ "${failed:-0}" != 0 ] && { echo "a gating run failed on ${head:0:9}"; exit 4; }
  sleep 120
done
exit 5
