# Record stage costs and enforce a shared worker budget

Status: open
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
