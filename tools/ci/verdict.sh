#!/usr/bin/env bash
# verdict.sh: a verification job reruns only when something it reads has changed (#741).
#
# A job's fingerprint hashes the mode and object id of every tracked file it reads at the tested
# commit, the job's name (with its fault worker) and the runner image. The workflow files, this
# script and the pinned assets' SHA-256s (in verify.yml and tools/ci) are tracked inputs like any
# other. When a job's work passes, its last step saves a cache entry
# verdict-JOB-FINGERPRINT-RUN-ATTEMPT and is named "Record the JOB verdict FINGERPRINT".
#
# The cache entries only index candidate runs: a later job trusts a candidate run after checking
# with the Actions API that it is this repository's CI, that it tested a commit whose inputs for
# JOB are the same as ours (so it ran these same workflows and this script), and that JOB's own
# check (verify / replay) passed with its step "Record the JOB verdict FINGERPRINT", which runs
# only after the job's work passed. A forged entry can only name a run that really did the work.
# Scheduled and dispatched runs always do the work, so their diagnostics print; a dispatched
# target records no verdict. Delete a job's entries (gh cache delete) to force its rerun.
#
#   verdict.sh inputs JOB [TREE]       the tracked paths JOB reads at TREE (default HEAD)
#   verdict.sh tree JOB [TREE]         the SHA-256 of those paths' ls-tree entries
#   verdict.sh fingerprint JOB [TREE]  the tree hash, JOB and the runner image
#   verdict.sh find JOB FINGERPRINT    the id of a run that passed JOB at FINGERPRINT; exit 1 if none
#   verdict.sh step                    in a job: reuse a verdict or say why the work runs
# JOB is content, replay or faults-WORKER.
set -uo pipefail

# Whether JOB reads PATH. Everything is an input unless the job provably never reads it.
reads() {
  local job=$1 path=$2
  case $path in
    # The workflows of every job in the run, and the script that judges them.
    .github/workflows/* | tools/ci/verdict.sh) return 0 ;;
    # Agents' instructions: no crate, test or CI tool a verification job runs opens them.
    .agents/* | AGENTS.md) return 1 ;;
    # The clean crates' tests include_str! the behaviour specs, and verify's report reads the
    # specs and their notes.
    docs/behaviours/*) ;;
    # The other prose docs: no include_str!, test or CI tool reads them.
    docs/*.md) return 1 ;;
  esac
  case $job in
    # The data and import tests read the crates and the workspace manifests. The ROM is
    # a runtime input. No data or import source or test names a file under these directories.
    content) case $path in tools/* | scenarios/* | movies/* | docs/*) return 1 ;; esac ;;
  esac
  return 0
}

# Every ls-tree entry of TREE that JOB reads, one per line.
entries() {
  local job=$1 tree=${2:-HEAD} entry
  git ls-tree -r -z --full-tree "$tree" | while IFS= read -r -d '' entry; do
    if reads "$job" "${entry#*$'\t'}"; then printf '%s\n' "$entry"; fi
  done
}

tree_hash() {
  local listing
  listing=$(entries "$1" "${2:-HEAD}") && [ -n "$listing" ] || return 1
  printf '%s\n' "$listing" | sha256sum | cut -d' ' -f1
}

# The image is its OS and its release date: one release (ImageVersion 20261116.133.1) is served
# under several build numbers (20261116.376.1) at once, by runners picked at random.
fingerprint() {
  local tree release=${ImageVersion:-}
  tree=$(tree_hash "$1" "${2:-HEAD}") || return 1
  printf 'verdict 1\njob %s\nimage %s %s\ntree %s\n' "$1" "${ImageOS:-}" "${release%%.*}" "$tree" |
    sha256sum | cut -d' ' -f1
}

# JOB's check name in a CI run: verify / content, verify / replay, verify / faults (WORKER).
check() {
  case $1 in
    faults-*) echo "verify / faults (${1#faults-})" ;;
    *) echo "verify / $1" ;;
  esac
}

# Whether run RUN attempt ATTEMPT passed JOB's work at FINGERPRINT on inputs whose hash is TREE.
proven() {
  local run=$1 attempt=$2 job=$3 fp=$4 tree=$5 repo=$GITHUB_REPOSITORY tested
  tested=$(timeout 30 gh api "repos/$repo/actions/runs/$run" --jq '
    select(.path == ".github/workflows/ci.yml" and .head_repository.id == .repository.id)
    | .repository.full_name as $repo | .referenced_workflows[]
    | select(.path == ($repo + "/.github/workflows/verify.yml@" + .sha)) | .sha') || return 1
  [[ $tested =~ ^[0-9a-f]{40}$ ]] || return 1
  git cat-file -e "$tested^{commit}" 2>/dev/null ||
    timeout 120 git fetch -q --no-tags --depth=1 --filter=blob:none origin "$tested" || return 1
  [ "$(tree_hash "$job" "$tested")" = "$tree" ] || return 1
  local passed
  # Only JOB's own check counts, and only when it passed: any other job could name a step so.
  passed=$(timeout 30 gh api "repos/$repo/actions/runs/$run/attempts/$attempt/jobs?per_page=100" --jq "[
    .jobs[] | select(.name == \"$(check "$job")\" and .conclusion == \"success\") | .steps[]
    | select(.name == \"Record the $job verdict $fp\" and .conclusion == \"success\")] | length") ||
    return 1
  [ "$passed" -ge 1 ] 2>/dev/null
}

find_verdict() {
  local job=$1 fp=$2 prefix="verdict-$1-$2-" tree key suffix run attempt
  tree=$(tree_hash "$job") || return 1
  # Newest first; a few candidates are enough, since any passing run proves the verdict.
  for key in $(timeout 30 gh api -X GET "repos/$GITHUB_REPOSITORY/actions/caches" \
    -f key="$prefix" -f sort=created_at -f direction=desc -F per_page=10 \
    --jq '.actions_caches[].key'); do
    suffix=${key#"$prefix"} run=${suffix%-*} attempt=${suffix##*-}
    [[ $run =~ ^[1-9][0-9]*$ && $attempt =~ ^[1-9][0-9]*$ ]] || continue
    if proven "$run" "$attempt" "$job" "$fp" "$tree"; then echo "$run"; return 0; fi
  done
  return 1
}

# In a verification job: name it, fingerprint it, and reuse a passing verdict unless the run is
# scheduled or dispatched (COLD=true), which always does the work.
step() {
  local job=${JOB:?} fp run url
  [ -n "${WORKER:-}" ] && job+="-$WORKER"
  echo "job=$job" >>"$GITHUB_OUTPUT"
  if ! fp=$(fingerprint "$job"); then
    echo "::warning::$job: no fingerprint, so the work runs and no verdict is recorded"
    return 0
  fi
  if [ -n "${TARGET:-}" ]; then
    echo "$job at $fp: a dispatched target does its work and records no verdict"
    return 0
  fi
  echo "fingerprint=$fp" >>"$GITHUB_OUTPUT"
  printf '%s %s\n' "$job" "$fp" >"$RUNNER_TEMP/verdict"
  if [ "${COLD:-false}" = true ]; then
    echo "$job at $fp: a scheduled or dispatched run always does its work"
  elif run=$(find_verdict "$job" "$fp"); then
    url="$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$run"
    echo "reused=$run" >>"$GITHUB_OUTPUT"
    echo "$job at $fp: reused the passing verdict of run $run ($url)"
    echo "$job reused the verdict of [run $run]($url) at fingerprint \`$fp\`" >>"$GITHUB_STEP_SUMMARY"
  else
    echo "$job at $fp: no passing verdict, so the work runs"
  fi
}

case ${1:-} in
  inputs) entries "${2:?JOB}" "${3:-HEAD}" | cut -f2- ;;
  tree) tree_hash "${2:?JOB}" "${3:-HEAD}" ;;
  fingerprint) fingerprint "${2:?JOB}" "${3:-HEAD}" ;;
  find) find_verdict "${2:?JOB}" "${3:?FINGERPRINT}" ;;
  step) step ;;
  *) sed -n '2,23s/^# \{0,1\}//p' "$0"; exit 2 ;;
esac
