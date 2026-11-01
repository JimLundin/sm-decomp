#!/usr/bin/env bash
# pick.sh <commit>...: cherry-pick lane commits onto the current branch (the landing branch), in order.
# Stops at the first conflict and leaves it for you; prints "picked", "CONFLICT <commit>: <files>".
# scenarios/lock.toml conflicts are resolved as a union and re-checked; every other file is yours.
set -u
PY="${SM_PY:-/c/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe}"
if git status | grep -qE "rebase in progress|cherry-picking|You have unmerged"; then echo "BUSY: finish or abort first"; exit 1; fi
for c in "$@"; do
  if git cherry-pick "$c" >/dev/null 2>&1; then echo "picked $c"; continue; fi
  files=$(git diff --name-only --diff-filter=U | tr '\n' ' ')
  if [ "$files" = "scenarios/lock.toml " ]; then
    "$PY" -I - <<'EOF'
import re
p = "scenarios/lock.toml"
s = open(p, encoding="utf-8").read()
s = re.sub(r"<<<<<<< [^\n]*\n(.*?)=======\n(.*?)>>>>>>> [^\n]*\n", lambda m: m.group(1) + m.group(2), s, flags=re.S)
open(p, "w", encoding="utf-8", newline="\n").write(s)
EOF
    if "$PY" tools/scenario.py lock --check >/dev/null 2>&1; then
      git add scenarios/lock.toml && GIT_EDITOR=true git cherry-pick --continue >/dev/null && { echo "picked $c (lock union)"; continue; }
    fi
  fi
  echo "CONFLICT $c: $files"; exit 2
done
