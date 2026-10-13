# Preparation tooling interfaces

Status: proposed interface split; no production authority activated
Blocked by: 01 for implementation; 02 and 03 for final validation
Triage: ready-for-agent

Read-only analysis against preparation baseline
`deceefdb568990fe273cc3f6cdf849726d8380bb`. Follow [readiness](status.md) and
the existing [spec](../spec.md). These interfaces are proposals, not current CLI
capabilities. No build, claim, gameplay run or Git mutation was performed.

## Existing seam to retain

`assignment.rs` exposes `Request`, `Job`, `Claim`, `Artifact`, `Update`,
`WorkResult`, `Assignment`, `Receipt` and `operate(Request)`. It owns atomic
ledger/ticket publication, attempts, typed retries and stopped observations.
`coordination.rs::operate` derives native jobs and validates results before
calling that engine; its private submission/gate/authentication helpers retain
native gameplay admission. `main.rs` registers the `assign` CLI and stdin route.
The Ceres driver only parses `assignment::Request`, calls `operate` and prints
TOML; its sealed results establish ownership, not native gameplay acceptance.

## Ticket 02: engine authority boundary

Own `crates/pipeline/src/assignment.rs`, a new sibling
`crates/pipeline/src/authority.rs`, and
`crates/pipeline/tests/assignment_queue.rs`. Keep `Claim` and existing result
shapes unchanged so legacy tuple/artifact seals remain byte-compatible.

Minimal public additions for ticket 03 to consume:

```rust
pub(crate) struct SessionAuthority { pub session_id: String, pub generation: u64 }
// Request gains #[serde(default)] authority: Option<SessionAuthority>.
pub(crate) fn main_checkout(checkout: &Path) -> Result<PathBuf>;
```

`Request.coordinator` remains the persistent role token and claim owner. The
engine checks that token against real `session_id` and current generation on
every mutation, including identical retries. Store role/effort policy separately
from historical ledger entries; an absent/pending production authority rejects
`decomp-parallel` mutation. Preserve legacy requests under their explicit bridge
policy rather than silently creating fresh roles. Status remains read-only.

Use the existing global assignment lock to inspect all assignment-v1 effort
ledgers and admitted legacy holdings before reservation: overlapping path
prefixes, exact ticket identity, duplicate bridged claims, submitted ownership,
and active integration all count. A sidecar is role/cutover policy, not a second
queue. Recovery preserves original claim tuples; generation transfer rejects old
writers and requires sealed real-owner/child-stop evidence. Ambiguous or surviving
children retain their holdings. Expose typed role status/register/transfer/stage
operations from `authority.rs`; ticket 02 chooses their final shape
and supplies it before ticket 03's authority integration.

Ticket 01 observed only the Ceres ledger. Affected-family/decomp-port claimed
Markdown holds enter the bridge as exact ticket paths and writable resources;
retain their source/evidence identity without inventing claim tuples. Prepared
role registrations may exist while production authority remains pending; role
preparation alone grants neither gameplay start nor automatic queue activation.

## Ticket 03: maintained infrastructure and handoff adapter

Own new `crates/pipeline/src/session.rs`, new
`crates/pipeline/tests/decomp_sessions.rs`, the adapter changes in
`crates/pipeline/src/coordination.rs`, CLI wiring in `crates/pipeline/src/main.rs`,
and session-command examples in `docs/agents/decomp-sessions.md`. Develop
disjointly from engine edits; defer shared route/authority wiring until ticket
02 supplies its contract. No Cargo dependency or native `translation.rs` edit is
needed initially; new test modules can include the source like existing queue
tests. The adapter calls the same `assignment::operate` with ticket 02 authority.

Proposed bounded route: `sm-pipeline decomp-session OP --request REQUEST.toml`.
Use serde tagged request types, retaining original queue `Request` inside the
infrastructure envelope. Operations are infrastructure plan/next/record and
worker inspect/start/checkpoint/resume; resume returns reviewed argv/cwd and
liveness prerequisites rather than executing an arbitrary saved command.
Agent launch remains the caller's subagent tool; `start` records its returned
actual worker identity, never fabricates a session from `/root` or a PID.

Persist typed `Handoff` TOML beside the required rendered
`sessions/SESSION/JOB-ATTEMPT.md`. Fields: original `Claim`, real session/worker
identities and authority generation, exact approved job/files and baseline,
worktree/branch, full WIP/submitted commits, run/process command/start identity,
sealed `Artifact` evidence and dependencies, achieved stages/limitations,
cleanup and `NextCommand { cwd, argv, env }`. Atomic publication carries a
sequence/revision so identical retries succeed and conflicting rewrites fail.
Inspect/resume compares the current claim, job revision and source before
returning compact pointers; stale handoffs retain history and cannot launch.

Scoped `checkpoint` checks the assigned worktree and all dirty/staged paths
against exact approved files, commits only there, then publishes the new full
commit and next command. Reject an unapproved staged file before Git writes.
An interruption after the commit reconstructs that same commit from the owned
worktree instead of creating another. Main Git/index remains the integrator's.

Infrastructure publication must identify its admission domain explicitly.
Legacy import can retain original hash-validated receipts without relabeling
them. New infrastructure scope/result validation needs exact baseline/files,
reviewed bounded test gates, sealed results and cleanup; it must never accept a
native gameplay scope through a raw engine request. Native jobs keep the
`coordination.rs` admission path. Shared helpers can be extracted only in a
later sequential integration edit if duplication becomes material.

## Coordination and acceptance

Agree the additive authority field and typed role entry points before code
integration; ticket 03 may build the handoff/legacy envelope first. Ticket 02
owns engine locking and policy, ticket 03 owns command/handoff validation, and
ticket 03 wires `main.rs` sequentially after the engine contract. Reuse queue
tests; add focused two-
effort/generation controls for 02 and handoff retry/stale/scoped-commit controls
for 03. These tests prepare ticket 04's disposable real-session drill. Keep the
production queue staged pending and gameplay unlaunched until validated cutover
and the user's later launch instruction. Analysis produced no run outputs;
cleanup is not applicable to this step.
