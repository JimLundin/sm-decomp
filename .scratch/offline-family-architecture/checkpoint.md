# Offline architecture implementation checkpoint

Updated: 2026-10-15
Owner session: `01a13c3b-d421-7bb3-80a2-ec91045678e7` (`/root`)
User authorization: approved the rendered design and instructed "go ahead".

## Source and ownership

The complete assembled implementation is in
`/home/jim/sm-decomp-offline-verifier`, branch
`architecture/offline-verifier-20261015`. Frozen verification source is
`15d5a7e`; subsequent handoff/checkpoint commits change documentation only,
apart from test-only overlay `09ea8ca`. It moves the pointer-normalization C-helper check
from `pipeline/tests/family.rs` to `offline-verification/tests/dispatcher_pointer.rs`.
It preserves every assertion and does not change production or stamped source.
Main is at `5c011eb` and will receive the verified branch after the final gates.
Main's foundation commits were confirmed identical to worker commits `446dd64`
and `aeb6632`; the assembly merge preserves the newer implementation and imports
main's approved tracking documents.

Installed production policies, registries, claims, Original evidence and
unrelated mutation worktrees remain owned by their existing sessions. This
infrastructure task grants no gameplay Translation stage.

Shared process capacity remains
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`.
All Cargo work is locked/offline; the optimized verification build uses one job.

## Implementation

- Foundations are integrated and ticket 02 is resolved. Source identity and
  operation-local source context belong to `decomp-source`; process execution,
  caching, leases, bounded IO and accounting belong to `tool-execution`.
- Ordinary and Native verification, Fixture declarations, recipes, scheduling,
  comparisons and retained admission belong to `offline-verification`.
  Recipe/compiler/Fixture decisions have one private implementation. Temporary
  forwarding modules and obsolete Native recipe code are removed.
- Pipeline owns application composition, Cargo package selection, Translation
  approval, the installed-controller executable fence and cleanup. Ordinary
  admission authenticates retained producer provenance without requiring
  today's executable identity. Source stamps remain conservative and cover
  all three extracted crates.
- Final Native implementation `f5433bb` is integrated as `da02c90`; its cleanup
  evidence `a59690e` is integrated as `e0dc9ab`. Final Cargo policy/caller change
  `8f7782b` is integrated as `e299e5a`, with handoff evidence at `600eb74`.
- Final private-interface/source-publication cleanup is `4c5c03d`. The
  source freeze/assembly merge is `15d5a7e`.

## Verification

- Baseline at unchanged Rust source `a6d53c5`: 31 passed, two optional
  measurements ignored. Log `out/offline-architecture/baseline-20261015/tests.log`,
  SHA-256 `ba041cd4d4147072c3e04b20f9fd2e5178ca204e35101c10ab1fafece7cebdde`.
  Both cleanup passes succeeded; do not rerun this baseline.
- Foundation checks: 35 distinct tests passed; see ticket 02 and
  `handoff-foundations.md`. Intermediate verifier checks: 28 passed, one optional
  real reservation proof ignored; log
  `out/offline-architecture/verifier-unit-20261015/tests.log`, SHA-256
  `a782707a489ad27dab325c0dfb026efd39493e888fb49b89e1c88a423df09398`.
  Both scoped cleanup passes succeeded and its empty TMPDIR was removed.
- Native worker: all 11 owned tests passed, and pipeline production/test
  compilation passed. Caller maintained-source checks passed. These are
  implementation evidence, not final real Native proof results.
- Combined `cargo check --locked --offline --workspace --all-targets -j 2`
  passed at `600eb74`; log
  `out/offline-architecture/combined-all-targets-check.log`. Final cleanup
  removed the newly unused export/wrapper reported by this check. The new
  owners' dependency graph has no upward dependency.
- Final formatting passed. The optimized combined test build passed in
  31m58s from the verifier worktree:
  `cargo test --locked --offline --release -j 1 -p tool-execution -p decomp-source -p offline-verification -p pipeline --no-run`.
- Persistent proof parent:
  `/home/jim/sm-decomp-offline-architecture-proofs/15d5a7e-20261015`.
  It holds `source-commit.txt`, `frozen-source/` with the private ROM,
  `prerequisites.sha256`, `logs/`, `tools/` and isolated `tmp/`.
  Build log: `logs/test-build.log`; dependency review: `logs/dependencies.txt`.
  The emitted pipeline test reader is retained under `tools/sm-pipeline-tests`;
  its exact Cargo fingerprint is in `logs/pipeline-test-build-fingerprint.json`.
  Final Cargo output and `cmp` agree with the retained artifact path
  `target/release/build/pipeline/5c82b7b185b372cf/out/sm_pipeline-5c82b7b185b372cf`.
- All 63 owner tests passed: 13 `decomp-source`, 11 `tool-execution`,
  39 `offline-verification`; one optional real reservation measurement ignored.
  Logs: `logs/decomp-source.log`, `tool-execution.log`, `offline-verification.log`.
  Both cleanup passes over `owner-tmp` succeeded with zero bytes; it is removed.
- Pipeline unit tests passed: 165 passed, 21 ignored, 224.51 seconds
  (`logs/pipeline-unit-2.log`). This includes helper-dependent confinement,
  lifecycle, ordered launch and replay controls. Both cleanup passes over
  `unit-tmp-2` succeeded with zero bytes. Two tiny synthetic launch fixtures
  remain for final disposal. The Original supervisor proof removes its own root.
- The initial unit run had 150 passes and 15 setup failures because the private
  ROM and current helper environment were missing (`logs/pipeline-unit.log`).
  Diagnostics remain; the complete passing rerun supersedes this result.
  The ROM is copied and hashed. Current release sandbox, recorded-rewrite,
  evidence reader, pipeline CLI and pipeline test reader are retained in
  `tools/`; `tools/executables.sha256` records exact hashes.
- Both cleanup passes over the first `unit-tmp/` succeeded with zero targets
  and zero bytes (`logs/unit-setup-failure-cleanup*.toml`). Unsealed constructor
  inputs remain there; there are no receipts or verdicts. Preserve diagnostics
  and remove their disposable Git metadata/temporary inputs at final cleanup.
- The explicit catalog proof passed: exactly one test selected, 19.79 seconds;
  `logs/catalog.log` and `logs/catalog.time`. All 177 declarations prepared,
  covering 150,204 declared Fixture pairs plus the 360-pair animation auxiliary.
  This gate launches no producer. `catalog/cleanup*.toml` records both successful
  zero-byte cleanup passes. Keep `catalog-source/`, including generated PLM
  inputs, intact. Do not rerun this completed gate without a source change.
- Ordinary actual-C verification passed: 16 Fixture pairs, two routines.
  Retained verdict: `ordinary/1792051572065878-996235-0/verdict.toml`.
  Requests: `ordinary-verify.toml`, `ordinary-authenticate.toml`; logs:
  `logs/ordinary.log`, `ordinary.time`, `ordinary-authenticated-after-cleanup.toml`.
  Cleanup reclaimed zero bytes; deduplication reclaimed 4,194,304 bytes across
  48 snapshots. Authentication passed afterward.
- CLI integration: 76 tests passed across `affected`, `assignment_queue`,
  `baseline`, `benchmark`, `build_preparation`, `callback`, `cleanup`, `decomp`,
  `decomp_sessions` (`logs/integration.log`). One optional build measurement
  was ignored; the following evidence target lacked its helper at setup.
  Both cleanup passes over `integration-tmp` succeeded.
- With all helpers configured, `evidence_metrics` passed five tests and
  `family` passed 18, with two optional tests ignored (`logs/integration-2.log`).
  The one failure read the removed private pipeline C-helper path; the
  test-only relocation above fixes its ownership. Both cleanup passes over
  `integration-tmp-2` succeeded with zero bytes. The relocated owner integration
  test passed (`logs/dispatcher-pointer.log`); both cleanup passes reclaimed
  zero bytes and its empty TMPDIR is removed. The exact overlay is retained in
  `logs/test-relocation-commit.txt` and `logs/test-relocation.patch`.
- Pending CLI targets: `fixture`, `hdma_probe`, `iteration_demonstrations`,
  `live`, `live_frames`, `metrics`, `no_python`, `observations`, `packed`,
  `parity`, `pattern_promotion`, `patterns`, `plm_spawn`, `prefix`, `processes`,
  `producer_metrics`, `readiness_feedback`, `readiness_interrupt`,
  `resource_metrics`, `routine_jobs`, `selection`, `session_authority`.
  These are running in exec session 11117, with the changed family target,
  locked/offline release, one Cargo job, one test thread, all four current
  helper variables and fresh `integration-tmp-3`. Log: `logs/integration-3.log`.
  Other passing groups need not be repeated.
- Both real Native execution proofs remain pending. Follow
  `native-verification-plan.md`; preserve the 12 complete recipes (8,159 pairs),
  full-family negative controls and current budgets. `SM_NATIVE_PERFORMANCE_ONLY`
  must be absent. Use the retained test reader and `frozen-source` overrides;
  keep the original `15d5a7e` source identity separate from the test-only overlay.

## Cleanup and remaining work

Foundation and recipe worktrees are integrated, cleaned and removed. Native
and caller workers are finished and idle; both targets and temporary editing
tools are cleaned. Their handoffs include measurements and retained logs. No
proof depends on either implementer worktree. Root owns their final retirement.
Root's fourteen temporary Rust editing source/executable files were also removed,
reclaiming 32,178,176 allocated bytes; the measurement and disk observation are
in `logs/edit-tools-before-cleanup.txt` and `logs/disk-after-edit-tool-cleanup.txt`.

Remaining implementation worktrees are:

- `/home/jim/sm-decomp-offline-verifier`: root assembly and active verification.
- `/home/jim/sm-decomp-offline-native`: stopped, clean, integrated.
- `/home/jim/sm-decomp-offline-callers`: stopped, clean, integrated.

Next: finish the relocated pointer check and remaining CLI suites, then the
real recipe/full-family Native proofs. Retain exact tools and source,
run both cleanup passes, authenticate retained evidence, integrate on main,
retire only these worktrees/build outputs and authenticate again. Resolve
tickets 03–05 and the spec only after those checks and cleanup succeed.
