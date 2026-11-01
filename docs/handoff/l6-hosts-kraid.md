# Lane L6 handoff (hosts.rs families + Kraid onto the scheduler's lease)

Branch laneL6/hosts-kraid, tip 910e393ba, 7 commits on L5 batch-1 tip 0cc19efe8.
Worktree: C:\Users\Jim.Lundin\AppData\Local\sm-decomp-experiments\20261031\laneL6 (clean).
Qualification worktree: ...\20261031\laneL6-base (detached; real seed/movie copies, no junctions).

## Commits (each GATE OK locally)
- 4bd4200ae fire arc (behaviour: lease merge)
- a79fc3137 boulders (behaviour: lease merge)
- d07df68ad statues (behaviour: lease merge)
- bed674472 etecoons (behaviour: lease merge)
- b06856a81 dachora (behaviour: lease merge)
- 50cd4a92a Mini Kraid (behaviour: lease merge)
- 910e393ba Kraid + kraid_lint/kraid_sinking/kraid_fingernails (behaviour: lease merge)

## Qualification so far (stopped at the wind-down)
- At b06856a81 (first 5 commits), lockstep: chain 19 40 41 42 44 68 73 75 82 109 113 115 118-120 123 125-127
  129 132-134 136 168 188-192 205 207 311-313: 35/35 PASS. Segments PASS: boss-kraid, boss-kraid-pause-400/950/1030/1960,
  enemy-dachora-shinespark, enemy-etecoon, enemy-etecoon-jumps, boss-phantoon-second-round, enemy-boulder-bounce/roll.
  NOT run yet: enemy-fire-arc, area-norfair, boss-phantoon, phantoon-exit-spawn-parameter, area-tourian, area-wrecked-ship.
- Not run at all: check-lock (every candidate chain room and the 11 locked segments are in scenarios/lock.toml), the
  re-tally, and any replay of 50cd4a92a / 910e393ba. Mini Kraid's rooms are chain 69-72, 74, 76-81 (A471/A4DA/A521/A56B);
  the baseline candidate list above never ran Mini Kraid.
- Ratchet: 0cc19efe8 1835 sites / 96 files -> 910e393ba 1803 / 93.
- Baseline tally (0cc19efe8, local, $TEMP/collab/laneL6/tally-base/sum.txt): Game-hosted session_slot on these rows was
  dominated by kraid.rs 206k, etecoon 59k, kraid_lint 27k, kraid_sinking 21k, boulder 12k, dachora 5.6k, phantoon 1k,
  phantoon_actions 3.4k, statues 1.4k, fire_arc 1.3k.
- Scripts: $TEMP/collab/laneL6/qual.sh TAG COMMIT (build, lockstep, check-lock, ratchet, tally; NOTALLY=1 skips the tally),
  rows.sh, tally.sh, build.sh, ed.py (LF-preserving replace helper; Python write_text on Windows writes CRLF otherwise).
  Root says the scenario format lands soon: switch rows.sh to `--corpus scenarios` once it does.

## Dispatch layout (for merging with L5)
- sm/src/game/session_runtime.rs, at the end: `impl Runtime` with kraid_and_hosts_ai / _instruction / _touch / _shot,
  each called first in actor_ai_legacy / actor_instruction_legacy / actor_touch_legacy / actor_shot_legacy.
- species.rs: only line deletions (arms, `+ x::Host` bounds, import names). One edit: species::shot's unused `j` became `_j`.
- Lease impls for Runtime live in sm/src/game/hosts.rs (fire_arc, boulder, statues, etecoon, kraid::Lease).
- Deleted: sm/src/game/actors/kraid_lint.rs (+ mod line), room_mechanisms::lower_acid, session_runtime::spawn_collision_plm,
  the display_refresh queue_native_transfer re-export.

## Left
1. Phantoon + phantoon_actions: initialize<H: Host> still runs on the game; move it to the lease (spawn_hdma and the
   minimap go foreign). Dispatch its AI/touch/shot/hurt and the call_a7_function instruction (10977418) in the helpers.
   run_function_on's fallback runs the game's table; make it unreachable. In `dead`, hoist `game.area()` out of the
   foreign closure. Then delete phantoon::Host's run_function/direction_to_player and `BossLease<Game = Self>`.
2. Kraid pause hooks (gameplay.rs): kraid_pause::Host still needs SlotAccess for Kraid's y. Give it a
   `kraid_y` method that the game answers from a lease (Runtime::new(game).session().actors.slots[0].y). Row clears now go
   through TilemapQueue, which takes the BG2SC byte as a parameter.
3. Delete the remaining Host-for-Game impls in hosts.rs: phantoon, kraid::Host (it keeps the boss word and the observer;
   could move to the lease via foreign). dachora_run, etecoon, statues, fire_arc and boulder are already gone.
4. kraid::Host, fire_arc, boulder, statues, etecoon, dachora and mini_kraid still bound `Game: BossEffects`. BossEffects
   has SlotAccess as a supertrait, so L5's BossEffects/SlotAccess change has to drop it.

## Traps
- verification/flags_kraid_cases.rs asserts RAM is unchanged after is_dead. It now opens Runtime::new around the call.
  No tool here calls that DLL export, so it hasn't been exercised.
- Statue/etecoon/dachora function words now dispatch in-module; any value outside their `function` mods hits the
  unreachable diagnostic instead of the shared bank table.
- laneL6-base seeds are real copies (504 MB). scenario-dachora came from laneT4. No junctions.
