# Engine modernization progress

Updated: **2026-10-23 08:13 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Samus movement and room contact now share an operation-owned core.**

Committed as [`26d6b986`](https://github.com/JimLundin/sm-decomp/commit/26d6b98684b49c89ab03703a36c1cbff0f2b8c7a).

- Migrated horizontal/vertical movement, wall-jump collision and pose-change
  collision, including nested mechanism setup and resumed movement.
- Removed the raw displacement-pointer chain; temporary scan state has one owner.
- Production and candidate builds passed. **All 31 comparison suites passed**,
  with all 30 earlier results unchanged and five matching fatal-diagnostic pairs.
- Added 2,248 runtime comparison cases, including 48 expected failures, plus
  authored-state and boundary checks.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 105,958 lines in 202 files |
| Counted unsafe function declarations | 2,847 |
| Prospective composition, not yet adopted | 114,282 lines in 309 files |

This stage adds 12 lines to main and removes 28 unsafe function declarations.
The complete composition plus retained verification grows by 2,422 lines.
The target remains below 100,000 maintained Rust lines, with all unsafe
operations removed.

## Work in progress

**Bomb, power-bomb and spread-bomb lifecycle and terrain contact.**

The approved next change will share fuse, movement and terrain rules, and keep
candidate projectile state owned while scanning terrain. The validation plan
includes actual power-bomb radius growth reaching Noob Tube from a separated
origin. Source review and production/frontend/verifier and candidate library
builds passed on their first attempts. The new terrain comparison suite and the
broad lifecycle compatibility suite now pass. Actual firing with a 60-tick fuse,
followed by the retained HDMA radius producer, reached the separated Noob Tube
at tick 88 with radius `0x2b60`. This is a constructed gameplay check with clipped
drawing, not a rendering or movie test.

Remaining regression checks and final review are in progress. One retained test
still expected the native terrain callback that this change deliberately removes;
its coverage assertion is being updated to observe direct owned execution.
The bomb source is not yet committed. The figures above describe the last
committed, fully verified milestone.

Persistent room and scheduler state still live in RAM. Renderer/audio replacement
and an independently running game slice remain required parts of the destination.

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
