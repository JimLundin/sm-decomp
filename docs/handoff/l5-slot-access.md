# Lane L5 handoff (SlotAccess for Game): tip 35d263700 on laneL5/slot-access (base origin/land/batch13 9d0c0ab2f)
## Commits (each GATE OK; ratchet ram_sites 1842 -> 1809, coupled_files 99 -> 94, never rose)
- Batch 1 (QUALIFIED, root picking): 72ace745e atomic, 36f89408f fireflea, cf466d33c skultera, bec697954 shaktool,
  73b59e243 bull, 0cc19efe8 kihunter. Full route check-lock 327/327 locked PASS; the 4 unlocked rooms (chain 11, 86,
  211, 330) PASS in lockstep; 44/44 segments PASS in lockstep; the 34 locked of those PASS in check-lock.
- Batch 2 (NOT QUALIFIED; its run was stopped at wind-down): 7a348328d yapping_maw, f693c3460 zeb, d15aee487 geega,
  beaf59ab8 powamp, bfb5d7f45 wrecked_ship_spark, 47bd619bb blue_brinstar_face_block, e9ba78203 morph_ball_eye,
  0858204ac escape_critters, dbda50e28 magdollite, 35d263700 spore_spawn. Next: qualify 35d263700 (qual6.sh).
- BossState gained lease queries: player_angle, shot_direction, current_random, select_enemy, kill_current,
  area_boss_byte (Runtime: typed Session reads; Game's impl delegates to EnemyHost and goes with the impl).
- WIP, not committed: Crocomire onto the lease. $TEMP/collab/laneL5/crocomire-wip.patch, which `git apply` takes
  on 35d263700. It has 2 errors left: the visual_runtime.rs:7 re-export of the removed room_visuals
  advance_crocomire/align_crocomire_background, and session_runtime's crocomire_shot fn pointers. For the second,
  use closures `|r| count_hit(r)`, or match in actor_shot_legacy.
## Families still on Game (baseline full-route tally at 9d0c0ab2f; batch-1/2 re-tally never finished)
- Baseline Game share, session_slot!: chain shards 62% / 47% / 30% / 11%; segments 27% and 62% (two sets);
  boss_ram 0-18% chain, 87% segments (Crocomire).
- Still on Game after batch 2: crocomire_reactions (+room_visuals align/advance_crocomire), draygon_flight,
  draygon_death, draygon_grab, draygon_sweep, draygon_eye (look_at_samus/set_function in draygon_function),
  botwoon (head fns in b3_pre_instruction, set_hitbox/set_spitting instr), torizo (Ai, huge instruction table),
  baby_metroid (wait_for_camera), mother_brain_fight run_function, mother_brain_visuals seal_wall,
  menus/game_over. Lane L6 owns the hosts.rs set plus Kraid: kraid*, mini_kraid, phantoon(+_actions), etecoon,
  dachora_run, boulder, fire_arc and statues.
- Spore Spawn's arms in species::draygon_function (10873627..) still run spore fns with Game as its own lease.
  They go with the Draygon move.
## Deleting SlotAccess for Game at 35d263700: 58 errors (errs-tip.txt)
- Supertraits: BossState and BossLease for Game (boss_lease.rs) and BossEffects (host.rs: drop its SlotAccess
  supertrait and add explicit bounds). EnemyHost, TouchHost, ShotHost and DrawHookHost (species_host.rs).
  AiHost, PreInstructionHost and InstructionHost (session_runtime.rs:130-159). gameplay_frame::Host and Kraid's
  pause hooks (gameplay.rs). menus Host and cinematic_host Host (menu_host.rs). calculate_xray_window, called
  foreign (player_transitions_runtime.rs:119, visual_runtime.rs:151). mother_brain_fight Lease/Host and
  baby_metroid_* (sm_a9.rs). escape_typewriter (hosts.rs:353). PaletteHost (mother_brain_visuals.rs:7).
- Family Host for Game left: baby_metroid, botwoon, torizo Ai, kraid_lint, crocomire, draygon_eye/flight,
  spore_spawn, plus L6's hosts.rs set.
## Pattern used (see L3's handoff too)
- Arms go in Runtime::actor_ai_legacy / actor_instruction_legacy / actor_touch_legacy / actor_shot_legacy /
  actor_hook_legacy (session_runtime.rs) as owned_call. Delete the species.rs arms and the family's `+ X::Host`
  bound. TOUCH/SHOT tables are separate from the ai table: move those arms too (spore and crocomire have them).
- A family's `fn(state)` dispatched through pre_instruction becomes a lease main_ai that matches its state.
- Bounds: `BossLease<Game: BossEffects>` (+ ActorAccess when edit/resolve_actor are needed). Replace
  game.lease().X with lease ops: write_slice_native/le -> set_slot_word, lease_native_word -> slot_word,
  lease_native_record -> cartridge::native_record. Hoist session_slot! out of foreign closures.
- Avoid boss_ram::get/set and word! for new reads: they count in the ratchet. Use the typed BossState queries.
## Tools ($TEMP/collab/laneL5; local only)
- smcheck.sh: quick rustc check of sm_data, sm_game, sm_runtime and sm metadata (verify cfg). Logs go to
  game.log/sm.log. errs.py is in laneL3.
- commit.sh MSGFILE [git args]: cargo fmt, gate.sh, ratchet, then commit only on GATE OK. ed.py + py: LF edits.
- recipe.py OUT ROOMS|all: chain recipe. tally6.sh NAME (ROOMS=none|list, SEGS=...): instrumented full route in
  4 shards + segments, run in laneL3-base. qual6.sh TAG: build, full-route check-lock in 4 shards, lockstep of the
  4 unlocked rooms and segments, then check-lock of the locked segments.
- After S4 lands on batch13, the corpus is `--corpus scenarios` and UNLOCKED moves to notes, so update the scripts.
## Traps
- Python write_text on Windows writes CRLF: always pass newline="\n" (ed.py does).
- Heredocs mangle backslash escapes: write such scripts with the Write tool (see mag2.py).
- Never edit a script that is running in the background; bash reads it lazily.
- laneL3-base: seeds are junctions into laneL3 (use cmd /c rmdir). I copied missing seeds from laneD and
  inputs from laneT4.
- ed.sub with count>1 matches substrings, so a 4-space pattern also hits 8-space lines.
