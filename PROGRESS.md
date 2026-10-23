# Engine modernization progress

Updated: **2026-10-23 12:33 UTC**

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

## Consolidation audit: current blocker

The current owned projectile candidate is **not ready for production adoption**.
Its flight bridge still calls production flight, animation playback, clear and
numeric motion dispatch. Adopting that wrapper would retain the implementations
we need to retire. Two supported motions (Speed Echo and UnknownProj8027) and
broad-index update continuation still depend on the old dispatch path.

The next replacement must cover the complete projectile lifecycle: selection,
firing, updates, retirement, trails, drawing and weapon feedback. It must reuse
existing shared algorithms under one projectile owner, migrate actual callers,
and remove superseded bank orchestration and per-operation RAM adapters.
Interactions with Samus, enemies, room objects and the backend must have explicit
boundaries that preserve effect order and returned state. Persistent whole-engine
ownership remains a later milestone; RAM cannot have a competing mutable owner.

The completed source inventory counts **5,236 candidate lines** against **2,213
lines of practical retirement**: **+3,023 lines before missing work**. The candidate
count was independently checked against the actual composition files; retirement
is a reviewed body inventory, not yet a buildable deletion patch. Direct adoption
of this candidate is rejected.

The replacement design uses concrete projectile state and one private compatibility
boundary. Existing shared algorithms bind directly to that state. A single byte
mapping handles ordinary selectors and overlapping legacy accesses, replacing the
candidate's repeated encoded/runtime forwarding and old-algorithm fallbacks.
Foreign calls publish the owner and reacquire their actual returned state.

The complete scope includes both missing motions, all lifecycle callers, drawing,
feedback, reflection/reset/impact, and ordered launch history. The first local
source draft exercises ownership and animation through this boundary. Review found
and corrected a bytewise-versus-atomic write mismatch. All **nine focused checks
pass** through the established Python/DLL workflow (exit 0), covering all 366 owned
bytes, boundary-crossing aliases, live selection, partial writes, foreign returned
state/unwind and the unchanged shared animation core for five selectors. Storage
and content are simplified test substitutes: this proves the access mechanism,
not production gameplay or complete subsystem integration. The standalone test
executable was blocked by Windows policy; security settings were not changed.
Full isolated subsystem implementation is proceeding from this design. The full
state mapping has passed independent source review: 391 distinct words / 782 bytes
cover shots, trails, controls and launch history without overlapping ownership.
Direct flight, animation, lifecycle, beam, missile, bomb and special-attack bindings
are authored; collision, presentation and caller migration remain in progress.
These replacement sources have not yet been built or validated as an integrated game.

The current production baseline was rechecked on the Speed Booster route from boot:
**8,780 frames, zero failing frames, exit 0** (26.09 seconds of replay). It reaches
427 gameplay frames in one room and no credits. This establishes the parent result
for a relevant animation/drawing/trail route; it does not validate the replacement
or cover special attacks and reflection. Those still need focused integrated checks.

The full replacement is estimated at 2,270-3,690 new production lines, against the
conservative 2,213-line retirement inventory: approximately **+60 to +1,480
production lines**. Verification additions are estimated separately at 600-1,200
lines. These are planning ranges, not measured patch results or claimed savings.
The structural gain must be one authoritative implementation with retired old paths;
actual total cost and behavior checks are required before integration.

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
