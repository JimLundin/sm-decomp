#!/usr/bin/env bash
# adjusted_vectors.sh [--gate] [-C ROOT] [DOMAIN...]: list each spec entry whose switch is
# Parameter and that no vector runs at a value other than the reference one (PROCESS, the spec
# gate). Reads committed files only: the spec, docs/behaviours/DOMAIN.md, where an entry is a
# `### Name` heading and its switch a `- switch: parameter` line (any case, emphasis allowed);
# and the vectors, scenarios/vectors/DOMAIN.toml, where a vector runs an entry adjusted when its
# `entries` name it and it carries a `rules` table (`rules = { ... }` or `[vector.rules]`), the
# observer's per-vector overrides of the reference rules. With no DOMAIN, every spec that has a
# vector file.
#
# Prints one line per entry without one, `adjusted: DOMAIN: ENTRY has no adjusted vector`, then a
# count per domain. Advisory, exit 0, unless --gate: then exit 1 when any entry lacks one. The
# spec gate runs it with --gate on its domain; a domain specced before the rule meets it at its
# next spec round.
set -uo pipefail
gate=0
root=.
while [ $# -gt 0 ]; do
  case $1 in
    --gate) gate=1; shift ;;
    -C) root=$2; shift 2 ;;
    *) break ;;
  esac
done
domains=("$@")
if [ ${#domains[@]} -eq 0 ]; then
  for spec in "$root"/docs/behaviours/*.md; do
    d=$(basename "$spec" .md)
    case $d in *.decode|*.gaps) continue ;; esac
    [ -f "$root/scenarios/vectors/$d.toml" ] && domains+=("$d")
  done
fi

missing=0
for d in "${domains[@]}"; do
  spec=$root/docs/behaviours/$d.md
  vectors=$root/scenarios/vectors/$d.toml
  [ -f "$spec" ] || { echo "adjusted_vectors: no spec $spec" >&2; exit 2; }
  mapfile -t params < <(awk '
    /^### / { entry = $2 }
    tolower($0) ~ /^- switch: *\**parameter/ && entry != "" { print entry; entry = "" }' "$spec")
  # The entries of every vector that carries a `rules` table, one per line.
  adjusted=$(awk '
    function flush() { if (rules) printf "%s", names; names = ""; rules = 0; inv = 0 }
    /^\[\[vector\]\]/ { flush(); inv = 1; next }
    /^\[vector\.rules\]/ { rules = 1; next }
    /^\[/ { flush(); next }
    inv && /^rules *=/ { rules = 1 }
    inv && /^entries *=/ { collecting = 1 }
    collecting {
      line = $0; sub(/^entries *= */, "", line)
      while (match(line, /"[^"]*"/)) { names = names substr(line, RSTART + 1, RLENGTH - 2) "\n"; line = substr(line, RSTART + RLENGTH) }
      if ($0 ~ /\]/) collecting = 0
    }
    END { flush() }' "$vectors" 2>/dev/null)
  without=0
  for entry in "${params[@]}"; do
    grep -qxF "$entry" <<<"$adjusted" && continue
    echo "adjusted: $d: $entry has no adjusted vector"
    without=$((without + 1))
  done
  echo "adjusted: $d: $(( ${#params[@]} - without )) of ${#params[@]} Parameter entries have an adjusted vector"
  missing=$((missing + without))
done
[ "$gate" -eq 1 ] && [ "$missing" -gt 0 ] && exit 1
exit 0
