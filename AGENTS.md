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

## Rust style

Code in the target packages is idiomatic Rust, not transpiled C. Preserve the
original's behavior, not its C shapes:

- Iterate (`iter().enumerate()`, `iter_mut()`) instead of `for i in 0..N { a[i] }`.
- Parenthesize mixed shift, bitwise and arithmetic expressions.
- Name checks: `sm_data::table::check_index(i, N)`, never a no-op expression
  such as `[(); N][i];`.
- Use `From`/`TryFrom` and `wrapping_*` instead of chains of `as` casts.
- Use `std::fs` and owned buffers, not libc `fopen`/`fread`/`malloc`.
- Store text files with LF line endings (`.gitattributes` enforces it).

`cargo clippy -p sm_data -p sm_game -p sm_runtime --all-targets` must pass in
CI. Fix findings; do not add `allow`s for these patterns.

Checks assert behavior, not wording: never match on panic or `Die` text. The
verifier does not decide engine behavior; where an existing check forces us to
keep something it alone needs, tag it `LEGACY(<kind>)` and list it in
[docs/LEGACY.md](docs/LEGACY.md), then retire it when the check is fixed.

Pre-2026-10-17 workflows, `.scratch/`, readiness gates, acceptance pointers and
translation scopes are inactive in `archive/pre-transpile-2026-10-17`.
