# 04: Validate and accept one new timed instruction pattern

**What to build:** When measured supported additions offer no useful further gain, take one observed opcode/addressing behavior through complete semantics and independent timing validation, real-ROM generated execution and cold acceptance. The new compiled support remains bounded to its declared modes and timing branches.

**Blocked by:** 01: Make timed-pattern validation extensible; 02: Select a measured routine/width increment.

**Status:** implemented; conditional promotion demonstration pending supported exhaustion

- [ ] Use measured interpreted execution to choose one useful pattern only after explaining why existing support cannot deliver useful further gain. Record the selected opcode/addressing behavior, applicable width/mode variants, timing branches, expected gain and excluded unrelated expansion.
- [x] Preserve the accepted predecessor and allocate exact-source evidence before mutation. Validate candidate step bodies without enabling the pattern in normal compiled support until its required semantics and timing tests pass.
- [x] Run the complete available affected-opcode corpus through actual emitted/executed steps, with verified corpus identities, ordered case ledgers and honest completion counts. Assert registers, flags, memory and applicable PC/control effects for all declared supported conditions; development samples cannot establish acceptance.
- [x] Supply independently specified fixtures for missing applicable widths, decimal/binary/status modes, addressing branches and control/stack/interrupt coverage. Record justified exclusions or substitutions; do not skip supported failures.
- [x] Establish independently sourced ordered bus and exact timing evidence, including applicable memory-speed, penalties/boundaries, interrupt polling/preemption, pending DMA/HDMA, refresh, frame rollover and resumability. Verify passive observers do not change execution and state why a condition is inapplicable.
- [x] Resolve known failures affecting the selected pattern before support is enabled. ADC remains blocked by its recorded overflow failures until resolved; unrelated unsupported failures remain linked debt and interpreted execution.
- [ ] Require targeted evidence for every declared mode/timing branch, at least one real-ROM generated execution, and cold execution of every newly enabled or changed address/MX key. Remove unexecuted proposed keys from the generated table and retain deferred inventory.
- [ ] Require focused instruction/bus comparison, candidate check/lint, relevant existing-pattern regressions and fresh full 18,000-frame generated parity. Verify independent evidence reuse or refresh affected readiness gates when shared dependencies change.
- [ ] Advance acceptance only after complete mandatory evidence and positive comparable compiled execution gain. Retain failed candidates, the accepted predecessor, source changes, coverage, timings and rationale in a reproducible brief; domain extension needs separate independent validation first.

## Completion evidence

- Prospective native AD body shares exact timed emitter and fetch/PC execution with production. AD remains disabled in ordinary support; existing supported increments take precedence.
- Complete public proof run `20261010T124052.761996Z-804dab489d86` passed all 10,000 pinned native AD cases plus eight independent fixtures (10,008 expected/run/passed, zero failed/skipped). Instruction 53.91s; independent timing 22.38s. `verify_proof` rechecked preserved source, ledgers, binaries, processes and hardware oracle identities.
- Six AD-specific hardware regressions passed: ordered 6/8/12-clock accesses and wide crossing, register read effects, final-data-cycle IRQ/NMI sampling, exact DMA/HDMA waits/refresh, frame/input rollover, checkpoint resume and passive observation.
- Two public prospective command tests and 12 readiness reuse tests pass; recomp tests, format and recomp/snes all-target clippy pass.
- `pattern-prove` and `pattern-promote` commands implement proof-before-promotion, verified supported exhaustion, observed AD key selection, actual deferred-table exclusions, preserved pre-mutation proposal, fresh complete promoted-source gates and a final proof/bounded-table/real-ROM acceptance hook.
- Remaining unchecked end-to-end criteria are conditional: current validated profile still offers useful supported routine/width increments, so enabling AD or claiming a new accepted AD increment would violate the selection contract. Final integrated ticket07 demonstrates the supported route; actual AD promotion must await verified exhaustion and fresh candidate gates. The code path is implemented; no actual AD acceptance is claimed.
