# Engine modernization progress

Updated: **2026-10-23 12:08 UTC**

**Status: active development. The owned engine is not yet consolidated into production.**

The destination is a purpose-built Rust engine that plays Super Metroid from boot
to credits, with owned gameplay state and its own rendering and audio. The SNES
backend remains the comparison oracle during migration.

## Latest verified change

**Charged-shot palette rules and output selection are shared.**

[The shared consumer](sm/src/game/beam_charge_palette.rs) replaces duplicate
main/owned glow and charge-cycle logic. Real weapon producers, palette callers,
demo restoration, rainbow updates and resumed audio are covered by the new checks.

- Both release builds and independent review passed.
- All **60 selected suites passed**; all **59 retained result vectors** are unchanged.
- New checks cover **378 comparisons across 318 authored scenarios**, including
  ten two-way image-getter comparisons and separately qualified checked-only cases.
- Nine existing assertion pairs remain unchanged.

| Measure | Current value |
| --- | ---: |
| Main Rust source | 107,432 lines in 221 files |
| Counted unsafe function declarations | 2,789 |
| Prospective composition, not adopted | 115,832 lines in 330 files |
| This change: main growth | 110 lines |
| This change: composition plus verification growth | 1,602 lines |

## Direction correction: consolidate the engine

Preparation and palette together add **256 production lines**. They remove
duplicate rules, but adapters outweigh those deletions. They establish sharing
and partial safety improvements, not net simplification or production adoption
of the owned engine. The user's concern about this trajectory is valid.

The next task is a **complete subsystem consolidation proposal**, beginning with
the existing owned projectile candidate. It must identify a clear public API and
state owner, migrate real production callers, and name the bank orchestration
and RAM adapters deleted in the same adoption. Adoption and deletion costs will
be counted together. If that boundary cannot yet be adopted, the proposal must
identify the specific blockers and minimum closing work before more coding.

Further isolated shared consumers, package moves and candidate-only cleanup are
not the next milestone. Persistent Session ownership, full-game coverage and
independent rendering/audio remain unfinished.

## Verification limits

The checks exercise selected actual implementations and constructed scenarios.
Caller fixtures use no-op movement and disabled minimap; they do not prove full
movement physics or boot-to-credits coverage. Typed outputs are modeled separately.
Generic native palette copying and visor code remain unchanged safety debts.
Scoped restored-image tests do not establish frontend/checkpoint/oracle adoption.

Known native-access exclusions and the inherited Intro mismatch remain unresolved.
Older controlled preparation fixtures fail identically on their untouched parent
and remain excluded; their valid active-input subset still passes. ROM-derived
fixtures stay local. No full-renderer/movie result is claimed for this change.

This file is pushed with verified changes or material status changes. See
[PLAN](docs/PLAN.md), [HANDOFF](docs/HANDOFF.md), and
[the accepted architecture](docs/target-architecture/README.md).
