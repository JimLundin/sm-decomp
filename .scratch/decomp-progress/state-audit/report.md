# Current progress reporting audit, 2026-10-13

Status: resolved; publication, viewer checks and scoped cleanup complete
Owner: /root
Ticket: [04](../issues/04-current-state-audit.md)
Scope: reporting, retained evidence and source correspondence only

The report missed one existing Rust implementation and its completed bounded
actual-C checks. The refreshed inventory now includes
`DisplayViewablePartOfRoom` as both translated and verified. No decomp gameplay
was implemented, no new fixture or replay executed, and no integration
acceptance or assignment status was advanced.

| Independent reported stage | Before | After | C body lines after |
| --- | ---: | ---: | ---: |
| Translated | 1,460 | 1,461 | 16,527 |
| Verified | 924 | 925 | 10,103 |
| Integrated | 38 | 38 | 775 |
| Live exercised | 121 | 121 | 1,642 |
| Translating observations | 222 | 222 | 3,170 |
| Fixture declared definitions | 834 | 835 | 9,108 |

The denominator remains 6,053 pinned lexical C definitions / 75,229 body lines.
These independent stage counts can overlap and do not measure full-game
completion. Translating observations preserve assignment history and are not an
active-worker count. Sixteen stale records and one replaced record remain
excluded.

## Source correspondence

The fresh copy of the Rust source extractor scans all 45 `crates/port/src/`
files and 177 top-level TOML files, using newly copied reviewed aliases/roles.
Compared with the previous map, all prior gameplay bodies are byte-identical;
the changes are the public `lib.rs` registration and the new `game_host.rs`
forwarding adapter. The latter forwards seven scheduled entries to existing
Rust owners and contributes zero second gameplay bodies.

The original scanner recognized executable match arms but skipped
`room_scroll::dispatch_host`'s equality guard. Independently comparing
`room_scroll.rs:300` to pinned `sm_80.c:1657` verifies the admitted 17-column
display implementation: foreground and conditional-background tile preparation,
ordered synchronous host VRAM flushes and four X-block increments per column.
The hardware flush itself remains C-owned.

The reviewed alias adds one native mapping, bringing the automated map to
1,453 definitions / 16,454 body lines. The separately reviewed eight inlined
Ridley helpers add 73 body lines, explaining the qualified translated union
of 1,461 / 16,527. All 835 native fixture declarations have reviewed mappings.
Synthetic fixture seams remain outside the denominator. See
[the fresh source map note](note.md),
[the exact source delta](source-delta.json) and
[the original inline review](../ridley-inline-translation-audit.md).

The original map, notes, configuration and pinned source inventory retain their
exact bytes; [preserved-inputs.sha256](preserved-inputs.sha256) checks them.

## Retained checks and remaining gates

[Independent evidence findings](evidence-findings.md) records primary verdicts,
source identities and authentication receipts. All 324 unique path/hash
contracts across 323 registry evidence paths were read. Exactly two old hash
versions differ; every affected old observation was already stale. No new stale
coverage was found.

The shared host passed seven projectile and three room-display actual-C fixture
pairs across five roots. All 490 sealed artifacts, current Rust dependencies
and harness inputs match their retained hashes. Four projectile roots already
counted as verified; room display contributes one. Source is merged and
implementer worktree cleanup is retained, but durable host integration remains
running. This audit publishes verification without inferring integration or
live acceptance. PLM/HDMA/VRAM dependencies remain host-owned.

Later Ridley hardware-v2 controlled and natural verdicts now appear as frozen
historical verified observations. Their 33- and 20-root domains overlap prior
verification, so no unique count increase is attributed to those records.
They retain both PPU views, discriminator/open-bus and exact fixture admission
limitations. Whole-lifecycle integration, boss/explosion inputs and broad live
acceptance remain pending.

Controller infrastructure is committed/submitted and unintegrated. Its final
diagnostic matched 7,740 updates through original frame 8,317; the next update
diverged at frame 8,508 on preceding controller poll count. Both short and long
diagnostics are nonaccepting and add no gameplay stage credit. Ordinary-reset
Ceres escape and original-ROM parity remain open.

## Published observations

Four new immutable update TOMLs live here:

- [Room display translation](room-display-6053-current-translated-audit-20261013.toml)
- [Shared host verification](shared-game-host-6053-current-verified-audit-20261013.toml)
- [Ridley controlled historical domain](ridley-v2-controlled-6053-frozen-verified-audit-20261013.toml)
- [Ridley natural historical domain](ridley-v2-natural-6053-frozen-verified-audit-20261013.toml)

Maintained `progress refresh` rebuilt fixture declarations, then
`progress update` validated and published those records. A natural update
attempt hit the ten-second concurrent registry lock limit; its identical
sequential retry succeeded. The empty original stdout and successful retry
receipt remain. Final registry has 261 observations.
[Publication consistency](publication-consistency.json) confirms every original
observation, source identity, stale ID and replaced ID is unchanged.

Report viewer compaction and velocity are tracked separately in
[ticket 05](../issues/05-compact-layout-and-velocity.md). The graph deduplicates
native definition/stage reports by recorded report date; publication dates
include audit imports and are not implementation dates.

## Completion and cleanup

Final served/offline reports preserve all notices, evidence contexts and exact
stage/correction history; stored JSON, served JSON and embedded HTML agree.
Windows browser launch succeeded. The source coverage burndown added by
[ticket 06](../issues/06-coverage-burndown-and-bottlenecks.md) validates the full
1,461-symbol reviewed set and separates committed source velocity from imports.

Scoped ordinary/dedup cleanup succeeded with zero supported disposable bytes.
The identified audit executable was removed; original immutable inputs still
pass `preserved-inputs.sha256`. Final viewer/build/profile measurements, retained
dependencies and free disk evidence are in
[cleanup/result.md](../burndown/cleanup/result.md). The checkpoint owner
transferred main Git/index at 07:57 UTC; reporting source is checkpointed at
`263ad5acb096e66ae5d272c6f94534ae50c4b963`. Earlier raw receipts remain in
[the original receipt archive](checkpoint-retention.md), and the final reporting
raw archive preserves every original stdout/log byte without overwriting it.
