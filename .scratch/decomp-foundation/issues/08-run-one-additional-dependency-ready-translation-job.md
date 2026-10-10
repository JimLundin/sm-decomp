# 08: Run one additional dependency-ready translation job

**What to build:** Demonstrate a bounded routine loop by selecting, translating and accepting one further exercised pure routine using the frozen interfaces and live host.

**Blocked by:** 06: Exercise the Rust decompressor in the live C game

Status: resolved
Triage: ready-for-agent

- [x] Select one small routine or tightly coupled pure family with observed execution and fixtures; record pinned C/assembly context, RAM/data contracts, callees, source freshness and exclusions.
- [x] Preserve the accepted predecessor, execute synthetic and captured differential cases, and diagnose the first mismatch before acceptance.
- [x] Accept only exercised passing behavior and prove its live replacement hits during independent C/mixed replay; unknown behavior stays visible.
- [x] Provide a bounded job runner with explicit requested count, stop/failure status and progress showing accepted exercised routines plus remaining C/CPU fallback.
- [x] Serialize shared build/evidence changes and allocate disjoint module ownership before concurrent translations. Avoid introducing hardware-host migration or timing-sensitive MMIO as part of this demonstration.


## Completion evidence

Implemented on `decomp-foundation-ticket-08`, based on integration commit
`899b422`. See [routine contract and command](../../../docs/DECOMP_NEXT_RANDOM.md).
The disjoint standalone Rust candidate leaves every sealed decompressor source
identity unchanged. The predecessor ticket06 acceptance was independently verified
before and after execution and its pointer was never rewritten.

Accepted local run: `out/decomp/ticket08/20261010T184747.278832Z`.
Verdict SHA-256: `b3dcfb952b4c044aad3d4b9bea74f51bc8bbf1bd1326d0cef615b92c351e0857`.
All 65,536 seeds match actual pinned C and Rust with complete surrounding WRAM
invariance and independently worked edge expectations. Sixteen actual C live calls
retain full before/after states and pass separate C/Rust offline processes.
All 8,783 actual calls per epoch match the exhaustive C result table and recorded
semantic LE effects. Four independent replay processes, each reset and replayed
twice, match complete checkpoints, routine state fixtures and hardware effects.
Mixed epochs execute 8,783 Rust RNG calls, zero C RNG calls and 23 accepted Rust
decompressor calls; measured gameplay C-to-CPU fallback remains zero in this prefix.
Original movie parity, hardware multiplication timing and full Rust ownership
remain outside this acceptance.

First failed run `20261010T184717.573438Z` retained an incorrect handwritten
$FFFF expected value. C/Rust exhaustive outputs agreed; diagnosis of the original
carry retained by `ADC #$0011` corrected the independent expectation to 269
(and $3300 to 18). Second failed run `20261010T184733.898452Z` retained the missing
worktree asset error before replay. Neither error progress accepted a job.

Validation: public bounded-job tests first failed before implementation, then
passed for unknown jobs and requested counts 0/2; `python3 -m unittest
tools.test_decomp_jobs tools.test_decomp_live` passes (asset integration test skipped
because the complete fresh integration evidence above already ran). Python compile
checks and `git diff --check` pass. Current source snapshots, frozen manifest and
predecessor seal were checked again after the accepted run.
