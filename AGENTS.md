# Agent instructions

Read [docs/PLAN.md](docs/PLAN.md) first, then [docs/REVIEW.md](docs/REVIEW.md).
PLAN owns the implementation loop, verification, Git concurrency and resource
limits. REVIEW's **Milestone plan** takes priority over PLAN's work order; re-read
it at the start of each item and take the next unclaimed lane item. The reviewer
may change goals/process (user grant, 2026-10-25). End each commit body with `Score:`.

Before changing subsystem boundaries, state ownership, or backend architecture,
read [the accepted destination](docs/target-architecture/README.md), including
its Windows migration policy. Read linked technical contracts when the task
reaches them; current work and checks still follow PLAN.

Follow PLAN's Fast loop and definition of done: one focused change, named local
rows, `sh tools/progress.sh`, then commit/push with the result and explained
metric increases. Preserve behavior and SNES RAM compatibility. If stuck for
over an hour on an item, record findings in PLAN and stop.

`sm/src` is authoritative; there is no C to regenerate from. Resolve unclear
behavior with the oracle, `ram_map.rs` and [bank logs](https://patrickjohnston.org/bank/).
Keep ROMs and derived game data out of Git. Improve the existing verifier instead
of adding process tooling, evidence stores or dashboards. Never reserve Git locks.

Pre-2026-10-17 workflows, `.scratch/`, readiness gates, acceptance pointers and
translation scopes are inactive in `archive/pre-transpile-2026-10-17`.
