# Decomp session checkpoint, 2026-10-13

Status: checkpoint prepared; source and receipts verified
Blocked by: None
Owner: checkpoint session `01a13402-afaa-7501-b97b-15d3b08434be`
Integration branch: `affected-family-assignment`
Starting HEAD: `e8c9c3e10ac71bf0377dc00d7de5764a4263b445`

The user requested committing existing work before planning independent,
restartable decomp sessions. They explicitly requested stopping the current
Ceres worker after its current check. No new gameplay experiment is authorized
by this checkpoint. Existing completed checks and immutable receipts are kept.

## Source and owner inventory

| Work | Source | State at checkpoint |
| --- | --- | --- |
| Shared synchronous Samus/room host | `73547105d870f8b05f8c6bcf55b768c223889fdb` | Merged by `e8c9c3e`; ten actual-C pairs and post-cleanup authentication retained. Durable integration completion still needs reconciliation. |
| Shared actual-C callback adapter | `3fcc7fd` | Committed on the integration branch. |
| Compact controller diagnostic | `223f8a1b0c7382817b1290d13c7a41f4e722d5e1` | Submitted through the original durable claim; clean worktree at `/tmp/sm-ceres-continuation-compact-controller-1`; integration pending. |
| Progress report details refinement | `crates/pipeline/src/progress.html.in` | Source, issue, browser assertions and cleanup records included in the checkpoint; registry publication is independent. |
| Natural Ridley fixture generator | `crates/pipeline/src/ridley_subsystem_captured.rs`, `tools/port/ridley-subsystem-captured.toml` | Recovered source and fixture descriptor retained; this main-checkout module is not registered in the CLI. Its presence is not gameplay acceptance. |
| Whole Ridley candidate | `/tmp/sm-ridley-whole-live-source` | Frozen source only, no Git/build metadata; authored sources backed up in the checkpoint archive. Ticket 141 remains claimed for its broader gates. |
| Active-PPU Ridley candidate | `.scratch/decomp-port/ridley-hardware-recovery/source` | Frozen source and retained reader; authored sources backed up in the checkpoint archive. |
| Earlier Ridley snapshot | `.scratch/decomp-port/ridley-natural-recovery/snapshots` | Authenticated small source snapshot backed up in the checkpoint archive. |

The continuation root `01a133eb-76e2-7073-9aef-d97c03a812e4` failed with
`Encrypted function output content could not be decrypted or decoded`.
Its worker `01a133ec-4ef9-78b2-a8d7-248726023ce5` and reviewer
`01a133f4-de2d-71e3-82ce-ead915ca3017` were read directly: both finished
and are idle. The worker reports all child processes finished. An independent
process check found only the intentional progress server, PID 259921, bound to
`127.0.0.1:4185`. The stopped worker's committed source remains owned until
submission/integration is recorded; elapsed time does not release its claim.

## Controller evidence and remaining work

The worker reports eight unit tests and two public prefix tests passing.
Its final completed check and read-only review are recorded in its task log.
Both scoped cleanup passes returned zero bytes; the successful short verdict
authenticated afterward.

Retained verdicts under `out/port/ceres-continuation/controller/`:

| Run | Result | SHA-256 |
| --- | --- | --- |
| `1791873256094852042-256891` | 2,240 frames; 1,736 updates and 1,887 samples match; RNG control rejects; 46.71 seconds; diagnostic only | `79c00cb7bb32a120f981b1c14f99b639e773d88f5f544bdb41e964409ce7a916` |
| `1791873514143170077-260799` | Complete 8,600-frame native capture; subject timeout retained | `7c8c04d2f53d3f7a247a715a807d2dcf0e2d0004be13facd780c3d74a717fe88` |
| `1791874177761738742-271877` | Final 8,600-frame diagnosis; prefix ends at 7,740 updates / frame 8,317 | `051a7195bf0ed3461eae81a9789f052a01f5c616d0d7ed716a37848c2a59d07e` |

The first provenance divergence is ordinal 7,741, original frame 8,508,
event 63,799: 7,960 C polls versus 7,959 native polls. Investigate the extra
continuation poll during setup, including NMI disable/enable. The worker made
no corrective scheduler change. Ordinary-reset Ceres escape, original-ROM
parity and credits progression remain open.

## Retention and restore

Git retains authored source, local issues, assignment state, fixture descriptors,
small reports, source snapshots and receipt archives. Captured WRAM/ROM/PPU data,
bulk generated runs, build trees and frozen reader executables remain local at
their authenticated paths. Ignore rules distinguish these artifacts without
ignoring `.scratch/` issues or specs. Existing traced data is never added to Git.

`checkpoint/frozen-sources-20261013.tar.gz` backs up the three source-only
contexts above. External dependencies, ROMs, movies, run outputs, build trees,
Git metadata and legacy Python files are excluded. Originals are preserved.
Extract from `/` on the same machine to restore their original source paths:

```sh
sha256sum -c .scratch/decomp-parallel/checkpoint/archive-sha256.txt
tar -xzf .scratch/decomp-parallel/checkpoint/frozen-sources-20261013.tar.gz -C /
tar -xzf .scratch/decomp-parallel/checkpoint/raw-receipts-20261013.tar.gz
tar -xzf .scratch/decomp-parallel/checkpoint/generated-receipts-20261013.tar.gz
```

Inspect archive members before restoring elsewhere. The source backup does not
replace sealed gameplay evidence, shared `out/evidence` objects, subject
`evidence.toml` pointers, retained readers or private inputs. A new session on
this machine can use their current paths. A new machine needs those dependencies
restored and authenticated separately; a Git clone alone is not a replay backup.

## Completion evidence

The original `compact-controller` claim was recorded as submitted by the existing
Rust infrastructure driver. The ledger retains its source ownership and creates
`integrate-compact-controller`; host integration remains pending reconciliation.
The stopped workers and clean branch establish a source checkpoint, independently
of any later gameplay acceptance.

Source backup: 537 regular files, 22,459,547 bytes, compressed to 1,954,062 bytes.
Raw receipt backup: 810 files, 10,887,631 bytes, compressed to 712,812 bytes.
Generated receipt backup: 94 files, 188,625,596 bytes, compressed to 28,539,068
bytes. Exact large TOML/JSON receipts are archived instead of expanding Git
diffs; originals remain at their sealed paths. Staged Markdown handoffs may
change as work status; hashed run artifacts remain byte-for-byte.
All archive/member hashes and the current report snapshot hashes pass.

Final commit, staging review, focused checks and cleanup measurements are appended
before closing this checkpoint.

`cargo test -p pipeline --locked --offline --bin sm-pipeline progress::tests`
passes all 19 tests. Existing final desktop/offline/file-picker assertions and
cleanup records pass; the generated report and shared registry authenticate
against their final snapshot list. `git diff --cached --check` passes after
archiving raw outputs with their original whitespace.

Both scoped cleanup passes on `.scratch/ceres-continuation` succeed with zero
bytes and zero deduplicated snapshots. The unsealed infrastructure-driver build
tree was 152,780,387 apparent bytes; `cargo clean --manifest-path
.scratch/ceres-continuation/Cargo.toml --target-dir .scratch/ceres-continuation/build`
removes 467 files (170.2 MiB reported). The controller worktree is submitted and
unintegrated, so its source and reader remain intentionally available. Existing
integrated host implementer worktree was already removed. The frozen source
contexts and intentional progress server are retained. Filesystem available
space after this cleanup: 224,972,095,488 bytes.
