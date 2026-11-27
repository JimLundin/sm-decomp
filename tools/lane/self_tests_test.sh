#!/usr/bin/env bash
# Exercise the real selector against a Git fixture. Each suite records its invocation.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
repo=$scratch/repo
mkdir -p "$repo/tools/lane" "$repo/tools/cleanroom"
cp "$here"/{changes,self_tests}.sh "$repo/tools/lane/"
cd "$repo"
git init -q
git config user.name Fixture
git config user.email fixture@example.invalid
git config commit.gpgsign false
git config status.showUntrackedFiles no

suites=(tools/cleanroom/{launch,launch_config,code_check,canary,blind_copy,adjusted_vectors,import_check}_test.sh
  tools/lane/{worktree,gate,self_tests}_test.sh)
for suite in "${suites[@]}"; do
  printf '#!/bin/sh\ntouch "$RAN/%s"\n' "$(basename "$suite" _test.sh)" >"$suite"
done
for path in tools/lane/gate.sh tools/lane/land.py tools/cleanroom/{import,lib}.sh; do touch "$path"; done
git add -A
git commit -qm base
base=$(git rev-parse HEAD)

run() {
  RAN=$(mktemp -d "$scratch/ran-XXXXXX")
  export RAN
  output=$(bash tools/lane/self_tests.sh "$@")
}
expect() {
  local want actual
  want=$(printf '%s\n' "$@" | LC_ALL=C sort)
  actual=$(find "$RAN" -type f -printf '%f\n' | LC_ALL=C sort)
  [[ $actual = "$want" ]] || { echo "expected [$want], got [$actual]: $output" >&2; exit 1; }
}
all=(launch launch_config code_check canary blind_copy adjusted_vectors import_check worktree gate self_tests)

run HEAD; expect
echo edited >>tools/lane/gate.sh
run HEAD; expect gate
git add tools/lane/gate.sh
run HEAD; expect gate
git show HEAD:tools/lane/gate.sh >tools/lane/gate.sh
run HEAD; expect gate   # Staged and unstaged edits cancel in a diff against HEAD.
git restore tools/lane/gate.sh
git commit -qm 'Edit the gate'
run "$base"; expect gate

echo edited >>tools/lane/land.py
run HEAD; expect
echo "land.py: $output"
git restore tools/lane/land.py

echo edited >>tools/cleanroom/import.sh
run HEAD; expect import_check
echo "import.sh: $output"
run "$base"; expect gate import_check
git restore tools/cleanroom/import.sh
git mv tools/lane/gate.sh tools/lane/renamed
run HEAD; expect "${all[@]}"   # An unknown lane file falls back to every suite.
git restore --staged --worktree :/

mkdir -p crates/clean/types/src
touch crates/clean/types/src/untracked.rs
run HEAD; expect import_check
unlink crates/clean/types/src/untracked.rs

# A change to any shared import helper or build input keeps the import test.
for path in tools/cleanroom/{lib.sh,export.sh,export_workspace.py,code_check.sh,check_manifest.py,allowlist.txt,shape_scan.awk,scan_tool.sh} crates/clean/.clippy.toml \
    crates/tools/src/bin/cleanroom/scan.rs crates/clean/world_blocks/src/model.rs rust-toolchain.toml Cargo.toml Cargo.lock crates/clean/demo/Cargo.toml; do
  mkdir -p "$(dirname "$path")"
  echo change >>"$path"
  run HEAD
  [[ -f $RAN/import_check ]] || { echo "$path skipped import_check: $output"; exit 1; }
  if git ls-files --error-unmatch "$path" >/dev/null 2>&1; then git restore -- "$path"; else unlink "$path"; fi
done

git mv tools/lane/gate.sh removed-gate.sh
run HEAD; expect gate  # Leaving a subject directory still selects the old path.
run; expect "${all[@]}"
if bash tools/lane/self_tests.sh missing-branch >/dev/null 2>&1; then echo 'invalid base succeeded'; exit 1; fi
echo 'SELF-TEST SELECTION OK'
