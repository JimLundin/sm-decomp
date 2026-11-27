#!/usr/bin/env bash
# scan_brief.sh BRIEF: checks 1 (addresses) and 2 (names) on a lane brief before it is sent. A
# brief is prose, so the Rust shape checks don't apply; exits 1 on any finding.
set -euo pipefail
[ $# -eq 1 ] || { echo "usage: scan_brief.sh BRIEF" >&2; exit 2; }
case "$1" in
  *.rs) echo "scan_brief.sh: a brief is text, not Rust: $1" >&2; exit 2 ;;
esac
here=$(cd "$(dirname "$0")" && pwd)
brief=$1
if [ "$brief" = - ]; then
  # Standard input, saved for the scan to read as a file.
  brief=$(mktemp)
  trap 'rm -f -- "$brief"' EXIT
  cat >"$brief"
fi
bash "$here/scan.sh" "$brief"
