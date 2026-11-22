#!/usr/bin/env bash
# Check the required witnesses and the whole reviewed baseline at once: the witnesses run beside
# three indexed shards, one per core of the runner, and the shard receipts merge into RECEIPT,
# byte for byte the receipt one unsharded check writes. Every part runs to its end; each part's log
# is printed under its own label, and any failing part fails the check.
set -uo pipefail
receipt=${1:?usage: replay_parallel.sh RECEIPT}
: "${SM_CACHE:?SM_CACHE is required}" "${SM_ROM:?SM_ROM is required}" "${SM_AUDIO_NATIVE:?SM_AUDIO_NATIVE is required}"
shards=3
logs=$(mktemp -d)
trap 'rm -rf "$logs"' EXIT

labels=(witnesses)
"${SM_PY:-python3}" tools/ci/required_witnesses.py "" > "$logs/witnesses.log" 2>&1 &
pids=($!)
merge=()
for ((shard = 0; shard < shards; shard++)); do
  labels+=("shard-$shard")
  bash tools/ci/replay.sh --shard "$shard/$shards" --write "$logs/shard-$shard.toml" \
    > "$logs/shard-$shard.log" 2>&1 &
  pids+=($!)
  merge+=(--receipt "$logs/shard-$shard.toml")
done

failed=0
for i in "${!pids[@]}"; do
  label=${labels[$i]}
  if wait "${pids[$i]}"; then
    echo "::group::$label passed"
    cat "$logs/$label.log"
    echo "::endgroup::"
  else
    status=$?
    failed=1
    echo "::error::$label failed with exit $status"
    cat "$logs/$label.log"
  fi
done
if ((failed)); then exit 1; fi
"${SM_TOOLS:-target/verify/tools}" replay merge scenarios/replay/index.toml --all \
  --baseline scenarios/replay/baseline.toml "${merge[@]}" --write "$receipt"
