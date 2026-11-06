#!/usr/bin/env bash
# pick.sh <commit>...: cherry-pick lane commits onto the current branch (the landing branch), in order.
# Stops at the first conflict and leaves it for you; prints "picked", "CONFLICT <commit>: <files>".
set -u
if git status | grep -qE "rebase in progress|cherry-picking|You have unmerged"; then echo "BUSY: finish or abort first"; exit 1; fi
for c in "$@"; do
  if git cherry-pick "$c" >/dev/null 2>&1; then echo "picked $c"; continue; fi
  files=$(git diff --name-only --diff-filter=U | tr '\n' ' ')
  echo "CONFLICT $c: $files"; exit 2
done
