# Agent instructions

The goal is the new game engine in the new crates (`crates/sm_data`, `sm_game`,
`sm_runtime`, `sm_tools`, `sm_app`), behaving like the real game. `sm/` is the
compatibility layer and oracle tooling we are moving off.
[docs/REVIEW.md](docs/REVIEW.md)'s Milestone plan sets the active phase and records
Jim's decisions; [docs/PLAN.md](docs/PLAN.md) holds the standing rules.
For measured counts, retained job evidence or timing, read the dated
[progress report](docs/PROGRESS_REPORT.md). PLAN owns current coordination and
reporting rules, including Jim's pattern-wide migration work order.

Load the skill for your role first; each is short and self-contained.

| You are | Load |
| --- | --- |
| a lane agent, on any task | [sm-lane](.agents/skills/sm-lane/SKILL.md), then your task skill |
| cutting sm_game RAM coupling | [sm-decouple](.agents/skills/sm-decouple/SKILL.md) |
| fixing a faithful-only divergence | [sm-hardware](.agents/skills/sm-hardware/SKILL.md) |
| triaging drift, a lock mismatch or a failing row | [sm-drift](.agents/skills/sm-drift/SKILL.md) |
| adding a corpus scenario | [sm-scenario](.agents/skills/sm-scenario/SKILL.md) |
| proving any change | [sm-check](.agents/skills/sm-check/SKILL.md) |
| reporting or handing off | [sm-handoff](.agents/skills/sm-handoff/SKILL.md) |
| the root coordinator | [sm-coordinate](.agents/skills/sm-coordinate/SKILL.md), [sm-land](.agents/skills/sm-land/SKILL.md), [sm-resources](.agents/skills/sm-resources/SKILL.md) |

The root alone pushes to `main` and `land/*` and uploads to ci-data. Give Jim
progress updates in the conversation and keep durable evidence in the repository.

Before changing ownership, interfaces or backend architecture, read the
[accepted destination](docs/target-architecture/README.md) and its Windows policy.

`sm/src` and the real game are the authority for behaviour. Use the oracle,
`ram_map.rs` and the [bank logs](https://patrickjohnston.org/bank/) for unclear
behaviour. A difference from hardware is fixed, or listed in [docs/LEGACY.md](docs/LEGACY.md).

## Rust style

Code in the target packages is idiomatic Rust, not transpiled C. Preserve the
original's behaviour, not its C shapes:

- Iterate (`iter().enumerate()`, `iter_mut()`) instead of `for i in 0..N { a[i] }`.
- Parenthesize mixed shift, bitwise and arithmetic expressions.
- Name checks: `sm_data::table::check_index(i, N)`, never a no-op expression
  such as `[(); N][i];`.
- Use `From`/`TryFrom` and `wrapping_*` instead of chains of `as` casts.
- Use `std::fs` and owned buffers, not libc `fopen`/`fread`/`malloc`.
- Store text files with LF line endings (`.gitattributes` enforces it).

`cargo clippy -p sm_data -p sm_game -p sm_runtime -p sm_tools --all-targets` must
pass in CI. Fix findings; don't add `allow`s for these patterns.

Checks assert behaviour, not wording: never match on panic or `Die` text. The
verifier doesn't decide engine behaviour. Where an existing check forces us to
keep something only it needs, tag it `LEGACY(<kind>)` and list it in
[docs/LEGACY.md](docs/LEGACY.md), then retire it when the check is fixed.

Pre-2026-10-17 workflows, `.scratch/`, readiness gates, acceptance pointers and
translation scopes are inactive in `archive/pre-transpile-2026-10-17`.
