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

active() { if [ -f "$private/active.tsv" ]; then cat "$private/active.tsv"; fi; }

# A NUL anywhere marks binary data. Read the whole file before deciding: an early
# grep match can overlook a binary tail, and an unreadable file must never look like text.
text_file() {
  local nul_count
  nul_count=$(LC_ALL=C tr -cd '\000' <"$1" | wc -c) || die "cannot read $1"
  [ "$nul_count" -eq 0 ]
}

strip_token() {
  local status=0
  grep -vF "$1" "$2" || status=$?
  [ "$status" -le 1 ] || die "cannot read $2"
}

unplant() {
  active | while IFS=$'\t' read -r token kind path; do
    case $kind in
      file) [ -f "$path" ] || continue
            if ! text_file "$path"; then continue; fi
            strip_token "$token" "$path" >"$path.canary-tmp" || die "cannot unplant $path"
            mv "$path.canary-tmp" "$path" || die "cannot replace $path" ;;
      glossary) rm -f "$path" ;;
    esac
  done
  : >"$private/active.tsv"
}

record_token() { # TOKEN KIND PATH
  printf '%s\t%s\t%s\n' "$1" "$2" "$3" >>"$private/active.tsv"
  printf '%s\t%s\t%s\n' "$1" "$3" "$(date -u +%FT%TZ)" >>"$private/tokens.tsv"
}

plant_file() {
  if ! text_file "$1"; then return 0; fi
  # A file under 20 lines is a state file some tool reads (a hash, a pointer), not a source.
  local lines
  lines=$(wc -l <"$1") || die "cannot read $1"
  [ "$lines" -ge 20 ] || return 0
  token=$(new_token)
  line=$(comment_for "$1" "$token")
  awk -v c="$line" 'NR == 1 { print c } { print } NR % 400 == 0 { print c } END { print c }' \
    "$1" >"$1.canary-tmp" || die "cannot read $1"
  mv "$1.canary-tmp" "$1" || die "cannot replace $1"
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

# Planting rewrites several related records. Hold the lock only while taking a check's
# snapshot; scans of separate exports can then run together using their own immutable records.
lock_records() {
  local deadline=$((SECONDS + 5))
  mkdir -p "$private"
  while [ "$SECONDS" -lt "$deadline" ]; do
    if mkdir "$private/.record-lock" 2>/dev/null; then
      locked=1
      return 0
    fi
    sleep 0.05
  done
  die "record is busy: $private/.record-lock"
}
release_records() {
  if [ "$locked" -eq 1 ]; then
    rmdir "$private/.record-lock"
    locked=0
  fi
}
with_record_lock() (
  locked=0
  trap 'release_records' EXIT
  lock_records
  "$@"
)

# grep's status 1 means no matches; all other failures must stop the check. Keeping this
# inside each pipeline preserves pipefail for sort and the other producers as well.
matches() {
  local status=0
  grep "$@" || status=$?
  [ "$status" -le 1 ] || return "$status"
}

check() (
  [ $# -ge 1 ] || die "usage: canary.sh check TRANSCRIPT_OR_DIR..."
  work=$(mktemp -d) || die "cannot create check scratch directory"
  locked=0
  trap 'release_records; rm -rf -- "$work"' EXIT
  lock_records
  [ -s "$private/tokens.tsv" ] || die "nothing planted: no $private/tokens.tsv"
  cp "$private/tokens.tsv" "$work/tokens.tsv"
  for record in fingerprints.txt fingerprints.tsv; do
    if [ -f "$private/$record" ]; then cp "$private/$record" "$work/$record"; fi
  done
  release_records
  cut -f1 "$work/tokens.tsv" | sort -u >"$work/patterns"
  if [ -f "$work/fingerprints.txt" ]; then
    # Shared text in an export's first commit is not a leak. Tokens are never excluded.
    LC_ALL=C sort -u "$work/fingerprints.txt" >"$work/fingerprints"
    for p in "$@"; do
      if [ -e "$p/.git" ]; then
        git -C "$p" rev-list --max-parents=0 HEAD >"$work/roots"
        root=$(head -1 "$work/roots")
        [ -n "$root" ] || die "no export root in $p"
        matches_git=0
        git -C "$p" grep -hoIF -f "$work/fingerprints" "$root" -- . >"$work/given.raw" || matches_git=$?
        [ "$matches_git" -le 1 ] || die "cannot scan export root in $p"
        LC_ALL=C sort -u "$work/given.raw" >"$work/given"
        LC_ALL=C comm -23 "$work/fingerprints" "$work/given" >"$work/kept"
        mv "$work/kept" "$work/fingerprints"
      fi
    done
    cat "$work/fingerprints" >>"$work/patterns"
  fi
  hits=0
  report() {
    while IFS= read -r m; do
      src=$(awk -F '\t' -v m="$m" '$1 == m { print $2; exit }' "$work/tokens.tsv")
      if [ -n "$src" ]; then
        kind=token
      else
        kind=fingerprint
        src=unknown
        if [ -f "$work/fingerprints.tsv" ]; then
          src=$(awk -F '\t' -v m="$m" '$1 == m { print $3; exit }' "$work/fingerprints.tsv")
        fi
      fi
      echo "BREACH $kind from $src in $1: ${m:0:80}"
    done
  }
  for p in "$@"; do
    [ -e "$p" ] || die "$p does not exist"
    found=$(matches -rhoIF -f "$work/patterns" -- "$p" | sort -u)
    if [ -n "$found" ]; then hits=1; report "$p" <<<"$found"; fi
    if [ -e "$p/.git" ]; then
      # Materialize history so git failure cannot become a grep no-match verdict.
      git -C "$p" log --all -p --format=fuller >"$work/history"
      found=$(matches -hoIF -f "$work/patterns" "$work/history" | sort -u)
      if [ -n "$found" ]; then hits=1; report "$p (history)" <<<"$found"; fi
    fi
  done
  if [ "$hits" -eq 0 ]; then echo "CANARY CLEAN $*"; fi
  return "$hits"
)

case ${1:-} in
  plant) with_record_lock plant ;;
  check) shift; check "$@" ;;
  unplant) with_record_lock unplant; echo "canary: unplanted" ;;
  list) active | cut -f2,3 ;;
  *) die "usage: canary.sh plant | check PATH... | unplant | list" ;;
esac
