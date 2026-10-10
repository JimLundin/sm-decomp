# 08: Run one additional dependency-ready translation job

**What to build:** Demonstrate a bounded routine loop by selecting, translating and accepting one further exercised pure routine using the frozen interfaces and live host.

**Blocked by:** 06: Exercise the Rust decompressor in the live C game

Status: open
Triage: ready-for-agent

- [ ] Select one small routine or tightly coupled pure family with observed execution and fixtures; record pinned C/assembly context, RAM/data contracts, callees, source freshness and exclusions.
- [ ] Preserve the accepted predecessor, execute synthetic and captured differential cases, and diagnose the first mismatch before acceptance.
- [ ] Accept only exercised passing behavior and prove its live replacement hits during independent C/mixed replay; unknown behavior stays visible.
- [ ] Provide a bounded job runner with explicit requested count, stop/failure status and progress showing accepted exercised routines plus remaining C/CPU fallback.
- [ ] Serialize shared build/evidence changes and allocate disjoint module ownership before concurrent translations. Avoid introducing hardware-host migration or timing-sensitive MMIO as part of this demonstration.
