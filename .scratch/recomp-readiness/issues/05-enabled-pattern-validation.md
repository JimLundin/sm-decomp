# 05: Validate every enabled timed pattern through the real step path

**What to build:** An explicit validation command executes the complete available affected-opcode corpus and applicable independent timing cases for all currently enabled compiled patterns.

**Blocked by:** 01: Capture immutable runs and verified preflight.

**Status:** ready-for-agent

- [ ] Exercise immediate LDA/LDX, REP/SEP, absolute STA, NOP and DEX through actual emitted and executing resumable step bodies, rather than only the untimed semantic emitter.
- [ ] Run complete available affected-opcode cases, batched if necessary; record corpus identity, case names and expected/run/passed/failed/skipped counts. Inactive or sampled corpus output is insufficient.
- [ ] Cover applicable native M/X widths, status/decimal/binary modes, addressing behavior and branches; assert registers, flags, memory and relevant PC/control effects. Fill missing applicable cases with independently specified fixtures.
- [ ] Validate ordered bus cycles and exact timing, memory-speed/boundary behavior, interrupt polling/preemption, pending DMA/HDMA, refresh, frame/input boundaries and resumability where applicable; document conditions that do not apply.
- [ ] Bind timing expectations to documented hardware sources or independent emulator observations, and verify observers are passive.
- [ ] Any failure affecting an enabled pattern blocks readiness and remains reported; unsupported ADC overflow failures remain explicit linked debt and do not justify skipping supported cases.
- [ ] Produce instruction/timing reports and durations suitable for inclusion with the candidate's check/lint evidence.

