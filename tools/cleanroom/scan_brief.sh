#!/usr/bin/env bash
# scan_brief.sh BRIEF: checks 1 (addresses) and 2 (names) on a lane brief before it is sent, and
# the score guard: a brief tells the implementer the code rules, never the score, the budget, k,
# the fit or token weights (MODEL §1 decision 10; score_words.txt). A brief is prose, so the Rust
# shape checks don't apply; exits 1 on any finding.
set -euo pipefail
[ $# -eq 1 ] || { echo "usage: scan_brief.sh BRIEF" >&2; exit 2; }
case "$1" in
  *.rs) echo "scan_brief.sh: a brief is text, not Rust: $1" >&2; exit 2 ;;
esac
here=$(cd "$(dirname "$0")" && pwd)
brief=$1
if [ "$brief" = - ]; then
  # Standard input, read once for both scans.
  brief=$(mktemp)
  trap 'rm -f -- "$brief"' EXIT
  cat >"$brief"
fi
status=0
bash "$here/score_scan.sh" "$brief" || status=$?
bash "$here/scan.sh" "$brief" || { s=$?; [ "$status" -ge "$s" ] || status=$s; }
exit "$status"
