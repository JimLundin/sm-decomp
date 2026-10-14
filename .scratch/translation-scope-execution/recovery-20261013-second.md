# Deterministic executor continuation

Status: implementation proved; production preparation pending
Owner: `01a136a0-49bb-7690-a360-48def3c2c09b`
Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`
Verified source: `f845516b975c3bfc413800c6bed9e329696a79e6`
Proof/documentation checkpoint: `9dd4361`
Completion: [completion-20261014.md](completion-20261014.md)

## Completed executor — production preparation is next

The current continuation has inspected the same three children through
collaboration: `bounded_git` and `managed_admission` errored with encrypted-output
transport failures; `supervisor_audit` completed. No replacement was launched.
At startup no Cargo, native gate, validator or supervisor writer remained in the
selected run directories; proof 11's later writer is recorded below. Root owns
the returned source and this worktree's Git/index.
The production integrator thread is observed completed; its held production
claims and registered generation remain untouched.

Proof 10 finished with a real managed acceptance, both standard queue completions
and three progress observations, but its outer supervisor stayed `Publishing`.
Ticket authentication reconstructed only one rendering while normal publishing
preserves the preceding generated ticket bytes through successive transitions.
The unchanged retained graph now authenticates after the bounded lifecycle
correction. The normal ticket producer and original human prose are unchanged.
Known lifecycle stages are limited to four; skipped projections after an
interruption are admitted only as exact normal renderings, and foreign prose
still rejects.

The read-only proof 10 diagnostic passed in 16.07 seconds at
`verification/managed-accepted-diagnostic-10-green.{stdout,stderr}.log`.
Its stdout SHA-256 is
`08abdcfa58dea1928e28dd3fbdb7fd9934839540eb339dffef08ce9453b8d75a`;
stderr is empty. Two real ticket-publishing regressions passed in 0.06 seconds
at `verification/managed-ticket-projection-green-02.{stdout,stderr}.log`;
stdout SHA-256 is
`80dfe5ae31c69ed6712147307b1f54ec5b364dc7597965c043a20929c1dd8158`.
The affected migration suite passed 11 tests with two ignored diagnostics;
admitted-progress regression passed. Pipeline Clippy's one cloned-slice finding
was corrected and the final Clippy run passed with the established exceptions.

The fresh managed proof at `managed-gates/actual-c-managed-11` was launched once
at 2026-10-13 22:35:37 UTC with the command below and completed successfully.
Its former writer PID 1098911 (parent 3389, start ticks 6514166, boot
`7119fcce-5593-4526-aa9b-07127767c4ef`) is observed stopped. Exec session 39653
returned exit 0. Do not rerun this or the earlier passing producer matrices.

The full managed interface test passed in 601.65 seconds. Its loop took
103.420 seconds; execution and recovery took 587.984 seconds excluding fixture
construction. Four actual-C gates contain two Fixture pairs each;
six unique gate producers supply 24 references with 18 reuses. Fourteen operation
receipts are retained. Finalization retained 27,249,752 bytes; publication
retained 1,203,738 bytes. Run logical bytes are 27,480,037; retired checkouts
reclaimed 7,666,300 logical bytes. Exact pending recovery allocated zero new
producer evidence. Original job/claim/worker/attempt and original owner source
are unchanged; both normal queue entries complete and three distinct stages
publish. Partial/changed/symlink/hardlink pending and host/acceptance tampering
all refuse. This is a disposable Observed/Helper diagnostic with synthetic
stopped-owner observation; it establishes no production liveness, shipped
gameplay, original-ROM parity or Live replay acceptance.

Both cleanup passes for 09/10/11 passed with zero eligible bytes and 12 source
snapshots each. Managed 11's exact installed controller passed read-only
authentication in 36.63 seconds (session 50837, exit 0). Final pipeline Clippy with
the documented shared exceptions passed in 34.65 seconds (session 48294, exit 0);
the current locked/offline CLI build passed in 15.12 seconds (session 85450, exit 0).
The affected source remains byte-identical to checkpoint f845516.

Both final verification-directory cleanup passes passed with zero eligible
bytes. The Rust disposal plan passed reader authentication, process-reference
and alias checks. Its 23 closed targets measure 5,195,113,120 logical bytes and
4,825,128,960 reclaimable allocated bytes; available space is 186,285,498,368 bytes.
Receipt: `verification/disposable-retirement-plan-final-01.jsonl`. It contains
only disposable Cargo/protocol targets, unreferenced helper copies in failed 06/07
and six ignored C copies in this worktree. It excludes every sealed reader,
private original source/input graph and `/tmp/sm-complete-supervisor-976030-0`.

The unchanged Rust disposal helper applied successfully. Its receipt is
`verification/disposable-retirement-apply-final-01.jsonl`; all 23 selected paths
are absent, 5,195,113,120 logical bytes and 4,825,128,960 allocated bytes were
retired. Remaining space is 191,102,251,008 bytes. The protected
`/tmp/sm-complete-supervisor-976030-0` and all sealed readers remain intact.

Read-only actual-c-02 authentication passed in 27.31 seconds (session 54720, exit 0)
through `supervisor-native/tools/sm-supervisor-retained-reader-02`; managed 11
authentication passed in 41.98 seconds (session 97359, exit 0) through its exact
`managed-gates/actual-c-managed-11/installed/controller`. Retained output stems
are `verification/{native-02,managed-11}-authenticate-after-retirement-final-01`.
Both verification-directory cleanup passes then passed with zero eligible bytes
and `verification-cleanup-{ordinary,deduplicate}-final-02` receipts. The compact
receipt helper published 47 exact references and the managed checkpoint.

Both final helper binaries and their source are retired. The four helper paths
measured 9,656,336 logical bytes and 9,666,560 allocated bytes; their plan/apply
receipts are `verification/helper-retirement-{plan,apply}-final-01.jsonl`.
Combined allocated reclamation is 4,834,795,520 bytes. Final observed available
space is 191,088,582,656 bytes. No verification process remains. The disposable
Cargo targets are absent and must not be recreated only for inspection.
Tickets 02–07 are resolved; ticket 08 remains open.

```sh
cd /home/jim/sm-decomp-translation-execution
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
SM_MANAGED_NATIVE_PROOF_ROOT=/home/jim/sm-decomp-translation-execution/.scratch/translation-scope-execution/managed-gates/actual-c-managed-11 \
/tmp/sm-translation-execution-target/debug/build/pipeline/7fa9a98880d37858/out/sm_pipeline-7fa9a98880d37858 \
  --exact translation_execution::managed_tests::managed_native_publication_preserves_original_claim_and_recovers_exact_pending_once \
  --ignored --nocapture --test-threads=1 \
  > .scratch/translation-scope-execution/verification/managed-native-proof-11.stdout.log \
  2> .scratch/translation-scope-execution/verification/managed-native-proof-11.stderr.log
```

This executable is 240,702,016 bytes, SHA-256
`cde6329250ae3657385aa9a887ccdb6fc881a3329f4495757449e37c9bd4ecb8`.
The producer command above is historical. Inspection uses the retained exact
readers, and all affected checks/cleanup are complete. Formatting of all three
affected Rust files passes. The disposable retirement and compact-receipt helpers
are removed; their receipts remain. Preserve all sealed readers, source and
private input graphs. Do not repeat the passing broad/native/linker matrices.

Final read-only authority status 06 and main status are byte-identical to 05.
Main HEAD and all four authority/Ceres/canonical/progress hashes match the
recorded production baseline. Next work is ticket 08's exact retained original
package and concrete per-original preparation, sequencing, catalog/canonical
work and fresh owner reconciliation. Production activation requires its
separate explicit decision after that package is reviewable; no approval is
requested here. [Final completion](completion-20261014.md) and
[47 retained references](verification/completion-f845516.references.json)
record the exact completed source, results and cleanup.

## Recovery history

The user asked this replacement to continue the separate deterministic Rust
pipeline, not the gameplay swarm. The previous root
`01a13642-1d9c-74b2-8ff6-a1440621241f` failed with the encrypted function-output
decoding error. Direct thread inspection observes three surviving children:
native gates `01a1364b-470f-7022-bc8d-9c92a383230f`, migration
`01a1364b-e119-7ce1-9247-1cae7e2eb3b8`, and revision retention
`01a1364b-8ffc-7cd2-b6be-87f244026e93`. Their existing ownership is retained.
No duplicate children were launched. App-server follow-up to inherited v2
children is refused; this replacement inspects their durable handoffs and
thread state. Their source edits are preserved.

Root owns `crates/pipeline/src/translation_execution.rs`, its combined interface
tests, shared wiring, and worktree Git/index checkpoints. Existing children own
their native adapter/build/input files, migration/queue fencing, and revision
archive files respectively. All use the shared pipeline root and workers 4,
with independent build targets. Main production queue/authority/progress and
gameplay submissions remain outside this effort.

Current core verification is retained at
`verification/recovery-core-01.{stdout,stderr}.log`; it passes the complete
recovered `translation-execution` suite. Root continues exact revision recovery,
aggregate resource accounting, terminal publication authentication, complete
native/infrastructure supervisor proof and final scoped cleanup.

The agreed production switch remains a separate explicit user decision after
the concrete proved implementation and migration package are reviewable.

## Current continuation — 2026-10-14

The same real root thread remains `01a136a0-49bb-7690-a360-48def3c2c09b`.
Direct collaboration inspection now reports both `bounded_git` and
`managed_admission` errored with encrypted-output transport failures;
`supervisor_audit` completed. Root owns their already returned source and the
managed fixture. No child was relaunched and no duplicate agent was created.

The managed fixture now uses a small locked/offline Cargo project with pinned
`serde_json = 1.0.151`; artifact JSON supplies its executable path. Attempt07's
manual-link metadata-stub failure remains retained. Attempt08 passed the Cargo
build (3.76 seconds), then correctly stopped before rewriting because its
512-MiB gate cap could not reserve 814,900,336 bytes. The stopped result retained
3,169,571 bytes and retired 3,831,466 checkout bytes. Its exact controller/reader,
Installation, journal, source and diagnostics are preserved. Source cap is now
1 GiB, inside the unchanged 8-GiB total attempt budget. This changes only the
diagnostic fixture's reviewed installation, not production enforcement.

Build09 passed. The single fresh proof finished at
`managed-gates/actual-c-managed-09`, with logs
`verification/managed-native-proof-09.{stdout,stderr}.log`. Its exact command is
the built test executable's `--exact
translation_execution::managed_tests::managed_native_publication_preserves_original_claim_and_recovers_exact_pending_once
--ignored --nocapture --test-threads=1`, with `SM_MANAGED_NATIVE_PROOF_ROOT` set
to that absolute root and the shared pipeline/worker settings. The fixture
retains `installed/controller` before sealing Installation, and contains a
separate ignored read-only retained managed-publication authenticator.

Both scoped cleanup passes passed for all managed roots through08 and for the
production preparation; receipts are
`verification/{managed,preparation}-cleanup-{ordinary,deduplicate}-final-01.toml`.
Each reclaimed zero eligible bytes. Disposable Cargo targets and unreferenced
tool copies still require measured retirement after the final affected checks.
Do not overwrite retained Installation/readers or run another producer proof
solely for measurement. Inspect the current writer before resuming any action.

Read-only authority status 04 still matches the saved production identities and
all four main hashes. Main HEAD remains `c2cae9ac145f9343cd345e6f978bff0774510b56`.
Proof 09's completed failure and correction are retained below. Final cleanup,
measured disposal and retained authentication remain part of the current next
action above.

### Original queue correction and completed proof 10

Source checkpoint `bcf3de30865e117bfa520e481df6a6b5a718490d` retains exact pending
recovery and the locked/offline managed fixture. Proof 09 passed all four actual-C
gates and retired producer checkouts, but retained `Publishing` without an
accepted result. Its finalization retained 27,249,762 bytes and reclaimed
7,666,307 checkout bytes. Original host source/queue remained unchanged.

The new ignored read-only test
`translation_migration::tests::retained_managed_intent_and_controller_projection_authenticate_without_mutation`
isolated the real refusal in 7.71 seconds: `managed queue schema or integration
owner changed`. A legacy Running implementation can have no integration owner
before the normal Submitted transition sets it. The production check now retains
that exact absence while Running, accepts only the current fenced owner after
submission, and rejects missing/foreign or prematurely changed owners. The same
unchanged retained 09 graph passes the correction and foreign-owner refusal in
7.93 seconds; red/green logs are
`verification/managed-publication-diagnostic-09{,-green}.{stdout,stderr}.log`.

Build12 passed. One necessary fresh complete proof of this corrected source
finished at `managed-gates/actual-c-managed-10`, with
`verification/managed-native-proof-10.{stdout,stderr}.log`. Its command is the
same exact managed test above, replacing only the proof root with 10. The test
records loop and recovery time separately and derives gate producer/reuse counts
from authenticated receipts when complete. Proof 10 failed after 97.34 seconds
because the outer supervisor remained `Publishing`; the real acceptance graph
and corrected read-only diagnosis are described in the current checkpoint above.
Root owns the current production check and test source. Original10/09/08
Installation/readers and diagnostics remain immutable.

After observing proof 10, finish affected migration/progress/Clippy checks,
scoped cleanup and measured disposal. The task-specific Rust retirement helper
is `/tmp/sm-translation-retirement{.rs,}`; it has a closed list of disposable
targets, authenticates retained controller readers, rejects process references
and path aliases, and measures logical/allocated/reclaimable bytes. Run `plan`
before `apply`, retain compact receipts, then retire the helper itself. Do not
delete `/tmp/sm-complete-supervisor-976030-0`'s sealed state, source or referenced
installed tools, and do not delete any managed Installation/readers/private
source graph. Production activation remains outside this completion.

## Historical checkpoint 27b8 and its then-current next action

The latest root continuation retained the existing `/root/bounded_git`,
`/root/managed_admission` and `/root/supervisor_audit` children. Direct
collaboration inspection then reported bounded Git running, managed admission errored
with the encrypted-output transport failure, and supervisor audit completed.
The new diagnostic managed proof was reassigned to the existing bounded Git
child; no replacement was launched. Its ownership is only
`crates/pipeline/src/translation_managed_tests.rs` and necessary test helpers in
`translation.rs`. Root owns production fixes, module wiring, builds and the
worktree Git/index. All earlier owned source is in checkpoint `27b8b0a`.

The completed actual-C supervisor proof is bound by
[actual-c-02-checkpoint.json](supervisor-native/actual-c-02-checkpoint.json),
with 16 Fixture pairs across two enemy-jump routines, 34 gate producers,
102 reused references, a 255.393-second loop and zero new resume evidence.
The full native test and infrastructure/retry/failure/accounting suites passed;
[compact references](verification/checkpoint-27b8b0a.references.json) retain exact
results. Both native proof cleanup passes and its post-cleanup authentication
passed. Keep its exact retained reader and original private source dependencies.
The Installation's original controller path was a disposable test target later
rebuilt; its immutable installation is not rewritten. Historical read-only
authentication uses the retained reader and source/evidence graph.

The real three-Ceres package was built read-only and separately revalidated as
`validated-with-blockers`; its original builder refusal and byte-identical retry
decisions remain immutable. See
[preparation checkpoint](production-preparation/preparation-checkpoint-27b8b0a.md).
The validator's actual catalog read-budget failure was reproduced and fixed with
an explicit bounded 300-second/8-GiB total read allowance. This changes no
producer output limit or unique 1-GiB migration reference extent.

The remaining managed recovery gap is fixed in Root's current source:
completed resume binds actual `accepted.pending` bytes to the retained exact
acceptance, authenticates the full original graph, then promotes and repairs only
accepted projections. Read-only inspection still requires `accepted.json`.
At checkpoint 27b8 the new managed actual-C test was being authored against this
fix. Its source and the subsequent proof results are now retained above. Retain
concrete failures; do not repeat the already passing broad/linker matrix.

Read-only startup inspection on this continuation observed no build, gate or
validator writers in the selected run directories. Current authority status is
retained in `verification/recovery-authority-status-03.stdout.toml`: integrator
`ceres-20261013`, session `01a13592-d02b-7143-8489-a15b03f19da2`, generation 2.
Main HEAD is unchanged at `c2cae9ac145f9343cd345e6f978bff0774510b56`, and the
authority/Ceres ledger/canonical ledger/progress hashes are unchanged:

```text
832bce6368c9b156b6776364e1356a659f6c46f34760c520e0568e7c94da2bbd authority
77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89 Ceres ledger
b0afdf5de1625810d07a1e2fe33f5a1e52703589b05899942d32529e04997832 canonical ledger
ca493db9139b369f948084da820946bf2c39fcd955cda76cee6c3127cd927ec4 progress
```

After the single managed proof and affected focused gates pass, finish both
scoped cleanup passes, retire disposable targets/diagnostics/scripts and ignored
fixture copies, reauthenticate the retained native proof, and checkpoint exact
Rust/docs/tickets/compact evidence. Do not stage binaries, ROMs, bulk evidence or
build trees. Tickets 02–07 remain claimed until those completion steps; ticket 08
and the production switch remain pending with the documented real blockers.
