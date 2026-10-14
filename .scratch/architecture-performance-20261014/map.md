# Architecture performance work map

Status: resolved

- [01: Confined output accounting](issues/01-accounting.md) — Type: task; resolved; `449f1dc` (source `7a481ca`, tests `054b969`).
- [02: Runnable Fixture capacity](issues/02-scheduling.md) — Type: task; resolved; `d593bc8`.
- [03: Authenticated build preparation](issues/03-builds.md) — Type: task; resolved; `3a732b1`, `0ddfae6`.
- [04: Authenticated evidence restoration](issues/04-evidence.md) — Type: task; resolved; `35ef637`.
- [05: Live replay transactions](issues/05-live.md) — Type: task; resolved; `0e57ff9`.

## Decisions so far

The user authorized all five candidates in separate worktrees. Each implementation
is committed under `/home/jim/sm-decomp-architecture-20261014/01-accounting`,
`02-scheduling`, `03-builds`, `04-evidence`, and `05-live`. Candidate01 adopts
the existing ticket-10 owner's reviewed result. This effort changed no main
source, installed controller, production policy or gameplay stage.

All five compose without conflicts at `bc5c1cf9ba2965ef6513273deb4880a93d770cc9`
on `architecture/perf-composition-20261014`. **32 combined controls passed**,
both retained actual-C verdicts authenticate after cleanup, and the temporary
composition worktree/build outputs were removed. See [composition](composition.md)
for exact sources, binaries, tests and cleanup. The five requested worktrees
remain available; authoritative coordination documents live in 03-builds and
main's `.scratch` link exposes that same directory.

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
