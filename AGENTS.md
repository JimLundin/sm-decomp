# Agent instructions

> **Pause: Jim's review is waiting.** Before more lane work, read
> [MODEL-REVIEW.md](docs/architecture/MODEL-REVIEW.md) and answer every row of its
> Acknowledgement table. `tools/check_review_ack.sh` fails the gate and CI until you do.

The goal is the new game engine in the new crates (`crates/sm_data`, `sm_game`,
`sm_runtime`, `sm_tools`, `sm_app`), behaving like the real game. `sm/` is the
compatibility layer and oracle tooling we are moving off.
[docs/REVIEW.md](docs/REVIEW.md) holds Jim's decisions and the milestones, and
[docs/PLAN.md](docs/PLAN.md) the standing rules. #177 holds live status; measure
counts with the tools rather than reading them from a document.

Load the skill for your role first; each is short and self-contained.

| You are | Load |
| --- | --- |
| a lane agent, on any task | [sm-lane](.agents/skills/sm-lane/SKILL.md), then your task skill |
| cutting the engine's ties to the old game (foreign calls, Game host impls, address sites) | [sm-decouple](.agents/skills/sm-decouple/SKILL.md) |
| fixing a faithful-only divergence | [sm-hardware](.agents/skills/sm-hardware/SKILL.md) |
| triaging drift, a lock mismatch or a failing row | [sm-drift](.agents/skills/sm-drift/SKILL.md) |
| adding a corpus scenario | [sm-scenario](.agents/skills/sm-scenario/SKILL.md) |
| proving any change | [sm-check](.agents/skills/sm-check/SKILL.md) |
| reporting or handing off | [sm-handoff](.agents/skills/sm-handoff/SKILL.md) |
| the root coordinator | [sm-coordinate](.agents/skills/sm-coordinate/SKILL.md), [sm-land](.agents/skills/sm-land/SKILL.md), [sm-resources](.agents/skills/sm-resources/SKILL.md) |

The root alone pushes to `main` and `land/*` and uploads to ci-data. Give Jim
progress updates in the conversation and keep durable evidence in the repository.

Before changing ownership, interfaces or backend architecture, read the
[accepted destination](docs/target-architecture/README.md) and its migration policy.

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
- Store text files with LF line endings (`.gitattributes` and CI enforce it).

Model the domain with types, not raw numbers:

- Use an `enum` for a state, mode, kind or choice instead of a `u16` code.
  Avoid magic constants and state words compared with `==`. Match on the enum
  exhaustively; don't fall back to `_ =>` when every case can be named.
- Give an enum variant the data it carries, rather than storing that data in
  sibling fields that only mean something in one state.
- Use a newtype for an index or a quantity that must not mix with others: slot,
  room, pose, frame count.
- Use a typed flag set, a struct of `bool`s or an enum set, instead of bit masks
  tested inline.
- Keep behaviour tables as data: arrays of typed records or enums, plus a small
  interpreter. Don't repeat a code path once per entry.
- Convert at the edges. Turn ROM bytes and checkpoint formats into the enum
  once, with `TryFrom`, and keep the raw value out of engine logic.
  - When the original treats an invalid value as an error, map it to
    `Fatal::unreachable`.
  - When the original tolerates an invalid value, keep it as a named variant
    (for example `Unknown(u16)`) and list it in docs/LEGACY.md.
  - Stay byte-exact: the original's numeric value goes back out wherever it is
    observed.

`cargo clippy -p sm_data -p sm_game -p sm_runtime -p sm_tools --all-targets` must
pass in CI, and so must `cargo clippy -p sm_app --all-targets`, with default
features and with `--no-default-features --features slice`. Fix findings; don't
add `allow`s for these patterns.

Checks assert behaviour, not wording: never match on panic or `Die` text. The
verifier doesn't decide engine behaviour. Where an existing check forces us to
keep something only it needs, tag it `LEGACY(<kind>)` and list it in
[docs/LEGACY.md](docs/LEGACY.md), then retire it when the check is fixed.
