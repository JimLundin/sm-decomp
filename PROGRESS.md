# Engine modernization progress

Updated: **2026-10-23 15:42 UTC**

**We are consolidating the old bank-organized code into complete gameplay systems.**
Projectiles and room mechanisms are delivered. Palette effects are being verified.

## Road to the engine

| Stage | Status | What reaching it means |
| :--- | :---: | :--- |
| 1. Agree the destination | ✅ Done | A documented engine design and migration approach. |
| **2. Consolidate gameplay systems** | **▶ CURRENT** | Put each system's behavior and state together; migrate callers and delete the old implementations. |
| 3. Keep game state owned between updates | ◻ Ahead | Systems retain their state without relying on shared SNES memory as the authority. |
| 4. Separate the graphics and audio backend | ◻ Ahead | The game supplies explicit drawing/audio outputs and runs independently of SNES emulation. |
| 5. Demonstrate the complete game | ◻ Ahead | Validate the finished engine from boot to credits. |

These are milestones, not equal portions of the remaining work. Code reduction
and removal of unsafe operations continue throughout the migration.

## Current delivery: palette effects

**Purpose:** one place owns effect creation, timing, script execution and color
changes, replacing the old bank module and its memory-access adapter.

| Step | Status | Evidence / next action |
| :--- | :---: | :--- |
| Capture the existing behavior | ✅ Done | 512 reference cases saved before replacement. |
| Implement and connect the replacement | ✅ Local | All 13 caller files migrated; old `sm_8d` deleted locally. |
| Build and compare behavior | ▶ Running | Formatting passes; combined build passes. Running: all 512 comparisons and a bounded game replay through palette upload. |
| Publish the verified change | ◻ Pending | Commit and push after checks and independent review pass. |

This delivery is **not yet on main**. Its state ownership lasts during an
operation; persistent ownership is still stage 3. Preliminary production size
increases, so this is an ownership/safety improvement, not a code-reduction claim.

## Delivered milestones

| Delivery | Visible result | Verification | On GitHub |
| :--- | :--- | :--- | :--- |
| Player projectiles | One system for shots, trails and weapon updates; 14 adapters retired. | 106 regression groups; full-memory comparisons. | ✅ Delivered |
| Room mechanisms | Creation, scripts, drawing and interactions consolidated; **51 old files deleted**, including 24 adapters. | 1,018 baseline outcomes; **28,314 game frames, zero failures**. | [81a524cc](https://github.com/JimLundin/sm-decomp/commit/81a524cc) |
| Shared state transfer | Projectile and mechanism systems use one implementation for passing state to legacy calls and recovering it. | Both systems' comparisons and game route pass. | [15d75180](https://github.com/JimLundin/sm-decomp/commit/15d75180) |
| Formatting and public CI | Full repository formatting enforced; Windows frontend build provisioned. | [Format and release build pass](https://github.com/JimLundin/sm-decomp/actions/runs/39979717734). | [7ddd6331](https://github.com/JimLundin/sm-decomp/commit/7ddd6331) |

## How we count progress

| Measure | Reporting rule |
| :--- | :--- |
| **Production code** | Engine/game behavior, runtime adapters and their callers count toward the reduction goal. |
| **Verification support** | Harnesses, comparison exports and probes are shown separately and excluded from the engine-reduction total. New palette cases live in a dedicated verification directory. |
| **Total maintained code** | Still disclosed so verification and build-support costs remain visible. |
| **Real simplification** | Count deleted implementations and shared algorithms; file moves and formatting changes earn no reduction credit. |

Current committed Rust total: **108,903 lines, including verification**.
A reconciled production-only total is pending; the raw total is not the new
engine-size measure. Palette delivery will report its production and verification
deltas separately. Historical mixed totals below retain their original meaning.

<details>
<summary>Technical evidence and limits</summary>

- Room mechanisms: 1,015 full-memory/return/native-trace outputs match, two native
  diagnostics match, and one bounded nonreturn is observed. Ownership checks and
  the qualified-command checks pass. The game route covers 12 rooms and nine
  enemy definitions, including message completion and 120 following frames.
- These checks do not establish exhaustive branches or boot-to-credits fidelity.
  Two retained native pose intervals had undefined accesses in the old Rust;
  their helper checks and static review are not defined native equivalence.
  The bounded nonreturn observes a timeout, not its preceding write dynamically.
- Historical room-mechanism total, **including verification**: -228 lines,
  -4,214 tokens, -6,464 LF bytes. The integration removed 1,154 lines after 926
  lines of reference preparation had been added. Reclassification must not
  confuse this combined figure with a production-only figure.
- Shared state transfer: production -49 lines, registration +1, verification
  +40; combined -8 lines / -5 tokens / -133 LF bytes.
- Earlier projectile adoption: +1,389 mixed lines, reported then as 596
  runtime/probe lines and 793 verification lines. Runtime and probes still need
  separation before presenting a production-only value.
- Formatting added 229 layout lines across 75 files; it changed no behavior.
  The local release/frontend/verifier build and GitHub formatting/build pass.
  The separate manual ROM workflow has not been migrated by the public CI fix.
- Current state owners still relinquish control for legacy calls. Whole-memory
  restore/load, external writers and shared suspension prevent persistent
  ownership. Rendering/audio still use the emulated backend; unsafe operations
  remain. Reference captures stay local; no ROM-derived fixtures are committed.

</details>

[Work plan](docs/PLAN.md) · [Technical handoff](docs/HANDOFF.md) ·
[Accepted engine design](docs/target-architecture/README.md)
