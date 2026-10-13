# Maintain Rust session commands and durable handoffs

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Type: task
Owner: `/root/session_tools_finish`, thread `01a13474-5fcc-7861-84f8-5c353938bdd2`
Recovery coordinator: `01a13460-4aa9-7fe0-865e-a8f54ffca310`

The maintained Rust queue already reserves and records work. Ceres infrastructure
uses a local Rust wrapper and manual handoffs. Provide a supported interface for
those contracts so a fresh session can delegate and resume from files. Follow
the [runbook](../../../docs/agents/decomp-sessions.md); implement tooling and
tests in Rust with no new Python source or runtime dependency.

## Work

- Reuse existing assignment/coordination schemas and commands. Replace the
  legacy infrastructure wrapper where needed with typed maintained commands
  that preserve exact claim lineage, evidence seals and admission distinctions.
  Keep native gameplay scope validation intact when admitting infrastructure.
- Persist worker handoffs at
  `.scratch/decomp-parallel/sessions/SESSION/JOB-ATTEMPT.md` with actual session
  and worker identities, full claim tuple, approved files/baseline, worktree,
  full WIP/submitted commits, runs and process command/start identity, evidence
  hashes/dependencies, stages/limitations, cleanup state and exact next command.
- Make inspect/start/resume workflows executable and bounded. Scoped WIP
  commits happen before long checks and at completed compile/experiment
  boundaries; main Git/index checkpoints remain with one integrator. Keep bulk
  logs on disk and return compact pointers to avoid oversized session output.
- Integrate the authority/transfer contract from ticket 02 after disjoint
  interface development. Update the runbook's commands only when supported and
  validated; keep the future ledger unactivated until ticket 04.
- Verify main-checkout and worktree inspection, interrupted result publication,
  exact retries, stale attempt rejection and reconstruction of the next command
  from durable files. Reuse existing tests and add controls for new behavior.

## Acceptance

A fresh session can inspect the same authority from any assigned worktree,
launch actual work through a subagent, identify the latest scoped commit and
resume a reviewed command from its handoff without transcript reconstruction.
Supported infrastructure commands preserve legacy contracts. Tests exercise
interruption and rejection behavior; no claim is implicitly released by age.
Ticket 02's final authority/transfer contract is integrated and validated before
this ticket resolves, while disjoint interface development may start beside it.
Two actual concurrent sessions remain ticket 04's acceptance gate.

## Completion evidence

Session source commits through `9a5516c2f1a930b7e960daf60694694c410d3990`,
retired submission retry `599d2b8`, typed integration tests `05a165a`/`a338409`
and engine baseline propagation `0b46487` are integrated in frozen source
`c8799f326ffe198b652301f08866f1b7a7bd9ebc` on
`decomp-preparation-20261013`. The [runbook](../../../docs/agents/decomp-sessions.md)
contains maintained CLI routes and a concrete inspect request. The
[durable session handoff](../preparation/03-session-handoff.md) and
[integration regression](../preparation/cutover/integration-handoff-test.md)
retain exact source, scope, claim, process, evidence and next-command contracts.

Final combined verification passed 15 session controls, 23 queue and 10 authority
controls; the existing native mandatory-facts test separately passed without
gameplay execution. Tests cover actual CLI parsing, exact legacy request-row
admission, checkpoint intent/publication interruption, scoped Git checks,
real-child Start retry, checked Adopt, typed generated-integration Start/Resume,
successful nonzero infrastructure gates, retired-worktree submission retry,
stale/tampered evidence rejection and original native admission. Ticket 04 owns
actual independent sessions. No Python source/runtime was added.

Both cleanup passes succeeded, 0 bytes. Session and typed-integration private
targets reclaimed 1,884,150,932 and 1,218,133,244 bytes. Integrated clean worktrees
were retired after small evidence retention at original paths without Git/build
and main `03-session-evidence`/`integration-handoff-evidence`; original and
relocation manifests reauthenticated. Final combined target cleanup is recorded
under ticket 02. Free space observed afterward: 223,381,778,432 bytes. Source,
bounded test/cleanup receipts and immutable legacy evidence remain retained;
production canonical authority and original claim lifecycle are unchanged.

## Comments

2026-10-13: checked maintenance adoption replaces the directly observed
completed, clean preparation worker `01a1342e-6cad-7cb1-80d7-6a484c469bbf`.
The replacement owns only the approved session/CLI/native adapter/test/runbook
paths in `/tmp/sm-decomp-preparation-session-tools-20261013`, baseline
`deceefdb568990fe273cc3f6cdf849726d8380bb`. First WIP source commit:
`c98e1e4d099fb29eff0ea029c72f75dba212bf1f`. Final authority integration and tests
remain required; original gameplay admission is retained. The main checkout's
`../preparation/03-session-handoff.md` records exact recovery and next actions.
