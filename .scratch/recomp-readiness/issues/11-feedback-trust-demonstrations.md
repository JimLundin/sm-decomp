# 11: Automate the feedback trust demonstrations

**What to build:** An automated suite proves that the complete workflow accepts valid execution and refuses faulty, uncovered, stale or incomplete evidence.

**Blocked by:** 10: Assemble readiness gates, atomic acceptance and safe recovery.

**Status:** ready-for-agent

- [ ] Correct execution produces a fresh identity-bound pass and can advance acceptance only with every required gate satisfied.
- [ ] An executed incorrect compiled instruction fails and names the altered entry at the earliest observable state/bus-timing disagreement.
- [ ] A transient wrong compiled effect restored later in the frame is caught by focused comparison even with matching frame-end bytes.
- [ ] An added unexecuted key yields uncovered despite execution by older entries; empty compilation cannot pass.
- [ ] Preflight/build errors, interrupted/truncated captures, incompatible identities, source/input contamination, stale prior verdicts and absent terminal completion cannot yield acceptance.
- [ ] A valid paired checkpoint reproduces the corresponding cold trace/outcome; incompatible/corrupt checkpoints are rejected and rebuilt.
- [ ] Deliberately faulty fixtures and generated artifacts are isolated from normal generated code, reviewed configuration and accepted state; record each expected failure as a successful demonstration without accepting its candidate.
- [ ] Publish demonstration reports into the mandatory readiness gate so acceptance requires their successful completion.

