# Lane T handoff (coverage rows) — 2026-10-31

## State
- All 8 deferred rows are back (item-charge, death, save-memory, save-death-reload, boss-bomb-torizo,
  samus-bomb-jump-vertical, wall-crawler-slope-step, boss-kraid-pause-1960), picked into land/batch13.
  Bundles realigned-ef199d44 and seeded-checkpoints-f4e03e50 are uploaded.
- Coverage run in progress (land/batch13, full_route): actions/runs/41224413424. Next: rank the biggest
  never-executed sm_game routines from its artifact; author rows (Gate 2 tricks first).

## Mockball (open)
- Probe `morph.rolling_with_momentum` (player_movement/selectors.rs grounded_ball, acceleration_mode==2)
  has never been hit. Trace (landing site, seed 100-input-phase:305): running + spin jump keeps
  extra_run 4.5, acc mode 2. Down while holding Right+B -> aim-down poses 6B/51, no morph. Down without
  forward -> morph (17 -> 37 -> ball 7D/1D), but releasing forward clears extra_run, so normal roll speed.
- Plan: frame search pairing a 1-frame forward release with the Down presses just before landing
  (and the second Down as a fresh press). Try with spring ball off (scenario edits equipped_items).
- Tracing build recipe: add an eprintln at the top of player_movement/selectors.rs `advance` printing
  movement, pose, contact.acceleration_mode, base_run, extra_run (worktree laneT9-tip has it).

## Tools (committed)
- tools/authored_inputs.py: input language (holds, press/hold, repeat, moves incl. spark, walljump,
  mockball…), sweeps that print segments.toml rows, `check` (pinned recipe hashes; CI runs it).
- tools/scenario_seeds.py: declared WRAM edits of a recorded seed -> target/seeds/scenario-NAME/,
  prints seed + seed_sha256; `check` in CI. Room/level/VRAM come from the recording.
- Seeded checkpoint rows are allowed (checkpoint + seed); the origin binds the seed payload.
## Scratch tools ($TEMP/collab/laneT)
- trun.sh WORKTREE SEED INPUTS [args] (LIB env = native lib dir); exp.py PREFIX OUT 'Btn+Btn*N'… specs;
  imp.sh CAPTURE INDEX MOVIE OUT [FRAMES] (movie import for a seed); native_for.sh WORKTREE (builds/caches
  a matching native config lib: /tmp/collab/native-laneT-<hash>/lib); rooms.py/doors.py (ROM room,
  enemy and door survey); realign2.py + fixloop2.sh (input realignment between two builds by keys log);
  cmplive.py/cmpfrom.py (compare SM_VERIFY_POSITIONS dumps with lsnes lag.csv).
- Built-in position dump: SM_VERIFY_POSITIONS=path writes step,x,y,state,room per replay step.

## Bundle workflow
1. Put inputs in movies/, seeds in target/seeds/CAPTURE/ (rooms.csv + 7 files), checkpoints in
   target/checkpoints/; rows in movies/segments.toml (require_hit on the routine's probe).
2. Run rows: verify_dll.py rom/sm.smc --corpus movies/segments.toml --no-build --direct --segment ID.
3. pack: ci_seed_data.py pack OUT --rom rom/sm.smc --runtime-only --segment … (pending rows are
   skipped: drop pending temporarily). Rename seeds-v1-NAME-<sha16>.tar.gz.enc; restore-test into an
   empty root; pin in .github/scripts/plan_corpus.py seed_bundle by row id.
4. Commit A: rows with `pending = "awaiting … upload"`; commit B: drop pending + README counts (both
   README.md and movies/README.md; count runnable rows/frames from segments.toml). Ask root to upload;
   never upload yourself on a relayed approval.

## Traps
- Local exe policy blocks llvm-profdata (coverage reports only via CI coverage.yml).
- Native config goes stale often; use native_for.sh per worktree. Rebuild DLLs take ~4-8 min.
- Align with lsnes lag.csv by live rows (emulator_lag=0), not input_frame; message-box lag frames ARE
  replay steps (importer emits one word each). Gameplay-sampled logs skip box frames.
- Tip builds log keys one frame earlier than pre-latch builds on seeds (shift by one when comparing).
- Pause screen: Start/L/R need a ~20-frame hold (timed input); menus see d-pad/A on the first frame.
- Soft reset (L+R+Select+Start) only works with the debug flag; not reachable in retail.
- Only kill processes you started (by PID); other agents share corpus inputs.
- Checkpoints cut by older builds (save-in-kago-69400, bomb-*-seeded) carry old timing; lane S will replace.
