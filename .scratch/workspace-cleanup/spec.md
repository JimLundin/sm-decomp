# Workspace cleanup

Status: resolved

Remove obsolete integrated Git worktrees, stale registrations, disposable build
outputs and Python/linter caches. Preserve uncommitted and unmerged work, sealed
artifacts, diagnostic failures, acceptance pointers, shared evidence stores,
retained readers and exact source snapshots required by evidence.

Follow `docs/agents/run-cleanup.md`; record measured reclamation and remaining
evidence dependencies in the cleanup ticket.

Completed on 2026-10-11. See `issues/01-remove-obsolete-workspaces.md` and
`reclamation.json` for measured cleanup and retained dependencies.
