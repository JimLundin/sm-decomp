# Agent instructions

Read [docs/PLAN.md](docs/PLAN.md) first, then [docs/REVIEW.md](docs/REVIEW.md).
REVIEW's **Milestone plan** takes priority over PLAN's work order; re-read it at
each item and take the next unclaimed lane item. The reviewer may change goals
and process (user grant, 2026-10-25).

Use the specific self-contained skill for an explicit objective:

| Objective | Skill |
| --- | --- |
| Source implementation | [sm-decomp-source](.agents/skills/sm-decomp-source/SKILL.md) |
| Source/composition review | [sm-decomp-review](.agents/skills/sm-decomp-review/SKILL.md) |
| Build qualification | [sm-decomp-build](.agents/skills/sm-decomp-build/SKILL.md) |
| Replay qualification | [sm-decomp-replay](.agents/skills/sm-decomp-replay/SKILL.md) |
| Git/publication | [sm-decomp-publish](.agents/skills/sm-decomp-publish/SKILL.md) |
| Resource admission/cleanup | [sm-decomp-resources](.agents/skills/sm-decomp-resources/SKILL.md) |
| Core task coordination | [sm-decomp-coordinate](.agents/skills/sm-decomp-coordinate/SKILL.md) |
| Process improvement | [sm-decomp-improve](.agents/skills/sm-decomp-improve/SKILL.md) |

Load only that objective's reached technical contracts. Existing
[sm-decomp-work](.agents/skills/sm-decomp-work/SKILL.md) handoffs retain a short dispatcher.
PLAN owns the Fast loop, verification, Git/resources, progress and completion.

Root coordinates core work, integrates source, owns the sole Git writer and one
machine resource ledger. Delegate bounded core source implementation, build and
replay scopes with exact pins/budgets and keep ready owners supplied.
Work on main, publish frequent small qualified code commits promptly, and continue
implementation/direct handoffs while CI runs. Task issues are retired.
Keep [Jim's conversation](https://github.com/JimLundin/sm-decomp/issues/6)
open for remote feedback, read it at work boundaries and respond there.
Keep durable source/review/qualification records in the repository and owned artifacts.
GitHub updates lead with the result, identify owner and next action, and use
short spaced facts; put commands/hashes/inventories in fenced blocks or details.
Keep the conversation opening current; preserve older evidence before retirement.

Before changing ownership, interfaces or backend architecture, read the
[accepted destination](docs/target-architecture/README.md) and its Windows policy.

`sm/src` is authoritative; preserve behavior and SNES RAM compatibility. Use the
oracle, `ram_map.rs` and [bank logs](https://patrickjohnston.org/bank/) for unclear behavior.

Pre-2026-10-17 workflows, `.scratch/`, readiness gates, acceptance pointers and
translation scopes are inactive in `archive/pre-transpile-2026-10-17`.
