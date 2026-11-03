# Typed slot views handoff (2026-11-03)

Branch `collab/typed-all` (PR #155) converts the address-API layer to typed Session
fields. Engine address APIs (`ram_ratchet.py`, ROM record reads excluded) fall from
5805 on main `0115740f5` to 170.

## Pattern

`crates/sm_game/src/actors/slot_view.rs`:
- `slot_view!` declares a family's record fields (`Actor` lanes) and its extended words (`var_NN`).
- `Family::of(host, k)` reads and `FamilyEdit::of(host, k)` stores.
- A dropped edit that borrowed the record runs the Runtime's input refresh once, as the old per-byte projected stores did.
- Slot 0's Botwoon death words are served from `Session::botwoon_death`.
- Non-slot words use their Session fields. Where the old store refreshed inputs, the new one calls `SessionAccess::words_stored()` after it.
- Families' `Lease` traits require `SlotState`, which only `Runtime` implements.
- The `boss_ram` gateway is gone; callers use `boss_interaction`'s word map.

Commits that change write order:
- Lease merges: the Mother Brain tube landing and unpause hook; Draygon, with all its routines now on the lease.
- Record byte order: the Ceres elevator overlay.

Everything else is `[exact]`.

## Replays decide

Replays always run with Legacy presentation. `Presentation::Owned` comes only from a
supplied or restored session start, which only verification cases reach. So OAM, the
display mirrors, the NMI counter and the Samus-half pointers are plain RAM in replays.

A new Session owner must also move every other writer of its bytes onto it. Owning
the room scroll cells ($CD20..) made `boss-crocomire` diverge at $CD24 and
`file-copy-clear` diverge at tick 837 (run 41561271426), because other writers still
reach those bytes through RAM. The scroll, pause hook ($0601..$0606), right border
($1874) and Work Robot palette ($D652..$D657) owners were reverted for this reason.
They return once their writers move too; the sm/src room and menu code is among them.

## Remaining 170 sites, by blocker

1. **New Session owners, plus all of their writers.**
   - Scroll cells, pause hooks, right border and Work Robot palette (see above).
   - Drop origin $F434/$F436: `sm/src/game/scroll_runtime.rs:32` refreshes inputs from raw RAM while the Session is current.
   - Melt tiles at $4000..
2. **Legacy presentation.**
   - OAM: `atmosphere`, `player_drawing`.
   - $57/$69/$5B6, $71D..$722.
   - Phantoon's `set_mosaic`/`enable_bg2`/`frame`.
   - Owning the presentation on the normal replay path is an architecture step.
3. **Still on `Game`.**
   - The game-over menu: 11 sites. A cohort patch exists at `$TEMP/typed-hosts/game_over_cohort.patch` but raises `sm_game_ram_sites` by 3.
   - Shooting stars (ending, menus host): 19 sites.
   - `beam_windows`/`calculate_xray_window`, which runs inside `foreign`.
   - Kraid's unpause coroutine.
   - `yapping_maw::appearance`: the verifier's sprite alias cases 117/118 run it on Game at off-table enemy 0xE0DA.
   - `ridley` 1 and `demo_rooms`/`hud`: hosts that `Game` also implements.
4. **Address-indexed by design.**
   - Out-of-range fallbacks: Phantoon's part and shot fallbacks, `visual_host` handles, `bg2_staging::set_unbounded` for Crocomire's melt cursor.
   - Original reads that run past their tables: Crocomire melt columns, covern $0AF6+k, rinka $87FE+id, draygon_sweep $9002+i past $97DC, the statues level block.
   - Level-data indexes that run past the cells.
   - A typed index would panic where RAM didn't, so these need a LEGACY entry or a bounded typed fallback.
5. **Machinery.** `session_slots.rs`, `projection.rs` and `ram.rs`. Delete them once 1-4 are gone.
6. **Counter noise.** `enemy_projectiles/content` (a `self` ROM record) and `menus/canvas` (a workspace `Point`) are not RAM.

## Caveat

Palette writes index `colors[i]`/`targets[i]` directly: work_robot, covern, face block
and draygon. An out-of-range index would panic where the old code wrote into the next
RAM block. Real palette indexes are always in range.
