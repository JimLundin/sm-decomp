# Delivered source reconciliation

Implementation base: `3ad1bf3f02961de309279ff071aa396af942dc1e`.
Integration branch: `pipeline-upgrade-integration`.

The planning snapshot predates the delivered migration. Migration revision
`c89db9bcf824a3c4f973550d390a95c3f5d05f97` is merged into this base. All seven
local migration tickets are resolved. Its completed controls, cleanup and merge
checks are recorded in `../rust-tooling/evidence/completion.md` and
`../rust-tooling/evidence/merge-integration.md`; these are prerequisites, not
substitutes for the upgrade's fresh current-source admission.

Current `family.rs` already implements `case_order`, `continue_from`, matching
full-WRAM inheritance and all eight external host words. Current native live
templates implement `SpawnEprojWithGfx` and the synchronous eight-word bridge.
Ticket 01 verifies these delivered paths and fixes any defects found; it does
not build a duplicate migration.

Native stored evidence is `sm-evidence-v2`. Preserve that delivered format and
its observation identities. References to preserving `sm-evidence-v1` in the
planning snapshot mean preserving the existing stored evidence contract; they
do not authorize downgrading current v2 evidence. Historical sealed readers,
formats, verdicts and exact source dependencies remain unchanged. Small native
requests, pointers and new measurements use typed Rust and versioned TOML.

The producer-to-collector FIFO still carries legacy text: `effect_writer.c.in`
uses `vfprintf`, and `packed::capture` selects evidence `capture`. Compressed
binary *storage* does not establish that the proposed binary *transport* exists.
Tickets 04–08 remain conditional on ticket 03's measured selection.

Public test boundaries were confirmed by the user on 2026-10-12: family/live
CLI commands, process execution and metrics receipts, and evidence capture/import.

Run verification at capacity two in the separate shared root
`/tmp/sm-pipeline-upgrade-workers-2`. The existing main root is configured for
four; never change its capacity or mix capacities in one root. Keep evidence
under the integration checkout's `out/`, preserve source dependencies before
removing implementer worktrees, and authenticate retained evidence after cleanup.
