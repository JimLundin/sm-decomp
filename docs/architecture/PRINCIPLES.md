# Principles

Architecture lane, `research/architecture`, 2026-11-05. Jim reviewed this on 2026-11-05 and
agreed with the principles. His decisions on K5, K8, K9 and K10 are recorded under each
one.

These principles come before the architecture. Every decision in
[ARCHITECTURE.md](ARCHITECTURE.md) cites the principles it follows. A decision that no
principle supports is either a missing principle or a wrong decision.

They come from two attempts:

- **Attempt 1, the transpiled game on the RAM image.** snesrev's C, turned into Rust,
  running on a 128 KiB WRAM image. The oracle came from the same harness.
- **Attempt 2, incremental extraction.** Domains moved into `sm_game` while the old game
  kept running. This needed host traits, `foreign` calls, leases, a Session ↔ RAM codec and
  ratchets on site counts.

The evidence is the [retrospective](RETROSPECTIVE.md) (`research/retrospective`,
`bcc9e48`). "Retro A1" means its mistake A1. "Retro W6" means the thing that worked,
number 6. "Rule 12" means its design rule 12.

Each principle has a one-line rule, the lesson behind it, and how we will know it is being
followed. "How we know" is a check that runs or a question a reviewer asks. It is never a
line count.

## Project and design principles

**P1. The real game's observable behaviour is the specification.**
The original's code shape and memory layout are not part of it.
*Why:* attempt 1 copied snesrev's code and RAM layout. It also inherited snesrev's oracle
patches, so the engine matched the patches, not the console. That gave 13 days of
"0 failures", and then 280 of 329 chain rooms failed on the faithful oracle (Retro A1, V1).
*How we know:* every rule names the observation it reproduces: a stream, a fact, a
bank-log routine or an oracle measurement. Review question: "what would the player see
differently if this were otherwise?"

**P2. Build from knowledge, not by tuning against diffs.**
A rule is written from the bank logs and from oracle measurements made beforehand. It is
not found by editing until the diff goes away. Jim: "we neither start everything from
scratch nor guess as we run against the oracle."
*Why:* fixes found against diffs held for one row and broke another. A masked bug showed up
90 frames from its cause (Retro V2).
*How we know:* each domain lists its routines and the measurements it needs before its
code lands, and a fix names the routine it follows. Measurements that set rules (port echo
latency, door frame counts, IRQ write positions) come before the code that depends on them.

**P3. Keep what is proven; rebuild what is not.**
These carry over: the oracle, the corpus, the lock format, facts, lockstep localisation,
`[exact]` refactors, LEGACY, typed mutants and `land.sh` (Retro section 3). The engine code
does not carry over.
*Why:* attempt 2 refactored a translation toward the target and kept its shape
(Retro A1). But the verification practices that worked are the only evidence we have.
*How we know:* new crates cannot depend on old ones, and CI checks this by name (rule 2). A
module that carries over names the contract it meets. No game module carries over.

**P4. The engine's structure follows the hardware. A harness's limits are parameters.**
The Tick is a video frame with a real NMI, a real line order and real lag. An oracle's
shortcuts (no beam, line-granular interrupts, patches, its colour conversion) live in the
Reference profile and in LEGACY, never in the shape of the Tick.
*Why:* FOUNDATION matched the old frame stepper "by construction", which builds harness
artefacts into the engine, the same kind of mistake as matching snesrev's patches
(Retro section 5, V1). The new oracle runs a real beam (docs/oracle/HARNESS.md), and Mesen
comes later.
*How we know:* every timing rule that differs between oracles is a named field of
`OracleRules`. Swapping oracles changes data and locks, not types.

**P5. One authority per piece of state. Translation goes one way.**
The engine owns its state. The only translation is oracle machine state into engine terms,
in verification. Nothing translates back.
*Why:* the lease and codec bridge cost about 85% of every Tick, needed 334 host traits, and
took 91 lease-merge commits to unwind. It also kept the Session RAM-shaped (Retro A2, A3;
rules 3-4).
*How we know:* there is no encoder from State to WRAM. A dependency check fails if an engine
crate depends on `ram_map`, the oracle or verification.

**P6. Verification observes from outside and never shapes the engine.**
It compares outputs (video memory, picture, sound ports, SRAM) and a decoded typed State.
Engine code has no probes, observers or verify cfgs, and no field exists only because the
oracle has a byte.
*Why:* RAM comparison shaped the Session (Retro A3). Probes inside game code broke moves
(Retro A6).
*How we know:* a source check covers the cfgs and probes. Review question for a new State
field: "which engine rule reads this?"

**P7. One mechanism per job, and one path per responsibility.**
One interpreter, one State schema, one Checkpoint codec, one projection per stream, one
digest, one decoder, one runner. A replacement deletes what it replaces in the same change.
*Why:* attempt 2 had six Checkpoint codecs, two hand-kept projections and several
presentation paths, roughly 10-16k lines of duplicates that all had to agree (Retro A5).
*How we know:* ARCHITECTURE lists "the one X" for each job. No cfg or flag chooses between
two implementations of game behaviour.

**P8. All state is explicit plain data, and a Tick is a function of State and input.**
Nothing lives in globals, in caches that outlive a Tick, or in another process.
*Why:* state split between the RAM image, `Game` and the Session hid bugs in "which copy is
current". A mid-frame lease lost a queued sound (Retro A2).
*How we know:* on every green row, Checkpoint → restore → advance gives the same digests,
and a one-field mutant is reported at the right Tick and stream.

**P9. Name the original's mechanisms. Don't emulate them, and don't flatten them away.**
Its waits, interrupt work, instruction lists and limits become typed engine concepts. They
are not a CPU model, and they are not timing hidden inside a pass.
*Why:* attempt 1 emulated them (a 65816 on the RAM image). Attempt 2 hid them behind
patched waits and a deferred message box, and in arguments about who owns the Tick
(Retro A2, V1).
*How we know:* every wait site the oracle stops at maps to a named `Wait`. The decoder
rejects a stack it can't map instead of guessing.

**P10. The gate is the new engine, in free run, against the lock, with facts.**
A row counts only if the new spine executes it, every compared stream matches, and its
facts hold. Lockstep only finds the first bad Tick.
*Why:* Gate 1 passed 114/114 rows, but the code that ran was the transpiled host. Rows
reached nothing and still "matched" (Retro A4, V3, M1; rules 8, 13, 16).
*How we know:* the gate build excludes the old crates by name. Reports keep "reached",
"matches" and "deterministic" apart.

**P11. Every check must be able to fail.**
Every gate stream has a scheduled mutant that must fail it, and that includes the decoder.
Every exclusion is a named LEGACY row that the mutation sample covers.
*Why:* mutation checks that only looked for anchor text, and three surviving mutants on the
first typed run (Retro V3; rules 14-15).
*How we know:* a scheduled mutation job, with a red result when a mutant survives.

**P12. The reference shares no code with the engine, and every difference is named.**
The unmodified ROM runs on an emulator we don't share code with. There are no tolerances
and no unnamed masks. A difference between engine and oracle, or between oracle and
hardware, is a LEGACY row.
*Why:* the oracle and the engine shared an origin, a harness and a PPU, so agreement proved
little (Retro V1, V2, W9).
*How we know:* each lock exclusion is a named field. A second reference (Mesen, hardware
captures) is in the build order, starting with pixels and lag.

**P13. Measure outcomes; structural counts are advisory.**
*Why:* RAM sites rose from 3,932 to 6,933 during moves. Baselines were redefined upward,
and all four strict ratchets reached 0 while the old game was still what ran (Retro M1).
*How we know:* every measure that can fail CI needs the engine to run alone: green rows,
green (row, stream) pairs, facts, verified Ticks/s.

**P14. Verified speed is a requirement from the first slice.**
*Why:* at about 20 Ticks/s, CI throttled all other work (Retro V4).
*How we know:* slice 0 has a Ticks/s target. Every landing reports Ticks/s against the
oracle in the same job.

**P15. Prove a design end to end on a thin slice before going broad.**
*Why:* attempt 2 moved breadth across every domain while none of it ran alone, and the
engine first ran on the last day (Retro A4, W7).
*How we know:* slice 0 runs `Session::open`, `step`, our renderer and headless replay, and
is green, before a second row cluster starts. The spine is built by one lane (rule 25).

**P16. Lines are a cost, and a general mechanism needs real users.**
Prefer data over code and one mechanism over per-case code. A generic layer is built when
its second real user exists, not before. Each domain has a written budget.
*Why:* 324 public traits, 179 of them hosts, began as reasonable narrow traits
(Retro A2, A7; rules 6, 11).
*How we know:* each trait and generic layer names its users in ARCHITECTURE. A domain
that passes its budget stops for a design review.

**P17. Vanilla is one configuration, and options enter at named seams.**
Gameplay, timing, presentation, content and input are separate profiles, resolved when the
Session opens. They never become forks spread through the logic.
*Why:* Map Rando, widescreen, quick transitions and native lag are known futures. Wiring
each one in wherever it touches would recreate attempt 2's tangle.
*How we know:* each profile field is read by exactly one module. Every corpus row runs on
the default profiles.

**P18. Exact output and intent output are separate, and intent never feeds back.**
The engine emits the bytes the oracle compares. Beside them it emits their meaning: what
was drawn where, which song is playing. Enhancements read the meaning and never write to
the Session.
*Why:* widescreen, HD art and replacement audio need more than SNES-layout bytes hold. If
enhancements had to read the bytes, they would push on the exact path.
*How we know:* presentation receives `&State` or intent records, never `&mut`. A corpus
check confirms that intent rendered at native width equals the exact picture.

**P19. Every decision records its reason and the alternative rejected.**
We make the engineering decisions and Jim vetoes. Questions for Jim are goal trade-offs,
each with context. The rules live in one short file Jim approves.
*Why:* Jim rejected a 35-item list with no context, and rules drifted across four documents
(Retro P1; rule 23).
*How we know:* every ARCHITECTURE decision has a "rejected" line. QUESTIONS holds only goal
questions.

## Coding principles

AGENTS.md's Rust style is the base: enums with data, newtypes, typed flag sets, behaviour
tables as data, conversion at the edges with `TryFrom`, no lint allows, and no matching on
panic text. These extend it where the lessons demand.

**K1. No addresses in engine code.**
Program places are labels, and content is referred to by ID. ROM and RAM addresses appear
only in `sm_data`'s import tables and in verification.
*Why:* 1,861 `const X: u16 = 0x...` and ROM addresses used as state (Ridley's
`LIFTOFF: u16 = 0xb2f3`) are still in today's `sm_game` (Retro A1).
*How we know:* a source check rejects SNES address literals in `sm_game`, `sm_runtime` and
`sm_app`.

**K2. Make invalid states unrepresentable, including progress through time.**
A routine that spans Ticks is an explicit typed cursor. No async, no generators, no flag
that means "half done".
*Why:* RAM-named word bags and sibling fields kept the Session RAM-shaped (Retro A3).
*How we know:* every Tick-spanning routine is a variant of the closed `Frame` enum. Review
question: "which variant owns this field?"

**K3. Concrete context types, not host traits.**
A domain receives a struct of disjoint borrows. A trait needs two real implementations or
a derive. A trait that exists so one system can host another is forbidden.
*Why:* 334 host traits and 83 blanket impls (Retro A2; rule 6).
*How we know:* ARCHITECTURE lists every public trait with its implementations, and new
traits are a review item.

**K4. Capabilities are typed.**
Guard types decide who may write what: video memory only at a commit point, presentation
only through `&`.
*How we know:* the compiler. A write outside a guard does not type-check.

**K5. The engine fails in types, not panics.**
Engine code has no `unwrap`, no `expect`, no `panic!` and no indexing that can panic.
Arithmetic states how it overflows (`wrapping_*`, `checked_*`, `saturating_*`). An engine
fault is a `SessionFault`, and a full table is a gameplay result. The one exception is the
original's own crashes, where accuracy requires modelling them. The working answer is that
they become an `OriginalFailure` outcome, and the details are decided when the first one is
built.
*Why:* panics in Tick paths ended whole replays and hid where they started. Silent
overflow is a behaviour difference: the original's arithmetic is 16-bit and wraps.
*Jim (2026-11-05):* confirmed. He would reconsider only if the rule grew code volume by a
factor or more.
*Measured:* the lint set below was applied to FOUNDATION's compile-checked sketch
(`scratch/foundation_sketch`, 1,123 non-blank, non-comment production lines). The
restriction lints (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`,
`arithmetic_side_effects`, `unreachable`) found 15 sites outside tests. Fixing them took the count to
1,138 lines (+1.3%). `pedantic` and `nursery` then added 21 findings. After fixing them the
count is 1,146 lines (+2.0% in total), plus about ten doc lines for the `# Errors` sections.
The patterns that kept the cost small:
- iterators instead of indices: `iter_mut().enumerate().find(..)` returns the slot itself,
  and `zip` pairs two arrays;
- `get`, `split_at_mut_checked` and `split_first_mut` with `?` and `let-else`;
- a step past the end is `checked_add(1).ok_or(fault)?`, and "go back one" is replaced by
  remembering where you were;
- fixed arrays indexed by an enum, through a destructuring `match` (`let [music, l1, l2, l3]
  = &mut ports`), which cannot be out of range;
- newtype indices that only the slot store can construct, so they are valid by construction;
- the rotate-and-chain idiom (`tail.iter_mut().chain(head)`) for wrap-around writes.
*How we know:* the lint set in K8, denied in every engine crate. Tests may use `unwrap`,
`expect`, `panic` and indexing, through clippy's `allow-*-in-tests` configuration, not
through `allow` attributes.

**K6. Mechanical code is derived once.**
The Checkpoint encode, decode and diff come from one derive. They are not written by hand
for each type.
*Why:* six codecs drifted apart (Retro A5).
*How we know:* State types have no hand-written codec impls.

**K7. Safe Rust only in the engine.**
*How we know:* `unsafe_code = "forbid"` in every engine crate.

**K8. The strictest, most current idiomatic Rust: the newest stable toolchain and a strict lint set.**
- **Toolchain:** the newest stable release, pinned exactly in `rust-toolchain.toml` and
  bumped in its own PR within a week of each release. Edition 2024 (or newer when one
  ships). Engine crates use no `#![feature]` gates. Nightly is for tooling only (rustfmt's
  unstable options, miri).
- **Idioms:** the current language: let-chains, `let-else`, `impl Trait` in argument and
  return position where it fits, const generics, `?` everywhere, iterator adaptors, and
  `From`/`TryFrom` instead of `as` (AGENTS.md).
- **Lints, all denied in engine crates, with no `allow` attributes:** `clippy::pedantic`;
  `clippy::nursery`; and from `restriction`: `unwrap_used`, `expect_used`, `panic`,
  `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, `unreachable`, `todo`,
  `unimplemented`, `wildcard_enum_match_arm` (AGENTS.md's "no `_ =>` when every case can be
  named"), `float_arithmetic` in `sm_game` (the original has no floats), `print_stdout`,
  `print_stderr`, `dbg_macro` and `allow_attributes`. Rust's `missing_docs` is denied for
  public items of `sm_data` and `sm_game`. A nursery lint that misfires is dropped from the
  workspace set, with the reason recorded beside the set. It is never silenced at a site.
*Decision:* stable, not nightly. Unstable features can change or disappear between
nightlies, and that would force the churn this rebuild exists to avoid. Nightly would buy
features we don't need for an engine of plain data and functions.
*Jim (2026-11-05):* asked for "the strictest, most idiomatic Rust possible", meaning the
most current language, styles and behaviour. He confirmed the newest stable toolchain, with
nightly for tooling only.
*How we know:* the workspace `[lints]` table and `rust-toolchain.toml`, checked in CI. A
source check rejects `#![feature` and `#[allow(` in engine crates.

**K9. A modern reader can understand the code without the original.**
The code must make sense to someone who has never seen the transpilation, the C or the
assembly. Names come from the domain: official names for enemies, bosses, items, rooms and
mechanics, never addresses, routine labels or decompiler names. A reader understands a
module from its names and types without opening the bank logs. A bank-log reference is a
short source line in a doc comment where a behaviour is surprising. It is never the
structure of the module.
*Why:* attempt 1's code was organised by the original's routines and named by addresses
(Retro A1). Understanding it took the bank logs open beside it, so every change was slow
and risky.
*Jim (2026-11-05):* asked for this explicitly.
*How we know:* review question: "could someone who has never seen the original follow this
module from its names and types?" A source check rejects routine-label names (`func_80_8343`
and the like) in engine crates.

**K10. The code explains itself; comments say why, never what.**
Names and types carry the meaning. There are no comments that restate what the code does. A
comment exists to justify the unexpected:
- an original quirk being reproduced;
- an order that matters;
- a value that looks wrong but is the game's own;
- an oracle difference, with its LEGACY row.

Doc comments on public items say briefly what the item is for and its contract. A bank-log
reference may be the source line of such a comment where the behaviour is surprising, not
narration. This is how K9 and K10 fit together: a reader learns the module from names and
types (K9), and comments carry only the surprises (K10).
*Jim (2026-11-05):* his own principle.
*How we know:* review question: "does this comment say anything the code doesn't?" There
is no comment-ratio metric.
