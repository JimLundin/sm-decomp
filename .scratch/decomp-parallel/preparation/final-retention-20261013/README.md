# Final preparation retention audit

Owner: `/root/retention_audit`; 2026-10-13. This artifact directory is the only
new writable scope. No production claim, gameplay source, main Git/index or
historical evidence was edited.

## Retained source dependencies

All four frozen preparation checkouts authenticate their exact source
fingerprints, each covering 435 source files. The new Rust read-only inspector
implements the maintained fingerprint's ordered path/hash serialization and
checks its expected hash. Its `*-source.sha256` manifests and
`*-source-reauthentication.txt` outputs retain each individual source seal.

| Exact checkout | Full HEAD | Required dependency |
| --- | --- | --- |
| `/tmp/sm-decomp-preparation-integration-20261013` | `c8799f326ffe198b652301f08866f1b7a7bd9ebc` | Historical original `drill/concurrency-drill.toml`, `readiness/current-owners/prepare-request.toml` and `ticket-validator-evidence/validator-build.toml` executed-subject identity |
| `/tmp/sm-decomp-preparation-final-source-20261013` | `2d59536c059a1ca9c31d9278bff2d9222eafa56c` | Historical `drill-final` source/build identity and retained completed-retry failure |
| `/tmp/sm-decomp-ticket-validator-20261013` | `f54fbed0f30f2c84e2c6aba171dcc54e96099c9a` | Historical separately authored validator source/build identity |
| `/tmp/sm-decomp-preparation-retry-source-20261013` | `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd` | Current actual retry drill/readiness engine and exact activation source |

The historical integration checkout is a frozen evidence subject, although its
branch name still says integration. Maintained `authority.rs::checked_drill`
requires its actual `HEAD`, common Git root and exact canonical checkout path;
`drill_evidence.rs` requires its retained filesystem fingerprint. Removing Git
or retaining only a few files would break this contract. The historical
readiness README explicitly requires keeping frozen subject and isolated Git.
Therefore all four frozen Git subjects remain, together with the historical
and current isolated fixture Git repositories. No disposable targets were
found in these checkouts. Integrated serial-fix and completed-retry implementer
worktrees were already retired by their recorded integrator cleanup; there is
no unretired disposable preparation implementer among the current registrations.
The unresolved Ceres compact-controller worktree is preserved under its
original held submission and acceptance requirements.

## Completion evidence

The process inspector found zero matching cwd/open-fd references to the
historical integration subject and each finalized evidence scope. Thirty-nine
unrelated process fd directories were inaccessible; this is an observation
limit, and no deletion relies on it. Process inspection and commands remain
under this directory. Utility normal and deduplication passes succeeded over
these seven completed scopes:

- `preparation/integration-evidence`
- `preparation/03-session-evidence`
- `preparation/integration-handoff-evidence`
- `preparation/ticket-validator-evidence`
- `preparation/serial-fix-evidence`
- `.scratch/decomp-integration-retry`
- This `final-retention-20261013` output directory

Each scope ran the retained frozen project utility:

```sh
.scratch/decomp-parallel/drill-retry/bin/sm-pipeline cleanup --root SCOPE --apply
.scratch/decomp-parallel/drill-retry/bin/sm-pipeline cleanup --root SCOPE --deduplicate --apply
```

Each pass returned `bytes = 0` and succeeded. The executed retained project
binary is SHA-256
`04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573`.
Receipts `*-cleanup1.toml` and `*-cleanup2.toml` retain exact applied output.
All six completed preparation artifact/relocation manifests and the completed
retry worker manifest pass after cleanup. Historical original/final drill
recursive retained-artifact authentication passes 565 dependencies via the
existing read-only independent reader. These reports authenticate the retained
source/readers/private dependency paths without moving or resealing history.

Measured reclamation: **0 bytes**. No worktree or build directory was removed.
Filesystem available space after the scoped passes: **221,316,112,384 bytes**.
Before/after `df -B1`, logical directory measurements and retained worktree
list are attached. Other agents wrote independent outputs concurrently, so a
filesystem free-space difference is not treated as cleanup reclamation.

Final current drill/supplement semantic authentication is published separately
in this directory after the harness owner finalizes its immutable manifest.

The supplemental interruption/replay semantic review and auth314 are now in
`final-drill-review.md` and `final-drill-authentication.json`; final maintained
validation needs the final manifest owner to consume matching absolute prompt
metadata copies before refreshing seals. Our delegated parent was directly
observed idle with encrypted-output failure. The independently active recovery
coordinator `01a13559-e517-72d1-88b5-e554fed61da4` was informed through the
existing task message tool at the drill delegate's explicit coordination request.
This reviewer left canonical publication to that owner.

Final output cleanup was repeated after the review; both passes again returned
zero bytes, and all three retained reader seals authenticate. Last available
filesystem space observation: **221,237,899,264 bytes** (`disk-final.txt`).
