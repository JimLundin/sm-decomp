# Bounded hierarchical retention

Status: ready for root integration
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

Implementation and focused verification are complete. Source checkpoints are
`1067f3954a2f2681c7b423b182f6609c46e0d8bd` (red public regression),
`e61f67d14426e7a4d4611a6559c57ab23c18be33` (green implementation), and
`f995999f16fba4fd983f6cedbfe6bbe67fecc3f0` (expanded public controls).
The worktree contains only the three owned files' changes from the baseline.
Scoped formatting and `git diff --check` passed.

All test processes finished before cleanup. Both ordinary and deduplicating
cleanup passes succeeded with zero bytes reclaimed and no cleanup targets:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
  /tmp/sm-ticket10-root-verification/bin/sm-pipeline-30bef71-opt1 cleanup \
  --root /tmp/sm-ticket10-retention-scale-verification-20261014 --apply
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
  /tmp/sm-ticket10-root-verification/bin/sm-pipeline-30bef71-opt1 cleanup \
  --root /tmp/sm-ticket10-retention-scale-verification-20261014 --deduplicate --apply
```

That retained CLI was authenticated before use with SHA-256
`8a916402c9659b5f65cc39742744421fdce3755bfbf774948f62308a5bc7e07a`.
`cleanup.*` and `deduplicate.*` preserve its output. No temporary
`translation-retention-scale-*` directory survives the fixture destructors.
The intentionally corrupted records belonged only to those private fixtures.

The disposable target held 775,364,608 allocated bytes. This command removed
it completely; Cargo reported 1,518 files and 859.1 MiB logical bytes:

```sh
CARGO_BUILD_JOBS=1 cargo clean --locked --offline \
  --target-dir /tmp/sm-ticket10-retention-scale-target-20261014
```

`target-clean.*` preserves the result. Post-cleanup available disk space was
165,678,686,208 bytes; the retained verification logs occupy 45,056 allocated
bytes at `/tmp/sm-ticket10-retention-scale-verification-20261014`.

| Retained artifact | SHA-256 |
| --- | --- |
| `red.stdout.log` | `a657be5e8febbbbff4be9068480d367e3b06baf8106ce2b6b004e2357b7430bb` |
| `red.stderr.log` | `7f03b9a64e0e78401639d12bf0fa82c7bd519302c930d1d171ca7c2cd0a8615e` |
| `green.stdout.log` | `67de87dce473e93ace316301c11320dc4c72a59285c1079d4ec56dddb96ac6d5` |
| `controls.stdout.log` | `3e7f5f2c6e40885ed7e2f18306e1dab8b790ea4dd3a3ee99f73d34fdd6d019b1` |
| `controls.stderr.log` | `c8077a936273dabded7c6b3e16903f3c9a2ef296dd6d88b919cd19c4b397e65c` |
| `cleanup.stdout.toml` | `db5bac8a65aebadf59f0fdf35212019050be91606a9165b298d3f3d192ecacdc` |
| `deduplicate.stdout.toml` | `2624524f0fb2f448e1351f8dad525947deffaa761537e3f3206162e1f18829b7` |
| `target-clean.stderr.log` | `6f275ca39d00df2fbec78e5f4ea8839598b10c4dc247aece36e2840932e1a1a5` |

The tested implementation SHA-256 is
`92e3d866737878e3a5138154cf3478f7e5923d2ecd520328f53b767bad1ac326`;
the final test file SHA-256 is
`26f13b9d7a6be8e04ac91770f5432102387e589fc97c24c1c2d7dd954c1d050b`.
Tests depend on normal locked/offline Rust dependencies and create all their
own retained input files. They use no ROM, installed production state, producer
binary, private fixture input or implementation-worktree source at runtime.
The ignored worktree `deps` symlink is only for workspace Cargo metadata.

Root owns integration, full-suite validation and retirement of this clean
implementation worktree after integration (448,532,480 allocated bytes before
the final handoff edit). No retained evidence requires its absolute source
paths or a surviving target binary, so no source relocation is needed.
To repeat focused verification from the integrated checkout, use a new private
target with:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 CARGO_BUILD_JOBS=1 \
  cargo test -p translation-execution --locked --offline --target-dir PRIVATE_TARGET \
  --test retention --test retention_scale -- --test-threads=1
```
