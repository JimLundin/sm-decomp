#!/usr/bin/env bash
# Fails while docs/architecture/MODEL-REVIEW.md has an acknowledgement row still marked pending.
# Jim's review of MODEL.md must be answered item by item before lane work continues.
# Remove this script, its gate and CI lines, and the banners once every row is answered and Jim
# has seen the answers.
set -u
f=docs/architecture/MODEL-REVIEW.md
[ -f "$f" ] || exit 0
pending=$(grep -E '^\| R[0-9]+ \| pending \|' "$f" | cut -d'|' -f2 | tr -d ' ' | tr '\n' ' ')
if [ -n "$pending" ]; then
  echo "REVIEW NOT ACKNOWLEDGED: Jim's review of MODEL.md is waiting for you."
  echo "  Pause after your current commit. Read $f and answer every row of its"
  echo "  Acknowledgement table (accepted / rejected / deferred / ask Jim, with one line why)."
  echo "  Pending: $pending"
  exit 1
fi
