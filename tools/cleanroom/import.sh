#!/usr/bin/env bash
# import.sh DEST BRANCH: bring an implementer's commits out of the export at DEST onto BRANCH,
# which must be the branch checked out, clean, in the current worktree of this repository.
#
# The commits after the export's one commit go across as `git format-patch` patches, applied with
# `git am`, authorship and messages kept. Each must touch only the [import] paths of
# allowlist.txt (the domain's clean crates), add no symlink or submodule, and carry the trailer
# `Role: implementer`; otherwise nothing is applied. The export's crates must also pass
# code_check.sh (nothing cargo would run or read at build time, and no variant import), add no
# scan.sh hit to a file that didn't hold it at the export's start, and add no function the
# mirror scan maps onto one routine (against the spec and decode notes on BRANCH), or nothing is
# applied. If BRANCH lacks a clean crate the export started with, its empty scaffold is committed
# first, so the patches apply as they were made. The workspace takes
# every folder of crates/clean, so that commit also brings Cargo.lock up to the new crate. Wiring
# the model into the engine is the integrator's next commit.
#
# Once applied, the export's crates must hold the capability rule (no files, network, processes,
# environment or unsafe code): Clippy with the repository's crates/clean/.clippy.toml and only the workspace's
# forbidden lints, which runs none of the code. A finding resets BRANCH to where it was: nothing
# applied. Then Clippy checks every clean crate in the workspace. An export holds one
# domain's model and the shared types, so a change to the types can break a crate its implementer
# never saw. If this run fails, the import fails naming the run and the crates, with the patches
# left applied: the root commits a mechanical fix itself, or resets and sends the change back
# (PROCESS, the import step). Override: CARGO (the cargo to run).
set -euo pipefail
. "$(dirname "$0")/lib.sh"

[ $# -eq 2 ] || die "usage: import.sh DEST BRANCH"
dest=$1 branch=$2
[ "$(git branch --show-current)" = "$branch" ] || die "check out $branch here first"
[ -z "$(git status --porcelain --untracked-files=no)" ] || die "the worktree has uncommitted changes"
# The export's history is read from a mirror of its objects and refs: the session could write its
# .git, and its configuration, attributes or hooks must not run anything here.
scratch=$(mktemp -d)
checked=$scratch/checked
mkdir "$checked"
trap 'rm -rf "$scratch"' EXIT
export_mirror "$dest" "$scratch/export.git"

# The domain is the one model crate the export started with.
root=$(git_isolated "$scratch/export.git" rev-list --max-parents=0 HEAD)
mapfile -t models < <(git_isolated "$scratch/export.git" ls-tree -r --name-only "$root" crates/clean/ | model_dirs)
[ ${#models[@]} -eq 1 ] || die "the export holds ${#models[@]} model crates, not one"
model=${models[0]}
domain=$(domain_of "${model#crates/clean/}")
echo "import: $domain from $dest onto $branch"
assert_isolated "$dest" "$domain" 1

mapfile -t writable < <(allowlist_section import "$domain")
mapfile -t commits < <(git_isolated "$scratch/export.git" rev-list --reverse "$root..HEAD")
[ ${#commits[@]} -gt 0 ] || die "no commits after the export's"

refused=0
for c in "${commits[@]}"; do
  subject=$(git_isolated "$scratch/export.git" log -1 --format=%s "$c")
  # --raw shows modes; -M off so a rename's source path is listed as a deletion.
  while read -r _ new_mode _ _ _ path; do
    in_entries "$path" "${writable[@]}" || { echo "refused ${c:0:9} $subject: touches $path"; refused=1; }
    case $new_mode in 120000|160000)
      echo "refused ${c:0:9} $subject: $path is a symlink or submodule"; refused=1 ;;
    esac
  done < <(git_isolated "$scratch/export.git" diff-tree -r --no-commit-id --no-renames --raw "$c" | sed 's/^://; s/\t/ /')
  role=$(git_isolated "$scratch/export.git" log -1 --format='%(trailers:key=Role,valueonly)' "$c" | tr -d '[:space:]')
  [ "$role" = implementer ] || { echo "refused ${c:0:9} $subject: no 'Role: implementer' trailer"; refused=1; }
done
[ "$refused" -eq 0 ] || die "nothing applied"

# What cargo would run or read while it builds the patches' code (a build script, a proc macro,
# an include! of another file) is checked before anything is applied: cargo runs it with this
# user's full access, which launch.sh cannot confine. The check runs on the committed tree of the
# export's HEAD, which is what the patches add up to.
git_isolated "$scratch/export.git" archive --format=tar HEAD -- crates | tar -x -m -C "$checked"
git show HEAD:Cargo.toml >"$checked/Cargo.toml"
bash "$cleanroom_dir/code_check.sh" "$checked" || die "code_check.sh refused the export's crates; nothing applied"

# The clean room's scan (scan.sh's checks 1-3: the original's addresses, names and shapes) on what
# the implementer changed: each file the commits add or change, at the export's HEAD, against the
# same file at the export's start. A finding counts as new when that file holds it, by class and
# detail, more often than it did at the start, so the hits already on the trunk (a clean type
# documented in the player's words that the lexicon also holds) don't block, and one the patches
# add anywhere does. Line numbers are left out of the comparison: an edit above a hit moves it.
# A file moved or renamed has no start, so each of its hits is new. A scan that reports a finding
# this can't read refuses the import rather than passing it. A hit that is a legitimate word goes
# back to the implementer, or into scan_allow.txt as a reviewed entry before the import runs again.
mapfile -t changed < <(git_isolated "$scratch/export.git" diff --name-only --no-renames --diff-filter=d "$root" HEAD -- crates/clean)
existed=()
[ ${#changed[@]} -eq 0 ] || mapfile -t existed < <(git_isolated "$scratch/export.git" ls-tree -r --name-only "$root" -- "${changed[@]}")
started=$scratch/started hits=$scratch/hits
mkdir "$started" "$hits"
[ ${#existed[@]} -eq 0 ] || git_isolated "$scratch/export.git" archive --format=tar "$root" -- "${existed[@]}" | tar -x -m -C "$started"
scan_hits() { # DIR FILE...: the scan's findings in DIR's FILEs, as `file: class: detail`
  local dir=$1 out status=0 line parsed=0
  shift
  [ $# -gt 0 ] || return 0
  out=$(bash "$cleanroom_dir/scan.sh" "${@/#/$dir/}") || status=$?
  [ "$status" -le 1 ] || die "scan.sh failed; nothing applied"
  while IFS= read -r line; do
    case $line in "$dir/"*:[0-9]*:\ *:\ *)
      line=${line#"$dir/"}; echo "${line%%:*}: ${line#*:*: }"; parsed=$((parsed + 1)) ;;
    esac
  done <<<"$out"
  # Exit 1 is a finding: one this can't read would leave both sides empty and pass the import.
  [ "$status" -eq 0 ] || [ "$parsed" -gt 0 ] || die "scan.sh found something in a form import.sh can't read; nothing applied"
}
scan_hits "$started" "${existed[@]}" >"$hits/start"
scan_hits "$checked" "${changed[@]}" >"$hits/head"
added=$(LC_ALL=C comm -13 <(LC_ALL=C sort "$hits/start") <(LC_ALL=C sort "$hits/head"))
if [ -n "$added" ]; then
  sed 's/^/scan: /' <<<"$added"
  die "scan.sh finds $(wc -l <<<"$added") hits the export's start doesn't hold; nothing applied"
fi
echo "import: scan.sh finds nothing new in ${#changed[@]} changed files"

# The mirror scan (scan.sh --mirror, the clean room's check 4) on the model crate at the export's
# start and at its HEAD, against the spec and decode notes BRANCH holds: a non-private function
# that opens with an entry whose source is one routine, and that the start didn't have (by file
# and detail), refuses the import. The spec's own share of single-routine entries is the
# observer's, so only these findings are compared. A report whose count of them disagrees with
# the findings this reads refuses the import rather than passing it.
# Both are read from BRANCH's commit, not the worktree, where an untracked file would stand in.
spec=$scratch/spec.md decode=$scratch/decode.md
git show "HEAD:docs/behaviours/$domain.md" >"$spec" 2>/dev/null ||
  die "no spec docs/behaviours/$domain.md on $branch for the mirror scan; nothing applied"
git show "HEAD:docs/behaviours/$domain.decode.md" >"$decode" 2>/dev/null || decode=-
started_model=$scratch/started_model
mkdir "$started_model"
git_isolated "$scratch/export.git" archive --format=tar "$root" -- "$model" | tar -x -m -C "$started_model"
mirror_hits() { # DIR: the mirror scan's entry-point findings in DIR's model crate, as `file: detail`
  local dir=$1 out status=0 line parsed=0 counted
  [ -d "$dir/$model" ] || return 0
  out=$(bash "$cleanroom_dir/scan.sh" --mirror "$spec" "$decode" "$dir/$model") || status=$?
  [ "$status" -le 1 ] || die "scan.sh --mirror failed; nothing applied"
  while IFS= read -r line; do
    case $line in "$dir/"*:[0-9]*:\ entry-point:\ *)
      line=${line#"$dir/"}; echo "${line%%:*}: ${line#*: entry-point: }"; parsed=$((parsed + 1)) ;;
    esac
  done <<<"$out"
  counted=$(sed -n 's/^mirror: .*; \([0-9]*\) of [0-9]* tagged non-private functions map onto one routine$/\1/p' <<<"$out")
  [ "$counted" = "$parsed" ] || die "scan.sh --mirror reported in a form import.sh can't read; nothing applied"
}
mirror_hits "$started_model" >"$hits/mirror-start"
mirror_hits "$checked" >"$hits/mirror-head"
added=$(LC_ALL=C comm -13 <(LC_ALL=C sort "$hits/mirror-start") <(LC_ALL=C sort "$hits/mirror-head"))
if [ -n "$added" ]; then
  sed 's/^/mirror: /' <<<"$added"
  die "the mirror scan finds $(wc -l <<<"$added") functions the export's start doesn't hold that open with a single-routine entry; nothing applied"
fi
echo "import: the mirror scan finds no new function that opens with a single-routine entry"

cargo=${CARGO:-cargo}
before=$(git rev-parse HEAD)
missing=()
for dir in crates/clean/types "$model"; do
  [ -n "$(git ls-tree --name-only HEAD -- "$dir")" ] || missing+=("$dir")
done
if [ ${#missing[@]} -gt 0 ]; then
  git_isolated "$scratch/export.git" archive --format=tar "$root" -- "${missing[@]}" | tar -x -m
  # A new crate is a workspace member at once: the lock takes it, offline, as the clippy runs
  # below need it locked.
  "$cargo" metadata -q --offline --format-version 1 >/dev/null || die "cargo can't lock the new crates"
  git add -- "${missing[@]}" Cargo.lock
  git commit -q -m "Clean room: the empty clean crates for $domain, as exported [exact]"
  echo "import: scaffold committed for ${missing[*]}"
fi
scaffolded=$(git rev-parse HEAD)

includes=()
for w in "${writable[@]}"; do includes+=("--include=${w}*"); done
if ! git_isolated "$scratch/export.git" format-patch --no-textconv --no-ext-diff --stdout "$root..HEAD" | git am -q --3way "${includes[@]}"; then
  git am --abort 2>/dev/null || true
  die "a patch didn't apply; $branch is at $(git rev-parse --short HEAD), the scaffold (if any) kept"
fi
# Belt and braces: what the patches landed is inside the writable paths.
outside=$(git diff --name-only "$scaffolded" HEAD | while IFS= read -r p; do in_entries "$p" "${writable[@]}" || echo "$p"; done)
[ -z "$outside" ] || die "applied changes outside the clean crates: $(echo $outside); inspect $scaffolded..HEAD"
applied="$(git rev-parse --short "$before")..$(git rev-parse --short HEAD)"

# The capability rule, on the export's own crates, before any other run: every lint allowed but
# those the workspace forbids (lib.sh). Clippy runs none of the code. A finding, or code that
# doesn't compile, takes the branch back to where it was, as code_check.sh's refusal does.
log=$(mktemp)
if ! CLIPPY_CONF_DIR="$(git rev-parse --show-toplevel)/crates/clean" "$cargo" clippy --locked -p types -p "$(model_package "$domain")" \
    --all-targets --no-deps -- "${capability_lints[@]}" >"$log" 2>&1; then
  awk '/^error|-->/ && shown++ < 20' "$log"
  rm -f "$log"
  git reset -q --hard "$before"
  die "the capability rule refused the export's crates; nothing applied"
fi

# The clean crates, by package name: the workspace takes every folder of crates/clean.
packages=()
while IFS= read -r dir; do
  name=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$dir/Cargo.toml" | head -1)
  [ -n "$name" ] || die "no package name in $dir/Cargo.toml"
  packages+=("$name")
done < <(for dir in crates/clean/*/; do [ -f "$dir/Cargo.toml" ] && echo "${dir%/}"; done)
[ ${#packages[@]} -gt 0 ] || die "the workspace holds no clean crate"
selected=()
for p in "${packages[@]}"; do selected+=(-p "$p"); done

if env -u CLIPPY_CONF_DIR "$cargo" clippy --locked --keep-going "${selected[@]}" --all-targets --no-deps -- -D warnings >"$log" 2>&1; then
  echo "import: clean-room clippy clean: ${packages[*]}"
else
  failed=$(sed -n 's/^error: could not compile `\([^`]*\)`.*$/\1/p' "$log" | sort -u | tr '\n' ' ')
  echo "IMPORT CLIPPY FAIL (clean-room): ${failed:-no crate named}"
  awk '/^(error|warning)|-->/ && shown++ < 20' "$log"
  rm -f "$log"
  die "clippy fails after the import; the patches are applied ($applied): commit a mechanical fix as the root, or git reset --hard $(git rev-parse --short "$before") and send the change back"
fi
rm -f "$log"
echo "IMPORT OK ${#commits[@]} commits onto $branch, $applied"
