# Engine modernization progress

Updated: **2026-10-23 08:17 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Bomb, power-bomb and spread-bomb lifecycle and terrain contact now share safe cores.**

- Removed the old bank traversal and duplicate lifecycle rules. The candidate
  scans terrain directly with owned projectile state.
- Production and candidate builds passed. **All 34 comparison suites passed**,
  with all 31 retained results unchanged and five matching fatal-diagnostic pairs.
- Added 1,892 runtime comparison pairs, including 54 expected failures, plus two
  checked-only failure groups and 15 authored-state scenarios.
- Actual firing with a 60-tick fuse and real HDMA radius generation reached a
  separated Noob Tube at tick 88. This constructed check uses clipped drawing;
  it does not establish rendering or movie coverage.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 106,183 lines in 205 files |
| Counted unsafe function declarations | 2,834 |
| Prospective composition, not yet adopted | 114,712 lines in 313 files |

This stage adds 225 lines to main and removes 13 unsafe function declarations.
The complete composition plus retained verification grows by 2,850 lines.
Adapters and preservation of owner state on failure account for the growth.
The target remains below 100,000 maintained Rust lines, with all unsafe
operations removed.

## Work in progress

**Continue the player-projectile lifecycle and its remaining consumers.**

The next scope under investigation is the duplicated special-beam attack
production and motion rules. Persistent room and scheduler state still live in
RAM. Persistent Session ownership, renderer/audio replacement and an independently
running game slice remain required parts of the destination.

## Verification limits

The comparison suites exercise selected real implementations and constructed
scenarios. They do **not** establish boot-to-credits coverage, visible rendering
fidelity, full room decompression or an independent backend. Known native-access
exclusions and the inherited Intro mismatch remain unresolved. ROMs and
ROM-derived fixtures stay local and are not published here.

This file is updated and pushed when verified milestones land or material work
status changes. See [the active plan](docs/PLAN.md),
[the detailed handoff](docs/HANDOFF.md) and
[the accepted architecture](docs/target-architecture/README.md) for scope,
validation details and remaining work.
