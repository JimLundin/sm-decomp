# Fresh frozen-source Native verification

Status: active; building exact current release readers
Owner: `/root/recover_extraction_verification`
Integration owner session: `01a13e7f-ab01-7413-a1c2-288b08a4b719`
Frozen source: `7b27ea1a29bb2ade3bc6483f99bf59626e298671`
Worktree: `/home/jim/sm-decomp-performance-integration`
Branch: `performance/verification-integration-20261015`

## Ownership and required scope

This is a new proof, independent of the failed historical PLM attempt. The
candidate combines the extracted owners, metrics changes, storage-only example
and Native operation admission fix. The integration owner keeps this worktree
frozen and owns merges, shared registration, final admission and retirement.
This verifier writes only its new proof parent and this handoff. Ignored
read-only-use links supply the existing `deps/snesrev-sm`, `deps/rustysnes`
and private ROM prerequisites; no tracked candidate source is modified.

Retained parent:
`/home/jim/sm-decomp-performance-native-proof/7b27ea1-20261015`.

Required gates retain the original Native plan: complete catalog preparation,
twelve complete recipe declarations / 8,159 Fixture pairs with PLM first, the
complete 16-pair full-real Native proof including negative controls, applicable
application checks, both cleanup passes and independent retained authentication.
No `SM_NATIVE_PERFORMANCE_ONLY`, domain reduction, deadline enlargement or
historical-attempt replay is authorized.

## Source and build checkpoint

`source-commit.txt` and `tracked-source.sha256` seal the complete source.
`frozen-source/` and `catalog-source/` were independently copied from the exact
Git archive and checked against those hashes; each has a copied private ROM.
`prerequisites.sha256` records the pinned C archive, original/copied ROM and
captured Ridley input hashes. The Native harness retains and authenticates all
other declared absolute Original capture dependencies at their existing paths.

The idle old extraction Cargo target is exclusively reserved for these builds:
`CARGO_TARGET_DIR=/home/jim/sm-decomp-offline-verifier/target`. Its earlier emitted
readers are already separately retained. Process inspection found no writer
before reuse. All Cargo commands use locked/offline release, one job, and a
separate 1,800-second build bound. All producers use the shared root
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`.

Active reader build: exec session `80649`; observed timeout PID `1507495`,
Cargo PID `1507496`. TMPDIR is `<proof-parent>/build-tmp`.

```sh
timeout --signal=TERM 1800s cargo test --locked --offline --release -j 1 \
  -p pipeline --bin sm-pipeline --no-run
```

Exact stdout/stderr and command/resource results are in `logs/reader-build.log`
and `logs/reader-build.time`. Do not launch a duplicate. Retain the exact emitted
test executable immediately, then build the current CLI and separate sandbox,
recorded-rewrite and evidence helpers. The 33 CLI integration targets are not
rebuilt merely to obtain this reader. Later execution invokes retained binaries
directly so a narrowed Cargo command cannot change the feature cohort.

## Gate configuration

The twelve families, in order, are `plm-spawn`, `projectile-script-leaves`,
`collision`, `power_bomb`, `game-host-projectile`, `game-host-room-scroll`,
`ridley-subsystem-captured`, `plm-instructions`, `pause-direct-animation`,
`animated-tile-frames`, `animated-tile-spawns`, `ceres-pad`.

Recipes retain four Fixture jobs in the shared four-worker pool, 600-second
per-family gates, 64-GiB aggregate output, 180-second compiler limits, and
five-second / 16-MiB Fixture limits. Preparation uses the original 600-second /
8-GiB contexts. The full-real proof retains its 180-second gate, 1-GiB output,
60-second compilers and five-second / 512-KiB Fixtures. Reconciliation and
independent Native authentication keep their original 60-second bounds.

Proof roots will be `catalog/`, `recipes/` and `full-real/` under this new parent.
Each explicit ignored gate must select exactly one test. Retain all source,
producer identities/results, indexed operations, receipts and failure diagnostics.
Check unchanged receipt/result hashes and mtimes on retry and authentication.

## Cleanup and remaining work

No verification producer has run yet. Wait for every relevant writer before
cleaning its completed or failed root. Use the separately retained current CLI:

```sh
<proof-parent>/tools/sm-pipeline cleanup --root <finished-root> --apply
<proof-parent>/tools/sm-pipeline cleanup --root <finished-root> --deduplicate --apply
```

Retain exact stdout, bytes reclaimed, free space and source/reader/private-input
dependencies. Independently authenticate Native receipts afterward using the
retained test reader's exact `retained_native_proofs_authenticate_without_execution`
test and the proof's `retained-authentication.toml`. The integrator owns worktree/
target retirement and the final post-retirement admission. A concrete gate failure
is reported immediately; it is not silently retried or treated as acceptance.
