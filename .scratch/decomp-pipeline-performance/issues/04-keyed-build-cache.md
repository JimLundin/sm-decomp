# Reuse immutable compiler artifacts across worktrees

Status: resolved
Blocked by: 02
Triage: ready-for-agent
Type: task

Introduce one build-cache module whose interface builds or restores artifacts from
an explicit input closure and compiler command. Start with evidence reader,
decompressor and RNG, then stable pinned C translation-unit objects. Generated
candidate host/wrappers and changed banks remain separate compilation inputs.

Keys must cover exact source/header/module contents, manifest/generated source,
compiler identity and options, target/platform, relevant environment and Cargo
lock/config. Publish atomically under a per-key lock. Authenticate bytes on reuse,
copy exact linked binaries and provenance into the run, and retain source snapshots.
Keep cache retention separate from finished-run cleanup. Never reuse live execution
as a consequence of a binary-cache hit.

## Acceptance

- Cold/warm/one-source-changed builds produce passing actual-C evidence.
- Two worktrees requesting the same key perform one build and obtain sealed outputs.
- Changed dependency/options/compiler and corrupt cache artifacts invalidate reuse.
- Interrupted cache publication cannot expose incomplete artifacts.
- Record measured warm savings and cache size/retention policy; clean scoped runs.

## Completion evidence

Rust per-key locks and atomic content seals authenticate artifacts; keys cover exact inputs, compiler identity/options/platform/environment and Cargo inputs. Concurrent equivalent checkouts build once; corruption, changed header/options/input and abandoned builds reject. C objects are rebuilt only as needed; generated hosts relink fresh. Cold evidence build 90.27s; warm restore 0.228s.

Integration evidence and actual measurements: [implementation.md](../implementation.md).
Final serial/parallel 45000 integration passed and reauthenticated after cleanup.
Scoped cleanup and deduplication finished (zero additional bytes on final passes);
prior scoped snapshots reclaimed 1768292352 bytes and disposable builds reclaimed
843648190 bytes. Shared evidence dependencies preserved; final free bytes
256302972928. Integration branch: `decomp-pipeline-performance`.
