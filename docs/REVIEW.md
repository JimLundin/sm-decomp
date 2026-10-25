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
  them as banks retire. After A0.5, lane A tries instrumentation coverage
  (`cargo llvm-cov` on a segment run) to prove which code a segment executed,
  replacing hand-placed probes.
- Verification Rust (38,993 now, 33% of production) must fall from here; target
  under ~15,000 once the movies drive coverage.


**Lane A — coverage (one agent).** User direction 2026-10-24: drive coverage
with the published any% and 100% TASes (`movies/manifest.toml`), not hand-authored
holds. The 33 current routes in `tools/movie_inputs.py` are hand-written button
holds spliced onto an any% prefix; at ~1 check/hour they cannot reach 44/44.
Working hypothesis: the TAS desyncs because it was recorded on bsnes v085, where
some frames lag (the game misses an NMI and ignores input), while this engine
runs exactly one game frame per input word. Steps, one commit each:

- **A0.1 Locate — done (7867e91).** Any% on main: 129,712 frames, 0 oracle
  failures, but Samus never leaves Ceres (six rooms, no credits). Replay the full any% import on main with room logging; record
  under work item 1 the first frame and room where Samus leaves the TAS's route.
- **A0.2 Measure lag — done (cec060e).** Original lsnes capture: 129,712 inputs,
  127 gameplay rooms, 24,619 emulator-lag frames, hashes in the manifest. Finding:
  flat NMI counts do not identify lag, so A0.3 aligns on measured wait-entry input.
- **A0.3 Align — in progress (1b726be: 8,250 native steps, 0 failures; options
  return-tile quirk fixed with a 5-frame regression row).** Blocker named by the
  agent: the next door has 19 unmapped NMI advances (an extra native NMI hidden by
  oracle patches). **Next commit:** fix door scheduling so the aligned prefix passes
  the first Ceres door, with its regression row; extend frame count each commit.
  **07:52:** PLAN says original timing now matches 23 doors but the work is held
  locally while the 24 hand recipes requalify. Do not hold it: land the door-timing
  code and its regression row this hour; re-derive recipe input offsets mechanically
  in the same commit, and for any recipe that truly fails name it and its first
  differing frame in the commit body (those routes are replaced at A0.5 anyway).
  Also check why the capture shows no credits: if the movie's input ends before the
  credits, A0.4 continues with neutral input after the last frame; record which.
  **08:52:** door timing landed (9b8f29c, 61789c0: 9,266 steps). New blocker: at
  native index 9,266 the original spends 41 acknowledged NMIs (with input reads) in
  an audio upload the native engine does instantly. The agent took the documented
  fallback (hand routes, c7cca91 +1 check); accepted, but A0 is not dropped.
  **Next A0 commit (time-box one hour, then back to routes):** map the upload in the
  importer, not the engine: collapse the recorded words read while gameplay waits in
  that upload to one native step (keep the last read), as A0.3 already does for
  measured lag; replay and report the new aligned step count, or the first differing
  frame and which word differs (frame counter, NMI counter, music timer). That is
  measured data, not a guessed offset. Hand routes: one pending pickup/boss per
  commit, each with its segment row; ~1 check/hour is the 18:00 pace.
- **A0.4 Replay.** Aligned any% from boot to credits, 0 failures; then 100%
  (222,788 frames) and low%.
- **A0.5 Map.** Point pending `segments.toml` rows at frame windows of the
  aligned any%/100% runs with their existing expectations; many rows may flip in
  one commit, each named.

If A0.2 shows no lag, or the aligned movie still desyncs, record the first
divergence and its cause (boot alignment, input latch, RNG, APU handshake) under
work item 1 and fix that cause; hand routes are the fallback only after a
written finding. Time-box A0 to 8 hours of work before reporting.

**Lane B — bank retirement (other agents, one bank each).** Order: `sm_94`
(761) → `sm_91` (1,237) → `sm_90` (1,955) → `sm_88` (1,956) → `sm_81` (2,572)
→ `sm_80` (2,494) → `sm_82` (4,670) → `sm_8b` (4,938). Enemy banks start only
when lane A covers their encounters: `sm_a7` after Phantoon, `sm_aa` after
Golden Torizo, `sm_a6` after Ridley, `sm_a9` after Mother Brain; `sm_9b` after
Grapple. For each function in the bank: delete it if unreachable or duplicated
by an owned module, otherwise move it into the owning module on safe Session
state (no raw `g_ram`, no new Host/effects/adapter type). Name the checkpoint
segments that exercise it. A bank is done when its file is deleted. Total
production must fall with each bank; unsafe lines must fall with every commit.
Drafts not adopted within 2 hours are recorded in one line and dropped.
**`sm_94` deleted (ca42bc5, 06:51 CET); replacement ratio ~75% over the three
commits (owned +~580 / bank −772), slightly above the 70% guide.** Next bank
`sm_91` (1,237 lines, 104 unsafe), one family per commit: (1) `Samus_InputHandler`,
`Samus_Input_*`, `Samus_LookupTransitionTable`, pose helpers into the owned
player-input/pose module; (2) demo input (`*DemoInput*`, `DemoSetFunc_*`,
`LoadDemoData`); (3) suit pickups/palettes, `Samus_DealDamage`, `Samus_Initialize`,
X-ray; (4) `SomeMotherBrainScripts*`, `SamusFunc_E633*`, then delete the file. Mother
Brain paths are not movie-covered yet: move them unchanged and say so. Aim each
commit at ≤70% replacement; the production waypoint depends on it.
**07:52:** families 1-2 landed (4f8b05d, 563a45e); `sm_91` 844 lines left, ratio so
far ~80% (60% then 105%). The demo interpreter re-matches ROM opcode addresses
(`0x8427`…) and added an `Effects` method; express remaining families as data or
plain owned logic, no new trait methods. Even at 70%, `sm_91`+`sm_90` cut production
only ~840, so the 18:00 production line (≤114,000) also needs deleting unreachable
functions and production verify/probe hooks as each family moves; say how many.
**08:52:** `sm_91` 238 lines left (delete next commit); `sm_90` started (745daad,
1,514 left, 82% ratio). Hour ratio ~80%; production −267/h is just on the 18:00
pace. New owned modules still key handlers by ROM address (`0xe713`, `0xe8cd`,
`0xec1d` via `from_raw`); make them enums/fn values as each family moves, which is
where the remaining 10 points of ratio are.

**Waypoints** (Europe/Stockholm; review clock; a miss gets a diagnosis, not a new plan):

| By | Checks | Banks deleted | Legacy | Production | Result |
| --- | ---: | --- | ---: | ---: | --- |
| 10-25 06:00 | A0.4 any% to credits | sm_94 | ≤54,600 | ≤116,500 | **MISS** |
| 10-25 18:00 | ≥34/44 | + sm_91, sm_90 | ≤51,500 | ≤114,000 | |
| 10-26 18:00 | 44/44 | + sm_88, sm_81, sm_80 | ≤44,500 | ≤110,000 | |
| 10-27 18:00 | 44/44 | + sm_82, sm_8b | ≤35,000 | ≤104,000 | |

**06:00 miss (recorded 05:52 CET):** reached A0.2 smoke only (checks 24/44),
sm_94 not deleted (legacy 55,315, production 116,670). Cause: no agent has
committed since 7867e91 (23:48 CET), six hours idle; the plan was not the
blocker. Corrective step: on restart, land A0.2's Ceres-only capture and sm_94
step 1 first; the 18:00 waypoint stands unchanged and now carries sm_94 too.
**Corrective step done:** agents restarted ~05:50; sm_94 deleted 06:51 CET.

Map brief is done (c1fe795); no further map work unless the user asks.

## Check-ins

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
- **2026-10-25 00:55 (7867e91):** 2 agent commits since c1fe795, both `A:`
  (a427ba4 is the user-requested map SVG, not lane progress); **lane B 0 commits**
  (first silent check-in). A0.1 done, A0.2 smoke done. No Rust changed: Score
  `checks 24/44 · legacy 55315 · production 116670 · unsafe 4962 · verification
  38993` (= last); warnings 377. 06:00 waypoint: lane A on pace if the full
  capture lands by ~02:00; lane B behind (sm_94 734 lines left). No overall stall.
- **2026-10-25 00:20 (user direction):** size goal and verification policy added
  to the milestone plan: ~75k aspiration, ≤70% replacement ratio per bank,
  verification = movie segments as data, no new fixtures or production hooks.
- **2026-10-24 23:58 (user direction):** Lane A switched from hand-authored routes
  to aligning the published any%/100% TASes (A0.1–A0.5 above); waypoints moved.

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
