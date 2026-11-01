# Lane L3 handoff (actor leases): tip 39c421957 on origin/land/batch13, 15 commits, all picked by root

## Game-hosted session_slot!/boss! calls (local tally, never committed)
- Rows: chain 1-13, 96, 100, 118, 140, 200, 319, 320, 324; segments boss-ridley, boss-ceres-ridley,
  area-ceres, samus-forced-ceres, samus-scripted-intro, boss-mother-brain, mother-brain-baby-metroid-drain, mother-brain-escape-door.
- Lane Q -> tip: ridley 106k->0, corpses 30.6k->0, ceres_doors 5.3k->0, ceres_ridley 3.1k->0, escape_typewriter 827->0,
  ridley_tables 88->0, baby_metroid_encounter 226->0, baby_metroid_cutscene 1.5k->0, mother_brain_tubes 40->0,
  scrolling_sky 100.9k->0, zebetites 1.6k->0, gamet 220->0, mother_brain_visuals 3.6k->1 (seal_wall),
  mother_brain_fight 269k->2 (run_function), baby_metroid 217->4 (wait_for_camera). boss! Game calls: 0.
- session_slot! Game share on these rows: 35% -> 0.00% (7 calls, 3 sites). These rows don't exercise the families listed below.

## Deleting `SlotAccess for Game` (sm/src/game_state.rs:~1830): 61 errors (tried locally, reverted)
- Supertrait/blanket bounds: BossState for Game + BossLease for Game (game/actors/boss_lease.rs:26,85, plus 7
  BossEffects calls in it); EnemyHost/TouchHost/ShotHost/DrawHookHost for Game (actors/species_host.rs);
  AiHost/PreInstructionHost/InstructionHost for Game (session_runtime.rs:120-148); mother_brain_fight::Lease+Host,
  baby_metroid_encounter/cutscene Host for Game (sm_a9.rs:143-255); escape_typewriter::Host for Game (hosts.rs:353);
  PaletteHost for Game (mother_brain_visuals.rs:7); gameplay_frame::Host + Kraid pause/unpause hooks (gameplay.rs:13,100-109);
  menus Host + cinematic_host::Host (menu_host.rs:130,200); calculate_xray_window foreign (player_transitions_runtime.rs:119, visual_runtime.rs:151).
- Families still Game-hosted (their `Host for Game`): atomic, baby_metroid, botwoon, crocomire_reactions, draygon_eye (type Game), draygon_flight
  (+BossEffects for Game), escape_critters, fireflea, kraid_lint, magdollite, shaktool (Ai), skultera, spore_spawn, torizo (Ai),
  yapping_maw; in hosts.rs: boulder, phantoon, kraid, etecoon, dachora_run, fire_arc, statues (and hosts.rs:60 `type Game = Game`).
- Next: tally across the full route (or those families' boss rows), move family by family, then delete the impl once it compiles.

## Moving a family onto the scheduler's lease (pattern used here)
1. Entry points: AI arms go in `Runtime::actor_ai_legacy` (sm/src/game/session_runtime.rs) as `self.owned_call(module::fn)`; bank $A9 goes
   in `Runtime::bank_a9_routine` / `bank_a9_instruction` (sm_a9.rs). Instructions, touch, shot and draw hooks go in actor_*_legacy before the
   species fallback. Then delete the species.rs arm and its `Native*` variant, and drop the AiHost/InstructionHost bound.
2. Make the family generic over a lease trait: `pub trait Lease: BossLease<Game: BossEffects + ...> { lease-side effects }`.
   Implement it for `Runtime<'_>` in sm (foreign inside for game-only effects). Delete `Host for Game`.
3. Inside the bodies: remove `foreign(|g| ...)` wrappers whose body is lease-generic. Hoist session_slot!/random reads out of the foreign
   closures that stay. `word!(CUR_ENEMY_INDEX)` becomes `current_enemy()`. Raw RAM goes through `set_slot_byte`/`slot_word`
   (compatibility_byte counts in the ratchet).
4. Title: `(behaviour: lease merge)`; a pure read hoist is `[exact]`.

## Tools ($TEMP/collab/laneL3, local only)
- qual.sh TAG: in laneL3-base (a detached qualification worktree), runs build.sh, rows.sh (chain + segments lockstep), then tally.sh.
- qual_lock.sh TAG: check-lock (`--mode check-lock --lock scenarios/lock.toml`) on the locked rows, plus lockstep on chain 11 and 4 unlocked segments.
- tally.sh NAME ROOMS... (SEGS="..." env): applies $TEMP/collab/laneQ/instrument.py (counts SlotAccess::tally_host game vs lease),
  builds, replays with SM_SLOT_TALLY, restores the 4 instrumented files (`git checkout`), then writes sum.txt (laneQ/tally_sum.py)
  and byfn.txt (byfn.py: per-function).
- errs.py ROOT SMLOG: lists the compile errors' source lines.

## Traps
- Never edit a worktree while its DLL build runs ("source changed during build"); use laneL3-base for builds.
- Seeds: other lanes' cleanup deleted the laneT2/laneQ seed dirs. laneL3/target/seeds holds copies; laneL3-base links to them by junction,
  so remove a junction with `cmd /c rmdir`, never rm -rf. Segment .inputs came from $TEMP/collab/laneL2/corpus/movies.
- Heredoc python loses backslashes; write regex scripts with the Write tool.
- `area()` clashes between BossState and other host traits; that is why escape_typewriter's became `area_index`.
- Supertrait methods resolve through the bound; an explicit `use Trait as _` becomes an unused-import error.
- Local skips: boss-bomb/golden-torizo (no checkpoint here); mother-brain-baby-metroid-sacrifice is the known pending row.
