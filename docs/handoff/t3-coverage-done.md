Lane T3 handoff (2026-11-01)
State: 18 new corpus rows plus one engine fix. All of it is on land/batch13, or rebased onto it at
b42563632 (laneT3/coverage). Every new row passes lockstep patched and faithful, free run and
check-lock. lock_gate.py check: 624 locked, 41 unlocked.
- Batch A (bundle seeds-v1-routeenemies): eleven enemy rows from route rooms, plus samus-ceres-push
  (Ceres movie words to 1000, then released).
- Batch B (seeds-v1-grapplewall): samus-grapple-walljump in the Colosseum (wallgrab, then walljump).
  It found a bug: facing left, the grapple wall-jump check didn't negate the distance ($90:9CFB).
  Fixed in jump.rs.
- Batch C (seeds-v1-unreached): boss-draygon-grapple-held, boss-ceres-ridley-fireballs,
  boss-phantoon-second-round, enemy-dachora-shinespark (new scenario `dachora`), enemy-etecoon-jumps.
- Lock commit 4ee954261: locked 4 re-seeded rows. Croc, ice-return-business, music-fir-selector10
  and pickup-reserve went to UNLOCKED as drift, with LEGACY rows.
- movies/README.md: forced fall/animate_fall ($90:E37F) and suit_pickup_shot ($90:EF5E) are unused
  in retail.

Next gaps, in priority order:
- Wrecked Ship chozo hand (hands.rs setup). Not started by T3. T2's WIP scenario
  ($TEMP/collab/laneT2/wip-ws-hand.patch, room CAAE) leaves Samus stuck at 723,110. The Lower
  Norfair hand (B1E5) needs space jump owned.
- kihunter.drift: the wings' fall after a kihunter loses them. Releasing the controller in 184/205
  doesn't reach it; shoot one with a weak beam (a scenario that changes the beams).
- baby_metroid gradually_accelerate_towards_samus: releasing at DCB1 (320) doesn't reach it.
- The rest of dachora_run and etecoon (wall_jump, tunnel_jump, ...): longer releases from
  scenario-dachora:44 and 100-input-phase:43.
- Ask the root for a fresh coverage run after A-C land.

Deferred: no route room has the geometry. They are not proven unreachable, so they have no
README note.
- grapple spike_contact: spikes within reach of a hanging rope (64 px max plus 48 of body), none
  found. A293's ceiling spikes need a horizontal swing; pumping tops out near 0x2400 from vertical.
- position_anchored (C77E) and the crouching special angles: no grapple block at standing or
  crouching gun height is reachable.

Tools in $TEMP/collab/laneT3:
- tr2.sh / tr3.sh: recipe runs with a positions dump (mk/*.pos).
- sweep.sh: per-room probe sweep. `PROBES="a b" xargs -P 7 -L 1 bash sweep.sh < rooms.txt`.
- posdiag.py WORKTREE [revert]: adds pose, facing, grapple fn, HUD item, grapple angle, length and
  walljump timer to SM_VERIFY_POSITIONS. Revert before committing; it changes the DLL source stamp.
- seedsha.py: a seed's seed_sha256. mkrows.py / pend.py: write row blocks and add pending lines.
- Scans: gscan/wgscan/sscan2 (grapple and spikes), ascan (standing grapple), dbg3.py
  SEEDS_DIR ROOM (level map).

Traps:
- Native lib: /tmp/collab/native-laneT-3df45d743235/lib. Others give "stale native utilities".
- Any source change means a ~15 min rebuild. Don't edit sources while runs or the gate are going.
- Rebasing: `git diff > patch; git checkout -- .; rebase; git apply`. No stash. rebase --continue
  refused while unstaged files were present.
- lock.toml rebase conflicts: resolve as a sorted union of [[scenario]] blocks.
- Pack bundles before adding pending lines (pack skips pending rows):
  `ci_seed_data.py pack --runtime-only --segment ...`. Pick a unique first token for the bundle
  name, because plan_corpus shards by it.
- Enabling a row needs its lock entry in the same commit, recorded with
  `--mode lock --oracle faithful`; otherwise lock_gate fails.
- `--mode determinism` replays half the budget, so late require_hit probes fail it. Not a divergence.
- Grapple recipes: fire with a fresh X press after any turn. Holding Up while extending keeps the
  aim, but Up right after a connect reels in. The rope auto-reels to 8 after a connect; Down sets
  it to 63/64.
- Scenario x/y edits are ignored during elevator entry (seed 34). Use another visit (43).
- Seeds copied from laneD: low-input-phase:129 and 100-boundary:259.
