# Deterministic executor continuation

Status: implementing
Owner: `01a136a0-49bb-7690-a360-48def3c2c09b`
Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`
Checkpoint: `27b8b0ac78080d9ccc953956e1829bf0571efbe2`

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
archive files respectively. All use the shared pipeline root and workers4,
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

Build09 passed. The single fresh proof is running at
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

Read-only authority status04 still matches the saved production identities and
all four main hashes. Main HEAD remains `c2cae9ac145f9343cd345e6f978bff0774510b56`.
Next action: observe the proof09 result, preserve concrete failures, finish only
affected verification, both final cleanup passes, measured disposal and retained
authentication, then checkpoint exact source/docs/tickets/compact receipts.

## Recovered checkpoint and current next action

The latest root continuation retained the existing `/root/bounded_git`,
`/root/managed_admission` and `/root/supervisor_audit` children. Direct
collaboration inspection reports bounded Git running, managed admission errored
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
The new managed actual-C test is being authored against this fix. Wait for its
stable-source message before wiring and rebuilding. Retain concrete failures;
do not repeat the already passing broad/linker matrix.

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
build trees. Tickets 02–07 remain claimed until those completion steps; ticket08
and the production switch remain pending with the documented real blockers.
