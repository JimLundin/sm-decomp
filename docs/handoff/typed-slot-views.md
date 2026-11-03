# Typed slot views handoff (2026-11-03)

Branch `collab/typed-all` (PR #155) converts the address-API layer to typed Session
fields. Engine address APIs (`ram_ratchet.py`, ROM record reads excluded) fall from
5805 on main `0115740f5` to 232.

## Pattern

`crates/sm_game/src/actors/slot_view.rs`: `slot_view!` declares a family's record
fields (`Actor` lanes) and extended words (`var_NN`). `Family::of(host, k)` reads;
`FamilyEdit::of(host, k)` stores. A dropped edit that borrowed the record runs the
Runtime's input refresh once, as the old per-byte projected stores did. Slot 0's
Botwoon death words are served from `Session::botwoon_death`. Non-slot words use
their Session fields, followed by `SessionAccess::words_stored()` where the old
store refreshed inputs. Families' `Lease` traits require `SlotState`, which only
`Runtime` implements.

Lease merges (write order changed): Mother Brain tube landing and unpause hook,
Draygon (all routines on the lease). Everything else is `[exact]`.

## Remaining 232 sites, by blocker

1. **No owned Session representation.**
   - HDMA wave/eye-beam/window tables at $9000-$9300 and $9800: `hdma_effects` 21, `beam_windows` 9, `xray_background`, `mother_brain_rainbow`.
   - OAM $370-$592: `atmosphere`, `player_drawing`.
   - Shooting-star records $0E0C-$108B over the actor slots: `shooting_star_motion` 19.
   - Work Robot palette cycle $D652-$D657: 13.
   - Pause/unpause hooks $0601-$0606, scroll bytes $CD20.., melt tiles $4000..: Kraid, Crocomire, statues.
   - VRAM write queue $00D0/$0100, drop origin $F434/$F436.
   - Each needs a Session field with encoding, checkpoint lane and layout test.
2. **Owned only under an owned presentation.**
   - Display mirrors $52/$59/$5D/$69, Mode 7 matrix, NMI counter, FX lanes $1984/$1986.
   - These are Phantoon's 15 and some of Kraid's, Crocomire's and Ceres doors'.
   - They need the presentation to be always owned, or owner-aware typed APIs for the legacy case.
3. **Still on `Game`.**
   - The game-over menu (menus host): 11 sites. A cohort patch exists at `$TEMP/typed-hosts/game_over_cohort.patch`, but it raises `sm_game_ram_sites` by 3.
   - Kraid's unpause coroutine (`kraid_pause`): 2 sites.
   - `gamet::settle`, `dachora::load_palette` and `yapping_maw::appearance` run on `BossState`/`BossLease`, which `Game` also implements.
   - `calculate_xray_window` runs inside `foreign`.
4. **Gateways.**
   - `boss_lease::boss_ram`: about 80 words with mixed owners.
   - `bg2_staging`: Crocomire's `melt_down` index can pass word 2047.
   - `session_slots.rs` and `projection.rs` are the machinery itself; they are deleted once 1-3 are gone.
5. **Address-indexed by design.** Out-of-range fallbacks (Phantoon), buggy original reads past tables (Crocomire, covern, rinka $87FE+id, draygon_sweep $9002+i, statues level block), and level-data indexes past the cells. A typed index panics where RAM didn't, so these need a LEGACY entry or a bounded typed fallback.

## Caveat

Palette writes index `colors[i]`/`targets[i]` directly (work_robot, covern, face
block, draygon). An out-of-range index would panic where the old code wrote the next
RAM block; real palette indexes are in range.
