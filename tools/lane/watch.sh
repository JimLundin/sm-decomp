#!/usr/bin/env bash
# watch.sh <pr> [expected-sha]: poll a landing PR every 2 minutes and land it when green
# (tools/lane/land.sh, which with expected-sha lands only that head).
# Exit 0 landed, 3 main moved (rebase), 4 a gating run failed or was cancelled on the head (read
# its log), 5 timed out, 6 timed out with the PR's head still not expected-sha.
# Run it in the background; it prints one line per poll.
set -u
pr=$1 expected=${2:-}; here=$(dirname "$0")
branch=$(gh pr view "$pr" --json headRefName --jq .headRefName)
gates='["CI","Verify"]'
rc=5
for _ in $(seq 1 90); do
  out=$(bash "$here/land.sh" "$pr" $expected 2>&1); rc=$?; echo "$out" | tail -1
  case $rc in 0 | 3 | 4) exit $rc ;; esac
  # A head that isn't the expected one yet is waited for: GitHub shows a push after a delay.
  if [ $rc != 6 ]; then
    head=$(gh pr view "$pr" --json headRefOid --jq .headRefOid)
    failed=$(gh run list --branch "$branch" --json conclusion,headSha,workflowName,event \
      --jq "[.[] | select((.conclusion==\"failure\" or .conclusion==\"cancelled\") and .event==\"pull_request\" and .headSha==\"$head\" and (.workflowName as \$w | $gates | index(\$w)))] | length")
    [ "${failed:-0}" != 0 ] && { echo "a gating run failed on ${head:0:9}"; exit 4; }
  fi
  sleep 120
done
[ $rc = 6 ] && exit 6
exit 5
