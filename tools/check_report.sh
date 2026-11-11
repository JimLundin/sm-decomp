#!/usr/bin/env bash
# check_report.sh: docs/REPORT.md, docs/timeline.svg and README.md's status between its markers are
# what `tools report` generates for this tree (verify::report).
# A stale page fails with the one command that regenerates them all. On Windows, where Defender
# blocks the tools executable, the same check runs as verify's lib test through
# tools/lane/run_tests.sh. Override: CARGO (the cargo to run).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    exec bash tools/lane/run_tests.sh verify -- report::tests::committed_report_is_current
    ;;
  *) exec "${CARGO:-cargo}" run --locked -q -p tools --bin tools -- report --check ;;
esac
