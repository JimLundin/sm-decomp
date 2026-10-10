# 06: Compare focused instruction and bus execution

**What to build:** A diagnostic command cold-replays a selected interval at corresponding instruction boundaries and finds transient state or timing disagreements even when frame-end bytes match.

**Blocked by:** 02: Run cold fallback and generated parity with complete evidence.

**Status:** ready-for-agent

- [ ] Align corresponding reference/subject instruction boundaries and compare decoded CPU, interrupt/wait/stop state, CPU/master clocks and ordered fetch/read/write/implied cycles.
- [ ] Compare relevant mutable hardware and pending state affecting subsequent execution, including DMA/HDMA, refresh and frame/input boundaries.
- [ ] Observation must not consume interrupts, add bus reads, clock hardware or otherwise change execution; verify observed and unobserved results agree.
- [ ] Capture effects before later instructions cancel them and demonstrate a transient incorrect compiled effect whose frame-end bytes match.
- [ ] Retain the first observable mismatch and preceding matching boundary, both PC/opcode/MX values, compiled key, input digest/index/pad, before/after differences, clocks/scanline and differing bus event/state.
- [ ] Retain up to 64 preceding instructions and bus events on each side, paired snapshots and exact rerun commands; optional WRAM symbols enrich diagnosis but missing symbols never block it.

