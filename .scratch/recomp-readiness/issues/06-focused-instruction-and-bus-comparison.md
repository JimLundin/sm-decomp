# 06: Compare focused instruction and bus execution

**What to build:** A diagnostic command cold-replays a selected interval at corresponding instruction boundaries and finds transient state or timing disagreements even when frame-end bytes match.

**Blocked by:** 02: Run cold fallback and generated parity with complete evidence.

**Status:** done

- [x] Align corresponding reference/subject instruction boundaries and compare decoded CPU, interrupt/wait/stop state, CPU/master clocks and ordered fetch/read/write/implied cycles.
- [x] Compare relevant mutable hardware and pending state affecting subsequent execution, including DMA/HDMA, refresh and frame/input boundaries.
- [x] Observation must not consume interrupts, add bus reads, clock hardware or otherwise change execution; verify observed and unobserved results agree.
- [x] Capture effects before later instructions cancel them and demonstrate a transient incorrect compiled effect whose frame-end bytes match.
- [x] Retain the first observable mismatch and preceding matching boundary, both PC/opcode/MX values, compiled key, input digest/index/pad, before/after differences, clocks/scanline and differing bus event/state.
- [x] Retain up to 64 preceding instructions and bus events on each side, paired snapshots and exact rerun commands; optional WRAM symbols enrich diagnosis but missing symbols never block it.


## Completion evidence

- Public Reference/Subject single-step APIs share production dispatcher and compare full decoded CPU/pending old-I, wait/stop/interrupt state, architectural clocks and full mutable hardware. Maintained passive observer patch exposes real CPU and internal DMA effects plus ordinary-save omissions (DMA scratch and per-dot PPU pending).
- Three diagnostic integration tests pass: 100 matching observed boundaries with plain replay equality, transient incorrect compiled store detected before later matching frame-end RAM, and exact internal DMA A/B access order with observed/plain hardware equality. Focused report retains earliest mismatch, preceding match, paired snapshots and bounded64 history.
- Real identity-bound fallback diagnostic run20261010T082311.024312Z-e080baed492a passed8 focused instructions (26.93 seconds including isolated builds). Preflight verified all actual patched source hashes. Earlier contaminated run correctly rejected source mutation during build.
- After merging execution coverage accounting, just check passes all workspace/hardware tests and22 Python tests (two explicit optional long parity integrations skipped); just lint passes.
- Cold focused command and reusable module retain precise rerun arguments and scope; cached claims remain pending ticket07. Optional symbols are not required. Root dependency copy remains unchanged until maintained patch is explicitly applied by integration workflow.
