# Final retention successor audit

Status: scoped retention complete; current supplemental drill authentication is owned by its finalizer.
Owner: `/root/audit_retention`; integration owner `01a13559-e517-72d1-88b5-e554fed61da4`.

The predecessor [retention audit](../final-retention-20261013/README.md) completed
while this successor inspected it. Its seven scopes each have successful normal
and deduplication cleanup receipts, with zero bytes reclaimed. Its six retained
artifact/relocation manifests, completed-retry worker manifest and 565 historical
drill dependencies authenticated after cleanup. The observed predecessor cleanup
process PID 520983 had exited before this successor acted. Historical evidence
and its predecessor were left intact.

All four frozen preparation subjects freshly authenticate the exact recorded
435-file source fingerprints. The `*-source.toml` and `*-source.sha256` outputs
are retained here. Fresh cwd/open-fd inspections found zero matching references
to the historical integration, final-source and validator subjects; 39 unrelated
fd directories were inaccessible. No deletion relies on that observation limit.

No disposable preparation implementer remains among the registered worktrees.
The integrated serial-fix and completed-retry implementers were already retired
with their source/evidence preserved in their integration cleanup records.
The remaining preparation registrations are required frozen evidence subjects:

| Retained checkout | Exact HEAD | Dependency |
| --- | --- | --- |
| `/tmp/sm-decomp-preparation-integration-20261013` | `c8799f326ffe198b652301f08866f1b7a7bd9ebc` | Original drill executed-subject identity and historical readiness |
| `/tmp/sm-decomp-preparation-final-source-20261013` | `2d59536c059a1ca9c31d9278bff2d9222eafa56c` | Historical final drill and completed-retry diagnostic |
| `/tmp/sm-decomp-ticket-validator-20261013` | `f54fbed0f30f2c84e2c6aba171dcc54e96099c9a` | Separate historical validator build/source and readiness identity |
| `/tmp/sm-decomp-preparation-retry-source-20261013` | `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd` | Current actual drill, readiness and activation source |

Maintained drill validation needs the frozen source fingerprint, canonical
checkout path, HEAD and common Git root. Historical readiness also directly
reads the frozen subject and validator Git HEAD. Retaining source snapshots
without Git would break these contracts. No build target was found in these
subjects. The held legacy Ceres controller worktree remains untouched.

## Completion evidence

This successor ran the retained project utility against this new output scope:

```sh
.scratch/decomp-parallel/drill-retry/bin/sm-pipeline cleanup --root .scratch/decomp-parallel/preparation/final-retention-successor-20261013 --apply
.scratch/decomp-parallel/drill-retry/bin/sm-pipeline cleanup --root .scratch/decomp-parallel/preparation/final-retention-successor-20261013 --deduplicate --apply
```

Both passes succeeded and reclaimed **0 bytes**. Their exact outputs are
`cleanup1.toml` and `cleanup2.toml`. No worktree or output directory was removed.
Filesystem available space afterward was **221,304,995,840 bytes** (`disk-after.txt`).
Concurrent independent writes mean the filesystem-space delta is not a cleanup
reclamation measurement.

Retain the frozen source/Git subjects, independent fixture Git histories,
exact source snapshots and session/command logs, immutable acceptance/diagnostic
receipts, historical/current Rust evidence readers and their sealed builds,
the packed live shared evidence store and verdict pointers, private inputs and
the original Ceres ledger/bridge/submission dependencies. The current drill
finalizer must authenticate its final supplemental manifest after its own
cleanup; root must authenticate the readiness result after final cleanup.
No further broad retention work is required to finish this update.
