# 01: Retain the executable C baseline

**What to build:** Make the existing headless C baseline reproducible from the integration branch, preserving provenance, deterministic replay, observed fallback and decompression fixtures.

**Blocked by:** None (can start immediately)

Status: resolved
Triage: ready-for-agent

- [x] Prepare and build the pinned C candidate from verified source and ROM identities, preserving required notices and explicit build/replay errors and timeouts.
- [x] Replay the existing any% 9,000-frame prefix in two independent processes; compare progress, terminal WRAM/SRAM and captured routine fixtures.
- [x] Keep the C candidate independent of the upstream normalized dual-run comparator and record in-memory ROM patches, input bit conversion and unverified original-ROM parity.
- [x] Persist per-call source address, destination, sequence, before/after state and fixture identity as structured data; validate measured fallback and wrapper coverage with an exercised control.
- [x] Retain existing implementation where it meets the criteria; report gaps without relabeling historical recomp evidence as decomp acceptance.

## Completion evidence

Implemented on `decomp-foundation-ticket-01`, based on integration commit
`7d4bede`. The completing commit retains the existing pinned C implementation
and adds public command controls and structured captures.

- Public CLI test failed first because ROM/output options and structured
  capture/control evidence were absent; it now passes using real GCC, C game
  and 65816 execution, plus a missing-input nonaccepting control.
- `python3 -m unittest discover -s tools -p test_decomp_baseline.py`: 2 passed.
- Two independent 9,000-frame replay processes match logs, terminal WRAM/SRAM,
  23 routine manifests and 46 complete WRAM fixture states. Evidence:
  `/home/jim/sm-decomp/out/decomp/ticket01/20261010T181153.663151Z/verdict.json`.
- Observation control executed 1 fallback call, 3 fallback CPU instructions,
  and 1 decompression call with known literal output checks.
- Actual full 4 MiB routine-entry ROM snapshots are retained by content identity;
  initial 3 MiB expansion and 690 changed patch bytes are recorded separately.
  No upstream or ROM-derived bytes were committed.
- Original-ROM parity, same-unit wrapper coverage and transient-write acceptance
  remain explicitly outside this baseline's claims.
