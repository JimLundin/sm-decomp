# Conditional binary framing and exact literal ingestion

Status: deferred
Blocked by: 03
Triage: ready-for-agent
Type: task
Phase: conditional: binary selected in 03

Reuse Event/TraceWriter/ingest_trace and implement sm-capture-v1 only after selection. Extract the shared scanner, strict header/tag/length/range checks, 40-byte END counts, EOF checks and legacy digest. Preserve bytes, stream/observation/segment identities and historical readers.

## Acceptance

Typed direct pushes require a clean scanner boundary. Otherwise feed rendered bytes through pending partial/long-line state; finalize at END. Preserve second-token frame inference, signed literals and segmentation. Implement meaningful Rust parser controls.

Follow the complete contract in [plan.md](../plan.md). Record source revision,
controls, measured results and scoped cleanup under Completion evidence before
resolving. Implementation is pending.

## Comments

2026-10-13 decomp swarm startup: ticket03 explicitly left binary transport
unselected and tickets04–08 deferred/unclaimed. Record that existing decision
in lifecycle status so automatic ready-ticket discovery honors this ticket's
binary-selection prerequisite. No binary work is claimed or completed here.
