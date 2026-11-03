# Typed slot views handoff (2026-11-03)

Branch `collab/typed-all` (PR #155) converts the address-API layer to typed Session
fields. Engine address APIs (`ram_ratchet.py`, ROM record reads excluded) fall from
5805 on main `0115740f5` to 91.

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
display mirrors and the NMI counter are plain RAM in replays.

A new Session owner must move every writer of its bytes onto it. Four owners have
landed this way:
- room scroll cells ($CD20..$CD51)
- pause hooks ($0601..$0606)
- right border ($1874)
- Work Robot palette ($D652..$D657)

The resulting checkpoint formats are SMROOMS 9 and SMACTORS 8. Replays can't check
$1874, because the verifier masks $186E..$1886.

The first scroll attempt diverged `boss-crocomire` at $CD24 (run 41561271426). Six
Crocomire `word_transfers!` stores still used the cell indexes as RAM addresses. Once
those stores were fixed, the four failing rows passed (run 41574730839).

Raw RAM accesses to the scroll cells still run on Game:
- `demo_rooms.rs:272`
- `hdma_program.rs:297-298`
- the boot WRAM clear

## Remaining 91 sites, by blocker

1. **New Session owners, plus all of their writers.**
   - (Drop origin $F434/$F436 is now owned by `enemy_projectiles.drop_origin` (SMESHOT 3). The $3000..$6FFF tile buffer is owned by the presentation workspace; Kraid's `load_bg2` reads it typed.)
2. **Legacy presentation.**
   - (OAM $0370..$0591 is now owned by `LegacyPresentation::objects`, decoded and encoded at every lease boundary; cannon and atmosphere tiles store through it.)
   - $57/$69/$5B6. (The Samus half requests $071D..$0722 are now owned by `LegacyPresentation::samus_requests`, decoded and encoded at every lease boundary.)
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
   - Original reads and stores that run past their tables: Crocomire's `erase_melt_columns` (offsets reach $8xxx/$Cxxx/$10xxx) and melt column heights, covern $0AF6+k, rinka $87FE+id, draygon_sweep $9002+i past $97DC, the statues level block.
   - Level-data indexes that run past the cells.
   - A typed index would panic where RAM didn't, so these need a LEGACY entry or a bounded typed fallback.
   - Done (2026-11-03): these go through `legacy_overflow` (`LEGACY(original-overflow)` in docs/LEGACY.md), typed where in range; cycle_report counts them as `legacy_overflow`.
5. **Machinery.** `session_slots.rs`, `projection.rs` and `ram.rs`. Delete them once 1-4 are gone.
6. **Counter noise.** `enemy_projectiles/content` (a `self` ROM record) and `menus/canvas` (a workspace `Point`) are not RAM.

## Caveat

Palette writes index `colors[i]`/`targets[i]` directly: work_robot, covern, face block
and draygon. An out-of-range index would panic where the old code wrote into the next
RAM block. Real palette indexes are always in range.
