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

## Production command

The checked production migration is active. Run the installed controller from
a normal shell. No active Codex session is required:

```sh
cd /home/jim/sm-decomp
/home/jim/sm-decomp-production-execution/bin/sm-pipeline \
  translation-production --checkout "$PWD" run
```

Replace `run` with `status` to inspect the sealed production registry. Repeating
`run` recovers the same attempt. It does not repeat a failed gate or request
another model response. `FOUNDRY_TOKEN` is needed only when an approved attempt
reaches its model-backed rewrite. Existing submissions bypass rewriting.

The migrated Ceres host and controller keep their original submitted commits,
claim attempts and linked integration. Host precedes controller. Their original
Cargo, material callback and diagnostic acceptance still requires confined
adapters; the current contracts retain an authenticated prerequisite stop and
publish no successful stage. Pause equipment rendering remains explicitly held
until its complete 607 actual-C pairs and 10 Rust tests have a composite adapter.
Its already-merged source is preserved. Activation does not complete these gates.

The registry at `.scratch/translation-execution/production.json` binds both
original ledgers, installed policies and the ordered plan. The marker at
`.scratch/decomp-parallel/deterministic-execution.json` fences older unrestricted
dispatchers. A crash after the marker but before registry publication requires
`translation-production --request REQUEST.json recover`; both paths stay fenced
until the exact checked activation finishes. Do not delete the marker to resume
an old dispatcher. Restoring an older orchestration policy requires a separately
checked ownership transfer with stopped supervisors and preserved evidence.

New acceptance adapters and successor scopes require fresh reviewed policies.
The current migration does not authorize reducing original verification or
rebasing a successor silently after another scope completes.

## One-command Foundry launch

The maintained `translation-launch` command owns an ordered plan of approved
Translation scopes. It installs sealed configuration, prepares each scope,
invokes the model, runs the prescribed gates, integrates, publishes and cleans
up. The model returns Source replacements; it chooses no pipeline operation.

A concrete local request is prepared in the architecture worktree. From
`/home/jim/sm-decomp-translation-execution`, run:

```sh
./out/translation-launch/bin/sm-pipeline translation-launch \
  --request "$PWD/.scratch/translation-scope-execution/launch-foundry.json" run
```

This configuration runs the approved closed `enemy-jump-instructions` scope:
two routines and 16 actual-C Fixture pairs, with candidate and merged focused
and Offline gates. Its source, ROM, recovered C, complete closed catalog,
coverage, provenance and budgets are frozen. It retains acceptance under
`model-launch/operator-run-01`. The production queue migration is recorded in the
[switch ticket](../.scratch/translation-scope-execution/issues/08-production-switch.md).
Original-ROM parity and Live replay retain their separate gates.

The request uses the existing `FOUNDRY_TOKEN` and the Foundry Responses endpoint,
with frozen model `gpt-6.1-sol`, reasoning effort `high`, 32,768 output tokens,
a 300-second model deadline and a 1-MiB wire response limit. Credentials are
read only when a new exchange starts and passed to the trusted HTTP helper.
They are excluded from requests, retained evidence and diagnostics. The helper
sends `tools: []`, `tool_choice: "none"`, strict Source replacement JSON Schema,
and `store: false`. It uses one fresh HTTP/1 connection, disables redirects and
proxy discovery, and makes no automatic retry. Non-success gateway bodies are
discarded because they can contain authorization diagnostics.

The [Responses request](https://developers.openai.com/api/reference/resources/responses/methods/create)
and [Structured Outputs](https://developers.openai.com/api/docs/guides/structured-outputs)
contracts are implemented directly in Rust. No Codex session, model-selected
command or language runtime orchestrates the launch.

`translation-launch-request-v1` supplies an exact helper Reference, provider,
output root and ordered scope-template References. Preparation copies the
approved policy, original claim and verification contract into a new sealed
Installation. The template installer accepts isolated scopes; final managed
Installations use their separately reviewed migration and `--plan`. The
`translation-launch-plan-v1` records exact Installation References and any
earlier `after` dependencies. The existing supervisor still checks all managed
ownership and admission. Each scope uses its declared frozen source and
integration baseline; an ordering dependency does not rebase its policy.

```sh
./out/translation-launch/bin/sm-pipeline translation-launch \
  --request "$PWD/.scratch/translation-scope-execution/launch-foundry.json" inspect
./out/translation-launch/bin/sm-pipeline translation-launch \
  --plan "$PWD/.scratch/translation-scope-execution/model-launch/operator-run-01/plan.json" \
  repair enemy-jump-instructions 'reviewed repair reason'
```

`prepare` installs and authenticates configuration without a model or gate
producer. `run` advances in declared order and stops at the first unfinished
scope; a pending or stopped run exits nonzero after its typed report. Repeating
`run` reconciles the same attempt. `inspect` authenticates read-only.
`repair SCOPE REASON` deliberately starts a new rewrite attempt; `retry` runs a
new evaluation of the retained candidate; `cancel` stops the first unfinished
scope. All three preserve the supervisor's frozen attempt budget. Continue the
ordered plan with `run` after the deliberate action completes.

An immutable model intent precedes sending. The helper inherits the operation
lease and owns deadline enforcement and publication after a controller crash.
Recovery consumes its original sealed response or records ambiguous delivery;
it never resends that intent. A complete fsynced wire response can be recovered
before source application. Changed endpoint, model, request, capsule or helper
identity refuses reuse. Stopped attempts require deliberate repair.

The [Foundry/actual-C checkpoint](../.scratch/translation-scope-execution/model-launch/checkpoint.md)
records the passing CLI lifecycle, ordered-plan/recovery tests and cleanup.
The installed CLI is retained independently of disposable Cargo targets; use
that exact binary for this prepared request.

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
fenced managed admission. New rewriting currently supports original Running
native Implement claims with complete plain `family-verify` declarations,
original NativeScope exercise provenance, prescribed coverage and conservative
catalog preparation. Imported submissions retain their original source and
bypass rewriting; linked integration retains its original claim. The launcher
refuses Submitted/Integrate rewrites, unsupported
legacy source fingerprints, Live replay, batch and specialized native recipes.
Infrastructure evidence uses the explicitly authenticated
`reviewed-in-process-rust-tests` acceptance model and is not accepted as gameplay
or original-ROM parity.

Production uses the original-lineage installations and checked ownership
recorded in the [migration checkpoint](../.scratch/translation-scope-execution/production-migration/checkpoint.md).
The user authorized that switch on 2026-10-14. Future scope admission or adapter
changes require their own frozen policies and exact verification contracts.

Original production acceptance has a separate reviewed preparation path:

```sh
sm-pipeline translation-original-prepare --config /absolute/PREPARATION.json
/absolute/OUTPUT/bin/sm-pipeline translation-original-install --config /absolute/OUTPUT/original-production.json
/absolute/OUTPUT/bin/sm-pipeline translation-production --checkout /home/jim/sm-decomp run
```

The preparation config seals the original registry and stopped installation,
reviewed source HEAD, helper, native compiler, matching Cargo/rustc pair, vendor
closure, optional external-dependency cache, original requirements and auxiliary
pause animation plan. The Rust builder installs a new controller outside the
checkout and freezes the complete current conservative Offline catalog. Keep
the old installed executable: its prerequisite stop requires that exact reader.
Preparation itself establishes no acceptance.

The separate `.scratch/translation-execution/original-production.json` marker
binds the reviewed continuation. Each successor installation is created only
after the previous scope's completed source and queue publication authenticate.
Original Submitted source bypasses rewriting; a held Running integration keeps
its owner and attempt. A different original submitter remains on the Submitted
claim while the current fenced integrator owns the derived integration.

Use `status` for read-only inspection and `recover` for interrupted execution.
An unchanged stopped or completed result reuses its retained journal without a
new attempt, gate or capture. A changed installation is refused; it cannot
silently replace an existing reviewed marker or retry a frozen stopped attempt.
After preparation, keep new logs outside the checkout: the continuation seals
the exact pre-existing modified and untracked `.scratch` inventory as well as
every original ledger. Infrastructure controls, compact diagnostics, gameplay
Offline verification, Live replay and original-ROM parity remain distinct.

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
post-retirement authentication. The later production activation and its retained
acceptance blockers are recorded in the migration checkpoint above.
