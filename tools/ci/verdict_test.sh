#!/usr/bin/env bash
# verdict_test.sh: tools/ci/verdict.sh's input lists keep every known input of each job, every
# file a crate include_str!s or include_bytes!s, and a changed input changes the job's fingerprint
# while a changed doc no job reads does not. Its trust checks answer the Actions API from recorded
# responses: only the job's own passing check proves a verdict. Needs jq. Run from the repository
# root.
set -u
verdict=tools/ci/verdict.sh
fail=0
no() { echo "FAIL $*"; fail=1; }

declare -A has
for job in content replay faults-0; do
  has[$job]=$(bash "$verdict" inputs "$job") || no "$job: no input list"
done
reads() { grep -qxF "$2" <<<"${has[$1]}"; }

# Every job reads the workflows, this judge, the toolchain, the workspace, vendor/ and its crates.
common=(.github/workflows/ci.yml .github/workflows/verify.yml .github/workflows/quality.yml
  tools/ci/verdict.sh rust-toolchain.toml Cargo.toml Cargo.lock crates/data/Cargo.toml
  crates/import/src/lib.rs crates/clean/types/Cargo.toml "$(git ls-files vendor | head -1)")
# The replay and fault jobs also read the engine, the scenarios, the movies, the CI fetchers and
# the fault manifest, and the docs their tests open.
replay=(crates/game/src/lib.rs crates/verify/required-witnesses.toml scenarios/replay/index.toml
  scenarios/replay/baseline.toml scenarios/locks/lsnes.toml "$(git ls-files movies | head -1)"
  tools/ci/replay.sh tools/ci/replay_parallel.sh tools/ci/required_witnesses.py
  tools/ci/fetch_audio_native.sh tools/check_mutants.py tools/mutants.toml
  tools/oracle/lsnes_server.lua docs/architecture/domains.toml docs/behaviours/world-camera.md
  docs/REPORT.md README.md)
for job in content replay faults-0; do
  for path in "${common[@]}"; do reads "$job" "$path" || no "$job does not read $path"; done
done
for job in replay faults-0; do
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
# Each fault worker is its own verdict.
[ "$(bash "$verdict" fingerprint faults-0)" != "$(bash "$verdict" fingerprint faults-1)" ] ||
  no "two fault workers share a fingerprint"
# One image release under two build numbers is one image; a new release is a new one.
image() { ImageOS=ubuntu24 ImageVersion=$1 bash "$verdict" fingerprint replay; }
[ "$(image 20261116.133.1)" = "$(image 20261116.376.1)" ] || no "one image release has two fingerprints"
[ "$(image 20261116.133.1)" != "$(image 20261123.1.1)" ] || no "two image releases share a fingerprint"

# Trust: a fake gh answers each Actions API call from a recorded response. Each case indexes one
# candidate run under replay's key; only the good one may prove the verdict.
api=$(mktemp -d)
trap 'rm -rf "$api"' EXIT
cat >"$api/gh" <<'EOF'
#!/usr/bin/env bash
shift
while [ $# -gt 0 ]; do
  case $1 in
    -X | -f | -F) shift 2 ;;
    --jq) expr=$2; shift 2 ;;
    *) path=$1; shift ;;
  esac
done
case $path in
  */actions/caches) file=caches ;;
  */attempts/1/jobs*) file=jobs ;;
  */actions/runs/*) file=run ;;
esac
jq -r "$expr" "$FAKE_API/$file.json"
EOF
chmod +x "$api/gh"
fp=$(bash "$verdict" fingerprint replay)
head=$(git rev-parse HEAD)
moved=$(git commit-tree "$game" -p HEAD -m "planted engine change")
# case_run NAME HEAD-REPOSITORY-ID TESTED-COMMIT CHECK CHECK-CONCLUSION
case_run() {
  mkdir -p "$api/$1"
  printf '{"actions_caches":[{"key":"verdict-replay-%s-4242-1"}]}\n' "$fp" >"$api/$1/caches.json"
  printf '{"path":".github/workflows/ci.yml","repository":{"id":1,"full_name":"o/r"},
    "head_repository":{"id":%s},"referenced_workflows":[{"path":"o/r/.github/workflows/verify.yml@%s",
    "sha":"%s"}]}\n' "$2" "$3" "$3" >"$api/$1/run.json"
  printf '{"jobs":[{"name":"%s","conclusion":"%s","steps":[{"name":"Record the replay verdict %s",
    "conclusion":"success"}]}]}\n' "$4" "$5" "$fp" >"$api/$1/jobs.json"
  PATH="$api:$PATH" FAKE_API="$api/$1" GITHUB_REPOSITORY=o/r bash "$verdict" find replay "$fp"
}
command -v jq >/dev/null || no "the trust checks need jq"
[ "$(case_run good 1 "$head" "verify / replay" success)" = 4242 ] || no "a passing replay check proves nothing"
case_run other-job 1 "$head" "quality / Quality" success >/dev/null && no "another job's step proves replay"
case_run failed 1 "$head" "verify / replay" failure >/dev/null && no "a failed replay check proves replay"
case_run fork 2 "$head" "verify / replay" success >/dev/null && no "a fork's run proves replay"
case_run moved 1 "$moved" "verify / replay" success >/dev/null && no "a run on other inputs proves replay"

[ "$fail" = 0 ] && echo "VERDICT INPUTS OK"
exit "$fail"
