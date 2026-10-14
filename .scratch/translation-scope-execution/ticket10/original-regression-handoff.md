# Original supervisor prerequisite regression

Status: ready for root integration
Owner: `/root/review_standards`
Worktree: `/tmp/sm-ticket10-original-regression-20261014`
Branch: `ticket10-original-regression`
Baseline: `30bef717a560f450c1e11cf5f77573f91996a3a3`

Own only `crates/pipeline/src/translation_original_tests.rs`, its test-module
registration in `translation_original.rs`, and this handoff. Root owns the main
checkout/index and Original implementation. Other workers own native recipes,
confinement and compact recovery; do not revert their work or mutate production.

The agreed seam is `translation_execution::execute`, the public lifecycle shared
by the installed single-scope command and ordered launcher. The regression uses
an isolated Original PauseEquipment prerequisite failure, retaining the primary
247-Fixture declaration and real original Rust test source while deliberately
omitting auxiliary direct animation. It must stop with that exact missing-family
diagnosis, finalize bounded evidence and retire its diagnostic candidate checkout.
Unchanged inspect/resume/run must preserve the stopped view and every retained
file's identity and modification time. Submitted source, original claim and
installed policy remain exact.

This fixture grants no native, managed production, Live replay or original-ROM
acceptance. Actual Original producer failure and ordered continuation proofs
remain root-owned. It uses no test-only production hooks.

## Verification evidence

The focused build passed with six existing warnings. The public prerequisite
regression passed in 36.75 seconds. Its stopped view has exactly this reason:

> pause equipment requires auxiliary direct animation; plain Native-family acceptance is insufficient

The result remains attempt 1 under its original policy seal, with
`cleanup_complete`, `retained_authenticatable` and `worker_retired` all true.
It reports 46,505 retained bytes and 156,070 reclaimed bytes. Its policy allows a
second deliberate attempt, so unchanged resume cannot pass merely by exhausting
the attempt budget. The fixture retains all ten original Rust test declarations;
it does not execute those controls or supply gameplay acceptance.

The ignored retained-reader control passed after both cleanup passes (23.16
seconds), then again after disposable Cargo target retirement (24.85 seconds).
It calls the same public inspect/resume/run lifecycle and verifies that every
fixture file keeps its hash, extent and modification time. No new fixture, gate,
operation, model request or capture is created by that reader.

Successful fixture: `/tmp/sm-complete-supervisor-1300786-0`.
Its exact Submitted source is `ac7076718f5a7c2f94467a6610322ea283f46311`.
Logs and installed readers:
`/tmp/sm-ticket10-original-regression-verification`.

| Artifact | SHA-256 |
| --- | --- |
| `bin/sm-pipeline-original-tests-v2` | `50e076dfc91716285e1654eec8f443e1e0f6e67a1de95763a3f809f463230acc` |
| Successful fixture `installation.json` | `6a975992daa82ec69a9ca56ecc8541533b3a2c54c83e44140d650806c7be51a0` |
| Successful fixture `stopped-view.json` | `435f1eaaaff4d761c64feac693fe6ddc41523b3c49944eb1d7c4b777557170fa` |
| Successful fixture `original-claim.json` | `ca98f4308d7cc0c4b2d2797e277516648787a5d320020d4eef26c5a7206deabb` |
| `original-prerequisite-v2.stdout.log` | `1e216eaf60e1f68679c27848593be9d7b4605666676e994f68aad7b45d4fb8d5` |
| `retained-after-cleanup.stdout.log` | `37bd2c366ede37d129ab9bd1c40063d84078d0ad390d479250b94a560722adec` |

The first run failed on fixture setup because `/usr/bin/false` was not owned by
the controller. The corrected fixture copies that readiness probe into its own
installed-tools directory. The failed setup created no supervisor attempt;
its source, Installation, reader and logs remain at
`/tmp/sm-complete-supervisor-1299019-0` and `original-prerequisite.*`. This is a
setup diagnostic, not an Original stopped-run acceptance claim.

## Cleanup and dependencies

All writers finished before cleanup. Both `cleanup --apply` and
`cleanup --deduplicate --apply` succeeded on each fixture root, with zero
additional bytes reclaimed. Successful-fixture allocated usage remained
7,892,992 bytes. `original-cleanup.*`, `original-deduplicate.*` and
`setup-failure-{cleanup,deduplicate}.*` retain the utility output.

The private `/tmp/sm-ticket10-original-regression-target` had 1,856,675,840
allocated bytes. `cargo clean --locked --offline --target-dir` removed its 2,375
files (Cargo reported 1.7 GiB); `target-clean.*` retains the result. Observed free
space afterward was 162,909,319,168 bytes. No compile or test writer remains.

Keep both fixture roots and the retained readers outside the worktree. The
successful Installation binds its copied helper/compiler/probe, the retained v2
reader, and the matching `nightly-2026-09-29-x86_64-unknown-linux-gnu` Cargo/rustc
pair. `retained-reader-runtime.txt` records its ordinary system-library closure.
The helper bytes were copied from the reviewed v4 helper with SHA-256
`1ec74e0bcc1024ae460d74b47148114526bf6743e2cc4a87302f9ecf551ab6d9`.
The retained prerequisite reader needs no ROM or implementation-worktree files.

Root may retire this implementation worktree after integrating the scoped
commit. Authenticate the same saved failure afterward with:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
SM_ORIGINAL_SUPERVISOR_PROOF_ROOT=/tmp/sm-complete-supervisor-1300786-0 \
timeout 120 /tmp/sm-ticket10-original-regression-verification/bin/sm-pipeline-original-tests-v2 \
  --ignored --exact \
  translation_original::tests::retained_original_prerequisite_failure_authenticates_without_execution \
  --nocapture
```

To run the ordinary regression after integration, build the pipeline tests with
the shared workers4 settings and run exactly
`translation_original::tests::original_missing_auxiliary_prerequisite_stops_once_and_resume_preserves_submission`.
Supply `SM_TRANSLATION_TEST_HELPER` with the reviewed helper. The reused Project
fixture also accepts `SM_TRANSLATION_RECORDED_REWRITE=/usr/bin/false`; no model
response is used. Set `SM_SUPERVISOR_KEEP_INFRA=1` only when retaining a new
diagnostic fixture for evidence.
