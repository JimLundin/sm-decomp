# Measure the native Rust pipeline and select its next upgrade

Status: claimed

## Session scope, confirmed 2026-10-12

Complete optimization and improvement of the decomp pipeline itself. A separate
session continues actual decomp and gameplay porting afterward. This effort owns
native tooling, orchestration, measurement, bottleneck improvements, evidence
handling, verification and cleanup. Existing gameplay routes are regression
workloads for pipeline changes; discovering or implementing new translated
routines, gameplay families or porting frontiers is outside this effort.

Completion delivers the integrated, verified pipeline improvements and their
measured limits, with retained evidence and cleanup complete, for that separate
decomp session to use. Binary transport remains conditional on measurement.

Authoritative plan: `plan.md`, rendered locally from `visual/plan.mdx`.
Committed optimization branch: `decomp-pipeline-performance` at `e820a6b`.
Implementation basis: active native Rust migration worktree, pinned in
`optimization-source-snapshot.toml` and assessed in `optimization-progress.md`.

First integrate the migration owner's delivered native Rust pipeline with current
decomp semantics, extend missing native metrics using typed models/TOML sidecars,
and measure/profile a controlled native text baseline. Tickets 01–03 define this
first phase. Record the next-bottleneck selection before binary implementation.

Tickets 04–08 are conditional on selecting binary: opt-in `sm-capture-v1`, native
C templates and Rust selection, exact `sm-evidence-v1` compatibility, real failure
controls, controlled benchmarks/full-route admission and promotion/retention.
No new Python tooling or duplicate native migration ports are proposed.

Implementation branch: `pipeline-upgrade-integration`, based on delivered
merge `3ad1bf3`. Current source and format reconciliation is recorded in
[reconciliation.md](reconciliation.md); absolute benchmark budgets are frozen in
[benchmark-budgets.md](benchmark-budgets.md). Ticket 01 is resolved; ticket 02 is
integrated into this worktree with scope/process/resource controls and fresh
regression admission; its telemetry overhead gate remains open. The integrated
support worktrees have been removed. Conditional
tickets cannot be claimed without the selection decision in 03.

## Planning evidence

Read committed optimization Git objects and, at the user's request, inspected
the active optimization worktree status/diffs/source/notes without mutation.
Planned contracts were
checked against Event/TraceWriter/literal scanning, generated host emitters,
private-store capture/publication, and shared lease behavior. Independent review
and local plan validation are recorded under `planning-evidence.md`.
