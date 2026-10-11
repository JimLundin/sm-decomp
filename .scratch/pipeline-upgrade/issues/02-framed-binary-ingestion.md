# Define binary framing and exact literal ingestion

Status: open
Blocked by: 01
Triage: ready-for-agent
Type: task

Reuse Event/TraceWriter/ingest_trace. Extract shared literal scanner and implement sm-capture-v1 decoder, strict header/tag/length/range checks, 40-byte END counters, EOF checks and collector legacy digest receipt. Preserve restored bytes, stream/observation IDs and legacy frame/line segmentation. Typed pushes require a clean shared scanner boundary; otherwise feed rendered bytes through pending partial/long-line state and finalize at END.

## Acceptance

Follow the exact contract and verification/rollout gates in `../plan.md`.
Record source revision, controls, measured results and scoped cleanup under
Completion evidence before resolving. Implementation is pending.
