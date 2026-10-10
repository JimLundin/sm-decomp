# 06: Exercise the Rust decompressor in the live C game

**What to build:** The existing C scheduler and hardware host invoke the accepted Rust decompressor through an explicit ABI during independent mixed replay.

**Blocked by:** 05: Prove and freeze offline translation acceptance

Status: open
Triage: ready-for-agent

- [ ] Keep one canonical C-owned WRAM array and a validated immutable ROM view; establish bounded Rust borrowing, callback behavior if used, reset and error propagation at the ABI.
- [ ] Enable replacement only for the accepted routine contract; successful calls execute Rust, and errors stop replay without silently invoking the original routine.
- [ ] Replay the any% prefix independently with C and mixed implementations; compare corresponding state and relevant hardware effects, requiring nonzero Rust routine hits.
- [ ] Prove reset and repeat replay determinism and an exercised ABI error control; record Rust/C routine hits and remaining gameplay CPU fallback.
- [ ] Label success mixed C/Rust integration, preserving the distinction from adapted original-ROM movie parity.
