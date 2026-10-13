# Native camera scrolling frame ownership

Status: resolved
Blocked by: 05, 20
Triage: ready-for-agent
Type: task

Owner: `/root/projectile_admission`
Worktree: `/tmp/sm-camera-scroll`
Branch: `decomp-camera-scroll`
Baseline: `b6f30bd`

Translate the complete `MainScrollingRoutine` ($90:94EC) call boundary,
`Samus_HandleScroll_X` ($90:95A0), `Samus_HandleScroll_Y` ($90:964F), and
the Spore Spawn scrolling-finished hook ($90:9589). Reuse the existing verified
room autoscroll/trigger and Samus distance helpers without editing those modules.

## Scope and dependencies

Writable ownership: new `crates/port/src/camera_scroll.rs`,
`crates/port/tests/camera_scroll.rs`, `tools/port/camera-scroll.toml` and
isolated Rust camera inventory/fixture producer files. Root owns shared public
registration, catalog and live/family wiring. This lane is independent of
tickets138/139. Source is pinned `snesrev/sm` bank90 plus existing Rust room and
Samus dependency closures, established by tickets05/20.

Retained room/scroll leaves have material live execution, but neither their
counts nor caller placement establish whole camera-frame ownership. Record
fresh actual-C total/active camera-frame counts and complete WRAM captures.
The native gameplay frame calls the boundary after contacts and before drawing.

Preserve byte pose direction/movement fields, unaligned camera table indices
0..7, 16-bit wrapping signed comparisons, fractional layer1 layout, X-before-Y
shared-state order, optional hook and previous Samus position/fraction copies.
Unsupported hook, table index or room extent must select C before publication.

## Required verification

- Actual compiled pinned C versus Rust full WRAM/raw returns for direct helpers,
  whole frame, wrapping/fraction boundaries, native aliases and active captures.
- Independent ordering, unsupported-before-publication and state preservation
  controls. Do not use malformed C table or ROM accesses as an oracle.
- Four C/mixed replay subjects with nonzero exclusive Rust whole camera-frame
  ownership and matching state/effects for the selected prefix; final merged
  replay preserves projectile admission. Original-ROM parity is separate.
- Scoped cleanup/deduplication, exact-source retention, integrated worktree
  removal and retained evidence authentication before resolution.

## Comments

2026-10-12: assigned by root after ticket137's completed admission reconciliation.
Shared worker root `/home/jim/sm-decomp/out/pipeline`, capacity four. Initial
experiments are bounded at sixty seconds. Tools and tests use Rust.

## Completion evidence

2026-10-13 recovery: gameplay integrated at
`7512b6479afe55da270931a9f3ae26c9d558e5a5`, shared live wiring at
`3ce28ba58aba319b60bbc949153baafd4433132e`. Current actual-C family467pairs
and8semantic controls pass; family post-cleanup authentication retained at
`../recovery-integrated-camera-scroll-post-cleanup-authentication.toml`.

The129712-input four-subject composed gate records94425exclusive Rust
`MainScrollingRoutine` calls per mixed subject,35580active,0refusals, with C
traversal/active counts matching. Full active WRAM captures, periodic/terminal
state and ordered effects match; zero fallback and preserved native137 admission.
Post-cleanup authentication `../recovery-composed-post-cleanup-authentication.toml`
passes four subjects/1881artifacts. Ordinary cleanup48176296bytes/dedup0; exact
source snapshots and packed store/pointers/readers remain, stale worktrees retired.
Full evidence and limits `../recovery-live-completion-20261013.md`, disk receipt
`../recovery-live-final-disk.txt`. Original-ROM parity and credits progression
remain separate from this complete camera translation/integration.
