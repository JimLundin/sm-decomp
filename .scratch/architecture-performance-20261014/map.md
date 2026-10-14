# Architecture performance work map

Status: resolved

- [01: Confined output accounting](issues/01-accounting.md) — Type: task; resolved; `449f1dc` (source `7a481ca`, tests `054b969`).
- [02: Runnable Fixture capacity](issues/02-scheduling.md) — Type: task; resolved; `d593bc8`.
- [03: Authenticated build preparation](issues/03-builds.md) — Type: task; resolved; `3a732b1`, `0ddfae6`.
- [04: Authenticated evidence restoration](issues/04-evidence.md) — Type: task; resolved; `35ef637`.
- [05: Live replay transactions](issues/05-live.md) — Type: task; resolved; `0e57ff9`.
- [06: Main integration](issues/06-main-integration.md) — Type: task; resolved; verified source `c0bd906`.

## Decisions so far

The user authorized all five candidates in separate worktrees, then requested
their integration and a ready main workspace. Candidate01 adopts the existing
ticket-10 owner's reviewed result. All five are now on local main, composed with
the later Native waves and strict reservations. Production Installation,
policy and gameplay stages retain their separate acceptance requirements.

All five compose without conflicts at `bc5c1cf9ba2965ef6513273deb4880a93d770cc9`
on `architecture/perf-composition-20261014`. **32 combined controls passed**,
both retained actual-C verdicts authenticate after cleanup, and the temporary
composition worktree/build outputs were removed. See [composition](composition.md)
for historical sources, binaries, tests and cleanup. The latest source
`c0bd9061572ce56e4ef5218c8f60dd05aa25bd82` passed **47 controls**, including the
complete 16-pair Native proof. The five implementer worktrees are retired and
their branches retained. Authoritative documents are tracked real files in main;
the previous symlink to 03-builds is replaced. See [integration](integration.md)
for main readiness, retained evidence and final cleanup.

## Performance evidence

| Candidate | Established result | Scope |
| --- | --- | --- |
| Accounting | 7.24x file / 4.39x pipe steady loops; 2.72x / 2.61x scaling at four workers | Fixed confined producer, populated retained-file workload |
| Scheduling | Peer wait 622–633 ms -> 0.2–10.4 ms | Three Fixture pairs, fixed 600 ms root delay, fresh/fork |
| Builds | Compiler identity probes 8 -> 1; eight v4 objects and three helpers reused | No material C wall-time improvement established |
| Evidence | Inventory 13.7–25.1%, verification 12.8–26.0% faster | Identical sealed synthetic stores, ABBA comparison |
| Live | Warm adapter allocations 20,000 -> 0; refused WRAM copies eliminated | Synthetic adapter overhead; accepted timings overlap |

These results do not establish a full-pipeline speedup or global optimality.
No authenticated complete native Live replay was run; the compiled Live/Ridley
controls and their limits are recorded explicitly. The sandbox is a separate
binary selected by installed path/hash; staged combined binaries do not change
production selection.
