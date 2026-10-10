# Exercise live family failure controls
Status: claimed
Blocked by: 04, 07
Triage: ready-for-agent

Test exact generated ABI guards, caught Rust failures, tampered offline artifacts
and a candidate that passes a narrow offline fixture but diverges when actually
executed in gameplay. Reject before live build on altered seal, and stop without
fallback on failed Rust calls. Add harness output/dependency identity support
needed by complete animation routines; retain all compiler/candidate diagnostics.
