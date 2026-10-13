# Run parallel decomp sessions

Use this runbook when starting a decomp session, resuming after a crash,
or handing a worker to a replacement coordinator. The durable queue owns work;
the session launches subagents and records their results.

Read the [parallel plan](../../.scratch/decomp-parallel/spec.md) and
[ticket map](../../.scratch/decomp-parallel/map.md) for the next dependency-ready
work. Read the [checkpoint](../../.scratch/decomp-parallel/checkpoint-20261013.md)
for recovered source and retained evidence and the
[preparation status](../../.scratch/decomp-parallel/preparation/status.md)
for the current launch boundary and final validation subject. For the user's
later gameplay swarm launch, read
[the launch handoff](../../.scratch/decomp-parallel/preparation/launch-handoff.md)
and its actual final drill/readiness receipts before preparing production
authority. Follow [assignments](assignments.md)
for claim/result schemas, [translation progress](translation-progress.md) for
stage publication, and [run cleanup](run-cleanup.md) before handing work back.

## Inspect before claiming

1. Start in the main checkout and use the shared process-worker budget:

   ```sh
   cd /home/jim/sm-decomp
   export SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline
   export SM_PIPELINE_WORKERS=4
   git status --short --untracked-files=normal
   git worktree list --porcelain
   git log -4 --oneline
   sha256sum -c .scratch/decomp-parallel/checkpoint/archive-sha256.txt
   cargo run -p pipeline --locked --offline --bin sm-pipeline -- assign status --checkout /home/jim/sm-decomp --effort ceres-continuation --coordinator ceres-20261013 --capacity 3
   ```

   Until the plan records a completed authority cutover, the shared queue is
   `.scratch/ceres-continuation/ledger.toml`, with persistent coordinator token
   `ceres-20261013` and assignment capacity `3`. The command above inspects it
   without changing claims. Assignment capacity counts reserved/running jobs;
   `SM_PIPELINE_WORKERS` limits shared fixture/build execution and is a separate
   budget. Inspect other existing ledgers and claimed tickets for overlapping
   ownership before selecting writable paths. The proposed
   `.scratch/decomp-parallel/ledger.toml` becomes authoritative only after the
   plan's owner reconciliation and recovery/concurrency drill are recorded.

2. Inspect the exact sessions, agents and child processes named by each active
   claim and handoff through the available thread/collaboration tools. Record
   the real session/thread id alongside the registered agent identity; names
   such as `/root` alone can recur in independent sessions. Inspect processes
   writing to that claim's worktree and run directories. A controller crash
   can leave its children running.

3. Read each claim's generated assignment, local ticket, approved scope and
   last handoff. Identify its full `(id, revision, attempt, coordinator)`, source
   baseline, writable paths, branch, worktree and batch path. Compare pending
   revisions and stale-input markers with the returned queue status. Retain
   live or ambiguous owners' claims while reconciling their state.

Inspection is complete when every active claim has an identified surviving
owner or an observed stopped owner, and every intended writable path has one
owner. A new `.scratch/` effort shares the repository ownership boundary.
The maintained engine checks held paths, ticket identity and linked submissions
across existing efforts, including path aliases, under the shared mutation lock.

Preparation and gameplay launch are separate. Before the user's later swarm
launch instruction, keep production authority pending and inspect existing
claims read-only. At launch, follow the readiness package, recheck exact real
owners and source identities, and prepare/activate only the reviewed policy.
`decomp-authority validate-activation --request REQUEST.toml` checks the same
drill and legacy bridge as activation while returning `state = "prepared"`.
It requires a prepared policy; the preparation readiness namespace supplies a
private policy view while the host production authority and canonical ledger
remain absent.

## Claim and dispatch

Use the plan's current authority and integration policy. Reserve one job per
available subagent slot, launch its actual implementation through a subagent,
then record `started` with the identity returned by the launch. Read-only
research and independent subsystem implementation can run concurrently;
merged-source verification and shared registration belong to one integrator.

For native verification or a reviewed `translation-scope-v1` job, use the
documented `assign plan`, `assign next` and `assign record` commands. For
example, after setting `DECOMP_BASE` to the ticket's full approved commit:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- assign plan --checkout /home/jim/sm-decomp --effort ceres-continuation --coordinator ceres-20261013 --capacity 3 --changed-since "$DECOMP_BASE" --no-translations
```

The mutable examples below are for the observed owner admitted by the current
coordination policy. Managed mutations additionally require that real caller's
registered role and current generation. Register distinct dispatcher roles
through `decomp-authority` after reviewed preparation; give each launched child
the exact returned claim. `--no-translations`
controls new research discovery; existing queued work can still be selected.

`plan` is read-only. After reviewing that selection and confirming a free
subagent slot, reserve work and inspect the returned `queue.assignment`:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- assign next --checkout /home/jim/sm-decomp --effort ceres-continuation --coordinator ceres-20261013 --capacity 3 --changed-since "$DECOMP_BASE" --no-translations --available-slots 1
```

After launching that assignment, prepare the exact typed `started` update from
[assignments](assignments.md) in `UPDATE.toml`, then record it:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- assign record --checkout /home/jim/sm-decomp --effort ceres-continuation --coordinator ceres-20261013 --capacity 3 --update UPDATE.toml
```

A researched gameplay scope uses `--scope SCOPE.toml`; the maintained schema
currently supports 1–32 coherent routines with exact permitted gameplay source,
test and manifest paths. Select whole coherent subsystems and their dependencies
within that contract. Use [assignments](assignments.md) for scope promotion and
result admission.

Existing Ceres infrastructure jobs were published through the Rust
`.scratch/ceres-continuation/assignment_driver.rs` wrapper. The maintained
`decomp-session` adapter replays only exact original requests listed as requests
in the authority policy's sealed legacy bridge. Its typed envelope names the
matching operation (`infrastructure-plan`, `infrastructure-next` or
`infrastructure-record`), the original `queue` request plus current optional
authority, and `admission = { domain = "legacy", original_request = { path,
sha256 } }`. Original Job/Claim/Update/result bytes remain unchanged; receipt
artifacts and arbitrary bridge evidence cannot authorize a request. Native
gameplay uses `assign` and its existing scope/result validation.

Run a reviewed envelope with the matching operation:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- decomp-session infrastructure-record --request REQUEST.toml
```

New infrastructure admission uses `domain = "infrastructure"` with sealed
`scopes`. A `decomp-infrastructure-scope-v1` artifact declares full baseline
commit, an implementation Job with exact pipeline source/test or docs files,
submission and integration `cargo test --locked --offline` argv/timeouts, and
limitations. Jobs derive from those seals; raw caller jobs are rejected.
Submission gates have at most 60 seconds, integration gates at most 3600.
The adapter requires an exact scoped source commit, runs nonzero tests,
seals stdout/stderr and source identity, and rechecks cached publication on
retry. Integration also requires complete sealed cleanup and retired
implementer worktree. These receipts describe infrastructure acceptance.

For an implementation receipt, use exactly its returned workspace and baseline:

```sh
git worktree add -b "$DECOMP_BRANCH" "$DECOMP_WORKTREE" "$DECOMP_BASE"
```

Set those variables from the approved receipt and its `base-head` input. Each
worker gets its own branch, worktree, batch/run directory and handoff, plus the
shared pipeline settings. Give it the exact writable files, gates, routine
scope, dependency evidence and experiment bounds. The main checkout's Git/index
has one owner; workers commit only in their assigned worktree. The integrator
owns shared catalog/CLI registration and inventory publication.

Dispatch is complete when the ledger stores the returned worker identity and
the worker has a durable handoff. A definitely failed launch uses
`launch-failed`; an ambiguous launch keeps its reservation until checked.

## Checkpoint each worker

Write a `decomp-worker-handoff-v1` typed TOML in a reviewed `start` request.
`decomp-session start --request REQUEST.toml` records the actual launched child
UUID in the existing claim, then atomically publishes authoritative TOML and
rendered Markdown at `.scratch/decomp-parallel/sessions/SESSION/JOB-ATTEMPT`.
The handoff's `agent` is a display name; `worker_session_id` is the actual queue
owner. `session_id` and `authority_generation` name the current dispatch role.
If publication stops after `started`, retry the identical request against the
durable running child. Inspect the child before any launch. The ledger remains
claim authority.

Use `decomp-session checkpoint --request REQUEST.toml` before a long gate,
after each completed gate, and before handing work back. The request contains
the next sequence handoff and bounded commit `message`. Checkpoint validates
every dirty/staged and baseline-to-HEAD file before staging only exact approved
files in the assigned isolated worktree. A retained checkpoint intent recovers
an interrupted Git commit without creating another commit. Identical handoff
retries repair rendered Markdown; changed same-sequence retries fail.

Each handoff records:

- Session/thread id, registered worker identity, coordinator and the exact
  claim tuple; ticket, approved scope, writable files and baseline commit.
- Branch/worktree, latest full WIP or submitted commit, batch/run paths,
  active process ids, command and start metadata when known, and the latest
  observed process state. Recheck that identity after a crash because a PID
  can be reused.
- Achieved stages and their limitations; exact passing and failing evidence
  paths with SHA-256 hashes, required frozen source, shared evidence objects,
  subject pointers, readers and private input dependencies.
- Exact next command and working directory, unresolved failure, current source
  changes, cleanup result and remaining cleanup action.

Make scoped WIP commits at those phase boundaries so completed source survives
a lost session. Stage exact owned files; exclude ROMs, captured gameplay,
build trees and bulk run output using the existing retention rules. A WIP
commit preserves source; publish `translated`, `verified`, `integrated` and
`live-exercised` independently only with their required evidence. The main
Git owner checkpoints queue changes, tickets and handoffs in scoped commits.

A checkpoint is complete when another session can locate the claim, its exact
source, retained evidence dependencies and next command from disk.

Inspect from any assigned worktree using a typed `inspect` request with `queue`
and optional absolute `handoff` path. `resume` requires the same exact current
credential, source commit, claim attempt/revision and sealed evidence, and
returns reviewed argv/cwd/environment plus liveness prerequisites. It executes
no saved command. The caller inspects actual owners/process start identities,
then delegates or runs that command. Reviewed commands use locked/offline cargo
tests and the shared workers4 budget. Replies retain compact durable pointers.

A read-only inspection request has this minimal form; set `checkout` to the
main checkout or assigned worktree and `effort` to the current ledger:

```toml
operation = "inspect"
[queue]
operation = "status"
checkout = "/home/jim/sm-decomp"
effort = "ceres-continuation"
coordinator = "ceres-20261013"
capacity = 3
```

Run it with `sm-pipeline decomp-session inspect --request INSPECT.toml`.
Mutation request files additionally carry `[queue.authority]` with the calling
real `session_id` and current `generation` from `decomp-authority` status.

After a checked role transfer, a surviving child retains its original claim.
Use `adopt` with current `queue` authority, the old absolute `handoff` and the
sealed `transfer` artifact recorded by the current role. The adapter checks
the exact old role generation and observed surviving child, then writes a new
session handoff whose dependencies retain the old handoff/transfer/child proof.
The original handoff remains immutable. Resume from the returned new pointer.

## Recover a crash

1. Repeat startup inspection. Surviving workers retain their original claim,
   scope and process budget. Recovering a parent does not release its children.
   Resume their monitoring and result recording rather than launching duplicates.
   Submitted work continues through integration with its writable ownership
   retained; its stopped implementer is not a reason to release that submission.
2. For an actually stopped owner, preserve and commit its scoped source and
   retain diagnostic evidence first. Record the liveness check, including its
   real thread identity and any finished/stopped verification processes, in the
   handoff. If owner inspection is unavailable, retain ownership until that
   inspection can be completed.
3. Seal the supported stopped-observation TOML with the stored `agent`, original
   `coordinator`, exact `[claim]`, and observed status `completed`, `failed`,
   `interrupted` or `not-launched`. Omit `agent` only when the stored claim never
   acquired one. Hash it and record `event = "owner-stopped"` with
   `observation = { path = "/absolute/path", sha256 = "..." }` under the
   original claim's coordinator token. Use [assignments](assignments.md) and
   the typed schema rather than changing `ledger.toml` by hand.
4. Inspect the resulting status before requesting a replacement attempt. Give
   that attempt its newly returned workspace and claim identity; recover useful
   source through scoped commits. Keep old evidence and attempt history intact.

The queue has no time-based claim expiry. It checks the sealed observation's
identity and allowed stopped status; the agent must establish real liveness
through the tools. A lost response permits an identical retry only while the
same attempt and input revision remain current. Changed source, revisions or
attempts require reconciliation before accepting old results.

## Integrate and finish

One observed live session owns each integration role, and one session owns the
main Git/index. Record both ownership boundaries in the handoff. Managed
integration uses an explicitly registered integrator token, real session id and
generation; dispatcher submission cannot acquire it. Keep `ceres-20261013` for
existing Ceres lineage. A checked integrator transfer accounts for every held
claim, child and writer, retains live/submitted ownership, and fences old
generations on every mutation and result retry. Adopt an existing typed handoff
before resuming its exact source and reviewed command.

The integrator admits the exact scoped submission, freezes merged source,
verifies all affected Offline families and required gates, publishes achieved
stages, completes both cleanup passes, and records authenticated completion.
Independent workers continue in their own worktrees while integration runs.
Infrastructure integration validates baseline-to-submitted-commit writable
ownership while retaining clean exact shared merged HEAD as the integration
source. This permits sequential disjoint merges without assigning the first
submission's files to the second worker. Keep the exact submitted claim/source
and frozen merged source distinct in typed handoffs and evidence.
Completed infrastructure result retries authenticate the original sealed
historical source/gates/cleanup and exact completed claim lineage. Later clean
merged HEAD may descend from that source; replay preserves original acceptance
and executes no gate. Running/new integration still binds frozen current source.
Shared pipeline/tools edits can invalidate source fingerprints even when
gameplay paths are disjoint; freeze the verification checkout, inspect stale
jobs and repeat affected verification when inputs change.

Follow [run cleanup](run-cleanup.md) for measured reclamation, source retention,
integrated worktree removal and post-cleanup authentication. Preserve failed
diagnoses as well as successful receipts. Cleanup starts after all processes
writing to its selected directories finish or stop. Local Git commits preserve
the plan and authored source; replay recovery on another machine also requires
the retained evidence store, readers and private inputs listed in the handoff.

Completion requires the durable integrated result, stage observations,
authenticatable retained evidence, cleanup records and a clean scoped Git
checkpoint. The checkpoint's Ceres host/controller observations are historical;
inspect current live status and the launch handoff for the same held claims.
Preparation resolves 01–05 only after final drill/readiness/cleanup receipts;
spec/map and gameplay 06–20 retain their own unresolved gates. Production
activation and gameplay launch require the user's separate launch instruction.

## Session prompts

Fresh implementation session:

> Work in `/home/jim/sm-decomp`. Follow `AGENTS.md` and
> `docs/agents/decomp-sessions.md`. Read the parallel spec and ticket map, inspect
> shared claims and actual owners, then select the first dependency-ready scope
> permitted by the current authority. Run actual work through subagents in
> isolated claimed worktrees. Persist each claim, exact identity, scoped WIP
> commit, evidence and next command in its handoff. Use the shared pipeline
> budget. Hand results to the observed integration owner and complete cleanup.

Replacement coordinator session:

> Resume `/home/jim/sm-decomp` from the parallel checkpoint and session handoffs.
> Follow `docs/agents/decomp-sessions.md`. Inspect the original claims, real
> sessions, surviving children and run processes before launching work. Continue
> live owners' claims; recover stopped owners through checked observations under
> their original coordinator token. Use the exact retained source/evidence and
> next commands. Reconcile the integration owner before touching shared source
> or the main Git/index, delegate actual work, and checkpoint the resulting state.
