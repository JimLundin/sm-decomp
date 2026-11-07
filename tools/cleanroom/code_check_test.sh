#!/usr/bin/env bash
# code_check_test.sh: code_check.sh passes code_check_fixtures/clean and, on every other fixture
# (named TAG__variant), fails and names TAG. Run from anywhere; exit 1 lists what went wrong.
here=$(cd "$(dirname "$0")" && pwd)
bad=0
fail() { echo "FAIL $*"; bad=1; }

out=$(bash "$here/code_check.sh" "$here/code_check_fixtures/clean" 2>&1) || fail "clean fixture refused: $out"
n=0
for dir in "$here"/code_check_fixtures/*__*/; do
  name=$(basename "$dir"); tag=${name%%__*}; n=$((n + 1))
  if out=$(bash "$here/code_check.sh" "$dir" 2>&1); then
    fail "$name passed"
  else
    grep -q ": $tag: " <<<"$out" || fail "$name did not report $tag: $out"
  fi
done
[ "$n" -gt 0 ] || fail "no fixtures"
[ "$bad" -eq 0 ] && echo "CODE CHECK FIXTURES OK: clean passes, $n forbidden fixtures each named"
exit $bad
