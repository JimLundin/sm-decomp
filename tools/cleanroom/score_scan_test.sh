#!/usr/bin/env bash
# score_scan_test.sh: the score guard (MODEL §1 decision 10) on its fixtures. score_scan.sh passes
# score_scan_fixtures/clean.md and flags every line of score_scan_fixtures/mentions.md, each
# pattern of score_words.txt matching one of them; and scan_brief.sh refuses mentions.md as a
# brief. Run from anywhere; exit 1 lists what went wrong.
here=$(cd "$(dirname "$0")" && pwd)
fixtures=$here/score_scan_fixtures
bad=0
fail() { echo "FAIL $*"; bad=1; }

out=$(bash "$here/score_scan.sh" "$fixtures/clean.md" 2>&1) || fail "clean.md refused: $out"

lines=$(grep -c . "$fixtures/mentions.md")
flagged=$(bash "$here/score_scan.sh" "$fixtures/mentions.md" 2>/dev/null | grep -c ': score-word: ')
[ "$flagged" -eq "$lines" ] || fail "mentions.md: $flagged of $lines lines flagged"
patterns=0
while IFS= read -r pattern; do
  case $pattern in ''|'#'*) continue ;; esac
  patterns=$((patterns + 1))
  grep -q -i -E -e "$pattern" "$fixtures/mentions.md" || fail "no line of mentions.md matches $pattern"
done <"$here/score_words.txt"
[ "$patterns" -gt 0 ] || fail "score_words.txt holds no pattern"

out=$(bash "$here/scan_brief.sh" "$fixtures/mentions.md" 2>&1) && fail "scan_brief.sh passed mentions.md"
grep -q ': score-word: ' <<<"$out" || fail "scan_brief.sh did not run the score guard: $out"
out=$(bash "$here/scan_brief.sh" - <"$fixtures/mentions.md" 2>&1) && fail "scan_brief.sh passed mentions.md on standard input"

[ "$bad" -eq 0 ] && echo "SCORE GUARD OK: clean passes, $lines mentions flagged by $patterns patterns"
exit $bad
