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
`legacy` = `cat sm/src/game/sm_*.rs | wc -l`. First lane-B commit adds that row
to `tools/progress.sh`.

**Lane A — coverage (one agent).** Land required checks in this order; reorder
only when the route forces it, and say so in the commit: pickup-power-bomb →
boss-spore-spawn → boss-crocomire → item-wave → item-grapple → area-wrecked-ship
→ boss-phantoon → item-gravity → boss-botwoon → item-space-jump → boss-draygon →
item-spring-ball → item-plasma → item-x-ray → item-screw-attack →
boss-golden-torizo → boss-ridley → area-tourian → boss-mother-brain → credits.
One commit per passing check (recipe + expectation only). If one check takes
over 3 hours, write the blocker under work item 1 and move to the next row.

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

**Waypoints** (review clock; a miss gets a diagnosis, not a new plan):

| By | Checks | Banks deleted | Legacy | Production |
| --- | ---: | --- | ---: | ---: |
| 10-25 06:00 | ≥27/44 | sm_94 | ≤54,600 | ≤116,500 |
| 10-25 18:00 | ≥32/44 | + sm_91, sm_90 | ≤51,500 | ≤114,000 |
| 10-26 18:00 | ≥38/44 | + sm_88, sm_81, sm_80 | ≤44,500 | ≤110,000 |
| 10-27 18:00 | 44/44 | + sm_82, sm_8b | ≤35,000 | ≤104,000 |

Map brief is done (c1fe795); no further map work unless the user asks.

## Check-ins

- **2026-10-24 23:52 (c1fe795):** 2 agent source commits in the ~20 min since
  8499a33, both lane B (`B:sm_94`), 0 docs-only; no stall. Milestone plan picked up
  at once: subjects prefixed, `Score:` line present, `progress.sh` reports legacy.
  Production 116,960 → 116,670 (incl. fda4ebd); legacy 55,342 → 55,315; verification
  38,993 (+109); unsafe 4,962 (−3); warnings 377 lines. No lane A commit yet (24/44;
  06:00 waypoint needs 27). Map: fixes (a)/(b) landed and render on both widths; brief done.
- **2026-10-24 23:10 (5096059):** user asked the review to steer harder. Added
  the milestone plan above (two lanes, bank-retirement unit, scoreboard line,
  waypoints) and made it binding from PLAN.md. Power Bomb route landed (5096059)
  as a non-required segment; checks still 24/44.
- **2026-10-24 22:52 (98aafd4):** 4 agent source commits since 2754947 (3 code,
  1 map SVG), 0 docs-only; no stall. **Coverage flat at 24/44** (no recipe commit;
  Power Bomb route advanced only in PLAN text). Production 117,104 → 116,960 (−144);
  verification 37,843 → 38,884 (**+1,041, 7× the deletion**); unsafe 4,978 → 4,965;
  warnings 339 unique (cargo; PROGRESS 378). PLAN.md 202 lines. Map: SVG text only,
  fits; brief items (a)/(b) untouched for a second check-in.
- **2026-10-24 21:52 (2754947):** 2 agent source commits since 3222fe3, 0
  docs-only; no stall. **Coverage moving again:** 23 → 24/44 (Norfair Reserve,
  9787aa2). Production 117,662 → 117,104 (−558, four projectile modules retired
  in one layer deletion, as asked); verification 37,843 (+339, tests moved out of
  production); unsafe 4,978 (−4); warnings 380 unique. PLAN.md 202 lines.
  Map: only the PROGRESS SVG text changed; rendered, fits.
- **2026-10-24 20:52 (3222fe3):** 1 agent source commit since 674d87a (code,
  −24 production, behaviour-equivalent helper reuse), 0 docs-only; no commits in
  the last 50 min. Not a full stall, but **coverage stalled a third check-in**
  (23/44, no recipe since 17:41) and throughput fell 4 → 1 commit. Production
  117,662 (−24); verification 37,504, unsafe 4,982 flat; warnings 381 unique.
  PLAN.md 202 lines (fixed). Map: SVG text only; re-rendered, fine on both widths.
- **2026-10-24 19:55 (674d87a):** 4 agent source commits since f0f163d (3 code,
  1 map), 0 docs-only; no overall stall, but **coverage stalled**: 23/44 for a
  second check-in, no recipe commit since a56fdd1 (17:41). Production 117,869 →
  117,686 (−183; first legacy-bank retirement, −59); verification 37,504 (+22);
  unsafe 4,982 (−60); warnings 380 unique (PROGRESS method: 432). PLAN.md 235 lines
  (rising). Map: Enter on "Kraid" now opens `sm_a7.rs`; remaining fixes below.
- **2026-10-24 18:52 (f0f163d):** 6 agent source commits since 2f18ba6, 0
  docs-only; no stall. Corpus flat 23/44 (hour spent making 20 unit tests run,
  which also reset checkpoint compatibility). Production 117,913 → 117,869 (−44,
  first decline; 0 legacy-bank lines retired); verification 37,482 (+53); unsafe
  5,042 flat; warnings 431. PLAN.md 228 lines. Map: brief largely met (see below).
  Changed: directive asks for legacy deletion, the unsafe-block quick win, shorter commits.
- **2026-10-24 17:52 (2f18ba6):** 1 agent source commit since a1d8ee2
  (a56fdd1: Maridia + Ice-return recipes, Python/TOML only), 1 docs-only
  (PROGRESS graph rewrite) = 50%. No stall, but corpus slowed 7 → 1 check/hour
  (23/44). Production 117,913 flat (third check-in without decline);
  verification 37,429, unsafe 5,042 flat; warnings 430 unique. PLAN.md 212 lines.
  Map: unchanged, brief not started (2nd check-in); rendered, Kraid search still
  lists only verification files. Changed: consolidation now runs in parallel.
- **2026-10-24 16:52 (a1d8ee2):** 2 agent source commits since 0ac7280, 0
  docs-only; no stall. Corpus 15 → 22/44 (+Bomb Torizo, Ceres, doors/elevator,
  save/death/continue, Ice Beam), zero failing frames. Production 117,986 →
  117,913; verification 35,653 → 37,429 (`verify.rs` +1,121 for six checks);
  unsafe 5,042 flat; warnings 430 unique (cargo: 835 incl. 405 duplicates;
  PROGRESS says 482). Map: brief issued 5 min earlier, not started, not re-rendered.
- **2026-10-24 15:42 (0ac7280):** directives picked up. Warnings 2,426 → 482;
  PLAN.md 1,642 → 203 lines with HISTORY.md; PROGRESS generated by
  `tools/progress.sh`; checkpoint corpus started at 15/44 required checks.
  Adoption correctly held. Watch: corpus `.inputs` files are gitignored with
  only checksums committed; a fresh agent must be able to rebuild them.

## Lane watch (2026-10-24 23:52)

- d0d1961 is a sound bank-retirement step (−90 engine, adapter and `CollInfo`
  record deleted, `Definitions` implemented directly on `Game`), but it moved a
  `printf` `unsafe` block into an impl that was `#[forbid(unsafe_code)]`. Route
  ROM-pointer warnings through a safe logger when the bank's last family goes.
- Commit bodies are compressed to the point of code ("no-default376, source1f25").
  Keep ~8 lines but write words: which checks ran, which segments, result.
- Lane A has not committed since the plan; the next review counts it.

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
