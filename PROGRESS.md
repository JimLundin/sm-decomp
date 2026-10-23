# Engine modernization progress

Updated: **2026-10-23 09:58 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Missile and super-missile firing now share safe rules.**

Implementation: [shared missile rules](sm/src/game/missile_firing.rs).

- Retired duplicate reservation and firing algorithms. Shared definitions also
  serve bomb, power-bomb and spread-bomb callers.
- Production and candidate builds passed. **All 51 comparison suites passed**,
  with all 46 retained result vectors unchanged.
- Added 463 three-way RAM/APU/failure-state comparisons, including 64 failing
  cases, plus two separate failure-publication checks and six typed sequences.
- Eight chains reach retirement from actual missile/super input through 190
  ticks. All four Super Missile chains create extensions, and all four static-wall
  chains impact. These are component tests, not full-game or rendering coverage.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 106,815 lines in 212 files |
| Counted unsafe function declarations | 2,814 |
| Prospective composition, not yet adopted | 115,343 lines in 321 files |

This stage adds 156 lines to main and removes two unsafe function declarations.
The complete composition adds 192 lines; including retained verification, growth
is 1,641 lines. Safe adapters and independent checks are included in those costs.
The target remains below 100,000 maintained Rust lines, with all unsafe
operations removed.

## Work in progress

**Continue the player-projectile lifecycle and its remaining consumers.**

Next is complete bomb, power-bomb and spread-bomb creation: input, reservation,
ammunition, charge/cancellation and initialization through their existing lifetime
and terrain rules. Caller and ordering review is complete; implementation design
and cost estimates come next.

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
