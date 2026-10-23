# Engine modernization progress

Updated: **2026-10-23 10:42 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Bomb, power-bomb and spread-bomb creation now share safe rules.**

Implementation: [shared bomb creation](sm/src/game/bomb_creation.rs).

- Retired duplicate bank and owned creation algorithms, with actual movement-HUD
  and Draygon callers using the shared rules.
- Production and candidate builds passed. **All 54 comparison suites passed**,
  with all 51 retained result vectors unchanged.
- Added 1,838 three-way RAM/APU/failure-state comparisons, 14 separate checked
  failure cases and eight independent typed sequences.
- Twelve natural chains cover 1,206 ticks. Spread charge comes from actual input;
  all five bombs flash, explode and retire. Actual power-bomb firing reaches the
  Noob Tube trigger through natural fuse expiry and real HDMA growth.
- These are component checks with documented constructed scenes and limits,
  not full-game or rendering coverage.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 106,997 lines in 214 files |
| Counted unsafe function declarations | 2,810 |
| Prospective composition, not yet adopted | 115,494 lines in 323 files |

This stage adds 182 lines to main and removes four unsafe function declarations.
The complete composition adds 151 lines; including retained verification, growth
is 2,332 lines. Safe adapters and independent checks are included in those costs.
The target remains below 100,000 maintained Rust lines, with all unsafe
operations removed.

## Work in progress

**Continue the player-projectile lifecycle and its remaining consumers.**

Complete HUD switching, cooldown and weapon coordination has passed source
review and both production/candidate builds. The shared rules now replace the
duplicate algorithms, and an in-memory input refresh replaces an unnecessary
Session transfer. The candidate required one correction to preserve existing
generic bounds on slot release; its rebuild passed.

Behavior checks are in preparation, covering selection input through weapon
lifecycles, dirty inventory state, and real grapple/Xray scheduling effects.
These source changes are uncommitted. The verified results above remain the
bomb-creation milestone.

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
