#!/usr/bin/env bash
# canary.sh plant | check PATH... | unplant | list: canaries prove what an implementer read
# (MODEL §5, "Canaries prove what was read").
#
# Two mechanisms, one record:
#   Tokens: a unique random token per file, planted into the forbidden sources that live outside
#     git, where planting ships nowhere: the bank-log copies under $TEMP, the reviews under
#     $TEMP/collab/reviews and, in the old engine's tree (the main checkout's sm/), an untracked
#     glossary file. A token line goes at the top of each file, every 400 lines and at the end, so a
#     whole read, a head or a paged read sees one; a file under 20 lines is a tool's state file and is
#     left alone. Each source directory also gets a glossary file
#     whose lines pair common search words with its token, so a grep across it prints one.
#   Fingerprints: the committed forbidden sources (docs/behaviours/*.decode.md and
#     crates/verify/src/{decode,source,strand}.rs, every version on every ref) get no token. A
#     token committed there would ship into every checkout, every export built from that commit
#     and every lane's history, and a token in a working copy only catches reads of that working
#     copy, not `git show` or the web. Instead their distinctive lines are the canary: each line of
#     40 or more characters that appears in no other tracked .rs, .md or .toml file. A transcript
#     that holds one quotes a forbidden file, by whichever route it was read.
#
# The record lives outside every repository, in $CLEANROOM_PRIVATE (default ~/.cleanroom):
# tokens.tsv (token, source, planted at), fingerprints.txt and fingerprints.tsv (line, blob,
# path). Old tokens stay in the record, so a transcript from an earlier lane is still checked.
# Reading the record itself trips every canary at once.
#
#   plant      strip earlier tokens, plant fresh ones, rebuild the fingerprints (run in a checkout
#              of this repository). CANARY_ALSO adds space-separated files or directories.
#   check P... search files, directories and export repositories (their whole history, as
#              patches) for any token or fingerprint. Exit 1 on a hit, 2 if nothing is planted.
#   unplant    remove the planted lines and glossary files; the record stays.
#   list       the sources currently planted.
set -euo pipefail
private=${CLEANROOM_PRIVATE:-$HOME/.cleanroom}
temp=$(cygpath -u "${TEMP:-/tmp}" 2>/dev/null || echo "${TEMP:-/tmp}")
die() { echo "canary: $*" >&2; exit 2; }

words="samus speed velocity momentum position pose jump fall gravity run walk dash morph ball
spin wall slope ground air water lava acid enemy room door collision block timer input pad frame
missile beam bomb grapple energy health damage knockback camera scroll subpixel cap accel"

comment_for() {
  case $1 in
    *.html|*.htm|*.md) echo "<!-- trace $2 -->" ;;
    *.rs) echo "// trace $2" ;;
    *) echo "; trace $2" ;;
  esac
}
new_token() { echo "cr$(od -An -tx1 -N12 /dev/urandom | tr -d ' \n')"; }

# The forbidden sources outside git that get tokens: files, and the directories that get a
# glossary file. The main checkout's sm/ gets only the glossary: its tracked files stay clean.
sources() {
  for d in "$temp"/c4a/logs* "$temp/banklogs" "$temp/collab/reviews" ${CANARY_ALSO:-}; do
    [ -e "$d" ] && echo "$d"
  done
  for f in "$temp"/bank*.txt "$temp"/bank*.html; do [ -f "$f" ] && echo "$f"; done
  return 0
}
main_checkout() {
  common=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || return 0
  echo "$(dirname "$common")"
}

active() { [ -f "$private/active.tsv" ] && cat "$private/active.tsv"; return 0; }

unplant() {
  active | while IFS=$'\t' read -r token kind path; do
    case $kind in
      file) [ -f "$path" ] || continue
            { grep -vF "$token" "$path" || true; } >"$path.canary-tmp" && mv "$path.canary-tmp" "$path" ;;
      glossary) rm -f "$path" ;;
    esac
  done
  : >"$private/active.tsv" 2>/dev/null || true
}

record_token() { # TOKEN KIND PATH
  printf '%s\t%s\t%s\n' "$1" "$2" "$3" >>"$private/active.tsv"
  printf '%s\t%s\t%s\n' "$1" "$3" "$(date -u +%FT%TZ)" >>"$private/tokens.tsv"
}

plant_file() {
  # A file under 20 lines is a state file some tool reads (a hash, a pointer), not a source.
  [ "$(wc -l <"$1")" -ge 20 ] || return 0
  token=$(new_token)
  line=$(comment_for "$1" "$token")
  awk -v c="$line" 'NR == 1 { print c } { print } NR % 400 == 0 { print c } END { print c }' \
    "$1" >"$1.canary-tmp" && mv "$1.canary-tmp" "$1"
  record_token "$token" file "$1"
}
plant_glossary() { # DIR FILENAME
  token=$(new_token)
  path=$1/$2
  for w in $words; do comment_for "$path" "$w: $token"; done >"$path"
  record_token "$token" glossary "$path"
}

fingerprints() {
  top=$(git rev-parse --show-toplevel) || die "plant runs in a checkout of the repository"
  forbidden=(':(glob)docs/behaviours/*.decode.md' crates/verify/src/decode.rs
    crates/verify/src/source.rs crates/verify/src/strand.rs)
  work=$(mktemp -d)
  # Every version of each forbidden file reachable from any ref.
  git -C "$top" log --all --no-renames --no-abbrev --format= --raw -- "${forbidden[@]}" \
    | awk '$4 != "0000000000000000000000000000000000000000" { print $4 "\t" $6 }' | sort -u >"$work/blobs"
  # Lines found anywhere else in the tracked sources are not distinctive.
  git -C "$top" grep -h -I -e '' HEAD -- '*.rs' '*.md' '*.toml' \
    ':(exclude,glob)docs/behaviours/*.decode.md' ':(exclude)crates/verify/src/decode.rs' \
    ':(exclude)crates/verify/src/source.rs' ':(exclude)crates/verify/src/strand.rs' \
    | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | LC_ALL=C sort -u >"$work/common"
  # Distinctive lines: 40+ characters, printable ASCII without quotes or backslashes, so they
  # read the same raw and inside a JSON transcript, and 12+ letters or digits: a table rule such
  # as "| --- | --- |" is in every Markdown file and a substring of any wider one.
  : >"$work/map"
  while IFS=$'\t' read -r blob path; do
    git -C "$top" cat-file blob "$blob" | tr -d '\r' \
      | LC_ALL=C awk -v b="$blob" -v p="$path" '{ sub(/^[ \t]+/, ""); sub(/[ \t]+$/, "") }
          length($0) >= 40 && $0 !~ /[^ -~]/ && $0 !~ /["\\]/ {
            words = $0; if (gsub(/[A-Za-z0-9]/, "", words) >= 12) print $0 "\t" b "\t" p }' >>"$work/map"
  done <"$work/blobs"
  LC_ALL=C sort -u -t$'\t' -k1,1 "$work/map" >"$work/map.sorted"
  LC_ALL=C join -t$'\t' -v1 "$work/map.sorted" "$work/common" >"$private/fingerprints.tsv"
  cut -f1 "$private/fingerprints.tsv" >"$private/fingerprints.txt"
  echo "canary: $(wc -l <"$private/fingerprints.txt") fingerprints from $(wc -l <"$work/blobs") versions of the committed sources"
  rm -rf "$work"
}

plant() {
  mkdir -p "$private"
  if git -C "$private" rev-parse --git-dir >/dev/null 2>&1; then die "$private is inside a repository"; fi
  unplant
  sources | while IFS= read -r src; do
    if [ -d "$src" ]; then
      find "$src" -type f ! -name '*.canary-tmp' ! -name 'glossary.txt' | while IFS= read -r f; do plant_file "$f"; done
      plant_glossary "$src" glossary.txt
    else
      plant_file "$src"
    fi
  done
  main=$(main_checkout)
  if [ -n "$main" ] && [ -d "$main/sm/src" ]; then
    if git -C "$main" ls-files --error-unmatch sm/src/glossary.rs >/dev/null 2>&1; then
      die "$main/sm/src/glossary.rs is tracked; choose another canary file name"
    fi
    plant_glossary "$main/sm/src" glossary.rs
  fi
  fingerprints
  echo "canary: $(active | wc -l) sources planted; record in $private"
}

check() {
  [ $# -ge 1 ] || die "usage: canary.sh check TRANSCRIPT_OR_DIR..."
  [ -s "$private/tokens.tsv" ] || die "nothing planted: no $private/tokens.tsv"
  cut -f1 "$private/tokens.tsv" | sort -u >"$private/.patterns"
  [ -f "$private/fingerprints.txt" ] && cat "$private/fingerprints.txt" >>"$private/.patterns"
  hits=0
  report() { # LABEL: each matched string with its source
    while IFS= read -r m; do
      src=$(grep -F -- "$m" "$private/tokens.tsv" | head -1 | cut -f2 || true)
      [ -n "$src" ] && kind=token || { kind=fingerprint; src=$(grep -F -- "$m" "$private/fingerprints.tsv" | head -1 | cut -f3 || true); }
      echo "BREACH $kind from $src in $1: ${m:0:80}"
    done
  }
  for p in "$@"; do
    [ -e "$p" ] || die "$p does not exist"
    found=$(grep -rhoIF -f "$private/.patterns" -- "$p" | sort -u || true)
    [ -n "$found" ] && { hits=1; echo "$found" | report "$p"; }
    if [ -d "$p/.git" ]; then
      # The export's commits, messages and diffs included.
      found=$(git -C "$p" log --all -p --format=fuller | grep -hoIF -f "$private/.patterns" | sort -u || true)
      [ -n "$found" ] && { hits=1; echo "$found" | report "$p (history)"; }
    fi
  done
  rm -f "$private/.patterns"
  [ "$hits" -eq 0 ] && echo "CANARY CLEAN $*"
  return "$hits"
}

case ${1:-} in
  plant) plant ;;
  check) shift; check "$@" ;;
  unplant) unplant; echo "canary: unplanted" ;;
  list) active | cut -f2,3 ;;
  *) die "usage: canary.sh plant | check PATH... | unplant | list" ;;
esac
