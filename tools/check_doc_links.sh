#!/usr/bin/env sh
# check_doc_links.sh [file.md ...]: every relative Markdown link in the given files (default:
# every tracked .md file) names a file that exists and, with #anchor, a heading in that file.
# Exit 1 lists each broken link as file: target.
slug() { tr 'A-Z' 'a-z' | sed -e 's/[^a-z0-9 _-]//g' -e 's/ /-/g'; }
[ $# -gt 0 ] || set -- $(git ls-files '*.md')
bad=0
for f in "$@"; do
  dir=$(dirname "$f")
  for link in $(grep -o '\](\([^)# ]*\)\(#[^) ]*\)\?)' "$f" | sed 's/^](\(.*\))$/\1/'); do
    case $link in http*|mailto:*) continue ;; esac
    path=${link%%#*}; anchor=; case $link in *'#'*) anchor=${link#*#} ;; esac
    target=$f; [ -n "$path" ] && target=$dir/$path
    if [ ! -e "$target" ]; then echo "$f: $link"; bad=1; continue; fi
    [ -n "$anchor" ] && [ -f "$target" ] || continue
    sed -n 's/^#\{1,6\} \(.*\)$/\1/p' "$target" | slug | grep -qx -- "$anchor" \
      || { echo "$f: $link"; bad=1; }
  done
done
exit $bad
