# Measure native text baseline and choose next bottleneck

Status: open
Blocked by: 02
Triage: ready-for-agent
Type: task
Phase: first phase

Reuse native batch/live commands and add a Rust benchmark module. Freeze combined source, inputs, compiler/readers, absolute host budgets and equivalent cache/store warmth. Run three alternating text-baseline repeats at 1000/45000 inputs, capacity two/four on a quiet host. Separate cold builds/empty stores. Include publication, final authentication and scoped cleanup.

## Acceptance

Report median/range, full boundary, PID resources and source/configuration. Attribute the critical path with bounded CPU profiling where needed. Explicitly select binary only if text conversion/transport offers a credible total gain; otherwise record deferral and write a separate scoped plan for the measured cost. No binary ticket may be claimed before that decision.

Follow the complete contract in [plan.md](../plan.md). Record source revision,
controls, measured results and scoped cleanup under Completion evidence before
resolving. Implementation is pending.
