# Compare recommendations with committed optimization work

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

Compare the prior speed/parallelism/accuracy review against committed branch
`decomp-pipeline-performance` at `e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`.
Read its Git objects from the decomp checkout; preserve the optimization
worktree and branch. Distinguish implementation, measured benefit, remaining
work, and integration/accuracy validation required for current decomp sources.

Findings: `../optimization-comparison.md`, with supporting measurement review
at `../optimization-measurements.md`.

## Answer

Most first-wave recommendations are implemented at `e820a6b`: shared leases,
compiler caches, narrowed input contracts, parallel fixture pairs/private live
stores, single-pass authenticated inventories and shared import. Prioritize
integration with current dependent WRAM/host chains and native WithGfx ownership
gates, followed by full-route verification and complete resource/wall timing.
Original-ROM accuracy work remains separate.

## Completion evidence

Inspected committed branch source and checked-in measurement/verification
records using read-only Git objects from the decomp checkout. No protected
physical worktree/run artifacts were read, and no new authentication is claimed.
The source index records exact commit and blob identities. Citation/JSON parsing
and whitespace checks pass. Reports distinguish study versus implementation,
capture versus end-to-end time, and frozen branch versus current decomp evidence.

Cleanup scope is documentation only: no replay/build/extraction/cache/worktree
was created; zero bytes reclaimed and no run directory requires utility cleanup.
Remaining disk space at completion check: 254,830,178,304 bytes. Retained evidence
dependencies and every worktree remain intact. Optimization branch remains
`e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`. Comparison committed on
`decomp-port-integration`; no merge was performed.
