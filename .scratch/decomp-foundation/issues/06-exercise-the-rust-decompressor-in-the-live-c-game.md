# 06: Exercise the Rust decompressor in the live C game

**What to build:** The existing C scheduler and hardware host invoke the accepted Rust decompressor through an explicit ABI during independent mixed replay.

**Blocked by:** 05: Prove and freeze offline translation acceptance

Status: resolved
Triage: ready-for-agent

- [x] Keep one canonical C-owned WRAM array and a validated immutable ROM view; establish bounded Rust borrowing, callback behavior if used, reset and error propagation at the ABI.
- [x] Enable replacement only for the accepted routine contract; successful calls execute Rust, and errors stop replay without silently invoking the original routine.
- [x] Replay the any% prefix independently with C and mixed implementations; compare corresponding state and relevant hardware effects, requiring nonzero Rust routine hits.
- [x] Prove reset and repeat replay determinism and an exercised ABI error control; record Rust/C routine hits and remaining gameplay CPU fallback.
- [x] Label success mixed C/Rust integration, preserving the distinction from adapted original-ROM movie parity.

## Resolution

The dedicated live host links the current sealed Rust static library through a
synchronous checked ABI with canonical C-owned WRAM. The unchanged scheduler and
hardware replay the frozen prefix independently. Four fresh processes each run
two epochs after the real C reset; all ten complete state checkpoints, all 46
call before/after states and ordered PPU/APU effects match. Mixed epochs execute
23 Rust decompression calls, zero C calls, seven observed command modes and zero
measured gameplay CPU fallback. Relative XOR is covered offline and unexercised
by this prefix. Seven ABI controls and an actual first-gameplay-call invalid
source control prove repeat lifecycle, checked failure and no fallback on error.

Retained offline pointer: `/home/jim/sm-decomp/out/decomp/ticket06/offline/decomp-acceptance.json`.
Live evidence: `/home/jim/sm-decomp/out/decomp/ticket06/live/20261010T183924.854808Z/verdict.json`
(first full raw effects run); current compressed-effects replay evidence is
recorded below on completion. Acceptance remains mixed C/Rust integration with
original-ROM movie parity explicitly not tested.

Checks: all 63 offline fixtures and all 11 negative acceptance controls; six
Rust memory tests; nine public offline Python tests; public live missing-seal
rejection and full replay/ABI-stop test.

The full gzip-effects public integration test passed (2 tests, 126 seconds).
Source unchanged for its host/ABI; the retained repeat below additionally records
explicit observed/unexercised mode names and compiler identities in the verdict.

Current gzip evidence: `/home/jim/sm-decomp/out/decomp/ticket06/live/20261010T184155.718973Z/verdict.json`
reports `mixed-C-Rust-integration`, all eight epoch inventories matched, seven
observed modes, relative XOR unexercised, and the exercised error control stopped
without fallback. Its complete compressed artifacts occupy about 325 MiB.
