# Metrics recovery verification, 2026-10-12

Status: implementation integrated; overhead gate remains open.

Recovered implementation `a3290f5`, integration `bd82cf2`, support-history
merges `8d1f071` and `3260224`. Source and exact restart instructions:
`../checkpoint-02.md`. Ticket 02 is not resolved by these partial results.

## Validation

Stable logs: integration `out/ticket-02-final/logs/`.

- `sm-resume-metrics-controls.log`: 27 public controls passed: evidence metrics
  5, family 10, process metrics 5, packed 2 and resource metrics 5. Includes
  SIGTERM/timeout descendant teardown, capacity release, actual C CPU/RSS,
  concurrent RSS, sparse/hardlink/symlink allocated disk, unavailable/truncated
  fields, parallel producers, capture/import/authentication stages and exact
  enabled/disabled evidence identities.
- `sm-resume-model-evidence.log`: complete pipeline-model/evidence tests passed.
- `sm-resume-clippy.log`, `sm-resume-fmt.log`: strict Clippy across all targets
  of pipeline/model/evidence and workspace formatting passed. Diff whitespace
  passed before committing. The previous session's `controls-final.log` also
  records 4 process/cache and 8 real producer fault controls passing.
- Previous `unit-final.log`: 59/60 passed; the missing-ROM fixture test failed.
  `sm-resume-fixture-regression.log` reruns that complete 44-domain comparison
  successfully after restoring the worktree ROM link. The passing 59 were not
  rerun without a new reason. Initial continuation evidence tests also failed
  from an absent default reader path; rebuilding and explicitly selecting the
  reader yielded the final passing public controls.
- Fresh release build and actual current-source system/decompressor/RNG/live
  admission passed. Exact paths are in the checkpoint. These fresh runs retain
  provenance/source material under the integration checkout and authenticate
  independently of removed support worktrees.

The fresh live root measurement includes nested build/cache/process accounting,
parallel capture, shared import/publication, verdict publication, scoped cleanup
and retained authentication. Its sampled resource fields are complete with the
documented rolling `/proc`/between-samples limitations. Elapsed 403.799239476s
was measured under load and is not a comparable performance sample.

## Telemetry measurement limit

Retained Rust control source and executable are in
`out/ticket-02-final/telemetry-control/` and `retained-tools/telemetry-control`.
It invokes public native packed commands using the same admitted binaries,
four 45,000-input subjects, exact evidence comparison, three alternating on/off
pairs and fresh private/shared stores. No binary transport is implemented.

First sample was stopped after detecting substantial unrelated CPU/memory
activity in the host process list. Native SIGTERM teardown produced an
interrupted root receipt, four interrupted subject receipts, no accepted
pointers and no surviving FIFOs. `sm-resume-telemetry-pairs.stderr` records the
refusal. There is no completed pair/report or <=2% overhead claim. Ticket 03
cannot begin admitted sampling until this gate passes on a quiet host.

## Completion cleanup evidence

- `sm-pipeline cleanup --root out/ticket-02-resume --apply` and corresponding
  `--deduplicate --apply`: both succeeded, 0 bytes reclaimed.
- `sm-pipeline cleanup --root out/ticket-02-final --apply`: succeeded, 0 bytes.
  Corresponding dedup succeeded: 332,450,186 bytes, 2,162 snapshots checked.
  Receipts: `sm-resume-prior-{cleanup,dedup}.toml` and
  `sm-resume-final-{cleanup,dedup}.toml`.
- `live-auth-after-cleanup.toml`: authenticated four subjects, 148 artifacts
  each, retained binary valid, sm-evidence-v2. Decomp, RNG and system public
  authentication receipts also passed after cleanup/worktree removal.
- Cargo clean removed metrics target 3.4 GiB, its worktree target 1.2 GiB,
  resource target 993.5 MiB and evidence target 633.1 MiB (Cargo's logical
  reports). Release/control and older ticket-02 evidence targets were cleaned
  after retaining the executables; exact receipts are `sm-resume-clean-*.log`.
  The failed fixture matrix contained 20,661,695 logical bytes before removal.
- Clean integrated worktrees `/tmp/sm-pipeline-upgrade-metrics`,
  `/tmp/sm-pipeline-upgrade-resources` and
  `/tmp/sm-pipeline-upgrade-evidence-metrics` were removed. Their final control
  files were identical in integration before recording the support merges.
- Remaining filesystem free: 222,377,586,688 bytes at the final measurement
  (54,291,403 available blocks of 4,096 bytes). Other host activity means
  free-space changes are not wholly attributable to this cleanup.

Retain the integration checkout, its source/admission snapshots, shared
dependencies and ROM/movie paths, `out/ticket-02-final/evidence-store`, retained
readers/executables and accepted evidence. Deduplicated interrupted diagnostics
remain at `telemetry-pairs/sample-1-disabled/` (1,682,902,854 logical bytes).
The generic utility does not delete this unsupported partial capture staging;
it is retained failure evidence, not accepted or reusable performance evidence.
Original ticket-01 historical dependencies remain intact. No target tree or
support Git worktree is needed for the new retained authentication.
