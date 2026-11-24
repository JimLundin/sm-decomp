#!/usr/bin/env bash
# verdict_test.sh: tools/ci/verdict.sh's input lists keep every known input of each job, every
# file a crate include_str!s or include_bytes!s, and a changed input changes the job's fingerprint
# while a changed doc no job reads does not. Run from the repository root.
set -u
verdict=tools/ci/verdict.sh
fail=0
no() { echo "FAIL $*"; fail=1; }

declare -A has
for job in content replay faults-0 replay@played; do
  has[$job]=$(bash "$verdict" inputs "$job") || no "$job: no input list"
done
reads() { grep -qxF "$2" <<<"${has[$1]}"; }

# Every job reads the workflows, this judge, the toolchain, the workspace, vendor/ and its crates.
common=(.github/workflows/ci.yml .github/workflows/verify.yml tools/ci/verdict.sh
  rust-toolchain.toml Cargo.toml Cargo.lock crates/data/Cargo.toml crates/import/src/lib.rs
  crates/clean/types/Cargo.toml "$(git ls-files vendor | head -1)")
# The replay and fault jobs also read the engine, the scenarios, the movies, the CI fetchers and
# the fault manifest, and the docs their tests open.
replay=(crates/game/src/lib.rs crates/verify/required-witnesses.toml scenarios/replay/index.toml
  scenarios/replay/baseline.toml scenarios/locks/lsnes.toml "$(git ls-files movies | head -1)"
  tools/ci/replay.sh tools/ci/replay_parallel.sh tools/ci/required_witnesses.py
  tools/ci/fetch_audio_native.sh tools/check_mutants.py tools/mutants.toml
  tools/oracle/lsnes_server.lua docs/architecture/domains.toml docs/behaviours/world-camera.md
  docs/REPORT.md README.md)
for job in content replay faults-0 replay@played; do
  for path in "${common[@]}"; do reads "$job" "$path" || no "$job does not read $path"; done
done
for job in replay faults-0 replay@played; do
  for path in "${replay[@]}"; do reads "$job" "$path" || no "$job does not read $path"; done
done

# Every literal include_str!/include_bytes! path in a crate, from the including file's folder.
checked=0
while IFS=: read -r file target; do
  path=$(realpath -m --relative-to=. "$(dirname "$file")/$target")
  git ls-files --error-unmatch "$path" >/dev/null 2>&1 || continue
  checked=$((checked + 1))
  reads replay "$path" || no "replay does not read $path, which $file includes"
  case $file in
    crates/data/* | crates/import/* | crates/canonical/* | crates/clean/types/*)
      reads content "$path" || no "content does not read $path, which $file includes" ;;
  esac
done < <(git grep -oE 'include_(str|bytes)!\("[^"]+"\)' -- 'crates/*.rs' |
  sed -E 's/^([^:]*):include_(str|bytes)!\("([^"]+)"\)$/\1:\3/')
[ "$checked" -gt 0 ] || no "found no include_str! or include_bytes! in crates/"

# Planted changes, on trees built from HEAD's: a comment in an engine source or a CI fetcher reruns
# replay, one in data reruns both, and a prose doc no job reads reruns neither.
planted() {
  local path=$1 index blob
  index=$(mktemp)
  GIT_INDEX_FILE=$index git read-tree HEAD
  blob=$({ git cat-file blob "HEAD:$path"; printf '\n// planted\n'; } | git hash-object -w --stdin)
  GIT_INDEX_FILE=$index git update-index --cacheinfo "100644,$blob,$path"
  GIT_INDEX_FILE=$index git write-tree
  rm -f "$index"
}
declare -A base
for job in content replay; do base[$job]=$(bash "$verdict" tree "$job"); done
game=$(planted crates/game/src/lib.rs)
data=$(planted crates/data/src/lib.rs)
fetcher=$(planted tools/ci/replay.sh)
doc=$(planted docs/STATUS.md)
[ "$(bash "$verdict" tree replay "$game")" != "${base[replay]}" ] || no "a changed engine source keeps replay's fingerprint"
[ "$(bash "$verdict" tree content "$data")" != "${base[content]}" ] || no "a changed data source keeps content's fingerprint"
[ "$(bash "$verdict" tree replay "$data")" != "${base[replay]}" ] || no "a changed data source keeps replay's fingerprint"
[ "$(bash "$verdict" tree replay "$fetcher")" != "${base[replay]}" ] || no "a changed CI fetcher keeps replay's fingerprint"
[ "$(bash "$verdict" tree content "$fetcher")" = "${base[content]}" ] || no "a CI fetcher changes content's fingerprint"
for job in content replay; do
  [ "$(bash "$verdict" tree "$job" "$doc")" = "${base[$job]}" ] || no "docs/STATUS.md changes $job's fingerprint"
done
# Each fault worker and dispatched target is its own verdict.
[ "$(bash "$verdict" fingerprint faults-0)" != "$(bash "$verdict" fingerprint faults-1)" ] ||
  no "two fault workers share a fingerprint"

[ "$fail" = 0 ] && echo "VERDICT INPUTS OK"
exit "$fail"
