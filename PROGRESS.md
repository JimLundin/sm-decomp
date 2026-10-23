# Engine modernization progress

Updated: **2026-10-23 09:46 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Beam charge, firing and initial collision now share safe rules.**

Implementation: [shared beam rules](sm/src/game/beam_firing.rs).

- Retired duplicate firing and collision dispatch. Ordinary owned collision
  scans execute directly, including selector aliases and failure handling.
- Production and candidate builds passed. **All 46 comparison suites passed**.
  All 36 previous behavior suites remain; two deliberately removed compatibility
  transfers explain the sole retained counter change.
- Added 1,294 three-way RAM/APU/failure-state comparisons, including 14 failing
  cases, plus five separate injected read-failure checks.
- Ten chains reach retirement from actual ordinary, charged or hyper input,
  through initial terrain and 628 travel/playback ticks, without injected phases
  or lifetimes. These are component tests, not full-game or rendering coverage.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 106,659 lines in 209 files |
| Counted unsafe function declarations | 2,816 |
| Prospective composition, not yet adopted | 115,151 lines in 318 files |

This stage adds 252 lines to main and removes 13 unsafe function declarations.
The complete composition adds 426 lines; including retained verification, growth
is 1,874 lines. Safe adapters and independent checks are included in those costs.
The target remains below 100,000 maintained Rust lines, with all unsafe
operations removed.

## Work in progress

**Continue the player-projectile lifecycle and its remaining consumers.**

The complete missile and super-missile firing implementation has passed source
review, formatting checks and production/candidate builds. It covers reservation,
ammunition, placement, launch and cancellation. The comparison fixtures and
independent typed client are being reviewed; behavior verification is pending.
These source changes are not committed yet. The verified results above remain
the beam milestone.

Persistent room and scheduler state still live in RAM. Persistent Session
ownership, renderer/audio replacement and an independently running game slice
remain required parts of the destination.

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
