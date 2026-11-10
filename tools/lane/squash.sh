#!/usr/bin/env bash
# squash.sh SUBJECT LANE: land a round as one commit. In the landing worktree, with HEAD at the
# base the round lands on (origin/main), make one commit whose tree is LANE's and whose parent is
# HEAD, and fast-forward the checked-out branch to it. LANE must already hold HEAD (rebase it
# first), so the squashed tree is exactly the tree the lane's gate passed on.
#
# The message is SUBJECT, then each of LANE's commits since HEAD, oldest first, under its role
# from its `Role:` line: spec (observer), gate (adversary), model (implementer), integration
# (integrator), engineering (engineer), root, and other for a commit with none. Its trailers are
# every Role and every Co-Authored-By line the commits carry, each once, so the clean room's
# provenance stays in main's history. import.sh checks each implementer commit on the lane
# branch before this runs; the lane branch keeps every original commit.
# Prints "SQUASHED <hash> <n> commits from <lane>"; exit 1 on a refusal, nothing changed.
set -euo pipefail

die() { echo "squash: $*" >&2; exit 1; }
[ $# -eq 2 ] || die "usage: squash.sh SUBJECT LANE"
subject=$1 lane=$2

case $subject in
  *'[exact]' | *'(behaviour: '*')') ;;
  *) die "the subject ends with [exact] or (behaviour: <what>)" ;;
esac
[ -z "$(git status --porcelain --untracked-files=no)" ] || die "the worktree has uncommitted changes"
if git status | grep -qE "rebase in progress|cherry-picking|You have unmerged"; then
  die "finish or abort the rebase or cherry-pick first"
fi
tip=$(git rev-parse --verify -q "$lane^{commit}") || die "no commit $lane"
base=$(git rev-parse HEAD)
git merge-base --is-ancestor "$base" "$tip" || die "$lane doesn't hold HEAD; rebase it onto ${base:0:9} first"
[ -z "$(git rev-list --merges "$base..$tip")" ] || die "$lane has merge commits since HEAD"
mapfile -t commits < <(git rev-list --reverse "$base..$tip")
[ ${#commits[@]} -gt 0 ] || die "$lane has no commits since HEAD"

# The roles in the order a round runs; any other role follows, in the order first seen.
order=(observer adversary implementer integrator engineer root none)
heading() {
  case $1 in
    observer) echo "Spec (observer)" ;;
    adversary) echo "Gate (adversary)" ;;
    implementer) echo "Model (implementer)" ;;
    integrator) echo "Integration (integrator)" ;;
    engineer) echo "Engineering (engineer)" ;;
    root) echo "Root" ;;
    none) echo "Other (no Role line)" ;;
    *) echo "$1" ;;
  esac
}
# Whether the line $1 is one of the lines that follow it.
listed() {
  local line=$1 seen
  shift
  for seen in "$@"; do [ "$seen" = "$line" ] && return 0; done
  return 1
}

declare -A parts=()
roles=() trailers=()
for c in "${commits[@]}"; do
  # A commit made with `-m "Role: x" -m "Co-Authored-By: ..."` holds its role in a paragraph of
  # its own, which git's trailer parser doesn't see, so both are read as lines of the message.
  body=$(git log -1 --format=%B "$c")
  role=$(printf '%s\n' "$body" | sed -n 's/^Role:[[:space:]]*\([^[:space:]]*\).*$/\1/p' | head -n 1)
  role=${role:-none}
  line="- ${c:0:9} $(git log -1 --format=%s "$c")"
  if [ -z "${parts[$role]+set}" ]; then
    parts[$role]=$line
    roles+=("$role")
  else
    parts[$role]+=$'\n'"$line"
  fi
  while IFS= read -r trailer; do
    listed "$trailer" "${trailers[@]}" || trailers+=("$trailer")
  done < <(printf '%s\n' "$body" | grep -iE '^(role|co-authored-by):' || true)
done

ordered=()
for role in "${order[@]}"; do
  if [ -n "${parts[$role]+set}" ]; then ordered+=("$role"); fi
done
for role in "${roles[@]}"; do
  listed "$role" "${order[@]}" || ordered+=("$role")
done

message=$subject$'\n'
for role in "${ordered[@]}"; do
  message+=$'\n'"$(heading "$role"):"$'\n'"${parts[$role]}"$'\n'
done
message+=$'\n'"Squashed from $lane at ${tip:0:9}: ${#commits[@]} commits on ${base:0:9}."$'\n\n'
# Each commit's first role in the round's order, then any other Role line, then every
# Co-Authored-By line.
written=()
for role in "${ordered[@]}"; do
  if [ "$role" != none ]; then written+=("Role: $role"); fi
done
for trailer in "${trailers[@]}"; do
  case $trailer in
    [Rr]ole:*) listed "$trailer" "${written[@]}" || written+=("$trailer") ;;
  esac
done
for trailer in "${trailers[@]}"; do
  case $trailer in [Rr]ole:*) ;; *) written+=("$trailer") ;; esac
done
for trailer in "${written[@]}"; do message+="$trailer"$'\n'; done

squashed=$(printf '%s' "$message" | git commit-tree "$tip^{tree}" -p "$base")
git merge -q --ff-only "$squashed"
echo "SQUASHED ${squashed:0:9} ${#commits[@]} commits from $lane"
