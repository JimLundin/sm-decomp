#!/usr/bin/env bash
# import.sh DEST BRANCH: bring an implementer's commits out of the export at DEST onto BRANCH,
# which must be the branch checked out, clean, in the current worktree of this repository.
#
# The commits after the export's one commit go across as `git format-patch` patches, applied with
# `git am`, authorship and messages kept. Each must touch only the [import] paths of
# allowlist.txt (the domain's clean crates), add no symlink or submodule, and carry the trailer
# `Role: implementer`; otherwise nothing is applied. The export's crates must also pass
# code_check.sh (no file, network, process or build-time reach), or nothing is applied. If BRANCH
# lacks a clean crate the export started with, its empty scaffold is committed first (trailer
# `Role: integrator`), so the patches
# apply as they were made. Wiring the crates into the workspace is the integrator's next commit.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

[ $# -eq 2 ] || die "usage: import.sh DEST BRANCH"
dest=$1 branch=$2
[ "$(git branch --show-current)" = "$branch" ] || die "check out $branch here first"
[ -z "$(git status --porcelain --untracked-files=no)" ] || die "the worktree has uncommitted changes"

# The domain is the one model crate the export started with.
root=$(git_isolated "$dest" rev-list --max-parents=0 HEAD)
mapfile -t models < <(git_isolated "$dest" ls-tree --name-only "$root" crates/clean_model/)
[ ${#models[@]} -eq 1 ] || die "the export holds ${#models[@]} model crates, not one"
domain=${models[0]#crates/clean_model/}
echo "import: $domain from $dest onto $branch"
assert_isolated "$dest" "$domain" 1

mapfile -t writable < <(allowlist_section import "$domain")
mapfile -t commits < <(git_isolated "$dest" rev-list --reverse "$root..HEAD")
[ ${#commits[@]} -gt 0 ] || die "no commits after the export's"

refused=0
for c in "${commits[@]}"; do
  subject=$(git_isolated "$dest" log -1 --format=%s "$c")
  # --raw shows modes; -M off so a rename's source path is listed as a deletion.
  while read -r _ new_mode _ _ _ path; do
    in_entries "$path" "${writable[@]}" || { echo "refused ${c:0:9} $subject: touches $path"; refused=1; }
    case $new_mode in 120000|160000)
      echo "refused ${c:0:9} $subject: $path is a symlink or submodule"; refused=1 ;;
    esac
  done < <(git_isolated "$dest" diff-tree -r --no-commit-id --no-renames --raw "$c" | sed 's/^://; s/\t/ /')
  role=$(git_isolated "$dest" log -1 --format='%(trailers:key=Role,valueonly)' "$c" | tr -d '[:space:]')
  [ "$role" = implementer ] || { echo "refused ${c:0:9} $subject: no 'Role: implementer' trailer"; refused=1; }
done
[ "$refused" -eq 0 ] || die "nothing applied"

# The code the patches leave behind is checked before anything is applied: cargo test and build
# run it with this user's full access, which launch.sh cannot confine. The check runs on the
# committed tree of the export's HEAD, which is what the patches add up to.
checked=$(mktemp -d)
trap 'rm -rf "$checked"' EXIT
git_isolated "$dest" archive --format=tar HEAD -- crates | tar -x -m -C "$checked"
bash "$cleanroom_dir/code_check.sh" "$checked" || die "code_check.sh refused the export's crates; nothing applied"

before=$(git rev-parse HEAD)
missing=()
for dir in crates/clean_types "crates/clean_model/$domain"; do
  [ -n "$(git ls-tree --name-only HEAD -- "$dir")" ] || missing+=("$dir")
done
if [ ${#missing[@]} -gt 0 ]; then
  git_isolated "$dest" archive --format=tar "$root" -- "${missing[@]}" | tar -x -m
  git add -- "${missing[@]}"
  git commit -q -m "Clean room: the empty clean crates for $domain, as exported [exact]" -m "Role: integrator"
  echo "import: scaffold committed for ${missing[*]}"
fi

includes=()
for w in "${writable[@]}"; do includes+=("--include=${w}*"); done
if ! git_isolated "$dest" format-patch --stdout "$root..HEAD" | git am -q --3way "${includes[@]}"; then
  git am --abort 2>/dev/null || true
  die "a patch didn't apply; $branch is at $(git rev-parse --short HEAD), the scaffold (if any) kept"
fi
# Belt and braces: what landed is inside the writable paths.
outside=$(git diff --name-only "$before" HEAD | while IFS= read -r p; do in_entries "$p" "${writable[@]}" || echo "$p"; done)
[ -z "$outside" ] || die "applied changes outside the clean crates: $(echo $outside); inspect $before..HEAD"
echo "IMPORT OK ${#commits[@]} commits onto $branch, $(git rev-parse --short "$before")..$(git rev-parse --short HEAD)"
