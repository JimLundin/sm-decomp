#!/usr/bin/env bash
# scan.sh PATH...: the clean room's checks 1-3 (addresses, names, Rust shapes) on files,
# directories or `-` (standard input), against the hashed lexicon in tools/cleanroom/scan_lexicon
# and the player vocabulary in tools/cleanroom/scan_allow.txt. Prints one line per finding and the
# counts per class; exits 1 on any finding.
#
# scan.sh --mirror SPEC DECODE MODEL...: check 4, routine mirroring (SPEC's entries through the
# decode notes DECODE, or `-`; MODEL's non-private functions, `pub` and `pub(crate)` alike).
# Fails above the single-routine limit or on any such function that maps onto one routine.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
if [ "${1:-}" = "--mirror" ]; then
  shift
  exec bash "$here/scan_tool.sh" mirror "$@"
fi
exec bash "$here/scan_tool.sh" scan "$@"
