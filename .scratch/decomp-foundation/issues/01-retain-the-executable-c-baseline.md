# 01: Retain the executable C baseline

**What to build:** Make the existing headless C baseline reproducible from the integration branch, preserving provenance, deterministic replay, observed fallback and decompression fixtures.

**Blocked by:** None (can start immediately)

Status: open
Triage: ready-for-agent

- [ ] Prepare and build the pinned C candidate from verified source and ROM identities, preserving required notices and explicit build/replay errors and timeouts.
- [ ] Replay the existing any% 9,000-frame prefix in two independent processes; compare progress, terminal WRAM/SRAM and captured routine fixtures.
- [ ] Keep the C candidate independent of the upstream normalized dual-run comparator and record in-memory ROM patches, input bit conversion and unverified original-ROM parity.
- [ ] Persist per-call source address, destination, sequence, before/after state and fixture identity as structured data; validate measured fallback and wrapper coverage with an exercised control.
- [ ] Retain existing implementation where it meets the criteria; report gaps without relabeling historical recomp evidence as decomp acceptance.
