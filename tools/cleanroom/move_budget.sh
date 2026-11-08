#!/usr/bin/env bash
# move_budget.sh SPEC: moves a behaviour spec's size paragraphs out of its header into a
# `## Budget` section at the end of its decode notes (MODEL §1 decision 10: the implementer is
# not told the score or the budget, which are for evaluation only).
#
# SPEC is docs/behaviours/<domain>.md; the notes are docs/behaviours/<domain>.decode.md beside
# it. A paragraph of the header moves, unchanged, when it starts with one of:
#   `Size is counted in logical tokens`, `**Budget`, `**Baseline`, `Budget:`, `Area `, `- Area `
# (the counting rule, the budget and its k, the old-code baseline, the tool's budget line and the
# areas with their `old` figures). The header runs from `## Header` to the first `---` or `## `
# heading after it. Refuses a spec with none of them and notes that already hold `## Budget`.
#
# Then score_scan.sh checks the spec: a mention of the score or the budget left anywhere in it
# (outside the moved paragraphs) is printed and the exit status is 1, so it is edited by hand.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
die() { echo "move_budget: $*" >&2; exit 2; }

[ $# -eq 1 ] || die "usage: move_budget.sh docs/behaviours/<domain>.md"
spec=$1
case $spec in
  *.decode.md|*.gaps.md) die "$spec is not a spec" ;;
  *.md) ;;
  *) die "$spec is not a Markdown spec" ;;
esac
notes=${spec%.md}.decode.md
[ -f "$spec" ] || die "no spec $spec"
[ -f "$notes" ] || die "no decode notes $notes"
! grep -q '^## Budget[[:space:]]*$' "$notes" || die "$notes already has a Budget section"

tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT

# Split the spec: what stays to $tmp/spec, the moved paragraphs to $tmp/moved, a blank line
# between them. Runs of blank lines left behind in the header collapse to one.
awk -v kept="$tmp/spec" -v moved="$tmp/moved" '
  function moves(line) {
    return line ~ /^(Size is counted in logical tokens|\*\*Budget|\*\*Baseline|Budget:|Area |- Area )/
  }
  function flush(   i) {
    if (n == 0) return
    if (moves(para[1])) {
      if (count++ > 0) print "" > moved
      for (i = 1; i <= n; i++) print para[i] > moved
    } else {
      for (i = 1; i <= n; i++) emit(para[i])
    }
    n = 0
  }
  # Inside the header, a blank line after a blank line is dropped; elsewhere every line stays.
  function emit(line) {
    if (header && line == "" && blank) return
    blank = (line == "")
    print line > kept
  }
  {
    sub(/\r$/, "")
    if (!header && $0 ~ /^## Header[[:space:]]*$/) { header = 1; emit($0); next }
    if (header && ($0 == "---" || $0 ~ /^## /)) { flush(); header = 0; done = 1 }
    if (header) {
      if ($0 == "") { flush(); emit("") } else { para[++n] = $0 }
      next
    }
    emit($0)
  }
  END { flush(); if (!done && !header) exit 3; exit(count == 0 ? 4 : 0) }
' "$spec" || case $? in
  3) die "$spec has no ## Header" ;;
  4) die "$spec has no size paragraph in its header" ;;
  *) die "splitting $spec failed" ;;
esac

{
  # One blank line between the notes' last line and the section.
  awk '{ sub(/\r$/, ""); lines[NR] = $0 } END { last = NR; while (last > 0 && lines[last] == "") last--; for (i = 1; i <= last; i++) print lines[i] }' "$notes"
  printf '\n## Budget\n\n'
  printf '%s\n' "Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or" \
    "the budget. \`tools spec-units\` reads k and the areas here, and \`tools report\` the areas' old" \
    "figures and the commit."
  printf '\n'
  cat "$tmp/moved"
} >"$tmp/notes"

cp "$tmp/spec" "$spec"
cp "$tmp/notes" "$notes"
echo "move_budget: moved $(grep -c . "$tmp/moved") lines from $spec to the Budget section of $notes"
if ! bash "$here/score_scan.sh" "$spec"; then
  echo "move_budget: $spec still mentions the score or the budget; edit those lines by hand" >&2
  exit 1
fi
