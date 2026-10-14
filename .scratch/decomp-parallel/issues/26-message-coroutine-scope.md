# Review actual message coroutine and PLM continuation dependency

Status: open
Blocked by: 12, 18, 21, 22, 25
Triage: ready-for-agent
Type: research

Consumers:28 and milestones07–09.
Origin: [research15 message closure](../run-20261013/research-15/Answer.md).

## Question / Work

Establish direct exercise, hardware/audio/controller dependencies and an exact
coherent22-routine scope around `DisplayMessageBox_Async`. Preserve nested
states0614/0615 and `message_box_das0l_value` outside WRAM, all returns,
selection/timers, exact table/tile/HDMA extents and ordered byte MMIO/DMA.
PLM state0613 resumes must skip the preceding preinstruction/timer/opcode,
clear the queue only on completion and restore the same slot continuation.

Review proposed message_box source/tests/manifest separately from integrator-owned
message adapter and PLM adapter changes. Initial closing PLM domain remains usable
without messages; this child is required for a wider observed message domain.
Canned message returns provide no actual coroutine acceptance.

## Answer / Acceptance

Publish the exact22 definitions, exercised roots/yields/resumes, dependency and
asset seals, own-state/no-retry controls, current baseline/writable hashes and
bounded submission/integration/live/original-ROM argv. Proposed initial Cargo
message_box test cap60s and actual-C integration cap3600s are in the sealed
research15 children. Respect native C versus original-ROM delay/timing differences.

## Completion evidence

Pending admitted scope and implemented service; cleanup and reauthentication
precede any achieved stage.
