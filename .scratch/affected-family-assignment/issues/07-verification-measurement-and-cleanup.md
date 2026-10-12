# Verify focused work, measure selection and finish cleanup

Status: open
Blocked by: 06
Triage: ready-for-agent
Type: task

## Required work

Run small actual-C/Rust controls covering selected/skipped families, imported
dependencies, new modules/untracked files, one manifest, assets, rename/deletion,
missing/stale/incomplete closure and shared-input fallback. Run concurrent claim,
restart, stale-result, research promotion, writable conflict and checked recovery
controls. Keep existing reuse/admission rejection controls.

Measure bounded before/after selection on a representative frontier and report
families/artifact authentication work avoided, elapsed time and limitations.
Use at most approximately60seconds per initial experiment. No prospective speedup
claim without results. Complete appropriate formatting, Rust tests and Clippy
against frozen final source. Full Live replay is required if implementation has
changed execution or admission contracts; otherwise focused controls fulfill the
agreed scope.

Run both scoped cleanup passes for all finished test/measurement directories,
including failures. Remove integrated implementer worktrees and disposable build
outputs, preserve source/evidence dependencies, reauthenticate retained evidence
after cleanup/relocation and record reclaimed bytes/free space. Resolve the spec
only after every required ticket completes.

## Comments

Final evidence and cleanup are owned by the integration coordinator.
