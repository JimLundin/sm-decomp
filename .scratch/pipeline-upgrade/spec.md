# Upgrade packed capture transport and measurement

Status: open

Authoritative plan: `plan.md`, rendered locally from `visual/plan.mdx`.
Baseline optimization branch: `decomp-pipeline-performance` at `e820a6b`.

Deliver an opt-in `sm-capture-v1` binary FIFO for typed PPU/APU events and exact
literal records, reusing `sm-evidence-v1`, private stores, leases and import.
Add complete process/stage telemetry and prove exact text/binary observation
compatibility. Promote only after current-decomp integration, correctness,
resource and controlled end-to-end performance gates pass.

Implementation is proposed. This planning task changes documentation only and
preserves the optimization worktree/branch. Tickets remain open for execution.

## Planning evidence

Read current source and pinned optimization Git objects. Planned contracts were
checked against Event/TraceWriter/literal scanning, generated host emitters,
private-store capture/publication, and shared lease behavior. Independent review
and local plan validation are recorded under `planning-evidence.md`.
