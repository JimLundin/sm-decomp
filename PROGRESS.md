# Engine modernization progress

Updated: **2026-10-23 11:19 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**HUD switching, cooldown and weapon coordination now share safe rules.**

Implementation: [weapon coordination](sm/src/game/weapon_coordinator.rs).

- Retired duplicate bank and owned algorithms. Actual normal/demo preparation
  and deselection callers now use the shared rules.
- Removed an unnecessary Session transfer while preserving its inventory/input
  refresh in memory, at the original point in the sequence.
- Production and candidate builds passed. **All 58 selected suites passed**,
  with all 54 retained result vectors unchanged.
- New HUD checks include 1,194 three-way state comparisons and 15 natural chains.
  They cover selection through weapon retirement, all five spread bombs, real
  grapple/Xray scheduling, and power-bomb HDMA reaching the Noob Tube trigger.
- Separate active-input preparation checks passed 74 comparisons. An older
  controlled fixture fails identically on the untouched parent; its obsolete
  callback expectations are documented in the detailed handoff.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 107,176 lines in 217 files |
| Counted unsafe function declarations | 2,794 |
| Prospective composition, not yet adopted | 115,590 lines in 326 files |

This stage adds 179 lines to main and removes 16 unsafe function declarations.
The complete composition adds 96 lines; including retained verification, growth
is 2,636 lines. Adapters, generated test overlays and independent checks are
included. The target remains below 100,000 maintained Rust lines, with all
unsafe operations removed.

## Work in progress

**Player preparation replacement is approved and being implemented.**

The implementer and independent reviewer have agreed the scope: share entry
setup, normal/demo ordering, weapon dispatch and finalization, then retire the
duplicate bank and candidate implementations. New checks will cover real demo
input and later restoration, message suspension, intro gating and failure
prefixes. These changes have not yet been built or verified.

Preparation is only part of a player frame; contacts, enemies, suspension and
later movement retain their ordering. The proposal estimates 150–280 additional
main-source lines for safe shared sequencing and adapters; it claims neither a
source reduction nor persistent ownership.

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
