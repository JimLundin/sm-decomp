# Establish current batch cost and honest reuse timing

Status: claimed
Blocked by: None
Triage: ready-for-agent
Type: task

## Scope and ownership

Own batch/verification phase instrumentation and its public behavior tests in
`pipeline`, `offline-verification` and `tool-execution::metrics`, plus this effort's
measurement configuration/results. Reconcile the active extraction before edits;
freeze its integrated source and exact tools. This proposal is not a live claim.

## Required work

Record planning, input authentication, compiler/cache work, shared-capacity wait,
Fixture execution/comparison, evidence publication/admission and cleanup separately.
Fix the reused `FamilyRecord.seconds` problem: retain historical run time as such
and measure current reuse authentication independently. Include failed and resumed
work and label summed producer duration separately from elapsed time.

Enable existing detailed metrics with `SM_PIPELINE_METRICS=enabled`; measure
sampling overhead and keep cheap stage timing distinct from expensive probes.
Record source/tools/profile, coverage counts, cache state, byte/file counts,
resource bounds and host activity. Specify bounded workloads and the minimum useful
improvement before comparing candidates, following [validation](../validation.md).
Begin with small representative controls; defer full catalog runs until required.

## Acceptance

Public batch controls distinguish fresh, reused, rejected and interrupted work;
reuse time is measured on this invocation. Required stages have complete monotonic
timings or explicit unavailable reasons. Baseline scenarios and finite budgets are
reproducible and preserve their required domain. Historical disabled metrics and
the stripped-plan timeout are not reused as a successful performance baseline.

## Completion evidence

Pending. Record exact commands, nonzero test counts, baseline samples, noise and
limits, both scoped cleanup passes, bytes/free space and retained dependencies.

## Comments

The first implementation task once source ownership and baseline are available.

2026-10-15: claimed by session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Provisional source is the clean extracted commit `8bf8acf610e1a94a1a3bccd5d718dae4a4244857`;
worktree `/home/jim/sm-decomp-performance-metrics`, branch
`performance/batch-metrics-20261015`. Final acceptance waits for the extraction's
pending frozen-source verification and cleanup. No installed policy is modified.
