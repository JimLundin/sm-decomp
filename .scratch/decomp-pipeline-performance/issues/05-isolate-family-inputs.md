# Compile and seal the actual family dependency closure

Status: resolved
Blocked by: 02
Triage: ready-for-agent
Type: task

Move canonical Wram and shared primitives into a module without gameplay module
declarations. Re-export them from the public port library. Start with system/audio
harnesses, then callback/PLM/instruction harnesses. Generate only the selected family
and necessary gameplay module declarations; derive and seal actual compiler inputs,
including external declared dependencies. Some projectile closures will stay broad.

Do not narrow dependency hashes while the wrapper still compiles lib.rs with every
gameplay module. New evidence needs a versioned dependency contract; historical
evidence retains its original conservative validation. Source path mapping must
allow authentication of identical snapshots in a different worktree without
accepting changed bytes.

## Acceptance

- Changing an unrelated module leaves system/audio evidence reusable.
- Changing Wram, selected module or an imported transitive module rejects reuse.
- Actual-C full WRAM/return tests and captured-state checks pass.
- Preserve exact compiler input snapshots and validate relocated identical evidence.
- Measure recompilation and invalidation reduction; complete cleanup.

## Completion evidence

Canonical Wram moved without behavior change into memory.rs and reexported. Rust generates conservative narrow wrappers and seals actual compiler dependencies under compiler-v2. Unrelated audio changes leave system reusable; canonical memory and real/transitive dependencies reject. Exact source snapshots and tool identity checks retain relocated authentication.

Integration evidence and actual measurements: [implementation.md](../implementation.md).
Final serial/parallel 45000 integration passed and reauthenticated after cleanup.
Scoped cleanup and deduplication finished (zero additional bytes on final passes);
prior scoped snapshots reclaimed 1768292352 bytes and disposable builds reclaimed
843648190 bytes. Shared evidence dependencies preserved; final free bytes
256302972928. Integration branch: `decomp-pipeline-performance`.
