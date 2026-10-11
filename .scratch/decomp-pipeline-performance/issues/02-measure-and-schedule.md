# Record stage costs and enforce a shared worker budget

Status: resolved
Blocked by: 01
Triage: ready-for-agent
Type: task

Add monotonic elapsed timing to batch jobs, source validation, compiler commands,
fixture execution, evidence hashing/authentication, cleanup and live captures.
Separate store-lock wait from producer/collector activity; currently the producer
timer starts before the lock is acquired. Record CPU and peak staging usage where
available. Failed jobs must retain timings and diagnostic output.

Introduce configurable local concurrency with one resource budget shared by
worktrees, separate from evidence/run ownership locks. Avoid nested fan-out where
eight family workers each start four subjects or compilers. Use conservative
defaults and explicit CLI overrides; resource tokens must release on failure.
Limit the batch record lock to its actual batch, with atomic latest-pointer updates,
so separate batch records in one checkout can run independently.

## Acceptance

- Timings distinguish a deliberately queued store lock from actual replay time.
- Independent batches progress concurrently; the same record cannot have two writers.
- Interrupted/failed workers release scheduling tokens and preserve evidence.
- Compare worker counts under one-machine load; keep exact verification gates.
- Complete scoped cleanup and record reclaimed bytes/free space.

## Completion evidence

Rust OS leases share one bounded process budget across worktrees; per-batch ownership and atomic pointers allow independent batches. Timing records distinguish worker/store wait, producer and sealing. Process death, rejection and interruption checks pass. Store-before-worker acquisition avoids CPU slots blocked on publication.

Integration evidence and actual measurements: [implementation.md](../implementation.md).
Final serial/parallel 45000 integration passed and reauthenticated after cleanup.
Scoped cleanup and deduplication finished (zero additional bytes on final passes);
prior scoped snapshots reclaimed 1768292352 bytes and disposable builds reclaimed
843648190 bytes. Shared evidence dependencies preserved; final free bytes
256302972928. Integration branch: `decomp-pipeline-performance`.
