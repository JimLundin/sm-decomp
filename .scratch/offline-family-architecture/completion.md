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
| Release build and owner/application suites | running | `logs/test-build.log` |
| Ordinary actual-C proof | pending | `ordinary-verify.toml` |
| Complete catalog preparation | pending | `catalog/` |
| Twelve Native recipes, 8,159 Fixture pairs | pending | `recipes/` |
| Full actual-C Native family and negative controls | pending | `full-real/` |
| Retained authentication after cleanup and retirement | pending | `logs/` |

The two focused regressions compile and execute generated PLM wrappers in
ordinary/confined placement with canonical/renamed modules, and authenticate a
saved flat-PLM rejection after deadline expiry with absent helper/compiler paths.
The replay preserves receipt hashes and mtimes and launches no operation.
Existing oversized-coverage/extent rejection remains passing.

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
