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
Priority of the numbers (from 2026-10-25 16:00): **unsafe** and checks first, then
legacy, then size. Legacy alone can fall by moving code; unsafe cannot.

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

**Throughput reset 2 (2026-10-25 16:00, user direction: finish within days, use
hosted CI).** Measured by the reviewer before this change:
- Pace: 36 lane commits in the 24h to 14:00, ~150 legacy lines and ~10 unsafe lines
  each. At that rate 4,600 unsafe lines take weeks. Hand-moving one function per
  commit cannot meet the goal; most unsafe comes from a few access primitives.
- Primitive experiment (compile-only, scripted, ~10 min): make `word!` bounds-checked
  `g_ram` indexing, then strip `unsafe` from every fn the compiler accepts (fixpoint):
  269 fns become safe, unsafe lines 4,603 -> 4,170, plus 224 now-unneeded `unsafe`
  blocks for `cargo fix`. On the oracle: 40 of 40 runnable rows pass (Segments run
  40203144686, branch `ci/safe-word`).
- What keeps the remaining ~1,100 bank fns unsafe (by fn): enemy raw pointers
  (`Get_X(k)` / `(*E).field`) ~450, `ram_ptr!` ~360, only calls to unsafe fns ~280
  (fall once callees are safe), ROM pointers ~110, hardware/IO ~50.
- Hosted oracle: `.github/workflows/segments.yml` runs every route-generated row
  (40 rows, 14 Windows shards) in 32 min with the ROM from secrets; 0 failures on
  b06f9ca (run 40199817511). Standard runners are free on this public repo.
- `ci.yml` had been red on every push since 10-24 11:53 (`--example sm_verify` and
  the detached build), unseen because the Fast loop builds neither.

**Lane M — mechanical safety (one agent, new: start a fourth agent if the machine
allows, else it takes the B3 slot; M's local work is `cargo check` only).**
Unit of work: one primitive made safe across the whole crate by a script, then the
fixpoint strip, in one commit. Push it to a `ci/<topic>` branch first; land on main
when Segments is green there (rebase and rerun the script, never hand-merge a
codemod). Body: the script (or its path), unsafe before/after, the Segments run.
- **M1** safe `word!` + fixpoint + `cargo fix` for unused `unsafe`. Prototype green:
  `ci/safe-word` (42c59cb on b06f9ca) passed all 40 runnable rows on Segments run
  40203144686. Land it: rebase onto main, apply the `ram_map.rs` change, rerun
  `tools/codemod/strip_unsafe.py` (on that branch), `cargo fix`, push to `ci/m1`,
  land when green. Target unsafe <= 4,000.
- **M1 revised (16:55 check-in; lane M stalled two check-ins, 0 commits, no `ci/m1`).**
  B2's finding (PLAN item 2) is right: 42c59cb's whole-RAM `&mut` can alias live
  enemy `*mut` pointers, and its strip made raw-pointer `Load24` safe. Do not land it.
  Sound first deliverable, **M1a, due 18:00 as a `ci/m1a` push**: the fixpoint strip
  alone, `word!` unchanged, stripping `unsafe` only from fns whose signature has no raw
  pointer and whose body then compiles with no `unsafe {}` block; then `cargo fix` for
  unused blocks. Then M2 (removes retained enemy pointers), then safe `word!` (M1b).
  Owner: the B3 agent (sm_a4 quiet this hour) until a fourth agent starts.
- **M2** enemy records by slot: replace `Get_X(k) -> *mut` and `(*E).field` with a
  safe slot accessor that re-borrows `game.g_ram` per access; codemod per bank,
  common enemies first (`sm_a2`, `sm_a3`, `sm_a8`, `sm_b3`), then bosses; fixpoint
  after each. Report unsafe per bank.
- **M3** `ram_ptr!` -> checked byte/word/slice views. **M4** ROM pointers ->
  `&'static` slices or typed table reads; hardware register fns safe.
After M4, re-measure and set the next primitive from what still holds unsafe.

**Lane C — CI as the verification pipeline (user direction 2026-10-25).**
- **C0 (now, lane-A agent, before more rows):** make `ci.yml` green on main
  (`--example sm_verify`, detached projectile build). Whoever breaks it fixes it next.
  Compile half done (c563093). Still red at 83281dc: the hand-written mutation steps
  (`player_detached/mutations.py`, `room_mutations.py`). **C0b, next lane-A commit:**
  delete those two steps and their scripts (old proof ritual; per-bank scoped
  cargo-mutants and Segments replace them); verification Rust falls with it.
- **C1 (reviewer, done with this change):** `segments.yml` runs on every push to main
  and on `ci/**` branches; dispatch with `segments=` for chosen rows.
- **C2 (lane A, after C0):** seed and lag-log rows in CI. Seed rows need the seed
  files; prefix rows need lsnes lag logs. Both are ROM-derived: store them as
  base64 secrets like the ROM, or as an encrypted artifact keyed by a secret. Waits on
  the user's yes (asked 2026-10-25 16:00). Then shard by seed so no shard replays
  from boot (target: whole corpus under 15 min).

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
- **A1.1 done for any%** (9290c2d: 197 room seeds, hashes only in
  `movies/room-seeds-any.csv`); the 100% capture rides along with A1.2. Crocomire's
  hand route is done (75fa0b1). No more hand routes: lane A's next code is A1.2
  `--seed` (first deliverable by 15:52: load one seed and report the first frame
  that diverges, even if no-go). A0 stays parked at
  10,555 with its finding in PLAN item 1; A1 does not need it.
- **A1.2 go reported, not landed (14:52 check-in).** PLAN item 1 (written in 4943d38)
  says a seeded trial passed 600 frames and reached the captured next room, with
  "integration after the queued bank changes", and the 100% capture (331 seeds) is
  done; none of it is committed. That is held work, a stall. Land it now as its own
  `A:` commit: `--seed`, the 100% index CSV and one `seed =` row naming the room and
  frames. It is verification-side, so it waits for no bank queue. The lane-A agent
  does no bank work until it lands. Then A1.3: flip one pending row per commit,
  starting with the cheapest, to reach ≥28/44 by 18:00.

**Lane B — bank retirement (up to three agents, one bank each).** Unchanged method:
for each function, delete it if unreachable or duplicated, otherwise move it onto safe
Session state with native types (enums/fn values, named views, range tests), no new
Host/effects/adapter layer; a bank is done when its file is deleted, with the full
corpus and scoped mutants run once before deletion. Report the replacement ratio per
commit; aim ≤70% (bd76e2a ran 154%: production must fall with each move; a
commit that grows production needs a one-line reason no smaller form exists). Each commit also deletes what the move makes dead (verify/probe
hooks: 607 lines/94 files, duplicated owned paths). Queues:
- **B1:** `sm_90` (finish) → `sm_81` → `sm_80` → `sm_82`.
- **B2:** `sm_88` → `sm_8b`.
- **B3 (new):** finish `sm_89` (48 lines left, the 18:00 deletion) before more `sm_a0`;
  small banks `sm_89` (182) → `sm_a0` (458) → `sm_8f` (421) → `sm_b2`
  (880), then enemy families already met in a passing row (name the row and the enemy;
  e.g. Ceres/Crateria/Brinstar/Norfair common enemies in `sm_a2`, `sm_a3`, `sm_a8`,
  `sm_b3`). `sm_a4` Crocomire (1,630) is unlocked by `boss-crocomire` and follows
  `sm_89` in B3. Other boss banks wait for their A1.3 row: `sm_a5` Draygon,
  `sm_a6` Ridley, `sm_a7` Kraid/Phantoon, `sm_aa` Torizos, `sm_a9` Mother Brain,
  `sm_9b` Grapple.
If only two lane-B agents run, B3 waits; never two agents on one bank.
Commit unit (from 16:00): a whole function family or small bank, several hundred lines
is fine; run locally only the rows the change reaches, Segments on main runs the rest.
After each M step lands, rebase onto it before continuing; do not re-add `unsafe`.

History: A0.1-A0.2 done (7867e91, cec060e); A0.3 aligned 9,266 steps, then an
elapsed-NMI experiment reached 10,555 (parked 11:52). `sm_94` deleted (ca42bc5, ratio
~75%), `sm_91` deleted (52ba836, ~79%).

**Waypoints** (Europe/Stockholm; reset 2026-10-25 12:30; a miss gets a diagnosis and one corrective step):

| By | Checks | Unsafe lines | Banks deleted (cumulative) | Legacy | Production | Result |
| --- | ---: | ---: | --- | ---: | ---: | --- |
| 10-25 06:00 | A0.4 any% to credits | | sm_94 | <=54,600 | <=116,500 | **MISS** |
| 10-25 18:00 | >=28/44; A1.2 go/no-go recorded | CI green; M1 on a branch | + sm_91, sm_90, sm_89 | <=50,300 | <=115,300 | |
| 10-26 12:00 | >=36/44; C2 decided | <=3,900 (M1 landed) | + sm_a0 | <=47,000 | <=114,000 | |
| 10-26 18:00 | >=38/44 | <=3,300 (M2 common enemies) | + sm_88, sm_8f, sm_81 | <=44,000 | <=112,500 | |
| 10-27 18:00 | 44/44 | <=2,300 (M2 bosses, M3) | + sm_80, sm_b2, first boss bank | <=36,000 | <=109,000 | |
| 10-28 18:00 | 44/44 + 100% seeds | <=1,200 (M4) | + sm_82, sm_8b | <=28,000 | <=105,000 | |

Unsafe targets after M1 are estimates from the fn counts above; the reviewer
re-baselines them after M2's first bank.

The previous 18:00 and 10-26 production lines (≤114,000, ≤110,000) assumed a 70%
ratio and missed three hours running at ~80%; the reset lines assume 75% plus hook
deletion. End-state budgets in the size goal stand (≤100,000 with all banks retired,
~75,000 aspiration).

**06:00 miss (recorded 05:52 CET):** no agent committed 23:48-05:50; the plan was not
the blocker. Corrective step done: agents restarted ~05:50; sm_94 deleted 06:51 CET.

Map brief is done (c1fe795); no further map work unless the user asks.

**Progress feed (user request, 2026-10-25).** Every push to main posts the measured
line deltas, lane, ratio and flags to issue #1 (`progress-feed.yml`). It reads the
`Score:` line for checks, so keep that line exact; agents do not maintain the feed.

## Check-ins

- **2026-10-25 16:55 CET (3ea69a0), stall in lane M (2nd check-in, 0 commits):** 7 agent
  commits since 9b062a7: A 4 (Draygon, Space Jump, MB spawn regression, C0 compile),
  B1 `sm_81` 1, B2 `sm_88` 2 (unsafe -31 then +33: backend contracts restored, sound),
  B3 0, M 0; all `Score:`, 0 docs-only. Score `checks 37/44 · legacy 49512 · production
  115340 · unsafe 4546 · verification 39240` (last 35 · 49698 · 115375 · 4554 · 39232);
  warnings 357. 18:00: checks/legacy/banks met, production 40 over, `ci.yml` red,
  M1 unsound. Changed: M1a (sound strip) to B3 agent; C0b drops mutation steps.
- **2026-10-25 15:55 CET (9b062a7), no stall, short hour (15 min after reset 2):**
  5 agent commits since f2d3a05: A 2 (6f579a3 Ridley, f207b36 Phantoon: checks 33 -> 35),
  B1 `sm_81` 1 (a2088b5), B2 `sm_88` 1 (89%), B3 `sm_a4` 1 (44%); M 0 (lane opened at
  15:40); all labelled with `Score:`, bodies <=8 lines naming rows. Score `checks 35/44 ·
  legacy 49698 · production 115375 · unsafe 4554 · verification 39232` (last 33 · 49901 ·
  115411 · 4567 · 39228); warnings 357 (362). 18:00: checks, legacy, banks met;
  production 75 over (on pace); unsafe/M1 branch and green `ci.yml` still open. No changes.
- **2026-10-25 15:40 CET (f2d3a05), throughput reset 2 at the user's request:** checks
  33/44 (18:00 target 28 already met), sm_90 and sm_89 deleted, legacy 49,901, unsafe
  4,567. Added lane M (scripted unsafe-primitive codemods), lane C (hosted CI as the
  verification pipeline; `segments.yml` landed), unsafe-first scoreboard and unsafe
  waypoints. PLAN's Fast loop rewritten in ASCII: an agent's editor re-encoded it each
  commit (15 KB -> 53 KB in three commits). `ci.yml` red since 10-24 11:53: C0.
  16:10: safe-`word!` prototype passes 40/40 rows on hosted Segments; M1 ready to land.
- **2026-10-25 14:52 CET (b06f9ca), lane-A held work:** 11 agent commits since bd76e2a:
  **A 0**, B1 `sm_90` 6 (incl. b06f9ca fixing 21a3f50's missing file), B2 `sm_88` 2,
  B3 `sm_89` 1 + `sm_a0` 1 (before sm_89 finished), 1 unlabelled user-requested feed
  (2ccf738); 0 docs-only; all B with `Score:` (4943d38's `·` is mojibake). Score
  `checks 27/44 · legacy 50974 · production 115629 · unsafe 4603 · verification 38936`
  (last 27 · 51744 · 115957 · 4648 · 38936); warnings 362 (367); verify/probe/observer
  607/92. Ratios 52-91%, production −328. 18:00: legacy, production, sm_90 (522 left),
  sm_89 (48) on pace; **checks behind**: A1.2 go held locally, see Lane A. Verification
  flat 3 hours: no fixture deleted. Changed: land A1.2 now; finish sm_89 before sm_a0.
- **2026-10-25 13:52 CET (bd76e2a), no stall:** 4 agent commits since 603425e: A 3
  (5354e6c, 75fa0b1 code; a822933 docs-only resource policy), B1 `sm_90` 1 (bd76e2a,
  ratio 154%), **B2/B3 0** (sm_89 unstarted), all labelled with `Score:`. Score `checks
  27/44 · legacy 51744 · production 115957 · unsafe 4648 · verification 38936` (last 26 ·
  51792 · 115931 · 4650 · 38936); warnings 367; verify/probe/observer 607/92. **Boss row
  flipped: `boss-crocomire`** (+2 regression rows). 18:00: checks on pace; legacy,
  production, banks and A1.2 **behind** (A1.1 not started; production rose). Changed:
  hand routes closed; `sm_a4` unlocked in B3; production must fall per move.
  Post-push: 9290c2d landed A1.1 (any%), marked done; map tiles change (5569c08,
  Jim's request) renders cleanly at 1440 and 390 px.
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
