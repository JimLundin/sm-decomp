# Exercise live family failure controls
Status: resolved
Blocked by: 04, 07
Triage: ready-for-agent

Test exact generated ABI guards, caught Rust failures, tampered offline artifacts
and a candidate that passes a narrow offline fixture but diverges when actually
executed in gameplay. Reject before live build on altered seal, and stop without
fallback on failed Rust calls. Add harness output/dependency identity support
needed by complete animation routines; retain all compiler/candidate diagnostics.

## Completion evidence

c3386a7 merged in 30a349c. Four public live tests exercise tampered accepted
artifact rejection before build, actual generated ABI null/extent/overlap/id
rejection and caught panic, and optional real gameplay wrong-candidate control.
The wrong ClearUnusedOam candidate passes its declared empty-loop offline case
but diverges in a 200-frame live replay; the invoked integration test passed.
Nine family tests additionally verify typed/captured inputs and banked pointer
results. Source dependencies and C diagnostic output are retained and validated.
