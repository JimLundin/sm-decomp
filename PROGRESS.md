# Engine modernization progress

Updated: **2026-10-23 07:05 UTC**

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

Setup dispatch is verified. Remaining creation/contact consumers and state
ownership transfers are next. RAM still owns the current gameplay state; raw
contact pointers and fatal diagnostics remain explicit compatibility boundaries.
Renderer/audio replacement and an independently running game slice remain
required parts of the destination.

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
