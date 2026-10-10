# 11: Automate the feedback trust demonstrations

**What to build:** An automated suite proves that the complete workflow accepts valid execution and refuses faulty, uncovered, stale or incomplete evidence.

**Blocked by:** 10: Assemble readiness gates, atomic acceptance and safe recovery.

**Status:** done

- [x] Correct execution produces a fresh identity-bound pass and can advance acceptance only with every required gate satisfied.
- [x] An executed incorrect compiled instruction fails and names the altered entry at the earliest observable state/bus-timing disagreement.
- [x] A transient wrong compiled effect restored later in the frame is caught by focused comparison even with matching frame-end bytes.
- [x] An added unexecuted key yields uncovered despite execution by older entries; empty compilation cannot pass.
- [x] Preflight/build errors, interrupted/truncated captures, incompatible identities, source/input contamination, stale prior verdicts and absent terminal completion cannot yield acceptance.
- [x] A valid paired checkpoint reproduces the corresponding cold trace/outcome; incompatible/corrupt checkpoints are rejected and rebuilt.
- [x] Deliberately faulty fixtures and generated artifacts are isolated from normal generated code, reviewed configuration and accepted state; record each expected failure as a successful demonstration without accepting its candidate.
- [x] Publish demonstration reports into the mandatory readiness gate so acceptance requires their successful completion.


## Completion evidence

Implemented the mandatory nine-case suite in `tools/readiness_feedback.py`, exposed by `just readiness-feedback` and integrated with publication-time verification. `feedback-probe` builds synthetic tables through the real decoder/emitter. The first wrong write at instruction index 3 names $00:8004/MX=3; complete frame-end CPU/clocks, WRAM and framebuffer still match. Actual replay profiles exercise changed-key and empty-table refusal. Private repositories exercise real lifecycle/acceptance failure commands without targeting maintained source or workspace acceptance. Repeated DMA validates complete paired state, cold/warm trace equality, reuse and corruption/incompatible-cache rebuilds.

Fresh scoped run `20261010T101106.148315Z-9c7bff335e57` passed all nine in 29.0 seconds. Complete-domain run `20261010T102036.267015Z-71f00ab0685d` also passed all nine in 30.7 seconds alongside check/lint, 70011 enabled-pattern cases, both complete parity runs and 15184 strict independent checkpoints; it correctly finalized error because of an independent discovery evidence-name collision and did not accept. Fixed the collision with a regression test retaining both real subprocess/result reports; focused run `20261010T105125.024883Z-f11a34b188c1` passed complete paired validation and 65 warm instructions in 0.279 seconds. Final complete readiness rerun belongs to ticket 12.

`just check` includes 52 Python tests (two optional full replay tests skipped; mandatory full-domain command executed independently), and `just lint` passes. Fault-report tampering remains rejected even when its digest is recomputed. Initial failed fixture runs and the interrupted corpus run remain preserved honestly.

Final complete run `20261010T112635.727097Z-6bf369628146` passed all nine feedback demonstrations and every required gate; atomic publication succeeded. Valid execution therefore advanced acceptance only after complete scope and sealed-origin verification.
