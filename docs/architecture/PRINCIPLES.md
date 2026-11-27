# Principles

Jim agreed these principles on 2026-11-05. Every decision in [ARCHITECTURE.md](ARCHITECTURE.md)
cites the principles it follows; [MODEL.md](MODEL.md) applies them to the game code. "How we know"
is a check that runs or a question a reviewer asks, never a line count.

## The thesis (Jim, 2026-11-05)

**We model Super Metroid's behaviour in its entirety.** The engine is a model of what the game
does, made of named behaviours, not a copy of the original's code. Because each behaviour is an
identifiable part of the model, it can be:

- reproduced exactly, which is vanilla, proven against the oracle;
- adjusted, for example the original's poor space jump timing window;
- added (respin and other quality-of-life rules) or removed (lag).

A behaviour hidden in the structure (a timing accident, a constant buried in a routine, a rule
spread across modules) can be neither adjusted nor removed: a modelling failure even if it matches.

## Project and design principles

**P1. The real game's observable behaviour is the specification.** The original's code shape and
memory layout are not part of it. (An engine that matched a patched oracle once reported "0
failures" while failing most rooms on the real game.)
*How we know:* every rule names the observation it reproduces. Review question: "what would the
player see differently if this were otherwise?"

**P2. Build from knowledge, not by tuning against diffs.** A rule is written from the bank logs
and from oracle measurements made beforehand, not found by editing until the diff goes away.
*How we know:* a domain's spec (`docs/behaviours/`) lists its behaviours, sources and test
vectors before its code lands.

**P3. Keep what is proven; rebuild what is not.** Independent oracle capture, pinned records, `[exact]` refactors,
LEGACY, meaningful mutants and `land.sh` remain. Previous executable gameplay is removed;
reviewed services remain by responsibility. Indexed replay uses the actual production entry.

**P4. Observable hardware behaviour is part of the model. Replay limits are explicit.** The Tick is
a frame boundary with observed NMI, line order and lag requirements. Scoped transport
or motion entries do not imply a complete clock or frame dispatcher. An oracle's shortcuts (line-granular
interrupts, its colour conversion) live in the Reference profile and in LEGACY, never in the shape
of the Tick. *How we know:* swapping oracles changes `OracleRules` and locks, not types.

**P5. One authority per piece of state. Translation goes one way.** The engine owns its state. The
only translation is oracle machine state into engine terms, in verification. Nothing translates back.
*How we know:* there is no encoder from State to WRAM. (A two-way RAM codec once cost most of
every Tick and kept the engine RAM-shaped.)

**P6. Verification observes from outside and never shapes the engine.** It compares outputs and a
decoded typed State. Engine code has no probes, observers or verify cfgs, and no field exists only
because the oracle has a byte.
*How we know:* `deny.toml` (no engine crate reaches `verify`), the forbidden `unexpected_cfgs`,
`check_crate_rules.sh`'s name scan; and for a new field, "which engine rule reads this?"

**P7. One mechanism per job, and one path per responsibility.** One interpreter, one State schema,
one Checkpoint codec, one projection per stream, one digest, one decoder, one encoder, one runner. A
replacement deletes what it replaces in the same change.
*How we know:* ARCHITECTURE names "the one X" for each job. No cfg or flag chooses between two
implementations of game behaviour.

**P8. All state is explicit plain data, and a Tick is a function of State and input.** Nothing lives
in globals, in caches that outlive a Tick, or in another process.
*How we know:* an indexed case's checkpoint continuation gives the same results as the run it
was saved from, and a one-field mutant is reported at the right cut and field.

**P9. Name the original's mechanisms. Don't emulate them, and don't flatten them away.** Its waits,
interrupt work, instruction lists and limits become typed engine concepts, not a CPU model and not
timing hidden inside a pass.
*How we know:* every wait site the oracle stops at maps to a named `Wait`. The decoder rejects a stack
it can't map instead of guessing.

**P10. Acceptance executes production with carried State against independent expectations.**
Scoped and integrated cases use the same entry and runner. Compared cuts and outputs must
match; missing dependencies stay unsupported. *How we know:* reports separate completed
behaviour from held unsupported boundaries and disclose borrowed inputs.

**P11. Every check must be able to fail.** Every supported responsibility has a planted fault that
must fail its indexed case (`tools/mutants.toml`). Every exclusion is a named LEGACY row. A
surviving mutant is red.

**P12. The reference shares no code with the engine, and every difference is named.** The unmodified
ROM runs on emulators we don't share code with. No tolerances, no unnamed masks: a difference between
engine and oracle, or oracle and hardware, is a LEGACY row. A second reference (hardware
captures) is in the build order.

**P13. Measure outcomes; structural counts are advisory.** Only measures that need the engine to run
alone can fail CI: indexed production comparisons, reviewed boundaries, facts and verified throughput. (Structural ratchets once reached zero
while the old code still ran.)

**P14. Verified speed is a requirement from the first slice.** Every landing reports Ticks/s. (At
about 20 Ticks/s, verification once throttled all other work.)

**P15. Prove a design end to end on a thin slice before going broad.**

**P16. Size is a cost, and a general mechanism needs real users.** Prefer data over code and one
mechanism over per-case code. A generic layer is built when its second real user exists. Each domain
has a budget from its spec (MODEL §1 decision 7).
*How we know:* each trait and generic layer names its users in ARCHITECTURE; `tools size` reports
each domain's fit to its budget. The root uses that guideline after independent review to
decide whether a cut would improve the code (MODEL §1 decision 19).

**P17. Vanilla is one configuration, and options enter at named seams.** Gameplay, timing,
presentation, content and input are profiles resolved when the Session opens, never forks spread
through the logic.
*How we know:* each profile field has one reader; every corpus row runs on the defaults.

**P18. Exact output and intent output are separate, and intent never feeds back.** The engine emits
the bytes the oracle compares and, beside them, their meaning. Enhancements read the meaning and never
write to the Session.
*How we know:* presentation receives `&State` or intents, never `&mut`. Intents rendered at native
width equal the exact picture.

**P19. Every decision records its reason.** We make the engineering decisions and Jim vetoes.
Questions for Jim are goal trade-offs, each with its context. [RULES](../RULES.md) indexes each rule at its canonical home.

**P20. Identify each behaviour, and model it so it can be reproduced exactly or switched off.** Lag,
load times and the console's other emergent behaviours become named, modelled behaviours. In vanilla
a modelled behaviour is exact; as an option it can be turned off, and the option records what it
breaks (for example lag-dependent tricks in Map Rando).
A speedrunner must be able to execute every trick in the TAS movies we use.
*How we know:* the behaviour catalogue (MODEL §3) is this principle's artifact, reported on every
landing (ARCHITECTURE D24). Every TAS movie syncs with all behaviours on.

## Coding principles

[Rust style](#rust-style) is the base. These principles extend it.

**K1. No addresses in engine code.** Program places are labels, and content is referred to by ID. ROM
and RAM addresses appear only in `data`'s import tables and in verification.
*How we know:* `tools/check_crate_rules.sh` rejects SNES address literals in `game`, `runtime` and `app`.

**K2. Make invalid states unrepresentable, including progress through time.** A routine that spans
Ticks is an explicit typed cursor: no async, no generators, no flag that means "half done".
*How we know:* review question: "which variant owns this field?"

**K3. Concrete context types, not host traits.** A domain receives a struct of disjoint borrows. A
trait needs two real implementations or a derive. A trait that exists so one system can host another
is forbidden.
*How we know:* ARCHITECTURE §4.3 lists every public trait; a new trait is a review item.

**K4. Capabilities are typed.** Guard types decide who may write what: video memory only at a commit
point, presentation only through `&`. The compiler checks it.

**K5. The engine fails in types, not panics.** No `unwrap`, `expect`, `panic!` or indexing that can
panic in engine code. Arithmetic states how it overflows (`wrapping_*`, `checked_*`, `saturating_*`):
the original's arithmetic is 16-bit and wraps. An engine fault is a typed production fault; a full table is a
gameplay result; the original's own crashes are an `OriginalFailure`.
*Jim (2026-11-05):* confirmed, unless it grew code volume by a factor; measured, it cost about 2%.
Patterns: iterators instead of indices; `get`, `split_at_mut_checked` and `split_first_mut` with
`?` and `let-else`; fixed arrays indexed by an enum through a destructuring `match`; newtype indices
only the slot store can construct; `tail.iter_mut().chain(head)` for wrap-around writes. Fixed-point
types use named methods (`wrapping_add`), since `arithmetic_side_effects` flags operator impls.
*How we know:* the K8 lint set; tests loosen it only through clippy.toml's `allow-*-in-tests`.

**K6. Mechanical code is derived once.** The Checkpoint codec and diff come from one derive; State
types have no hand-written codec impls.

**K7. Safe Rust only in the engine:** `unsafe_code = "forbid"`.

**K8. The strictest, most current idiomatic Rust.**
- **Toolchain:** the newest stable release, pinned in `rust-toolchain.toml` and bumped in its own
  change within a week of a release. Edition 2024. No `#![feature]` gates; nightly is for tooling only.
- **Idioms:** let-chains, `let-else`, `impl Trait` where it fits, const generics, `?`, iterator
  adaptors, `From`/`TryFrom` instead of `as`.
- **Lints, all denied, with no `allow` attributes** (the workspace `[lints]` in `Cargo.toml`):
  `clippy::pedantic`; `clippy::nursery`; `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`,
  `arithmetic_side_effects`, `as_conversions`, `unreachable`, `todo`, `unimplemented`,
  `wildcard_enum_match_arm`, `print_stdout`, `print_stderr`, `dbg_macro`;
  `float_arithmetic` in `game` (the original has no floats); `missing_docs`. Forbidden, so no
  attribute lifts them: `allow_attributes` and `allow_attributes_without_reason`,
  `unstable_features`, `unexpected_cfgs`, `unsafe_code` and the clean room's `disallowed_*`. A nursery lint that
  misfires is dropped from the workspace set with its reason recorded there, never silenced at a site.

*Jim (2026-11-05):* "the strictest, most idiomatic Rust possible"; stable, because unstable features
churn. *How we know:* `Cargo.toml`, `rust-toolchain.toml` and `tools/check_crate_rules.sh` (no
`expect` in the engine), in CI.

**K9. A modern reader can understand the code without the original.** Names come from the domain:
official names for enemies, bosses, items, rooms and mechanics, never addresses, routine labels or
decompiler names. A reader understands a module from its names and types without opening the bank
logs. A bank reference lives only in the observer's decode notes (`docs/behaviours/<domain>.decode.md`),
never in a rebuilt domain's code, spec or catalogue; module docs describe the domain, never the
original's call order. Engine code not yet rebuilt still cites the bank logs in comments, which
leave with it.
*Jim (2026-11-05):* asked for this. *How we know:* review question: "could someone who has never
seen the original follow this module?" A source check rejects routine-label names (`func_80_8343`).

**K10. The code explains itself; comments say why, never what.** A comment justifies the unexpected:
an original quirk being reproduced, an order that matters, a value that looks wrong but is the game's
own, an oracle difference with its LEGACY row. Doc comments on public items say what the item is for
and its contract.
*Jim (2026-11-05):* his own principle. *How we know:* "does this comment say anything the code doesn't?"

## Rust style

Code is idiomatic Rust, not transpiled C. Preserve the original's behaviour, not its shapes:

- Iterate (`iter().enumerate()`, `iter_mut()`) instead of `for i in 0..N { a[i] }`.
- Parenthesize mixed shift, bitwise and arithmetic expressions.
- Name checks with a function that says what it checks, never a no-op expression such as `[(); N][i];`.
- Use `From`/`TryFrom` and `wrapping_*` instead of chains of `as` casts.
- Use `std::fs` and owned buffers.
- Store text files as UTF-8 with LF line endings (`.gitattributes` and CI enforce LF).

Model the domain with types, not raw numbers:

- Use an `enum` for a state, mode, kind or choice instead of a `u16` code. Avoid magic constants
  and state words compared with `==`. Match exhaustively; don't fall back to `_ =>` when every
  case can be named.
- Give an enum variant the data it carries, rather than sibling fields that mean something in one state.
- Use a newtype for an index or a quantity that must not mix with others: slot, room, pose, frame count.
- Use a typed flag set, a struct of `bool`s or an enum set, instead of bit masks tested inline.
- Keep behaviour tables as data: arrays of typed records or enums, plus a small interpreter.
- Convert at the edges. Turn ROM bytes and record formats into the enum once, with `TryFrom`.
  Where the original treats an invalid value as an error, map it to `Fatal::unreachable`; where it
  tolerates one, keep a named variant (`Unknown(u16)`) and list it in LEGACY. Stay byte-exact: the
  original's value goes back out wherever it is observed.

Quality and the local source gate use `cargo clippy --locked --workspace --all-targets --no-deps`
to lint every workspace crate
under its manifest policy. `tools/lane/gate.sh` also promotes all warnings to errors.
Clippy loads `crates/clean/.clippy.toml` for the clean crates and `clippy.toml` for the other crates.
Fix findings at the source.

Shared check, documentation and data rules are in [RULES](../RULES.md#authority-and-shared-guardrails).
