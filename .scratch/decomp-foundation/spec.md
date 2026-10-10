# Decomp foundation

Status: open

The authoritative direction is [the agreed decomp pivot](../../docs/DECOMP_PIVOT.md).
The [executable C baseline](../../docs/DECOMP_BASELINE.md) records work already
performed and its limitations. Reuse that implementation and verify it against
ticket 01; its recorded replay does not establish original-ROM parity or Rust
ownership.

## Approved implementation scope

Retain the preparation baseline, accept the offline RAM decompression pilot,
prove its live Rust replacement, investigate a bounded original-ROM input
adapter, and demonstrate one additional dependency-ready routine job.

The input-adapter investigation can run alongside the offline pilot after
baseline verification. Its diagnostic result does not gate C/Rust translation.
A successful adapter is necessary before later original-ROM gameplay acceptance.

This batch ends after those foundation deliverables. Full any% completion,
full Rust gameplay ownership, hardware-host migration and Map Rando require
follow-on bounded tickets informed by the foundation evidence. The long-term
direction and behavior-preservation requirements of the agreed pivot remain
in force.

## Tickets

- [x] [01: Retain the executable C baseline](issues/01-retain-the-executable-c-baseline.md) — blocked by None.
- [x] [02: Run one literal decompression fixture through C and Rust](issues/02-run-one-literal-decompression-fixture-through-c-and-rust.md) — blocked by 01.
- [x] [03: Complete synthetic RAM decompression coverage](issues/03-complete-synthetic-ram-decompression-coverage.md) — blocked by 02.
- [x] [04: Replay captured decompression calls offline](issues/04-replay-captured-decompression-calls-offline.md) — blocked by 03.
- [ ] [05: Prove and freeze offline translation acceptance](issues/05-prove-and-freeze-offline-translation-acceptance.md) — blocked by 04.
- [ ] [06: Exercise the Rust decompressor in the live C game](issues/06-exercise-the-rust-decompressor-in-the-live-c-game.md) — blocked by 05.
- [x] [07: Verify an update-aligned original-ROM replay prefix](issues/07-verify-an-update-aligned-original-rom-replay-prefix.md) — blocked by 01.
- [ ] [08: Run one additional dependency-ready translation job](issues/08-run-one-additional-dependency-ready-translation-job.md) — blocked by 06.

## Completion

Resolve this spec only after all eight required tickets are resolved, with
integration-branch and verification evidence. Keep routine equivalence, mixed
C/Rust integration and original-ROM replay verdicts distinct. A reproducible
nonaccepting result may complete the bounded adapter investigation under ticket
07's criteria; it cannot establish adapted movie completion.
