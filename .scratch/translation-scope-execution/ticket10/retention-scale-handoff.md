# Bounded hierarchical retention

Status: in progress
Owner: `/root/review_standards`
Worktree: `/tmp/sm-ticket10-retention-scale-20261014`
Branch: `ticket10-retention-scale`
Baseline: `6bbbb36c747cc7cdbfb50572e8f4fb2e1555f7eb`

Root retains the claimed ticket, main checkout/index, integration, full-suite
verification and production state. This worker owns only
`crates/translation-execution/src/retention.rs`,
`crates/translation-execution/tests/retention_scale.rs`, and this handoff.
Other workers own native recipes, Original gates and pipeline changes.

The agreed public seam is `finalize`, `authenticate_finalization` and
`retained_entries`, using real immutable temporary files. New inventories above
65,536 references use at most 256 groups; each group retains the existing leaf
or paged format. The resulting bound is 16,777,216 references, with unchanged
4 MiB metadata records. Old leaf/paged records retain their original bytes and
paths on replay. Authentication must bind exact ordered references, child
context/path and byte limits, with no recursive group nesting.

The retention API and `OperationContext` have no deadline parameter. This work
does not change upstream deadline ownership or introduce an unrelated clock.

## Verification evidence

The first public regression creates 65,537 immutable files and requires the
bounded hierarchy, exact enumeration, authentication and unchanged replay.
The baseline build completed in 1m 35s with one Cargo job. Its first run failed
at the expected schema assertion: the baseline emitted
`translation-retention-pages-v1`, while the boundary requires
`translation-retention-groups-v1`. Logs are `red.stdout.log` and
`red.stderr.log` under the verification root below. The test process exited;
its temporary fixture was removed by the test's scoped destructor. No producer
test or production command ran.

Initial green verification passed all six existing retention tests (3.51s) and
the 65,537-reference hierarchy/replay regression (70.05s), after a 9.94s
incremental build. Logs are `green.stdout.log` and `green.stderr.log`.

The implementation adds a fixed grouped schema, keeps persisted paged records
on their historical format, plans all raw/metadata/cleanup bytes before
publication, and authenticates only group-to-page-to-leaf edges. Child manifest
paths are also checked during `retained_entries`; composite leaf cleanup paths
and extents are checked against their canonical zero-removal records.
Additional corruption, historical replay and metadata reservation controls are
in progress. This checkpoint is not final verification or production approval.

## Completion evidence

Pending implementation, verification and run cleanup. Disposable Cargo output
is isolated at `/tmp/sm-ticket10-retention-scale-target-20261014`; logs will be
retained at `/tmp/sm-ticket10-retention-scale-verification-20261014`.
