# 05: Validate every enabled timed pattern through the real step path

**What to build:** An explicit validation command executes the complete available affected-opcode corpus and applicable independent timing cases for all currently enabled compiled patterns.

**Blocked by:** 01: Capture immutable runs and verified preflight.

**Status:** done

- [x] Exercise immediate LDA/LDX, REP/SEP, absolute STA, NOP and DEX through actual emitted and executing resumable step bodies, rather than only the untimed semantic emitter.
- [x] Run complete available affected-opcode cases, batched if necessary; record corpus identity, case names and expected/run/passed/failed/skipped counts. Inactive or sampled corpus output is insufficient.
- [x] Cover applicable native M/X widths, status/decimal/binary modes, addressing behavior and branches; assert registers, flags, memory and relevant PC/control effects. Fill missing applicable cases with independently specified fixtures.
- [x] Validate ordered bus cycles and exact timing, memory-speed/boundary behavior, interrupt polling/preemption, pending DMA/HDMA, refresh, frame/input boundaries and resumability where applicable; document conditions that do not apply.
- [x] Bind timing expectations to documented hardware sources or independent emulator observations, and verify observers are passive.
- [x] Any failure affecting an enabled pattern blocks readiness and remains reported; unsupported ADC overflow failures remain explicit linked debt and do not justify skipping supported cases.
- [x] Produce instruction/timing reports and durations suitable for inclusion with the candidate's check/lint evidence.


## Completion evidence

- Fresh immutable `just pattern-check` run `20261010T081059.869367Z-8a43c9eb6555` passed all 70,000 native corpus cases plus 11 independent boundary vectors, with zero failures/skips. Instruction and independent timing reports are retained under its checks/patterns evidence. Scoped verdict is pass and ineligible for readiness acceptance.
- Independent generated-path tests assert original IRQ/NMI/HDMA oracles, passive observation, slow/fast/12-clock access phases, pending DMA 48/54-clock stalls and refresh crossing. Complete state/ordered active bus/PC corpus comparisons use the shared production emitter/fetch path.
- Required `just check` and `just lint` pass; 13 Python tests include actual compiled-command correct/faulty expectation behavior. Legacy fuzz generated output remains preserved.
- Inactive pin-level addresses/signals cannot be observed by the abstract bus and are explicitly excluded; affected enabled patterns have complete state and ordered active-cycle evidence. Unsupported ADC overflow debt remains linked.
