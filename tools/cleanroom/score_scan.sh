#!/usr/bin/env bash
# score_scan.sh PATH...: refuses implementer-facing material that mentions the score, the budget,
# k, the fit, token weights or an old-code baseline (MODEL §1 decision 10), by the patterns in
# score_words.txt beside it. PATH is a file or a directory (searched whole, `.git` and `target`
# skipped). Prints `file:line: score-word: text` per finding; exits 1 on any, 2 on a bad call.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
words=${SCORE_WORDS:-$here/score_words.txt}
[ $# -ge 1 ] || { echo "usage: score_scan.sh PATH..." >&2; exit 2; }
[ -f "$words" ] || { echo "score_scan.sh: no word list at $words" >&2; exit 2; }
for path in "$@"; do
  [ -e "$path" ] || { echo "score_scan.sh: no such path $path" >&2; exit 2; }
done

patterns=$(mktemp)
trap 'rm -f -- "$patterns"' EXIT
grep -v -E '^[[:space:]]*(#|$)' "$words" >"$patterns"
[ -s "$patterns" ] || { echo "score_scan.sh: $words holds no pattern" >&2; exit 2; }

hits=$(grep -r -H -n -I -i -E -f "$patterns" --exclude-dir=.git --exclude-dir=target -- "$@" || true)
if [ -n "$hits" ]; then
  # file:line:text -> file:line: score-word: text
  sed -E 's/^(([A-Za-z]:)?[^:]*:[0-9]+):/\1: score-word: /' <<<"$hits"
  echo "score scan: $(wc -l <<<"$hits") lines mention the score or the budget" >&2
  exit 1
fi
