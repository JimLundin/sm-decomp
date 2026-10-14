# Deterministic Translation scope execution

The `translation-execute` command owns one approved Translation scope through
preparation, restricted rewriting, source application, verification, integration,
publication and cleanup. Its installed Rust controller selects every operation.
The rewrite worker receives a Rewrite capsule and returns Source replacements
for approved Rust files. It has no host tools, authority, ledger or evidence
access. Tests, manifests and verification declarations stay frozen.

The implementation lives in `crates/translation-execution` and the pipeline's
`translation_execution`, `translation_native`, `translation_infrastructure` and
`translation_migration` modules. The local implementation contract and evidence
are in [.scratch/translation-scope-execution](../.scratch/translation-scope-execution/spec.md).

## Installed operation

A reviewed, controller-owned `translation-supervisor-installation-v1` JSON file
binds the full original claim, baseline commit, approved-file hashes, native or
infrastructure contract, installed binaries, protected resource roots, workers4
capacity, deadlines and aggregate byte limits. Runtime checks authenticate its
actual bytes and executable identities before admission. Relative or aliased
roots, overlapping writable resource roots and unavailable enforcement refuse
launch. A historical completed proof is evidence for its own installed scope.

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
  sm-pipeline translation-execute --installation /absolute/INSTALLED.json run
```

A production installation uses the exact current original queue owner and
fenced managed admission. New execution currently supports original Running
native Implement claims with complete plain `family-verify` declarations,
original NativeScope exercise provenance, prescribed coverage and conservative
catalog preparation. It refuses Submitted/Integrate rewrites, unsupported
legacy source fingerprints, Live replay, batch and specialized native recipes.
Infrastructure evidence uses the explicitly authenticated
`reviewed-in-process-rust-tests` acceptance model and is not accepted as gameplay
or original-ROM parity.

Production must not be activated by copying a proof policy. Prepare original
lineage, review exact supported per-claim installations, reconcile real owners
and current authority, then obtain the separate explicit switch decision in the
agreed spec. This feature branch has activated no production migration.

## Recovery and deliberate evaluation

```sh
sm-pipeline translation-execute --installation /absolute/INSTALLED.json inspect
sm-pipeline translation-execute --installation /absolute/INSTALLED.json resume
sm-pipeline translation-execute --installation /absolute/INSTALLED.json retry 'reviewed reason'
sm-pipeline translation-execute --installation /absolute/INSTALLED.json repair 'reviewed reason'
sm-pipeline translation-execute --installation /absolute/INSTALLED.json cancel 'operator reason'
```

`inspect` reads an existing journal and authenticates accepted history without
initializing, allocating, repairing projections or executing a producer.
`resume` reconciles the current attempt's durable side effects. A completed or
stopped attempt returns its original result; resume cannot start another
candidate evaluation. Missing exact publication projections can be repaired
only from the accepted result. A completed managed publication with a missing
`accepted.json` can promote its surviving `accepted.pending` only when the actual
bytes match the retained acceptance Reference, canonical record, completion and
full source/queue/evidence graph. Partial, changed, linked or aliased pending
files are preserved and refused. `inspect` remains read-only and refuses that
missing projection. Tampered evidence and ambiguous writers retain ownership
and require explicit recovery.

`retry` deliberately reevaluates the last failed candidate without rewriting.
`repair` creates another recorded attempt and requests changed source from the
retained failed candidate. Each requires a reason, authenticated failure and a
remaining frozen attempt slot. Gate failure stops the current attempt after
writer teardown and bounded diagnosis. Cancellation during finalization or
publication preserves that already selected disposition and finishes recovery.

Every side effect has a durable intent and exact sealed result. Candidate and
integrated commits remain distinct. Revision archives retain authenticated raw
Git objects so failed-candidate retry, integration and historical inspection
survive retirement of producer checkouts. Host source publication uses a checked
index/HEAD transaction and exact queue/ticket/progress projections under one
fenced owner.

Managed ticket recovery authenticates the exact normal publication sequence,
including skipped projections after an interruption. It preserves the original
human prose and refuses unrelated ticket edits. An original Running claim keeps
its original integration-owner state until normal submission installs the
current fenced integrator.

## Resource and verification boundaries

Linux namespaces expose individual declared inputs and installed runtime files.
The rewrite worker and candidate compiler/runtime cannot reach protected host
roots. The runner enforces aggregate output and temporary-file extent while the
writer runs, tracks unlinked open files, limits stdout/stderr and terminates the
process tree on deadline, output overflow or unavailable descriptor inspection.
A hard-limit failure has bounded diagnostics and grants no acceptance.

The original stage deadline covers reservation traversal, runtime discovery,
chunked input hashing, setup and producer launch. Recovery of sealed successful
or failed producer results needs no runtime relaunch and does not extend that
saved deadline. Trusted post-teardown reconciliation has its own finite IO
budget. Unique retained bytes, journal/result metadata and repeated trusted
reads have separate bounded accounting.

The native adapter preserves actual-C Fixture pairs, Fixture chains and frozen
source/dependency identities. Only matching accepted evidence can be reused.
Unknown dependency changes leave an Offline family affected. A missing
inventory fact, exercise provenance, fixture coverage or harness input stops
before rewriting. Live replay and original-ROM parity retain separate gates;
the representative enemy-jump-instructions proof establishes neither.

## Read-only migration preparation

```sh
sm-pipeline translation-migration build-preparation --request /absolute/REQUEST.json
sm-pipeline translation-migration validate-preparation --migration /absolute/PACKAGE.json
```

The `translation-migration-build-v1` request supplies canonical main checkout,
original effort and claim IDs, exact stopped-owner observation References,
representative proof Installation/Completion References and a private output
root below this architecture worktree's `production-preparation/`. Submitted
source comes from each unchanged original result commit. Running integration
requires an explicit exact current committed HEAD. The builder writes only
`OUTPUT/package/`, retains original source/input snapshots and keeps migration
state `prepared`. It creates no production lock, claim or owner-stop event.

Identical complete or interrupted retries authenticate the original request,
authority, ledger, source blobs, captured inputs and output closure. Complete
retry returns byte-identical receipt bytes and runs no fresh validator. An
initial refusal remains historical; a separate later read-only report can
revalidate the same immutable package. Unavailable policies, overlapping held
ownership, specialized catalog recipes and unprepared claims remain explicit
blockers.

Builder defaults are 60 seconds, 64 MiB, 4096 files and 4 MiB per source file;
hard maxima are 300 seconds and 256 MiB. The existing preparation validator has
an explicit 300-second/8-GiB total read allowance for complete proof
reauthentication followed by the full catalog. Its unique retained-reference
closure remains at 1 GiB; these read limits change no producer output budget.

## Retention and validation

Follow [run cleanup](agents/run-cleanup.md) after successful, failed and
interrupted runs. Both cleanup passes, measured reclamation, retired integrated
producer checkouts and post-cleanup authentication are completion requirements.
Keep exact source snapshots, sealed operation/result/journal records, accepted
pointers, retained readers and referenced private inputs. ROMs, readers, build
trees and bulk output do not belong in Git checkpoints.

The maintained proof at
[actual-c-02-checkpoint.json](../.scratch/translation-scope-execution/supervisor-native/actual-c-02-checkpoint.json)
binds the complete native attempt, measured run, cleanup and retained reader.
The [original Ceres preparation](../.scratch/translation-scope-execution/production-preparation/preparation-checkpoint-27b8b0a.md)
records the actual three-claim package and its current switch blockers. Retained
proofs remain authenticatable independently of disposable build targets.

The complete [managed checkpoint](../.scratch/translation-scope-execution/managed-gates/actual-c-managed-11-checkpoint.json)
separately proves original claim preservation, standard submission/integration,
exact ticket/progress publication, pending recovery and tamper refusal through
an installed Rust protocol worker. It is a disposable diagnostic with synthetic
stopped-owner observation. [Final completion](../.scratch/translation-scope-execution/completion-20261014.md)
records affected checks, measured disposal and both retained proofs'
post-retirement authentication. Production preparation and the agreed separate
switch remain open.
