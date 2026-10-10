# Decomp foundation

Status: resolved

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
- [x] [05: Prove and freeze offline translation acceptance](issues/05-prove-and-freeze-offline-translation-acceptance.md) — blocked by 04.
- [x] [06: Exercise the Rust decompressor in the live C game](issues/06-exercise-the-rust-decompressor-in-the-live-c-game.md) — blocked by 05.
- [x] [07: Verify an update-aligned original-ROM replay prefix](issues/07-verify-an-update-aligned-original-rom-replay-prefix.md) — blocked by 01.
- [x] [08: Run one additional dependency-ready translation job](issues/08-run-one-additional-dependency-ready-translation-job.md) — blocked by 06.

## Completion

Resolve this spec only after all eight required tickets are resolved, with
integration-branch and verification evidence. Keep routine equivalence, mixed
C/Rust integration and original-ROM replay verdicts distinct. A reproducible
nonaccepting result may complete the bounded adapter investigation under ticket
07's criteria; it cannot establish adapted movie completion.

## Resolution evidence

All eight tickets are resolved on `decomp-foundation-integration`. Review fixes
were implemented in `57703be`: acceptance now verifies the complete retained
artifact inventory and current original instruction CPU, synthetic and captured
admission share one bounds validator, and ABI safety documentation and Rust
formatting/lint findings are corrected.

Final accepted decompression evidence is
`/home/jim/sm-decomp/out/decomp/review/offline/decomp-acceptance.json`: 63 fixtures,
38 exercised cases, 11 rejecting controls and 2,930 verified artifacts. The
accepted additional-routine run is
`/home/jim/sm-decomp/out/decomp/review/jobs/20261010T185638.603810Z/verdict.json`
(SHA-256 `a2e58037755569015582d4d339e231bd9fb4c7f7c6c2863b7648dbfa2f2a81bf`).
It verifies all 65,536 RNG seeds, 16 captured calls and four independent mixed/C
processes each replayed twice after reset. Each mixed epoch executes 23 Rust
decompressions and 8,783 Rust RNG calls with zero measured gameplay CPU fallback.

Standards and spec review findings were fixed and relevant public Python tests,
six Rust memory tests, formatting, Clippy and whitespace checks passed. Fresh
offline and live acceptance was regenerated after code changes; artifact mutation
tests prove that consumers reject altered retained evidence.

The bounded original-ROM adapter remains reproducibly nonaccepting under ticket
07's permitted completion criteria. Full any% completion, original-ROM gameplay
parity, full Rust ownership, hardware-host migration and Map Rando remain future
work. Two maintained Rust routines have exercised acceptance within the declared
foundation scope.
