# External review

Living third-party review, commissioned by the user. A reviewer checks in
periodically, updates this file and, when needed, the Review directives
section of [PLAN.md](PLAN.md#review-directives-user-direction-2026-10-24).
The user directs implementation agents to adjust to it. Git history keeps
earlier versions; this file holds only current findings.

## Milestone plan (2026-10-24)

Set by the reviewer at the user's request so progress is countable. It replaces
"consolidate exercised subsystems" with one unit of work: **retire a legacy bank**.
Baseline at 5096059: 24/44 checks; `game/sm_*.rs` 23 files, **55,342 lines, 3,160
unsafe lines** (of 116,960 production and 4,965 unsafe total).

**Scoreboard** — last line of every agent commit body, measured locally:
`Score: checks 24/44 · legacy 55342 · production 116960 · unsafe 4965`.
`legacy` = `cat sm/src/game/sm_*.rs | wc -l`. From 2026-10-25 append
`· verification N` (verification Rust lines, as `progress.sh` reports).
Priority of the numbers: checks and legacy first, then unsafe, then size.

**Size goal (user aspiration, 2026-10-24).** Lines should fall substantially:
owned code uses native types and data instead of re-expressing 65816 routines.
Reference: snesrev's routine-by-routine C is ~76,500 game-logic lines. Budgets,
total production: ≤110,000 by 10-26 (layering check), ≤100,000 when all banks
are retired, **aspiration ~75,000** for the finished engine (game logic ~60,000,
platform/renderer/audio ~15,000). Each retired bank's replacement code should be
at most ~70% of the bank lines it removes; report the ratio per bank. Do not
compress formatting, delete comments or move code into verification paths to
meet a budget; behaviour and quirks still win over size.

**Verification policy (user direction, 2026-10-24).** Verification means movie
slices turned into focused tests, kept out of production code:

- A test is data: a `movies/segments.toml` row (movie, frame window from a
  checkpoint, per-frame oracle comparison, expectations) run by the one existing
  generic runner. New evidence for a change is a new or named row, not new Rust.
- Every frame-comparison failure becomes a regression row: the shortest window
  from a checkpoint before the first differing frame through a few frames after,
  committed with its fix.
- No new hand-written fixture modules (the `verification/*_cases.rs`,
  `*_checks.rs` style that pokes RAM/ROM words into a constructed world). Delete
  an existing fixture once a segment exercises the same code; that deletion is
  progress. A constructed test is allowed only for behaviour no movie reaches,
  under ~100 lines, with the reason in its first line.
- Production code gets no new probes, observers or `cfg(sm_verify_dll)` blocks
  (589 production lines in 91 files mention verify/probe/observer today); remove
  them as banks retire. After A1.3, lane A tries instrumentation coverage
  (`cargo llvm-cov` on a segment run) to prove which code a segment executed,
  replacing hand-placed probes.
- Verification Rust (38,993 now, 33% of production) must fall from here; target
  under ~15,000 once the movies drive coverage.


**Reset (2026-10-25 12:30, user grant to change goals and process).** Diagnosis
after 18 hours: the code moves, but slowly. (1) Each commit carried a heavy proof
ritual (mutation kills, independent controls, optimized + frontend builds, clean
reruns), so a 50-line bank family took 30-60 minutes on a machine limited to three
compilers; PLAN's **Fast loop** now replaces it. (2) Lane A spent ~10 hours aligning
one 129,712-frame movie end to end and is stuck on timing drift at the first Ceres
audio upload; hand routes give ~1 check/hour and the 18 pending rows are late-game.
(3) Lane B ran one or two agents on two banks while 30,000 of the 51,800 legacy lines
are enemy AI gated behind coverage. The changes below remove those three limits.

**Lane A — coverage (one agent): room-seeded TAS segments (A1).** Stop aligning the
whole movie. Restart alignment at every room instead: the TAS emulator gives the true
state at each room entry, so drift never accumulates past one room. Both the oracle
and the native branch start each frame from the same state, so an imperfect seed
can only lose route fidelity, never fake a pass.
- **A1.1 Dump (≤2h).** Extend `tools/lsnes_lag_log.lua`: at the first gameplay
  frame after each room change ($079B changes, $0998 back to gameplay) write WRAM,
  SRAM, VRAM, CGRAM, OAM and APU RAM (lsnes memory-region reads) plus the movie index to
  `target/seeds/` (local, ROM-derived, never committed; hashes only). Run on any% and
  100%. Commit the Lua and the room/index list it prints.
- **A1.2 Seed (≤3h).** Add `--seed PATH` to the verifier (verification side, not
  production): boot as now, load the seed's memories, rebuild owned state from RAM
  through the existing codecs/checkpoint loader, then run that room's inputs from the
  existing lag-aware importer. **Go:** one seed runs ≥600 frames with 0 oracle
  failures and reaches the same next room as the capture. **No-go:** name the state
  that could not be derived from RAM and fix that converter: RAM-to-Session loading
  is on the long-term path (save states, Map Rando), not throwaway.
- **A1.3 Flip rows.** `segments.toml` gains `seed = "<movie>:<room index>"` in place
  of an authored prefix. Point the 18 pending rows (bosses, late items, Wrecked Ship,
  Maridia, Tourian, credits) at 100%/any% seeds, several per commit, each named. Then
  move existing authored rows onto seeds and delete their recipes from
  `tools/movie_inputs.py` (credit: support code falls).
- Until A1.2 says go, at most one hand route per hour, starting with the frozen
  Crocomire route ($1994 scratch write at 167,281, regression row). A0 stays parked at
  10,555 with its finding in PLAN item 1; A1 does not need it.

**Lane B — bank retirement (up to three agents, one bank each).** Unchanged method:
for each function, delete it if unreachable or duplicated, otherwise move it onto safe
Session state with native types (enums/fn values, named views, range tests), no new
Host/effects/adapter layer; a bank is done when its file is deleted, with the full
corpus and scoped mutants run once before deletion. Report the replacement ratio per
commit; aim ≤70%. Each commit also deletes what the move makes dead (verify/probe
hooks: 607 lines/94 files, duplicated owned paths). Queues:
- **B1:** `sm_90` (finish) → `sm_81` → `sm_80` → `sm_82`.
- **B2:** `sm_88` → `sm_8b`.
- **B3 (new):** small banks `sm_89` (182) → `sm_a0` (458) → `sm_8f` (421) → `sm_b2`
  (880), then enemy families already met in a passing row (name the row and the enemy;
  e.g. Ceres/Crateria/Brinstar/Norfair common enemies in `sm_a2`, `sm_a3`, `sm_a8`,
  `sm_b3`). Boss banks wait for their A1.3 row: `sm_a4` Crocomire, `sm_a5` Draygon,
  `sm_a6` Ridley, `sm_a7` Kraid/Phantoon, `sm_aa` Torizos, `sm_a9` Mother Brain,
  `sm_9b` Grapple.
If only two lane-B agents run, B3 waits; never two agents on one bank.

History: A0.1-A0.2 done (7867e91, cec060e); A0.3 aligned 9,266 steps, then an
elapsed-NMI experiment reached 10,555 (parked 11:52). `sm_94` deleted (ca42bc5, ratio
~75%), `sm_91` deleted (52ba836, ~79%).

**Waypoints** (Europe/Stockholm; reset 2026-10-25 12:30; a miss gets a diagnosis and one corrective step):

| By | Checks | Banks deleted (cumulative) | Legacy | Production | Result |
| --- | ---: | --- | ---: | ---: | --- |
| 10-25 06:00 | A0.4 any% to credits | sm_94 | ≤54,600 | ≤116,500 | **MISS** |
| 10-25 18:00 | ≥28/44; A1.2 go/no-go recorded | + sm_91, sm_90, sm_89 | ≤50,300 | ≤115,300 | |
| 10-26 18:00 | ≥38/44 | + sm_88, sm_a0, sm_8f, sm_81 | ≤44,000 | ≤112,500 | |
| 10-27 18:00 | 44/44 | + sm_80, sm_b2, first boss bank | ≤36,000 | ≤109,000 | |
| 10-28 18:00 | 44/44 + 100% seeds | + sm_82, sm_8b | ≤28,000 | ≤105,000 | |

The previous 18:00 and 10-26 production lines (≤114,000, ≤110,000) assumed a 70%
ratio and missed three hours running at ~80%; the reset lines assume 75% plus hook
deletion. End-state budgets in the size goal stand (≤100,000 with all banks retired,
~75,000 aspiration).

**06:00 miss (recorded 05:52 CET):** no agent committed 23:48-05:50; the plan was not
the blocker. Corrective step done: agents restarted ~05:50; sm_94 deleted 06:51 CET.

Map brief is done (c1fe795); no further map work unless the user asks.

## Check-ins

- **2026-10-25 12:52 CET (603425e), stall (first, 28 min after reset):** 0 agent
  commits since ad9bef2 (12:22); no branches or held work visible. Score unchanged
  `checks 26/44 · legacy 51792 · production 115931 · unsafe 4650 · verification
  38936`. Likely cause: agents re-reading the reset plan. Next code expected by 13:52:
  A1.1 Lua room dump, B1 Crystal Flash setup, B3 `sm_89` start (B2 `sm_88` may resume:
  the three-queue plan supersedes item 2's sm_88 pause). 18:00 all lanes behind pace.
- **2026-10-25 12:30 CET (ad9bef2), reset at the user's request:** since 11:52, B1
  `sm_90` ×2 (145cf8c Crystal Flash, 3a4dac1 Draygon grab; ratios 64%, 76%), A 1
  (ad9bef2 probe removal, −13 production). Score `checks 26/44 · legacy 51792 ·
  production 115931 · unsafe 4650 · verification 38936`. Changed: PLAN's Fast loop
  replaces the validation ritual; lane A moves to room-seeded TAS segments (A1);
  lane B gains a third queue (small banks, covered enemies); waypoints reset.
- **2026-10-25 11:52 CET (80d0782):** 2 agent commits (half last hour): A 1 (80d0782,
  verifier cache tooling only), B 1 (e2fd645 `B:sm_88`), 0 unlabelled, 0 docs-only, both
  with `Score:`. Score `checks 26/44 · legacy 51925 · production 115959 · unsafe 4662 ·
  verification 38944` (last 26 · 52014 · 115992 · 4679 · 38944); warnings 367 (flat);
  verify/probe/observer 611/94 (flat). 18:00: legacy on pace, **checks, production and
  sm_90 (no commit, 1,039 left) behind**. **Lane-A stall (coverage):** second hour with no
  alignment or coverage code; elapsed-NMI model still local. B −33 production, not ≥100.
- **2026-10-25 10:52 CET (8588674):** 4 agent commits: **A 0**, B 4 (6b1b013,
  6c4a6f7, 8588674 `B:sm_90`; e091835 `B:sm_88`), 0 unlabelled, 0 docs-only, all with
  `Score:`. Score `checks 26/44 · legacy 52014 · production 115992 · unsafe 4679 ·
  verification 38944` (last 26 · 52390 · 116068 · 4707 · 39047); warnings 367 (370);
  verify/probe/observer 611/94 (615/93). 18:00: legacy ahead, sm_90 on pace,
  **checks and production behind**. Verification −103 (fixture lines deleted). No stall.
- **2026-10-25 09:52 CET (e6a39a5):** 4 agent commits: A 2 (bd003cd gate-spawn fix +
  10-frame row, e6a39a5 Wave pickup), B 2 (52ba836 **sm_91 deleted**, 84838b5
  `B:sm_90`), 0 unlabelled, 0 docs-only, all with `Score:`. Score `checks 26/44 ·
  legacy 52390 · production 116068 · unsafe 4707 · verification 39047` (last 25 ·
  52834 · 116131 · 4741 · 39040); warnings 370 (371); verify/probe/observer 615/93
  (flat). 18:00: checks on pace (8 in 8h), sm_90 and legacy on pace, **production
  behind** (−63/h vs −260/h). A0 upload experiment reported; next step named. No stall.
- **2026-10-25 08:52 CET (6361b0e):** 8 agent commits: A 3 (9b8f29c, 61789c0,
  c7cca91), B 5 (`B:sm_91` ×4, `B:sm_90` 745daad), 0 unlabelled, 0 docs-only, all
  with `Score:`. Score `checks 25/44 · legacy 52834 · production 116131 · unsafe 4741
  · verification 39040` (last 24 · 54148 · 116398 · 4892 · 39023); warnings 371 (371);
  production verify/probe/observer flat at 615 lines/93 files (`git grep -ci`, path
  filter as progress.sh). 18:00: lane B on pace (legacy, production, sm_91 238 left);
  lane A behind (25/44, needs ~1 check/hour). Hour's ratio ~80% (−1,314 legacy,
  −267 production). A0.3 parked at 9,266 steps; lane A back on hand routes under the
  written-finding fallback. PLAN.md 221 lines (>200). No stall.
- **2026-10-25 07:52 CET (563a45e):** 2 agent commits, both `B:sm_91` (4f8b05d,
  563a45e), Score lines present; **lane A 0 commits** (first silent hour; door work
  held locally, see A0.3), 0 unlabelled, 0 docs-only. Score `checks 24/44 · legacy
  54148 · production 116398 · unsafe 4892 · verification 39023` (last 24 · 54543 ·
  116478 · 4928 · 39010); warnings 371 (375); production verify/probe/observer 587
  lines/91 files. 18:00: lane B on pace for legacy, behind on production; lane A
  behind (0/10 checks). Verification +13 in a fixture host file: must fall. No stall.
- **2026-10-25 06:52 CET (ca42bc5):** agents back after a 6h stall. 5 agent
  commits: A 2 (cec060e, 1b726be), B 3 (`B:sm_94`), 0 unlabelled, 0 docs-only; all
  carry `Score:` (ca42bc5's has mojibake `Â·`). **sm_94 deleted** (ratio ~75%).
  A0.2 done; A0.3 at 8,250 aligned steps, 0 failures. Score now `checks 24/44 ·
  legacy 54543 · production 116478 · unsafe 4928 · verification 39010` (last 24 ·
  55315 · 116670 · 4962 · 38993); warnings 375 (377). No stall. 18:00 waypoint:
  lane B on pace for legacy, behind on production; lane A behind (needs +10 checks).
  Verification +17 (generic `state:` expectation, 2 segment row sets): acceptable,
  but it must fall; production verify/probe/observer lines flat (613, same grep).
- **2026-10-25 01:52–05:52 CET (9feb2f2…a172326):** five consecutive full stalls,
  0 agent commits since 7867e91 (23:48 CET); escalated to Jim; 06:00 waypoint missed.

## Lane watch (2026-10-24 23:52)

- d0d1961 is a sound bank-retirement step (−90 engine, adapter and `CollInfo`
  record deleted, `Definitions` implemented directly on `Game`), but it moved a
  `printf` `unsafe` block into an impl that was `#[forbid(unsafe_code)]`. Route
  ROM-pointer warnings through a safe logger when the bank's last family goes.
- Commit bodies are compressed to the point of code ("no-default376, source1f25").
  Keep ~8 lines but write words: which checks ran, which segments, result.
- 00:55: lane A is executing A0 as written (logger read-only, no frames dropped,
  hashes only, no fixture Rust). Label map/report commits `A:` only beside a lane
  A code or tool commit; a427ba4 alone counted as no lane progress.

## Corpus watch (2026-10-24 22:52)

- Consolidation is clean (no unsafe added, verify cfg kept behind diagnostic
  calls, coverage named per commit: Crateria 800, Ceres 8340–8780, boot 77381+),
  but small: −144 production/hour puts 110,000 ~48 hours out. 98aafd4 is +5
  production for 7 unsafe functions retired; acceptable, stated.
- Verification grew 7× faster than production fell, mostly hand-written fixture
  checks per adoption. Directive now caps verification added at production
  deleted per adoption; checkpoint segments are the preferred evidence.
- Coverage produced no commit this hour. If the Power Bomb route is not a checked
  segment by next check-in, that is a coverage stall for the third time today.

## Codebase map (user request, 2026-10-24)

**Status 23:52 (rendered c1fe795, 1440x900 and 390x844, no page errors): done.**
First screen shows status, targets and Kraid/Grapple/Doors buttons without
scrolling on both widths; search "Kraid" → one match "Kraid · sm_a7.rs", Enter
opens it with the game label in title and side panel; root files collapsed;
support files hidden. Left as is: graph nodes still show raw bank names
(`sm_82.rs`), and `sm_a7` is labelled only "Kraid" (Phantoon etc. lack aliases).
Neither justifies more map work; Pages generates from HEAD.

The user finds the [codebase map](https://jimlundin.github.io/sm-decomp/) too
overwhelming. Reviewed at snapshot a20e890d (already behind `main`).

What works: polished, offline, fast; breadcrumbs, search and the one-hop file
graph are good. Keep those.

Why it overwhelms:
- It answers "which files reference which", but readers ask "what does this
  part of the game do, is it migrated yet, and what is next". Nothing shows
  owned versus legacy: owned folders (`player_movement/`) and legacy banks
  (`sm_a9.rs`, 5.7k lines) look identical in one 259-tile alphabetical grid.
- Every screen is three full columns of text. The right panel repeats all
  nine curated tensions on every view, whatever is selected.
- Verification, docs, build and tools (~290 of 591 files) get equal weight
  with the game.
- Wording is internal jargon ("interval-scoped", "projection", "foreign
  calls", "compatibility aliases") plus long evidence disclaimers.
- Enter in the search box does not open the single match.
- The 656 KB `index.html` is regenerated and committed by hand, so it goes
  stale and adds report commits.

Brief (one agent, in parallel with corpus work; keep it small):
1. **Status first.** Default overview: gameplay responsibilities as tiles
   named in game terms (Samus movement, enemies, bosses, rooms and doors,
   projectiles, HUD, audio...), each coloured Owned / Partly owned / Legacy
   and sized by production lines. Derive status from existing markers
   (`forbid(unsafe_code)`, `unsafe`, `g_ram`, bank file); no hand-maintained
   status. Show remaining legacy lines against the 110k / 100k targets.
2. **Hide what readers don't need.** Verification, docs, build and tools are
   off by default behind one toggle.
3. **Context only.** The side panel shows only tensions touching the current
   selection; the overview shows at most the top three. Cut tensions to one
   plain sentence each plus "what resolves it". Move the evidence-limits text
   behind a single "About this map" link.
4. **Plain language.** Rename regions and descriptions so someone who knows
   the game but not the code can follow. Let search find game concepts
   ("Kraid", "grapple", "doors") via a small alias table.
5. **Fixes.** Enter opens the top search match.
6. **Stop committing the build output.** Generate `index.html` from `HEAD`
   in the Pages workflow and stop committing it. Agents no longer refresh
   the map; it is always current. This follows the user's local-validation
   policy: agents need not watch the workflow.

Done when: a first-time visitor can answer "what is migrated, what is left,
where is Kraid's code" from the first screen without scrolling, at desktop
and phone width; the generator and template together do not grow by more
than ~300 lines; no `index.html` commits remain in agent history afterwards.

## Initial review (2026-10-24, a408c8f)

## Baseline measured at a408c8f

| Measure | Value |
| --- | --- |
| Production Rust in `sm/src` (excluding `verification`/`verify` paths) | 94,285 on 2026-10-21 → **117,986** now |
| Verification Rust | 1,316 on 2026-10-21 → **35,653** now |
| Lines containing `unsafe` in `sm/src` | 8,537 on 2026-10-17 → **5,042** now |
| Files with `forbid(unsafe_code)` | 289 |
| Commits since 2026-10-21 touching only `docs/`, `PROGRESS.md`, `README.md`, `AGENTS.md` | **226 of 412 (55%)** |
| `docs/PLAN.md` | 1,642 lines (mandatory pre-read) |
| Markdown in `docs/` + `PROGRESS.md` | ~15,000 lines, plus 875 KB of HTML reports |
| `cfg(sm_verify_dll)` sites in production code | 183 |
| `.probe("…")` calls in production code | 114 |
| `cargo check --no-default-features --lib` warnings | 2,426 (1,084 unnecessary parentheses, 228 unused variables, 83 unused imports) |
| `#[test]` functions | 39 |
| Exercised normal gameplay for the adopted player work | ~8,577 frames, one gameplay room; any% replay reaches six rooms, no credits |

## What is working

- The per-frame ROM oracle is the project's strongest asset.
- Owned modules such as `game/player_movement/` are good Rust: forbidden
  unsafe, typed state, real enums (`LiquidAtHeight`), no RAM pointers.
- Unsafe lines fell by 41% and the library compiles.

## Findings and directives

### 1. Coverage is the bottleneck (highest priority)

Adoption claims rest on very little exercised gameplay. One room of normal
play cannot establish fidelity for locomotion, collision or transitions, and
targeted mutations only test what the replay already reaches.

Directive: build an oracle checkpoint corpus before further adoption.
- Keep the replay in sync with the TAS (input/lag alignment) as far as it can
  go, and/or capture `.smvchk` checkpoints from the oracle side covering every
  area (Crateria, Brinstar, Norfair, Wrecked Ship, Maridia, Tourian), each
  boss, the main item pickups, doors/elevators, save/load, death, and credits.
- Checkpoints are ROM-derived: generate them locally, never commit them. Commit
  only the recipe (movie, frame ranges, required probes) so any agent can
  regenerate the corpus.
- Done when one command replays every segment and reports per-segment
  pass/fail and rooms visited. Every later adoption must name which segments
  exercise it.

### 2. Production size is growing against the goal

The goal is below 100,000 production lines; it went from 94k to 118k in three
days. Adoptions add Host/Effects/Content layers and adapters while the bank
files stay large (`sm_a9.rs` 5.7k, `sm_8b.rs` 4.9k, `sm_a6.rs` 4.9k).

Directive:
- Every adoption commit reports production lines added and bank/legacy lines
  deleted in the same commit. Net production growth needs a one-line reason.
- Prefer deleting legacy bank code over adding new layers. Do not introduce a
  new Host trait, effects/content split or adapter unless an existing one
  cannot serve.
- Next production target: back under 110,000 before starting a new subsystem.

### 3. Process outweighs product

Over half of recent commits change only reports. `AGENTS.md` already forbids
dashboards and evidence stores, yet PROGRESS.md, the atlas and the
architecture report are maintained by hand on most commits.

Directive:
- Rewrite `docs/PLAN.md` to at most ~200 lines: goal, rules, validation
  strategy, the current item and the next three. Move dated history and
  completed-item narratives to `docs/HISTORY.md`, which agents need not read.
- `PROGRESS.md` becomes a short table generated by `tools/progress.sh`
  (production lines, verification lines, unsafe lines, warnings, checkpoint
  segments passing). Update it at most once per adopted milestone, in the
  same commit as the milestone.
- Stop refreshing `docs/codebase-map`, `docs/architecture.html` and
  `docs/target-architecture/report.html` per commit. Refresh only on user request.
- Target: under 20% of commits are docs-only.

### 4. Verification hooks have leaked into production code

183 `cfg(sm_verify_dll)` blocks and 114 probe strings sit in gameplay code.
Mutation tests patch source text from Python, so any reformat breaks them.

Directive:
- Route probes through one trait method on the host with a no-op production
  implementation; remove inline `cfg(sm_verify_dll)` from gameplay functions
  where a host method can carry the difference.
- Evaluate `cargo-mutants` (scoped to owned modules) to replace text-patch
  mutation recipes. Keep the Python recipes only where cargo-mutants cannot
  reach, and do not add new ones without trying it first.
- Do not grow the verification tree for each adoption; reuse the checkpoint
  corpus from item 1.

### 5. Hygiene

- One mechanical commit: `cargo fix` and `cargo clippy --fix` for the c2rust
  leftovers (parentheses, unused imports/variables), verified by a full
  replay. Then keep the warning count from rising; record it in progress.
- Explicitly `#[allow(improper_ctypes)]` the eight FFI declarations whose
  layout is already guarded by `offset_of!` asserts (e.g. `SpcPlayer`, `Dsp`),
  with a comment, so real FFI warnings stand out.
- Add plain `#[test]` unit tests for pure owned logic (speed tables, liquid
  physics, pose resolution). They are fast and catch errors before the oracle.

### 6. CI on main

Many agents commit directly to `main`. The 2026-10-24 local validation policy
stays: CI does not gate an individual adoption. But a red `main` is a stop
signal: the next adoption does not start until main is green again or the
failing commit is reverted.

## Order of work

1. Hygiene commit (item 5, first two bullets): small, mechanical, unblocks
   readable warnings.
2. PLAN/PROGRESS reduction (item 3).
3. Checkpoint corpus (item 1).
4. Resume subsystem adoption under the size rule (item 2), using the corpus,
   with probe consolidation (item 4) done inside the subsystems being touched.
