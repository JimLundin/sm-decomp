# Retrospective: what the first engine taught us

Research lane `research/retrospective`, based on main `52c2117`, 2026-11-05. It is input
for the rebuild's architecture lane ([REVIEW](../REVIEW.md#rebuild-on-a-new-foundation-jim-2026-11-05)).
It changes no code and binds nothing.

Sources: the full history of `origin/main` (4,450 commits from 2026-10-17), the frozen
[progress report](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/PROGRESS_REPORT.md)
and [REVIEW edition](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/REVIEW.md)
at `e281ce9`, the docs on main, issues #177, #231 and #239-#254, PR #282, and
`docs/foundation/FOUNDATION.md` on `origin/research/foundation-design`.

How numbers were measured:

- Line counts are `git grep -c ''` over `*.rs` at the last commit before a date. They
  include tests, so they are higher than `production_lines`.
- Churn is `git log --numstat` without the transpile commit. Renames inflate it, so treat
  it as an order of magnitude.
- "(est.)" marks an estimate or an inference. Everything else is quoted from a commit, a
  doc or an issue.

## 1. The arc in brief

| Dates | Approach | What happened |
| --- | --- | --- |
| before 10-17 | A static recompiler and per-actor translation pipeline | The first visible commit (`ca9522f20`) holds 867 files and 366k lines, with "six translated, unverified actor candidates". Replaced wholesale on 10-17. |
| 10-17 | c2rust transpile of snesrev/sm (`1311d3b4d`) | One Rust crate, 260k lines, checked per frame against the ROM on the emulated 65816. By the evening any% ran 129,712 frames with 0 failures. |
| 10-17 to 10-20 | Mechanical clean-up in place | 260k to 108k lines (unused bindings, native types, shared layouts). Every commit quoted "any% 129712/0". |
| 10-21 to 10-28 | "Own" and "share" modules inside `sm`; lanes A, B, M; bank retirement | Lines back up to 153k. Verification grew fixtures, focused comparisons and Python (8k lines by 10-26). |
| 10-29 | Milestone plan; crates `sm_data`, `sm_game`, `sm_runtime` | Phase 0 (verification loop), Phase 1 (package graph), Phase 2 (cut the backend cycle). Code moved into `sm_game` behind host traits that `Game` implements. |
| 10-30 to 11-02 | Moves through leases; typed Session; RAM-site ratchet | A lease decodes RAM into a Session and encodes it back. Faithful oracle added 10-30. RAM sites 2,057 to 1,159 by 11-02. |
| 10-31 | Scenario harness and oracle digest lock | Free run against per-domain running digests becomes the frequent gate; lockstep only localises. |
| 11-03 to 11-04 | Boundary ratchets: `foreign_calls`, `game_host_impls`, `session_entries`; lease merges; speed-first | All four strict ratchets reach 0 by 11-04 evening. Session-space compare through an oracle RAM to Session decoder. |
| 11-04 | External review (#254) | "Nothing yet proves the engine runs alone, the state model is still RAM-shaped, and the safety net has gaps." |
| 11-05 | Gate 1 slice (#282), then the rebuild decision (#284) | 114/114 rows match, but what runs is the transpiled game with its emulators removed. Jim freezes the old engine and asks for a new foundation. |

Size over time (all Rust lines, tests included):

| Date (end of) | `sm/` | `crates/` | Total | Python | Markdown |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10-17 | 260,032 | 0 | 260,032 | 277 | 629 |
| 10-20 | 107,520 | 0 | 107,520 | 353 | 2,662 |
| 10-23 | 120,499 | 0 | 120,499 | 2,168 | 12,401 |
| 10-26 | 152,690 | 0 | 152,690 | 8,196 | 2,850 |
| 10-29 | 167,731 | 39,564 | 207,295 | 7,660 | 2,958 |
| 11-01 | 84,889 | 159,921 | 244,810 | 18,996 | 6,084 |
| 11-03 | 90,476 | 185,289 | 275,765 | 10,067 | 8,197 |
| 11-05 | 63,057 | 220,108 | 283,165 | 10,341 | 7,592 |

Churn over the whole history: Rust +1.75M / -1.47M lines, Markdown +98k / -90k, Python
+30k / -19k. Roughly 5 Rust lines were written for each one that survives, and 12
Markdown lines for each one that survives (est., renames included).

The short version: the code moved from `sm/` into `crates/` and the total grew by 76k
lines after the clean-up low. The behaviour that runs did not change owner.

## 2. Mistakes

Each entry gives what we did, what it cost, the root cause, and the rule that would have
prevented it. Architecture and verification come first. Process is shorter.

### Architecture mistakes

#### A1. We used a translation as the foundation and refactored toward the target

**What we did.** On 10-17 we transpiled snesrev/sm and declared `sm/src` the source of
truth. Every later step was a refactor of that code toward the accepted architecture.

**Cost.**
- The transpile fixed the shape of everything: the `Game` god object, the RAM image
  (`ram_map.rs`, 1,169 names), C function boundaries, ROM addresses as state.
- On 10-29 the 376 `game/` files (118k lines) were, apart from 44, one dependency cycle
  with `game_state`, `ram_map`, `rtl`, `cpu_infra` and `snes/`. `Game` appeared in 145
  files, `ram_map` addresses in 102 (review baseline, `2c2928e`).
- Moving code out of the cycle added glue: "moves add 1.3-1.9x lines of lease/host
  glue"; production rose 155,275 to 172,456 between `9325dd0` and `54f1c79`.
- 19 days later the code still carried the C: 3,681 `as` casts and 1,861 `const X: u16 =
  0x...` in `crates/sm_game/src` today (#251 counted 4,660 and 1,860). Ridley's states
  are ROM addresses (`LIFTOFF: u16 = 0xb2f3`).
- Jim's verdict on 11-05 was to freeze all of it.

**Root cause.** A transpile gives a green test on day one, so every step looked safe. But
"green against the oracle" never measured distance from the target shape. Each refactor
kept the C's decomposition because that was the cheapest way to stay byte-exact.

**Rule.** Write engine code fresh, to the target types. Old code is reference for
behaviour, read beside the bank logs, never the starting text of a module. The compiler
must make it impossible for new code to call old code.

#### A2. We extracted behaviour through a byte-compatible bridge (leases, host traits, `foreign`)

**What we did.** Moved code ran in `sm_game` but executed inside the old `Game`. A lease
decoded the RAM image into a Session, ran typed code, and encoded the Session back.
`foreign(f)` dropped from a lease back into `&mut Game`. `Game` implemented the engine's
host traits.

**Cost.**
- Size of the bridge: `sm_game` has 324 public traits, 179 of them named `*Host`,
  `*Lease`, `*Access` or `*Owner`, and 82 blanket impls over a generic host. #231 counted
  334 host traits, 83 blankets and about 50 species lease traits. The export codec was
  4,685 lines at its peak (`engine_ram_export_lines`, 4,620 today).
- Speed: about 20 ticks/s, with "about 85% of each tick" in the encode and decode at lease
  and `foreign` boundaries (memory note, 11-03; PERFORMANCE). 34 encodes per tick before
  the lease merges.
- Bugs the bridge itself caused: a mid-frame lease lost a queued sound effect at
  `$0643..$0665` on three rows; "Cause. Not isolated" (tick handoff, risk 1).
- Unwinding it: 91 commits tagged "(behaviour: lease merge)" from 10-31 to 11-04, an
  11-stage plan in a 1,911-line handoff, and the boundary ratchets (`foreign_calls` 495 to
  0, `game_host_impls` 98 to 0, `session_entries` 117 to 0) over about two days.
- All of that work is now frozen reference. None of the bridge survives into the rebuild.

**Root cause.** The migration policy required the old path to keep working at every step
("RAM remains authoritative until a complete producer/consumer interaction migrates",
ENGINE_STATE_MIGRATION). That made two authorities for one state and a codec between
them. The codec then had to be exact both ways, which kept the Session RAM-shaped (A3).

**Rule.** One authority for each piece of state, from the first line. No bidirectional
codec between the engine and the old layout. The only translation is one-way: oracle
machine state into engine state, in verification.

#### A3. The Session was a typed copy of RAM

**What we did.** Session fields mirrored WRAM so the export could round-trip byte for byte.

**Cost.**
- The Session decodes 126,781 bytes of WRAM (#251).
- RAM-named word bags: 18 enums and 202 macro sites (#231 theme F); `session_word_view_sites`
  is still 165.
- "Address-keyed dispatch (869 arms)" (#231 theme C).
- #251 had to defer the real typing work "until the RAM encoder goes", because "while the
  engine encodes Session back to RAM, every field must round-trip byte for byte".
- AGENTS.md's enum and newtype rules (#235, 11-04) arrived after 160k lines already
  existed in the other shape.

**Root cause.** The verifier compared whole RAM images, and the engine had to export
RAM for it (A2). Verification shaped the engine's state.

**Rule.** The engine's state is designed from the game's behaviour: enums with data,
newtypes, typed sets. Nothing in the engine exists to make a comparison easy. If a stream
needs the original's bytes, verification computes them from engine state, or the decoder
maps the oracle's bytes into engine terms.

#### A4. The engine did not run on its own until the last day

**What we did.** The Phase plan put the vertical slice (Phase 3, Gate 1) after the
decoupling. Phases 1 and 2 moved code.

**Cost.**
- On 11-04 the review "found no `Session::open` or `Session::step`", and `sm_app` still ran
  its frame loop through `sm` (#248).
- Gate 1 then landed in one PR in under a day (#282, 3,341 lines). Its own text says "the
  Session in `sm_game` and the `sm_runtime` renderer are not yet what runs". What runs is
  "the old game's host (the transpiled `sm/src/game`, the RAM image, the register-level PPU,
  DMA and HDMA models) with its emulators removed".
- So every gate went green on code that the target forbids.

**Root cause.** "Progress means behaviour crossing into a target package" (REVIEW,
10-29). Crossing was measured by where files lived and what they called, not by what ran.
A gate defined by "builds without `sm`" passed as soon as the old code was moved into
another crate.

**Rule.** The new engine runs end to end from the first slice: `Session::open`, `step`,
our renderer, headless replay. A gate counts only rows that the new spine executes, and
its build excludes the old crates by name.

#### A5. Several paths for one responsibility

**What we did.** Old and new paths lived side by side behind flags and cfgs.

**Cost.**
- Six checkpoint codecs (#231 theme D, 3-4k lines).
- Prepared-content and ROM double paths in room entry and message composition (theme I,
  2-4k lines).
- Several presentation paths for Samus, HDMA and scene provenance (theme E, 4-6k lines).
- Two hand-kept projections, engine side and oracle side, for every typed domain
  (FOUNDATION 8.1).
- The native renderer only compared, under a verify cfg, resyncing every frame (#177).
- Every duplicate had to agree, so changes cost twice. #231 sized the duplication at
  roughly 10-16k lines (est., themes D, E and I).

**Root cause.** Keeping the old path green while the new one grew, plus no rule that the
new path replaces the old one in the same change.

**Rule.** One path per responsibility. A replacement deletes what it replaces in the same
change. No feature flag or cfg chooses between two implementations of game behaviour.

#### A6. Verification hooks lived inside engine code

**What we did.** Probes (`verify_probe::hit`), observers and verify cfg blocks sat in game
code. Scenario rows required probe names (`require_hit` in `movies/segments.toml`).

**Cost.**
- Moves had to carry every probe across. The Polyp move dropped `lavaquake.update`, and
  the Ceres doors broke rooms 1-22 when a constant was renamed during a move (memory note
  "move checklist", 10-31).
- The tick handoff lists "verifier hooks" as its own blocker (section 2.3), and stage 10
  needed "the six verification hooks on the lease" before `foreign` could go.
- PLAN now forbids it: "Target packages get no probes, observers or verify cfg blocks."

**Root cause.** It was easier to instrument the code than to observe its state from
outside.

**Rule.** The engine exposes typed state and typed output. Verification reads them from
outside. Engine code has no verification cfg, probe or observer.

#### A7. Engine size grew while we called the work "moves"

**What we did.** The line ratchet became advisory on 10-31 so landings could ship (Jim:
"we will inevitably pivot to strict code volume reduction later").

**Cost.** Production lines rose from about 172k (10-30) to 207,963 (#177, 11-05) against a
100k goal, then a 125k phase target. #231's plan expected about 170k after the decoupling
and named 31-45k of cuts that need structural change.

**Root cause.** Size was treated as clean-up for later. The structure that causes size
(the bridge, duplicates, one trait per host need) was being added at the same time.

**Rule.** Each domain has a size budget, written before it is built and reported on every
landing. A domain that passes its budget stops for a design review before it grows more.

### Verification mistakes

#### V1. The reference was not independent of the code under test

**What we did.** The engine came from snesrev's C, and the oracle came from snesrev's
harness. That harness patched the ROM to fit the C: `kPatchedCarrys` (155 sites), about 50
`FixBugHook`/`PatchBugs` hooks, and startup ROM patches (memory note, 10-30).

**Cost.**
- 13 days (10-17 to 10-30) of "0 failures" were agreement with a patched game.
- The patched oracle applied 174 entries the engine reproduced, 39 of them
  gameplay-visible (10-31).
- With the faithful oracle, "280 of 329 chain rooms fail" (10-31). The fix needed a
  per-entry `patched`/`follows` mechanism (`tools/snesrev_patches.py`).
- Both sides of a lockstep frame drew through the same emulated PPU, so pixel agreement
  proved nothing (#250). There is still no reference other than LakeSnes.

**Root cause.** The oracle and the implementation shared an origin, a harness and a PPU.

**Rule.** The reference must not share code with the engine. Run the unmodified ROM. List
each harness patch the oracle keeps, with its effect. Add a second emulator or hardware
captures for anything an emulator could get wrong, starting with pixels.

#### V2. Whole-RAM lockstep with masks and per-frame adoption

**What we did.** Each frame, the verifier ran the oracle, rewound, ran the engine,
normalized some ranges, compared all of WRAM, then adopted a selected state.
`normalize_snapshot_ram` copied listed ranges from one side to the other first.

**Cost.**
- 31 masked ranges (tick handoff, risk 2), including direct page, `$0611..$0617` and the
  current enemy.
- Masked bugs surfaced far from their cause: a missing grapple `STZ` showed up 90 frames
  later as multiplier temporaries at `$05E9` (memory note, 10-31).
- Adoption every frame hid drift. REVIEW: "Free run is stricter than lockstep: masked
  scratch that later changes behaviour shows up."
- It forced the RAM-shaped Session (A3) and the bridge (A2).
- It was expensive enough that "running lockstep across everything on every change is
  too costly" (REVIEW, 10-31).

**Root cause.** RAM was the only representation both sides shared, so it became the
comparison. The masks were how we dealt with what did not fit.

**Rule.** Compare outputs and typed state in free run. Lockstep is for finding the first
bad tick, never for passing. Every exclusion is a named LEGACY row with a reason, and the
mutation sample covers it.

#### V3. Checks that could not fail, and rows that tested nothing

**Cost.**
- Mutation: Quality only checked that mutation anchor text existed. The nightly mutant
  runs were removed on 11-03, and the last two scheduled runs had failed (#242). When a
  typed mutant catalogue arrived on 11-05, it found three survivors (`b427b0d84`).
- Rows: `save-death-reload` and `shot-gate-spawn-parameter` had 0 failing frames, but
  neither side reached the scenario (#239). Only the facts caught it.
- The last full any% replay had zero pair differences but 121 missing gameplay facts
  (#249, 11-01).
- Self-replay results were reported next to fidelity results. PR #282 had to say
  explicitly that its determinism runs are "not fidelity evidence".

**Root cause.** A passing comparison was treated as proof. Nobody regularly asked
whether a planted bug would be caught, or whether the row reached its behaviour.

**Rule.** Every gate stream has a mutant that must fail it, run on a schedule. Every row
has facts that prove it reached its behaviour. Reports keep "reached", "matches" and
"deterministic" apart.

#### V4. Verification cost throttled all other work

**Cost.**
- About 20 ticks/s at the start of 11-03; the PR gate took about 28 minutes, later 9.5,
  then about 7 (memory notes; #254).
- Shards hit 55-minute limits; long-prefix rows needed 153,606 setup ticks, and 10,000 of
  them took 1,145 s (frozen report, 11-02).
- The checkpoint fingerprint hashed source text (85 files, 46,745 lines), so it
  invalidated every checkpoint on 9 of the first 11 commits of 10-29.
- Checkpoint formats changed often (e.g. "SMDISPL 7", "SMAUXGS 20"), and seeds had to be
  re-cut.
- Jim made speed the ordering rule on 11-03 because "the CI replay pipeline throttles
  everything".

**Root cause.** The verifier's cost came from the bridge (A2), and checkpoints were tied
to the engine's layout and source.

**Rule.** Verified replay speed is a design requirement from slice 0, measured on every
landing. Starts are keyed to oracle-side identities (seed digest, input prefix), never to
engine layout or source text. One canonical checkpoint format.

#### V5. Verification code grew and churned like a second engine

**Cost.**
- `sm/src/verification` saw +159k / -98k lines and holds 54k today. The cap set on 10-31
  was 57,802.
- The pipeline was written in Python (19k lines by 11-01), then ported to Rust from 11-03
  (VERIFY_PIPELINE).
- The scenario harness was approved as "one runner... never runs beside it as a second
  framework" (REVIEW condition 2), yet both existed together for days.

**Root cause.** Every engine shape needed its own verification adapters. The tooling
language was not settled first.

**Rule.** Verification has its own budget and one runner in one language. It depends
only on engine output types and the oracle, so engine refactors don't touch it.

#### V6. Seeded rows stood in for the whole game

**Cost.** 157 of 161 corpus rows start from a seed (FOUNDATION section 2). There is no low%
route, and the full movies have no continuous acceptance run (#249). "Selected rooms or
imported prefixes don't establish that" (PLAN).

**Root cause.** Seeds made rows cheap and the full movies were slow (V4).

**Rule.** Seeds are for localising. Continuous boot-to-credits runs of the any%, 100% and
low% movies stay on a schedule, and their facts are checked.

### Measurement mistakes (between architecture and verification)

#### M1. Lexical ratchets stood in for the goal

**What we did.** Progress was ratcheted by counting syntax: RAM sites, coupled files,
address APIs, `foreign` calls, host impls, lease entries.

**Cost.**
- Moves raised `sm_game_ram` sites from 3,932 to 6,933 and coupled files from 67 to 111
  (10-30), because the moved code carried `word!`/`slot!` addressing.
- Conversions into address macros hid RAM access from the count. Jim ruled on 11-02 that
  "address-shaped access is RAM access". One branch (`land/owned-pattern152`) added about
  2,650 address APIs that cost 12-18% of replay time.
- Definitions moved under the numbers: "rebaseline 85/14 to 141/24" (`ec0c14a73`), "11 ->
  36" (`cdce7e169`), and an "honest recount of 36" in #177.
- `sm_game_ram_sites` reached 0 on 11-04 while `Game` still implemented 82 host traits and
  every lease translated about 5,500 lines of export (memory note, 11-04).
- All four strict ratchets reached 0, and the slice that ran was still the transpiled
  game (A4).

**Root cause.** The counts were cheap to compute and easy to move. They measured access
syntax, not ownership or what executes. Twice-daily reviews that treated "no new commits"
as a stall rewarded moving a count (est., inference from REVIEW's cadence rule and 71
review check-in commits).

**Rule.** The gate measures behaviour that the new engine executes: green rows and green
(row, stream) pairs. Structural counts are advisory diagnostics. A count never replaces
a gate, and a new measure needs at least one reading that requires the engine to run
alone.

### Process mistakes (short)

#### P1. Rules and reports grew faster than code

- The frozen PROGRESS_REPORT reached 1,404 lines of qualification prose, with hashes,
  receipts and run IDs.
- Markdown churn was +98k / -90k for 7.6k lines that survive.
- #252 found contradictions between README, PLAN, REVIEW and skills; "reviewer may change
  goals/process"; and evidence in `$TEMP/collab/...`, with 48 artifacts recorded missing.
- One handoff is 1,911 lines long.
- Jim rejected a 35-item question list on 11-05 because the questions had "basically no
  context" and most were engineering choices already delegated (memory note).
- **Rule.** One short rules file Jim approves. Evidence is CI runs and PR text. Handoffs
  are short and die with their lane. Ask Jim only goal trade-offs, each with context.

#### P2. Landing mechanics

- #116 carried 94 commits, five red CI runs, and main idled 7 hours.
- Merge commits landed combinations that Verify never ran (#240).
- Fixed by a cap of about 25 commits per landing and `tools/lane/land.sh` (fast-forward,
  one at a time, onto a tested tree).
- **Rule.** Keep land.sh. Small landings, each on an exact tested tree.

#### P3. Many long-lived parallel lanes on one shared shape

- 120 remote branches and dozens of `collab/*` lanes stacked on each other. The ratchet
  numbers in commit messages were often "recorded after rebasing" another lane
  (e.g. `12a3c245a`, `ba7cfe351`).
- Parallel lanes on one coupled code base spent effort on rebases and
  re-baselines (est.).
- **Rule.** Parallel lanes only where the package boundaries already separate them. The
  spine is built by one lane until its interfaces hold.

## 3. What worked and should be kept

1. **The oracle as the authority, running the unmodified ROM.** The faithful oracle
   matched the lsnes hardware capture on rows 6 and 324, where the patched one did not
   (memory note, 10-31). Jim's rule that the real game decides held up.
2. **The oracle digest lock.** It stores per-domain running digests per scenario, with a
   combined mark every 1,000 ticks. It is a few hundred bytes, holds no ROM-derived data,
   and checks free run. Coverage grows by adding scenarios, not by running lockstep
   over everything. With the speed work of 11-03 (#176), the PR gate fell from about 28
   to about 9.5 minutes, and it is now about 7 minutes (#254).
3. **Lockstep only to localise.** A lock mismatch reruns its 1,000-tick span in lockstep,
   which names the first tick and field.
4. **Facts in scenarios.** They caught rows that matched but reached nothing (#239). The
   review: "Prefer adding facts over adding frames."
5. **`[exact]` refactors.** 863 commits carry `[exact]`: a refactor replays byte-identical,
   and it may never change the lock. That kept a huge refactor volume from drifting.
6. **Session-space compare through a one-way decoder** (`ee0379433`, 11-04). Oracle RAM is
   decoded into engine terms, and the first differing field is named. The decoder can be
   many-to-one. All "neither" bytes were resolved (#177 C).
7. **A vertical slice, once tried, is cheap and finds real bugs.** #282 took under a day.
   It found that the Tick's closing NMI latches the next Tick's controller word. A CI job
   built it with `sm/` deleted.
8. **Typed mutants.** `sm_mutants` found three surviving mutants on its first run.
9. **No tolerances, with a LEGACY register.** Every divergence is fixed or listed. That
   turned the patched-oracle discovery into a finite list instead of an argument.
10. **Profiling before optimising.** The 11-04 `perf` baseline showed that "game logic
    isn't near the top". The time went to settlement, PPU pixels, input projection and the
    export. That redirected speed work.
11. **Compiler-enforced package rules.** CI rejected `sm_game` depending on `sm`, and new
    target code forbids `unsafe`. The boundary held. The leak went through traits that
    `Game` implemented, which the rule did not see.
12. **The typed style rules and clippy in CI** (AGENTS.md). They came late, but they are
    the right standard for new code.
13. **External review with concrete work items** (#254). It found the "engine doesn't run
    alone" gap that our own measures hid.
14. **One landing at a time through `land.sh`.**

## 4. Design rules for the new architecture

Each rule names the mistake (A, V, M, P) or the success (W, numbered as in section 3)
it comes from.

| # | Rule | From |
| --- | --- | --- |
| 1 | New engine code is written to target types. It never starts from transpiled text. Old code is read as behaviour reference only. | A1 |
| 2 | The compiler forbids new crates from depending on old ones, by name, in CI. | A1, A4, W11 |
| 3 | Each piece of state has one authority. No codec from engine state back to the original's layout. | A2 |
| 4 | The only translation is one-way, oracle to engine terms, and it lives in verification. | A2, W6 |
| 5 | The engine's state is modelled from behaviour (enums with data, newtypes, typed sets). Nothing in it exists only to ease a comparison. | A3, V2 |
| 6 | No trait exists only so the old system can host new code. Prefer concrete context types. Count traits and blanket impls per domain. | A2 |
| 7 | The spine runs end to end in the first slice: open, step, our renderer, headless replay. | A4, W7 |
| 8 | The gate counts rows that the new spine executes, and nothing else. | A4, M1 |
| 9 | One path per responsibility. A replacement deletes the old path in the same change. No cfg or flag picks between two game behaviours. | A5 |
| 10 | No probes, observers or verify cfgs in engine code. Verification reads typed state and output from outside. | A6 |
| 11 | Each domain has a written size budget, reported on every landing. Going over it triggers a design review. | A7 |
| 12 | The reference shares no code with the engine: the unmodified ROM, with each kept harness patch listed. Pixels get a second reference. | V1, W1 |
| 13 | Free run against the lock is the gate. Lockstep only localises. | V2, W2, W3 |
| 14 | Every exclusion from comparison is a named LEGACY row, and the mutation sample covers it. | V2, W9 |
| 15 | Every gate stream has a scheduled mutant that must fail it. | V3, W8 |
| 16 | Every row has facts proving it reached its behaviour. Reports keep "reached", "matches" and "deterministic" apart. | V3, W4 |
| 17 | Verified replay speed is a requirement from slice 0, with a target and a measure on every landing. | V4, W10 |
| 18 | Starts are keyed to oracle-side identities. One canonical checkpoint format, versioned by schema, not by source text. | V4, A5 |
| 19 | Verification has its own budget, one runner, one language (Rust), and depends only on engine output types and the oracle. | V5 |
| 20 | Continuous any%, 100% and low% runs from boot stay on a schedule, with facts. | V6 |
| 21 | Refactors are `[exact]`: byte-identical replay, never a lock change. | W5 |
| 22 | Structural counts are advisory. A measure counts only if it needs the engine to run alone. | M1 |
| 23 | One short approved rules file. Evidence is CI runs and PR text, never local paths. Ask Jim only goal trade-offs, with context. | P1 |
| 24 | Land one PR at a time by fast-forward onto a tested tree. | P2, W14 |
| 25 | The spine is built by one lane. Parallel lanes start where package boundaries separate them. | P3 |

## 5. FOUNDATION.md against these lessons

FOUNDATION.md (`origin/research/foundation-design`, 1,129 lines, 2026-11-05) is a draft.
This section is deliberately critical. "Avoids" means the design as written prevents the
mistake. "Partly" means it helps but leaves a gap. "Repeats" means the same pattern
could come back.

| Mistake | Verdict | Why |
| --- | --- | --- |
| A1 translation as foundation | Avoids | J7 renames the old crates to `crates/legacy/*` and starts fresh crates, "so the compiler guarantees the new engine cannot call the old code". Risk: "a module from the old crates ... moves across once it meets the new contracts". The list (digest, compositor, lock, trace) is small. Keep it closed, and don't move game modules this way. |
| A2 byte-compatible bridge | Avoids | No encoder. The decoder is one-way (8.2). The wram stream is not produced by the engine. |
| A2 trait proliferation | Partly | The sketch's `Interpret<F>` host trait, `Family` with three associated types, and `Machine<S, A, M, X>` are generic frameworks. The old engine's 334 host traits started as reasonable "narrow host traits" too. Set a trait budget and prefer concrete context types per family. |
| A3 RAM-shaped state | Partly | Gameplay state is typed (4.1), and raw values convert at the edges. But video memory is a SNES-layout image in the Session (J3). The behavioural reason is real: the game reads VRAM back (R16). One stated benefit, though, is that it "makes `vram`, `cgram` and `oam` direct digests", which is verification shaping the engine. Check that the readbacks really need the whole image, and keep the comparison argument out of the decision. |
| A3 public schema | Partly | "The State schema is public plain data" (J8) so the decoder and projections can build and read it. That is sound, but the decoder's coverage rule ("every State field has a decode rule") can pull the schema back toward RAM if fields get added to hold oracle bytes. The rule should run the other way too: no field exists only because the oracle has a byte. |
| A4 engine runs late | Avoids, with a risk | Slice 0 is the Morph Ball Room on the new spine. But four preparation steps come first (crate plan, derive macro, schema skeleton, two oracle measurements, decoder skeleton). Time-box them. The derive macro (J9) is a framework built before any row is green. |
| A5 several paths | Avoids | One checkpoint codec through `Canonical`, replacing six. One projection per stream for both sides. One interpreter for eight families and four cursors. |
| A6 hooks in the engine | Avoids | Projections in `sm_tools` read `State`. The design has no probes. Say it as a rule in the design, not only in PLAN. |
| A7 size | Partly | It has a budget (83-123k), with domain ranges, and "a domain running past its upper bound is a design review". The upper end is close to the old phase target, and the low end of the domain report was 57-84k for `sm_game`. Report it per landing from slice 0, not only "when its first rows turn green". |
| V1 reference independence | Repeats, in part | J2 has the engine reproduce the faithful oracle, "including its `kept` `b` and `c` patches" (soft reset and the gunship save confirmation one frame late, message box deferral). Section 1 says Tick boundaries match the oracle "by construction" because the Tick copies how the oracle's harness steps. That builds harness artefacts (no overrun lag, line-granular IRQs) into the engine's structure. It is the same kind of mistake as matching snesrev's patches, though smaller and listed. Make these Reference-profile parameters, keep the strand design able to express hardware timing, and plan the Mesen check (#250) before the Tick structure sets. |
| V1 pixel reference | Partly | The compositor uses "the oracle's pixel rules (LakeSnes's colour conversion, brightness and priority), so the video digest can match". That makes LakeSnes the pixel authority. #250's second reference is not in the build order. |
| V2 masks and adoption | Avoids | The gate is free run on every stream except wram. Session-space lockstep only localises. |
| V2 unowned RAM | Partly | Dropping wram means scratch values the decoder does not model are never compared. #246 warned that tricks live there. Free run catches them only when they reach a typed field or an output on a path the corpus reaches. Keep a nightly report of oracle-side scratch behaviour where possible, and name the excluded ranges in LEGACY. |
| V3 checks that cannot fail | Avoids | "One mutant per stream" is part of slice 0's exit, and mutation sensitivity runs on every green row in Quality. |
| V3 facts | Not covered | Section 8.3 defines green by digests only. Rows need their facts too, or #239 recurs. |
| V4 speed | Repeats | FOUNDATION has no speed target and no measure. PERFORMANCE sets 600+ ticks/s headless and 100+ for the lock gate. Choices like copying uploads into a `DisplayFrame` every Tick, a CPU compositor and an `Upload { words: Box<[u16]> }` per write all have costs. Add a ticks/s target to slice 0's exit. |
| V4 checkpoint identity | Avoids | `Checkpoint { schema, fingerprint, state }` with a schema version. Starts come from decoded oracle seeds. |
| V5 verification size | Partly | It has a budget (12.5-19k) and one projection per stream. The decoder (4-6k), with PC and stack mapping to continuations, is the riskiest new verification code. It needs its own mutants. |
| V6 seeded rows | Partly | Boot is step 8 and the full movies are step 10. That is reasonable for building, but say that continuous scheduled runs of whatever is green start early, so continuity bugs show up before step 10. |
| M1 proxy measures | Partly | The gate is green rows plus (row, stream) pairs (J6), which needs the engine to run. But re-locking `ppu`, `apu` and the typed domains (J5) changes the lock that engine work must never change. Re-lock once, from the oracle side only, before the new engine's code exists, so the lock cannot follow the engine. |
| P1 rules and docs | Repeats, in part | The design is 1,129 lines plus three reports, and its first question list was rejected. It is a design, so length is more acceptable than in a report. Still, the build needs a short version (the rules of section 4 of this file, the types, the slice exits) that lanes can hold in mind. |
| P3 parallel lanes | Not covered | The build order is sequential by row cluster, which helps. It does not say who builds the spine. Rule 25 suggests one lane until the interfaces hold. |

### The three risks to raise first

1. **Oracle artefacts as structure (V1).** Matching the harness "by construction" is
   convenient now, but it makes the second reference (#250) and hardware timing expensive
   later. Keep them as profile parameters and LEGACY rows, not as the shape of the Tick.
2. **No speed budget (V4).** Speed is what throttled the last engine. Put a ticks/s target
   into slice 0's exit and measure every landing.
3. **Frameworks before rows (A2, A4).** The derive macro, `Family`, `Interpret<F>` and
   `Machine<S, A, M, X>` are reasonable, but each is a generic layer built before any row
   is green. Build slice 0 with the smallest concrete types that pass. Generalise once a
   second family needs the same shape.
