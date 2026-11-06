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

**P3. Keep what is proven; rebuild what is not.** The oracle, the lock format, facts, lockstep
localisation, `[exact]` refactors, LEGACY, typed mutants and `land.sh` carry over. No game code does.

**P4. The engine's structure follows the hardware. A harness's limits are parameters.** The Tick is
a video frame with a real NMI, a real line order and real lag. An oracle's shortcuts (line-granular
interrupts, its colour conversion) live in the Reference profile and in LEGACY, never in the shape
of the Tick. *How we know:* swapping oracles changes `OracleRules` and locks, not types.

**P5. One authority per piece of state. Translation goes one way.** The engine owns its state. The
only translation is oracle machine state into engine terms, in verification. Nothing translates back.
*How we know:* there is no encoder from State to WRAM. (A two-way RAM codec once cost most of
every Tick and kept the engine RAM-shaped.)

**P6. Verification observes from outside and never shapes the engine.** It compares outputs and a
decoded typed State. Engine code has no probes, observers or verify cfgs, and no field exists only
because the oracle has a byte.
*How we know:* `check_crate_rules.sh`; and for a new field, "which engine rule reads this?"

**P7. One mechanism per job, and one path per responsibility.** One interpreter, one State schema,
one Checkpoint codec, one projection per stream, one digest, one decoder, one encoder, one runner. A
replacement deletes what it replaces in the same change.
*How we know:* ARCHITECTURE names "the one X" for each job. No cfg or flag chooses between two
implementations of game behaviour.

**P8. All state is explicit plain data, and a Tick is a function of State and input.** Nothing lives
in globals, in caches that outlive a Tick, or in another process.
*How we know:* on every green row, Checkpoint → restore → advance gives the same digests, and a
one-field mutant is reported at the right Tick and stream.

**P9. Name the original's mechanisms. Don't emulate them, and don't flatten them away.** Its waits,
interrupt work, instruction lists and limits become typed engine concepts, not a CPU model and not
timing hidden inside a pass.
*How we know:* every wait site the oracle stops at maps to a named `Wait`. The decoder rejects a stack
it can't map instead of guessing.

**P10. The gate is the engine, in free run, against the lock, with facts.** A row counts only if the
engine executes it, every compared stream matches and its facts hold. Lockstep only finds the first
bad Tick. *How we know:* reports keep "reached", "matches" and "deterministic" apart.

**P11. Every check must be able to fail.** Every gate stream has a mutant that must fail it, and so
does the decoder. Every exclusion is a named LEGACY row. A surviving mutant is red.

**P12. The reference shares no code with the engine, and every difference is named.** The unmodified
ROM runs on emulators we don't share code with. No tolerances, no unnamed masks: a difference between
engine and oracle, or oracle and hardware, is a LEGACY row. A second reference (Mesen, hardware
captures) is in the build order.

**P13. Measure outcomes; structural counts are advisory.** Only measures that need the engine to run
alone can fail CI: green pairs, facts, verified Ticks/s. (Structural ratchets once reached zero
while the old code still ran.)

**P14. Verified speed is a requirement from the first slice.** Every landing reports Ticks/s. (At
about 20 Ticks/s, verification once throttled all other work.)

**P15. Prove a design end to end on a thin slice before going broad.** Slice 0 runs open, step, the
renderer and headless replay, green, before a second cluster; one lane builds the spine.

**P16. Lines are a cost, and a general mechanism needs real users.** Prefer data over code and one
mechanism over per-case code. A generic layer is built when its second real user exists. Each domain
has a written budget.
*How we know:* each trait and generic layer names its users in ARCHITECTURE. A domain past its budget
stops for a design review.

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
Questions for Jim are goal trade-offs, each with its context. The rules live in one short file Jim
approves (AGENTS.md).

**P20. Identify each behaviour, and model it so it can be reproduced exactly or switched off.** Lag,
load times and the console's other emergent behaviours become named, modelled behaviours. In vanilla
a modelled behaviour is exact; as an option it can be turned off, and the option records what it
breaks (for example lag-dependent tricks in Map Rando).
A speedrunner must be able to execute every trick in the TAS movies we use.
*How we know:* the behaviour catalogue (MODEL §3) is this principle's artifact, reported on every
landing (ARCHITECTURE D24). Every TAS movie syncs with all behaviours on.

## Coding principles

AGENTS.md's Rust style is the base: enums with data, newtypes, typed flag sets, behaviour tables as
data, conversion at the edges with `TryFrom`, no lint allows, no matching on panic text. These extend it.

**K1. No addresses in engine code.** Program places are labels, and content is referred to by ID. ROM
and RAM addresses appear only in `data`'s import tables and in verification.
*How we know:* `tools/check_crate_rules.sh` rejects SNES address literals in `game`, `runtime` and `app`.

**K2. Make invalid states unrepresentable, including progress through time.** A routine that spans
Ticks is an explicit typed cursor: no async, no generators, no flag that means "half done".
*How we know:* every Tick-spanning routine is a variant of the closed `Frame` enum. Review question:
"which variant owns this field?"

**K3. Concrete context types, not host traits.** A domain receives a struct of disjoint borrows. A
trait needs two real implementations or a derive. A trait that exists so one system can host another
is forbidden.
*How we know:* ARCHITECTURE §4.3 lists every public trait; a new trait is a review item.

**K4. Capabilities are typed.** Guard types decide who may write what: video memory only at a commit
point, presentation only through `&`. The compiler checks it.

**K5. The engine fails in types, not panics.** No `unwrap`, `expect`, `panic!` or indexing that can
panic in engine code. Arithmetic states how it overflows (`wrapping_*`, `checked_*`, `saturating_*`):
the original's arithmetic is 16-bit and wraps. An engine fault is a `SessionFault`; a full table is a
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
  `wildcard_enum_match_arm`, `print_stdout`, `print_stderr`, `dbg_macro`, `allow_attributes`;
  `float_arithmetic` in `game` (the original has no floats); `missing_docs`. A nursery lint that
  misfires is dropped from the workspace set with its reason recorded there, never silenced at a site.

*Jim (2026-11-05):* "the strictest, most idiomatic Rust possible"; stable, because unstable features
churn. *How we know:* `Cargo.toml`, `rust-toolchain.toml` and `tools/check_crate_rules.sh`, in CI.

**K9. A modern reader can understand the code without the original.** Names come from the domain:
official names for enemies, bosses, items, rooms and mechanics, never addresses, routine labels or
decompiler names. A reader understands a module from its names and types without opening the bank
logs. A bank reference is one `Source:` line on an item where the behaviour is surprising; module docs
describe the domain, never the original's call order.
*Jim (2026-11-05):* asked for this. *How we know:* review question: "could someone who has never
seen the original follow this module?" A source check rejects routine-label names (`func_80_8343`).

**K10. The code explains itself; comments say why, never what.** A comment justifies the unexpected:
an original quirk being reproduced, an order that matters, a value that looks wrong but is the game's
own, an oracle difference with its LEGACY row. Doc comments on public items say what the item is for
and its contract.
*Jim (2026-11-05):* his own principle. *How we know:* "does this comment say anything the code doesn't?"
