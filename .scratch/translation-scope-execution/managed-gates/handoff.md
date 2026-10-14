# Managed original-result admission

Status: owned implementation verified; integration owner has remaining gates
Owner: `/root/managed_admission`
Worktree: `/home/jim/sm-decomp-translation-execution`
Branch: `architecture/translation-scope-execution-20261013`

The recovered child owns `translation_migration.rs` and narrow admission helpers
in `assignment.rs`, `progress.rs`, and `progress/tests.rs`. Root owns supervisor
authentication, host Git publication, shared wiring and checkpoints. This work
changes no main-checkout source, production claim, authority or inventory.

## Supported admission

Fresh execution requires the original `Running` native `Implement` claim under
the current original integrator token. Original claim tuple, real worker owner,
job/ticket, checkpoint source, scope/inventory/exercise dependencies and deadlines
remain immutable. Every original gate must map exactly to a frozen plain
`family-verify` declaration, with complete conservative catalog coverage.
Original NativeScope provenance names its exact original declaration/owner;
historical Offline proof cannot authorize another scope.

Cargo-test, Live replay, batch, specialized native, infrastructure and legacy
full-tree `source-fingerprint` adapters remain fail closed. Original Reserved,
Submitted and Integrate states cannot be freshly rewritten. Preparation retains
an exact original Submitted/standard-derived Integrate pair, including its older
representation without `base-head`, and reports held sequencing as unsupported.
Active migration still rejects all overlapping writable ownership. Legacy
abbreviated `base-head` bytes stay intact while validation resolves an unambiguous
full Git commit and checks original checkpoint ancestry.

## Implemented boundaries

- `AdmissionDraft` binds completed finalization, original ledger/source/ticket/
  progress snapshots and exact nonowned queue file inputs before snapshot side
  effects. `AdmissionIntent` authenticates its draft, normal original submission,
  native evidence, cleanup, progress plan and retained closure before host CAS.
- `assignment::FrozenJobInputs` binds schema, canonical checkout, full original
  Job and every maintained explicit/legacy nonowned file-input key to retained
  bytes. Approved writable candidate edits stay excluded. This optional internal
  input applies only to exact authenticated original Submitted result recording;
  public assignment requests cannot supply it.
- `record_admitted_locked(request, expected_claim, frozen_inputs)` and
  `reserve_admitted_integration_locked` retain the ordinary Submitted -> standard
  Integrate -> Integrated queue contract. Only affected original/derived tickets
  are projected, under current fenced authority and the shared mutation lock.
- `ControllerProjection` enumerates fixed ledger/ticket/progress/managed paths,
  including future absent records. Semantic validation permits exact authenticated
  current or interrupted normal projections. Unrelated files, ledger state,
  history, owner/generation, results, registry history and HTML are rejected.
- Earlier exact ticket lifecycle renderings are repairable. Progress recovery
  preserves exact stage/owner/source/evidence identity and canonicalizes observation
  order/timestamps; no arbitrary missing history or registry replacement is allowed.
- Accepted closure explicitly includes draft/intent/original snapshots and inputs,
  finalization references and source-publication intent/index-marker/result records.
  Source-stage context and observation IDs are bound before accepting controller dirt.
- Fully written exact pending draft/snapshot/completion/intent/accepted records
  can recover. Partial or changed pending bytes are preserved and fail closed for
  explicit recovery. Arbitrary temporary queue/progress files remain unsupported;
  there is no broad `.scratch` exemption or automatic metadata commit.
- Root owns deterministic stages; the redundant managed `stages.json` copy was
  removed. `publish(..., _stages)` retains its existing root-call signature.
- Managed metadata reserves 96 MiB. Each original metadata/input snapshot is
  bounded to 4 MiB, and the incremental retained graph must fit the reservation
  before source/queue mutation. Larger metadata or full-tree fingerprints stop.

## Verification

Use `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`.
Owned disposable target: `/tmp/sm-translation-managed-admission-target`.

- `test-build-05`: compilation passed, 42.58 seconds.
- `test-build-06`: compilation passed, 66 seconds; migration 8/8 and admitted
  progress 1/1 pass in `migration-test-05` and `progress-test-03`.
- `test-build-07`: compilation passed, 60 seconds; migration 10/10 and admitted
  progress 1/1 pass in `migration-test-06` and `progress-test-04`.
- `test-build-08`: compilation passed, 20.59 seconds; migration 11/11 and admitted
  progress 1/1 pass in `migration-test-07` and `progress-test-05`.
- `progress-test-02` retained the identified timestamp/order failure, fixed and
  covered by distinct admitted timestamps and partial-projection recovery tests.
- `test-build-09` and `test-build-10` sampled concurrent builder edits and failed
  only on incomplete builder symbols/type derivation. Final checks wait for that
  source to settle. No previous pass is described as covering later source.
- `test-build-11`: compilation passed on the settled production source in
  79 seconds. `migration-test-08` passed 11/11 and `progress-test-06` passed
  the admitted progress test 1/1. Formatting and scoped diff checks passed.
- `assignment-test-01`: the ordinary direct-module test target failed to compile
  because it lacked `inputs::bounded` and `translation_migration`. This failure
  is retained. Root owns extracting the exact production migration guard and
  wiring all three affected integration-test targets; the final repaired checks
  must follow that change. No test-only fence bypass was added.

The exact four owned source files tested by `test-build-11` are retained in
`tested-source-11/`, with a SHA-256 manifest authenticated before and after
cleanup. The test binary SHA-256 was
`0810e8f71bf030679843025e2edf056a3a6c9dbeaced7fc79851ad321e70b8df`;
the disposable binary has been removed. Root's later guard extraction is a
separate source state that needs its own final verification.

Tests include original fenced queue transitions and exact retries, candidate
writable edits, projected ticket changes, later host dependency changes, missing/
tampered frozen input closure, stale generation/claim, original owner preservation,
prepared historical linked pair, active overlap refusal, abbreviated baseline
preservation, stale hash/fingerprint/symlink refusal, exact pending recovery and
partial pending preservation, and progress projection conflict/tamper rejection.

These interface tests do not prove complete managed supervisor execution. Full
supported NativeScope actual-C proof through the managed publisher remains absent.
Existing enemy-jump proof covers bank A0; original direct exercise admission covers
bank86, so they are not interchangeable. Root coordinates the final supported proof.
Exact `accepted.pending` recovery has been reviewed against current source
publication fields but lacks a complete managed native execution test. Root has
the recommended proof: move exact accepted bytes to the pending name and resume,
then test preservation/refusal of partial or changed pending metadata. Queue
hardening review also reported nested unknown Entry/AttemptRecord fields and
bounded recovery-directory enumeration for Root's integration pass.

## Cleanup and handback

Owned cleanup completed after all owned writers stopped. Exec sessions 45190
(test-build-11) and 1285 (assignment-test-01) finished with exit 0 and 101,
respectively. Final process inspection is retained in
`target-removal-processes.txt`; it found no process referencing the owned target.
Both maintained cleanup commands passed:

```sh
.scratch/translation-scope-execution/native-gates/tools/sm-pipeline-cleanup cleanup --root /home/jim/sm-decomp-translation-execution/.scratch/translation-scope-execution/managed-gates --apply
.scratch/translation-scope-execution/native-gates/tools/sm-pipeline-cleanup cleanup --root /home/jim/sm-decomp-translation-execution/.scratch/translation-scope-execution/managed-gates --deduplicate --apply
```

Receipts `cleanup-ordinary.toml` and `cleanup-deduplicate.toml` report zero
reclaimed bytes. The retained cleanup reader SHA-256 is
`56f578572a0322da2ab0b22984ab220158e966f506c9c7c092c39757b7e76375`.
Only the stopped, disposable `/tmp/sm-translation-managed-admission-target` was
removed with `rm -r --`. Final target measurements were 2,388,981,180 logical
bytes and 2,393,395,200 allocated bytes. Remaining disk space was
186,860,761,088 bytes on `/dev/sdd`, recorded in `disk-after-cleanup.txt`.
All logs, failed diagnoses, tested source snapshots and their hashes remain.
No integrated implementer worktree belonged to this child; the shared feature
worktree remains with Root for integration. Root owns the final source checkpoint,
repaired assignment gates and complete native proof before a later explicit
production-switch decision. This child changed no production state.
