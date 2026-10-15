# Offline family architecture completion

Status: final verification in progress
Production source: `4a736587af557de9e44b3126ff08f6d6d11beee8`
Integration branch: `architecture/offline-verifier-20261015`
Main integration: pending final gates and cleanup

## Result

| Owner | Responsibility |
| --- | --- |
| `offline-verification` | Ordinary and Native Offline family verification, Fixture codec/catalog, private recipes, scheduling, comparison, receipts and authentication |
| `tool-execution` | Generic processes, compiler cache, worker capacity, bounded IO, publication and metrics |
| `decomp-source` | Pinned Recovered C source, source inventories/definitions and operation-local source cache |
| `pipeline` | CLI/batch/Live composition, Translation approval, installed-controller policy and cleanup |

The three extracted libraries have no dependency on `pipeline`. Ordinary and
Native execution share private recipe descriptions. Current evidence schemas,
complete Fixture domains, deadlines, ownership and retry without relaunch are
preserved. Conservative source stamps and affected-family selection remain;
precise selection and CLI-only independence are deferred as approved.

## Current proof

Retained parent:
`/home/jim/sm-decomp-offline-architecture-proofs/plm-fix-20261015`.
`source-commit.txt`, `tracked-source.sha256`, `prerequisites.sha256` and the
`frozen-source/` and `catalog-source/` snapshots identify the exact source and
private inputs. The recorded archive, ROM and captured Ridley hashes match the
reviewed prerequisites. Absolute Original capture dependencies remain in place.

| Gate | Result | Evidence under retained parent |
| --- | --- | --- |
| Formatting | passed | `logs/format.log` |
| Workspace all-target check, locked/offline | passed, 46.27 seconds | `logs/check.log` |
| Dependency direction | passed | `logs/dependencies.txt` |
| Corrected verifier tests | 41 passed, one optional proof ignored | `logs/regression-3.log` |
| Release build | passed, 20m24.05s; helper build passed, 1m20.54s | `logs/test-build.log`, `logs/helper-build.log` |
| Current owner suites | 65 unit tests and one dispatcher integration test passed; one optional measurement ignored | `logs/decomp-source.log`, `logs/tool-execution.log`, `logs/offline-verification.log`, `logs/dispatcher-pointer.log` |
| Current application suites | 165 unit tests passed, 21 ignored, 229.35s; CLI integration pending | `logs/pipeline-unit.log`, retained current test executables |
| Ordinary actual-C proof | passed, 16 pairs / two routines; post-cleanup authentication passed | `ordinary/1792064366867709-1385229-0/verdict.toml`, `logs/ordinary-authenticated-after-cleanup.toml` |
| Complete catalog preparation | passed, 177 declarations / 150,204 pairs plus 360 auxiliary pairs, 38.47s | `catalog/`, `logs/catalog.log` |
| Twelve Native recipes, 8,159 Fixture pairs | failed in PLM finalization after all 1,969 PLM comparisons passed; no Native receipt | `recipes/`, `logs/recipes.log`, [diagnostic](native-deadline-diagnostic.md) |
| Full actual-C Native family and negative controls | pending | `full-real/` |
| Retained authentication after cleanup and retirement | pending | `logs/` |

The two focused regressions compile and execute generated PLM wrappers in
ordinary/confined placement with canonical/renamed modules, and authenticate a
saved flat-PLM rejection after deadline expiry with absent helper/compiler paths.
The replay preserves receipt hashes and mtimes and launches no operation.
Existing oversized-coverage/extent rejection remains passing.

The stopped original coordinator's final verification is recovered by
`/root/recover_extraction_verification`, under integration owner session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`. Source remains frozen and no installed
policy or production claim is changed. Exact retained tools are hashed in
`tools/executables.sha256` and `tools/integration-executables.sha256`; the
complete execution environment is in `logs/recovered-execution-environment.toml`.
The recovery checkpoint records current processes and remaining gates.

## Historical evidence

The earlier parent `15d5a7e-20261015` remains unchanged. Its library/application,
catalog and ordinary actual-C checks passed. Its first five Native families
passed 2,192 Fixture pairs before PLM exposed the corrected wrapper/accounting
issues. All five retained outcomes authenticate independently after both cleanup
passes; every receipt/result hash and precise mtime remains unchanged.
These are historical results, not proof of the corrected production source.

Tracing that large authentication exceeded its unchanged 60-second input
deadline. The untraced retained reader passed in 46.98 seconds. Keep both the
diagnostics and passing result; no deadline was enlarged.

## Cleanup

All recipe, foundation, Native and caller implementer worktrees are retired;
their source and handoffs are integrated. Final assembly worktree retirement is
pending the current proofs. The checkpoint records earlier measured reclamation.

Both cleanup passes over the corrected-source regression TMPDIR succeeded.
Removing two unsealed synthetic worker binaries reclaimed 9,056,256 allocated
bytes (`logs/regression-disposable-build-bytes.txt`). Their diagnostic sources,
metrics and stderr remain. Current build output stays until the final reader
and all required evidence have been retained and authenticated.

Recovered owner, ordinary and catalog runs have completed both cleanup passes.
Ordinary deduplication reclaimed 4,194,304 bytes across 48 snapshots and its
retained verdict authenticated afterward. Empty owner/ordinary/catalog TMPDIRs
were cleaned and removed. Generated catalog PLM source remains in place, sealed
by `logs/generated-catalog-plm-inputs.sha256`.

The failed PLM recipe attempt completed both cleanup passes, reclaiming
430,571,520 bytes across 5,907 snapshots. All intent/verdict/index/chunk/result
hashes and mtimes are unchanged; no writer remains. Its Native authentication
is incomplete because finalization exhausted its unchanged 60-second budget
before publishing a receipt. The other eleven recipes are unexecuted.

The recovered pipeline unit suite also completed both cleanup passes, with zero
bytes reclaimed. Its fourteen small synthetic launch-plan/state files are
retained; its empty compact-recovery directory was removed. At the integration
owner's instruction, verification is handed back for the focused admission
repair, without launching CLI integration, full-real Native or another matrix.
All recovery writers have stopped. Worktree retirement remains with that owner.

Ordinary authentication passed again at handoff. PLM diagnostic seals and mtimes,
private prerequisites and exact executable hashes also passed again. Total
recovery utility reclamation is 434,765,824 bytes; final observed free space was
126,212,780,032 bytes. Native reauthentication remains pending because the failed
attempt has no Native receipt. Final architecture admission remains incomplete.
