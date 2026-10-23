# Engine modernization progress

Updated: **2026-10-23 11:53 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Player preparation now uses one shared safe sequence.**

Implementation: [player preparation](sm/src/game/player_preparation.rs).

- Replaced duplicate normal/demo setup, weapon coordination and finalization.
  Actual gameplay and intro callers use the shared implementation.
- Preserved captured handlers, live reads, packed memory aliases, later demo
  input restoration and message suspension between preparation and movement.
- Production and candidate builds passed. **All 59 selected suites passed**,
  with all 58 retained result vectors unchanged.
- New checks cover 883 three-way comparisons across 119 authored scenarios,
  including real demo input and weapon creation, later beta restoration,
  inactive/frozen paths and dirty owned state without premature publication.
- Two real dialogue chains each take 388 calls and run preparation only once
  across suspension. These are calls/resumptions, not movie frames.
- Four invalid-preparation assertion pairs match the frozen implementation;
  five inherited setup assertion pairs also remain unchanged.

| Verified measure | Value |
| --- | ---: |
| Maintained Rust source | 107,322 lines in 219 files |
| Counted unsafe function declarations | 2,790 |
| Prospective composition, not yet adopted | 115,721 lines in 328 files |

This stage adds 146 lines to main and removes four unsafe function declarations.
The complete composition adds 131 lines; including retained verification, growth
is 1,647 lines. All adapters, test overlays and helper changes are counted. One
verification category exceeds its estimate by 20 lines; the aggregate is below
its estimate. This is a safety and shared-behavior improvement, not a net reduction.

## Work in progress

**Charged-shot palette feedback replacement is approved for implementation.**

The next change shares glow timing, charge-palette cycling and output selection
between main and the owned candidate, retiring both duplicate rule bodies.
Checks will follow real weapon producers through palette updates, beta callers,
rainbow palette changes and resumed charge audio. The proposal estimates
117–220 additional main-source lines; it preserves existing transfers around
external palette writers and does not claim persistent ownership. Implementation
and verification are pending.

A candidate-only callback-conversion optimization was deferred because it has
no production contribution and is not required for adoption.

Persistent Session ownership still requires reset/load/oracle-rewind and external
writer handling. Preparation remains separate from later movement and scheduler
suspension. Renderer/audio replacement and an independently running game slice
remain required parts of the destination. The target remains below 100,000
maintained Rust lines, with all unsafe operations removed.

## Verification limits

The comparison suites exercise selected real implementations and constructed
scenarios. Demo tests use a no-op movement handler, and intro timer tests use a
frozen scene; they do not establish active movement/knockback physics or full
demo playback. Typed preparation callbacks are modeled; their bomb consumer is
real shared safe code. Assertion tests use the existing harness assertion handler.

These results do **not** establish boot-to-credits coverage, visible rendering
fidelity or an independent backend. Known native-access exclusions and the
inherited Intro mismatch remain unresolved. Older controlled preparation fixtures
fail identically on the untouched parent and remain explicitly excluded; their
valid active-input subset still passes. ROM-derived fixtures stay local.

This file is updated and pushed when verified milestones land or material work
status changes. See [the active plan](docs/PLAN.md),
[the detailed handoff](docs/HANDOFF.md) and
[the accepted architecture](docs/target-architecture/README.md) for scope,
validation details and remaining work.
