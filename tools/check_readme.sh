#!/usr/bin/env bash
# check_readme.sh: README.md is what `tools report --write` generates for this tree
# (verify::report), and the report measured beside it still generates.
# A stale README fails with the one command that regenerates it. On Windows, where Defender
# blocks the tools executable, the same check is verify's lib test `committed_readme_is_current`,
# which tools/lane/gate.sh runs when the README's inputs change. Override: CARGO (the cargo to run).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    exec bash tools/lane/run_tests.sh --suite lib verify -- report::tests::committed_readme_is_current --exact
    ;;
  *) exec "${CARGO:-cargo}" run --locked -q -p tools --bin tools -- report --check ;;
esac
