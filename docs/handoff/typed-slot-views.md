# Typed slot views handoff (2026-11-03)

Branch `collab/typed-all` (PR #155) converts the address-API layer to typed Session
fields. Engine address APIs (`ram_ratchet.py`, ROM record reads excluded) fall from
5805 on main `0115740f5` to 36, and `collab/f36-owners` takes them to 27.

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

The resulting checkpoint formats are SMROOMS 9 and SMACTORS 9 (the ending's star records add four spare-word owners). Owners whose writers stay on Game, which stores RAM and is re-adopted at the next lease, are scalar words like their neighbours: the demo input frame count $0A8A and spare $0A5E (SMPLAYER 5), and the ending's wavy Samus words $0D9C..$0DA1 (SMAUXGS 14). Replays can't check
$1874, because the verifier masks $186E..$1886.

The first scroll attempt diverged `boss-crocomire` at $CD24 (run 41561271426). Six
Crocomire `word_transfers!` stores still used the cell indexes as RAM addresses. Once
those stores were fixed, the four failing rows passed (run 41574730839).

Raw RAM accesses to the scroll cells still run on Game (the demo's first
scene hook now stores `Room::set_scroll`):
- `hdma_program.rs:297-298`
- the boot WRAM clear

## Remaining sites, by blocker

1. **New Session owners, plus all of their writers.**
   - (Drop origin $F434/$F436 is now owned by `enemy_projectiles.drop_origin` (SMESHOT 3). The $3000..$6FFF tile buffer is owned by the presentation workspace; Kraid's `load_bg2` reads it typed.)
2. **Legacy presentation.**
   - (OAM $0370..$0591 is now owned by `LegacyPresentation::objects`, decoded and encoded at every lease boundary; cannon and atmosphere tiles store through it.)
   - $57/$69/$5B6. (The Samus half requests $071D..$0722 are now owned by `LegacyPresentation::samus_requests`, decoded and encoded at every lease boundary.)
   - Phantoon's `set_mosaic`/`enable_bg2`/`frame`.
   - Owning the presentation on the normal replay path is an architecture step.
3. **Still on `Game`.**
   - The game-over menu: 11 sites. A cohort patch exists at `$TEMP/typed-hosts/game_over_cohort.patch` but raises `sm_game_ram_sites` by 3.
   - `beam_windows`/`calculate_xray_window`, which runs inside `foreign`.
   - (The HUD's digit pointer $00..$02 stores `Session::pointer_scratch`; the options screens at $7F:C000.. are read as `QueuedTilemaps` background blocks.)
   - The game-over `prepare` display mirrors: $53/$54/$86 have no display `Field`. Adding them changes `preparation::State`, `DisplayControls`, the Owned presentation's projection and checkpoint, and Game's frame-display ranges.
   - The demo's setup words $09E4/$09E6/$09E8/$0DF8/$0DFC: $09E4 and $0DF8 are input observations that `inputs_from_session` reads from RAM, and save loads and new games write them on Game.
   - The X-ray/suit window table at $9800 (`xray::set_window_word`): `fx_domain::Sources` owns it only while the Session has FX sources or an Owned presentation, and its store panics where the RAM store succeeds.
   - Game-side routines that touch enemy 0 (the game-over baby, demo rooms, Kraid's unpause) read and store a typed `Actor` through `ActorImage`, which `Game` implements over RAM in sm/.
4. **Address-indexed by design.**
   - Out-of-range fallbacks: Phantoon's part and shot fallbacks, `visual_host` handles, `bg2_staging::set_unbounded` for Crocomire's melt cursor.
   - Original reads and stores that run past their tables: Crocomire's `erase_melt_columns` (offsets reach $8xxx/$Cxxx/$10xxx) and melt column heights, covern $0AF6+k, rinka $87FE+id, draygon_sweep $9002+i past $97DC, the statues level block.
   - Level-data indexes that run past the cells.
   - A typed index would panic where RAM didn't, so these need a LEGACY entry or a bounded typed fallback.
   - Done (2026-11-03): these go through `legacy_overflow` (`LEGACY(original-overflow)` in docs/LEGACY.md), typed where in range; cycle_report counts them as `legacy_overflow`.
5. **Machinery.** `session_slots.rs` is gone (`session_slot!`, `SlotAccess`, `SessionSlots`); overflow paths resolve through `legacy_overflow::SlotBytes`. The views' byte helpers moved to `sm/src/projection.rs`: sm_game's views now require `projection::Views` (a supertrait of `CompatibilityRam`), which names words by `Field` and groups by `Layout`/`Decode` (cohorts, liquid, palette fade, camera, flow, room). `sm` implements it for `Game` over its projected bytes, and `observed_inputs` is a required method there. On a current lease the Runtime reads and stores the Session fields directly (`Field::session_word`/`session_word_mut`, `Decode::from_session`, `Layout::session_copy`/`store_session`; proved equal to the projected bytes by sm_game's projection tests); a dirty lease, the room header/door exit/FX blending stores, table elements past their tables and cohort stores (which run only on `Game`, through `session_cohort::store`'s lease) keep the byte path. `Field::address`, `Layout::visit` and `Decode::decode` stay in sm_game: `Game`'s generic `Views` impl, the Runtime's byte fallback, the verify build's held-copy checks and those tests use them. The `word!` macros still reach `compatibility_byte` (they count as RAM macros).
6. **Counter noise.** `enemy_projectiles/content` (a `self` ROM record) and `menus/canvas` (a workspace `Point`) are not RAM.

## Caveat

Palette writes index `colors[i]`/`targets[i]` directly: work_robot, covern, face block
and draygon. An out-of-range index would panic where the old code wrote into the next
RAM block. Real palette indexes are always in range.

## 2026-11-03 batch (PR after #164)

New owners: Player `moonwalk`/`debug_flag`/`spare_09e8`/`turn_request`/`spare_0dfc` ($09E4..$09E8,
$0DF8, $0DFC; SMPLAYER 6, input arms for $09E4/$0DF8) and display Fields `ObjectAddressLow/High`,
`MemorySelect` ($53/$54/$86; SMDISPL 5). The X-ray window, Mother Brain's rainbow, the corpse rot
tables and the cinematic $7F tilemap transfers read and store through their owners; out-of-range
remnants go through `legacy_overflow` (60 sites). What remains of the 14: `legacy_overflow.rs`
internals (11), `projection.rs`/`ram.rs` (the `CompatibilityRam` byte helpers Game serves from its
RAM image). No engine call site outside these three files names a RAM address.
