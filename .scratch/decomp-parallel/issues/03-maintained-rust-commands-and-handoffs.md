# Maintain Rust session commands and durable handoffs

Status: open
Blocked by: 01
Triage: ready-for-agent
Type: task

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

Append full commits, exact CLI examples, schema/compatibility checks, test
receipts, a representative handoff and retained dependencies. Complete scoped
[run cleanup](../../../docs/agents/run-cleanup.md) with measured reclamation/free
space and cleanup limitations before resolving.
