# Rust migration merge

The user requested commit and merge of the completed Python-to-Rust and internal
JSON migration. Integration preserves the newer native gameplay work, including
blue-ring and Mother Brain bomb initializer routing, projectile continuations,
the synchronous eight-word host ABI, WithGfx allocation, projectile rendering,
active-pool observation and batch admission.

Migration commit: `c89db9bcf824a3c4f973550d390a95c3f5d05f97`.
Target branch: `decomp-port-integration`, prior HEAD
`0fe47e36698e8530655e46ceffb0066c227e0815`.

Thirty newer-main JSON manifests were converted and roundtrip checked. Nine newer
Python generators were replaced with native declared fixture matrices and actual
C sequence capture. The strict typed manifest model retains the newer domain
description and the existing offset/data write contract.

## Focused verification

- Full port tests: `sm-merge-port-tests.log`.
- Pipeline unit tests: 57 passed; complete fixture generation was covered before
  the merge and excluded from this bounded merge check: `sm-merge-pipeline-unit.log`.
- Rendering adapter route regression: `sm-merge-rendering-route.log`.
- Family controls: 8 passed, `sm-merge-rest-controls.log`.
- Actual callback/ordered snapshots: 1 passed, `sm-merge-callback-diagnosis.log`.
- Strict Clippy for pipeline and port, all targets: `sm-merge-clippy-final.log`.
  Workspace formatting and conflict checks passed.
- A read-only metadata audit across the fixture matrices found only the missing
  domain field; it is now typed. A strict tuple-write probe verified both offsets
  and bytes. The optional complete matrix parse was stopped after several minutes
  without an additional error; no completed full parse is claimed for the merge.
  The pre-merge complete fixture/config checks remain documented in completion.md.
- Observations: 4 passed, packed publication/failure retention: 1 passed, actual
  quicksand allocation/tampering: 1 passed, `sm-merge-final-evidence-controls.log`.
  The current sm-evidence reader was built first. An earlier callback run rejected
  while merge sources were still being finalized; the diagnostic rerun passed.
- No maintained Python/internal JSON controls passed in the main checkout after
  removing its old tools/__pycache__. External corpus ingestion and immutable
  historical evidence remain as documented in completion.md.
  Final rerun: `sm-merge-native-guards.log` (2 passed).

## Cleanup

Test-owned temporary runs were removed by their cleanup guards. Packed finished
run cleanup reclaimed zero additional bytes (`sm-merge-packed-cleanup.toml`).
The integration Git worktree was removed after a read-only retention audit and
preserving exact source snapshots plus out, dependency/ROM symlinks and movies at
the same absolute paths. No retained evidence depends on target binaries; the
demonstration driver paths are provenance and their sealed copies live under out.
Removed target: 15566278634 logical bytes; redundant tools/port matrices:
157929412 logical bytes. Retained source crates occupy 3.2 MiB and tools 124 KiB.
Shared evidence stores, pointers, sealed readers and diagnostics remain retained.
The temporary converter target was also removed (193.0 MiB,
`sm-merge-converter-clean-schema.log`). The main build/cache and unrelated
developer worktrees are retained. Packed
authentication after removal passed with the unchanged verdict hash
`b37175079604dd3a027167844523f9e882e64e99bef6038e08b48bbd93cb04ab`
(`sm-merge-packed-auth-after-retention.toml`). Disk free after worktree removal:
230636158976 bytes (about 214.8 GiB).
