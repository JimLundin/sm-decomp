#!/usr/bin/env bash
# Lane worktrees that start from a warm build, and their cleanup.
#   worktree.sh add <branch> <path> <base>  a worktree on a new branch, its target/ seeded
#   worktree.sh remove <path>               the worktree, its target/ and its test-DLL folder
#   worktree.sh seed                        builds the one seed at origin/main (the root, off-hours)
# The seed is one worktree at $SM_SEED_DIR (default $TEMP/sm-seed), built the way lanes build:
# the gate, and the game and verify tests at opt-level 3. A new lane takes its build state, so its
# first build compiles only what differs from the seed.
#
# The build state is hard-linked, so a lane costs no disk until it rebuilds something. Cargo and
# rustc replace an artifact with a new file, which leaves the seed's untouched, but they rewrite
# dep-info files (`*.d` and the fingerprints) in place, so the lane gets its own copies of those.
# Where links fail (another volume), or with SM_SEED_COPY=1, everything is copied.
#
# Cargo judges a workspace crate fresh when its outputs are newer than its sources. The lane's
# tracked files are dated 2000-01-01, older than any build, and then the files that differ
# between the seed's commit and the lane's base are touched, so only the crates they feed
# rebuild. A target directory shared by two worktrees would be wrong: built under the same flags,
# artifacts have the same names, and each worktree's own source times would let one lane run
# what the other built.
# Overrides: SM_SEED_DIR, SM_SEED_COPY, SM_TEST_DLL_DIR.
set -euo pipefail
seed=${SM_SEED_DIR:-$TEMP/sm-seed}
# The build state worth carrying: clippy's metadata, the clean-room clippy pass's, and the test
# builds of tools/lane/test_dll.py. Release and verify-profile builds are a lane's own.
parts=(debug clean-clippy x86_64-pc-windows-gnu)

# The test-DLL folder run_tests.sh uses for worktree $1.
dll_dir() { echo "${SM_TEST_DLL_DIR:-$TEMP/sm-test-dll/$(basename "$1")-$(printf %s "$1" | cksum | cut -d' ' -f1)}"; }

# Links seed part $1 into target directory $2, with private copies of the files rewritten in
# place; fails, leaving nothing, where the file system can't link.
link_part() {
  cp -al "$seed/target/$1" "$2/" 2>/dev/null || { rm -rf "${2:?}/$1"; return 1; }
  # Extracting replaces each linked file with a new one, so the seed's stays as it was.
  (cd "$seed/target" && find "$1" \( -path '*/.fingerprint/*' -o -name '*.d' \) -type f -print0) \
    | tar -C "$seed/target" --null -T - -cf - | tar -C "$2" -xf -
}

case "${1:-}" in
  add)
    [ "$#" -eq 4 ] || { echo "usage: worktree.sh add <branch> <path> <base>" >&2; exit 2; }
    branch=$2 path=$3 base=$4
    git worktree add -q -b "$branch" "$path" "$base"
    if ! from=$(git -C "$seed" rev-parse -q --verify HEAD 2>/dev/null) || [ ! -f "$seed/target/.seeded" ]; then
      echo "worktree: no seed at $seed; the first build is cold"
      exit 0
    fi
    mkdir -p "$path/target"
    how=linked
    for part in "${parts[@]}"; do
      [ -d "$seed/target/$part" ] || continue
      if [ -n "${SM_SEED_COPY:-}" ] || ! link_part "$part" "$path/target"; then
        cp -a "$seed/target/$part" "$path/target/"
        how=copied
      fi
    done
    git -C "$path" ls-files -z | (cd "$path" && xargs -0r touch -c -h -d '2000-01-01 00:00:00' --)
    git -C "$path" diff --name-only -z "$from" HEAD -- | (cd "$path" && xargs -0r touch -c --)
    echo "worktree: $how the build of seed $(git -C "$seed" rev-parse --short HEAD)"
    ;;
  remove)
    [ "$#" -eq 2 ] || { echo "usage: worktree.sh remove <path>" >&2; exit 2; }
    # The path as run_tests.sh spells it, which names the test-DLL folder.
    path=$(cd "$2" && cd "$(git rev-parse --show-toplevel)" && pwd)
    rm -rf "$(dll_dir "$path")"
    rm -rf "$path/target"
    git worktree remove --force "$path"
    git worktree prune
    ;;
  seed)
    [ "$#" -eq 1 ] || { echo "usage: worktree.sh seed" >&2; exit 2; }
    git fetch -q origin
    if [ -d "$seed" ]; then
      git -C "$seed" checkout -q --detach origin/main
    else
      git worktree add -q --detach "$seed" origin/main
    fi
    rm -f "$seed/target/.seeded"
    (cd "$seed" && bash tools/lane/gate.sh && CARGO_PROFILE_TEST_OPT_LEVEL=3 bash tools/lane/run_tests.sh game verify -- --list >/dev/null)
    touch "$seed/target/.seeded"
    echo "worktree: seed at $(git -C "$seed" rev-parse --short HEAD), $(du -sh "$seed/target" | cut -f1)"
    ;;
  *) echo "usage: worktree.sh add <branch> <path> <base> | remove <path> | seed" >&2; exit 2 ;;
esac
