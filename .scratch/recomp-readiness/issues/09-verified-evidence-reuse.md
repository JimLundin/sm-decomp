# 09: Reuse evidence only when its dependencies remain unchanged

**What to build:** Validation can safely reuse verified evidence for eligible CPU-only candidates and explains exactly why any check must run again.

**Blocked by:** 04: Verify original-core provenance and strict synchronization; 07: Restore paired checkpoints validated against cold execution.

**Status:** ready-for-agent

- [ ] Verify evidence origin, digest, run ID, scope, completion and dependencies before reuse; record reused run IDs and the verified unchanged inputs in the new manifest.
- [ ] CPU-only increments may reuse accepted independent candidate synchronization and fallback evidence only when hardware, reference execution, input/reset, observer, comparator, features and settings are identical.
- [ ] Shared-path changes invalidate affected checks and require fresh cold fallback parity and independent candidate synchronization; uncertainty defaults to fresh execution.
- [ ] Documentation-only changes are irrelevant only when executable/build inputs are verified unchanged; check/lint reuse likewise requires identical executable/build inputs.
- [ ] Generated candidate acceptance always requires its own fresh cold parity; changed-key coverage and focused validation remain candidate-specific.
- [ ] Apply the paired-cache compatibility and equivalent-history contract rather than treating identical filenames, old verdicts or profile labels as provenance.

