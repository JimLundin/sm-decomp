# 09: Reuse evidence only when its dependencies remain unchanged

**What to build:** Validation can safely reuse verified evidence for eligible CPU-only candidates and explains exactly why any check must run again.

**Blocked by:** 04: Verify original-core provenance and strict synchronization; 07: Restore paired checkpoints validated against cold execution.

**Status:** done

- [x] Verify evidence origin, digest, run ID, scope, completion and dependencies before reuse; record reused run IDs and the verified unchanged inputs in the new manifest.
- [x] CPU-only increments may reuse accepted independent candidate synchronization and fallback evidence only when hardware, reference execution, input/reset, observer, comparator, features and settings are identical.
- [x] Shared-path changes invalidate affected checks and require fresh cold fallback parity and independent candidate synchronization; uncertainty defaults to fresh execution.
- [x] Documentation-only changes are irrelevant only when executable/build inputs are verified unchanged; check/lint reuse likewise requires identical executable/build inputs.
- [x] Generated candidate acceptance always requires its own fresh cold parity; changed-key coverage and focused validation remain candidate-specific.
- [x] Apply the paired-cache compatibility and equivalent-history contract rather than treating identical filenames, old verdicts or profile labels as provenance.



Implemented the shared versioned gate/domain contract, immutable dependency policies,
verified accepted-origin and history pointers, per-check reuse planning/import and
end-of-run artifact guards. Runtime/shared hardware/build/input/observer changes
force fresh fallback and synchronization; CPU-only config/emitter changes retain
reuse only with identical preserved shared dependencies. Generated, coverage and
focused checks stay candidate-specific. Checkpoint prechecks delegate restoration
to the exact-identity paired API and rebuild changed candidates cold.

Evidence: eleven reuse behavior tests pass; broad readiness suite 37 tests passes
(two optional full replay tests skipped), Rust formatting and diff checks pass.
A real preflight/reuse-plan run `20261010T085116.749117Z-932365adaf11` completes with
all checks fresh because no accepted readiness predecessor exists yet. It does
not imply acceptance. Ticket10 must import actual gate origins and seal all
mandatory evidence before publishing a verified accepted pointer; final12 performs
fresh current full-domain gates.
