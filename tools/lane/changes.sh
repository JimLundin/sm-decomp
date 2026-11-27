# Whether any path changed on the branch or locally, including staged and untracked files.
# A Git error counts as changed so a check never skips work on an unreadable tree.
changed_since() {
  local base=$1 status
  shift
  git diff --quiet --no-renames "$base...HEAD" -- "$@" || return 0
  status=$(git status --porcelain --no-renames --untracked-files=all -- "$@") || return 0
  [[ -n "$status" ]]
}
