# Reduce measured legacy trace restoration cost

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task

User-directed measurement revision replaces the hour-long telemetry tuning
cycle with targeted probes and shorter complete-pipeline controls. This ticket
targets repeated legacy rendering in evidence authentication; it does not
introduce binary transport or change gameplay.

## Acceptance

Preserve canonical objects/stream identity, exact decimal legacy bytes, event/
line/byte counts, corruption rejection and legacy SHA-256 authentication.
Measure the real retained-stream path before/after and complete a controlled
whole-pipeline comparison. New renderer must use bounded buffers and no unsafe
code. Final-source full-route regression and cleanup remain integration gates.

## Measurements

Actual resource-sampler probe over700 snapshot files:100 RSS scans0.003s,
100 RSS+allocated-disk scans0.092s, median0.024/0.814ms. Fixture removed.
Sampler cost alone does not explain the long control's apparent overhead.
Two-million-event SHA-identical rendering probe:standard130.760/133.890ms,
bounded decimal64.462/64.697ms. Real retained24MB hardware stream restored
with all seals:baseline median442694984ns,candidate64429448ns, five repeats
each; same stream548782cb106543c0e52262e7ac07d3848092dc86b039886536387290003e61a3.
These are focused restoration measurements, not full-wall performance claims.
Evidence logs/receipts under `out/ticket-03/`.

## Verification

Independent standard-format decimal oracle passed before/after. All12 evidence
store controls, strict all-target evidence Clippy, fmt and whitespace passed.
Bounded88-byte line uses pinned existing itoa1.0.18; no canonical format changes.

## Completion evidence

Implementation`c86c946`, integrated`3b53c74`. Controlled short complete boundary
improved19.389% (two old/new repeats), preserving exact observations; focused
restoration85.4% faster. Final-source129,712-input route accepted, all nine IDs
and16/4/4allocations with zero refusal/fallback; canonical state/effects match.
Cleanup/dedup and integrated worktree removal completed, retained live four
subjects/1,479artifacts and all predecessors authenticated afterward.
[Final delivery](../evidence/final-delivery.md) records controls, exact evidence,
bytes reclaimed, remaining disk, retained sources/readers/stores and limitations.
