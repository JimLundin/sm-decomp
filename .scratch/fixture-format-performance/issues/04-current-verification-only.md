# Keep one current native verification contract

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task

## Scope

The user explicitly rejects code and operational work for historical results.
Current verification performance takes precedence over historical compatibility.
Remove the obsolete readers and one-off migration/study code added by this
session. Keep current correctness checks and compact measurements/work records.

The audit found actual compatibility paths in native verification: inline
operation receipts, optional indexed-storage metadata, old reservation behavior,
and the historical single-module identity shortcut. Replace those with the
current indexed receipt and explicit candidate ownership contracts. Remove
tests whose purpose is preserving obsolete encodings or authenticating fixed
historical runs. Validate current native verification and rejection controls.

Do not edit installed production policies or other worktrees. Do not rewrite
Git history. Retire only this session's historical artifacts and reader copies;
this explicit user instruction supersedes the earlier requirement to preserve
those historical reader/source bundles.

## Comments

Ticket 03's bounded-read change passed all 248 selected tests before this
follow-up. Historical-checksum and replay work is no longer an objective.

## Implementation

- Require indexed native receipts, explicit candidate ownership, producer
  reservation anchors, current metadata capacity and fixture concurrency.
  Remove inline receipts, cumulative reservation charging, implicit single-root
  identity, missing-headless-header exceptions and five historical-only tests.
- Share each authenticated index/chunk read across receipt enumeration and
  extent accounting. Share one parsed native intent across all producer checks.
- Make the real native integration test create a current reviewed catalog and
  requirements, removing its dependency on a historical provenance run.
- Remove one-off Rust study/migration/audit programs and obsolete bundle checksum
  inventories. Keep current production commands/tests and compact work records.

## Completion evidence

The implementation is committed on `main` as `1c7e592`.

- Eleven focused native tests passed, including a 2,048-operation receipt whose
  enumeration/accounting each fit exactly one storage-read budget. The two
  receipt tests passed again after the final aggregate-reference bound check.
- Four strict producer reservation/deadline tests passed.
- The real native family test passed 16 Fixture pairs, two routines and 34
  producers. Retry reused every result. Missing coverage/dependencies, compiler
  access to undeclared files, runtime access to parent/sibling files, tampered
  invocations and prelaunch output reservation controls passed.
- The current headless recipe passed all three room-scroll Fixture pairs, each
  with 17 ordered callbacks, plus retry and authentication after cleanup.
- The real producer reservation test passed successful and failed producers,
  precise retained charging, expired retry without relaunch and quota rejection.
- All 18 selected native/producer tests passed. Formatting and `git diff --check`
  passed. Logs are `../evidence-audit/test-current-*.log`.

The project cleanup utility and deduplication pass completed on each of this
session's three archived bundle directories, with zero remaining cleanup
candidates and 24 snapshots checked. The directories were then retired at the
user's explicit request: 335,859,712 allocated bytes / 334,366,078 logical bytes.
Commands, paths and measurements are in `../evidence-audit/cleanup-retired-bundles.log`
and `retired-bundles-{allocated,logical}.txt`.

After every writer finished, the project utility cleaned and deduplicated
`/tmp/sm-fixture-evidence-audit-cNZPAN/current-native` and `test-runs`. The test
root had zero candidates. Native cleanup had zero removable intermediates and
deduplicated 10,485,760 bytes across 309 snapshots; the headless test had already
deduplicated 13,762,560 bytes across 213 snapshots and authenticated afterward.
The complete private build/run directory was then removed: 2,152,906,752 allocated
bytes / 2,138,316,307 logical bytes. Including the old bundles, the removed
directories accounted for 2,488,766,464 allocated bytes. Final available disk:
163,732,357,120 bytes. No worktree was created; installed policies and other
workspace work were not changed. No historical reader/source dependency remains
for this task.

Commands used the already-built `sm-pipeline` from the private target directory:

```sh
sm-pipeline cleanup --root RUN_ROOT --apply
sm-pipeline cleanup --root RUN_ROOT --deduplicate --apply
```

`RUN_ROOT` was each of the three paths in `retired-bundles-allocated.txt`, then
`/tmp/sm-fixture-evidence-audit-cNZPAN/current-native` and
`/tmp/sm-fixture-evidence-audit-cNZPAN/test-runs`. The archived roots were removed
with `rm -r --` at their exact paths after the utility passes. The current native
and test roots were removed with their owned parent directory,
`/tmp/sm-fixture-evidence-audit-cNZPAN`. Logs and byte/disk
measurements are in `../evidence-audit/cleanup-*.log`,
`cleanup-headless-deduplicated.toml`, `private-directory-{allocated,logical}.txt`
and `disk-after-cleanup.txt`.
