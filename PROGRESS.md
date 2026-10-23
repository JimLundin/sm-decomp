# Engine modernization progress

Updated: **2026-10-23 06:52 UTC**

**Status: active development. The complete modernized engine is not finished.**

The goal is a purpose-built Rust engine that plays Super Metroid from boot to
credits, with owned gameplay state and its own rendering and audio. The current
SNES backend remains the comparison oracle during migration.

## Latest verified milestone

**Noob Tube rupture and related block setups**, committed as
[`e5f30bdf`](https://github.com/JimLundin/sm-decomp/commit/e5f30bdf352c34a7890dcf7afac1e964e436673e).

- Shared the setup, keypress wait, rupture effects, water change and control resets.
- Removed ten bank bodies while retaining all eleven numeric dispatch identities.
- Production and candidate release builds passed; **all 28 comparison suites passed**.
- All 27 earlier suite results remained unchanged.
- Checked partial/full effect pools, room reentry, duplicate tubes and supported
  prefixes of the related unused scripts.

| Verified committed measure | Value |
| --- | ---: |
| Setup identities shared | 101 / 115 |
| Maintained Rust source | 105,590 lines in 196 files |
| Counted unsafe function declarations | 2,876 |
| Prospective composition, not yet adopted | 114,776 lines in 303 files |

Safety work has increased main-source size. The reduction target remains below
100,000 maintained Rust lines; moving code or adding wrappers does not count as
completing either reduction or unsafe removal.

## Work in progress

**Shared typed setup dispatch and closure of the remaining fourteen identities.**

- The new decoder contains all 115 identities and retains the original address
  separately from its semantic action.
- The implementation removes duplicated decoding, reverse mapping and setup
  forwarding, while preserving authored callbacks and nested collision behavior.
- Source review and the production release/frontend/verifier checks passed.
- The first candidate build found a private re-export error; the fix is prepared
  and the candidate rebuild is next.
- Component verification, fatal-diagnostic checks, final accounting and commit
  remain pending. These changes are not included in the verified figures above.

## What follows

Finish setup-dispatch validation and retirement, then continue the remaining
gameplay consumers and state-ownership transfers. Renderer/audio replacement and
an independently running game slice remain required parts of the destination.

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
