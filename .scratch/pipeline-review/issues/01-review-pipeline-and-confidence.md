# Review pipeline throughput and simulation confidence

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

## Questions

- Which measured pipeline costs and avoidable repeats should we address first?
- How should implementation work and expensive verification be parallelized?
- Are the references pinned, authenticatable and suitable for the claims made?
- What stability and accuracy have been demonstrated, and what remains unproved?

## Acceptance

Write cited Markdown findings separating measurements, current capabilities,
proposals and uncertainty. Record recommended next checks and scoped cleanup.
Do not touch the optimization worktree or branch.

## Answer

See `../pipeline-review.md` for measured pipeline costs, priorities, benchmark
plan and a resource-aware parallel setup. See `../reference-and-accuracy.md` for
primary-source provenance and distinct C/Rust, causal original-ROM and recording
profile accuracy claims. `../environment-and-inventory.json` retains read-only
capacity and logical inventory counts under concurrent load.

## Completion evidence

Reviewed at decomp checkpoint `bb184a2` on 2026-10-11. No gameplay/pipeline
source or existing evidence was changed. Read-only primary-source investigation
was delegated using the research skill; no new verification replay/build,
extraction, caches or implementer worktree was created. Both reports distinguish
retained authentication from checks performed for this review.

Cleanup scope is this documentation review: zero run intermediates and zero
bytes reclaimed; there is no run directory requiring `port_cleanup.py` or live
reauthentication. Existing sealed evidence, stores, readers, source snapshots
and all worktrees are retained. Remaining disk space is recorded in
`../environment-and-inventory.json`. The optimization worktree was not entered,
and branch `decomp-pipeline-performance` remains at
`e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`.

Verified local Markdown citation targets, JSON parsing and whitespace checks.
Implementation and controlled speed measurements are proposed future work.
