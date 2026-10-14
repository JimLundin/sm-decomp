# Bounded hierarchical retention

Status: verification passed; cleanup pending
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
Expanded controls passed both public integration tests in 147.62s after a
1.05s incremental build. `controls.stdout.log` and `controls.stderr.log` retain
that result. The 65,537-reference fixture now covers:

- Reversed input and identical duplicate normalization, conflicting kinds or
  extents, a changed inventory, and forbidden explicit retirement.
- Recovery of missing cleanup records at the root and leaf, with every
  surviving metadata and input file preserving inode, size and modification time.
- Changed retry context and byte limit, and a one-byte-short complete hierarchy
  budget rejected before publishing any parent manifest or group directory.
- Resealed parent/child context, byte-limit, child path and page path changes.
- The 256-group, 65,536-reference and 16-page group bounds; mismatched counts,
  incomplete prefixes, cross-group duplicate ordering, forbidden nested groups,
  an oversized child metadata record and relocated child cleanup records.
- Changed child manifests, cleanup observations and retained file contents.

The separate historical fixture serializes the original paged wire format from
17 leaves built through public `finalize`. It authenticates and enumerates all
65,537 references, then replays with identical results and metadata inodes,
sizes and modification times. No `groups` directory is created. This directly
checks historical records above the new 16-page group threshold.

The test mutators restore original bytes and read-only permissions. Both
temporary fixture destructors completed; no fixture or test process survives.
The original six regression tests passed against the same implementation;
only the new test file changed after that run. No full 16-million-reference
fixture or production catalog execution is claimed; root owns integration and
the full suite.

## Completion evidence

Implementation and focused verification are complete. Scoped run cleanup is
pending. Disposable Cargo output is isolated at
`/tmp/sm-ticket10-retention-scale-target-20261014`; logs are retained at
`/tmp/sm-ticket10-retention-scale-verification-20261014`.
