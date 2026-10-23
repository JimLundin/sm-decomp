# Engine modernization progress

Updated: **2026-10-23 09:09 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Special-beam attack creation and all six motion states now share safe rules.**

Verified source: [fe6a0d0f](https://github.com/JimLundin/sm-decomp/commit/fe6a0d0f079c09312dee996c181ac8c20ded155e).

- Retired duplicate creation, movement and retirement algorithms. Actual beam
  input and motion dispatch use the shared implementation.
- Production and candidate builds passed. **All 36 comparison suites passed**,
  with all 34 retained results unchanged.
- Added 6,015 three-way operation comparisons, including 126 expected failures,
  plus authored-state and geometry checks.
- All four attacks reached natural retirement from actual charged beam input,
  without injected phases or lifetimes: Wave 600 ticks, Ice 620, Spazer 109,
  Plasma 99. These are component tests, not full-game or rendering coverage.

| Verified measure | Value |
| --- | ---: |
| Setup identities decoded | 115 / 115 |
| Maintained Rust source | 106,407 lines in 207 files |
| Counted unsafe function declarations | 2,829 |
| Prospective composition, not yet adopted | 114,725 lines in 316 files |

This stage adds 224 lines to main and removes five unsafe function declarations.
The complete composition adds 13 lines; including retained verification, growth
is 2,151 lines. Safe adapters and independent checks are included in those costs.
The target remains below 100,000 maintained Rust lines, with all unsafe
operations removed.

## Work in progress

**Continue the player-projectile lifecycle and its remaining consumers.**

The approved next change shares ordinary beam charge input, firing and initial
collision dispatch, including direct owned scans and their failure handling.
The first implementation is written and undergoing independent source review.
Final corrections, production/candidate builds and behavior comparisons are
pending. These source changes are not yet committed; the verified results and
counts above still describe the special-attack milestone.

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
