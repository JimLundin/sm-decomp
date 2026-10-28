# Agent instructions

Read [docs/PLAN.md](docs/PLAN.md) first, then [docs/REVIEW.md](docs/REVIEW.md).
REVIEW's **Milestone plan** takes priority over PLAN's work order; re-read it at
each item and take the next unclaimed lane item. The reviewer may change goals
and process (user grant, 2026-10-25).

Use [sm-decomp-work](.agents/skills/sm-decomp-work/SKILL.md) for implementation,
review, build, replay and coordination. Load the assigned role and only the
technical contracts it reaches. PLAN remains authoritative for the Fast loop,
verification, Git/resource limits, progress, commit/push and definition of done.

Root owns implementation, coordination, Git, builds and the machine resource ledger.
Work on main and push coherent commits promptly. Assign bounded subagents directly;
task issues are retired. Keep [Jim's conversation](https://github.com/JimLundin/sm-decomp/issues/6)
open for remote feedback, read it at work boundaries and respond there.
Keep durable source/review/qualification records in the repository and owned artifacts.
Format GitHub comments using the still-valid
[Markdown communication directive](https://github.com/JimLundin/sm-decomp/issues/2#issuecomment-6445792069).

Before changing ownership, interfaces or backend architecture, read the
[accepted destination](docs/target-architecture/README.md) and its Windows policy.

`sm/src` is authoritative; preserve behavior and SNES RAM compatibility. Use the
oracle, `ram_map.rs` and [bank logs](https://patrickjohnston.org/bank/) for unclear behavior.

Pre-2026-10-17 workflows, `.scratch/`, readiness gates, acceptance pointers and
translation scopes are inactive in `archive/pre-transpile-2026-10-17`.
