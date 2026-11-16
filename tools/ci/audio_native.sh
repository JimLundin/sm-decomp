#!/usr/bin/env bash
# Required native paired evidence, separate from unsupported ordinary cache rows.
set -euo pipefail
: "${SM_AUDIO_NATIVE:?SM_AUDIO_NATIVE is required}"
receipt=$(mktemp)
receipt_resolved=$(cd "$(dirname "$receipt")" && printf '%s/%s' "$(pwd -P)" "$(basename "$receipt")")
cleanup() {
  "${SM_PY:-python3}" - "$receipt" "$receipt_resolved" <<'PY'
import pathlib
import sys

receipt = pathlib.Path(sys.argv[1])
if receipt.is_symlink() or receipt.resolve(strict=True) != pathlib.Path(sys.argv[2]).absolute():
    raise SystemExit("audio receipt path changed")
receipt.unlink()
PY
}
trap cleanup EXIT
name=harness::audio::native::tests::native_paired_arms_are_exact
"${CARGO:-cargo}" test --locked -p verify --profile verify --lib "$name" \
  -- --ignored --exact --nocapture | tee "$receipt"
grep -Eq '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out; finished in ' "$receipt"
for arm in control expire; do
  grep -Eq "^NATIVE audio-transport $arm compared184 publications764 replies736 fields7157 .*restorations184 skipped0 defects0 cpu_timing0/140$" "$receipt"
  grep -Eq "^PENDING audio-transport $arm boundaries573 restorations573 zero_elapsed_exact committed_retained pending_retained publication_once$" "$receipt"
done
echo 'NATIVE AUDIO RECEIPT OK'
