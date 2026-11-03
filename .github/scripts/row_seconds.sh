#!/bin/sh
# Regenerate .github/scripts/row_seconds.toml from the replay jobs of completed ci.yml runs:
#   sh .github/scripts/row_seconds.sh RUN_ID... > .github/scripts/row_seconds.toml
# A row's seconds run from its first "SEGMENT <id>:" line to its "PASS <id>" line in the job
# log, so they include its setup and any checkpoint prefix. Jobs named lock-* check the lock
# (engine only); oracle-* recordings are skipped; every other job is lockstep. A row measured
# in several runs keeps the mean. Logs carry only ids and timestamps here; nothing ROM-derived.
set -eu
repo=${GITHUB_REPOSITORY:-JimLundin/sm-decomp}
for run in "$@"; do
  gh api "repos/$repo/actions/runs/$run/jobs?per_page=100" --paginate \
    --jq '.jobs[] | select(.conclusion == "success") | "\(.id) \(.name)"' |
  while read -r id name; do
    # Called jobs are named "<caller job> / <job>" (ci.yml calls verify.yml).
    name=${name##* / }
    case $name in
      lock-*) mode=lock ;;
      oracle-*|record-*|rom-import|verify*|Verify*|Quality|detached|desktop-check) continue ;;
      *) mode=lockstep ;;
    esac
    gh run view --repo "$repo" --job "$id" --log | tr -d '\r' | awk -F '\t' -v mode="$mode" '
      function at(stamp,  t) { t = substr(stamp, 12, 16); split(t, p, ":"); return p[1] * 3600 + p[2] * 60 + p[3] }
      {
        line = $0; sub(/^[^\t]*\t[^\t]*\t/, "", line); stamp = substr(line, 1, 28); text = substr(line, 30)
        if (match(text, /^SEGMENT [A-Za-z0-9-]+:/)) {
          id = substr(text, 9, RLENGTH - 9)
          if (!(id in start)) start[id] = at(stamp)
        } else if (match(text, /^PASS [A-Za-z0-9-]+$/)) {
          id = substr(text, 6)
          if (id in start) { s = at(stamp) - start[id]; if (s < 0) s += 86400; print mode, id, s }
        }
      }'
  done
done | awk '
  { key = $1 " " $2; sum[key] += $3; count[key]++ }
  END { for (key in sum) { split(key, k, " "); printf "%s %s %.0f\n", k[1], k[2], sum[key] / count[key] } }' |
sort -k1,1 -k2,2 | awk -v runs="$*" '
  BEGIN {
    print "# Measured seconds of each scenario replay on a CI runner, by verifier mode."
    print "# sm_pipeline plan (pack.rs) packs replay jobs by them; a row missing here is"
    print "# estimated from its frames. Regenerate with .github/scripts/row_seconds.sh."
    print "# Runs: " runs
  }
  $1 != mode { mode = $1; printf "\n[%s]\n", mode }
  { printf "%s = %d\n", $2, ($3 < 1 ? 1 : $3) }'
