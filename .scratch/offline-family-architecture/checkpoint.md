# Offline architecture implementation checkpoint

Updated: 2026-10-15
Owner session: `01a13c3b-d421-7bb3-80a2-ec91045678e7` (`/root`)
User authorization: approved the rendered design and instructed "go ahead".

## Source and ownership

- Startup main HEAD: `77022896e6e92abe69c27c1c0f9786b838d63ba9`.
- Existing root changes are this session's glossary/review/design documents.
- Installed production `translation-production --checkout /home/jim/sm-decomp
  status` completed successfully and reported the existing active registry,
  retained host integration and prerequisite-blocked original work. This session
  does not adopt, resume or modify those claims or installed policies.
- Preserve the submitted compact-controller worktree and all original
  host/pause/controller sources and evidence. Existing mutation-testing and
  production-evidence worktrees are not ours to remove or reuse.
- No existing managed worktree attachment is available for this task. Source
  implementation will use dedicated worktrees from the approved current baseline.
- Shared process capacity: `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`,
  `SM_PIPELINE_WORKERS=4`. Cargo checks use `--locked --offline`.

## Current progress

- Ticket 01 is resolved; approved documents were checkpointed at `a6d53c5`.
- Ticket 02 is resolved. Main contains foundation implementation `e430361`
  (worker `446dd64`) and verification follow-up `d2cf88c` (worker `aeb6632`).
  Thirty-five distinct focused tests passed. Both cleanup passes succeeded;
  the stopped, clean foundation worktree was removed after integration.
  See `handoff-foundations.md` and ticket 02 for measured cleanup.
- Ticket 03 is active. Root verifier worktree is at `f9b0e49`, including
  common recipes `3166662`/`b0ab6d7`, compiler consolidation `996ed62` and
  the foundation follow-up `8ce4a83`. Ordinary and confined builds now share
  exact compiler descriptions. Producer authentication uses retained source
  snapshots and provenance without comparing today's executable identity.
- `/root/gameplay_architecture` is finishing Fixture command consolidation
  from `996ed62` in its retained recipe worktree, on branch
  `architecture/offline-fixtures-20261015`. It owns adapter case sections and
  private recipe command/dispatch code; root owns other verifier changes.
- `/root/translation_architecture` owns the Native extraction and application
  bridge. The opaque prepared handle owns indexed operation traversal and
  retained admission. Its proof migration includes read-only authentication
  after cleanup and implementer retirement. See `native-verification-plan.md`.
- Ticket 04 is claimed by `/root/pipeline_architecture` in the caller worktree
  from `b0ab6d7`. It owns non-Native pipeline callers, direct lower imports,
  removal of old verifier modules, source-copy constructors and conservative
  affected selection. Native-owned callers remain with the Native worker.
- Baseline verification at unchanged Rust source `a6d53c5` finished exit 0:
  31 passed (affected 9, callback 1, family 19, Fixture 1, PLM 1), two optional
  performance measurements ignored. Log:
  `out/offline-architecture/baseline-20261015/tests.log`, SHA-256
  `ba041cd4d4147072c3e04b20f9fd2e5178ca204e35101c10ab1fafece7cebdde`.
  Both scoped cleanup receipts report zero bytes and zero snapshots. Its empty
  TMPDIR was removed; do not rerun this completed baseline.
- Root owner unit tests are running in exec session `65713`; log
  `out/offline-architecture/verifier-unit-20261015/tests.log`, TMPDIR below
  that directory. `SM_PIPELINE_CLEANUP_BIN` points to the retained baseline
  `target/debug/sm-pipeline`. No final combined/ignored Native gate has run yet.
- Next: integrate the Native/caller/Fixture commits, remove private forwarding
  helpers, verify the final dependency/source interfaces, then run the required
  combined ordinary and explicit Native proofs on frozen source. Main Git/index
  and final cleanup remain root-owned. Installed production state is unchanged.

## Worktree and run inventory

| Owner | Checkout | Branch | Baseline | Responsibility |
| --- | --- | --- | --- | --- |
| `/root/pipeline_architecture` | `/home/jim/sm-decomp-offline-foundations` (removed) | `architecture/offline-foundations-20261015` | `a6d53c5` | Ticket 02 integrated and cleaned |
| `/root` | `/home/jim/sm-decomp-offline-verifier` | `architecture/offline-verifier-20261015` | `446dd64` | Ordinary verifier, crate/public interface, source contracts, catalog, execution imports and ordinary callers |
| `/root/gameplay_architecture` | `/home/jim/sm-decomp-offline-recipes` | `architecture/offline-fixtures-20261015` | `996ed62` | Complete shared Fixture command/dispatch wiring |
| `/root/translation_architecture` | `/home/jim/sm-decomp-offline-native` | `architecture/offline-native-20261015` | `446dd64` | New private Native implementation/operations, pipeline Native bridge/proofs and Native caller migration |
| `/root/pipeline_architecture` | `/home/jim/sm-decomp-offline-callers` | `architecture/offline-callers-20261015` | `b0ab6d7` | Ticket 04; non-Native application callers and source contracts |

Native caller ownership includes `translation_execution.rs`,
`translation_original.rs`, `translation_original_prepare.rs`,
`translation_original_tests.rs`, `translation_migration.rs`,
`translation_execution_tests.rs` and `translation_managed_tests.rs` for
Native-specific changes. Root handles other caller/import migrations and main
Git/index integration. Common recipe work owns new files only; root wires the
ordinary consumers after integration. No worker may overwrite another's edits.

Workers record exact commands and retained outputs in their own handoff before
returning code. Baseline fixture test workspaces are confined to the TMPDIR above;
their existing Drop implementations invoke project cleanup before removing them.
