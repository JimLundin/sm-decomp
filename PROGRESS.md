# Engine modernization progress

Updated: **2026-10-23 07:19 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Shared typed setup dispatch: all 115 setup identities decoded.**

- Main and candidate now share decoding and ordinary setup execution.
- Removed the old bank dispatcher and duplicate candidate mappings while
  preserving aliases, nested collision behavior and exact diagnostics.
- Production and candidate release builds passed.
- **29 comparison suites passed**, with all 28 earlier results unchanged;
  a supplemental gate suite passed separately on the same build.
- Five old/new fatal-diagnostic comparisons matched under the test harness.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 105,946 lines in 199 files |
| Counted unsafe function declarations | 2,875 |
| Prospective composition, not yet adopted | 114,270 lines in 306 files |

This stage adds 356 lines to main and removes 506 lines from the complete
prospective composition. Safety work has increased main-source size. The target
remains below 100,000 maintained Rust lines, with all unsafe operations removed.

## Work in progress

**Samus movement and room contact: implementation in progress.**

The next shared core will own each movement scan's temporary state and replace
raw displacement pointers through mechanism setup and nested spawns. It covers
horizontal/vertical movement, wall-jump collision and pose-change collision.
Source review and production release/frontend/verifier builds passed. The
prospective library build also passed. The initial candidate command mistakenly
enabled frontend dependencies and received an access-denied error; the usual
library-only command passed. Behavior comparisons, final accounting and the
implementation commit are still pending. The figures above remain the last
verified milestone.

Persistent gameplay state remains in RAM. Renderer/audio replacement and an
independently running game slice remain required parts of the destination.

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
