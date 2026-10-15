# Offline architecture implementation checkpoint

Updated: 2026-10-15
Owner session: `01a13c3b-d421-7bb3-80a2-ec91045678e7` (`/root`)
User authorization: approved the rendered design and instructed "go ahead".

Recovery: the original owner was confirmed `systemError` by the performance
implementation coordinator `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Its `/root/recover_extraction_verification` child owns only the final frozen-source
gates and this effort's retained proof output. The coordinator owns integration,
main Git/index and worktree retirement. No production claim or installed policy
has been adopted, replaced or changed. The worktree was recovered clean at
`8bf8acf610e1a94a1a3bccd5d718dae4a4244857`; its production source is byte-identical
to freeze `4a736587af557de9e44b3126ff08f6d6d11beee8`.

## Recovered verification handoff

All verification processes owned by the recovery child have finished. The
pipeline unit suite passed 165 tests (21 explicitly ignored), 229.35 seconds,
in exec session `42584`. Its log SHA-256 is
`b82e0e60bf227d776cb8dd4a5e4b3d7b9d907541998d99ad60e174f41c9a9e76`.
The previously recovered owner suites passed 65 unit tests and one dispatcher
integration test; the complete catalog and ordinary 16-pair gates also passed.

The first Native recipe gate failed during PLM finalization, after all 1,969
PLM pairs passed and all 3,941 producer operations were indexed. Its error is
`trusted input whole-stage deadline exhausted`; no Native receipt exists.
[The diagnostic](native-deadline-diagnostic.md) records exact source references,
timestamps, retained evidence, a non-producing reproducer proposal and the
source-derived repeated-admission count. Native acceptance is incomplete.

Both cleanup passes succeeded for every completed/failed recovery run. Total
utility reclamation is 434,765,824 bytes: 4,194,304 ordinary plus 430,571,520
PLM snapshot bytes. Unit cleanup reclaimed zero bytes; retain its fourteen
small synthetic launch-plan/state files listed in `logs/unit-retained-files.txt`.
Its empty compact-recovery directory was removed. All other recovery TMPDIRs
are removed. The original regression diagnostics remain unchanged.

At handoff, ordinary authentication passed again
(`logs/ordinary-authenticated-at-handoff.toml`); retained PLM intent/verdict/index/
chunk/result hashes and precise mtimes are unchanged. Exact reader/helper and
private prerequisite hashes passed again. Production source still matches the
freeze (`logs/production-source-at-handoff.diff` is empty). Free disk space was
126,212,780,032 bytes. The proof parent, source snapshots, tools, original C/ROM
and absolute Original capture dependencies remain in place.

At the integration owner's explicit instruction, no CLI integration, full-real
Native proof or matrix retry was launched after the unit suite. Return this
child's execution slot for the scoped Native/operation admission repair. The
child identity remains available for a later frozen candidate. The coordinator
owns source repair/merge, remaining gate claims, final admission and worktree
retirement; neither integration nor retirement is complete.

## Source and ownership

The complete assembled implementation is in
`/home/jim/sm-decomp-offline-verifier`, branch
`architecture/offline-verifier-20261015`. Current production freeze is
`4a736587af557de9e44b3126ff08f6d6d11beee8`, including the PLM wrapper/accounting
correction and passing regressions. Final proofs use the new retained parent
`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015`.
Historical frozen verification source is
`15d5a7e`; subsequent source changes are test-only overlays `09ea8ca` and
`1a770f8`. The first moves the pointer-normalization C-helper check
from `pipeline/tests/family.rs` to `offline-verification/tests/dispatcher_pointer.rs`.
It preserves every assertion and does not change production or stamped source.
The second completes the real recipe test's source snapshot from each selected
declaration's relative inputs, retaining absolute Original capture paths.
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
  `unit-tmp-2` succeeded with zero bytes. Its two synthetic launch fixtures were
  removed with the first unit run's unsealed setup inputs. The Original
  supervisor proof removes its own root.
- The initial unit run had 150 passes and 15 setup failures because the private
  ROM and current helper environment were missing (`logs/pipeline-unit.log`).
  Diagnostics remain; the complete passing rerun supersedes this result.
  The ROM is copied and hashed. Current release sandbox, recorded-rewrite,
  evidence reader, pipeline CLI and pipeline test reader are retained in
  `tools/`; `tools/executables.sha256` records exact hashes.
- Both cleanup passes over the first `unit-tmp/` succeeded with zero targets
  and zero bytes (`logs/unit-setup-failure-cleanup*.toml`). Its unsealed constructor
  inputs and Git metadata had no receipts or verdicts and are now removed.
  `logs/disposable-unit-fixtures.txt` retains their inventory;
  `logs/disposable-finished-tmp-bytes.txt` records 4,349,952 allocated bytes
  removed across both unit TMPDIRs and four empty finished TMPDIRs.
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
- `logs/integration-3.log`: the changed family target passed all 18 remaining
  tests (two ignored). Another 27 tests passed across `fixture`, `hdma_probe`,
  `iteration_demonstrations`, `live`, `live_frames`, `metrics`, `no_python`,
  `observations`, `packed`, `parity`, `pattern_promotion`. `patterns` stopped on
  a missing `deps/65816/v1` prerequisite in the isolated worktree. The test's
  developed corpus is separate; it still canonicalizes the existing pinned
  location and later copies the original EA/CA cases. A read-only link to main's
  existing corpus now provides this prerequisite; exact input hashes are in
  `logs/pattern-corpus-prerequisites.sha256`. No source change was needed.
  Both cleanup passes over `integration-tmp-3` succeeded; its rejected pattern
  report and diagnostic source snapshot remain.
- The remaining CLI targets passed all 37 tests: `patterns`,
  `plm_spawn`, `prefix`, `processes`, `producer_metrics`, `readiness_feedback`,
  `readiness_interrupt`, `resource_metrics`, `routine_jobs`, `selection`,
  `session_authority`. One session-process child entry is ignored by default.
  They used locked/offline release, one Cargo job, one test thread and all four
  current helper variables. Log: `logs/integration-4.log`. Both cleanup passes
  over `integration-tmp-4` succeeded. Overall final CLI coverage is 163 passed,
  four ignored, across 33 integration targets; none remains pending.
- The first real Native recipe attempt stopped during declaration, before any
  producer ran (`logs/recipes.log`, `recipes.time`, output `recipes/`). The
  diagnostic replay (`recipes-diagnostic/`, `logs/recipes-setup.strace.gz`) found
  the omitted declared input `crates/pipeline/src/ridley_subsystem_captured.rs`.
  That source still correctly belongs to pipeline; the test snapshot copied
  compiled harness inputs but omitted additional manifest-declared inputs.
  Both cleanup passes succeeded for both failed roots. Preserve diagnostics.
- Test-only fix `1a770f8` copies all selected declarations' relative inputs,
  including the direct-animation base declaration; absolute capture paths stay
  unchanged. Production/stamped source is unchanged from `15d5a7e`.
  The replacement pipeline test reader built successfully in 7m24s:
  `cargo test --locked --offline --release -j 1 -p pipeline --bin sm-pipeline --no-run`.
  Logs: `logs/native-reader-build.log`, `native-reader-build.time`.
  Exact changed test sources are retained in `native-reader-source/`, with
  `logs/native-reader-source-commit.txt` and `native-reader-test-overlay.patch`.
  It is retained as `tools/sm-pipeline-tests-1a770f8`, hash appended to
  `tools/executables.sha256`; `cmp` confirms the emitted Cargo executable.
  The original reader remains for the prior passing catalog/unit evidence.
  Exactly one revised recipe test was selected and finished in 717.60 seconds
  in exec session 53868, with a fresh `recipes-2/` root. Logs:
  `logs/recipes-2.log`, `recipes-2.time`, `recipes-2-selected-test.txt`.
  The first five families passed execution, authentication, unchanged retry
  and per-family cleanup (2,192 Fixture pairs). PLM then failed: its confined
  wrapper redeclared `plm`, and finalization reported `native fixture coverage
  missing` while accounting for the retained flat allocation manifest. The
  rejected compiler result is retained, but PLM published no Native receipt.
  Production fixes are in `plm_spawn.rs`, `recipe.rs`, and `native.rs`; focused
  regressions now pass. Preserve the old frozen source and evidence unchanged.
  Both whole-root cleanup passes succeeded with zero bytes reclaimed and 7,048
  snapshots checked. Independent historical authentication with ordinary and
  filtered strace exceeded the unchanged 60-second input deadline; diagnostics
  remain. The untraced reader passed all five families in 46.98 seconds
  (`logs/recipes-2-authentication-untraced.log`). Every Native receipt and
  producer-result hash and precise mtime is unchanged across authentication;
  see `logs/recipes-2-retained-{before,after}.{sha256,mtimes}`. Use untraced
  authentication for the large final matrix; tracing smaller controls remains
  possible without extending any deadline.
- Corrected-source verifier tests: 41 passed, one optional proof ignored,
  including generated PLM ABI execution in all four ordinary/confined and
  canonical/renamed placements, and saved flat-PLM rejection replay with absent
  helper/compiler paths, expired deadline and unchanged receipt hash/mtime.
  Existing inflated-coverage/extent denial still passes. Logs are under the new
  proof parent `/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015`,
  `logs/regression-3.log`. An initial test compile needed the explicit `Case`
  import; the next run passed all behavior assertions but two cleanup callbacks
  lacked `SM_PIPELINE_CLEANUP_BIN`. The complete passing rerun supplies it.
- Both actual Native execution proofs and independent post-cleanup/retirement
  Native authentication remain pending. Select all 12 declarations (8,159 pairs)
  exactly as in `native-verification-plan.md`, compiler limit 180 seconds,
  Fixture jobs/workers four, `SM_NATIVE_PERFORMANCE_ONLY` unset. The archive and
  private ROM hashes passed again before the first attempt.

## Cleanup and remaining work

### Corrected-source verification in progress

Recovery observed original build shell PID `1292644` stopped. Both retained time
records report exit 0: the combined optimized test build completed in 20m24.05s,
and helper builds in 1m20.54s. Exact emitted
pipeline test, CLI, sandbox, recorded-rewrite and evidence-reader executables
are retained in the corrected-source proof parent's `tools/`, with hashes in
`tools/executables.sha256`. The pipeline reader was compared byte-for-byte with
the emitted Cargo path; its Cargo fingerprint is retained. Source, private ROM,
pinned archive and captured Ridley input checks passed again at recovery.

The narrowed owner Cargo invocation in exec session `72561` selected a different
feature union and began rebuilding `offline-verification`. It was interrupted
before any test or verification producer ran; `logs/owners.log` and
`logs/owners.time` retain the diagnostic. Continue with the already-built exact
test executables listed in `logs/test-build.log`, avoiding a feature-union build.
Use one test thread, the shared workers4 pool, isolated `owner-tmp/`, and the
retained cleanup executable. Complete both scoped cleanup passes before
proceeding to application, ordinary,
catalog, all twelve real recipes (PLM first), full real Native controls and
standalone retained authentication. No full-catalog fresh execution is planned.

The direct emitted owner executables completed successfully: 13 `decomp-source`,
11 `tool-execution`, 41 `offline-verification` unit tests (one optional measurement
ignored), plus the one dispatcher pointer integration test. Logs and time records
are `logs/{decomp-source,tool-execution,offline-verification,dispatcher-pointer}.*`.
Both cleanup passes over the interrupted build's empty TMPDIR and the completed
owner TMPDIR passed, with zero bytes reclaimed; the empty TMPDIR was removed.
The ordinary actual-C gate is now running in exec session `22958`, using the
unchanged `ordinary-verify.toml` and retained current pipeline executable.
The complete intended environment is retained in
`logs/recovered-execution-environment.toml`; each explicit Native gate's `--list`
output selects exactly one test (`logs/*-selected-test.txt`).

Ordinary actual-C passed 16 Fixture pairs / two routines at
`ordinary/1792064366867709-1385229-0/verdict.toml`. Both run cleanup passes
succeeded; deduplication reclaimed 4,194,304 bytes across 48 snapshots, and
`logs/ordinary-authenticated-after-cleanup.toml` reports `offline-verified`.
Both passes over its empty TMPDIR succeeded, and that directory was removed.
The complete current catalog gate passed in 38.47 seconds: 177 declarations /
150,204 Fixture pairs plus the 360-pair animation auxiliary. Its own two cleanup
records report zero bytes, and its separate empty TMPDIR was cleaned and removed.
Generated PLM inputs are sealed in `logs/generated-catalog-plm-inputs.sha256`.

Current gate: all twelve required Native recipe declarations, PLM first,
in exec session `97392` / observed reader PID `1387304`. Exact stdout/stderr are
in `logs/recipes.log`, timing and exit status in `logs/recipes.time`, and the full
selection/environment in `logs/recovered-execution-environment.toml`. Use the
existing process and `recipes/` evidence; do not duplicate its producers. Its
fixture capacity remains four in the shared pool, compiler limit 180 seconds,
per-family gate limit 600 seconds, and complete domain 8,159 pairs. Other
owners' worktrees and producers are unchanged. Application suites, full Native
controls and independent final authentication remain pending after this gate.

Recipe gate `97392` has now stopped, exit 101 at 510.41 test seconds. First
failure: `trusted input whole-stage deadline exhausted` during the separate
60-second finalization/reconciliation block, before Native receipt publication.
All 1,969 PLM pairs / three routines passed actual-C comparison and all 3,941
producer records were indexed. The other eleven families did not run.
[The precise diagnostic and narrow reproducer proposal](native-deadline-diagnostic.md)
record measured boundaries, source references and the remaining localization
uncertainty. No deadline changed and no matrix retry was launched.
Both failed-root cleanup passes succeeded: 430,571,520 bytes deduplicated,
5,907 snapshots checked. All retained intent/verdict/index/chunk/result hashes
and mtimes remain equal. Its empty TMPDIR was cleaned and removed. Native
reauthentication remains impossible without a published receipt.

At the integration owner's direction, independent current-source application
checks continue. The default pipeline unit suite is running in exec session
`42584`, using `tools/sm-pipeline-tests`, one test thread, all current retained
helper variables and separate `unit-tmp/`. Exact stdout/stderr and command/time
records: `logs/pipeline-unit.log` and `logs/pipeline-unit.time`. CLI integration
and applicable full-family controls remain pending; source repair belongs to
the coordinator, not this verification worker.

Locked/offline workspace all-target compilation passed in 46.27 seconds
(`plm-fix-20261015/logs/check.log`). Release owner/application no-run build and
helper builds completed serially in original exec session 98089, one Cargo job
and the shared workers4 settings. Logs: `logs/test-build.log`, `logs/helper-build.log`.
Both `frozen-source/` and `catalog-source/` match `tracked-source.sha256` from
the freeze. Private archive, ROM and captured Ridley input hashes match the
reviewed prerequisites; `prerequisites.sha256` retains exact paths and values.

Both cleanup passes over the completed regression TMPDIR succeeded with zero
utility reclamation. Two unsealed synthetic worker binaries from the earlier
cleanup-callback setup failure were removed, reclaiming 9,056,256 measured
allocated bytes. Their diagnostic sources, metrics and stderr remain in
`regression-tmp/`; `logs/regression-disposable-build-bytes.txt` records the
measurement. No runtime or build process writes to that directory.

Foundation, recipe, Native and caller worktrees are integrated, cleaned and
removed. Native/caller retirement reclaimed 484,716,544 measured allocated
bytes; `logs/worker-worktrees-before-retirement.txt` and
`worktrees-after-worker-retirement.txt` retain the inspection/removal evidence.
`logs/ordinary-authenticated-after-worker-retirement.toml` records successful
ordinary authentication afterward. Their source/check logs are integrated;
no retained proof depends on either implementer worktree.
Root's fourteen temporary Rust editing source/executable files were also removed,
reclaiming 32,178,176 allocated bytes; the measurement and disk observation are
in `logs/edit-tools-before-cleanup.txt` and `logs/disk-after-edit-tool-cleanup.txt`.

The only remaining implementation worktree is
`/home/jim/sm-decomp-offline-verifier`, which owns root assembly and active
verification. Main currently also contains an unrelated untracked performance
effort; preserve it during integration.

The failed pattern test's unsealed Git metadata and the empty final CLI TMPDIR
were removed. Its rejected report and source snapshot remain. The Native
diagnostic trace was losslessly compressed; its decompressed hash matches the
original. These steps reclaimed 36,962,304 measured allocated bytes; see
`logs/failed-setup-disposable-before.txt`, `failed-setup-retained-after.txt`
and `recipes-setup-{uncompressed,decompressed}.sha256`.

Next: the integration owner coordinates the scoped repeated-admission repair
described in the diagnostic, with unchanged limits and evidence contracts.
Then return a frozen candidate and exact remaining gate claim. All twelve
declared recipes, full real Native controls and standalone Native authentication
remain required; the newer current-source CLI integration gate also remains
pending. Preserve this failed proof and the earlier historical proofs unchanged.
Retain exact tools and source, complete both cleanup passes after new runs,
authenticate retained evidence, integrate on main, retire only these worktrees/
build outputs and authenticate again. Resolve tickets 03–05 and the spec only
after those checks and cleanup succeed.
