# Lane C2 handoff (2026-11-01): re-seed the seven stale checkpoint rows

Branch laneC2/reseed, worktree ...\sm-decomp-experiments\20261031\laneC2, on origin/land/batch13 377fb815f
(the S4 scenario format). Tip cd241c5b1 is a WIP commit: recipes and scenario seeds only. Nothing is wired
into scenarios/corpus.toml yet, and no bundle is packed.

## Done (verified with direct replays, `--seed`, on this build)
| row | start | inputs (movies/, copies in laneC2/inputs) | result |
| --- | --- | --- | --- |
| boss-bomb-torizo | scenario-bomb-torizo-fight:27 (Torizo hp 100) | authored-bomb-torizo-fight (2542) | default PASS, boss at 2207; faithful 50 fails, PASS with --oracle-keep 0x86a680 |
| samus-bomb-jump-vertical | 100-input-phase:27 | authored-bomb-jump-vertical (1057) | jump.bomb at 1007; default and faithful PASS |
| wall-crawler-slope-step | 100-input-phase:29 | authored-wall-crawler-slope (release, 8 movie words + 2) | steep slope steps with flipped BTS (92/D2/52) from step 0 (trace build); not yet run as a row |
| save-memory | any-boundary:110 (B167 save room) | authored-norfair-save (609) | save.sram.committed at 223; faithful PASS; expect room:B167 (was A201) |
| save-death-reload | scenario-one-energy-save:110 | authored-save-death-continue (3021) | default PASS, save 223, restored 1755, save-death-continue at 2837; faithful 2 fails (2410, 2470) = $82:831E + $80:9AF3, 0 with both kept |
| shot-gate-spawn-parameter | 100-input-phase:103 | authored-gate-reclose (127 movie words + recipe, 435) | E64B spawns at step 286 (trace build); window prefix 285, ticks 10 |

## Left, in order
1. Edit scenarios/corpus.toml for the 6 ids above (+ save-memory). Keep facts and tags; set start = seed:<sha>.
   The scenario_seeds `write` output prints the sha for scenario-*. For captures, use movies/room-seeds-*.csv.
   - bomb-torizo: ticks 2542, after_expectations 120, `unlocked` = LEGACY $86:A680.
   - bomb-jump: ticks 1057, no prefix.
   - crawler: ticks 8.
   - save-memory: ticks 609, fact room:B167.
   - save-death: ticks 3021, `unlocked` = LEGACY $82:831E/$80:9AF3.
   - gate: prefix 285, ticks 10.
   Update the row comments (the checkpoint history notes are stale).
2. docs/LEGACY.md faithful table: add save-death-reload @2410 to the $82:831E row and @2470 to the $80:9AF3 row.
   Add a $86:A680 row (Bomb Torizo low-health drool, verdict b): boss-bomb-torizo @1158, eproj Y $1AE3.. +1 on hardware.
3. Run the rows in 3 modes: `verify_dll.py rom/sm.smc --corpus scenarios --no-build --direct --segment ID`,
   then `--oracle faithful`, then `--audio-trace-directory DIR` (physical needs --corpus). Physical is not run yet.
4. Pack: `ci_seed_data.py pack OUT --rom rom/sm.smc --runtime-only --segment ...` for the 7 rows.
   The seeds need target/seeds/{100-input-phase,any-boundary,scenario-bomb-torizo-fight,scenario-one-energy-save}.
   Name it seeds-v1-<name>-<sha16>.tar.gz.enc, restore-test it, then `scenario.py lock --pin ID seed=.. inputs=.. bundle=..`
   and `lock --bundle FILE`. Record the seeded no-prefix rows with `lock --record`. Then `lock --check` and the gate.
5. Cherry-pick 7048175e9 (oracle-policy checkpoint guard; script laneC/guard.py) on top. With the 7 rows moved,
   only demo-attract-setup (2619) and options-menu-return-parameter (329) keep checkpoints. Both regenerate from
   boot; check that they pass with the guard.
6. Send the root the bundle path and sha256 to upload; never upload it yourself.

## Traps
- The patched and faithful oracles split by a frame at Samus's first landing from a seed. Recipes wait at rest
  before each move (norfair-save does), or the two modes diverge (the first save recipe missed the station in faithful).
- In B167 a jump from a standstill fails (a 1-2px hop and a freeze). Jump while walking. Samus must stand with
  (x-8)>>4 == 7 on the station step, then step onto it (Left 3) to trigger the save.
- The gate (C82A closed, at 26,5) opens on a shot of trigger C46 at (25,5), shot straight up from x~413 below it.
  It closes (E64B) on a second trigger after the opening animation, so shots before ~200 do nothing.
  eproj.spawn.initialized also fires at 139 for a fune fireball (DFCA), so the row must window from 285.
- Bomb Torizo can't be beaten from seed 27's 69 energy with the power beam, even at 999 energy. Hence the hp edit;
  it trips the $86:A680 low-health carry in faithful.
- mk/*-inputs prefixes come from laneC2/imp.sh (movie_inputs.py --lag-log --seed); laneC2/tr.sh and rr.sh run replays
  with positions. trace.patch is the eproj/steep-slope eprintln trace build (worktree removed).
